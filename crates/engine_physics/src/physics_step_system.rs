use bevy_ecs::prelude::{Res, ResMut};
use engine_core::prelude::{DeltaTime, EventBus};

use crate::collision_event::CollisionEvent;
use crate::physics_res::PhysicsRes;

pub fn physics_step_system(
    dt: Res<DeltaTime>,
    mut physics: ResMut<PhysicsRes>,
    mut events: ResMut<EventBus<CollisionEvent>>,
) {
    physics.step(dt.0);
    for event in physics.drain_collision_events() {
        events.push(event);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    use bevy_ecs::prelude::{Schedule, World};
    use engine_core::prelude::{DeltaTime, Seconds};

    use crate::collision_event::{CollisionEvent, CollisionKind};
    use crate::physics_res::PhysicsRes;
    use crate::test_helpers::{SpyPhysicsBackend, spawn_entities};

    use super::*;

    fn setup_world(step_count: Arc<AtomicU32>) -> World {
        let mut world = World::new();
        world.insert_resource(PhysicsRes::new(Box::new(
            SpyPhysicsBackend::new().with_step_count(Arc::clone(&step_count)),
        )));
        world.insert_resource(EventBus::<CollisionEvent>::default());
        world.insert_resource(DeltaTime(Seconds(0.016)));
        world
    }

    #[test]
    fn when_system_runs_then_backend_is_stepped() {
        // Arrange
        let step_count = Arc::new(AtomicU32::new(0));
        let mut world = setup_world(Arc::clone(&step_count));
        let mut schedule = Schedule::default();
        schedule.add_systems(physics_step_system);

        // Act
        schedule.run(&mut world);

        // Assert
        assert_eq!(step_count.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn when_system_runs_with_no_events_then_buffer_remains_empty() {
        // Arrange
        let step_count = Arc::new(AtomicU32::new(0));
        let mut world = setup_world(Arc::clone(&step_count));
        let mut schedule = Schedule::default();
        schedule.add_systems(physics_step_system);

        // Act
        schedule.run(&mut world);

        // Assert
        let mut bus = world.resource_mut::<EventBus<CollisionEvent>>();
        let events: Vec<_> = bus.drain().collect();
        assert!(events.is_empty());
    }

    #[test]
    fn when_system_runs_twice_then_backend_stepped_twice() {
        // Arrange
        let step_count = Arc::new(AtomicU32::new(0));
        let mut world = setup_world(Arc::clone(&step_count));
        let mut schedule = Schedule::default();
        schedule.add_systems(physics_step_system);

        // Act
        schedule.run(&mut world);
        schedule.run(&mut world);

        // Assert
        assert_eq!(step_count.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn when_backend_produces_events_then_buffer_contains_them() {
        // Arrange
        let step_count = Arc::new(AtomicU32::new(0));
        let entities = spawn_entities(2);
        let event = CollisionEvent {
            entity_a: entities[0],
            entity_b: entities[1],
            kind: CollisionKind::Started,
        };
        let mut world = World::new();
        world.insert_resource(PhysicsRes::new(Box::new(
            SpyPhysicsBackend::new()
                .with_step_count(Arc::clone(&step_count))
                .with_events(vec![event]),
        )));
        world.insert_resource(EventBus::<CollisionEvent>::default());
        world.insert_resource(DeltaTime(Seconds(0.016)));
        let mut schedule = Schedule::default();
        schedule.add_systems(physics_step_system);

        // Act
        schedule.run(&mut world);

        // Assert
        let mut bus = world.resource_mut::<EventBus<CollisionEvent>>();
        let events: Vec<_> = bus.drain().collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], event);
    }
}
