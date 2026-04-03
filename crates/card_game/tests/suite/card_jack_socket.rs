#![allow(clippy::unwrap_used)]

use std::sync::{Arc, Mutex};

use bevy_ecs::prelude::*;
use card_game::card::component::CardZone;
use card_game::card::interaction::drag_state::{DragInfo, DragState};
use card_game::card::jack_cable::{Cable, Jack, JackDirection};
use card_game::card::jack_socket::{
    JackSocket, PendingCable, jack_socket_pick_system, jack_socket_release_system,
    jack_socket_render_system, pending_cable_drag_system,
};
use card_game::card::reader::{ReaderDragInfo, ReaderDragState, SignatureSpace};
use card_game::card::screen_device::{ScreenDevice, ScreenDragState, spawn_screen_device};
use card_game::plugin::CardGamePlugin;
use engine_app::prelude::App;
use engine_core::color::Color;
use engine_core::prelude::Transform2D;
use engine_input::mouse_button::MouseButton;
use engine_input::prelude::MouseState;
use engine_render::prelude::{Camera2D, RendererRes, ShaderRegistry, Shape, ShapeVariant};
use engine_render::testing::{ShapeCallLog, SpyRenderer};
use engine_scene::prelude::{transform_propagation_system, visibility_system};
use engine_ui::unified_render::unified_render_system;
use glam::Vec2;

fn run_socket_visual_sync(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(jack_socket_render_system);
    schedule.run(world);
}

fn make_preview_app() -> (App, ShapeCallLog) {
    let mut app = App::new();
    app.world_mut().insert_resource(ShaderRegistry::default());
    app.add_plugin(CardGamePlugin);

    let shape_calls: ShapeCallLog = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(Arc::new(Mutex::new(Vec::new())))
        .with_shape_capture(shape_calls.clone())
        .with_viewport(1024, 768);
    app.world_mut()
        .insert_resource(RendererRes::new(Box::new(spy)));
    app.world_mut().spawn(Camera2D {
        position: Vec2::ZERO,
        zoom: 1.0,
    });

    (app, shape_calls)
}

fn run_pending_preview_visuals(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(
        (
            pending_cable_drag_system,
            transform_propagation_system,
            visibility_system,
            unified_render_system,
        )
            .chain(),
    );
    schedule.run(world);
}

/// @doc: Every `JackSocket` must be synced into a concrete `Shape` so the unified renderer
/// has a visible connection point a player can target when dragging a cable. Without this
/// sync, sockets would exist only as logic components and be impossible to see.
#[test]
fn when_one_jack_socket_exists_then_one_shape_is_drawn() {
    // Arrange
    let mut world = World::new();
    let socket_entity = world
        .spawn((
            JackSocket {
                radius: 8.0,
                color: Color::WHITE,
            },
            Transform2D {
                position: Vec2::new(50.0, 50.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();

    // Act
    run_socket_visual_sync(&mut world);

    // Assert
    let shape = world.get::<Shape>(socket_entity).unwrap();
    assert_eq!(
        shape,
        &Shape {
            variant: ShapeVariant::Circle { radius: 8.0 },
            color: Color::WHITE,
        },
        "one JackSocket entity must be converted into a matching circle shape"
    );
}

/// @doc: `jack_socket_render_system` must be a complete no-op when no `JackSocket` entities
/// exist in the world. Any accidental shape insertion here would create a stray visible
/// connection point that does not correspond to a real jack.
#[test]
fn when_no_jack_sockets_exist_then_no_shapes_are_drawn() {
    // Arrange
    let mut world = World::new();

    // Act
    run_socket_visual_sync(&mut world);

    // Assert
    assert_eq!(world.query::<&Shape>().iter(&world).count(), 0);
}

/// @doc: The sync system must iterate all `JackSocket` entities, not just the first.
/// If it only materialized one socket, a wiring setup with multiple devices would show
/// only one connection point and make the others impossible to cable.
#[test]
fn when_two_jack_sockets_exist_then_two_shapes_are_drawn() {
    // Arrange
    let mut world = World::new();
    for pos in [Vec2::new(0.0, 0.0), Vec2::new(100.0, 50.0)] {
        world.spawn((
            JackSocket {
                radius: 8.0,
                color: Color::WHITE,
            },
            Transform2D {
                position: pos,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ));
    }

    // Act
    run_socket_visual_sync(&mut world);

    // Assert
    assert_eq!(
        world.query::<&Shape>().iter(&world).count(),
        2,
        "two JackSocket entities must produce exactly two synced Shape components"
    );
}

// ---------------------------------------------------------------------------
// TC005 — jack_socket_pick_system happy path
// ---------------------------------------------------------------------------

fn make_pick_world() -> World {
    let mut world = World::new();
    world.insert_resource(DragState::default());
    world.insert_resource(ReaderDragState::default());
    world.insert_resource(ScreenDragState::default());
    world.insert_resource(PendingCable::default());
    world
}

fn spawn_jack_socket_at(world: &mut World, pos: Vec2) -> Entity {
    world
        .spawn((
            Jack::<SignatureSpace> {
                direction: JackDirection::Output,
                data: None,
            },
            JackSocket {
                radius: 10.0,
                color: Color::WHITE,
            },
            Transform2D {
                position: pos,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id()
}

/// @doc: When the player clicks directly on a jack socket, `jack_socket_pick_system` must
/// record that socket's entity in `PendingCable.source`, beginning a cable drag. Without
/// this, clicking on a socket does nothing and the player has no way to initiate a
/// connection — the entire wiring interaction becomes inaccessible.
#[test]
fn given_cursor_over_jack_socket_when_left_mouse_just_pressed_then_pending_cable_source_is_set() {
    // Arrange
    let mut world = make_pick_world();
    let jack_entity = spawn_jack_socket_at(&mut world, Vec2::new(100.0, 100.0));
    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Left);
    mouse.set_world_pos(Vec2::new(100.0, 100.0));
    world.insert_resource(mouse);

    let mut schedule = Schedule::default();
    schedule.add_systems(jack_socket_pick_system);

    // Act
    schedule.run(&mut world);

    // Assert
    let pending = world.resource::<PendingCable>();
    assert_eq!(
        pending.source,
        Some(jack_entity),
        "clicking on a jack socket must set PendingCable.source to that jack's entity"
    );
}

// ---------------------------------------------------------------------------
// TC006 — miss: cursor outside socket radius
// ---------------------------------------------------------------------------

/// @doc: The pick system must use a radius-based hit test, not a bounding-box one.
/// If the cursor is beyond the socket's radius, no cable drag should start — otherwise
/// clicking anywhere near a jack but not on it would accidentally begin a connection,
/// making precise cable routing impossible on a crowded wiring table.
#[test]
fn given_cursor_outside_jack_socket_when_left_mouse_just_pressed_then_pending_cable_stays_none() {
    // Arrange
    let mut world = make_pick_world();
    spawn_jack_socket_at(&mut world, Vec2::new(100.0, 100.0));
    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Left);
    mouse.set_world_pos(Vec2::new(200.0, 200.0)); // far outside radius 10
    world.insert_resource(mouse);

    let mut schedule = Schedule::default();
    schedule.add_systems(jack_socket_pick_system);

    // Act
    schedule.run(&mut world);

    // Assert
    assert!(
        world.resource::<PendingCable>().source.is_none(),
        "cursor outside the socket radius must not start a cable drag"
    );
}

// ---------------------------------------------------------------------------
// TC007 — guard: card drag already active
// ---------------------------------------------------------------------------

/// @doc: The pick system must not start a cable drag while a card is already being dragged.
/// Allowing both drag states simultaneously would mean two different mouse-button consumers
/// fight over the same release event, leaving the game in an unrecoverable state where
/// neither the card nor the cable can be properly dropped.
#[test]
fn given_card_drag_active_when_left_mouse_pressed_over_socket_then_pending_cable_stays_none() {
    // Arrange
    let mut world = make_pick_world();
    let jack_entity = spawn_jack_socket_at(&mut world, Vec2::new(100.0, 100.0));
    world.insert_resource(DragState {
        dragging: Some(DragInfo {
            entity: jack_entity,
            local_grab_offset: Vec2::ZERO,
            origin_zone: CardZone::Table,
            stash_cursor_follow: false,
            origin_position: Vec2::ZERO,
        }),
    });
    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Left);
    mouse.set_world_pos(Vec2::new(100.0, 100.0));
    world.insert_resource(mouse);

    let mut schedule = Schedule::default();
    schedule.add_systems(jack_socket_pick_system);

    // Act
    schedule.run(&mut world);

    // Assert
    assert!(
        world.resource::<PendingCable>().source.is_none(),
        "jack socket pick must not fire while a card drag is in progress"
    );
}

// ---------------------------------------------------------------------------
// TC008 — guard: reader drag already active
// ---------------------------------------------------------------------------

/// @doc: The pick system must not start a cable drag while the reader itself is being
/// repositioned. If both interactions could start simultaneously, releasing the mouse
/// would need to resolve two drag completions with a single event, making it impossible
/// to place the reader and then immediately begin a cable connection without confusion.
#[test]
fn given_reader_drag_active_when_left_mouse_pressed_over_socket_then_pending_cable_stays_none() {
    // Arrange
    let mut world = make_pick_world();
    let dummy = world.spawn_empty().id();
    spawn_jack_socket_at(&mut world, Vec2::new(100.0, 100.0));
    world.insert_resource(ReaderDragState {
        dragging: Some(ReaderDragInfo {
            entity: dummy,
            grab_offset: Vec2::ZERO,
        }),
    });
    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Left);
    mouse.set_world_pos(Vec2::new(100.0, 100.0));
    world.insert_resource(mouse);

    let mut schedule = Schedule::default();
    schedule.add_systems(jack_socket_pick_system);

    // Act
    schedule.run(&mut world);

    // Assert
    assert!(
        world.resource::<PendingCable>().source.is_none(),
        "jack socket pick must not fire while the reader is being dragged"
    );
}

// ---------------------------------------------------------------------------
// TC009 — guard: pending cable already active
// ---------------------------------------------------------------------------

/// @doc: Once a cable drag is in progress, clicking again must not overwrite the source
/// jack with a new one. If re-picking were allowed, the first click's source entity would
/// be silently replaced, making it impossible to complete the intended connection and
/// causing the player's first click to be lost.
#[test]
fn given_pending_cable_already_active_when_left_mouse_pressed_over_second_socket_then_source_unchanged()
 {
    // Arrange
    let mut world = make_pick_world();
    let first = spawn_jack_socket_at(&mut world, Vec2::new(100.0, 100.0));
    spawn_jack_socket_at(&mut world, Vec2::new(200.0, 200.0));
    world.insert_resource(PendingCable {
        source: Some(first),
    });
    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Left);
    mouse.set_world_pos(Vec2::new(200.0, 200.0));
    world.insert_resource(mouse);

    let mut schedule = Schedule::default();
    schedule.add_systems(jack_socket_pick_system);

    // Act
    schedule.run(&mut world);

    // Assert
    assert_eq!(
        world.resource::<PendingCable>().source,
        Some(first),
        "source must remain the original jack while a cable drag is already in progress"
    );
}

// ---------------------------------------------------------------------------
// TC010 — pending_cable_drag_system draws preview line
// ---------------------------------------------------------------------------

/// @doc: While a cable drag is in progress, `pending_cable_drag_system` must draw a preview
/// line from the source socket to the cursor so the player can see the cable they are routing.
/// Without this visual feedback, cable connection is fully invisible — the player presses a
/// socket and nothing happens until they release, with no indication that a drag is active.
#[test]
fn given_pending_cable_active_and_mouse_pressed_when_drag_system_runs_then_preview_shape_is_drawn()
{
    // Arrange
    let (mut app, shape_calls) = make_preview_app();
    let source_jack = app
        .world_mut()
        .spawn((
            JackSocket {
                radius: 8.0,
                color: Color::WHITE,
            },
            Transform2D {
                position: Vec2::new(50.0, 50.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();
    app.world_mut().insert_resource(PendingCable {
        source: Some(source_jack),
    });
    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Left);
    mouse.set_world_pos(Vec2::new(150.0, 150.0));
    app.world_mut().insert_resource(mouse);

    // Act
    run_pending_preview_visuals(app.world_mut());

    // Assert
    assert_eq!(
        shape_calls.lock().unwrap().len(),
        1,
        "an active cable drag must draw exactly one preview line from source to cursor"
    );
}

// ---------------------------------------------------------------------------
// TC011 — no preview when no pending cable
// ---------------------------------------------------------------------------

/// @doc: `pending_cable_drag_system` must be a no-op when `PendingCable.source` is `None`.
/// If a preview were drawn without an active drag, a phantom cable would appear from some
/// arbitrary position on the table every frame, making the wiring surface look broken
/// even when the player has not started any connection.
#[test]
fn given_no_pending_cable_when_drag_system_runs_then_no_preview_shape_is_drawn() {
    // Arrange
    let (mut app, shape_calls) = make_preview_app();
    app.world_mut().insert_resource(PendingCable::default());
    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Left);
    mouse.set_world_pos(Vec2::new(150.0, 150.0));
    app.world_mut().insert_resource(mouse);

    // Act
    run_pending_preview_visuals(app.world_mut());

    // Assert
    assert!(
        shape_calls.lock().unwrap().is_empty(),
        "no pending cable means no preview line should be drawn"
    );
}

// ---------------------------------------------------------------------------
// TC012 — no preview when mouse not pressed
// ---------------------------------------------------------------------------

/// @doc: When the mouse button is not held, `pending_cable_drag_system` must not draw a
/// preview even if `PendingCable.source` is set. The mouse-pressed guard prevents the
/// system from drawing on the same frame the button was released before the release system
/// clears the pending state, avoiding a one-frame ghost cable.
#[test]
fn given_pending_cable_set_but_mouse_not_pressed_when_drag_system_runs_then_no_preview_drawn() {
    // Arrange
    let (mut app, shape_calls) = make_preview_app();
    let source_jack = app
        .world_mut()
        .spawn((
            JackSocket {
                radius: 8.0,
                color: Color::WHITE,
            },
            Transform2D::default(),
        ))
        .id();
    app.world_mut().insert_resource(PendingCable {
        source: Some(source_jack),
    });
    app.world_mut().insert_resource(MouseState::default()); // mouse not pressed

    // Act
    run_pending_preview_visuals(app.world_mut());

    // Assert
    assert!(
        shape_calls.lock().unwrap().is_empty(),
        "no preview must be drawn when the mouse button is not held"
    );
}

// ---------------------------------------------------------------------------
// TC013 — release on compatible socket spawns Cable
// ---------------------------------------------------------------------------

fn spawn_input_socket_at(world: &mut World, pos: Vec2) -> Entity {
    world
        .spawn((
            Jack::<SignatureSpace> {
                direction: JackDirection::Input,
                data: None,
            },
            JackSocket {
                radius: 10.0,
                color: Color::WHITE,
            },
            Transform2D {
                position: pos,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id()
}

/// @doc: Releasing the mouse over a compatible jack (Output→Input pair) must spawn a
/// `Cable` entity connecting the two jacks. Without this, the drag interaction produces
/// no persistent connection — the player sees the preview line appear and disappear with
/// nothing to show for it, making the wiring feature completely non-functional.
#[test]
fn given_pending_cable_and_cursor_over_compatible_socket_when_mouse_released_then_cable_entity_spawned()
 {
    // Arrange
    let mut world = make_pick_world();
    let output_jack = spawn_jack_socket_at(&mut world, Vec2::new(0.0, 0.0));
    let input_jack = spawn_input_socket_at(&mut world, Vec2::new(100.0, 0.0));
    world.insert_resource(PendingCable {
        source: Some(output_jack),
    });
    let mut mouse = MouseState::default();
    mouse.release(MouseButton::Left);
    mouse.set_world_pos(Vec2::new(100.0, 0.0));
    world.insert_resource(mouse);

    let mut schedule = Schedule::default();
    schedule.add_systems(jack_socket_release_system);

    // Act
    schedule.run(&mut world);

    // Assert
    let mut q = world.query::<&Cable>();
    let cables: Vec<_> = q.iter(&world).collect();
    assert_eq!(
        cables.len(),
        1,
        "releasing over a compatible socket must spawn exactly one Cable entity"
    );
    assert_eq!(cables[0].source, output_jack);
    assert_eq!(cables[0].dest, input_jack);
}

// ---------------------------------------------------------------------------
// TC014 — release on compatible socket clears PendingCable
// ---------------------------------------------------------------------------

/// @doc: After a successful cable connection the drag state must be cleared so that
/// subsequent mouse events are not incorrectly treated as part of a continuing drag.
/// If `PendingCable.source` were left set after the release, the next click anywhere on
/// the table would be processed by `jack_socket_pick_system` as if a drag were already
/// in progress — the guard condition would fire and block the new pick.
#[test]
fn given_pending_cable_and_cursor_over_compatible_socket_when_mouse_released_then_pending_cable_cleared()
 {
    // Arrange
    let mut world = make_pick_world();
    let output_jack = spawn_jack_socket_at(&mut world, Vec2::new(0.0, 0.0));
    spawn_input_socket_at(&mut world, Vec2::new(100.0, 0.0));
    world.insert_resource(PendingCable {
        source: Some(output_jack),
    });
    let mut mouse = MouseState::default();
    mouse.release(MouseButton::Left);
    mouse.set_world_pos(Vec2::new(100.0, 0.0));
    world.insert_resource(mouse);

    let mut schedule = Schedule::default();
    schedule.add_systems(jack_socket_release_system);

    // Act
    schedule.run(&mut world);

    // Assert
    assert!(
        world.resource::<PendingCable>().source.is_none(),
        "PendingCable must be cleared after a successful cable connection"
    );
}

// ---------------------------------------------------------------------------
// TC015 — release in empty space: no Cable spawned
// ---------------------------------------------------------------------------

/// @doc: Releasing the mouse in empty space (not over any socket) must cancel the pending
/// cable without spawning a `Cable` entity. The player may decide mid-drag that they want
/// to connect to a different destination — cancel-by-release-to-empty-space is the
/// standard escape hatch that prevents accidental incomplete connections from polluting
/// the wiring graph.
#[test]
fn given_pending_cable_and_cursor_in_empty_space_when_mouse_released_then_no_cable_spawned() {
    // Arrange
    let mut world = make_pick_world();
    let output_jack = spawn_jack_socket_at(&mut world, Vec2::new(0.0, 0.0));
    world.insert_resource(PendingCable {
        source: Some(output_jack),
    });
    let mut mouse = MouseState::default();
    mouse.release(MouseButton::Left);
    mouse.set_world_pos(Vec2::new(9999.0, 9999.0));
    world.insert_resource(mouse);

    let mut schedule = Schedule::default();
    schedule.add_systems(jack_socket_release_system);

    // Act
    schedule.run(&mut world);

    // Assert
    assert_eq!(
        world.query::<&Cable>().iter(&world).count(),
        0,
        "releasing in empty space must not spawn any Cable entity"
    );
}

// ---------------------------------------------------------------------------
// TC016 — release in empty space: PendingCable cleared
// ---------------------------------------------------------------------------

/// @doc: Even when no cable is spawned (drag cancelled to empty space), `PendingCable.source`
/// must be cleared on mouse release. If the cancelled drag left the source set, the next
/// left-click would be blocked by the pending-cable guard — the player would be stuck unable
/// to start any new interaction until they clicked a second time to clear the stale state.
#[test]
fn given_pending_cable_and_cursor_in_empty_space_when_mouse_released_then_pending_cable_cleared() {
    // Arrange
    let mut world = make_pick_world();
    let output_jack = spawn_jack_socket_at(&mut world, Vec2::new(0.0, 0.0));
    world.insert_resource(PendingCable {
        source: Some(output_jack),
    });
    let mut mouse = MouseState::default();
    mouse.release(MouseButton::Left);
    mouse.set_world_pos(Vec2::new(9999.0, 9999.0));
    world.insert_resource(mouse);

    let mut schedule = Schedule::default();
    schedule.add_systems(jack_socket_release_system);

    // Act
    schedule.run(&mut world);

    // Assert
    assert!(
        world.resource::<PendingCable>().source.is_none(),
        "PendingCable must always be cleared on mouse release, even when no cable is spawned"
    );
}

// ---------------------------------------------------------------------------
// TC017 — self-connection guard
// ---------------------------------------------------------------------------

/// @doc: Releasing the mouse over the same socket where the drag started must not spawn
/// a self-loop `Cable`. A cable connecting a jack to itself would create a signal cycle
/// where the jack's own output feeds back into itself, which is meaningless in the
/// signature propagation model and could cause infinite update loops.
#[test]
fn given_pending_cable_and_cursor_over_same_socket_when_mouse_released_then_no_cable_spawned() {
    // Arrange
    let mut world = make_pick_world();
    let output_jack = spawn_jack_socket_at(&mut world, Vec2::new(100.0, 100.0));
    world.insert_resource(PendingCable {
        source: Some(output_jack),
    });
    let mut mouse = MouseState::default();
    mouse.release(MouseButton::Left);
    mouse.set_world_pos(Vec2::new(100.0, 100.0)); // same position as source
    world.insert_resource(mouse);

    let mut schedule = Schedule::default();
    schedule.add_systems(jack_socket_release_system);

    // Act
    schedule.run(&mut world);

    // Assert
    assert_eq!(
        world.query::<&Cable>().iter(&world).count(),
        0,
        "releasing on the source socket itself must not create a self-loop cable"
    );
}

// ---------------------------------------------------------------------------
// TC018 — incompatible direction guard (Output + Output)
// ---------------------------------------------------------------------------

/// @doc: Connecting two Output jacks must not spawn a `Cable`. The signal propagation
/// model requires exactly one Output source and one Input destination per cable — an
/// Output→Output connection has no valid interpretation and would cause the propagation
/// system to write to a jack that already has its own data source, corrupting the signal
/// with whatever arrived first from an arbitrary iteration order.
#[test]
fn given_pending_cable_and_cursor_over_same_direction_socket_when_mouse_released_then_no_cable_spawned()
 {
    // Arrange
    let mut world = make_pick_world();
    let output_jack1 = spawn_jack_socket_at(&mut world, Vec2::new(0.0, 0.0));
    spawn_jack_socket_at(&mut world, Vec2::new(100.0, 0.0)); // also Output
    world.insert_resource(PendingCable {
        source: Some(output_jack1),
    });
    let mut mouse = MouseState::default();
    mouse.release(MouseButton::Left);
    mouse.set_world_pos(Vec2::new(100.0, 0.0));
    world.insert_resource(mouse);

    let mut schedule = Schedule::default();
    schedule.add_systems(jack_socket_release_system);

    // Act
    schedule.run(&mut world);

    // Assert
    assert_eq!(
        world.query::<&Cable>().iter(&world).count(),
        0,
        "two Output jacks must not be connected — direction compatibility is required"
    );
}

// ---------------------------------------------------------------------------
// TC019 — no pending cable on release: no Cable spawned
// ---------------------------------------------------------------------------

#[test]
fn given_no_pending_cable_when_mouse_released_over_socket_then_no_cable_spawned() {
    // Arrange
    let mut world = make_pick_world();
    spawn_jack_socket_at(&mut world, Vec2::new(0.0, 0.0));
    spawn_input_socket_at(&mut world, Vec2::new(100.0, 0.0));
    world.insert_resource(PendingCable::default());
    let mut mouse = MouseState::default();
    mouse.release(MouseButton::Left);
    mouse.set_world_pos(Vec2::new(100.0, 0.0));
    world.insert_resource(mouse);

    let mut schedule = Schedule::default();
    schedule.add_systems(jack_socket_release_system);

    // Act
    schedule.run(&mut world);

    // Assert
    assert_eq!(
        world.query::<&Cable>().iter(&world).count(),
        0,
        "no pending cable means mouse release must not spawn any Cable entity"
    );
}

// ---------------------------------------------------------------------------
// TC020–TC023 — spawn_screen_device
// ---------------------------------------------------------------------------

/// @doc: `spawn_screen_device` must produce an entity that carries both `ScreenDevice`
/// and `Transform2D` at the requested position. Without these two components together,
/// `screen_render_system` cannot locate the device on the table or look up its input jack,
/// making the screen invisible no matter what cable is connected to it.
#[test]
fn given_empty_world_when_spawn_screen_device_called_then_screen_device_entity_exists_with_transform()
 {
    // Arrange
    let mut world = World::new();

    // Act
    let (device_entity, _jack_entity) = spawn_screen_device(&mut world, Vec2::new(100.0, 50.0));

    // Assert
    assert!(world.get::<ScreenDevice>(device_entity).is_some());
    assert!(world.get::<Transform2D>(device_entity).is_some());
}

/// @doc: The jack entity spawned by `spawn_screen_device` must have `JackDirection::Input`
/// so that it is a valid cable destination. If the direction were `Output`, the
/// direction-compatibility check in `jack_socket_release_system` would reject any cable
/// dragged from a reader's output jack to the screen, making the screen unconnectable.
#[test]
fn given_empty_world_when_spawn_screen_device_called_then_jack_entity_has_direction_input() {
    // Arrange
    let mut world = World::new();

    // Act
    let (_device_entity, jack_entity) = spawn_screen_device(&mut world, Vec2::ZERO);

    // Assert
    let jack = world.get::<Jack<SignatureSpace>>(jack_entity).unwrap();
    assert_eq!(
        jack.direction,
        JackDirection::Input,
        "screen device jack must be an Input so cables can be connected from a reader Output"
    );
}

/// @doc: The jack entity spawned by `spawn_screen_device` must also carry a `JackSocket`
/// component so it is rendered as a visible dot and is included in the pick/release hit
/// tests. Without `JackSocket`, the jack exists only as an invisible ECS entity — the
/// player cannot see it, click it, or connect a cable to it.
#[test]
fn given_empty_world_when_spawn_screen_device_called_then_jack_entity_has_jack_socket() {
    // Arrange
    let mut world = World::new();

    // Act
    let (_device_entity, jack_entity) = spawn_screen_device(&mut world, Vec2::ZERO);

    // Assert
    let socket = world.get::<JackSocket>(jack_entity);
    assert!(
        socket.is_some(),
        "jack entity must have JackSocket for rendering and hit-testing"
    );
    assert!(
        socket.unwrap().radius > 0.0,
        "JackSocket radius must be positive"
    );
}

#[test]
fn given_empty_world_when_spawn_screen_device_called_then_signature_input_matches_jack_entity() {
    // Arrange
    let mut world = World::new();

    // Act
    let (device_entity, jack_entity) = spawn_screen_device(&mut world, Vec2::ZERO);

    // Assert
    let device = world.get::<ScreenDevice>(device_entity).unwrap();
    assert_eq!(
        device.signature_input, jack_entity,
        "ScreenDevice.signature_input must reference the spawned jack entity"
    );
}
