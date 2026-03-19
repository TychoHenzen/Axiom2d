use bevy_ecs::prelude::{Commands, Entity, Query, Res, ResMut};
use engine_core::prelude::Transform2D;
use engine_input::prelude::{MouseButton, MouseState};
use engine_physics::prelude::{Collider, PhysicsRes, RigidBody};
use engine_render::prelude::RendererRes;
use engine_scene::prelude::RenderLayer;
use glam::Vec2;

use crate::card::Card;
use crate::card_item_form::CardItemForm;
use crate::card_pick::{CARD_COLLISION_FILTER, CARD_COLLISION_GROUP};
use crate::card_zone::CardZone;
use crate::drag_state::DragState;
use crate::flip_animation::FlipAnimation;
use crate::hand::Hand;
use crate::hand_layout::HandSpring;
use crate::physics_helpers::activate_physics_body;
use crate::stash_grid::StashGrid;
use crate::stash_grid::find_stash_slot_at;
use crate::stash_toggle::StashVisible;

pub const HAND_DROP_ZONE_HEIGHT: f32 = 120.0;

fn is_hand_drop_zone(screen_y: f32, viewport_height: f32) -> bool {
    screen_y >= viewport_height - HAND_DROP_ZONE_HEIGHT
}

enum DropTarget {
    Stash { page: u8, col: u8, row: u8 },
    Hand,
    Table,
}

fn resolve_drop_target(
    screen_pos: Vec2,
    viewport_height: f32,
    stash_visible: bool,
    grid: &StashGrid,
    origin_zone: &CardZone,
) -> DropTarget {
    if stash_visible {
        if let Some((col, row)) = find_stash_slot_at(screen_pos, grid.width(), grid.height()) {
            let page = grid.current_page();
            if grid.get(page, col, row).is_none() {
                return DropTarget::Stash { page, col, row };
            }
            if let CardZone::Stash {
                page: op,
                col: oc,
                row: orow,
            } = *origin_zone
            {
                return DropTarget::Stash {
                    page: op,
                    col: oc,
                    row: orow,
                };
            }
            return DropTarget::Table;
        }
    }

    if viewport_height > 0.0 && is_hand_drop_zone(screen_pos.y, viewport_height) {
        DropTarget::Hand
    } else {
        DropTarget::Table
    }
}

#[allow(clippy::too_many_arguments)]
pub fn card_release_system(
    mouse: Res<MouseState>,
    mut drag_state: ResMut<DragState>,
    mut hand: ResMut<Hand>,
    mut physics: ResMut<PhysicsRes>,
    renderer: Res<RendererRes>,
    stash_visible: Res<StashVisible>,
    mut grid: ResMut<StashGrid>,
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

    let screen_pos = mouse.screen_pos();
    let (_, vh) = renderer.viewport_size();
    let vh = vh as f32;

    let target = resolve_drop_target(screen_pos, vh, stash_visible.0, &grid, &info.origin_zone);

    match target {
        DropTarget::Stash { page, col, row } => {
            let current_pos = transform_query
                .get(info.entity)
                .ok()
                .map(|(t, _)| t.position);
            drop_on_stash(
                info.entity,
                page,
                col,
                row,
                current_pos,
                &mut grid,
                &mut physics,
                &mut commands,
            );
        }
        DropTarget::Hand => {
            let face_up = card_query.get(info.entity).is_ok_and(|c| c.face_up);
            drop_on_hand(info.entity, face_up, &mut hand, &mut physics, &mut commands);
        }
        DropTarget::Table => {
            drop_on_table(info.entity, &mut physics, &mut commands, &transform_query);
        }
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
    let zone = if let Ok(index) = hand.add(entity) {
        CardZone::Hand(index)
    } else {
        CardZone::Table
    };
    let mut ec = commands.entity(entity);
    ec.remove::<RigidBody>()
        .insert(zone)
        .insert(RenderLayer::UI)
        .remove::<CardItemForm>()
        .insert(HandSpring::new());
    if !face_up {
        ec.insert(FlipAnimation::start(true));
    }
}

fn drop_on_stash(
    entity: Entity,
    page: u8,
    col: u8,
    row: u8,
    current_pos: Option<Vec2>,
    grid: &mut StashGrid,
    physics: &mut PhysicsRes,
    commands: &mut Commands,
) {
    physics.remove_body(entity);
    grid.place(page, col, row, entity)
        .expect("slot should be empty: guarded by is_none check above");
    let mut ec = commands.entity(entity);
    ec.remove::<RigidBody>()
        .insert(CardZone::Stash { page, col, row })
        .insert(RenderLayer::UI)
        .insert(CardItemForm);
    if let Some(pos) = current_pos {
        ec.insert(Transform2D {
            position: pos,
            rotation: 0.0,
            scale: Vec2::ONE,
        });
    }
}

fn drop_on_table(
    entity: Entity,
    physics: &mut PhysicsRes,
    commands: &mut Commands,
    transform_query: &Query<(&Transform2D, &Collider)>,
) {
    if let Ok((transform, collider)) = transform_query.get(entity) {
        activate_physics_body(
            entity,
            transform.position,
            collider,
            physics,
            CARD_COLLISION_GROUP,
            CARD_COLLISION_FILTER,
        );
    }
    commands
        .entity(entity)
        .insert(RigidBody::Dynamic)
        .insert(CardZone::Table)
        .insert(RenderLayer::World)
        .remove::<CardItemForm>();
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::{Arc, Mutex};

    use bevy_ecs::prelude::*;
    use engine_core::prelude::{TextureId, Transform2D};
    use engine_input::prelude::{MouseButton, MouseState};
    use engine_physics::prelude::{Collider, PhysicsRes, RigidBody};
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
    use crate::test_helpers::{AddBodyLog, RemoveBodyLog, SpyPhysicsBackend};

    fn run_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(card_release_system);
        schedule.run(world);
    }

    fn make_release_world(
        viewport_h: u32,
        screen_x: f32,
        screen_y: f32,
        stash_visible: bool,
    ) -> (World, RemoveBodyLog, AddBodyLog) {
        let remove_log: RemoveBodyLog = Arc::new(Mutex::new(Vec::new()));
        let add_log: AddBodyLog = Arc::new(Mutex::new(Vec::new()));
        let mut world = World::new();
        world.insert_resource(PhysicsRes::new(Box::new(
            SpyPhysicsBackend::new()
                .with_remove_body_log(remove_log.clone())
                .with_add_body_log(add_log.clone()),
        )));
        world.insert_resource(Hand::new(10));

        let log = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log).with_viewport(800, viewport_h);
        world.insert_resource(RendererRes::new(Box::new(spy)));

        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.release(MouseButton::Left);
        mouse.set_screen_pos(Vec2::new(screen_x, screen_y));
        world.insert_resource(mouse);

        world.insert_resource(crate::stash_grid::StashGrid::new(10, 10, 1));
        world.insert_resource(crate::stash_toggle::StashVisible(stash_visible));

        (world, remove_log, add_log)
    }

    #[test]
    fn when_mouse_released_while_dragging_then_drag_state_cleared() {
        // Arrange
        let (mut world, _, _) = make_release_world(600, 400.0, 100.0, false);
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
                stash_cursor_follow: false,
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
        let (mut world, _, _) = make_release_world(600, 400.0, 100.0, false);
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
                stash_cursor_follow: false,
            }),
        });
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        world.insert_resource(mouse);
        world.insert_resource(Hand::new(10));
        world.insert_resource(PhysicsRes::new(Box::new(
            engine_physics::prelude::NullPhysicsBackend::default(),
        )));
        let log = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log).with_viewport(800, 600);
        world.insert_resource(RendererRes::new(Box::new(spy)));
        world.insert_resource(crate::stash_grid::StashGrid::new(10, 10, 1));
        world.insert_resource(crate::stash_toggle::StashVisible(false));

        // Act
        run_system(&mut world);

        // Assert
        assert!(world.resource::<DragState>().dragging.is_some());
    }

    #[test]
    fn when_card_released_on_table_then_zone_unchanged() {
        // Arrange
        let (mut world, _, _) = make_release_world(600, 400.0, 100.0, false);
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
                stash_cursor_follow: false,
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
        let (mut world, _, _) = make_release_world(600, 400.0, 550.0, false);
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
                stash_cursor_follow: false,
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
        let (mut world, _, _) = make_release_world(600, 400.0, 550.0, false);
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
                stash_cursor_follow: false,
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
        let (mut world, _, _) = make_release_world(600, 400.0, 550.0, false);
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
                stash_cursor_follow: false,
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
        let (mut world, remove_log, _) = make_release_world(600, 400.0, 550.0, false);
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
                stash_cursor_follow: false,
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
        let (mut world, _, _) = make_release_world(600, 400.0, 100.0, false);
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
                stash_cursor_follow: false,
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
        let (mut world, _, add_log) = make_release_world(600, 400.0, 100.0, false);
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
                stash_cursor_follow: false,
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
        let (mut world, _, _) = make_release_world(600, 400.0, 550.0, false);
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
                stash_cursor_follow: false,
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
        let (mut world, _, _) = make_release_world(600, 400.0, 550.0, false);
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
                stash_cursor_follow: false,
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
        let (mut world, _, _) = make_release_world(600, 400.0, 100.0, false);
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
                stash_cursor_follow: false,
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
        let (mut world, _, _) = make_release_world(600, 400.0, 550.0, false);
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
                stash_cursor_follow: false,
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
        let (mut world, _, _) = make_release_world(600, 400.0, 550.0, false);
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
                stash_cursor_follow: false,
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        let zone = world.get::<CardZone>(entity).unwrap();
        assert_eq!(*zone, CardZone::Table);
    }

    #[test]
    fn when_viewport_height_zero_then_card_dropped_on_table_not_hand() {
        // Arrange
        let (mut world, _, add_log) = make_release_world(0, 400.0, 100.0, false);
        let entity = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Table,
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
                origin_zone: CardZone::Table,
                stash_cursor_follow: false,
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        let zone = world.get::<CardZone>(entity).unwrap();
        assert_eq!(*zone, CardZone::Table);
        let hand = world.resource::<Hand>();
        assert!(
            hand.cards().is_empty(),
            "card should not be added to hand when viewport height is 0"
        );
        let calls = add_log.lock().unwrap();
        assert_eq!(
            calls.len(),
            1,
            "physics body should be re-added for table drop"
        );
    }

    #[test]
    fn when_release_to_hand_then_handspring_inserted() {
        // Arrange
        let (mut world, _, _) = make_release_world(600, 400.0, 550.0, false);
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
                stash_cursor_follow: false,
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

    #[test]
    fn when_released_over_empty_stash_slot_then_card_placed_in_grid_and_zone_updated() {
        // Arrange
        use crate::stash_grid::StashGrid;
        // slot (0,1,0) center: x = 20 + 1*54 + 25 = 99.0, y = 20 + 0*54 + 25 = 45.0
        let (mut world, _, _) = make_release_world(600, 99.0, 45.0, true);
        let entity = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Stash {
                    page: 0,
                    col: 0,
                    row: 0,
                },
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
                origin_zone: CardZone::Stash {
                    page: 0,
                    col: 0,
                    row: 0,
                },
                stash_cursor_follow: false,
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(
            world.resource::<StashGrid>().get(0, 1, 0),
            Some(&entity),
            "card should be placed at slot (0,1,0)"
        );
        assert_eq!(
            *world.get::<CardZone>(entity).unwrap(),
            CardZone::Stash {
                page: 0,
                col: 1,
                row: 0
            }
        );
        assert!(world.resource::<DragState>().dragging.is_none());
    }

    #[test]
    fn when_released_over_empty_stash_slot_then_physics_removed_and_render_layer_becomes_ui() {
        // Arrange
        // slot (0,0,0) center: x = 45.0, y = 45.0
        let (mut world, remove_log, _) = make_release_world(600, 45.0, 45.0, true);
        let entity = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Stash {
                    page: 0,
                    col: 0,
                    row: 0,
                },
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
                origin_zone: CardZone::Stash {
                    page: 0,
                    col: 0,
                    row: 0,
                },
                stash_cursor_follow: false,
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(
            remove_log.lock().unwrap().len(),
            1,
            "remove_body called once"
        );
        assert_eq!(*world.get::<RenderLayer>(entity).unwrap(), RenderLayer::UI);
    }

    #[test]
    fn when_released_over_occupied_stash_slot_then_card_returned_to_origin() {
        // Arrange
        use crate::stash_grid::StashGrid;
        use crate::stash_toggle::StashVisible;
        // slot (0,0,0) center at (45, 45) — occupied by another entity
        // origin slot (0,1,0) is where the dragged card came from
        let (mut world, _, _) = make_release_world(600, 400.0, 45.0, false);
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.release(MouseButton::Left);
        mouse.set_screen_pos(Vec2::new(45.0, 45.0));
        world.insert_resource(mouse);
        let blocker = world.spawn_empty().id();
        let dragged_entity = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Stash {
                    page: 0,
                    col: 1,
                    row: 0,
                },
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
        let mut grid = StashGrid::new(10, 10, 1);
        grid.place(0, 0, 0, blocker).unwrap();
        world.insert_resource(grid);
        world.insert_resource(StashVisible(true));
        world.insert_resource(DragState {
            dragging: Some(DragInfo {
                entity: dragged_entity,
                local_grab_offset: Vec2::ZERO,
                origin_zone: CardZone::Stash {
                    page: 0,
                    col: 1,
                    row: 0,
                },
                stash_cursor_follow: false,
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(
            world.resource::<StashGrid>().get(0, 1, 0),
            Some(&dragged_entity),
            "card should be returned to origin slot (0,1,0)"
        );
        assert_eq!(
            *world.get::<CardZone>(dragged_entity).unwrap(),
            CardZone::Stash {
                page: 0,
                col: 1,
                row: 0
            }
        );
    }

    #[test]
    fn when_stash_card_released_outside_stash_and_hand_zone_then_zone_becomes_table() {
        // Arrange
        // x=600 is past the stash grid (grid ends ~560); y=200 is above the hand zone (≥480)
        let (mut world, _, _) = make_release_world(600, 600.0, 200.0, true);
        let entity = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Stash {
                    page: 0,
                    col: 0,
                    row: 0,
                },
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
                origin_zone: CardZone::Stash {
                    page: 0,
                    col: 0,
                    row: 0,
                },
                stash_cursor_follow: false,
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        let zone = world.get::<CardZone>(entity).unwrap();
        assert_eq!(*zone, CardZone::Table);
    }

    #[test]
    fn when_stash_card_released_in_hand_zone_then_card_added_to_hand() {
        // Arrange
        // x=600 is past the stash grid so find_stash_slot_at returns None;
        // y=550 ≥ 600-120=480, so is_hand_drop_zone fires
        let (mut world, _, _) = make_release_world(600, 600.0, 550.0, true);
        let entity = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Stash {
                    page: 0,
                    col: 0,
                    row: 0,
                },
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
                origin_zone: CardZone::Stash {
                    page: 0,
                    col: 0,
                    row: 0,
                },
                stash_cursor_follow: false,
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        let hand = world.resource::<Hand>();
        assert!(
            hand.cards().contains(&entity),
            "stash-origin card should be in Hand"
        );
    }

    #[test]
    fn when_stash_card_released_in_hand_zone_then_zone_becomes_hand_and_stash_slot_stays_empty() {
        // Arrange
        let (mut world, _, _) = make_release_world(600, 600.0, 550.0, true);
        let entity = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Stash {
                    page: 0,
                    col: 0,
                    row: 0,
                },
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
                origin_zone: CardZone::Stash {
                    page: 0,
                    col: 0,
                    row: 0,
                },
                stash_cursor_follow: false,
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        let zone = world.get::<CardZone>(entity).unwrap();
        assert_eq!(*zone, CardZone::Hand(0));
        // The stash grid should not have been repopulated with the card
        let grid = world.resource::<crate::stash_grid::StashGrid>();
        assert!(
            grid.get(0, 0, 0).is_none(),
            "stash slot should not be repopulated after drop-on-hand"
        );
    }

    #[test]
    fn when_table_card_released_over_empty_stash_slot_then_zone_becomes_stash() {
        // Arrange
        // slot (0,0,0) center: x=45.0, y=45.0
        let (mut world, _, _) = make_release_world(600, 45.0, 45.0, true);
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
                stash_cursor_follow: false,
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        let zone = world.get::<CardZone>(entity).unwrap();
        assert_eq!(
            *zone,
            CardZone::Stash {
                page: 0,
                col: 0,
                row: 0
            }
        );
        let grid = world.resource::<crate::stash_grid::StashGrid>();
        assert_eq!(grid.get(0, 0, 0), Some(&entity));
    }

    #[test]
    fn when_table_card_released_over_empty_stash_slot_then_physics_body_removed() {
        // Arrange
        // slot (0,0,0) center: x=45.0, y=45.0
        let (mut world, remove_log, _) = make_release_world(600, 45.0, 45.0, true);
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
                stash_cursor_follow: false,
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        let calls = remove_log.lock().unwrap();
        assert_eq!(calls.len(), 1, "remove_body should be called exactly once");
        assert_eq!(calls[0], entity);
    }

    #[test]
    fn when_hand_card_released_over_empty_stash_slot_then_not_in_hand_resource() {
        // Arrange
        // slot (0,0,0) center: x=45.0, y=45.0
        let (mut world, _, _) = make_release_world(600, 45.0, 45.0, true);
        // Simulate card_pick_system having added a physics body: entity has RigidBody::Dynamic
        let entity = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Hand(0),
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
                origin_zone: CardZone::Hand(0),
                stash_cursor_follow: false,
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        let hand = world.resource::<Hand>();
        assert!(
            !hand.cards().contains(&entity),
            "hand-origin card dropped on stash must not be in Hand resource"
        );
        let grid = world.resource::<crate::stash_grid::StashGrid>();
        assert_eq!(
            grid.get(0, 0, 0),
            Some(&entity),
            "card should be in stash grid at slot (0,0,0)"
        );
    }

    #[test]
    fn when_stash_card_released_outside_stash_and_hand_zone_then_drops_on_table() {
        // Arrange
        let (mut world, _, add_log) = make_release_world(600, 600.0, 200.0, true);
        let entity = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Stash {
                    page: 0,
                    col: 0,
                    row: 0,
                },
                RigidBody::Dynamic,
                Collider::Aabb(Vec2::new(30.0, 45.0)),
                Transform2D {
                    position: Vec2::new(10.0, 20.0),
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
                origin_zone: CardZone::Stash {
                    page: 0,
                    col: 0,
                    row: 0,
                },
                stash_cursor_follow: false,
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        let zone = world.get::<CardZone>(entity).unwrap();
        assert_eq!(*zone, CardZone::Table);
        let calls = add_log.lock().unwrap();
        assert_eq!(
            calls.len(),
            1,
            "physics body should be re-added for table drop"
        );
    }

    #[test]
    fn when_stash_card_released_in_hand_zone_then_drops_on_hand() {
        // Arrange
        let (mut world, _, _) = make_release_world(600, 600.0, 550.0, true);
        let entity = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Stash {
                    page: 0,
                    col: 0,
                    row: 0,
                },
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
                origin_zone: CardZone::Stash {
                    page: 0,
                    col: 0,
                    row: 0,
                },
                stash_cursor_follow: false,
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        let hand = world.resource::<Hand>();
        assert!(
            hand.cards().contains(&entity),
            "stash-origin card should be in Hand"
        );
        let zone = world.get::<CardZone>(entity).unwrap();
        assert_eq!(*zone, CardZone::Hand(0));
        let grid = world.resource::<crate::stash_grid::StashGrid>();
        assert!(
            grid.get(0, 0, 0).is_none(),
            "stash grid should not be repopulated"
        );
    }

    #[test]
    fn when_table_card_released_over_empty_stash_slot_then_placed_in_stash_and_physics_removed() {
        // Arrange
        // slot (0,0,0) center: x=45.0, y=45.0
        let (mut world, remove_log, _) = make_release_world(600, 45.0, 45.0, true);
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
                stash_cursor_follow: false,
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        let grid = world.resource::<crate::stash_grid::StashGrid>();
        assert_eq!(
            grid.get(0, 0, 0),
            Some(&entity),
            "card should occupy slot (0,0,0)"
        );
        let zone = world.get::<CardZone>(entity).unwrap();
        assert_eq!(
            *zone,
            CardZone::Stash {
                page: 0,
                col: 0,
                row: 0
            }
        );
        let calls = remove_log.lock().unwrap();
        assert_eq!(calls.len(), 1, "remove_body should be called once");
        assert_eq!(calls[0], entity);
    }

    #[test]
    fn when_hand_card_released_over_empty_stash_slot_then_placed_in_stash_not_in_hand() {
        // Arrange
        // slot (0,0,0) center: x=45.0, y=45.0
        // card_pick_system adds RigidBody::Dynamic before DragState is set
        let (mut world, _, _) = make_release_world(600, 45.0, 45.0, true);
        let entity = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Hand(0),
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
                origin_zone: CardZone::Hand(0),
                stash_cursor_follow: false,
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        let grid = world.resource::<crate::stash_grid::StashGrid>();
        assert_eq!(
            grid.get(0, 0, 0),
            Some(&entity),
            "card should be in stash grid"
        );
        let hand = world.resource::<Hand>();
        assert!(
            !hand.cards().contains(&entity),
            "hand resource should not contain the card after stash drop"
        );
    }

    #[test]
    fn when_released_at_stash_slot_then_stash_drop_takes_priority_over_hand_zone() {
        // Arrange
        // slot (0,0,0) center is at screen (45, 45) — above the hand zone, but
        // this test documents that stash check runs BEFORE is_hand_drop_zone
        let (mut world, _, _) = make_release_world(600, 45.0, 45.0, true);
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
                stash_cursor_follow: false,
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        let grid = world.resource::<crate::stash_grid::StashGrid>();
        assert_eq!(
            grid.get(0, 0, 0),
            Some(&entity),
            "card should be in stash slot"
        );
        assert!(!world.resource::<Hand>().cards().contains(&entity));
    }

    #[test]
    fn when_table_card_dropped_on_stash_slot_then_card_item_form_inserted() {
        use crate::card_item_form::CardItemForm;

        // Arrange
        // slot (0,0,0) center: x=45.0, y=45.0
        let (mut world, _, _) = make_release_world(600, 45.0, 45.0, true);
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
                stash_cursor_follow: false,
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        assert!(
            world.get::<CardItemForm>(entity).is_some(),
            "CardItemForm should be inserted when card is dropped on stash slot"
        );
    }

    #[test]
    fn when_stash_card_dropped_on_table_area_then_card_item_form_removed() {
        use crate::card_item_form::CardItemForm;

        // Arrange
        // x=600 is past the stash grid; y=200 is above the hand zone (≥ 600-120=480)
        let (mut world, _, _) = make_release_world(600, 600.0, 200.0, true);
        let entity = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Stash {
                    page: 0,
                    col: 0,
                    row: 0,
                },
                CardItemForm,
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
                origin_zone: CardZone::Stash {
                    page: 0,
                    col: 0,
                    row: 0,
                },
                stash_cursor_follow: false,
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        assert!(
            world.get::<CardItemForm>(entity).is_none(),
            "CardItemForm should be removed when card is dropped on table area"
        );
    }

    #[test]
    fn when_stash_card_dropped_on_hand_zone_then_card_item_form_removed() {
        use crate::card_item_form::CardItemForm;

        // Arrange
        // x=600 is past the stash grid; y=550 ≥ 600-120=480, so is_hand_drop_zone fires
        let (mut world, _, _) = make_release_world(600, 600.0, 550.0, true);
        let entity = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Stash {
                    page: 0,
                    col: 0,
                    row: 0,
                },
                CardItemForm,
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
                origin_zone: CardZone::Stash {
                    page: 0,
                    col: 0,
                    row: 0,
                },
                stash_cursor_follow: false,
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        assert!(
            world.get::<CardItemForm>(entity).is_none(),
            "CardItemForm should be removed when card is dropped on hand zone"
        );
    }

    #[test]
    fn when_table_card_dropped_on_stash_slot_then_only_dragged_entity_gains_card_item_form() {
        use crate::card_item_form::CardItemForm;

        // Arrange
        // slot (0,0,0) center: x=45.0, y=45.0
        let (mut world, _, _) = make_release_world(600, 45.0, 45.0, true);
        let bystander = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Table,
                Transform2D {
                    position: Vec2::new(200.0, 200.0),
                    rotation: 0.0,
                    scale: Vec2::ONE,
                },
            ))
            .id();
        let dragged = world
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
                entity: dragged,
                local_grab_offset: Vec2::ZERO,
                origin_zone: CardZone::Table,
                stash_cursor_follow: false,
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        assert!(
            world.get::<CardItemForm>(dragged).is_some(),
            "dragged entity should gain CardItemForm"
        );
        assert!(
            world.get::<CardItemForm>(bystander).is_none(),
            "bystander entity must not gain CardItemForm"
        );
    }

    #[test]
    fn when_card_dropped_on_stash_then_scale_reset_to_one() {
        // Arrange
        // slot (0,0,0) center: x=45.0, y=57.5
        let (mut world, _, _) = make_release_world(600, 45.0, 57.5, true);
        let entity = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Table,
                RigidBody::Dynamic,
                Collider::Aabb(Vec2::new(30.0, 45.0)),
                Transform2D {
                    position: Vec2::ZERO,
                    rotation: 0.0,
                    scale: Vec2::splat(0.833),
                },
                RenderLayer::World,
            ))
            .id();
        world.insert_resource(DragState {
            dragging: Some(DragInfo {
                entity,
                local_grab_offset: Vec2::ZERO,
                origin_zone: CardZone::Table,
                stash_cursor_follow: false,
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        let transform = world.get::<Transform2D>(entity).unwrap();
        assert_eq!(transform.scale, Vec2::ONE);
    }

    #[test]
    fn when_card_dropped_on_stash_then_rotation_reset_to_zero() {
        // Arrange
        // slot (0,0,0) center: x=45.0, y=57.5
        let (mut world, _, _) = make_release_world(600, 45.0, 57.5, true);
        let entity = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Table,
                RigidBody::Dynamic,
                Collider::Aabb(Vec2::new(30.0, 45.0)),
                Transform2D {
                    position: Vec2::ZERO,
                    rotation: 0.8,
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
                stash_cursor_follow: false,
            }),
        });

        // Act
        run_system(&mut world);

        // Assert
        let transform = world.get::<Transform2D>(entity).unwrap();
        assert_eq!(transform.rotation, 0.0);
    }
}
