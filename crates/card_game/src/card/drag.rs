use bevy_ecs::prelude::{Query, Res, ResMut};
use engine_core::prelude::Transform2D;
use engine_input::prelude::{MouseButton, MouseState};
use engine_physics::prelude::PhysicsRes;
use glam::Vec2;

use crate::drag_state::DragState;

pub const DRAG_GAIN: f32 = 20.0;
pub const MAX_ANGULAR_VELOCITY: f32 = 15.0;

pub fn card_drag_system(
    mouse: Res<MouseState>,
    drag_state: Res<DragState>,
    mut physics: ResMut<PhysicsRes>,
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
        physics.set_linear_velocity(info.entity, desired);
    } else {
        let raw_omega = arm.perp_dot(desired) / arm_len_sq;
        let omega = raw_omega.clamp(-MAX_ANGULAR_VELOCITY, MAX_ANGULAR_VELOCITY);
        let perp_arm = Vec2::new(-arm.y, arm.x);
        let v_center = desired - omega * perp_arm;
        physics.set_linear_velocity(info.entity, v_center);
        physics.set_angular_velocity(info.entity, omega);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::too_many_arguments)]
mod tests {
    use std::sync::{Arc, Mutex};

    use bevy_ecs::prelude::*;
    use engine_input::prelude::{MouseButton, MouseState};
    use engine_physics::prelude::PhysicsRes;
    use glam::Vec2;

    use super::{DRAG_GAIN, MAX_ANGULAR_VELOCITY, card_drag_system};
    use crate::card::zone::CardZone;
    use crate::drag_state::{DragInfo, DragState};
    use crate::test_helpers::{AngularVelocityLog, SpyPhysicsBackend, VelocityLog};

    fn run_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(card_drag_system);
        schedule.run(world);
    }

    fn setup_drag_world(
        entity: Entity,
        local_grab_offset: Vec2,
        body_pos: Vec2,
        body_rot: f32,
        cursor_pos: Vec2,
        mouse_pressed: bool,
    ) -> (World, VelocityLog, AngularVelocityLog) {
        let mut world = World::new();
        let velocity_log: VelocityLog = Arc::new(Mutex::new(Vec::new()));
        let angular_velocity_log: AngularVelocityLog = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyPhysicsBackend::new()
            .with_velocity_log(velocity_log.clone())
            .with_angular_velocity_log(angular_velocity_log.clone())
            .with_body(entity, body_pos, body_rot);
        world.insert_resource(PhysicsRes::new(Box::new(spy)));

        let mut mouse = MouseState::default();
        if mouse_pressed {
            mouse.press(MouseButton::Left);
        }
        mouse.set_world_pos(cursor_pos);
        world.insert_resource(mouse);

        world.insert_resource(DragState {
            dragging: Some(DragInfo {
                entity,
                local_grab_offset,
                origin_zone: CardZone::Table,
                stash_cursor_follow: false,
            }),
        });

        (world, velocity_log, angular_velocity_log)
    }

    use engine_core::prelude::Transform2D;

    use crate::test_helpers::spawn_entity;

    #[test]
    fn when_stash_cursor_follow_then_position_set_to_world_pos_and_no_physics_velocity() {
        // Arrange
        let vel_log: VelocityLog = Arc::new(Mutex::new(Vec::new()));
        let ang_log: AngularVelocityLog = Arc::new(Mutex::new(Vec::new()));
        let mut world = World::new();
        let entity = world
            .spawn(Transform2D {
                position: Vec2::ZERO,
                rotation: 0.0,
                scale: Vec2::ONE,
            })
            .id();
        let spy = SpyPhysicsBackend::new()
            .with_velocity_log(vel_log.clone())
            .with_angular_velocity_log(ang_log.clone())
            .with_body(entity, Vec2::ZERO, 0.0);
        world.insert_resource(PhysicsRes::new(Box::new(spy)));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::new(120.0, 80.0));
        world.insert_resource(mouse);
        world.insert_resource(DragState {
            dragging: Some(DragInfo {
                entity,
                local_grab_offset: Vec2::ZERO,
                origin_zone: CardZone::Table,
                stash_cursor_follow: true,
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        let pos = world.entity(entity).get::<Transform2D>().unwrap().position;
        assert_eq!(pos, Vec2::new(120.0, 80.0));
        assert!(vel_log.lock().unwrap().is_empty());
        assert!(ang_log.lock().unwrap().is_empty());
    }

    #[test]
    fn when_dragging_at_center_then_velocity_points_toward_cursor() {
        // Arrange
        let entity = spawn_entity();
        let (mut world, vel_log, _) = setup_drag_world(
            entity,
            Vec2::ZERO,
            Vec2::ZERO,
            0.0,
            Vec2::new(10.0, 0.0),
            true,
        );

        // Act
        run_system(&mut world);

        // Assert
        let calls = vel_log.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert!(
            calls[0].1.x > 0.0,
            "velocity should point toward cursor (+X)"
        );
    }

    #[test]
    fn when_cursor_equals_grab_point_then_zero_velocity() {
        // Arrange
        let entity = spawn_entity();
        let (mut world, vel_log, _) = setup_drag_world(
            entity,
            Vec2::ZERO,
            Vec2::new(5.0, 5.0),
            0.0,
            Vec2::new(5.0, 5.0),
            true,
        );

        // Act
        run_system(&mut world);

        // Assert
        let calls = vel_log.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert!(
            calls[0].1.length() < 1e-6,
            "velocity should be ~zero, got {}",
            calls[0].1
        );
    }

    #[test]
    fn when_dragging_at_center_and_cursor_twice_as_far_then_velocity_doubles() {
        // Arrange
        let entity_a = spawn_entity();
        let (mut world_a, log_a, _) = setup_drag_world(
            entity_a,
            Vec2::ZERO,
            Vec2::ZERO,
            0.0,
            Vec2::new(10.0, 0.0),
            true,
        );
        run_system(&mut world_a);

        let entity_b = spawn_entity();
        let (mut world_b, log_b, _) = setup_drag_world(
            entity_b,
            Vec2::ZERO,
            Vec2::ZERO,
            0.0,
            Vec2::new(20.0, 0.0),
            true,
        );
        run_system(&mut world_b);

        // Assert
        let calls_a = log_a.lock().unwrap();
        let calls_b = log_b.lock().unwrap();
        assert_eq!(calls_a.len(), 1);
        assert_eq!(calls_b.len(), 1);
        let ratio = calls_b[0].1.length() / calls_a[0].1.length();
        assert!(
            (ratio - 2.0).abs() < 1e-4,
            "velocity ratio should be 2.0, got {ratio}"
        );
    }

    #[test]
    fn when_dragging_at_center_then_no_angular_velocity() {
        // Arrange — grabbed at center, arm is zero → pure translation
        let entity = spawn_entity();
        let (mut world, _, ang_log) = setup_drag_world(
            entity,
            Vec2::ZERO,
            Vec2::ZERO,
            0.0,
            Vec2::new(10.0, 0.0),
            true,
        );

        // Act
        run_system(&mut world);

        // Assert
        let calls = ang_log.lock().unwrap();
        assert!(
            calls.is_empty(),
            "center grab should not set angular velocity"
        );
    }

    #[test]
    fn when_dragging_off_center_perpendicular_then_pure_rotation() {
        // Arrange — body at origin, grabbed at (10,0), cursor moves straight up to (10,5)
        // grab_world = (10, 0), arm = (10, 0)
        // desired = GAIN * (0, 5), which is perpendicular to arm → pure rotation
        let entity = spawn_entity();
        let (mut world, vel_log, ang_log) = setup_drag_world(
            entity,
            Vec2::new(10.0, 0.0),
            Vec2::ZERO,
            0.0,
            Vec2::new(10.0, 5.0),
            true,
        );

        // Act
        run_system(&mut world);

        // Assert — center velocity should be ~zero, angular velocity non-zero
        let v_calls = vel_log.lock().unwrap();
        let a_calls = ang_log.lock().unwrap();
        assert_eq!(v_calls.len(), 1);
        assert_eq!(a_calls.len(), 1);
        assert!(
            v_calls[0].1.length() < 1e-4,
            "perpendicular pull should produce ~zero center velocity, got {}",
            v_calls[0].1
        );
        assert!(
            a_calls[0].1.abs() > 1.0,
            "perpendicular pull should produce angular velocity, got {}",
            a_calls[0].1
        );
    }

    #[test]
    fn when_dragging_off_center_along_arm_then_pure_translation() {
        // Arrange — body at origin, grabbed at (10,0), cursor at (15,0)
        // displacement is along arm → pure translation, no rotation
        let entity = spawn_entity();
        let (mut world, vel_log, ang_log) = setup_drag_world(
            entity,
            Vec2::new(10.0, 0.0),
            Vec2::ZERO,
            0.0,
            Vec2::new(15.0, 0.0),
            true,
        );

        // Act
        run_system(&mut world);

        // Assert
        let v_calls = vel_log.lock().unwrap();
        let a_calls = ang_log.lock().unwrap();
        assert_eq!(v_calls.len(), 1);
        assert_eq!(a_calls.len(), 1);
        assert!(v_calls[0].1.x > 0.0, "should translate toward cursor");
        assert!(
            a_calls[0].1.abs() < 1e-6,
            "along-arm pull should produce no angular velocity, got {}",
            a_calls[0].1
        );
    }

    #[test]
    fn when_dragging_off_center_then_grab_point_velocity_matches_desired() {
        // Arrange — the rigid body constraint: v_grab = v_center + ω × arm
        // This must equal DRAG_GAIN * displacement for any rotation/offset
        let entity = spawn_entity();
        let rotation = std::f32::consts::FRAC_PI_4;
        let local_offset = Vec2::new(10.0, 0.0);
        let cursor = Vec2::new(15.0, 5.0);
        let (mut world, vel_log, ang_log) =
            setup_drag_world(entity, local_offset, Vec2::ZERO, rotation, cursor, true);

        // Act
        run_system(&mut world);

        // Assert — reconstruct grab point velocity and compare to desired
        let v_center = vel_log.lock().unwrap()[0].1;
        let omega = ang_log.lock().unwrap()[0].1;
        let (sin, cos) = rotation.sin_cos();
        let arm = Vec2::new(
            local_offset.x * cos - local_offset.y * sin,
            local_offset.x * sin + local_offset.y * cos,
        );
        let grab_world = arm; // body at origin
        let perp_arm = Vec2::new(-arm.y, arm.x);
        let v_grab = v_center + omega * perp_arm;
        let desired = DRAG_GAIN * (cursor - grab_world);
        assert!(
            (v_grab - desired).length() < 1e-2,
            "grab point velocity {v_grab} should equal desired {desired}"
        );
    }

    #[test]
    fn when_dragging_off_center_with_nonzero_body_pos_then_arm_computed_correctly() {
        // Arrange — body at (10, 0), grabbed at local (5, 0), cursor at (20, 0)
        // grab_world = (15, 0), arm = grab_world - body_pos = (5, 0)
        // desired = GAIN * ((20,0) - (15,0)) = GAIN * (5, 0)
        // With correct arm (5,0): displacement along arm → pure translation, no rotation
        // With mutated arm (grab_world + body_pos = (25,0)): completely wrong physics
        let entity = spawn_entity();
        let body_pos = Vec2::new(10.0, 0.0);
        let local_offset = Vec2::new(5.0, 0.0);
        let cursor = Vec2::new(20.0, 0.0);
        let (mut world, vel_log, ang_log) =
            setup_drag_world(entity, local_offset, body_pos, 0.0, cursor, true);

        // Act
        run_system(&mut world);

        // Assert — displacement along arm should yield ~zero angular velocity
        let a_calls = ang_log.lock().unwrap();
        assert_eq!(a_calls.len(), 1);
        assert!(
            a_calls[0].1.abs() < 1e-4,
            "along-arm pull with nonzero body_pos should give ~zero rotation, got {}",
            a_calls[0].1
        );

        // Assert — velocity should point in +X direction (toward cursor)
        let v_calls = vel_log.lock().unwrap();
        assert_eq!(v_calls.len(), 1);
        assert!(
            v_calls[0].1.x > 0.0,
            "velocity should point toward cursor (+X), got {}",
            v_calls[0].1.x
        );
    }

    #[test]
    fn when_arm_length_just_above_threshold_then_rotation_path_used() {
        // Arrange — arm_len_sq needs to be >= 1e-4 (arm length >= 0.01)
        // body at origin, grabbed at (0.02, 0), cursor pulled perpendicular at (0.02, 1.0)
        // arm = (0.02, 0), arm_len_sq = 0.0004 > 1e-4 → rotation path
        let entity = spawn_entity();
        let (mut world, _vel_log, ang_log) = setup_drag_world(
            entity,
            Vec2::new(0.02, 0.0),
            Vec2::ZERO,
            0.0,
            Vec2::new(0.02, 1.0),
            true,
        );

        // Act
        run_system(&mut world);

        // Assert — rotation path must be taken (angular velocity set)
        let a_calls = ang_log.lock().unwrap();
        assert_eq!(
            a_calls.len(),
            1,
            "rotation path should set angular velocity"
        );
        assert!(
            a_calls[0].1.abs() > 0.1,
            "angular velocity should be nonzero, got {}",
            a_calls[0].1
        );
    }

    #[test]
    fn when_not_dragging_then_no_velocity_set() {
        // Arrange
        let mut world = World::new();
        let vel_log: VelocityLog = Arc::new(Mutex::new(Vec::new()));
        let entity = spawn_entity();
        let spy = SpyPhysicsBackend::new()
            .with_velocity_log(vel_log.clone())
            .with_body(entity, Vec2::ZERO, 0.0);
        world.insert_resource(PhysicsRes::new(Box::new(spy)));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::new(10.0, 0.0));
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());

        // Act
        run_system(&mut world);

        // Assert
        let calls = vel_log.lock().unwrap();
        assert!(calls.is_empty());
    }

    #[test]
    fn when_dragging_but_mouse_not_pressed_then_no_velocity_set() {
        // Arrange
        let entity = spawn_entity();
        let (mut world, vel_log, _) = setup_drag_world(
            entity,
            Vec2::ZERO,
            Vec2::ZERO,
            0.0,
            Vec2::new(10.0, 0.0),
            false,
        );

        // Act
        run_system(&mut world);

        // Assert
        let calls = vel_log.lock().unwrap();
        assert!(calls.is_empty());
    }

    #[test]
    fn when_body_not_at_origin_then_arm_direction_is_grab_minus_body_pos() {
        // Arrange — body at (0,5), offset (3,0), cursor one unit above grab point (3,6)
        // grab_world = (3,5), correct arm = (3,5)-(0,5) = (3,0)
        // desired = GAIN*(0,1) is purely perpendicular to arm → pure rotation, v_center ≈ zero
        // mutant arm = (3,5)+(0,5) = (3,10): omega ≈ 0.55, v_center ≈ (5.5, 18.35)
        let entity = spawn_entity();
        let (mut world, vel_log, ang_log) = setup_drag_world(
            entity,
            Vec2::new(3.0, 0.0),
            Vec2::new(0.0, 5.0),
            0.0,
            Vec2::new(3.0, 6.0),
            true,
        );

        // Act
        run_system(&mut world);

        // Assert — perpendicular pull should produce ~zero center velocity
        let v_calls = vel_log.lock().unwrap();
        let a_calls = ang_log.lock().unwrap();
        assert_eq!(v_calls.len(), 1);
        assert_eq!(a_calls.len(), 1);
        assert!(
            v_calls[0].1.length() < 1.0,
            "perpendicular pull on correct arm should yield v_center ≈ zero, got {}",
            v_calls[0].1
        );
    }

    #[test]
    fn when_arm_len_sq_exactly_at_threshold_then_rotation_path_used() {
        // Arrange — arm_len_sq == 1e-4 exactly (arm = (0.01, 0))
        // `<` is false at exactly 1e-4 → rotation path → angular velocity IS set
        // `<=` would be true → center path → angular velocity NOT set
        let entity = spawn_entity();
        let (mut world, _vel_log, ang_log) = setup_drag_world(
            entity,
            Vec2::new(0.01, 0.0),
            Vec2::ZERO,
            0.0,
            Vec2::new(0.01, 1.0),
            true,
        );

        // Act
        run_system(&mut world);

        // Assert — rotation path must be taken (angular velocity set and nonzero)
        let a_calls = ang_log.lock().unwrap();
        assert_eq!(
            a_calls.len(),
            1,
            "rotation path should set angular velocity"
        );
        assert!(
            a_calls[0].1.abs() > 0.0,
            "angular velocity should be nonzero at exact threshold, got {}",
            a_calls[0].1
        );
    }

    #[test]
    fn when_edge_grab_produces_extreme_spin_then_angular_velocity_clamped() {
        // Arrange — body at origin, grabbed at (5,0), cursor at (5, 500)
        // raw_omega = arm.perp_dot(desired) / arm_len_sq would be huge
        let entity = spawn_entity();
        let (mut world, _vel_log, ang_log) = setup_drag_world(
            entity,
            Vec2::new(5.0, 0.0),
            Vec2::ZERO,
            0.0,
            Vec2::new(5.0, 500.0),
            true,
        );

        // Act
        run_system(&mut world);

        // Assert
        let a_calls = ang_log.lock().unwrap();
        assert_eq!(a_calls.len(), 1);
        assert!(
            a_calls[0].1.abs() <= MAX_ANGULAR_VELOCITY + 1e-6,
            "angular velocity should be clamped to ±{MAX_ANGULAR_VELOCITY}, got {}",
            a_calls[0].1
        );
    }
}
