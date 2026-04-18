pub mod collider;
pub mod collision_event;
pub mod hit_test;
pub mod physics_backend;
pub mod physics_command;
pub mod physics_command_apply_system;
pub mod physics_res;
pub mod physics_step_system;
pub mod physics_sync_system;
pub mod prelude;
pub mod rapier_backend;
pub mod rigid_body;

#[doc(hidden)]
pub mod test_helpers {
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::{Arc, Mutex};

    use bevy_ecs::prelude::{Entity, World};
    use engine_core::prelude::Seconds;
    use glam::Vec2;

    use crate::collider::Collider;
    use crate::collision_event::CollisionEvent;
    use crate::physics_backend::{PhysicsBackend, PhysicsError};
    use crate::rigid_body::RigidBody;

    pub fn spawn_entity() -> Entity {
        World::new().spawn(()).id()
    }

    pub fn spawn_entities(count: usize) -> Vec<Entity> {
        let mut world = World::new();
        (0..count).map(|_| world.spawn(()).id()).collect()
    }

    /// Configurable spy for `PhysicsBackend` used across all `engine_physics` tests.
    ///
    /// Tracks calls via `Arc<Mutex<..>>` / `Arc<AtomicU32>` fields and returns
    /// pre-configured data for position/rotation/events queries.
    pub struct SpyPhysicsBackend {
        pub positions: HashMap<Entity, Vec2>,
        pub rotations: HashMap<Entity, f32>,
        pub step_count: Arc<AtomicU32>,
        pub events: Vec<CollisionEvent>,
    }

    impl Default for SpyPhysicsBackend {
        fn default() -> Self {
            Self::new()
        }
    }

    impl SpyPhysicsBackend {
        pub fn new() -> Self {
            Self {
                positions: HashMap::new(),
                rotations: HashMap::new(),
                step_count: Arc::new(AtomicU32::new(0)),
                events: Vec::new(),
            }
        }

        pub fn with_step_count(mut self, step_count: Arc<AtomicU32>) -> Self {
            self.step_count = step_count;
            self
        }

        pub fn with_body(mut self, entity: Entity, position: Vec2, rotation: f32) -> Self {
            self.positions.insert(entity, position);
            self.rotations.insert(entity, rotation);
            self
        }

        pub fn with_position(mut self, entity: Entity, pos: Vec2) -> Self {
            self.positions.insert(entity, pos);
            self
        }

        pub fn with_rotation(mut self, entity: Entity, rot: f32) -> Self {
            self.rotations.insert(entity, rot);
            self
        }

        pub fn with_events(mut self, events: Vec<CollisionEvent>) -> Self {
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
        fn remove_body(&mut self, _: Entity) -> Result<(), PhysicsError> {
            Ok(())
        }
        fn body_position(&self, entity: Entity) -> Option<Vec2> {
            self.positions.get(&entity).copied()
        }
        fn body_rotation(&self, entity: Entity) -> Option<f32> {
            self.rotations.get(&entity).copied()
        }
        fn drain_collision_events(&mut self) -> Vec<CollisionEvent> {
            std::mem::take(&mut self.events)
        }
        fn body_linear_velocity(&self, _: Entity) -> Option<Vec2> {
            Some(Vec2::ZERO)
        }
        fn set_linear_velocity(&mut self, _: Entity, _: Vec2) -> Result<(), PhysicsError> {
            Ok(())
        }
        fn set_angular_velocity(&mut self, _: Entity, _: f32) -> Result<(), PhysicsError> {
            Ok(())
        }
        fn add_force_at_point(&mut self, _: Entity, _: Vec2, _: Vec2) -> Result<(), PhysicsError> {
            Ok(())
        }
        fn body_angular_velocity(&self, _: Entity) -> Option<f32> {
            None
        }
        fn set_damping(&mut self, _: Entity, _: f32, _: f32) -> Result<(), PhysicsError> {
            Ok(())
        }
        fn set_collision_group(&mut self, _: Entity, _: u32, _: u32) -> Result<(), PhysicsError> {
            Ok(())
        }
        fn set_body_position(&mut self, _: Entity, _: Vec2) -> Result<(), PhysicsError> {
            Ok(())
        }
        fn is_body_sleeping(&self, _: Entity) -> Option<bool> {
            None
        }
        fn sleep_body(&mut self, _: Entity) -> Result<(), PhysicsError> {
            Ok(())
        }
        fn wake_body(&mut self, _: Entity) -> Result<(), PhysicsError> {
            Ok(())
        }
    }

    pub struct RecordingPhysicsBackend {
        pub calls: Arc<Mutex<Vec<String>>>,
    }

    impl RecordingPhysicsBackend {
        pub fn new(calls: Arc<Mutex<Vec<String>>>) -> Self {
            Self { calls }
        }
    }

    impl PhysicsBackend for RecordingPhysicsBackend {
        fn step(&mut self, _dt: Seconds) {}
        fn add_body(&mut self, _: Entity, _: &RigidBody, _: Vec2) -> bool {
            self.calls
                .lock()
                .expect("lock poisoned")
                .push("add_body".into());
            true
        }
        fn add_collider(&mut self, _: Entity, _: &Collider) -> bool {
            self.calls
                .lock()
                .expect("lock poisoned")
                .push("add_collider".into());
            true
        }
        fn remove_body(&mut self, _: Entity) -> Result<(), PhysicsError> {
            self.calls
                .lock()
                .expect("lock poisoned")
                .push("remove_body".into());
            Ok(())
        }
        fn set_linear_velocity(&mut self, _: Entity, _: Vec2) -> Result<(), PhysicsError> {
            self.calls
                .lock()
                .expect("lock poisoned")
                .push("set_linear_velocity".into());
            Ok(())
        }
        fn set_angular_velocity(&mut self, _: Entity, _: f32) -> Result<(), PhysicsError> {
            self.calls
                .lock()
                .expect("lock poisoned")
                .push("set_angular_velocity".into());
            Ok(())
        }
        fn set_damping(&mut self, _: Entity, _: f32, _: f32) -> Result<(), PhysicsError> {
            self.calls
                .lock()
                .expect("lock poisoned")
                .push("set_damping".into());
            Ok(())
        }
        fn set_collision_group(&mut self, _: Entity, _: u32, _: u32) -> Result<(), PhysicsError> {
            self.calls
                .lock()
                .expect("lock poisoned")
                .push("set_collision_group".into());
            Ok(())
        }
        fn set_body_position(&mut self, _: Entity, _: Vec2) -> Result<(), PhysicsError> {
            self.calls
                .lock()
                .expect("lock poisoned")
                .push("set_body_position".into());
            Ok(())
        }
        fn add_force_at_point(&mut self, _: Entity, _: Vec2, _: Vec2) -> Result<(), PhysicsError> {
            self.calls
                .lock()
                .expect("lock poisoned")
                .push("add_force_at_point".into());
            Ok(())
        }
        fn body_position(&self, _: Entity) -> Option<Vec2> {
            None
        }
        fn body_rotation(&self, _: Entity) -> Option<f32> {
            None
        }
        fn drain_collision_events(&mut self) -> Vec<CollisionEvent> {
            vec![]
        }
        fn body_linear_velocity(&self, _: Entity) -> Option<Vec2> {
            None
        }
        fn body_angular_velocity(&self, _: Entity) -> Option<f32> {
            None
        }
        fn is_body_sleeping(&self, _: Entity) -> Option<bool> {
            self.calls
                .lock()
                .expect("lock poisoned")
                .push("is_body_sleeping".into());
            None
        }
        fn sleep_body(&mut self, _: Entity) -> Result<(), PhysicsError> {
            self.calls
                .lock()
                .expect("lock poisoned")
                .push("sleep_body".into());
            Ok(())
        }
        fn wake_body(&mut self, _: Entity) -> Result<(), PhysicsError> {
            self.calls
                .lock()
                .expect("lock poisoned")
                .push("wake_body".into());
            Ok(())
        }
    }
}
