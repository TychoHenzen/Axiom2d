#![allow(clippy::unwrap_used)]

use std::sync::{Arc, Mutex};

use bevy_ecs::prelude::*;
use card_game::card::component::CardZone;
use card_game::card::interaction::click_resolve::{ClickHitShape, Clickable, click_resolve_system};
use card_game::card::interaction::drag_state::{DragInfo, DragState};
use card_game::card::interaction::intent::InteractionIntent;
use card_game::card::jack_cable::{Cable, Jack, JackDirection, WireEndpoints};
use card_game::card::jack_socket::{
    CableFreeEnd, JackSocket, PendingCable, jack_socket_release_system, jack_socket_render_system,
    on_socket_clicked, pending_cable_drag_system,
};
use card_game::card::reader::{ReaderDragInfo, ReaderDragState, SignatureSpace};
use card_game::plugin::CardGamePlugin;
use engine_app::prelude::App;
use engine_core::color::Color;
use engine_core::prelude::Transform2D;
use engine_input::mouse_button::MouseButton;
use engine_input::prelude::MouseState;
use engine_render::prelude::{Camera2D, RendererRes, ShaderRegistry, Shape, ShapeVariant};
use engine_render::testing::{ShapeCallLog, SpyRenderer};
use engine_scene::prelude::{Visible, transform_propagation_system, visibility_system};
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
                connected_cable: None,
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
                connected_cable: None,
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

fn run_click_resolve(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(click_resolve_system);
    schedule.run(world);
}

fn make_pick_world() -> World {
    let mut world = World::new();
    world.insert_resource(DragState::default());
    world.insert_resource(ReaderDragState::default());
    world.insert_resource(PendingCable::default());
    world.insert_resource(engine_core::prelude::EventBus::<InteractionIntent>::default());
    world
}

fn spawn_jack_socket_at(world: &mut World, pos: Vec2) -> Entity {
    use engine_scene::prelude::{GlobalTransform2D, SortOrder};
    use glam::Affine2;
    let entity = world
        .spawn((
            Jack::<SignatureSpace> {
                direction: JackDirection::Output,
                data: None,
            },
            JackSocket {
                radius: 10.0,
                color: Color::WHITE,
                connected_cable: None,
            },
            Transform2D {
                position: pos,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
            Clickable(ClickHitShape::Circle(10.0)),
            GlobalTransform2D(Affine2::from_translation(pos)),
            SortOrder::default(),
        ))
        .id();
    world.entity_mut(entity).observe(on_socket_clicked);
    entity
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

    // Act
    run_click_resolve(&mut world);

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

    // Act
    run_click_resolve(&mut world);

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

    // Act
    run_click_resolve(&mut world);

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

    // Act
    run_click_resolve(&mut world);

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
        origin_cable: None,
        free_end: None,
    });
    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Left);
    mouse.set_world_pos(Vec2::new(200.0, 200.0));
    world.insert_resource(mouse);

    // Act
    run_click_resolve(&mut world);

    // Assert
    assert_eq!(
        world.resource::<PendingCable>().source,
        Some(first),
        "source must remain the original jack while a cable drag is already in progress"
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
                connected_cable: None,
            },
            Transform2D::default(),
        ))
        .id();
    app.world_mut().insert_resource(PendingCable {
        source: Some(source_jack),
        origin_cable: None,
        free_end: None,
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
                connected_cable: None,
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
    let free_end = world
        .spawn((
            CableFreeEnd,
            Transform2D {
                position: Vec2::new(100.0, 0.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();
    let wire_entity = world
        .spawn(WireEndpoints {
            source: output_jack,
            dest: free_end,
        })
        .id();
    world.insert_resource(PendingCable {
        source: Some(output_jack),
        origin_cable: Some(wire_entity),
        free_end: Some(free_end),
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
    let cable = world.get::<Cable>(wire_entity);
    assert!(
        cable.is_some(),
        "releasing over a compatible socket must add a Cable component to the wire entity"
    );
    let cable = cable.unwrap();
    assert_eq!(cable.source, output_jack);
    assert_eq!(cable.dest, input_jack);
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
        origin_cable: None,
        free_end: None,
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
        origin_cable: None,
        free_end: None,
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
        origin_cable: None,
        free_end: None,
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

/// @doc: The empty-space release path must also be safe when it runs in the same update
/// chain as `pending_cable_drag_system`, because that is the real schedule order used by
/// the game. If the release cleanup leaves a stale free-end or preview state behind, the
/// next system in the chain can panic when it tries to read a despawned cable.
#[test]
fn given_pending_cable_and_preview_when_release_chain_runs_then_cable_cleared_without_panic() {
    // Arrange
    let mut world = make_pick_world();
    let output_jack = spawn_jack_socket_at(&mut world, Vec2::new(0.0, 0.0));
    let free_end = world
        .spawn((
            CableFreeEnd,
            Transform2D {
                position: Vec2::new(20.0, 0.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();
    let cable_entity = world
        .spawn((
            Cable {
                source: output_jack,
                dest: free_end,
            },
            WireEndpoints {
                source: output_jack,
                dest: free_end,
            },
            Shape {
                variant: ShapeVariant::Polygon {
                    points: vec![Vec2::ZERO],
                },
                color: Color::WHITE,
            },
            Visible(false),
            Transform2D::default(),
        ))
        .id();
    world
        .entity_mut(output_jack)
        .get_mut::<JackSocket>()
        .unwrap()
        .connected_cable = Some(cable_entity);
    world.insert_resource(PendingCable {
        source: Some(output_jack),
        origin_cable: Some(cable_entity),
        free_end: Some(free_end),
    });
    let mut mouse = MouseState::default();
    mouse.release(MouseButton::Left);
    mouse.set_world_pos(Vec2::new(9999.0, 9999.0));
    world.insert_resource(mouse);

    let mut schedule = Schedule::default();
    schedule.add_systems((jack_socket_release_system, pending_cable_drag_system).chain());

    // Act
    schedule.run(&mut world);

    // Assert
    assert!(
        world.get_entity(cable_entity).is_err(),
        "cable entity must be despawned when the drag is released into empty space"
    );
    assert!(
        world.resource::<PendingCable>().source.is_none(),
        "release chain must clear PendingCable.source after empty-space release"
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
        origin_cable: None,
        free_end: None,
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
        origin_cable: None,
        free_end: None,
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
// TC-OCC-01 (spec TC015) — JackSocket.connected_cable / is_occupied
// ---------------------------------------------------------------------------

/// @doc: `JackSocket` must expose a `connected_cable: Option<Entity>` field and an
/// `is_occupied()` convenience method so the release system can refuse to double-connect
/// an already-used socket. Without occupancy tracking every socket would silently accept
/// multiple cables, making the wiring graph impossible to reason about.
#[test]
fn given_jack_socket_with_connected_cable_set_when_is_occupied_called_then_returns_true() {
    // Arrange
    let mut scratch_world = World::new();
    let dummy_cable = scratch_world.spawn_empty().id();
    let socket = JackSocket {
        radius: 10.0,
        color: Color::WHITE,
        connected_cable: Some(dummy_cable), // field does not exist yet → compile error
    };

    // Act
    let occupied = socket.is_occupied(); // method does not exist yet → compile error

    // Assert
    assert!(
        occupied,
        "is_occupied must return true when connected_cable is Some"
    );
}

// ---------------------------------------------------------------------------
// TC-OCC-02 (spec TC016) — occupied socket blocks second cable spawn
// ---------------------------------------------------------------------------

/// @doc: `jack_socket_release_system` must check `JackSocket.is_occupied()` before
/// spawning a cable. If an input socket already holds a cable, dropping a second pending
/// cable onto it must be silently rejected — otherwise one socket would be the destination
/// of two cables, creating an ambiguous signal fanin that the propagation system cannot
/// handle deterministically.
#[test]
fn given_occupied_input_socket_when_pending_cable_released_over_it_then_no_second_cable_spawned() {
    use engine_scene::prelude::{GlobalTransform2D, SortOrder};
    use glam::Affine2;

    // Arrange
    let mut world = make_pick_world();

    let existing_cable = world.spawn_empty().id();

    let output_socket = world
        .spawn((
            Jack::<SignatureSpace> {
                direction: JackDirection::Output,
                data: None,
            },
            JackSocket {
                radius: 10.0,
                color: Color::WHITE,
                connected_cable: None, // field does not exist yet → compile error
            },
            Transform2D {
                position: Vec2::new(0.0, 0.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
            GlobalTransform2D(Affine2::from_translation(Vec2::new(0.0, 0.0))),
            SortOrder::default(),
        ))
        .id();

    world.spawn((
        Jack::<SignatureSpace> {
            direction: JackDirection::Input,
            data: None,
        },
        JackSocket {
            radius: 10.0,
            color: Color::WHITE,
            connected_cable: Some(existing_cable), // occupied
        },
        Transform2D {
            position: Vec2::new(50.0, 0.0),
            rotation: 0.0,
            scale: Vec2::ONE,
        },
        GlobalTransform2D(Affine2::from_translation(Vec2::new(50.0, 0.0))),
        SortOrder::default(),
    ));

    world.insert_resource(PendingCable {
        source: Some(output_socket),
        origin_cable: None,
        free_end: None,
    });
    let mut mouse = MouseState::default();
    mouse.release(MouseButton::Left);
    mouse.set_world_pos(Vec2::new(50.0, 0.0));
    world.insert_resource(mouse);

    let mut schedule = Schedule::default();
    schedule.add_systems(jack_socket_release_system);

    // Act
    schedule.run(&mut world);

    // Assert — no new Cable component was spawned
    assert_eq!(
        world.query::<&Cable>().iter(&world).count(),
        0,
        "releasing over an occupied socket must not spawn a second Cable"
    );
}

// ---------------------------------------------------------------------------
// TC-OCC-03 (spec TC017) — successful connect marks both sockets occupied
// ---------------------------------------------------------------------------

/// @doc: When `jack_socket_release_system` successfully connects two jacks it must write
/// the new cable entity into `JackSocket.connected_cable` on both the source and destination
/// sockets. Without this, `is_occupied()` would always return false and the occupancy guard
/// added in TC-OCC-02 would never fire — every socket would remain permanently
/// re-connectable regardless of how many cables were already attached.
#[test]
fn given_two_free_compatible_sockets_when_cable_connected_then_both_sockets_connected_cable_is_set()
{
    use engine_scene::prelude::{GlobalTransform2D, SortOrder};
    use glam::Affine2;

    // Arrange
    let mut world = make_pick_world();

    let output_socket = world
        .spawn((
            Jack::<SignatureSpace> {
                direction: JackDirection::Output,
                data: None,
            },
            JackSocket {
                radius: 10.0,
                color: Color::WHITE,
                connected_cable: None,
            },
            Transform2D {
                position: Vec2::new(0.0, 0.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
            GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
            SortOrder::default(),
        ))
        .id();

    let input_socket = world
        .spawn((
            Jack::<SignatureSpace> {
                direction: JackDirection::Input,
                data: None,
            },
            JackSocket {
                radius: 10.0,
                color: Color::WHITE,
                connected_cable: None,
            },
            Transform2D {
                position: Vec2::new(50.0, 0.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
            GlobalTransform2D(Affine2::from_translation(Vec2::new(50.0, 0.0))),
            SortOrder::default(),
        ))
        .id();

    let free_end = world
        .spawn((
            CableFreeEnd,
            Transform2D {
                position: Vec2::new(50.0, 0.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();
    let wire_entity = world
        .spawn(WireEndpoints {
            source: output_socket,
            dest: free_end,
        })
        .id();
    world.insert_resource(PendingCable {
        source: Some(output_socket),
        origin_cable: Some(wire_entity),
        free_end: Some(free_end),
    });
    let mut mouse = MouseState::default();
    mouse.release(MouseButton::Left);
    mouse.set_world_pos(Vec2::new(50.0, 0.0));
    world.insert_resource(mouse);

    let mut schedule = Schedule::default();
    schedule.add_systems(jack_socket_release_system);

    // Act
    schedule.run(&mut world);

    // Assert
    let out_socket = world.get::<JackSocket>(output_socket).unwrap();
    let in_socket = world.get::<JackSocket>(input_socket).unwrap();
    assert!(
        out_socket.connected_cable.is_some(),
        "output socket must be marked occupied after successful connection"
    );
    assert!(
        in_socket.connected_cable.is_some(),
        "input socket must be marked occupied after successful connection"
    );
    assert_eq!(
        out_socket.connected_cable, in_socket.connected_cable,
        "both sockets must reference the same cable entity"
    );
}

// ---------------------------------------------------------------------------
// TC-OCC-04 (spec TC018) — clicking occupied socket begins disconnect-drag
// ---------------------------------------------------------------------------

/// @doc: Clicking an occupied socket (when no cable drag is already pending) must start a
/// disconnect-drag: `PendingCable.source` is set to the OTHER end of the existing cable,
/// the clicked socket's `connected_cable` is cleared, and `PendingCable.origin_cable`
/// records the cable entity so it can be re-attached or destroyed on release. Without this
/// the player has no way to move an existing connection — they would need to delete and
/// recreate every cable they want to reroute.
#[test]
fn given_occupied_socket_when_clicked_with_no_active_pending_cable_then_disconnect_drag_begins() {
    use engine_scene::prelude::{GlobalTransform2D, SortOrder};
    use glam::Affine2;

    // Arrange
    let mut world = make_pick_world();

    let output_socket = world
        .spawn((
            Jack::<SignatureSpace> {
                direction: JackDirection::Output,
                data: None,
            },
            JackSocket {
                radius: 10.0,
                color: Color::WHITE,
                connected_cable: None, // updated after cable entity created below
            },
            Transform2D {
                position: Vec2::new(0.0, 0.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
            GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
            SortOrder::default(),
        ))
        .id();

    // placeholder input socket; will have a proper id assigned below
    let input_socket_placeholder = world.spawn_empty().id();
    let cable_entity = world
        .spawn(Cable {
            source: output_socket,
            dest: input_socket_placeholder,
        })
        .id();

    let input_socket = world
        .spawn((
            Jack::<SignatureSpace> {
                direction: JackDirection::Input,
                data: None,
            },
            JackSocket {
                radius: 10.0,
                color: Color::WHITE,
                connected_cable: Some(cable_entity), // occupied
            },
            Transform2D {
                position: Vec2::new(50.0, 0.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
            Clickable(ClickHitShape::Circle(10.0)),
            GlobalTransform2D(Affine2::from_translation(Vec2::new(50.0, 0.0))),
            SortOrder::default(),
        ))
        .id();
    world.entity_mut(input_socket).observe(on_socket_clicked);

    // fix cable dest to real input_socket entity
    world
        .entity_mut(cable_entity)
        .get_mut::<Cable>()
        .unwrap()
        .dest = input_socket;
    // mark output socket occupied
    world
        .entity_mut(output_socket)
        .get_mut::<JackSocket>()
        .unwrap()
        .connected_cable = Some(cable_entity);

    world.insert_resource(PendingCable {
        source: None,
        origin_cable: None,
        free_end: None,
    });
    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Left);
    mouse.set_world_pos(Vec2::new(50.0, 0.0)); // cursor on the input socket
    world.insert_resource(mouse);

    // Act
    run_click_resolve(&mut world);

    // Assert
    let pending = world.resource::<PendingCable>();
    assert_eq!(
        pending.source,
        Some(output_socket),
        "PendingCable.source must be the other end of the disconnected cable"
    );
    assert_eq!(
        pending.origin_cable,
        Some(cable_entity),
        "PendingCable.origin_cable must hold the cable entity being rerouted"
    );
    assert!(
        world
            .get::<JackSocket>(input_socket)
            .unwrap()
            .connected_cable
            .is_none(),
        "clicked socket must be unoccupied after disconnect-drag begins"
    );
}

// ---------------------------------------------------------------------------
// TC-OCC-05 (spec TC019) — disconnect drag in empty space destroys cable
// ---------------------------------------------------------------------------

/// @doc: When a disconnect-drag (`origin_cable` is Some) is released over empty space the
/// system must despawn the cable entity and clear `connected_cable` on whichever socket
/// still references it. Without this, the player is left with a dangling cable entity and
/// a socket that permanently reports itself as occupied — blocking any future connection
/// to that socket until the game is restarted.
#[test]
fn given_disconnect_drag_when_released_in_empty_space_then_cable_despawned_and_socket_cleared() {
    use engine_scene::prelude::{GlobalTransform2D, SortOrder};
    use glam::Affine2;

    // Arrange
    let mut world = make_pick_world();

    // The source socket — still occupied at drag-release time (the dest end was pulled free)
    let source_socket = world
        .spawn((
            Jack::<SignatureSpace> {
                direction: JackDirection::Output,
                data: None,
            },
            JackSocket {
                radius: 10.0,
                color: Color::WHITE,
                connected_cable: None, // will be set after cable spawn
            },
            Transform2D {
                position: Vec2::new(0.0, 0.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
            GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
            SortOrder::default(),
        ))
        .id();

    let dummy_dest = world.spawn_empty().id();
    let cable_entity = world
        .spawn((
            Cable {
                source: source_socket,
                dest: dummy_dest,
            },
            Transform2D::default(),
        ))
        .id();

    world
        .entity_mut(source_socket)
        .get_mut::<JackSocket>()
        .unwrap()
        .connected_cable = Some(cable_entity);

    world.insert_resource(PendingCable {
        source: Some(source_socket),
        origin_cable: Some(cable_entity),
        free_end: None,
    });
    let mut mouse = MouseState::default();
    mouse.release(MouseButton::Left);
    mouse.set_world_pos(Vec2::new(9999.0, 9999.0)); // empty space
    world.insert_resource(mouse);

    let mut schedule = Schedule::default();
    schedule.add_systems(jack_socket_release_system);

    // Act
    schedule.run(&mut world);

    // Assert — cable entity despawned
    assert!(
        world.get_entity(cable_entity).is_err(),
        "cable entity must be despawned when disconnect-drag is released in empty space"
    );
    // Assert — source socket unoccupied
    assert!(
        world
            .get::<JackSocket>(source_socket)
            .unwrap()
            .connected_cable
            .is_none(),
        "source socket must be cleared after its cable is destroyed"
    );
}

// ---------------------------------------------------------------------------
// TC-OCC-06 (spec TC020) — disconnect drag over free socket re-attaches cable
// ---------------------------------------------------------------------------

/// @doc: When a disconnect-drag (`origin_cable` is Some) is released over a compatible free
/// socket, the existing cable entity must be mutated in-place (`Cable.dest` updated) and
/// the new socket must be marked occupied. Reusing the entity avoids invalidating the
/// `connected_cable` handle stored on the source socket — spawning a replacement entity
/// would leave the source socket pointing at a stale (now-despawned) entity id.
#[test]
fn given_disconnect_drag_when_released_over_free_compatible_socket_then_cable_reattached_to_new_socket()
 {
    use engine_scene::prelude::{GlobalTransform2D, SortOrder};
    use glam::Affine2;

    // Arrange
    let mut world = make_pick_world();

    let source_socket = world
        .spawn((
            Jack::<SignatureSpace> {
                direction: JackDirection::Output,
                data: None,
            },
            JackSocket {
                radius: 10.0,
                color: Color::WHITE,
                connected_cable: None, // will be set after cable spawn
            },
            Transform2D {
                position: Vec2::new(0.0, 0.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
            GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
            SortOrder::default(),
        ))
        .id();

    let new_dest = world
        .spawn((
            Jack::<SignatureSpace> {
                direction: JackDirection::Input,
                data: None,
            },
            JackSocket {
                radius: 10.0,
                color: Color::WHITE,
                connected_cable: None, // free
            },
            Transform2D {
                position: Vec2::new(50.0, 0.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
            GlobalTransform2D(Affine2::from_translation(Vec2::new(50.0, 0.0))),
            SortOrder::default(),
        ))
        .id();

    let stale_dest = world.spawn_empty().id();
    let cable_entity = world
        .spawn((
            Cable {
                source: source_socket,
                dest: stale_dest,
            },
            Transform2D::default(),
        ))
        .id();

    world
        .entity_mut(source_socket)
        .get_mut::<JackSocket>()
        .unwrap()
        .connected_cable = Some(cable_entity);

    world.insert_resource(PendingCable {
        source: Some(source_socket),
        origin_cable: Some(cable_entity),
        free_end: None,
    });
    let mut mouse = MouseState::default();
    mouse.release(MouseButton::Left);
    mouse.set_world_pos(Vec2::new(50.0, 0.0)); // cursor on new_dest
    world.insert_resource(mouse);

    let mut schedule = Schedule::default();
    schedule.add_systems(jack_socket_release_system);

    // Act
    schedule.run(&mut world);

    // Assert — same cable entity, dest updated
    let cable = world.get::<Cable>(cable_entity).unwrap();
    assert_eq!(
        cable.dest, new_dest,
        "cable.dest must point at the new destination socket after re-attach"
    );

    // Assert — new socket is now occupied
    let new_socket = world.get::<JackSocket>(new_dest).unwrap();
    assert_eq!(
        new_socket.connected_cable,
        Some(cable_entity),
        "new destination socket must be marked occupied with the reattached cable entity"
    );
}

// ---------------------------------------------------------------------------
// TC-F05 — clicking a socket must spawn a WireEndpoints immediately
// ---------------------------------------------------------------------------

/// @doc: The `WireEndpoints` on the immediately-spawned wire must reference the clicked
/// socket so that the wire render system draws from the correct world position each
/// tick. If the wire is only spawned on mouse release (the old behaviour), the player gets no
/// visual cable feedback during the drag — the wire materialises after the drop instead of
/// tracking the cursor in real time, making the wiring interaction feel broken.
#[test]
fn when_jack_socket_clicked_then_spawned_wire_source_endpoint_matches_socket() {
    // Arrange
    let mut world = make_pick_world();
    let socket = spawn_jack_socket_at(&mut world, Vec2::new(100.0, 100.0));
    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Left);
    mouse.set_world_pos(Vec2::new(100.0, 100.0));
    world.insert_resource(mouse);

    // Act
    run_click_resolve(&mut world);

    // Assert
    let mut q = world.query::<&WireEndpoints>();
    let endpoints: Vec<_> = q.iter(&world).collect();
    assert_eq!(
        endpoints.len(),
        1,
        "exactly one WireEndpoints must exist after socket click"
    );
    assert_eq!(
        endpoints[0].source, socket,
        "WireEndpoints.source must be the clicked socket entity"
    );
}

// ---------------------------------------------------------------------------
// TC-F06 — free end of the in-flight rope tracks cursor position
// ---------------------------------------------------------------------------

/// @doc: The `CableFreeEnd` entity spawned during a cable drag must have its `Transform2D`
/// position updated to the cursor's world position each frame by `pending_cable_drag_system`.
/// Without this update the rope's dest endpoint stays at the socket's position and the
/// rope collapses to a zero-length line instead of stretching toward the cursor, giving
/// the player no spatial feedback about where they are routing the cable.
#[test]
fn when_pending_cable_drag_system_runs_then_free_end_position_matches_cursor() {
    // Arrange
    let (mut app, _shape_calls) = make_preview_app();
    let world = app.world_mut();

    let socket = world
        .spawn((
            JackSocket {
                radius: 8.0,
                color: Color::WHITE,
                connected_cable: None,
            },
            Transform2D {
                position: Vec2::new(50.0, 50.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();

    let free_end = world
        .spawn((
            CableFreeEnd,
            Transform2D {
                position: Vec2::new(50.0, 50.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();

    world.insert_resource(PendingCable {
        source: Some(socket),
        origin_cable: None,
        free_end: Some(free_end),
    });

    let cursor_pos = Vec2::new(200.0, 150.0);
    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Left);
    mouse.set_world_pos(cursor_pos);
    world.insert_resource(mouse);

    // Act
    let mut schedule = Schedule::default();
    schedule.add_systems(pending_cable_drag_system);
    schedule.run(app.world_mut());

    // Assert
    let free_end_pos = app
        .world_mut()
        .get::<Transform2D>(free_end)
        .unwrap()
        .position;
    assert!(
        (free_end_pos - cursor_pos).length() < 0.01,
        "CableFreeEnd position must match cursor world pos {cursor_pos}, got {free_end_pos}"
    );
}

