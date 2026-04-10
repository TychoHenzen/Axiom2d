// EVOLVE-BLOCK-START
use bevy_ecs::prelude::{Entity, Query, Res, With};
use engine_core::prelude::Transform2D;

use crate::physics_res::PhysicsRes;
use crate::rigid_body::RigidBody;

pub fn physics_sync_system(
    physics: Res<PhysicsRes>,
    mut query: Query<(Entity, &mut Transform2D), With<RigidBody>>,
) {
    for (entity, mut transform) in &mut query {
        if let Some(pos) = physics.body_position(entity) {
            transform.position = pos;
        }
        if let Some(rot) = physics.body_rotation(entity) {
            transform.rotation = rot;
        }
    }
}
// EVOLVE-BLOCK-END
