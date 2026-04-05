#![allow(clippy::unwrap_used)]

use std::sync::{Arc, Mutex};

use bevy_ecs::prelude::*;
use card_game::card::component::CardZone;
use card_game::card::identity::signature::{CardSignature, Element};
use card_game::card::interaction::drag_state::{DragInfo, DragState};
use card_game::card::jack_cable::{Cable, Jack, JackDirection, signature_space_propagation_system};
use card_game::card::jack_socket::PendingCable;
use card_game::card::reader::{ReaderDragState, SIGNATURE_SPACE_RADIUS, SignatureSpace};
use card_game::card::screen_device::{
    ScreenDragState, display_axes, screen_pick_system, screen_render_system, spawn_screen_device,
};
use card_game::stash::grid::StashGrid;
use card_game::stash::toggle::StashVisible;
use engine_core::prelude::Transform2D;
use engine_input::prelude::{MouseButton, MouseState};
use engine_render::prelude::{Camera2D, RendererRes};
use engine_render::testing::{ShapeCallLog, SpyRenderer};
use engine_scene::prelude::{
    hierarchy_maintenance_system, transform_propagation_system, visibility_system,
};
use engine_ui::unified_render::unified_render_system;
use glam::Vec2;

// ---------------------------------------------------------------------------
// Shared helper
// ---------------------------------------------------------------------------

fn make_screen_world(jack_data: Option<SignatureSpace>) -> (World, ShapeCallLog) {
    let mut world = World::new();
    let (_device_entity, jack_entity) = spawn_screen_device(&mut world, Vec2::ZERO);
    world
        .get_mut::<Jack<SignatureSpace>>(jack_entity)
        .unwrap()
        .data = jack_data;

    let shape_calls: ShapeCallLog = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(Arc::new(Mutex::new(Vec::new())))
        .with_shape_capture(shape_calls.clone())
        .with_viewport(1024, 768);
    world.insert_resource(RendererRes::new(Box::new(spy)));
    world.spawn(Camera2D {
        position: Vec2::ZERO,
        zoom: 1.0,
    });

    (world, shape_calls)
}

fn run_screen_visuals(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(
        (
            hierarchy_maintenance_system,
            screen_render_system,
            transform_propagation_system,
            visibility_system,
            unified_render_system,
        )
            .chain(),
    );
    schedule.run(world);
}

// ---------------------------------------------------------------------------
// TC010 / TC011 — axis-pair extraction
// ---------------------------------------------------------------------------

/// @doc: Display 0 of the `ScreenDevice` maps to the Solidum/Febris element pair — the first two
/// dimensions of the 8D signature space. If `display_axes` returned the wrong pair (e.g. indices 2
/// and 3), a card hovering near the origin in Solidum/Febris space would appear on the wrong
/// panel, breaking the visual correspondence between the card's elemental identity and the display
/// that lights up. Verifying exact f32 values against the named Element variants catches any
/// index-offset bug at the axis-extraction layer before it reaches the rendering pipeline.
#[test]
fn when_display_index_is_zero_then_axes_map_to_solidum_and_febris() {
    // Arrange
    let center = CardSignature::new([0.3, 0.7, 0.1, 0.2, 0.4, 0.5, 0.6, 0.8]);
    let space = SignatureSpace {
        center,
        radius: SIGNATURE_SPACE_RADIUS,
    };

    // Act
    let (x, y) = display_axes(&space, 0);

    // Assert
    assert_eq!(x, space.center[Element::Solidum]);
    assert_eq!(y, space.center[Element::Febris]);
}

/// @doc: Display 3 maps to the Subsidium/Spatium element pair — the last two dimensions of the
/// 8D signature space. Testing the final panel alongside the first closes the bounds: if the
/// index formula `display_index * 2` is off by one in either direction, at least one of these
/// boundary tests will produce a wrong axis, catching fencepost errors that a single mid-range
/// test would miss. Together TC010 and TC011 bracket all four display panels with minimal
/// redundancy.
#[test]
fn when_display_index_is_three_then_axes_map_to_subsidium_and_spatium() {
    // Arrange
    let center = CardSignature::new([0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.9, -0.8]);
    let space = SignatureSpace {
        center,
        radius: SIGNATURE_SPACE_RADIUS,
    };

    // Act
    let (x, y) = display_axes(&space, 3);

    // Assert
    assert_eq!(x, space.center[Element::Subsidium]);
    assert_eq!(y, space.center[Element::Spatium]);
}

// ---------------------------------------------------------------------------
// TC012 — no draws when input jack has None
// ---------------------------------------------------------------------------

/// @doc: A `ScreenDevice` with no signal must still draw its chassis, stroke, socket, and
/// four panel backgrounds while hiding the four signal dots. Without this panel-only state
/// the display would vanish until a card is inserted, giving no indication that a connectable
/// device exists on the table.
#[test]
fn given_input_jack_data_is_none_when_screen_render_system_runs_then_only_panel_backgrounds_drawn()
{
    // Arrange
    let (mut world, shape_calls) = make_screen_world(None);

    // Act
    run_screen_visuals(&mut world);

    // Assert — body fill + body stroke + socket + 4 panels = 7 draw calls
    assert_eq!(
        shape_calls.lock().unwrap().len(),
        7,
        "disconnected screen must draw body, stroke, socket, and 4 panels with no signal dots"
    );
}

// ---------------------------------------------------------------------------
// TC013 — 4 draws when input jack has Some(SignatureSpace)
// ---------------------------------------------------------------------------

/// @doc: When a `SignatureSpace` arrives on the input jack, the screen must render the same
/// chassis/panel base plus 4 signal dots — 11 draw calls total including the body stroke and
/// socket. Fewer would leave some displays dark; more would duplicate visuals or invent
/// phantom signal markers.
#[test]
fn given_input_jack_has_signature_space_when_screen_render_system_runs_then_draws_eight_shapes() {
    // Arrange
    let signal = SignatureSpace {
        center: CardSignature::new([0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8]),
        radius: SIGNATURE_SPACE_RADIUS,
    };
    let (mut world, shape_calls) = make_screen_world(Some(signal));

    // Act
    run_screen_visuals(&mut world);

    // Assert — body fill + body stroke + socket + 4 panels + 4 dots
    assert_eq!(
        shape_calls.lock().unwrap().len(),
        11,
        "screen with active signal must draw body, stroke, socket, 4 panels, and 4 signal dots"
    );
}

// ---------------------------------------------------------------------------
// TC014 — full signal chain integration
// ---------------------------------------------------------------------------

/// @doc: This integration test validates the complete wiring chain from card insertion
/// to screen display: a card dropped into a reader populates the reader's output jack with
/// a `SignatureSpace`, the cable propagation system transfers that data to the screen's input
/// jack, and the screen render system draws the correct number of signal dots. If any link
/// in this chain is missing — the insert system doesn't write the jack, the propagation
/// system doesn't copy it, or the render system doesn't read it — the player sees a blank
/// screen despite a card being loaded, which would make the wiring feature appear broken.
#[test]
fn when_card_inserted_and_cable_propagated_then_screen_input_jack_holds_signature_space() {
    // Arrange — reader output jack
    let mut world = World::new();
    let sig = CardSignature::new([0.2, 0.4, -0.3, 0.7, 0.1, -0.5, 0.6, -0.2]);

    let reader_output_jack = world
        .spawn((
            Jack::<SignatureSpace> {
                direction: JackDirection::Output,
                data: None,
            },
            Transform2D::default(),
        ))
        .id();

    // Arrange — screen input jack
    let screen_input_jack = world
        .spawn((
            Jack::<SignatureSpace> {
                direction: JackDirection::Input,
                data: None,
            },
            Transform2D::default(),
        ))
        .id();

    // Arrange — cable connecting reader to screen
    world.spawn(Cable {
        source: reader_output_jack,
        dest: screen_input_jack,
    });

    // Manually populate the reader's output jack (simulates card_reader_insert_system)
    world
        .entity_mut(reader_output_jack)
        .get_mut::<Jack<SignatureSpace>>()
        .unwrap()
        .data = Some(SignatureSpace {
        center: sig,
        radius: SIGNATURE_SPACE_RADIUS,
    });

    // Act — run propagation
    let mut schedule = Schedule::default();
    schedule.add_systems(signature_space_propagation_system);
    schedule.run(&mut world);

    // Assert — screen input jack now holds the SignatureSpace
    let jack = world
        .get::<Jack<SignatureSpace>>(screen_input_jack)
        .unwrap();
    assert_eq!(
        jack.data,
        Some(SignatureSpace {
            center: sig,
            radius: SIGNATURE_SPACE_RADIUS,
        }),
        "screen input jack must receive the SignatureSpace after cable propagation"
    );
}

// ---------------------------------------------------------------------------
// screen_pick_system — behavioral tests
// ---------------------------------------------------------------------------

fn make_pick_world(device_world_pos: Vec2, cursor_world: Vec2, cursor_screen: Vec2) -> World {
    let mut world = World::new();
    spawn_screen_device(&mut world, device_world_pos);
    world.insert_resource(ScreenDragState::default());
    world.insert_resource(DragState::default());
    world.insert_resource(ReaderDragState::default());
    world.insert_resource(PendingCable::default());

    let mut mouse = MouseState::default();
    mouse.set_world_pos(cursor_world);
    mouse.set_screen_pos(cursor_screen);
    mouse.press(MouseButton::Left);
    world.insert_resource(mouse);
    world
}

fn run_pick_system(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(screen_pick_system);
    schedule.run(world);
}

/// @doc: Clicking inside the screen device bounding box must initiate a drag.
/// Catches: `replace screen_pick_system with ()`, `delete !` on `just_pressed` check,
/// `replace <= with >` on hit test bounds.
#[test]
fn when_cursor_inside_screen_device_bounds_and_left_clicked_then_drag_starts() {
    // Arrange — device at origin; cursor at exact center (delta = 0 < HALF_EXTENTS)
    let mut world = make_pick_world(Vec2::ZERO, Vec2::ZERO, Vec2::new(-9999.0, -9999.0));

    // Act
    run_pick_system(&mut world);

    // Assert
    assert!(world.resource::<ScreenDragState>().dragging.is_some());
}

/// @doc: When a card drag is already active, `screen_pick_system` must return without starting
/// a screen drag. Catches `replace || with && in screen_pick_system` (lines 270-272):
/// with `&&`, only all-active blocks the pick; with a single drag, the mutant proceeds.
#[test]
fn when_card_drag_active_and_cursor_on_screen_then_screen_drag_does_not_start() {
    // Arrange
    let mut world = make_pick_world(Vec2::ZERO, Vec2::ZERO, Vec2::new(-9999.0, -9999.0));
    let dummy = world.spawn_empty().id();
    world.resource_mut::<DragState>().dragging = Some(DragInfo {
        entity: dummy,
        local_grab_offset: Vec2::ZERO,
        origin_zone: CardZone::Table,
        stash_cursor_follow: false,
        origin_position: Vec2::ZERO,
    });

    // Act
    run_pick_system(&mut world);

    // Assert
    assert!(world.resource::<ScreenDragState>().dragging.is_none());
}

/// @doc: When the stash is visible and the cursor is inside the stash UI area,
/// `screen_pick_system` must not start a screen drag (stash UI has priority).
/// Catches `replace stash_ui_contains -> bool with true` mutations in callers:
/// this test ensures clicks inside the stash region are correctly blocked.
#[test]
fn when_stash_visible_and_cursor_inside_stash_bounds_then_screen_drag_blocked() {
    // Arrange — place device inside stash screen region; stash occupies x=[20,286], y=[20,427]
    // for a 5×5 grid. Cursor at (100, 100) is inside.
    let stash_screen_pos = Vec2::new(100.0, 100.0);
    let mut world = make_pick_world(Vec2::ZERO, Vec2::ZERO, stash_screen_pos);
    // Override mouse world_pos to be on the device, but screen_pos stays inside stash
    {
        let mut mouse = world.resource_mut::<MouseState>();
        mouse.set_world_pos(Vec2::ZERO);
        mouse.set_screen_pos(stash_screen_pos);
    }
    world.insert_resource(StashGrid::new(5, 5, 1));
    world.insert_resource(StashVisible(true));

    // Act
    run_pick_system(&mut world);

    // Assert
    assert!(world.resource::<ScreenDragState>().dragging.is_none());
}

/// @doc: When the stash is visible but the cursor is outside the stash UI area,
/// `screen_pick_system` must proceed and start a drag.
/// Catches `replace stash_ui_contains -> bool with true`: with the mutant,
/// clicking outside stash would still be blocked, so drag would not start.
#[test]
fn when_stash_visible_and_cursor_outside_stash_bounds_then_screen_drag_starts() {
    // Arrange — cursor screen_pos far outside stash bounds (stash right ≈ 286)
    let outside_stash = Vec2::new(800.0, 400.0);
    let mut world = make_pick_world(Vec2::ZERO, Vec2::ZERO, outside_stash);
    world.insert_resource(StashGrid::new(5, 5, 1));
    world.insert_resource(StashVisible(true));

    // Act
    run_pick_system(&mut world);

    // Assert
    assert!(world.resource::<ScreenDragState>().dragging.is_some());
}
