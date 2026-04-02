#![allow(clippy::unwrap_used)]

use std::cell::Cell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use engine_app::prelude::*;
use engine_core::color::Color;
use engine_core::types::Pixels;
use engine_ecs::prelude::{Phase, ResMut, Resource};
use engine_input::prelude::{ButtonState, KeyCode, KeyInputEvent};
use engine_render::prelude::RendererRes;
use engine_render::testing::SpyRenderer;
use engine_render::window::WindowConfig;

#[derive(Resource)]
struct Counter(u32);

fn increment(mut counter: ResMut<Counter>) {
    counter.0 += 1;
}

struct NoOpPlugin;
impl Plugin for NoOpPlugin {
    fn build(&self, _app: &mut App) {}
}

struct AnotherNoOpPlugin;
impl Plugin for AnotherNoOpPlugin {
    fn build(&self, _app: &mut App) {}
}

/// @doc: Plugin chaining must work — builder pattern foundation for configuration composition
#[test]
fn when_add_plugin_chained_twice_then_does_not_panic() {
    // Arrange
    let mut app = App::new();

    // Act
    app.add_plugin(NoOpPlugin).add_plugin(AnotherNoOpPlugin);
}

/// @doc: `plugin_count` must accurately track registrations — count mismatch indicates missing plugins
#[test]
fn when_one_plugin_added_then_plugin_count_is_one() {
    // Arrange
    let mut app = App::new();

    // Act
    app.add_plugin(NoOpPlugin);

    // Assert
    assert_eq!(app.plugin_count(), 1);
}

/// @doc: Multiple plugins must each increment count — counting must not collapse duplicate registrations
#[test]
fn when_two_distinct_plugins_added_then_plugin_count_is_two() {
    // Arrange
    let mut app = App::new();

    // Act
    app.add_plugin(NoOpPlugin).add_plugin(AnotherNoOpPlugin);

    // Assert
    assert_eq!(app.plugin_count(), 2);
}

struct CountingPlugin {
    counter: Rc<Cell<u32>>,
}

impl Plugin for CountingPlugin {
    fn build(&self, _app: &mut App) {
        self.counter.set(self.counter.get() + 1);
    }
}

/// @doc: Plugin build must be called exactly once — double-build duplicates systems and wastes memory
#[test]
fn when_plugin_added_then_build_called_exactly_once() {
    // Arrange
    let counter = Rc::new(Cell::new(0u32));
    let plugin = CountingPlugin {
        counter: Rc::clone(&counter),
    };

    // Act
    App::new().add_plugin(plugin);

    // Assert
    assert_eq!(counter.get(), 1);
}

/// @doc: `handle_redraw` must invoke `renderer.present()` — missing present swallows drawn frames
#[test]
fn when_handle_redraw_called_then_present_called_via_renderer_res() {
    // Arrange
    let log = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(Arc::clone(&log));

    let mut app = App::new();
    app.set_renderer(Box::new(spy));

    // Act
    app.handle_redraw();

    // Assert
    assert_eq!(log.lock().unwrap().as_slice(), &["present"]);
}

/// @doc: `handle_redraw` must handle missing renderer gracefully — prevents panic during headless testing
#[test]
fn when_handle_redraw_called_without_renderer_res_then_does_not_panic() {
    // Arrange
    let mut app = App::new();

    // Act
    app.handle_redraw();
}

/// @doc: Systems must run during `handle_redraw` — non-execution breaks game loop integration
#[test]
fn when_system_added_to_update_phase_then_runs_during_handle_redraw() {
    // Arrange
    let mut app = App::new();
    app.world_mut().insert_resource(Counter(0));
    app.add_systems(Phase::Update, increment);

    // Act
    app.handle_redraw();

    // Assert
    assert_eq!(app.world().resource::<Counter>().0, 1);
}

/// @doc: Systems must run per redraw call — missing re-execution breaks frame-rate scaling
#[test]
fn when_handle_redraw_called_twice_then_system_runs_twice() {
    // Arrange
    let mut app = App::new();
    app.world_mut().insert_resource(Counter(0));
    app.add_systems(Phase::Update, increment);

    // Act
    app.handle_redraw();
    app.handle_redraw();

    // Assert
    assert_eq!(app.world().resource::<Counter>().0, 2);
}

/// @doc: Phase execution order is enforced: Input→PreUpdate→Update→PostUpdate→Render — game logic depends on this
#[test]
fn when_systems_in_all_phases_then_run_in_canonical_order() {
    #[derive(Resource, Default)]
    struct Log(Vec<&'static str>);

    fn log_input(mut log: ResMut<Log>) {
        log.0.push("input");
    }
    fn log_pre_update(mut log: ResMut<Log>) {
        log.0.push("pre_update");
    }
    fn log_update(mut log: ResMut<Log>) {
        log.0.push("update");
    }
    fn log_post_update(mut log: ResMut<Log>) {
        log.0.push("post_update");
    }
    fn log_render(mut log: ResMut<Log>) {
        log.0.push("render");
    }

    // Arrange
    let mut app = App::new();
    app.world_mut().insert_resource(Log::default());
    app.add_systems(Phase::Input, log_input);
    app.add_systems(Phase::PreUpdate, log_pre_update);
    app.add_systems(Phase::Update, log_update);
    app.add_systems(Phase::PostUpdate, log_post_update);
    app.add_systems(Phase::Render, log_render);

    // Act
    app.handle_redraw();

    // Assert
    assert_eq!(
        app.world().resource::<Log>().0,
        vec!["input", "pre_update", "update", "post_update", "render"]
    );
}

/// @doc: `add_systems` must support builder chaining — broken chaining breaks fluent configuration API
#[test]
fn when_add_systems_chained_then_builder_pattern_works() {
    fn noop() {}

    // Act
    let mut app = App::new();
    app.set_window_config(WindowConfig::default())
        .add_systems(Phase::Update, noop);
}

/// @doc: App must initialize all 5 phase schedules — missing schedule breaks phase system behavior
#[test]
fn when_new_app_created_then_five_schedules_exist() {
    // Act
    let app = App::new();

    // Assert
    assert_eq!(app.schedule_count(), 5);
}

/// @doc: Draw calls always precede present — rendering into a swapped buffer would show stale frames
#[test]
fn when_render_phase_system_uses_renderer_res_then_draw_calls_precede_present() {
    fn render_system(mut renderer: ResMut<RendererRes>) {
        renderer.clear(Color::BLACK);
    }

    // Arrange
    let log = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(Arc::clone(&log));

    let mut app = App::new();
    app.add_systems(Phase::Render, render_system);
    app.set_renderer(Box::new(spy));

    // Act
    app.handle_redraw();

    // Assert
    assert_eq!(log.lock().unwrap().as_slice(), &["clear", "present"]);
}

/// @doc: Systems and present must both run — broken system execution or present skipping breaks rendering
#[test]
fn when_update_systems_exist_then_schedules_run_and_present_called() {
    // Arrange
    let log = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(Arc::clone(&log));

    let mut app = App::new();
    app.world_mut().insert_resource(Counter(0));
    app.add_systems(Phase::Update, increment);
    app.set_renderer(Box::new(spy));

    // Act
    app.handle_redraw();

    // Assert
    assert_eq!(app.world().resource::<Counter>().0, 1);
    assert_eq!(log.lock().unwrap().as_slice(), &["present"]);
}

/// @doc: Plugin-registered systems must run during `handle_redraw` — broken integration breaks plugin architecture
#[test]
fn when_plugin_calls_add_systems_then_system_runs_during_handle_redraw() {
    struct CounterPlugin;
    impl Plugin for CounterPlugin {
        fn build(&self, app: &mut App) {
            app.world_mut().insert_resource(Counter(0));
            app.add_systems(Phase::Update, increment);
        }
    }

    // Arrange
    let mut app = App::new();
    app.add_plugin(CounterPlugin);

    // Act
    app.handle_redraw();

    // Assert
    assert_eq!(app.world().resource::<Counter>().0, 1);
}

/// @doc: `handle_resize` must call `renderer.resize()` — missing resize breaks viewport on window change
#[test]
fn when_handle_resize_called_then_renderer_resize_is_called() {
    // Arrange
    let log = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(Arc::clone(&log));

    let mut app = App::new();
    app.set_renderer(Box::new(spy));

    // Act
    app.handle_resize(1024, 768);

    // Assert
    assert_eq!(log.lock().unwrap().as_slice(), &["resize"]);
}

/// @doc: Resize updates both the `WindowSize` resource and calls `renderer.resize()` — dual sync
#[test]
fn when_handle_resize_called_then_window_size_resource_is_updated() {
    // Arrange
    let mut app = App::new();
    let log = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(Arc::clone(&log));
    app.set_renderer(Box::new(spy));

    // Act
    app.handle_resize(1024, 768);

    // Assert
    let size = app.world().resource::<WindowSize>();
    assert_eq!(size.width, Pixels(1024.0));
    assert_eq!(size.height, Pixels(768.0));
}

/// @doc: Phase execution order is fixed: Input -> `PreUpdate` -> Update -> `PostUpdate` -> Render
#[test]
fn when_handle_redraw_called_then_pre_update_runs_before_update() {
    use engine_core::time::{ClockRes, DeltaTime, FakeClock, time_system};

    #[derive(Resource)]
    struct CapturedDelta(engine_core::types::Seconds);

    fn capture_delta(dt: engine_ecs::prelude::Res<DeltaTime>, mut captured: ResMut<CapturedDelta>) {
        captured.0 = dt.0;
    }

    // Arrange
    let mut app = App::new();
    let mut fake = FakeClock::default();
    fake.advance(engine_core::types::Seconds(0.016));
    app.world_mut()
        .insert_resource(ClockRes::new(Box::new(fake)));
    app.world_mut()
        .insert_resource(CapturedDelta(engine_core::types::Seconds(0.0)));
    app.add_systems(Phase::PreUpdate, time_system);
    app.add_systems(Phase::Update, capture_delta);

    // Act
    app.handle_redraw();

    // Assert
    let captured = app.world().resource::<CapturedDelta>();
    assert_eq!(captured.0, engine_core::types::Seconds(0.016));
}

/// @doc: Keyboard press events must reach `EventBus<KeyInputEvent>` — missing event prevents key input recognition
#[test]
fn when_app_receives_keyboard_press_then_event_pushed_to_bus() {
    // Arrange
    let mut app = App::new();
    app.world_mut()
        .insert_resource(engine_core::prelude::EventBus::<KeyInputEvent>::default());

    // Act
    app.handle_key_event(
        winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ArrowLeft),
        winit::event::ElementState::Pressed,
        false,
    );

    // Assert
    let events: Vec<_> = app
        .world_mut()
        .resource_mut::<engine_core::prelude::EventBus<KeyInputEvent>>()
        .drain()
        .collect();
    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0],
        KeyInputEvent {
            key: KeyCode::ArrowLeft,
            state: ButtonState::Pressed,
        }
    );
}

/// @doc: Keyboard release events must reach `EventBus<KeyInputEvent>` — missing releases trap keys pressed
#[test]
fn when_app_receives_keyboard_release_then_release_event_pushed_to_bus() {
    // Arrange
    let mut app = App::new();
    app.world_mut()
        .insert_resource(engine_core::prelude::EventBus::<KeyInputEvent>::default());

    // Act
    app.handle_key_event(
        winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ArrowLeft),
        winit::event::ElementState::Released,
        false,
    );

    // Assert
    let events: Vec<_> = app
        .world_mut()
        .resource_mut::<engine_core::prelude::EventBus<KeyInputEvent>>()
        .drain()
        .collect();
    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0],
        KeyInputEvent {
            key: KeyCode::ArrowLeft,
            state: ButtonState::Released,
        }
    );
}

/// @doc: Unidentified keys must not create events — unidentified key events would confuse input consumers
#[test]
fn when_app_receives_unidentified_physical_key_then_bus_remains_empty() {
    use winit::keyboard::NativeKeyCode;

    // Arrange
    let mut app = App::new();
    app.world_mut()
        .insert_resource(engine_core::prelude::EventBus::<KeyInputEvent>::default());

    // Act
    app.handle_key_event(
        winit::keyboard::PhysicalKey::Unidentified(NativeKeyCode::Unidentified),
        winit::event::ElementState::Pressed,
        false,
    );

    // Assert
    assert!(
        app.world()
            .resource::<engine_core::prelude::EventBus<KeyInputEvent>>()
            .is_empty()
    );
}

/// @doc: Synthetic keyboard events must be ignored — winit emits them when
/// a window gains focus, and treating them like real input can trigger
/// phantom hotkeys on startup.
#[test]
fn when_app_receives_synthetic_keyboard_press_then_bus_remains_empty() {
    // Arrange
    let mut app = App::new();
    app.world_mut()
        .insert_resource(engine_core::prelude::EventBus::<KeyInputEvent>::default());

    // Act
    app.handle_key_event(
        winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Digit2),
        winit::event::ElementState::Pressed,
        true,
    );

    // Assert
    assert!(
        app.world()
            .resource::<engine_core::prelude::EventBus<KeyInputEvent>>()
            .is_empty()
    );
}

/// @doc: Cursor moves must update `MouseState.screen_pos` — stale position breaks drag and hover feedback
#[test]
fn when_cursor_moved_event_received_by_app_then_screen_pos_updated() {
    // Arrange
    let mut app = App::new();
    app.world_mut()
        .insert_resource(engine_input::prelude::MouseState::default());

    // Act
    app.handle_cursor_moved(glam::Vec2::new(320.0, 240.0));

    // Assert
    let mouse = app.world().resource::<engine_input::prelude::MouseState>();
    assert_eq!(mouse.screen_pos(), glam::Vec2::new(320.0, 240.0));
}

/// @doc: Missing `MouseState` resource must not panic — handles optional mouse state gracefully
#[test]
fn when_cursor_moved_without_mouse_state_resource_then_does_not_panic() {
    // Arrange
    let mut app = App::new();

    // Act
    app.handle_cursor_moved(glam::Vec2::new(100.0, 100.0));
}

/// @doc: Mouse button events must reach `EventBus<MouseInputEvent>` — missing events break click/drag input
#[test]
fn when_mouse_button_event_received_by_app_then_event_pushed_to_bus() {
    // Arrange
    let mut app = App::new();
    app.world_mut()
        .insert_resource(engine_core::prelude::EventBus::<
            engine_input::prelude::MouseInputEvent,
        >::default());

    // Act
    app.handle_mouse_button(
        winit::event::MouseButton::Right,
        winit::event::ElementState::Pressed,
    );

    // Assert
    let events: Vec<_> = app
        .world_mut()
        .resource_mut::<engine_core::prelude::EventBus<engine_input::prelude::MouseInputEvent>>()
        .drain()
        .collect();
    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0],
        engine_input::prelude::MouseInputEvent {
            button: engine_input::prelude::MouseButton::Right,
            state: ButtonState::Pressed,
        }
    );
}

/// @doc: Missing `EventBus<MouseInputEvent>` must not panic — handles optional input bus gracefully
#[test]
fn when_mouse_button_event_received_without_bus_resource_then_does_not_panic() {
    // Arrange
    let mut app = App::new();

    // Act
    app.handle_mouse_button(
        winit::event::MouseButton::Left,
        winit::event::ElementState::Pressed,
    );
}

/// @doc: Scroll events must accumulate into `MouseState.scroll_delta` — stale scroll breaks wheel input
#[test]
fn when_scroll_event_received_by_app_then_mouse_state_scroll_delta_accumulated() {
    // Arrange
    let mut app = App::new();
    app.world_mut()
        .insert_resource(engine_input::prelude::MouseState::default());

    // Act
    app.handle_mouse_wheel(glam::Vec2::new(0.0, 3.0));

    // Assert
    let mouse = app.world().resource::<engine_input::prelude::MouseState>();
    assert_eq!(mouse.scroll_delta(), glam::Vec2::new(0.0, 3.0));
}

/// @doc: Phase timing records five entries per frame — one per schedule phase
#[test]
fn when_handle_redraw_called_with_profiler_then_five_phase_records_buffered() {
    use std::path::PathBuf;

    // Arrange
    let mut app = App::new();
    app.world_mut().insert_resource(
        engine_core::profiler::FrameProfiler::new(9999, PathBuf::from("unused.csv")),
    );

    // Act
    app.handle_redraw();

    // Assert — one record per phase (flush_interval 9999 keeps them in memory)
    let profiler = app
        .world()
        .resource::<engine_core::profiler::FrameProfiler>();
    assert_eq!(profiler.record_count(), 5);
}

/// @doc: `set_window_config` must update `WindowSize` resource — stale dimensions break layout calculations
#[test]
fn when_set_window_config_called_then_window_size_reflects_config() {
    // Arrange
    let mut app = App::new();
    let config = WindowConfig {
        width: 1920,
        height: 1080,
        ..WindowConfig::default()
    };

    // Act
    app.set_window_config(config);

    // Assert
    let size = app.world().resource::<WindowSize>();
    assert_eq!(size.width, Pixels(1920.0));
    assert_eq!(size.height, Pixels(1080.0));
}
