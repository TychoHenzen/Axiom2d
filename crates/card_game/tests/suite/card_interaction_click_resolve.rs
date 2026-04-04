#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::*;
use engine_core::prelude::EventBus;
use engine_input::prelude::{MouseButton, MouseState};
use engine_physics::prelude::Collider;
use engine_scene::prelude::{GlobalTransform2D, SortOrder};
use glam::{Affine2, Vec2};

use card_game::card::component::CardZone;
use card_game::card::interaction::click_resolve::{
    ClickHitShape, Clickable, click_resolve_system, on_card_clicked,
};
use card_game::card::interaction::drag_state::DragState;
use card_game::card::interaction::intent::InteractionIntent;
use card_game::card::jack_socket::PendingCable;
use card_game::card::reader::ReaderDragState;

fn run_click_resolve(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(click_resolve_system);
    schedule.run(world);
}

fn insert_base_resources(world: &mut World) {
    if world.get_resource::<DragState>().is_none() {
        world.insert_resource(DragState::default());
    }
    if world.get_resource::<ReaderDragState>().is_none() {
        world.insert_resource(ReaderDragState::default());
    }
    if world.get_resource::<PendingCable>().is_none() {
        world.insert_resource(PendingCable::default());
    }
    if world
        .get_resource::<EventBus<InteractionIntent>>()
        .is_none()
    {
        world.insert_resource(EventBus::<InteractionIntent>::default());
    }
}

/// @doc: When a card and reader overlap at the same position, the entity with higher
/// SortOrder wins click resolution — topmost entity gets the trigger.
#[test]
fn when_card_and_reader_overlap_then_topmost_card_picked() {
    // Arrange
    let mut world = World::new();

    // Spawn a reader entity at the same position with lower SortOrder
    let half = Vec2::new(30.0, 45.0);
    let _reader = world
        .spawn((
            Clickable(ClickHitShape::Aabb(half)),
            GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
            SortOrder::new(1),
        ))
        .id();

    // Spawn a card entity at the same position with higher SortOrder
    let card = world
        .spawn((
            card_game::test_helpers::make_test_card(),
            CardZone::Table,
            Collider::Aabb(half),
            Clickable(ClickHitShape::Aabb(half)),
            GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
            SortOrder::new(5),
        ))
        .id();

    // Register on_card_clicked observer on the card entity
    world.entity_mut(card).observe(on_card_clicked);

    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Left);
    mouse.set_world_pos(Vec2::ZERO);
    world.insert_resource(mouse);
    insert_base_resources(&mut world);

    // Act — schedule runs and flushes commands (including trigger_targets)
    run_click_resolve(&mut world);

    // Assert — exactly one PickCard intent for the card entity
    let mut bus = world.resource_mut::<EventBus<InteractionIntent>>();
    let intents: Vec<_> = bus.drain().collect();
    assert_eq!(intents.len(), 1, "expected exactly one PickCard intent");
    match &intents[0] {
        InteractionIntent::PickCard { entity, .. } => {
            assert_eq!(*entity, card, "topmost card must win click resolution");
        }
        other => panic!("expected PickCard, got {other:?}"),
    }

    // Reader drag state stays untouched
    assert!(
        world.resource::<ReaderDragState>().dragging.is_none(),
        "reader drag state must remain None"
    );
}

/// @doc: When no Clickable entity is under the cursor, no intent is emitted.
#[test]
fn when_no_clickable_entity_under_cursor_then_no_intent() {
    // Arrange
    let mut world = World::new();
    // No clickable entities spawned

    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Left);
    mouse.set_world_pos(Vec2::new(100.0, 100.0));
    world.insert_resource(mouse);
    insert_base_resources(&mut world);

    // Act
    run_click_resolve(&mut world);

    // Assert
    assert!(
        world.resource::<EventBus<InteractionIntent>>().is_empty(),
        "no intent should be emitted when no Clickable entity is under cursor"
    );
}
