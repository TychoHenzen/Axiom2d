use bevy_ecs::prelude::{Resource, World};
use engine_input::prelude::{InputState, KeyCode};
use engine_physics::prelude::PhysicsRes;
use glam::Vec2;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::card::identity::definition::{
    CardAbilities, CardDefinition, CardType, art_descriptor_default,
};
use crate::card::identity::signature::CardSignature;
use crate::card::interaction::pick::{CARD_COLLISION_FILTER, CARD_COLLISION_GROUP};
use crate::card::rendering::geometry::TABLE_CARD_SIZE;
use crate::card::rendering::spawn_table_card::spawn_visual_card;

#[derive(Resource)]
pub struct DebugSpawnRng(pub ChaCha8Rng);

impl Default for DebugSpawnRng {
    fn default() -> Self {
        Self(ChaCha8Rng::seed_from_u64(0xDEAD))
    }
}

pub fn debug_spawn_system(world: &mut World) {
    let count = {
        let Some(input) = world.get_resource::<InputState>() else {
            return;
        };
        if input.just_pressed(KeyCode::Digit1) {
            1
        } else if input.just_pressed(KeyCode::Digit2) {
            10
        } else if input.just_pressed(KeyCode::Digit3) {
            100
        } else {
            return;
        }
    };

    let placeholder_def = CardDefinition {
        card_type: CardType::Creature,
        name: String::new(),
        stats: None,
        abilities: CardAbilities {
            keywords: vec![],
            text: String::new(),
        },
        art: art_descriptor_default(CardType::Creature),
    };

    let mut entities = Vec::with_capacity(count);
    for _ in 0..count {
        let signature = {
            let rng = &mut world.resource_mut::<DebugSpawnRng>().0;
            CardSignature::random(rng)
        };
        let position = {
            use rand::Rng;
            let rng = &mut world.resource_mut::<DebugSpawnRng>().0;
            Vec2::new(rng.gen_range(-300.0..300.0), rng.gen_range(-200.0..200.0))
        };

        let entity = spawn_visual_card(
            world,
            &placeholder_def,
            position,
            TABLE_CARD_SIZE,
            true,
            signature,
        );
        entities.push(entity);
    }

    let mut physics = world.resource_mut::<PhysicsRes>();
    for entity in entities {
        physics
            .set_collision_group(entity, CARD_COLLISION_GROUP, CARD_COLLISION_FILTER)
            .expect("set_collision_group: body must exist for freshly spawned card");
    }
}
