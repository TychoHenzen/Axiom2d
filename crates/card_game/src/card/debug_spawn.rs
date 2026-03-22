use bevy_ecs::prelude::{Resource, World};
use engine_input::prelude::{InputState, KeyCode};
use engine_physics::prelude::PhysicsRes;
use glam::Vec2;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::card::definition::{CardAbilities, CardDefinition, CardType, art_descriptor_default};
use crate::card::geometry::TABLE_CARD_SIZE;
use crate::card::pick::{CARD_COLLISION_FILTER, CARD_COLLISION_GROUP};
use crate::card::signature::CardSignature;
use crate::card::spawn_table_card::spawn_visual_card;

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
        let _ = physics.set_collision_group(entity, CARD_COLLISION_GROUP, CARD_COLLISION_FILTER);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::card::base_type::{BaseCardTypeRegistry, populate_default_types};
    use crate::card::component::Card;
    use crate::test_helpers::SpyPhysicsBackend;
    use bevy_ecs::prelude::World;
    use engine_input::prelude::InputState;
    use engine_physics::prelude::PhysicsRes;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(InputState::default());
        world.insert_resource(DebugSpawnRng::default());
        world.insert_resource(PhysicsRes::new(Box::new(SpyPhysicsBackend::new())));
        let mut registry = BaseCardTypeRegistry::default();
        populate_default_types(&mut registry);
        world.insert_resource(registry);
        world
    }

    #[test]
    fn when_key1_pressed_then_one_card_spawned() {
        // Arrange
        let mut world = setup_world();
        world.resource_mut::<InputState>().press(KeyCode::Digit1);

        // Act
        debug_spawn_system(&mut world);

        // Assert
        let card_count = world.query::<&Card>().iter(&world).count();
        assert_eq!(card_count, 1);
    }

    #[test]
    fn when_key2_pressed_then_ten_cards_spawned() {
        // Arrange
        let mut world = setup_world();
        world.resource_mut::<InputState>().press(KeyCode::Digit2);

        // Act
        debug_spawn_system(&mut world);

        // Assert
        let card_count = world.query::<&Card>().iter(&world).count();
        assert_eq!(card_count, 10);
    }

    #[test]
    fn when_key3_pressed_then_hundred_cards_spawned() {
        // Arrange
        let mut world = setup_world();
        world.resource_mut::<InputState>().press(KeyCode::Digit3);

        // Act
        debug_spawn_system(&mut world);

        // Assert
        let card_count = world.query::<&Card>().iter(&world).count();
        assert_eq!(card_count, 100);
    }

    #[test]
    fn when_no_key_pressed_then_no_cards_spawned() {
        // Arrange
        let mut world = setup_world();

        // Act
        debug_spawn_system(&mut world);

        // Assert
        let card_count = world.query::<&Card>().iter(&world).count();
        assert_eq!(card_count, 0);
    }

    #[test]
    fn when_cards_spawned_then_each_has_unique_signature() {
        // Arrange
        let mut world = setup_world();
        world.resource_mut::<InputState>().press(KeyCode::Digit2);

        // Act
        debug_spawn_system(&mut world);

        // Assert
        let signatures: Vec<_> = world
            .query::<&Card>()
            .iter(&world)
            .map(|c| c.signature)
            .collect();
        for i in 0..signatures.len() {
            for j in (i + 1)..signatures.len() {
                assert_ne!(
                    signatures[i], signatures[j],
                    "cards {i} and {j} should have different signatures"
                );
            }
        }
    }
}
