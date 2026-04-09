#![allow(clippy::unwrap_used)]

use std::sync::{Arc, Mutex};

use bevy_ecs::prelude::*;
use engine_input::prelude::{InputState, KeyCode, MouseState};
use engine_render::prelude::{Camera2D, RendererRes};
use engine_render::testing::{ColoredMeshCallLog, SpyRenderer};
use glam::Vec2;

use card_game::card::interaction::drag_state::DragState;
use card_game::stash::grid::StashGrid;
use card_game::stash::hover::{
    ORBIT_AMPLITUDE, StashHoverPreview, lissajous_offset, stash_hover_preview_render_system,
    stash_hover_preview_system,
};
use card_game::stash::toggle::StashVisible;

fn run_system(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(stash_hover_preview_system);
    schedule.run(world);
}

fn run_render_system(world: &mut World) {
    world.insert_resource(engine_ui::draw_command::DrawQueue::default());
    let mut schedule = Schedule::default();
    schedule.add_systems(stash_hover_preview_render_system);
    schedule.run(world);
    // Drain the DrawQueue through unified_render to produce spy draw calls
    let mut render_schedule = Schedule::default();
    render_schedule.add_systems(engine_ui::unified_render::unified_render_system);
    render_schedule.run(world);
}

fn make_world_with_occupied_slot() -> (World, Entity) {
    use engine_core::prelude::{DeltaTime, Seconds};

    let mut world = World::new();

    let card_entity = world.spawn_empty().id();
    let mut grid = StashGrid::new(10, 10, 1);
    grid.place(0, 0, 0, card_entity).unwrap();

    let log = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(log).with_viewport(1024, 768);
    world.insert_resource(RendererRes::new(Box::new(spy)));
    world.spawn(Camera2D {
        position: Vec2::ZERO,
        zoom: 1.0,
    });

    world.insert_resource(grid);
    world.insert_resource(DragState::default());
    world.insert_resource(StashHoverPreview::default());
    world.insert_resource(DeltaTime(Seconds(0.016)));

    (world, card_entity)
}

fn make_world_with_spy(grid: StashGrid) -> (World, ColoredMeshCallLog) {
    use engine_core::prelude::{DeltaTime, Seconds};

    let mut world = World::new();
    let mesh_log: ColoredMeshCallLog = Arc::new(Mutex::new(Vec::new()));
    let log = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(log)
        .with_colored_mesh_capture(mesh_log.clone())
        .with_viewport(1024, 768);
    world.insert_resource(RendererRes::new(Box::new(spy)));
    world.spawn(Camera2D {
        position: Vec2::ZERO,
        zoom: 1.0,
    });
    world.insert_resource(grid);
    world.insert_resource(DragState::default());
    world.insert_resource(StashHoverPreview::default());
    world.insert_resource(DeltaTime(Seconds(0.016)));
    (world, mesh_log)
}

fn all_conditions_met(world: &mut World) {
    let mut input = InputState::default();
    input.press(KeyCode::ControlLeft);
    let mut mouse = MouseState::default();
    mouse.set_screen_pos(Vec2::new(45.0, 45.0));
    world.insert_resource(StashVisible(true));
    world.insert_resource(input);
    world.insert_resource(mouse);
}

// -- Trigger condition tests (update system) --------------------------

/// @doc: Hover preview requires three conditions: stash visible, Ctrl held, no active drag — ensures preview doesn't interfere with card movement.
#[test]
fn when_ctrl_held_and_cursor_over_occupied_slot_and_no_drag_then_hovered_entity_set() {
    // Arrange
    let (mut world, card_entity) = make_world_with_occupied_slot();
    all_conditions_met(&mut world);

    // Act
    run_system(&mut world);

    // Assert
    let preview = world.resource::<StashHoverPreview>();
    assert_eq!(preview.hovered_entity, Some(card_entity));
}

#[test]
fn when_stash_hidden_then_hovered_entity_none() {
    // Arrange
    let (mut world, _) = make_world_with_occupied_slot();
    all_conditions_met(&mut world);
    world.insert_resource(StashVisible(false));

    // Act
    run_system(&mut world);

    // Assert
    assert!(
        world
            .resource::<StashHoverPreview>()
            .hovered_entity
            .is_none()
    );
}

/// @doc: Ctrl key acts as preview toggle — requires explicit modkey to avoid accidental preview pop-ups during normal play.
#[test]
fn when_no_ctrl_pressed_then_hovered_entity_none() {
    // Arrange
    let (mut world, _) = make_world_with_occupied_slot();
    all_conditions_met(&mut world);
    world.insert_resource(InputState::default());

    // Act
    run_system(&mut world);

    // Assert
    assert!(
        world
            .resource::<StashHoverPreview>()
            .hovered_entity
            .is_none()
    );
}

#[test]
fn when_ctrl_right_pressed_then_hovered_entity_set() {
    // Arrange
    let (mut world, card_entity) = make_world_with_occupied_slot();
    all_conditions_met(&mut world);
    let mut input = InputState::default();
    input.press(KeyCode::ControlRight);
    world.insert_resource(input);

    // Act
    run_system(&mut world);

    // Assert
    assert_eq!(
        world.resource::<StashHoverPreview>().hovered_entity,
        Some(card_entity)
    );
}

#[test]
fn when_cursor_over_empty_slot_then_hovered_entity_none() {
    // Arrange
    let (mut world, _) = make_world_with_occupied_slot();
    all_conditions_met(&mut world);
    let mut mouse = MouseState::default();
    mouse.set_screen_pos(Vec2::new(
        45.0 + card_game::stash::constants::SLOT_STRIDE_W,
        45.0,
    ));
    world.insert_resource(mouse);

    // Act
    run_system(&mut world);

    // Assert
    assert!(
        world
            .resource::<StashHoverPreview>()
            .hovered_entity
            .is_none()
    );
}

#[test]
fn when_cursor_outside_stash_area_then_hovered_entity_none() {
    // Arrange
    let (mut world, _) = make_world_with_occupied_slot();
    all_conditions_met(&mut world);
    let mut mouse = MouseState::default();
    mouse.set_screen_pos(Vec2::new(800.0, 400.0));
    world.insert_resource(mouse);

    // Act
    run_system(&mut world);

    // Assert
    assert!(
        world
            .resource::<StashHoverPreview>()
            .hovered_entity
            .is_none()
    );
}

/// @doc: Dragging suppresses hover preview — prevents confusing double-rendering of card being held and its preview simultaneously.
#[test]
fn when_drag_active_then_hovered_entity_none() {
    // Arrange
    let (mut world, card_entity) = make_world_with_occupied_slot();
    all_conditions_met(&mut world);
    world.insert_resource(DragState {
        dragging: Some(card_game::card::interaction::drag_state::DragInfo {
            entity: card_entity,
            local_grab_offset: Vec2::ZERO,
            origin_zone: card_game::card::component::CardZone::Table,
            stash_cursor_follow: false,
            origin_position: Vec2::ZERO,
        }),
    });

    // Act
    run_system(&mut world);

    // Assert
    assert!(
        world
            .resource::<StashHoverPreview>()
            .hovered_entity
            .is_none()
    );
}

#[test]
fn when_conditions_fail_after_hovering_then_hovered_entity_cleared() {
    // Arrange — first frame: hover
    let (mut world, card_entity) = make_world_with_occupied_slot();
    all_conditions_met(&mut world);
    run_system(&mut world);
    assert_eq!(
        world.resource::<StashHoverPreview>().hovered_entity,
        Some(card_entity)
    );

    // Arrange — second frame: release Ctrl
    world.insert_resource(InputState::default());

    // Act
    run_system(&mut world);

    // Assert
    assert!(
        world
            .resource::<StashHoverPreview>()
            .hovered_entity
            .is_none()
    );
}

// -- Render system tests ----------------------------------------------

use card_game::card::identity::definition::{
    CardAbilities, CardDefinition, CardType, art_descriptor_default,
};
use card_game::card::rendering::spawn_table_card::spawn_visual_card;

fn make_test_def() -> CardDefinition {
    CardDefinition {
        card_type: CardType::Spell,
        name: "Fireball".to_owned(),
        stats: None,
        abilities: CardAbilities {
            keywords: vec![],
            text: "Deal 3 damage".to_owned(),
        },
        art: art_descriptor_default(CardType::Spell),
    }
}

fn spawn_card_in_world(world: &mut World) -> Entity {
    use card_game::card::identity::signature::CardSignature;
    spawn_visual_card(
        world,
        &make_test_def(),
        Vec2::ZERO,
        Vec2::new(60.0, 90.0),
        true,
        CardSignature::default(),
    )
}

#[test]
fn when_hovered_entity_none_then_no_colored_mesh_drawn() {
    // Arrange
    let grid = StashGrid::new(10, 10, 1);
    let (mut world, mesh_log) = make_world_with_spy(grid);

    // Act
    run_render_system(&mut world);

    // Assert
    let calls = mesh_log.lock().unwrap();
    assert!(
        calls.is_empty(),
        "no mesh should be drawn when not hovering"
    );
}

/// @doc: Hover preview renders baked card geometry — validates the render pipeline correctly projects stash cards to corner of screen.
#[test]
fn when_hovered_entity_set_then_baked_front_mesh_drawn() {
    // Arrange
    let grid = StashGrid::new(10, 10, 1);
    let (mut world, mesh_log) = make_world_with_spy(grid);
    let card = spawn_card_in_world(&mut world);
    world
        .resource_mut::<StashGrid>()
        .place(0, 0, 0, card)
        .unwrap();
    world.resource_mut::<StashHoverPreview>().hovered_entity = Some(card);

    // Act
    run_render_system(&mut world);

    // Assert — one draw_colored_mesh call for the baked front face
    let calls = mesh_log.lock().unwrap();
    assert_eq!(
        calls.len(),
        1,
        "expected exactly 1 draw_colored_mesh call for baked card"
    );
}

#[test]
fn when_viewport_zero_then_no_mesh_drawn() {
    use engine_core::prelude::{DeltaTime, Seconds};

    // Arrange
    let mut world = World::new();
    let log = Arc::new(Mutex::new(Vec::new()));
    let mesh_log: ColoredMeshCallLog = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(log)
        .with_colored_mesh_capture(mesh_log.clone())
        .with_viewport(0, 0);
    world.insert_resource(RendererRes::new(Box::new(spy)));
    world.spawn(Camera2D::default());
    world.insert_resource(StashGrid::new(10, 10, 1));
    world.insert_resource(DragState::default());
    world.insert_resource(DeltaTime(Seconds(0.016)));
    let card = spawn_card_in_world(&mut world);
    world.insert_resource(StashHoverPreview {
        hovered_entity: Some(card),
    });

    // Act
    run_render_system(&mut world);

    // Assert
    let calls = mesh_log.lock().unwrap();
    assert!(calls.is_empty());
}

/// @doc: The Lissajous orbit must return zero at t=0 so the fake cursor starts at
/// the card center, and must stay within the card's half-extents scaled by the
/// amplitude factor — preventing the shader highlight from drifting off-card.
#[test]
fn when_lissajous_at_time_zero_then_offset_is_zero() {
    // Act
    let offset = lissajous_offset(0.0, 100.0, 150.0);

    // Assert
    assert!(
        offset.x.abs() < 1e-6 && offset.y.abs() < 1e-6,
        "Lissajous must start at origin, got {offset:?}"
    );
}

#[test]
fn when_lissajous_at_any_time_then_within_amplitude_bounds() {
    // Arrange — sample many time points
    let half_w = 100.0;
    let half_h = 150.0;
    let max_x = half_w * ORBIT_AMPLITUDE;
    let max_y = half_h * ORBIT_AMPLITUDE;

    // Act + Assert
    for i in 0..1000 {
        let t = i as f32 * 0.01;
        let offset = lissajous_offset(t, half_w, half_h);
        assert!(
            offset.x.abs() <= max_x + 1e-4 && offset.y.abs() <= max_y + 1e-4,
            "Lissajous out of bounds at t={t}: {offset:?}, max=({max_x}, {max_y})"
        );
    }
}
