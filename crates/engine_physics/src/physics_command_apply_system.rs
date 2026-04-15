use bevy_ecs::prelude::ResMut;
use engine_core::prelude::EventBus;

use crate::physics_command::PhysicsCommand;
use crate::physics_res::PhysicsRes;

pub fn physics_command_apply_system(
    mut commands: ResMut<EventBus<PhysicsCommand>>,
    mut physics: ResMut<PhysicsRes>,
) {
    for command in commands.drain() {
        match command {
            PhysicsCommand::AddBody {
                entity,
                body_type,
                position,
            } => {
                physics.add_body(entity, &body_type, position);
            }
            PhysicsCommand::AddCollider { entity, collider } => {
                physics.add_collider(entity, &collider);
            }
            PhysicsCommand::RemoveBody { entity } => {
                let _ = physics.remove_body(entity);
            }
            PhysicsCommand::SetLinearVelocity { entity, velocity } => {
                let _ = physics.set_linear_velocity(entity, velocity);
            }
            PhysicsCommand::SetAngularVelocity {
                entity,
                angular_velocity,
            } => {
                let _ = physics.set_angular_velocity(entity, angular_velocity);
            }
            PhysicsCommand::SetDamping {
                entity,
                linear,
                angular,
            } => {
                let _ = physics.set_damping(entity, linear, angular);
            }
            PhysicsCommand::SetCollisionGroup {
                entity,
                membership,
                filter,
            } => {
                let _ = physics.set_collision_group(entity, membership, filter);
            }
            PhysicsCommand::SetBodyPosition { entity, position } => {
                let _ = physics.set_body_position(entity, position);
            }
            PhysicsCommand::AddForceAtPoint {
                entity,
                force,
                world_point,
            } => {
                let _ = physics.add_force_at_point(entity, force, world_point);
            }
        }
    }
}
