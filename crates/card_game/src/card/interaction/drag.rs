use bevy_ecs::prelude::{Query, Res, ResMut};
use engine_core::prelude::{EventBus, Transform2D};
use engine_input::prelude::{MouseButton, MouseState};
use engine_physics::prelude::{PhysicsCommand, PhysicsRes};
use glam::Vec2;

use crate::card::interaction::drag_state::DragState;

pub const DRAG_GAIN: f32 = 20.0;
pub const MAX_ANGULAR_VELOCITY: f32 = 15.0;

pub fn card_drag_system(
    mouse: Res<MouseState>,
    drag_state: Res<DragState>,
    physics: Res<PhysicsRes>,
    mut physics_commands: ResMut<EventBus<PhysicsCommand>>,
    mut transforms: Query<&mut Transform2D>,
) {
    let Some(info) = &drag_state.dragging else {
        return;
    };
    if !mouse.pressed(MouseButton::Left) {
        return;
    }

    if info.stash_cursor_follow {
        if let Ok(mut transform) = transforms.get_mut(info.entity) {
            transform.position = mouse.world_pos();
            transform.rotation = 0.0;
        }
        return;
    }

    let Some(grab_world) = physics.body_point_to_world(info.entity, info.local_grab_offset) else {
        return;
    };
    let Some(body_pos) = physics.body_position(info.entity) else {
        return;
    };
    let cursor = mouse.world_pos();
    let desired = DRAG_GAIN * (cursor - grab_world);
    let arm = grab_world - body_pos;
    let arm_len_sq = arm.length_squared();

    if arm_len_sq < 1e-4 {
        physics_commands.push(PhysicsCommand::SetLinearVelocity {
            entity: info.entity,
            velocity: desired,
        });
    } else {
        let raw_omega = arm.perp_dot(desired) / arm_len_sq;
        let omega = raw_omega.clamp(-MAX_ANGULAR_VELOCITY, MAX_ANGULAR_VELOCITY);
        let perp_arm = Vec2::new(-arm.y, arm.x);
        let v_center = desired - omega * perp_arm;
        physics_commands.push(PhysicsCommand::SetLinearVelocity {
            entity: info.entity,
            velocity: v_center,
        });
        physics_commands.push(PhysicsCommand::SetAngularVelocity {
            entity: info.entity,
            angular_velocity: omega,
        });
    }
}
