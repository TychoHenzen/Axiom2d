use bevy_ecs::prelude::{Res, ResMut};
use engine_core::prelude::DeltaTime;

use crate::collision_event::CollisionEventBuffer;
use crate::physics_res::PhysicsRes;

pub fn physics_step_system(
    dt: Res<DeltaTime>,
    mut physics: ResMut<PhysicsRes>,
    mut events: ResMut<CollisionEventBuffer>,
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

    use bevy_ecs::prelude::{Entity, Schedule, World};
    use engine_core::prelude::{DeltaTime, Seconds};
    use glam::Vec2;

    use crate::collider::Collider;
    use crate::collision_event::{CollisionEvent, CollisionKind};
    use crate::physics_backend::PhysicsBackend;
    use crate::physics_res::PhysicsRes;
    use crate::rigid_body::RigidBody;
    use crate::test_helpers::spawn_entities;

    use super::*;

    struct SpyPhysicsBackend {
        step_count: Arc<AtomicU32>,
        events: Vec<CollisionEvent>,
    }

    impl SpyPhysicsBackend {
        fn new(step_count: Arc<AtomicU32>) -> Self {
            Self {
                step_count,
                events: Vec::new(),
            }
        }

        fn with_events(mut self, events: Vec<CollisionEvent>) -> Self {
            self.events = events;
            self
        }
    }

    impl PhysicsBackend for SpyPhysicsBackend {
        fn step(&mut self, _dt: Seconds) {
            self.step_count.fetch_add(1, Ordering::Relaxed);
        }

        fn add_body(&mut self, _: Entity, _: &RigidBody, _: Vec2) -> bool {
            false
        }

        fn add_collider(&mut self, _: Entity, _: &Collider) -> bool {
            false
        }

        fn remove_body(&mut self, _: Entity) {}

        fn body_position(&self, _: Entity) -> Option<Vec2> {
            None
        }

        fn body_rotation(&self, _: Entity) -> Option<f32> {
            None
        }

        fn drain_collision_events(&mut self) -> Vec<CollisionEvent> {
            std::mem::take(&mut self.events)
        }

        fn body_linear_velocity(&self, _: Entity) -> Option<Vec2> {
            None
        }
        fn set_linear_velocity(&mut self, _: Entity, _: Vec2) {}
        fn set_angular_velocity(&mut self, _: Entity, _: f32) {}

        fn add_force_at_point(&mut self, _: Entity, _: Vec2, _: Vec2) {}

        fn set_damping(&mut self, _: Entity, _: f32, _: f32) {}
    }

    fn setup_world(step_count: Arc<AtomicU32>) -> World {
        let mut world = World::new();
        world.insert_resource(PhysicsRes::new(Box::new(SpyPhysicsBackend::new(
            Arc::clone(&step_count),
        ))));
        world.insert_resource(CollisionEventBuffer::default());
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
        let mut buffer = world.resource_mut::<CollisionEventBuffer>();
        let events: Vec<_> = buffer.drain().collect();
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
            SpyPhysicsBackend::new(Arc::clone(&step_count)).with_events(vec![event]),
        )));
        world.insert_resource(CollisionEventBuffer::default());
        world.insert_resource(DeltaTime(Seconds(0.016)));
        let mut schedule = Schedule::default();
        schedule.add_systems(physics_step_system);

        // Act
        schedule.run(&mut world);

        // Assert
        let mut buffer = world.resource_mut::<CollisionEventBuffer>();
        let events: Vec<_> = buffer.drain().collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], event);
    }
}
