use bevy_ecs::prelude::{Entity, Query, Res, ResMut, With};
use engine_core::prelude::EventBus;
use engine_physics::prelude::{PhysicsCommand, PhysicsRes};

use crate::card::component::Card;
use crate::card::component::CardZone;

pub const LINEAR_SLEEP_THRESHOLD: f32 = 2.0;
pub const ANGULAR_SLEEP_THRESHOLD: f32 = 0.5;

pub fn card_sleep_system(
    query: Query<(Entity, &CardZone), With<Card>>,
    physics: Res<PhysicsRes>,
    mut physics_commands: ResMut<EventBus<PhysicsCommand>>,
) {
    for (entity, zone) in &query {
        if !matches!(zone, CardZone::Table) {
            continue;
        }
        let Some(lin_vel) = physics.body_linear_velocity(entity) else {
            continue;
        };
        let Some(ang_vel) = physics.body_angular_velocity(entity) else {
            continue;
        };
        if lin_vel.length() > LINEAR_SLEEP_THRESHOLD {
            continue;
        }
        if ang_vel.abs() > ANGULAR_SLEEP_THRESHOLD {
            continue;
        }
        if physics.is_body_sleeping(entity) == Some(true) {
            continue;
        }
        physics_commands.push(PhysicsCommand::SleepBody { entity });
    }
}
