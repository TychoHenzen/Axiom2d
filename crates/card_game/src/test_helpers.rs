use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use bevy_ecs::prelude::{Entity, World};
use engine_core::prelude::Seconds;
use engine_physics::prelude::{Collider, CollisionEvent, PhysicsBackend, RigidBody};
use glam::Vec2;

pub(crate) fn spawn_entity() -> Entity {
    World::new().spawn_empty().id()
}

pub(crate) type AddBodyLog = Arc<Mutex<Vec<(Entity, Vec2)>>>;
pub(crate) type ColliderLog = Arc<Mutex<Vec<Entity>>>;
pub(crate) type RemoveBodyLog = Arc<Mutex<Vec<Entity>>>;
pub(crate) type DampingLog = Arc<Mutex<Vec<(Entity, f32, f32)>>>;
pub(crate) type VelocityLog = Arc<Mutex<Vec<(Entity, Vec2)>>>;
pub(crate) type AngularVelocityLog = Arc<Mutex<Vec<(Entity, f32)>>>;
pub(crate) type CollisionGroupLog = Arc<Mutex<Vec<(Entity, u32, u32)>>>;

/// Configurable spy for `PhysicsBackend` used across all card_game tests.
///
/// Pre-configure return data via builder methods (`with_body`, `with_angular_velocity`).
/// Capture calls via `Arc<Mutex<Vec<_>>>` fields passed in via builder methods.
pub(crate) struct SpyPhysicsBackend {
    pub positions: HashMap<Entity, Vec2>,
    pub rotations: HashMap<Entity, f32>,
    pub angular_velocities: HashMap<Entity, f32>,
    pub add_body_log: AddBodyLog,
    pub collider_log: ColliderLog,
    pub remove_body_log: RemoveBodyLog,
    pub damping_log: DampingLog,
    pub velocity_log: VelocityLog,
    pub angular_velocity_log: AngularVelocityLog,
    pub collision_group_log: CollisionGroupLog,
}

impl SpyPhysicsBackend {
    pub fn new() -> Self {
        Self {
            positions: HashMap::new(),
            rotations: HashMap::new(),
            angular_velocities: HashMap::new(),
            add_body_log: Arc::new(Mutex::new(Vec::new())),
            collider_log: Arc::new(Mutex::new(Vec::new())),
            remove_body_log: Arc::new(Mutex::new(Vec::new())),
            damping_log: Arc::new(Mutex::new(Vec::new())),
            velocity_log: Arc::new(Mutex::new(Vec::new())),
            angular_velocity_log: Arc::new(Mutex::new(Vec::new())),
            collision_group_log: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn with_body(mut self, entity: Entity, position: Vec2, rotation: f32) -> Self {
        self.positions.insert(entity, position);
        self.rotations.insert(entity, rotation);
        self
    }

    pub fn with_angular_velocity(mut self, entity: Entity, omega: f32) -> Self {
        self.angular_velocities.insert(entity, omega);
        self
    }

    pub fn with_add_body_log(mut self, log: AddBodyLog) -> Self {
        self.add_body_log = log;
        self
    }

    pub fn with_collider_log(mut self, log: ColliderLog) -> Self {
        self.collider_log = log;
        self
    }

    pub fn with_remove_body_log(mut self, log: RemoveBodyLog) -> Self {
        self.remove_body_log = log;
        self
    }

    pub fn with_damping_log(mut self, log: DampingLog) -> Self {
        self.damping_log = log;
        self
    }

    pub fn with_velocity_log(mut self, log: VelocityLog) -> Self {
        self.velocity_log = log;
        self
    }

    pub fn with_angular_velocity_log(mut self, log: AngularVelocityLog) -> Self {
        self.angular_velocity_log = log;
        self
    }

    #[allow(dead_code)]
    pub fn with_collision_group_log(mut self, log: CollisionGroupLog) -> Self {
        self.collision_group_log = log;
        self
    }
}

#[allow(clippy::unwrap_used)]
impl PhysicsBackend for SpyPhysicsBackend {
    fn step(&mut self, _dt: Seconds) {}
    fn add_body(&mut self, entity: Entity, _body_type: &RigidBody, position: Vec2) -> bool {
        self.add_body_log.lock().unwrap().push((entity, position));
        true
    }
    fn add_collider(&mut self, entity: Entity, _collider: &Collider) -> bool {
        self.collider_log.lock().unwrap().push(entity);
        true
    }
    fn remove_body(&mut self, entity: Entity) {
        self.remove_body_log.lock().unwrap().push(entity);
    }
    fn body_position(&self, entity: Entity) -> Option<Vec2> {
        self.positions.get(&entity).copied()
    }
    fn body_rotation(&self, entity: Entity) -> Option<f32> {
        self.rotations.get(&entity).copied()
    }
    fn drain_collision_events(&mut self) -> Vec<CollisionEvent> {
        Vec::new()
    }
    fn body_linear_velocity(&self, _: Entity) -> Option<Vec2> {
        Some(Vec2::ZERO)
    }
    fn set_linear_velocity(&mut self, entity: Entity, velocity: Vec2) {
        self.velocity_log.lock().unwrap().push((entity, velocity));
    }
    fn set_angular_velocity(&mut self, entity: Entity, angular_velocity: f32) {
        self.angular_velocity_log
            .lock()
            .unwrap()
            .push((entity, angular_velocity));
    }
    fn add_force_at_point(&mut self, _: Entity, _: Vec2, _: Vec2) {}
    fn body_angular_velocity(&self, entity: Entity) -> Option<f32> {
        self.angular_velocities.get(&entity).copied()
    }
    fn set_damping(&mut self, entity: Entity, linear: f32, angular: f32) {
        self.damping_log
            .lock()
            .unwrap()
            .push((entity, linear, angular));
    }
    fn set_collision_group(&mut self, entity: Entity, membership: u32, filter: u32) {
        self.collision_group_log
            .lock()
            .unwrap()
            .push((entity, membership, filter));
    }
}
