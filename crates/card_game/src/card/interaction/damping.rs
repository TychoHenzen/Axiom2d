use bevy_ecs::prelude::{Entity, Query, ResMut, With};
use engine_physics::prelude::PhysicsRes;

use crate::card::component::Card;
use crate::card::component::CardZone;

pub const BASE_LINEAR_DRAG: f32 = 8.0;
pub const BASE_ANGULAR_DRAG: f32 = 5.0;
pub(crate) const SPIN_DRAG_DECAY_RATE: f32 = 0.15;
pub(crate) const MIN_DRAG_FACTOR: f32 = 0.25;

#[must_use]
pub fn compute_card_damping(angular_velocity: f32) -> (f32, f32) {
    let factor = (-SPIN_DRAG_DECAY_RATE * angular_velocity.abs())
        .exp()
        .max(MIN_DRAG_FACTOR);
    (BASE_LINEAR_DRAG * factor, BASE_ANGULAR_DRAG * factor)
}

pub fn card_damping_system(
    query: Query<(Entity, &CardZone), With<Card>>,
    mut physics: ResMut<PhysicsRes>,
) {
    for (entity, zone) in &query {
        if !matches!(zone, CardZone::Table) {
            continue;
        }
        let Some(omega) = physics.body_angular_velocity(entity) else {
            continue;
        };
        let (linear, angular) = compute_card_damping(omega);
        physics
            .set_damping(entity, linear, angular)
            .expect("damped entity should have physics body");
    }
}
