use std::collections::HashMap;
use std::sync::{Mutex, mpsc};

use bevy_ecs::prelude::Entity;
use engine_core::prelude::Seconds;
use glam::Vec2;
use rapier2d::prelude::*;

use crate::collider::Collider;
use crate::collision_event::{CollisionEvent, CollisionKind};
use crate::physics_backend::{PhysicsBackend, PhysicsError};
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
    collision_recv: Mutex<mpsc::Receiver<rapier2d::geometry::CollisionEvent>>,
}

impl RapierBackend {
    /// Reset accumulated forces and torques on a body (test infrastructure).
    #[doc(hidden)]
    pub fn reset_body_forces(&mut self, entity: Entity) {
        if let Some(&handle) = self.entity_to_handle.get(&entity)
            && let Some(body) = self.bodies.get_mut(handle)
        {
            body.reset_forces(false);
            body.reset_torques(false);
        }
    }

    /// Query the rapier-internal body type for an entity (test infrastructure).
    #[doc(hidden)]
    pub fn rapier_body_type(&self, entity: Entity) -> Option<rapier2d::prelude::RigidBodyType> {
        let handle = self.entity_to_handle.get(&entity)?;
        let body = self.bodies.get(*handle)?;
        Some(body.body_type())
    }

    #[must_use]
    pub fn new(gravity: Vec2) -> Self {
        let (collision_send, collision_recv) = mpsc::channel();
        let (contact_force_send, _) = mpsc::channel();
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
            collision_recv: Mutex::new(collision_recv),
        }
    }
}

impl PhysicsBackend for RapierBackend {
    fn step(&mut self, dt: Seconds) {
        self.integration_parameters.dt = dt.0;
        self.pipeline.step(
            Vec2::new(self.gravity.x, self.gravity.y),
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            &mut self.ccd_solver,
            &(),
            &self.event_collector,
        );
    }

    fn add_body(&mut self, entity: Entity, body_type: &RigidBody, position: Vec2) -> bool {
        if self.entity_to_handle.contains_key(&entity) {
            tracing::warn!(?entity, "add_body: duplicate entity already registered");
            return false;
        }
        let rb = match body_type {
            RigidBody::Dynamic => RigidBodyBuilder::dynamic(),
            RigidBody::Static => RigidBodyBuilder::fixed(),
            RigidBody::Kinematic => RigidBodyBuilder::kinematic_position_based(),
        }
        .translation(Vec2::new(position.x, position.y))
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
                let rapier_points: Vec<Vec2> = points.clone();
                let Some(builder) = ColliderBuilder::convex_hull(&rapier_points) else {
                    tracing::warn!(?entity, "add_collider: convex hull build failed — points may be collinear or degenerate");
                    return false;
                };
                builder
            }
        }
        .active_events(ActiveEvents::COLLISION_EVENTS);
        let collider_handle =
            self.colliders
                .insert_with_parent(col.build(), body_handle, &mut self.bodies);
        self.collider_to_entity.insert(collider_handle, entity);
        true
    }

    fn remove_body(&mut self, entity: Entity) -> Result<(), PhysicsError> {
        let Some(handle) = self.entity_to_handle.remove(&entity) else {
            tracing::warn!(?entity, "remove_body: entity not found in physics world");
            return Err(PhysicsError::EntityNotFound(entity));
        };
        self.bodies.remove(
            handle,
            &mut self.island_manager,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            true,
        );
        Ok(())
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

    fn body_linear_velocity(&self, entity: Entity) -> Option<Vec2> {
        let handle = self.entity_to_handle.get(&entity)?;
        let body = self.bodies.get(*handle)?;
        let vel = body.linvel();
        Some(Vec2::new(vel.x, vel.y))
    }

    fn set_linear_velocity(&mut self, entity: Entity, velocity: Vec2) -> Result<(), PhysicsError> {
        let &handle = self.entity_to_handle.get(&entity).ok_or_else(|| {
            tracing::warn!(?entity, "set_linear_velocity: entity not found");
            PhysicsError::EntityNotFound(entity)
        })?;
        let body = self
            .bodies
            .get_mut(handle)
            .ok_or(PhysicsError::EntityNotFound(entity))?;
        body.set_linvel(Vec2::new(velocity.x, velocity.y), true);
        Ok(())
    }

    fn set_angular_velocity(
        &mut self,
        entity: Entity,
        angular_velocity: f32,
    ) -> Result<(), PhysicsError> {
        let &handle = self.entity_to_handle.get(&entity).ok_or_else(|| {
            tracing::warn!(?entity, "set_angular_velocity: entity not found");
            PhysicsError::EntityNotFound(entity)
        })?;
        let body = self
            .bodies
            .get_mut(handle)
            .ok_or(PhysicsError::EntityNotFound(entity))?;
        body.set_angvel(angular_velocity, true);
        Ok(())
    }

    fn body_angular_velocity(&self, entity: Entity) -> Option<f32> {
        let handle = self.entity_to_handle.get(&entity)?;
        let body = self.bodies.get(*handle)?;
        Some(body.angvel())
    }

    fn add_force_at_point(
        &mut self,
        entity: Entity,
        force: Vec2,
        world_point: Vec2,
    ) -> Result<(), PhysicsError> {
        let &handle = self.entity_to_handle.get(&entity).ok_or_else(|| {
            tracing::warn!(?entity, "add_force_at_point: entity not found");
            PhysicsError::EntityNotFound(entity)
        })?;
        let body = self
            .bodies
            .get_mut(handle)
            .ok_or(PhysicsError::EntityNotFound(entity))?;
        body.add_force_at_point(
            Vec2::new(force.x, force.y),
            Vec2::new(world_point.x, world_point.y),
            true,
        );
        Ok(())
    }

    fn set_damping(
        &mut self,
        entity: Entity,
        linear: f32,
        angular: f32,
    ) -> Result<(), PhysicsError> {
        let &handle = self.entity_to_handle.get(&entity).ok_or_else(|| {
            tracing::warn!(?entity, "set_damping: entity not found");
            PhysicsError::EntityNotFound(entity)
        })?;
        let body = self
            .bodies
            .get_mut(handle)
            .ok_or(PhysicsError::EntityNotFound(entity))?;
        body.set_linear_damping(linear);
        body.set_angular_damping(angular);
        Ok(())
    }

    fn set_collision_group(
        &mut self,
        entity: Entity,
        membership: u32,
        filter: u32,
    ) -> Result<(), PhysicsError> {
        let &handle = self.entity_to_handle.get(&entity).ok_or_else(|| {
            tracing::warn!(?entity, "set_collision_group: entity not found");
            PhysicsError::EntityNotFound(entity)
        })?;
        let groups = InteractionGroups::new(
            Group::from_bits_truncate(membership),
            Group::from_bits_truncate(filter),
            InteractionTestMode::And,
        );
        let body = self
            .bodies
            .get(handle)
            .ok_or(PhysicsError::EntityNotFound(entity))?;
        for &collider_handle in body.colliders() {
            if let Some(collider) = self.colliders.get_mut(collider_handle) {
                collider.set_collision_groups(groups);
            }
        }
        Ok(())
    }

    fn set_body_position(&mut self, entity: Entity, position: Vec2) -> Result<(), PhysicsError> {
        let &handle = self.entity_to_handle.get(&entity).ok_or_else(|| {
            tracing::warn!(?entity, "set_body_position: entity not found");
            PhysicsError::EntityNotFound(entity)
        })?;
        let body = self
            .bodies
            .get_mut(handle)
            .ok_or(PhysicsError::EntityNotFound(entity))?;
        body.set_next_kinematic_position(Pose {
            rotation: Rot2::IDENTITY,
            translation: Vec2::new(position.x, position.y),
        });
        Ok(())
    }

    fn drain_collision_events(&mut self) -> Vec<CollisionEvent> {
        let mut events = Vec::new();
        let recv = self
            .collision_recv
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        while let Ok(rapier_event) = recv.try_recv() {
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
