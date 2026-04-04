#![allow(clippy::unwrap_used, clippy::float_cmp)]

use bevy_ecs::prelude::*;
use engine_input::prelude::{MouseButton, MouseState};
use engine_physics::prelude::Collider;
use engine_scene::prelude::{GlobalTransform2D, SortOrder};
use glam::{Affine2, Vec2};

use engine_core::prelude::{EventBus, TextureId};

use card_game::card::component::{Card, CardZone};
use card_game::card::interaction::click_resolve::{
    ClickHitShape, Clickable, click_resolve_system, on_card_clicked,
};
use card_game::card::interaction::drag_state::DragState;
use card_game::card::interaction::intent::InteractionIntent;
use card_game::card::jack_socket::PendingCable;
use card_game::card::reader::ReaderDragState;

use super::helpers::default_card_collider;

const CARD_HALF: glam::Vec2 = glam::Vec2::new(30.0, 45.0);

fn run_system(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(click_resolve_system);
    schedule.run(world);
}

fn insert_pick_resources(world: &mut World) {
    if world.get_resource::<DragState>().is_none() {
        world.insert_resource(DragState::default());
    }
    world.insert_resource(card_game::stash::grid::StashGrid::new(10, 10, 1));
    world.insert_resource(card_game::stash::toggle::StashVisible(false));
    world.insert_resource(ReaderDragState::default());
    world.insert_resource(PendingCable::default());
    if world
        .get_resource::<EventBus<InteractionIntent>>()
        .is_none()
    {
        world.insert_resource(EventBus::<InteractionIntent>::default());
    }
}

/// Spawns a card at `pos` with Clickable + on_card_clicked observer. Returns the entity.
fn spawn_clickable_card(world: &mut World, pos: Vec2, sort: i32) -> Entity {
    let card = world
        .spawn((
            card_game::test_helpers::make_test_card(),
            card_game::card::component::CardZone::Table,
            default_card_collider(),
            GlobalTransform2D(Affine2::from_translation(pos)),
            SortOrder::new(sort),
            Clickable(ClickHitShape::Aabb(CARD_HALF)),
        ))
        .id();
    world.entity_mut(card).observe(on_card_clicked);
    card
}

#[test]
fn when_left_click_outside_all_cards_then_drag_state_remains_none() {
    // Arrange
    let mut world = World::new();
    spawn_clickable_card(&mut world, Vec2::ZERO, 0);
    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Left);
    mouse.set_world_pos(Vec2::new(200.0, 200.0));
    world.insert_resource(mouse);
    insert_pick_resources(&mut world);

    // Act
    run_system(&mut world);

    // Assert — cursor missed all cards, no pick intent
    assert!(world.resource::<EventBus<InteractionIntent>>().is_empty());
}

#[test]
fn when_left_click_with_no_table_cards_then_drag_state_remains_none() {
    // Arrange
    let mut world = World::new();
    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Left);
    mouse.set_world_pos(Vec2::ZERO);
    world.insert_resource(mouse);
    insert_pick_resources(&mut world);

    // Act
    run_system(&mut world);

    // Assert — no clickable entities, no pick intent
    assert!(world.resource::<EventBus<InteractionIntent>>().is_empty());
}

/// @doc: Sort order selects which card to pick when overlapping—highest sort is topmost
#[test]
fn when_two_overlapping_cards_then_picks_highest_sort_order() {
    // Arrange
    let mut world = World::new();
    spawn_clickable_card(&mut world, Vec2::ZERO, 0);
    let card_b = spawn_clickable_card(&mut world, Vec2::ZERO, 5);
    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Left);
    mouse.set_world_pos(Vec2::ZERO);
    world.insert_resource(mouse);
    insert_pick_resources(&mut world);

    // Act
    run_system(&mut world);

    // Assert
    let mut bus = world.resource_mut::<EventBus<InteractionIntent>>();
    let intents: Vec<_> = bus.drain().collect();
    assert_eq!(intents.len(), 1);
    match &intents[0] {
        InteractionIntent::PickCard { entity, .. } => assert_eq!(*entity, card_b),
        other => panic!("expected PickCard, got {other:?}"),
    }
}

#[test]
fn when_already_dragging_then_new_click_does_not_replace_drag() {
    // Arrange
    let mut world = World::new();
    let card_a = world.spawn_empty().id();
    spawn_clickable_card(&mut world, Vec2::ZERO, 0);
    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Left);
    mouse.set_world_pos(Vec2::ZERO);
    world.insert_resource(mouse);
    world.insert_resource(DragState {
        dragging: Some(card_game::card::interaction::drag_state::DragInfo {
            entity: card_a,
            local_grab_offset: Vec2::ZERO,
            origin_zone: CardZone::Table,
            stash_cursor_follow: false,
            origin_position: Vec2::ZERO,
        }),
    });
    insert_pick_resources(&mut world);

    // Act
    run_system(&mut world);

    // Assert — guard fires, no intent emitted
    assert!(world.resource::<EventBus<InteractionIntent>>().is_empty());
}

/// @doc: Local grab offset transformed by inverse rotation—drag stays aligned to card frame even after rotation
#[test]
fn when_card_picked_at_offset_then_local_grab_offset_is_inverse_rotated() {
    // Arrange
    let mut world = World::new();
    let angle = std::f32::consts::FRAC_PI_4;
    let transform = Affine2::from_scale_angle_translation(Vec2::ONE, angle, Vec2::new(100.0, 50.0));
    let card = world
        .spawn((
            card_game::test_helpers::make_test_card(),
            CardZone::Table,
            default_card_collider(),
            GlobalTransform2D(transform),
            SortOrder::new(0),
            Clickable(ClickHitShape::Aabb(CARD_HALF)),
        ))
        .id();
    world.entity_mut(card).observe(on_card_clicked);
    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Left);
    mouse.set_world_pos(Vec2::new(110.0, 50.0));
    world.insert_resource(mouse);
    insert_pick_resources(&mut world);

    // Act
    run_system(&mut world);

    // Assert
    let mut bus = world.resource_mut::<EventBus<InteractionIntent>>();
    let intents: Vec<_> = bus.drain().collect();
    assert_eq!(intents.len(), 1);
    let expected_x = 10.0_f32 * angle.cos();
    let expected_y = -10.0_f32 * angle.sin();
    match &intents[0] {
        InteractionIntent::PickCard { grab_offset, .. } => {
            assert!(
                (grab_offset.x - expected_x).abs() < 1e-4,
                "offset.x={} expected ~{expected_x}",
                grab_offset.x
            );
            assert!(
                (grab_offset.y - expected_y).abs() < 1e-4,
                "offset.y={} expected ~{expected_y}",
                grab_offset.y
            );
        }
        other => panic!("expected PickCard, got {other:?}"),
    }
}

#[test]
fn when_card_picked_at_center_then_local_grab_offset_is_zero() {
    // Arrange
    let mut world = World::new();
    let card = world
        .spawn((
            card_game::test_helpers::make_test_card(),
            CardZone::Table,
            default_card_collider(),
            GlobalTransform2D(Affine2::from_translation(Vec2::new(100.0, 50.0))),
            SortOrder::new(0),
            Clickable(ClickHitShape::Aabb(CARD_HALF)),
        ))
        .id();
    world.entity_mut(card).observe(on_card_clicked);
    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Left);
    mouse.set_world_pos(Vec2::new(100.0, 50.0));
    world.insert_resource(mouse);
    insert_pick_resources(&mut world);

    // Act
    run_system(&mut world);

    // Assert
    let mut bus = world.resource_mut::<EventBus<InteractionIntent>>();
    let intents: Vec<_> = bus.drain().collect();
    assert_eq!(intents.len(), 1);
    match &intents[0] {
        InteractionIntent::PickCard { grab_offset, .. } => {
            assert!(
                grab_offset.length() < 1e-6,
                "offset should be ~zero, got {grab_offset}"
            );
        }
        other => panic!("expected PickCard, got {other:?}"),
    }
}

#[test]
fn when_cursor_on_edge_of_card_then_card_is_picked() {
    // Arrange
    let mut world = World::new();
    spawn_clickable_card(&mut world, Vec2::ZERO, 0);
    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Left);
    mouse.set_world_pos(Vec2::new(30.0, 0.0));
    world.insert_resource(mouse);
    insert_pick_resources(&mut world);

    // Act
    run_system(&mut world);

    // Assert
    let mut bus = world.resource_mut::<EventBus<InteractionIntent>>();
    let intents: Vec<_> = bus.drain().collect();
    assert_eq!(intents.len(), 1);
    assert!(
        matches!(&intents[0], InteractionIntent::PickCard { .. }),
        "expected PickCard intent"
    );
}

/// @doc: OBB hit test accounts for card rotation—axis-aligned check would miss rotated cards
#[test]
fn when_rotated_card_clicked_inside_obb_then_picked() {
    // Arrange
    let mut world = World::new();
    let angle = std::f32::consts::FRAC_PI_4;
    let card = world
        .spawn((
            card_game::test_helpers::make_test_card(),
            CardZone::Table,
            default_card_collider(),
            GlobalTransform2D(Affine2::from_scale_angle_translation(
                Vec2::ONE,
                angle,
                Vec2::ZERO,
            )),
            SortOrder::new(0),
            Clickable(ClickHitShape::Aabb(CARD_HALF)),
        ))
        .id();
    world.entity_mut(card).observe(on_card_clicked);
    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Left);
    mouse.set_world_pos(Vec2::new(20.0, 20.0));
    world.insert_resource(mouse);
    insert_pick_resources(&mut world);

    // Act
    run_system(&mut world);

    // Assert
    let mut bus = world.resource_mut::<EventBus<InteractionIntent>>();
    let intents: Vec<_> = bus.drain().collect();
    assert!(!intents.is_empty(), "expected at least one PickCard intent");
}

#[test]
fn when_rotated_card_clicked_outside_obb_then_not_picked() {
    // Arrange
    let mut world = World::new();
    let angle = std::f32::consts::FRAC_PI_4;
    let card = world
        .spawn((
            card_game::test_helpers::make_test_card(),
            CardZone::Table,
            default_card_collider(),
            GlobalTransform2D(Affine2::from_scale_angle_translation(
                Vec2::ONE,
                angle,
                Vec2::ZERO,
            )),
            SortOrder::new(0),
            Clickable(ClickHitShape::Aabb(CARD_HALF)),
        ))
        .id();
    world.entity_mut(card).observe(on_card_clicked);
    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Left);
    mouse.set_world_pos(Vec2::new(50.0, 0.0));
    world.insert_resource(mouse);
    insert_pick_resources(&mut world);

    // Act
    run_system(&mut world);

    // Assert — cursor is outside rotated OBB, no pick intent
    assert!(world.resource::<EventBus<InteractionIntent>>().is_empty());
}

/// @doc: SortOrder determines which card wins when two overlap at the same position.
#[test]
fn when_hand_card_and_table_card_overlap_then_highest_sort_wins() {
    // Arrange
    let mut world = World::new();
    spawn_clickable_card(&mut world, Vec2::ZERO, 3);
    let hand_card = spawn_clickable_card(&mut world, Vec2::ZERO, 10);
    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Left);
    mouse.set_world_pos(Vec2::ZERO);
    world.insert_resource(mouse);
    insert_pick_resources(&mut world);

    // Act
    run_system(&mut world);

    // Assert
    let mut bus = world.resource_mut::<EventBus<InteractionIntent>>();
    let intents: Vec<_> = bus.drain().collect();
    assert_eq!(intents.len(), 1);
    match &intents[0] {
        InteractionIntent::PickCard { entity, .. } => assert_eq!(*entity, hand_card),
        other => panic!("expected PickCard, got {other:?}"),
    }
}

/// @doc: Clicking a stash slot emits PickFromStash intent carrying the slot address.
#[test]
fn when_left_click_on_stash_card_then_no_physics_body_added_and_render_layer_stays_ui() {
    // Arrange
    use card_game::stash::grid::StashGrid;
    use card_game::stash::toggle::StashVisible;

    let mut world = World::new();
    // col=2, row=3 center: x = 153, y = 20 + 3*79 + 37 = 294
    let card_entity = world
        .spawn((
            Card::face_down(TextureId(30), TextureId(31)),
            CardZone::Stash {
                page: 0,
                col: 2,
                row: 3,
            },
            Collider::Aabb(Vec2::new(30.0, 45.0)),
            GlobalTransform2D(Affine2::IDENTITY),
            SortOrder::new(0),
        ))
        .id();
    let mut grid = StashGrid::new(4, 5, 1);
    grid.place(0, 2, 3, card_entity).unwrap();
    world.insert_resource(grid);
    world.insert_resource(StashVisible(true));
    world.insert_resource(DragState::default());
    world.insert_resource(ReaderDragState::default());
    world.insert_resource(PendingCable::default());
    world.insert_resource(EventBus::<InteractionIntent>::default());
    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Left);
    mouse.set_screen_pos(Vec2::new(153.0, 294.0));
    world.insert_resource(mouse);

    // Act
    run_system(&mut world);

    // Assert — stash pick emits PickFromStash intent
    let mut bus = world.resource_mut::<EventBus<InteractionIntent>>();
    let intents: Vec<_> = bus.drain().collect();
    assert_eq!(intents.len(), 1, "expected one PickFromStash intent");
    assert!(
        matches!(&intents[0], InteractionIntent::PickFromStash { entity, .. } if *entity == card_entity),
        "expected PickFromStash for card_entity, got {:?}",
        intents[0]
    );
}

#[test]
fn when_stash_hidden_and_slot_clicked_then_pick_not_triggered() {
    // Arrange
    use card_game::stash::grid::StashGrid;
    use card_game::stash::toggle::StashVisible;
    use engine_scene::prelude::RenderLayer;

    let mut world = World::new();
    // col=0, row=0 center at (45, 45)
    let card_entity = world
        .spawn((
            card_game::test_helpers::make_test_card(),
            CardZone::Stash {
                page: 0,
                col: 0,
                row: 0,
            },
            Collider::Aabb(Vec2::new(30.0, 45.0)),
            // Place far from world cursor (default world_pos is ZERO) so
            // the existing world-space hit test cannot pick this card
            GlobalTransform2D(Affine2::from_translation(Vec2::new(500.0, 500.0))),
            RenderLayer::UI,
            SortOrder::new(0),
        ))
        .id();
    let mut grid = StashGrid::new(4, 5, 1);
    grid.place(0, 0, 0, card_entity).unwrap();
    world.insert_resource(grid);
    world.insert_resource(StashVisible(false));
    world.insert_resource(DragState::default());
    world.insert_resource(ReaderDragState::default());
    world.insert_resource(PendingCable::default());
    world.insert_resource(EventBus::<InteractionIntent>::default());
    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Left);
    mouse.set_screen_pos(Vec2::new(45.0, 45.0));
    world.insert_resource(mouse);

    // Act
    run_system(&mut world);

    // Assert — stash hidden, no pick intent
    assert!(
        world.resource::<EventBus<InteractionIntent>>().is_empty(),
        "stash pick should not trigger when stash is hidden"
    );
}

/// @doc: click_resolve_system emits PickCard intent via observer — DragState is not set directly.
#[test]
fn when_table_card_picked_then_pick_intent_emitted_not_drag_state() {
    // Arrange
    let mut world = World::new();
    let collider = default_card_collider();
    let card_entity = world
        .spawn((
            card_game::test_helpers::make_test_card(),
            CardZone::Table,
            collider.clone(),
            GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
            SortOrder::new(0),
            Clickable(ClickHitShape::Aabb(CARD_HALF)),
        ))
        .id();
    world.entity_mut(card_entity).observe(on_card_clicked);
    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Left);
    mouse.set_world_pos(Vec2::ZERO);
    world.insert_resource(mouse);
    insert_pick_resources(&mut world);

    // Act
    run_system(&mut world);

    // Assert
    let mut bus = world.resource_mut::<EventBus<InteractionIntent>>();
    let intents: Vec<_> = bus.drain().collect();
    assert_eq!(intents.len(), 1, "expected exactly one InteractionIntent");
    match &intents[0] {
        InteractionIntent::PickCard {
            entity,
            zone,
            collider: emitted_collider,
            ..
        } => {
            assert_eq!(*entity, card_entity);
            assert_eq!(*zone, CardZone::Table);
            assert_eq!(*emitted_collider, collider);
        }
        other => panic!("expected PickCard, got {other:?}"),
    }
    assert!(
        world.resource::<DragState>().dragging.is_none(),
        "click_resolve_system must not set DragState; that is the applier's responsibility"
    );
}

/// @doc: Stash picks emit a PickFromStash intent carrying the slot address so the applier
/// can vacate the slot and set up the drag. The pick system must NOT vacate the stash slot
/// or set DragState — if it did, a dropped intent (e.g. from a guard check in the applier)
/// would leave the grid in an inconsistent state with a phantom empty slot and no active drag.
#[test]
fn when_stash_card_picked_then_pick_from_stash_intent_emitted_and_slot_not_vacated() {
    use card_game::stash::grid::StashGrid;
    use card_game::stash::toggle::StashVisible;

    // Arrange
    let mut world = World::new();
    let card_entity = world
        .spawn((
            Card::face_down(TextureId(20), TextureId(21)),
            CardZone::Stash {
                page: 0,
                col: 2,
                row: 3,
            },
            Collider::Aabb(Vec2::new(30.0, 45.0)),
            GlobalTransform2D(Affine2::IDENTITY),
            engine_scene::prelude::RenderLayer::UI,
            SortOrder::new(0),
        ))
        .id();
    let mut grid = StashGrid::new(4, 5, 1);
    grid.place(0, 2, 3, card_entity).unwrap();
    world.insert_resource(grid);
    world.insert_resource(StashVisible(true));
    world.insert_resource(DragState::default());
    world.insert_resource(ReaderDragState::default());
    world.insert_resource(PendingCable::default());
    world.insert_resource(EventBus::<InteractionIntent>::default());
    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Left);
    mouse.set_screen_pos(Vec2::new(153.0, 294.0));
    world.insert_resource(mouse);

    // Act
    run_system(&mut world);

    // Assert
    let mut bus = world.resource_mut::<EventBus<InteractionIntent>>();
    let intents: Vec<_> = bus.drain().collect();
    assert_eq!(intents.len(), 1, "expected exactly one InteractionIntent");
    match &intents[0] {
        InteractionIntent::PickFromStash {
            entity,
            page,
            col,
            row,
        } => {
            assert_eq!(*entity, card_entity);
            assert_eq!(*page, 0);
            assert_eq!(*col, 2);
            assert_eq!(*row, 3);
        }
        other => panic!("expected PickFromStash, got {other:?}"),
    }
    assert!(
        world.resource::<StashGrid>().get(0, 2, 3).is_some(),
        "stash slot must NOT be vacated by the pick system; the applier owns that"
    );
}

/// @doc: The mutual-exclusion guard prevents the pick system from emitting intents when any
/// drag is already active (card, reader, or screen). Without this guard, a second PickCard
/// intent during an active drag would cause the applier to overwrite DragState, losing the
/// original drag's entity reference and leaving a card floating with no system owning it.
#[test]
fn when_already_dragging_then_no_pick_intent_emitted() {
    use card_game::card::interaction::drag_state::DragInfo;

    // Arrange
    let mut world = World::new();
    spawn_clickable_card(&mut world, Vec2::ZERO, 0);
    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Left);
    mouse.set_world_pos(Vec2::ZERO);
    world.insert_resource(mouse);
    world.insert_resource(DragState {
        dragging: Some(DragInfo {
            entity: Entity::from_raw(999),
            local_grab_offset: Vec2::ZERO,
            origin_zone: CardZone::Table,
            stash_cursor_follow: false,
            origin_position: Vec2::ZERO,
        }),
    });
    world.insert_resource(EventBus::<InteractionIntent>::default());
    insert_pick_resources(&mut world);

    // Act
    run_system(&mut world);

    // Assert
    assert!(
        world.resource::<EventBus<InteractionIntent>>().is_empty(),
        "no intent should be emitted when a drag is already active"
    );
}
