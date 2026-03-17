use bevy_ecs::prelude::{Commands, Entity, Query, Res, ResMut};
use engine_core::prelude::Transform2D;
use engine_input::prelude::{MouseButton, MouseState};
use engine_physics::prelude::{Collider, PhysicsRes, RigidBody};
use engine_render::prelude::RendererRes;
use engine_scene::prelude::RenderLayer;

use crate::card::Card;
use crate::card_damping::{BASE_ANGULAR_DRAG, BASE_LINEAR_DRAG};
use crate::card_pick::{CARD_COLLISION_FILTER, CARD_COLLISION_GROUP};
use crate::card_zone::CardZone;
use crate::drag_state::DragState;
use crate::flip_animation::FlipAnimation;
use crate::hand::Hand;
use crate::hand_layout::HandSpring;

pub const HAND_DROP_ZONE_HEIGHT: f32 = 120.0;

fn is_hand_drop_zone(screen_y: f32, viewport_height: f32) -> bool {
    screen_y >= viewport_height - HAND_DROP_ZONE_HEIGHT
}

pub fn card_release_system(
    mouse: Res<MouseState>,
    mut drag_state: ResMut<DragState>,
    mut hand: ResMut<Hand>,
    mut physics: ResMut<PhysicsRes>,
    renderer: Res<RendererRes>,
    mut commands: Commands,
    transform_query: Query<(&Transform2D, &Collider)>,
    card_query: Query<&Card>,
) {
    let Some(info) = drag_state.dragging else {
        return;
    };
    if !mouse.just_released(MouseButton::Left) {
        return;
    }

    let (_, vh) = renderer.viewport_size();
    let vh = vh as f32;
    let screen_y = mouse.screen_pos().y;

    if vh > 0.0 && is_hand_drop_zone(screen_y, vh) {
        let face_up = card_query.get(info.entity).is_ok_and(|c| c.face_up);
        drop_on_hand(info.entity, face_up, &mut hand, &mut physics, &mut commands);
    } else {
        drop_on_table(info.entity, &mut physics, &mut commands, &transform_query);
    }

    drag_state.dragging = None;
}

fn drop_on_hand(
    entity: Entity,
    face_up: bool,
    hand: &mut Hand,
    physics: &mut PhysicsRes,
    commands: &mut Commands,
) {
    physics.remove_body(entity);
    commands.entity(entity).remove::<RigidBody>();
    if let Ok(index) = hand.add(entity) {
        commands.entity(entity).insert(CardZone::Hand(index));
    } else {
        commands.entity(entity).insert(CardZone::Table);
    }
    commands.entity(entity).insert(RenderLayer::UI);
    commands.entity(entity).insert(HandSpring::new());
    if !face_up {
        commands.entity(entity).insert(FlipAnimation::start(true));
    }
}

fn drop_on_table(
    entity: Entity,
    physics: &mut PhysicsRes,
    commands: &mut Commands,
    transform_query: &Query<(&Transform2D, &Collider)>,
) {
    if let Ok((transform, collider)) = transform_query.get(entity) {
        let position = transform.position;
        physics.add_body(entity, &RigidBody::Dynamic, position);
        physics.add_collider(entity, collider);
        physics.set_damping(entity, BASE_LINEAR_DRAG, BASE_ANGULAR_DRAG);
        physics.set_collision_group(entity, CARD_COLLISION_GROUP, CARD_COLLISION_FILTER);
    }
    commands.entity(entity).insert(RigidBody::Dynamic);
    commands.entity(entity).insert(CardZone::Table);
    commands.entity(entity).insert(RenderLayer::World);
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::{Arc, Mutex};

    use bevy_ecs::prelude::*;
    use engine_core::prelude::{Seconds, TextureId, Transform2D};
    use engine_input::prelude::{MouseButton, MouseState};
    use engine_physics::prelude::{
        Collider, CollisionEvent, PhysicsBackend, PhysicsRes, RigidBody,
    };
    use engine_render::prelude::RendererRes;
    use engine_render::testing::SpyRenderer;
    use engine_scene::prelude::RenderLayer;
    use glam::Vec2;

    use super::card_release_system;
    use crate::card::Card;
    use crate::card_zone::CardZone;
    use crate::drag_state::{DragInfo, DragState};
    use crate::flip_animation::FlipAnimation;
    use crate::hand::Hand;
    use crate::hand_layout::HandSpring;

    fn run_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(card_release_system);
        schedule.run(world);
    }

    type RemoveBodyLog = Arc<Mutex<Vec<Entity>>>;
    type AddBodyLog = Arc<Mutex<Vec<(Entity, Vec2)>>>;

    struct SpyPhysicsBackend {
        remove_log: RemoveBodyLog,
        add_log: AddBodyLog,
    }

    impl SpyPhysicsBackend {
        fn new(remove_log: RemoveBodyLog, add_log: AddBodyLog) -> Self {
            Self {
                remove_log,
                add_log,
            }
        }
    }

    impl PhysicsBackend for SpyPhysicsBackend {
        fn step(&mut self, _dt: Seconds) {}
        fn add_body(&mut self, entity: Entity, _body_type: &RigidBody, position: Vec2) -> bool {
            self.add_log.lock().unwrap().push((entity, position));
            true
        }
        fn add_collider(&mut self, _: Entity, _: &Collider) -> bool {
            true
        }
        fn remove_body(&mut self, entity: Entity) {
            self.remove_log.lock().unwrap().push(entity);
        }
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
        fn set_damping(&mut self, _: Entity, _: f32, _: f32) {}
        fn set_collision_group(&mut self, _: Entity, _: u32, _: u32) {}
    }

    fn make_release_world(viewport_h: u32, screen_y: f32) -> (World, RemoveBodyLog, AddBodyLog) {
        let remove_log: RemoveBodyLog = Arc::new(Mutex::new(Vec::new()));
        let add_log: AddBodyLog = Arc::new(Mutex::new(Vec::new()));
        let mut world = World::new();
        world.insert_resource(PhysicsRes::new(Box::new(SpyPhysicsBackend::new(
            remove_log.clone(),
            add_log.clone(),
        ))));
        world.insert_resource(Hand::new(10));

        let log = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log).with_viewport(800, viewport_h);
        world.insert_resource(RendererRes::new(Box::new(spy)));

        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.release(MouseButton::Left);
        mouse.set_screen_pos(Vec2::new(400.0, screen_y));
        world.insert_resource(mouse);

        (world, remove_log, add_log)
    }

    #[test]
    fn when_mouse_released_while_dragging_then_drag_state_cleared() {
        // Arrange
        let (mut world, _, _) = make_release_world(600, 100.0);
        let entity = world
            .spawn((
                Transform2D {
                    position: Vec2::ZERO,
                    rotation: 0.0,
                    scale: Vec2::ONE,
                },
                Collider::Aabb(Vec2::new(30.0, 45.0)),
            ))
            .id();
        world.insert_resource(DragState {
            dragging: Some(DragInfo {
                entity,
                local_grab_offset: Vec2::ZERO,
                origin_zone: CardZone::Table,
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        assert!(world.resource::<DragState>().dragging.is_none());
    }

    #[test]
    fn when_mouse_released_while_not_dragging_then_no_panic_and_stays_none() {
        // Arrange
        let (mut world, _, _) = make_release_world(600, 100.0);
        world.insert_resource(DragState::default());

        // Act
        run_system(&mut world);

        // Assert
        assert!(world.resource::<DragState>().dragging.is_none());
    }

    #[test]
    fn when_mouse_not_released_then_drag_state_not_cleared() {
        // Arrange
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        world.insert_resource(DragState {
            dragging: Some(DragInfo {
                entity,
                local_grab_offset: Vec2::ZERO,
                origin_zone: CardZone::Table,
            }),
        });
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        world.insert_resource(mouse);
        world.insert_resource(Hand::new(10));
        world.insert_resource(PhysicsRes::new(Box::new(
            engine_physics::prelude::NullPhysicsBackend::new(),
        )));
        let log = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log).with_viewport(800, 600);
        world.insert_resource(RendererRes::new(Box::new(spy)));

        // Act
        run_system(&mut world);

        // Assert
        assert!(world.resource::<DragState>().dragging.is_some());
    }

    #[test]
    fn when_card_released_on_table_then_zone_unchanged() {
        // Arrange
        let (mut world, _, _) = make_release_world(600, 100.0);
        let entity = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Table,
                Transform2D {
                    position: Vec2::ZERO,
                    rotation: 0.0,
                    scale: Vec2::ONE,
                },
                Collider::Aabb(Vec2::new(30.0, 45.0)),
            ))
            .id();
        world.insert_resource(DragState {
            dragging: Some(DragInfo {
                entity,
                local_grab_offset: Vec2::ZERO,
                origin_zone: CardZone::Table,
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        let zone = world.get::<CardZone>(entity).unwrap();
        assert_eq!(*zone, CardZone::Table);
    }

    #[test]
    fn when_release_in_hand_area_from_table_then_card_added_to_hand() {
        // Arrange
        let (mut world, _, _) = make_release_world(600, 550.0);
        let entity = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Table,
                RigidBody::Dynamic,
                Collider::Aabb(Vec2::new(30.0, 45.0)),
                Transform2D {
                    position: Vec2::ZERO,
                    rotation: 0.0,
                    scale: Vec2::ONE,
                },
                RenderLayer::World,
            ))
            .id();
        world.insert_resource(DragState {
            dragging: Some(DragInfo {
                entity,
                local_grab_offset: Vec2::ZERO,
                origin_zone: CardZone::Table,
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        let hand = world.resource::<Hand>();
        assert_eq!(hand.cards(), &[entity]);
    }

    #[test]
    fn when_release_in_hand_area_from_table_then_zone_becomes_hand() {
        // Arrange
        let (mut world, _, _) = make_release_world(600, 550.0);
        let entity = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Table,
                RigidBody::Dynamic,
                Collider::Aabb(Vec2::new(30.0, 45.0)),
                Transform2D {
                    position: Vec2::ZERO,
                    rotation: 0.0,
                    scale: Vec2::ONE,
                },
                RenderLayer::World,
            ))
            .id();
        world.insert_resource(DragState {
            dragging: Some(DragInfo {
                entity,
                local_grab_offset: Vec2::ZERO,
                origin_zone: CardZone::Table,
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        let zone = world.get::<CardZone>(entity).unwrap();
        assert_eq!(*zone, CardZone::Hand(0));
    }

    #[test]
    fn when_release_in_hand_area_from_table_then_render_layer_becomes_ui() {
        // Arrange
        let (mut world, _, _) = make_release_world(600, 550.0);
        let entity = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Table,
                RigidBody::Dynamic,
                Collider::Aabb(Vec2::new(30.0, 45.0)),
                Transform2D {
                    position: Vec2::ZERO,
                    rotation: 0.0,
                    scale: Vec2::ONE,
                },
                RenderLayer::World,
            ))
            .id();
        world.insert_resource(DragState {
            dragging: Some(DragInfo {
                entity,
                local_grab_offset: Vec2::ZERO,
                origin_zone: CardZone::Table,
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        let layer = world.get::<RenderLayer>(entity).unwrap();
        assert_eq!(*layer, RenderLayer::UI);
    }

    #[test]
    fn when_release_in_hand_area_from_table_then_physics_body_removed() {
        // Arrange
        let (mut world, remove_log, _) = make_release_world(600, 550.0);
        let entity = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Table,
                RigidBody::Dynamic,
                Collider::Aabb(Vec2::new(30.0, 45.0)),
                Transform2D {
                    position: Vec2::ZERO,
                    rotation: 0.0,
                    scale: Vec2::ONE,
                },
                RenderLayer::World,
            ))
            .id();
        world.insert_resource(DragState {
            dragging: Some(DragInfo {
                entity,
                local_grab_offset: Vec2::ZERO,
                origin_zone: CardZone::Table,
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        let calls = remove_log.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0], entity);
    }

    #[test]
    fn when_release_on_table_from_hand_then_zone_becomes_table() {
        // Arrange
        let (mut world, _, _) = make_release_world(600, 100.0);
        let entity = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Hand(0),
                RigidBody::Dynamic,
                Collider::Aabb(Vec2::new(30.0, 45.0)),
                Transform2D {
                    position: Vec2::new(50.0, 50.0),
                    rotation: 0.0,
                    scale: Vec2::ONE,
                },
                RenderLayer::World,
            ))
            .id();
        world.insert_resource(DragState {
            dragging: Some(DragInfo {
                entity,
                local_grab_offset: Vec2::ZERO,
                origin_zone: CardZone::Hand(0),
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        let zone = world.get::<CardZone>(entity).unwrap();
        assert_eq!(*zone, CardZone::Table);
    }

    #[test]
    fn when_release_on_table_from_hand_then_physics_body_added() {
        // Arrange
        let (mut world, _, add_log) = make_release_world(600, 100.0);
        let entity = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Hand(0),
                RigidBody::Dynamic,
                Collider::Aabb(Vec2::new(30.0, 45.0)),
                Transform2D {
                    position: Vec2::new(50.0, 50.0),
                    rotation: 0.0,
                    scale: Vec2::ONE,
                },
                RenderLayer::World,
            ))
            .id();
        world.insert_resource(DragState {
            dragging: Some(DragInfo {
                entity,
                local_grab_offset: Vec2::ZERO,
                origin_zone: CardZone::Hand(0),
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        let calls = add_log.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, entity);
        assert_eq!(calls[0].1, Vec2::new(50.0, 50.0));
    }

    #[test]
    fn when_face_down_card_released_into_hand_then_flip_animation_inserted() {
        // Arrange
        let (mut world, _, _) = make_release_world(600, 550.0);
        let entity = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Table,
                RigidBody::Dynamic,
                Collider::Aabb(Vec2::new(30.0, 45.0)),
                Transform2D {
                    position: Vec2::ZERO,
                    rotation: 0.0,
                    scale: Vec2::ONE,
                },
                RenderLayer::World,
            ))
            .id();
        world.insert_resource(DragState {
            dragging: Some(DragInfo {
                entity,
                local_grab_offset: Vec2::ZERO,
                origin_zone: CardZone::Table,
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        let flip = world.get::<FlipAnimation>(entity);
        assert!(flip.is_some(), "expected FlipAnimation to be inserted");
        assert!(flip.unwrap().target_face_up, "expected target_face_up=true");
    }

    #[test]
    fn when_face_up_card_released_into_hand_then_no_flip_animation() {
        // Arrange
        let (mut world, _, _) = make_release_world(600, 550.0);
        let mut card = Card::face_down(TextureId(1), TextureId(2));
        card.face_up = true;
        let entity = world
            .spawn((
                card,
                CardZone::Table,
                RigidBody::Dynamic,
                Collider::Aabb(Vec2::new(30.0, 45.0)),
                Transform2D {
                    position: Vec2::ZERO,
                    rotation: 0.0,
                    scale: Vec2::ONE,
                },
                RenderLayer::World,
            ))
            .id();
        world.insert_resource(DragState {
            dragging: Some(DragInfo {
                entity,
                local_grab_offset: Vec2::ZERO,
                origin_zone: CardZone::Table,
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        let flip = world.get::<FlipAnimation>(entity);
        assert!(flip.is_none(), "expected no FlipAnimation for face-up card");
    }

    #[test]
    fn when_face_down_card_released_on_table_then_no_flip_animation() {
        // Arrange
        let (mut world, _, _) = make_release_world(600, 100.0);
        let entity = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Table,
                Collider::Aabb(Vec2::new(30.0, 45.0)),
                Transform2D {
                    position: Vec2::ZERO,
                    rotation: 0.0,
                    scale: Vec2::ONE,
                },
                RenderLayer::World,
            ))
            .id();
        world.insert_resource(DragState {
            dragging: Some(DragInfo {
                entity,
                local_grab_offset: Vec2::ZERO,
                origin_zone: CardZone::Table,
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        let flip = world.get::<FlipAnimation>(entity);
        assert!(flip.is_none(), "expected no FlipAnimation for table drop");
    }

    #[test]
    fn when_face_down_card_released_into_hand_then_also_added_to_hand() {
        // Arrange
        let (mut world, _, _) = make_release_world(600, 550.0);
        let entity = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Table,
                RigidBody::Dynamic,
                Collider::Aabb(Vec2::new(30.0, 45.0)),
                Transform2D {
                    position: Vec2::ZERO,
                    rotation: 0.0,
                    scale: Vec2::ONE,
                },
                RenderLayer::World,
            ))
            .id();
        world.insert_resource(DragState {
            dragging: Some(DragInfo {
                entity,
                local_grab_offset: Vec2::ZERO,
                origin_zone: CardZone::Table,
            }),
        });

        // Act
        run_system(&mut world);

        let hand = world.resource::<Hand>();
        assert!(hand.cards().contains(&entity), "expected card in hand");
        let flip = world.get::<FlipAnimation>(entity);
        assert!(flip.is_some(), "expected FlipAnimation also present");
    }

    #[test]
    fn when_hand_full_and_release_in_hand_area_then_card_stays_on_table() {
        // Arrange
        let (mut world, _, _) = make_release_world(600, 550.0);
        let existing = world.spawn_empty().id();
        let mut hand = Hand::new(1);
        hand.add(existing).unwrap();
        world.insert_resource(hand);
        let entity = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Table,
                RigidBody::Dynamic,
                Collider::Aabb(Vec2::new(30.0, 45.0)),
                Transform2D {
                    position: Vec2::ZERO,
                    rotation: 0.0,
                    scale: Vec2::ONE,
                },
                RenderLayer::World,
            ))
            .id();
        world.insert_resource(DragState {
            dragging: Some(DragInfo {
                entity,
                local_grab_offset: Vec2::ZERO,
                origin_zone: CardZone::Table,
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        let zone = world.get::<CardZone>(entity).unwrap();
        assert_eq!(*zone, CardZone::Table);
    }

    #[test]
    fn when_release_to_hand_then_handspring_inserted() {
        // Arrange
        let (mut world, _, _) = make_release_world(600, 550.0);
        let entity = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Table,
                RigidBody::Dynamic,
                Collider::Aabb(Vec2::new(30.0, 45.0)),
                Transform2D {
                    position: Vec2::ZERO,
                    rotation: 0.0,
                    scale: Vec2::ONE,
                },
                RenderLayer::World,
            ))
            .id();
        world.insert_resource(DragState {
            dragging: Some(DragInfo {
                entity,
                local_grab_offset: Vec2::ZERO,
                origin_zone: CardZone::Table,
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        assert!(
            world.get::<HandSpring>(entity).is_some(),
            "HandSpring should be inserted on release to hand"
        );
    }
}
