use bevy_ecs::prelude::{Component, Entity, World};
use engine_input::prelude::{InputState, KeyCode};
use engine_physics::prelude::PhysicsRes;

use crate::card::component::CardZone;

#[derive(Component)]
pub struct DebugSleepTint;

pub fn debug_sleep_indicator_system(world: &mut World) {
    let active = world
        .get_resource::<InputState>()
        .is_some_and(|input| input.just_pressed(KeyCode::F9));

    if !active {
        return;
    }

    let mut table_cards: Vec<(Entity, bool)> = Vec::new();
    let mut query = world.query::<(Entity, &CardZone, Option<&DebugSleepTint>)>();
    for (entity, zone, tint) in query.iter(world) {
        if matches!(zone, CardZone::Table) {
            table_cards.push((entity, tint.is_some()));
        }
    }

    let mut to_add = Vec::new();
    let mut to_remove = Vec::new();

    let physics = world.resource::<PhysicsRes>();
    for (entity, has_tint) in table_cards {
        let sleeping = physics.is_body_sleeping(entity) == Some(true);
        if sleeping && !has_tint {
            to_add.push(entity);
        } else if !sleeping && has_tint {
            to_remove.push(entity);
        }
    }

    for entity in to_add {
        world.entity_mut(entity).insert(DebugSleepTint);
    }
    for entity in to_remove {
        world.entity_mut(entity).remove::<DebugSleepTint>();
    }
}
