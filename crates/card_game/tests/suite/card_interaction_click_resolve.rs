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

fn make_mouse_pressed_at(pos: Vec2) -> MouseState {
    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Left);
    mouse.set_world_pos(pos);
    mouse
}

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

/// Calling `spawn_visual_card` attaches `Clickable` and the `on_card_clicked` observer,
/// so clicking the card at its center produces a `PickCard` intent.
#[test]
fn when_card_spawned_via_spawn_visual_card_then_click_resolves_to_pick_card() {
    use card_game::card::identity::definition::{
        CardAbilities, CardDefinition, CardType, art_descriptor_default,
    };
    use card_game::card::rendering::geometry::{TABLE_CARD_HEIGHT, TABLE_CARD_WIDTH};
    use card_game::card::rendering::spawn_table_card::spawn_visual_card;

    // Arrange
    let mut world = World::new();
    insert_base_resources(&mut world);
    // spawn_visual_card needs PhysicsCommand bus (used optionally)
    world.insert_resource(EventBus::<engine_physics::prelude::PhysicsCommand>::default());

    let pos = Vec2::ZERO;
    let def = CardDefinition {
        card_type: CardType::Spell,
        name: "Test".to_owned(),
        stats: None,
        abilities: CardAbilities { keywords: vec![], text: String::new() },
        art: art_descriptor_default(CardType::Spell),
    };
    let card = spawn_visual_card(
        &mut world,
        &def,
        pos,
        Vec2::new(TABLE_CARD_WIDTH, TABLE_CARD_HEIGHT),
        true,
        Default::default(),
    );

    // transform_propagation runs in LateUpdate — set it manually for the test
    world
        .entity_mut(card)
        .insert(GlobalTransform2D(Affine2::from_translation(pos)));

    world.insert_resource(make_mouse_pressed_at(pos));

    // Act
    run_click_resolve(&mut world);

    // Assert
    let mut bus = world.resource_mut::<EventBus<InteractionIntent>>();
    let intents: Vec<_> = bus.drain().collect();
    assert_eq!(intents.len(), 1, "expected one PickCard intent");
    assert!(
        matches!(&intents[0], InteractionIntent::PickCard { entity, .. } if *entity == card),
        "expected PickCard for spawned card, got {:?}",
        intents[0]
    );
}

/// Clicking a reader (no card on top) starts reader drag via the observer.
#[test]
fn when_reader_clicked_alone_then_reader_drag_starts() {
    use card_game::card::reader::spawn::spawn_reader;

    // Arrange
    let mut world = World::new();
    insert_base_resources(&mut world);
    world.insert_resource(EventBus::<engine_physics::prelude::PhysicsCommand>::default());

    let pos = Vec2::new(200.0, 200.0);
    let (reader_entity, _jack_entity) = spawn_reader(&mut world, pos);

    world
        .entity_mut(reader_entity)
        .insert(GlobalTransform2D(Affine2::from_translation(pos)));
    world.insert_resource(make_mouse_pressed_at(pos));

    // Act
    run_click_resolve(&mut world);

    // Assert
    let reader_drag = world.resource::<card_game::card::reader::ReaderDragState>();
    assert!(reader_drag.dragging.is_some(), "reader drag should start");
    assert_eq!(
        reader_drag.dragging.as_ref().unwrap().entity,
        reader_entity
    );
}

/// Clicking a JackSocket (from `spawn_screen_device`) sets `PendingCable.source`.
#[test]
fn when_socket_clicked_then_pending_cable_source_set() {
    use card_game::card::screen_device::spawn_screen_device;

    // Arrange
    let mut world = World::new();
    insert_base_resources(&mut world);
    world.insert_resource(EventBus::<engine_physics::prelude::PhysicsCommand>::default());

    let pos = Vec2::new(50.0, 50.0);
    let (device_entity, jack_entity) = spawn_screen_device(&mut world, pos);

    // socket is offset from device body
    let socket_pos = pos + Vec2::new(129.0, 0.0);
    world
        .entity_mut(jack_entity)
        .insert(GlobalTransform2D(Affine2::from_translation(socket_pos)))
        .insert(SortOrder::new(3));
    world
        .entity_mut(device_entity)
        .insert(GlobalTransform2D(Affine2::from_translation(pos)))
        .insert(SortOrder::new(1));

    world.insert_resource(make_mouse_pressed_at(socket_pos));

    // Act
    run_click_resolve(&mut world);

    // Assert
    let pending = world.resource::<card_game::card::jack_socket::PendingCable>();
    assert_eq!(pending.source, Some(jack_entity));
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
