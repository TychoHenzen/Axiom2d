use bevy_ecs::prelude::{Entity, Query, ResMut, With};
use engine_physics::prelude::PhysicsRes;

use crate::card::Card;
use crate::card_zone::CardZone;

pub const BASE_LINEAR_DRAG: f32 = 8.0;
pub const BASE_ANGULAR_DRAG: f32 = 5.0;
pub const SPIN_DRAG_DECAY_RATE: f32 = 0.15;
pub const MIN_DRAG_FACTOR: f32 = 0.25;

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
        physics.set_damping(entity, linear, angular);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::{Arc, Mutex};

    use bevy_ecs::prelude::*;
    use engine_core::prelude::TextureId;
    use engine_physics::prelude::PhysicsRes;

    use super::*;
    use crate::test_helpers::{DampingLog, SpyPhysicsBackend};

    #[test]
    fn when_zero_angular_velocity_then_base_drag_returned() {
        // Arrange / Act
        let (linear, angular) = compute_card_damping(0.0);

        // Assert
        assert!(
            (linear - BASE_LINEAR_DRAG).abs() < 1e-6,
            "expected linear={BASE_LINEAR_DRAG}, got {linear}"
        );
        assert!(
            (angular - BASE_ANGULAR_DRAG).abs() < 1e-6,
            "expected angular={BASE_ANGULAR_DRAG}, got {angular}"
        );
    }

    #[test]
    fn when_high_angular_velocity_then_drag_less_than_base() {
        // Arrange / Act
        let (linear, angular) = compute_card_damping(20.0);

        // Assert
        assert!(
            linear < BASE_LINEAR_DRAG,
            "expected linear < {BASE_LINEAR_DRAG}, got {linear}"
        );
        assert!(
            angular < BASE_ANGULAR_DRAG,
            "expected angular < {BASE_ANGULAR_DRAG}, got {angular}"
        );
    }

    #[test]
    fn when_negative_angular_velocity_then_same_as_positive() {
        // Arrange / Act
        let positive = compute_card_damping(5.0);
        let negative = compute_card_damping(-5.0);

        // Assert
        assert!(
            (positive.0 - negative.0).abs() < 1e-6,
            "linear: positive={}, negative={}",
            positive.0,
            negative.0
        );
        assert!(
            (positive.1 - negative.1).abs() < 1e-6,
            "angular: positive={}, negative={}",
            positive.1,
            negative.1
        );
    }

    #[test]
    fn when_extreme_angular_velocity_then_drag_floored_at_minimum() {
        // Arrange / Act
        let (linear, angular) = compute_card_damping(1000.0);

        // Assert
        let min_linear = BASE_LINEAR_DRAG * MIN_DRAG_FACTOR;
        let min_angular = BASE_ANGULAR_DRAG * MIN_DRAG_FACTOR;
        assert!(
            (linear - min_linear).abs() < 1e-6,
            "expected linear={min_linear}, got {linear}"
        );
        assert!(
            (angular - min_angular).abs() < 1e-6,
            "expected angular={min_angular}, got {angular}"
        );
    }

    #[test]
    fn when_increasing_angular_velocity_then_drag_monotonically_decreases() {
        // Arrange
        let omegas = [0.0, 1.0, 5.0, 10.0, 20.0];

        // Act
        let drags: Vec<(f32, f32)> = omegas.iter().map(|&w| compute_card_damping(w)).collect();

        // Assert
        for i in 1..drags.len() {
            assert!(
                drags[i].0 <= drags[i - 1].0,
                "linear drag should decrease: omega={} gave {}, omega={} gave {}",
                omegas[i - 1],
                drags[i - 1].0,
                omegas[i],
                drags[i].0
            );
        }
    }

    fn run_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(card_damping_system);
        schedule.run(world);
    }

    #[test]
    fn when_non_card_entity_then_set_damping_not_called() {
        // Arrange
        let damping_log: DampingLog = Arc::new(Mutex::new(Vec::new()));
        let mut world = World::new();
        world.insert_resource(PhysicsRes::new(Box::new(
            SpyPhysicsBackend::new().with_damping_log(damping_log.clone()),
        )));
        world.spawn_empty(); // entity with no Card component

        // Act
        run_system(&mut world);

        // Assert
        assert!(damping_log.lock().unwrap().is_empty());
    }

    #[test]
    fn when_card_on_table_with_zero_spin_then_base_damping_applied() {
        // Arrange
        let damping_log: DampingLog = Arc::new(Mutex::new(Vec::new()));
        let mut world = World::new();
        let entity = world
            .spawn((Card::face_down(TextureId(0), TextureId(0)), CardZone::Table))
            .id();
        let spy = SpyPhysicsBackend::new()
            .with_damping_log(damping_log.clone())
            .with_angular_velocity(entity, 0.0);
        world.insert_resource(PhysicsRes::new(Box::new(spy)));

        // Act
        run_system(&mut world);

        // Assert
        let calls = damping_log.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, entity);
        assert!((calls[0].1 - BASE_LINEAR_DRAG).abs() < 1e-4);
        assert!((calls[0].2 - BASE_ANGULAR_DRAG).abs() < 1e-4);
    }

    #[test]
    fn when_card_on_table_with_high_spin_then_reduced_damping_applied() {
        // Arrange
        let damping_log: DampingLog = Arc::new(Mutex::new(Vec::new()));
        let mut world = World::new();
        let entity = world
            .spawn((Card::face_down(TextureId(0), TextureId(0)), CardZone::Table))
            .id();
        let spy = SpyPhysicsBackend::new()
            .with_damping_log(damping_log.clone())
            .with_angular_velocity(entity, 20.0);
        world.insert_resource(PhysicsRes::new(Box::new(spy)));

        // Act
        run_system(&mut world);

        // Assert
        let calls = damping_log.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert!(
            calls[0].1 < BASE_LINEAR_DRAG,
            "expected reduced linear drag"
        );
        assert!(
            calls[0].2 < BASE_ANGULAR_DRAG,
            "expected reduced angular drag"
        );
    }

    #[test]
    fn when_card_with_no_physics_body_then_no_panic_and_no_damping() {
        // Arrange
        let damping_log: DampingLog = Arc::new(Mutex::new(Vec::new()));
        let mut world = World::new();
        world.spawn((Card::face_down(TextureId(0), TextureId(0)), CardZone::Table));
        // Spy has no angular velocity entry → body_angular_velocity returns None
        world.insert_resource(PhysicsRes::new(Box::new(
            SpyPhysicsBackend::new().with_damping_log(damping_log.clone()),
        )));

        // Act
        run_system(&mut world);

        // Assert
        assert!(damping_log.lock().unwrap().is_empty());
    }

    #[test]
    fn when_multiple_cards_on_table_then_set_damping_called_for_each() {
        // Arrange
        let damping_log: DampingLog = Arc::new(Mutex::new(Vec::new()));
        let mut world = World::new();
        let e1 = world
            .spawn((Card::face_down(TextureId(0), TextureId(0)), CardZone::Table))
            .id();
        let e2 = world
            .spawn((Card::face_down(TextureId(0), TextureId(0)), CardZone::Table))
            .id();
        let e3 = world
            .spawn((Card::face_down(TextureId(0), TextureId(0)), CardZone::Table))
            .id();
        let spy = SpyPhysicsBackend::new()
            .with_damping_log(damping_log.clone())
            .with_angular_velocity(e1, 0.0)
            .with_angular_velocity(e2, 5.0)
            .with_angular_velocity(e3, 20.0);
        world.insert_resource(PhysicsRes::new(Box::new(spy)));

        // Act
        run_system(&mut world);

        // Assert
        let calls = damping_log.lock().unwrap();
        assert_eq!(calls.len(), 3);
        let entities: Vec<Entity> = calls.iter().map(|c| c.0).collect();
        assert!(entities.contains(&e1));
        assert!(entities.contains(&e2));
        assert!(entities.contains(&e3));
    }

    #[test]
    fn when_card_in_hand_then_set_damping_not_called() {
        // Arrange
        let damping_log: DampingLog = Arc::new(Mutex::new(Vec::new()));
        let mut world = World::new();
        let table_entity = world
            .spawn((Card::face_down(TextureId(0), TextureId(0)), CardZone::Table))
            .id();
        world.spawn((
            Card::face_down(TextureId(0), TextureId(0)),
            CardZone::Hand(0),
        ));
        let spy = SpyPhysicsBackend::new()
            .with_damping_log(damping_log.clone())
            .with_angular_velocity(table_entity, 0.0);
        world.insert_resource(PhysicsRes::new(Box::new(spy)));

        // Act
        run_system(&mut world);

        // Assert
        let calls = damping_log.lock().unwrap();
        assert_eq!(calls.len(), 1, "only the Table card should get damping");
        assert_eq!(calls[0].0, table_entity);
    }
}
