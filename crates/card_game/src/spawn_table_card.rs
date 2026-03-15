use bevy_ecs::prelude::{Entity, World};
use engine_core::prelude::{Color, Pixels, Transform2D};
use engine_physics::prelude::{Collider, PhysicsRes, RigidBody};
use engine_render::prelude::Sprite;
use engine_scene::prelude::{RenderLayer, SortOrder};
use glam::Vec2;

use crate::card::Card;
use crate::card_damping::{BASE_ANGULAR_DRAG, BASE_LINEAR_DRAG};
use crate::card_zone::CardZone;

pub const CARD_WIDTH: f32 = 60.0;
pub const CARD_HEIGHT: f32 = 90.0;

pub fn spawn_table_card(world: &mut World, card: Card, position: Vec2, card_size: Vec2) -> Entity {
    let half = card_size * 0.5;
    let texture = if card.face_up {
        card.face_texture
    } else {
        card.back_texture
    };

    let entity = world
        .spawn((
            card,
            CardZone::Table,
            Sprite {
                texture,
                uv_rect: [0.0, 0.0, 1.0, 1.0],
                color: Color::WHITE,
                width: Pixels(card_size.x),
                height: Pixels(card_size.y),
            },
            Transform2D {
                position,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
            RigidBody::Dynamic,
            Collider::Aabb(half),
            RenderLayer::World,
            SortOrder(0),
        ))
        .id();

    if let Some(mut physics) = world.get_resource_mut::<PhysicsRes>() {
        physics.add_body(entity, &RigidBody::Dynamic, position);
        physics.add_collider(entity, &Collider::Aabb(half));
        physics.set_damping(entity, BASE_LINEAR_DRAG, BASE_ANGULAR_DRAG);
    }

    entity
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::{Arc, Mutex};

    use bevy_ecs::prelude::*;
    use engine_core::prelude::{Seconds, TextureId};
    use engine_physics::prelude::{
        Collider, CollisionEvent, PhysicsBackend, PhysicsRes, RigidBody,
    };
    use glam::Vec2;

    use super::*;

    type AddBodyLog = Arc<Mutex<Vec<(Entity, Vec2)>>>;
    type AddColliderLog = Arc<Mutex<Vec<Entity>>>;
    type DampingLog = Arc<Mutex<Vec<(Entity, f32, f32)>>>;

    struct SpyPhysicsBackend {
        add_body_log: AddBodyLog,
        add_collider_log: AddColliderLog,
        damping_log: DampingLog,
    }

    impl SpyPhysicsBackend {
        fn new(
            add_body_log: AddBodyLog,
            add_collider_log: AddColliderLog,
            damping_log: DampingLog,
        ) -> Self {
            Self {
                add_body_log,
                add_collider_log,
                damping_log,
            }
        }
    }

    impl PhysicsBackend for SpyPhysicsBackend {
        fn step(&mut self, _dt: Seconds) {}
        fn add_body(&mut self, entity: Entity, _body_type: &RigidBody, position: Vec2) -> bool {
            self.add_body_log.lock().unwrap().push((entity, position));
            true
        }
        fn add_collider(&mut self, entity: Entity, _collider: &Collider) -> bool {
            self.add_collider_log.lock().unwrap().push(entity);
            true
        }
        fn remove_body(&mut self, _: Entity) {}
        fn body_position(&self, _: Entity) -> Option<Vec2> {
            None
        }
        fn body_rotation(&self, _: Entity) -> Option<f32> {
            None
        }
        fn drain_collision_events(&mut self) -> Vec<CollisionEvent> {
            Vec::new()
        }
        fn body_linear_velocity(&self, _: Entity) -> Option<Vec2> {
            None
        }
        fn set_linear_velocity(&mut self, _: Entity, _: Vec2) {}
        fn set_angular_velocity(&mut self, _: Entity, _: f32) {}
        fn add_force_at_point(&mut self, _: Entity, _: Vec2, _: Vec2) {}
        fn body_angular_velocity(&self, _: Entity) -> Option<f32> {
            None
        }
        fn set_damping(&mut self, entity: Entity, linear: f32, angular: f32) {
            self.damping_log
                .lock()
                .unwrap()
                .push((entity, linear, angular));
        }
        fn set_collision_group(&mut self, _: Entity, _: u32, _: u32) {}
    }

    fn make_spy_world() -> (World, AddBodyLog, AddColliderLog, DampingLog) {
        let add_body_log: AddBodyLog = Arc::new(Mutex::new(Vec::new()));
        let add_collider_log: AddColliderLog = Arc::new(Mutex::new(Vec::new()));
        let damping_log: DampingLog = Arc::new(Mutex::new(Vec::new()));
        let mut world = World::new();
        world.insert_resource(PhysicsRes::new(Box::new(SpyPhysicsBackend::new(
            add_body_log.clone(),
            add_collider_log.clone(),
            damping_log.clone(),
        ))));
        (world, add_body_log, add_collider_log, damping_log)
    }

    #[test]
    fn when_spawning_table_card_then_physics_body_registered() {
        // Arrange
        let (mut world, add_body_log, _, _) = make_spy_world();
        let card = Card::face_down(TextureId(1), TextureId(2));
        let pos = Vec2::new(100.0, 50.0);

        // Act
        let entity = spawn_table_card(&mut world, card, pos, Vec2::new(CARD_WIDTH, CARD_HEIGHT));

        // Assert
        let calls = add_body_log.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, entity);
        assert_eq!(calls[0].1, pos);
    }

    #[test]
    fn when_spawning_table_card_then_physics_collider_registered() {
        // Arrange
        let (mut world, _, add_collider_log, _) = make_spy_world();
        let card = Card::face_down(TextureId(1), TextureId(2));

        // Act
        let entity = spawn_table_card(
            &mut world,
            card,
            Vec2::ZERO,
            Vec2::new(CARD_WIDTH, CARD_HEIGHT),
        );

        // Assert
        let calls = add_collider_log.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0], entity);
    }

    #[test]
    fn when_spawning_table_card_then_initial_damping_set() {
        // Arrange
        let (mut world, _, _, damping_log) = make_spy_world();
        let card = Card::face_down(TextureId(1), TextureId(2));

        // Act
        let entity = spawn_table_card(
            &mut world,
            card,
            Vec2::ZERO,
            Vec2::new(CARD_WIDTH, CARD_HEIGHT),
        );

        // Assert
        let calls = damping_log.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, entity);
        assert!((calls[0].1 - BASE_LINEAR_DRAG).abs() < 1e-4);
        assert!((calls[0].2 - BASE_ANGULAR_DRAG).abs() < 1e-4);
    }
}
