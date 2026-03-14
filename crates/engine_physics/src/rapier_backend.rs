use std::collections::HashMap;

use bevy_ecs::prelude::Entity;
use crossbeam::channel::Receiver;
use engine_core::prelude::Seconds;
use glam::Vec2;
use rapier2d::prelude::*;

use crate::collider::Collider;
use crate::collision_event::{CollisionEvent, CollisionKind};
use crate::physics_backend::PhysicsBackend;
use crate::rigid_body::RigidBody;

pub struct RapierBackend {
    gravity: Vec2,
    pipeline: PhysicsPipeline,
    integration_parameters: IntegrationParameters,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    bodies: RigidBodySet,
    colliders: ColliderSet,
    impulse_joints: ImpulseJointSet,
    multibody_joints: MultibodyJointSet,
    ccd_solver: CCDSolver,
    entity_to_handle: HashMap<Entity, RigidBodyHandle>,
    collider_to_entity: HashMap<ColliderHandle, Entity>,
    event_collector: ChannelEventCollector,
    collision_recv: Receiver<rapier2d::geometry::CollisionEvent>,
}

impl RapierBackend {
    #[must_use]
    pub fn new(gravity: Vec2) -> Self {
        let (collision_send, collision_recv) = crossbeam::channel::unbounded();
        let (contact_force_send, _contact_force_recv) = crossbeam::channel::unbounded();
        let event_collector = ChannelEventCollector::new(collision_send, contact_force_send);
        Self {
            gravity,
            pipeline: PhysicsPipeline::new(),
            integration_parameters: IntegrationParameters::default(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            impulse_joints: ImpulseJointSet::new(),
            multibody_joints: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            entity_to_handle: HashMap::new(),
            collider_to_entity: HashMap::new(),
            event_collector,
            collision_recv,
        }
    }
}

impl PhysicsBackend for RapierBackend {
    fn step(&mut self, dt: Seconds) {
        self.integration_parameters.dt = dt.0;
        self.pipeline.step(
            &vector![self.gravity.x, self.gravity.y],
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            &mut self.ccd_solver,
            None,
            &(),
            &self.event_collector,
        );
    }

    fn add_body(&mut self, entity: Entity, body_type: &RigidBody, position: Vec2) -> bool {
        if self.entity_to_handle.contains_key(&entity) {
            return false;
        }
        let rb = match body_type {
            RigidBody::Dynamic => RigidBodyBuilder::dynamic(),
            RigidBody::Static => RigidBodyBuilder::fixed(),
            RigidBody::Kinematic => RigidBodyBuilder::kinematic_position_based(),
        }
        .translation(vector![position.x, position.y])
        .build();
        let handle = self.bodies.insert(rb);
        self.entity_to_handle.insert(entity, handle);
        true
    }

    fn add_collider(&mut self, entity: Entity, collider: &Collider) -> bool {
        let Some(&body_handle) = self.entity_to_handle.get(&entity) else {
            return false;
        };
        let col = match collider {
            Collider::Circle(radius) => ColliderBuilder::ball(*radius),
            Collider::Aabb(half_extents) => ColliderBuilder::cuboid(half_extents.x, half_extents.y),
            Collider::ConvexPolygon(points) => {
                let rapier_points: Vec<_> = points.iter().map(|p| point![p.x, p.y]).collect();
                match ColliderBuilder::convex_hull(&rapier_points) {
                    Some(builder) => builder,
                    None => return false,
                }
            }
        }
        .active_events(ActiveEvents::COLLISION_EVENTS);
        let collider_handle =
            self.colliders
                .insert_with_parent(col.build(), body_handle, &mut self.bodies);
        self.collider_to_entity.insert(collider_handle, entity);
        true
    }

    fn remove_body(&mut self, entity: Entity) {
        if let Some(handle) = self.entity_to_handle.remove(&entity) {
            self.bodies.remove(
                handle,
                &mut self.island_manager,
                &mut self.colliders,
                &mut self.impulse_joints,
                &mut self.multibody_joints,
                true,
            );
        }
    }

    fn body_position(&self, entity: Entity) -> Option<Vec2> {
        let handle = self.entity_to_handle.get(&entity)?;
        let body = self.bodies.get(*handle)?;
        let pos = body.translation();
        Some(Vec2::new(pos.x, pos.y))
    }

    fn body_rotation(&self, entity: Entity) -> Option<f32> {
        let handle = self.entity_to_handle.get(&entity)?;
        let body = self.bodies.get(*handle)?;
        Some(body.rotation().angle())
    }

    fn add_force_at_point(&mut self, entity: Entity, force: Vec2, world_point: Vec2) {
        let Some(&handle) = self.entity_to_handle.get(&entity) else {
            return;
        };
        let Some(body) = self.bodies.get_mut(handle) else {
            return;
        };
        body.add_force_at_point(
            vector![force.x, force.y],
            point![world_point.x, world_point.y],
            true,
        );
    }

    fn set_damping(&mut self, entity: Entity, linear: f32, angular: f32) {
        let Some(&handle) = self.entity_to_handle.get(&entity) else {
            return;
        };
        let Some(body) = self.bodies.get_mut(handle) else {
            return;
        };
        body.set_linear_damping(linear);
        body.set_angular_damping(angular);
    }

    fn drain_collision_events(&mut self) -> Vec<CollisionEvent> {
        let mut events = Vec::new();
        while let Ok(rapier_event) = self.collision_recv.try_recv() {
            let (h1, h2, kind) = match rapier_event {
                rapier2d::geometry::CollisionEvent::Started(h1, h2, _) => {
                    (h1, h2, CollisionKind::Started)
                }
                rapier2d::geometry::CollisionEvent::Stopped(h1, h2, _) => {
                    (h1, h2, CollisionKind::Stopped)
                }
            };
            if let (Some(&entity_a), Some(&entity_b)) = (
                self.collider_to_entity.get(&h1),
                self.collider_to_entity.get(&h2),
            ) {
                events.push(CollisionEvent {
                    entity_a,
                    entity_b,
                    kind,
                });
            }
        }
        events
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::collision_event::CollisionKind;
    use crate::test_helpers::{spawn_entities, spawn_entity};

    #[test]
    fn when_rapier_step_on_empty_world_then_no_panic() {
        // Arrange
        let mut backend = RapierBackend::new(Vec2::new(0.0, -9.81));

        // Act
        backend.step(Seconds(0.016));
    }

    /// @doc: Body type mapping: ECS Dynamic → rapier Dynamic (free motion under forces)
    #[test]
    fn when_dynamic_body_added_then_position_is_queryable() {
        // Arrange
        let mut backend = RapierBackend::new(Vec2::ZERO);
        let entity = spawn_entity();

        // Act
        let added = backend.add_body(entity, &RigidBody::Dynamic, Vec2::new(3.0, 7.0));
        let pos = backend.body_position(entity);

        // Assert
        assert!(added);
        let pos = pos.unwrap();
        assert!((pos.x - 3.0).abs() < 1e-4);
        assert!((pos.y - 7.0).abs() < 1e-4);
    }

    #[test]
    fn when_same_entity_added_twice_then_second_returns_false() {
        // Arrange
        let mut backend = RapierBackend::new(Vec2::ZERO);
        let entity = spawn_entity();
        backend.add_body(entity, &RigidBody::Dynamic, Vec2::ZERO);

        // Act
        let second = backend.add_body(entity, &RigidBody::Dynamic, Vec2::ZERO);

        // Assert
        assert!(!second);
    }

    /// @doc: Body type mapping: ECS Static → rapier Fixed (immovable), ECS Kinematic → rapier `KinematicPositionBased` (script-driven)
    #[test]
    fn when_body_type_mapping_then_static_is_fixed_and_kinematic_is_position_based() {
        // Arrange
        let mut backend = RapierBackend::new(Vec2::ZERO);
        let entities = spawn_entities(2);
        let static_entity = entities[0];
        let kinematic_entity = entities[1];

        // Act
        backend.add_body(static_entity, &RigidBody::Static, Vec2::ZERO);
        backend.add_body(kinematic_entity, &RigidBody::Kinematic, Vec2::ZERO);

        // Assert
        let static_handle = backend.entity_to_handle[&static_entity];
        let kinematic_handle = backend.entity_to_handle[&kinematic_entity];
        let static_body = backend.bodies.get(static_handle).unwrap();
        let kinematic_body = backend.bodies.get(kinematic_handle).unwrap();
        assert_eq!(
            static_body.body_type(),
            rapier2d::prelude::RigidBodyType::Fixed
        );
        assert_eq!(
            kinematic_body.body_type(),
            rapier2d::prelude::RigidBodyType::KinematicPositionBased
        );
    }

    #[test]
    fn when_collider_variants_added_then_all_return_true() {
        // Arrange
        let mut backend = RapierBackend::new(Vec2::ZERO);
        let entities = spawn_entities(3);
        let (e1, e2, e3) = (entities[0], entities[1], entities[2]);
        backend.add_body(e1, &RigidBody::Dynamic, Vec2::ZERO);
        backend.add_body(e2, &RigidBody::Dynamic, Vec2::ZERO);
        backend.add_body(e3, &RigidBody::Dynamic, Vec2::ZERO);

        // Act
        let circle = backend.add_collider(e1, &Collider::Circle(2.0));
        let aabb = backend.add_collider(e2, &Collider::Aabb(Vec2::new(1.0, 0.5)));
        let polygon = backend.add_collider(
            e3,
            &Collider::ConvexPolygon(vec![Vec2::ZERO, Vec2::new(1.0, 0.0), Vec2::new(0.5, 1.0)]),
        );

        // Assert
        assert!(circle);
        assert!(aabb);
        assert!(polygon);
    }

    #[test]
    fn when_add_collider_for_unknown_entity_then_returns_false() {
        // Arrange
        let mut backend = RapierBackend::new(Vec2::ZERO);
        let entity = spawn_entity();

        // Act
        let result = backend.add_collider(entity, &Collider::Circle(1.0));

        // Assert
        assert!(!result);
    }

    #[test]
    fn when_dynamic_body_steps_under_gravity_then_y_changes() {
        // Arrange
        let mut backend = RapierBackend::new(Vec2::new(0.0, -9.81));
        let entity = spawn_entity();
        backend.add_body(entity, &RigidBody::Dynamic, Vec2::new(0.0, 10.0));
        backend.add_collider(entity, &Collider::Circle(0.5));

        // Act
        backend.step(Seconds(0.1));

        // Assert
        let pos = backend.body_position(entity).unwrap();
        assert!(pos.y < 10.0, "expected y < 10.0, got {}", pos.y);
    }

    #[test]
    fn when_dynamic_body_added_then_rotation_returns_some() {
        // Arrange
        let mut backend = RapierBackend::new(Vec2::ZERO);
        let entity = spawn_entity();
        backend.add_body(entity, &RigidBody::Dynamic, Vec2::ZERO);

        // Act
        let rotation = backend.body_rotation(entity);

        // Assert
        let rotation = rotation.expect("should return Some for living body");
        assert!(rotation.abs() < 1e-4, "initial rotation should be ~0");
    }

    /// @doc: Entity removal must clean up both rapier `RigidBody` and the entity↔handle map
    #[test]
    fn when_remove_body_on_rapier_then_position_returns_none() {
        // Arrange
        let mut backend = RapierBackend::new(Vec2::ZERO);
        let entity = spawn_entity();
        backend.add_body(entity, &RigidBody::Dynamic, Vec2::new(1.0, 2.0));

        // Act
        backend.remove_body(entity);

        // Assert
        assert!(backend.body_position(entity).is_none());
        assert!(backend.body_rotation(entity).is_none());
    }

    #[test]
    fn when_no_colliders_step_and_drain_then_no_events() {
        // Arrange
        let mut backend = RapierBackend::new(Vec2::ZERO);

        // Act
        backend.step(Seconds(0.016));
        let events = backend.drain_collision_events();

        // Assert
        assert!(events.is_empty());
    }

    /// @doc: Collision events flow: rapier `ChannelEventCollector` → drain → `CollisionEventBuffer` with entity resolution
    #[test]
    fn when_two_overlapping_circles_step_then_started_event_with_correct_entities() {
        // Arrange
        let mut backend = RapierBackend::new(Vec2::ZERO);
        let entities = spawn_entities(2);
        backend.add_body(entities[0], &RigidBody::Dynamic, Vec2::ZERO);
        backend.add_collider(entities[0], &Collider::Circle(1.0));
        backend.add_body(entities[1], &RigidBody::Dynamic, Vec2::ZERO);
        backend.add_collider(entities[1], &Collider::Circle(1.0));

        // Act
        backend.step(Seconds(0.016));
        let events = backend.drain_collision_events();

        // Assert
        assert_eq!(events.len(), 1, "expected 1 event, got {events:?}");
        assert_eq!(events[0].kind, CollisionKind::Started);
        let pair = (events[0].entity_a, events[0].entity_b);
        assert!(
            pair == (entities[0], entities[1]) || pair == (entities[1], entities[0]),
            "expected entities {:?}, got {:?}",
            (entities[0], entities[1]),
            pair
        );
    }

    #[test]
    fn when_drain_called_twice_without_step_then_second_is_empty() {
        // Arrange
        let mut backend = RapierBackend::new(Vec2::ZERO);
        let entities = spawn_entities(2);
        backend.add_body(entities[0], &RigidBody::Dynamic, Vec2::ZERO);
        backend.add_collider(entities[0], &Collider::Circle(1.0));
        backend.add_body(entities[1], &RigidBody::Dynamic, Vec2::ZERO);
        backend.add_collider(entities[1], &Collider::Circle(1.0));
        backend.step(Seconds(0.016));
        let _ = backend.drain_collision_events();

        // Act
        let events = backend.drain_collision_events();

        // Assert
        assert!(events.is_empty());
    }

    #[test]
    fn when_body_removed_after_collision_then_drain_does_not_panic() {
        // Arrange
        let mut backend = RapierBackend::new(Vec2::ZERO);
        let entities = spawn_entities(2);
        backend.add_body(entities[0], &RigidBody::Dynamic, Vec2::ZERO);
        backend.add_collider(entities[0], &Collider::Circle(1.0));
        backend.add_body(entities[1], &RigidBody::Dynamic, Vec2::ZERO);
        backend.add_collider(entities[1], &Collider::Circle(1.0));
        backend.step(Seconds(0.016));
        backend.remove_body(entities[0]);

        // Act
        backend.step(Seconds(0.016));
        let _ = backend.drain_collision_events();
    }

    #[test]
    fn when_remove_body_for_unknown_entity_on_rapier_then_no_panic() {
        // Arrange
        let mut backend = RapierBackend::new(Vec2::ZERO);
        let entity = spawn_entity();

        // Act
        backend.remove_body(entity);
    }

    #[test]
    fn when_add_force_at_point_for_unknown_entity_then_no_panic() {
        // Arrange
        let mut backend = RapierBackend::new(Vec2::ZERO);
        let entity = spawn_entity();

        // Act
        backend.add_force_at_point(entity, Vec2::new(100.0, 0.0), Vec2::ZERO);
    }

    #[test]
    fn when_set_damping_on_unknown_entity_then_no_panic() {
        // Arrange
        let mut backend = RapierBackend::new(Vec2::ZERO);
        let entity = spawn_entity();

        // Act
        backend.set_damping(entity, 5.0, 3.0);
    }

    fn apply_impulse(backend: &mut RapierBackend, entity: Entity, force: Vec2, point: Vec2) {
        backend.add_force_at_point(entity, force, point);
        backend.step(Seconds(0.016));
        let handle = backend.entity_to_handle[&entity];
        let body = backend.bodies.get_mut(handle).unwrap();
        body.reset_forces(false);
        body.reset_torques(false);
    }

    #[test]
    fn when_zero_damping_body_given_impulse_then_keeps_moving_after_force_stops() {
        // Arrange
        let mut backend = RapierBackend::new(Vec2::ZERO);
        let entity = spawn_entity();
        backend.add_body(entity, &RigidBody::Dynamic, Vec2::ZERO);
        backend.add_collider(entity, &Collider::Circle(0.5));
        backend.set_damping(entity, 0.0, 0.0);
        apply_impulse(&mut backend, entity, Vec2::new(5000.0, 0.0), Vec2::ZERO);

        // Act
        for _ in 0..10 {
            backend.step(Seconds(0.016));
        }

        // Assert
        let pos = backend.body_position(entity).unwrap();
        assert!(
            pos.x > 0.5,
            "expected undamped body to coast to x > 0.5 after impulse, got {}",
            pos.x
        );
    }

    #[test]
    fn when_high_linear_damping_then_travels_less_distance_than_zero_damping() {
        // Arrange
        let mut undamped = RapierBackend::new(Vec2::ZERO);
        let entity_u = spawn_entity();
        undamped.add_body(entity_u, &RigidBody::Dynamic, Vec2::ZERO);
        undamped.add_collider(entity_u, &Collider::Circle(0.5));
        undamped.set_damping(entity_u, 0.0, 0.0);

        let mut damped = RapierBackend::new(Vec2::ZERO);
        let entity_d = spawn_entity();
        damped.add_body(entity_d, &RigidBody::Dynamic, Vec2::ZERO);
        damped.add_collider(entity_d, &Collider::Circle(0.5));
        damped.set_damping(entity_d, 20.0, 0.0);

        // Act — one step with force, reset forces, then coast
        apply_impulse(&mut undamped, entity_u, Vec2::new(5000.0, 0.0), Vec2::ZERO);
        apply_impulse(&mut damped, entity_d, Vec2::new(5000.0, 0.0), Vec2::ZERO);

        for _ in 0..30 {
            undamped.step(Seconds(0.016));
            damped.step(Seconds(0.016));
        }

        // Assert
        let x_undamped = undamped.body_position(entity_u).unwrap().x;
        let x_damped = damped.body_position(entity_d).unwrap().x;
        assert!(
            x_damped < x_undamped * 0.5,
            "expected damped x ({}) < 50% of undamped x ({})",
            x_damped,
            x_undamped
        );
    }

    #[test]
    fn when_high_angular_damping_then_rotates_less_than_undamped() {
        // Arrange
        let mut undamped = RapierBackend::new(Vec2::ZERO);
        let entity_u = spawn_entity();
        undamped.add_body(entity_u, &RigidBody::Dynamic, Vec2::ZERO);
        undamped.add_collider(entity_u, &Collider::Circle(0.5));
        undamped.set_damping(entity_u, 0.0, 0.0);

        let mut damped = RapierBackend::new(Vec2::ZERO);
        let entity_d = spawn_entity();
        damped.add_body(entity_d, &RigidBody::Dynamic, Vec2::ZERO);
        damped.add_collider(entity_d, &Collider::Circle(0.5));
        damped.set_damping(entity_d, 0.0, 20.0);

        // Act — off-center force for 1 step to induce spin, then reset forces so they don't persist
        apply_impulse(
            &mut undamped,
            entity_u,
            Vec2::new(50.0, 0.0),
            Vec2::new(0.0, 1.0),
        );
        apply_impulse(
            &mut damped,
            entity_d,
            Vec2::new(50.0, 0.0),
            Vec2::new(0.0, 1.0),
        );

        let handle_u = undamped.entity_to_handle[&entity_u];
        let handle_d = damped.entity_to_handle[&entity_d];

        for _ in 0..10 {
            undamped.step(Seconds(0.016));
            damped.step(Seconds(0.016));
        }

        // Assert — compare angular velocity since cumulative rotation wraps
        let angvel_undamped = undamped.bodies.get(handle_u).unwrap().angvel().abs();
        let angvel_damped = damped.bodies.get(handle_d).unwrap().angvel().abs();
        assert!(
            angvel_damped < angvel_undamped * 0.5,
            "expected damped angvel ({}) < 50% of undamped angvel ({})",
            angvel_damped,
            angvel_undamped
        );
    }

    #[test]
    fn when_damping_reset_to_zero_then_body_moves_like_undamped() {
        // Arrange
        let mut reference = RapierBackend::new(Vec2::ZERO);
        let entity_r = spawn_entity();
        reference.add_body(entity_r, &RigidBody::Dynamic, Vec2::ZERO);
        reference.add_collider(entity_r, &Collider::Circle(0.5));
        reference.set_damping(entity_r, 0.0, 0.0);

        let mut reset = RapierBackend::new(Vec2::ZERO);
        let entity_s = spawn_entity();
        reset.add_body(entity_s, &RigidBody::Dynamic, Vec2::ZERO);
        reset.add_collider(entity_s, &Collider::Circle(0.5));
        reset.set_damping(entity_s, 20.0, 20.0);
        reset.set_damping(entity_s, 0.0, 0.0);

        // Act — one step with force, reset forces, then coast
        apply_impulse(&mut reference, entity_r, Vec2::new(5000.0, 0.0), Vec2::ZERO);
        apply_impulse(&mut reset, entity_s, Vec2::new(5000.0, 0.0), Vec2::ZERO);

        for _ in 0..30 {
            reference.step(Seconds(0.016));
            reset.step(Seconds(0.016));
        }

        // Assert
        let x_reference = reference.body_position(entity_r).unwrap().x;
        let x_reset = reset.body_position(entity_s).unwrap().x;
        assert!(
            (x_reset - x_reference).abs() < 1e-3,
            "expected reset body x ({}) ≈ reference x ({})",
            x_reset,
            x_reference
        );
    }

    #[test]
    fn when_zero_force_applied_at_center_then_body_does_not_move() {
        // Arrange
        let mut backend = RapierBackend::new(Vec2::ZERO);
        let entity = spawn_entity();
        backend.add_body(entity, &RigidBody::Dynamic, Vec2::ZERO);
        backend.add_collider(entity, &Collider::Circle(0.5));

        // Act
        backend.add_force_at_point(entity, Vec2::ZERO, Vec2::ZERO);
        backend.step(Seconds(0.016));

        // Assert
        let pos = backend.body_position(entity).unwrap();
        assert!(pos.x.abs() < 1e-4, "expected ~0 x, got {}", pos.x);
        assert!(pos.y.abs() < 1e-4, "expected ~0 y, got {}", pos.y);
    }

    #[test]
    fn when_sustained_x_force_at_center_then_only_x_grows_and_rotation_stays_zero() {
        // Arrange
        let mut backend = RapierBackend::new(Vec2::ZERO);
        let entity = spawn_entity();
        backend.add_body(entity, &RigidBody::Dynamic, Vec2::ZERO);
        backend.add_collider(entity, &Collider::Circle(0.5));

        // Act
        for _ in 0..5 {
            backend.add_force_at_point(entity, Vec2::new(1000.0, 0.0), Vec2::ZERO);
            backend.step(Seconds(0.016));
        }

        // Assert
        let pos = backend.body_position(entity).unwrap();
        let rot = backend.body_rotation(entity).unwrap();
        assert!(
            pos.x > 0.0,
            "expected x > 0 after repeated +x force, got {}",
            pos.x
        );
        assert!(pos.y.abs() < 1e-4, "expected y ≈ 0, got {}", pos.y);
        assert!(rot.abs() < 1e-5, "expected no rotation, got {}", rot);
    }

    #[test]
    fn when_sustained_x_force_at_offset_y_point_then_body_translates_and_rotates() {
        // Arrange
        let mut backend = RapierBackend::new(Vec2::ZERO);
        let entity = spawn_entity();
        backend.add_body(entity, &RigidBody::Dynamic, Vec2::ZERO);
        backend.add_collider(entity, &Collider::Circle(0.5));

        // Act
        for _ in 0..5 {
            backend.add_force_at_point(entity, Vec2::new(1000.0, 0.0), Vec2::new(0.0, 1.0));
            backend.step(Seconds(0.016));
        }

        // Assert
        let pos = backend.body_position(entity).unwrap();
        let rot = backend.body_rotation(entity).unwrap();
        assert!(pos.x > 0.0, "expected x > 0, got {}", pos.x);
        assert!(
            rot.abs() > 1e-5,
            "expected rotation from torque, got {}",
            rot
        );
    }

    #[test]
    fn when_rapier_body_rotated_then_body_point_to_world_matches_manual_transform() {
        // Arrange
        let mut backend = RapierBackend::new(Vec2::ZERO);
        let entity = spawn_entity();
        backend.add_body(entity, &RigidBody::Dynamic, Vec2::new(3.0, 4.0));
        backend.add_collider(entity, &Collider::Circle(0.5));
        backend.set_damping(entity, 0.0, 0.0);
        apply_impulse(
            &mut backend,
            entity,
            Vec2::new(50.0, 0.0),
            Vec2::new(3.0, 5.0),
        );
        for _ in 0..5 {
            backend.step(Seconds(0.016));
        }

        // Act
        let local_offset = Vec2::new(1.0, 0.5);
        let result = backend.body_point_to_world(entity, local_offset);

        // Assert
        let world_pt = result.unwrap();
        let pos = backend.body_position(entity).unwrap();
        let rot = backend.body_rotation(entity).unwrap();
        let (sin, cos) = rot.sin_cos();
        let expected = pos + Vec2::new(
            local_offset.x * cos - local_offset.y * sin,
            local_offset.x * sin + local_offset.y * cos,
        );
        assert!(
            (world_pt.x - expected.x).abs() < 1e-4,
            "x: got {}, expected {}",
            world_pt.x,
            expected.x
        );
        assert!(
            (world_pt.y - expected.y).abs() < 1e-4,
            "y: got {}, expected {}",
            world_pt.y,
            expected.y
        );
    }
}
