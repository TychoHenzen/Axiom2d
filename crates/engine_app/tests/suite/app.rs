#![allow(clippy::unwrap_used)]

use std::cell::Cell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use engine_app::app::format_window_title;
use engine_app::prelude::*;
use engine_core::color::Color;
use engine_core::time::{DeltaTime, FixedTimestep};
use engine_core::types::{Pixels, Seconds};
use engine_ecs::prelude::{Phase, Res, ResMut, Resource};
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

/// @doc: Phase execution order is enforced across all 18 phases. `FixedUpdate` fires once when
/// delta equals `step_size`. Startup executes only on the first frame.
#[test]
fn when_systems_in_all_phases_then_run_in_canonical_order() {
    #[derive(Resource, Default)]
    struct Log(Vec<&'static str>);

    fn log_startup(mut log: ResMut<Log>) {
        log.0.push("startup");
    }
    fn log_on_enable(mut log: ResMut<Log>) {
        log.0.push("on_enable");
    }
    fn log_fixed_update(mut log: ResMut<Log>) {
        log.0.push("fixed_update");
    }
    fn log_async_fixed_update(mut log: ResMut<Log>) {
        log.0.push("async_fixed_update");
    }
    fn log_on_collision(mut log: ResMut<Log>) {
        log.0.push("on_collision");
    }
    fn log_input(mut log: ResMut<Log>) {
        log.0.push("input");
    }
    fn log_update(mut log: ResMut<Log>) {
        log.0.push("update");
    }
    fn log_async(mut log: ResMut<Log>) {
        log.0.push("async");
    }
    fn log_animate(mut log: ResMut<Log>) {
        log.0.push("animate");
    }
    fn log_late_update(mut log: ResMut<Log>) {
        log.0.push("late_update");
    }
    fn log_on_became_visible(mut log: ResMut<Log>) {
        log.0.push("on_became_visible");
    }
    fn log_render(mut log: ResMut<Log>) {
        log.0.push("render");
    }
    fn log_post_render(mut log: ResMut<Log>) {
        log.0.push("post_render");
    }
    fn log_async_end_of_frame(mut log: ResMut<Log>) {
        log.0.push("async_end_of_frame");
    }
    fn log_on_pause(mut log: ResMut<Log>) {
        log.0.push("on_pause");
    }
    fn log_on_disable(mut log: ResMut<Log>) {
        log.0.push("on_disable");
    }
    fn log_on_destroy(mut log: ResMut<Log>) {
        log.0.push("on_destroy");
    }
    fn log_wait_for_vblank(mut log: ResMut<Log>) {
        log.0.push("wait_for_vblank");
    }

    // Arrange
    let mut app = App::new();
    app.world_mut().insert_resource(Log::default());
    // delta = step_size → FixedUpdate fires exactly once
    app.world_mut().insert_resource(DeltaTime(Seconds(0.1)));
    app.world_mut()
        .insert_resource(FixedTimestep::with_step_size(Seconds(0.1)));
    app.add_systems(Phase::Startup, log_startup);
    app.add_systems(Phase::OnEnable, log_on_enable);
    app.add_systems(Phase::FixedUpdate, log_fixed_update);
    app.add_systems(Phase::AsyncFixedUpdate, log_async_fixed_update);
    app.add_systems(Phase::OnCollision, log_on_collision);
    app.add_systems(Phase::Input, log_input);
    app.add_systems(Phase::Update, log_update);
    app.add_systems(Phase::Async, log_async);
    app.add_systems(Phase::Animate, log_animate);
    app.add_systems(Phase::LateUpdate, log_late_update);
    app.add_systems(Phase::OnBecameVisible, log_on_became_visible);
    app.add_systems(Phase::Render, log_render);
    app.add_systems(Phase::PostRender, log_post_render);
    app.add_systems(Phase::AsyncEndOfFrame, log_async_end_of_frame);
    app.add_systems(Phase::OnPause, log_on_pause);
    app.add_systems(Phase::OnDisable, log_on_disable);
    app.add_systems(Phase::OnDestroy, log_on_destroy);
    app.add_systems(Phase::WaitForVBlank, log_wait_for_vblank);

    // Act
    app.handle_redraw();

    // Assert
    assert_eq!(
        app.world().resource::<Log>().0,
        vec![
            "startup",
            "on_enable",
            "fixed_update",
            "async_fixed_update",
            "on_collision",
            "input",
            "update",
            "async",
            "animate",
            "late_update",
            "on_became_visible",
            "render",
            "post_render",
            "async_end_of_frame",
            "on_pause",
            "on_disable",
            "on_destroy",
            "wait_for_vblank",
        ]
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

/// @doc: App must initialize all 18 phase schedules — a mismatch between `PHASE_COUNT` and the enum
/// variants would silently drop a phase and break every system registered to it.
#[test]
fn when_new_app_created_then_eighteen_schedules_exist() {
    // Act
    let app = App::new();

    // Assert
    assert_eq!(app.schedule_count(), 18);
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

/// @doc: Phase execution order is fixed: Input runs before Update
#[test]
fn when_handle_redraw_called_then_input_runs_before_update() {
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
    app.add_systems(Phase::Input, time_system);
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

/// @doc: Cursor moves must reach the mouse input bus — stale position breaks drag and hover feedback
#[test]
fn when_cursor_moved_event_received_by_app_then_event_pushed_to_bus() {
    // Arrange
    let mut app = App::new();
    app.world_mut()
        .insert_resource(engine_core::prelude::EventBus::<
            engine_input::prelude::MouseInputEvent,
        >::default());

    // Act
    app.handle_cursor_moved(glam::Vec2::new(320.0, 240.0));

    // Assert
    let events: Vec<_> = app
        .world_mut()
        .resource_mut::<engine_core::prelude::EventBus<engine_input::prelude::MouseInputEvent>>()
        .drain()
        .collect();
    assert_eq!(
        events.as_slice(),
        &[engine_input::prelude::MouseInputEvent::Move {
            screen_pos: glam::Vec2::new(320.0, 240.0),
        }]
    );
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
        engine_input::prelude::MouseInputEvent::Button {
            button: engine_input::prelude::MouseButton::Right,
            state: ButtonState::Pressed,
        }
    );
}


/// @doc: Scroll events must reach the mouse input bus — stale scroll breaks wheel input
#[test]
fn when_scroll_event_received_by_app_then_event_pushed_to_bus() {
    // Arrange
    let mut app = App::new();
    app.world_mut()
        .insert_resource(engine_core::prelude::EventBus::<
            engine_input::prelude::MouseInputEvent,
        >::default());

    // Act
    app.handle_mouse_wheel(glam::Vec2::new(0.0, 3.0));

    // Assert
    let events: Vec<_> = app
        .world_mut()
        .resource_mut::<engine_core::prelude::EventBus<engine_input::prelude::MouseInputEvent>>()
        .drain()
        .collect();
    assert_eq!(
        events.as_slice(),
        &[engine_input::prelude::MouseInputEvent::Scroll {
            delta: glam::Vec2::new(0.0, 3.0),
        }]
    );
}

/// @doc: Phase timing records 18 entries per frame — one per schedule phase. The profiler loop
/// iterates `Phase::ALL` which contains all 18 phases. `FixedUpdate` must fire at least once to
/// record its entry, so we set `DeltaTime` to exactly one step. A count other than 18 means the
/// loop and the enum are out of sync.
#[test]
fn when_handle_redraw_called_with_profiler_then_eighteen_phase_records_buffered() {
    use std::path::PathBuf;

    // Arrange
    let mut app = App::new();
    app.world_mut()
        .insert_resource(engine_core::profiler::FrameProfiler::new(
            9999,
            PathBuf::from("unused.csv"),
        ));
    app.world_mut()
        .insert_resource(DeltaTime(Seconds(1.0 / 60.0)));
    app.world_mut().insert_resource(FixedTimestep::default());

    // Act
    app.handle_redraw();

    // Assert — one record per phase (flush_interval 9999 keeps them in memory)
    let profiler = app
        .world()
        .resource::<engine_core::profiler::FrameProfiler>();
    assert_eq!(profiler.record_count(), 18);
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

/// @doc: FPS title formatting must include the base title and rounded FPS count.
#[test]
fn when_format_window_title_called_then_title_includes_fps() {
    // Arrange
    let frame_time = Duration::from_secs_f64(1.0 / 60.0);

    // Act
    let title = format_window_title("Card Game", frame_time);

    // Assert
    assert_eq!(title, "Card Game - 60 FPS");
}

/// @doc: Startup systems must execute on the first `handle_redraw` call. Any system that populates
/// initial world state (spawn entities, set defaults) lives in Startup and must not be silently
/// skipped on frame zero.
#[test]
fn when_system_added_to_startup_phase_then_runs_on_first_handle_redraw() {
    // Arrange
    let mut app = App::new();
    app.world_mut().insert_resource(Counter(0));
    app.add_systems(Phase::Startup, increment);

    // Act
    app.handle_redraw();

    // Assert
    assert_eq!(app.world().resource::<Counter>().0, 1);
}

/// @doc: Startup systems must be one-shot — they run exactly once on the first frame and must not
/// re-execute on subsequent frames. Re-running Startup would re-spawn entities and reset state on
/// every frame.
#[test]
fn when_startup_system_registered_then_does_not_run_on_second_handle_redraw() {
    // Arrange
    let mut app = App::new();
    app.world_mut().insert_resource(Counter(0));
    app.add_systems(Phase::Startup, increment);

    // Act
    app.handle_redraw();
    app.handle_redraw();

    // Assert
    assert_eq!(app.world().resource::<Counter>().0, 1);
}

/// @doc: `FixedUpdate` must execute N times per frame where N = `floor(accumulated_delta` / `step_size`).
/// With delta=0.2s and `step_size=0.1s` the accumulator fires twice.
#[test]
fn when_fixed_update_system_registered_and_delta_covers_two_steps_then_runs_twice() {
    // Arrange
    let mut app = App::new();
    app.world_mut().insert_resource(Counter(0));
    app.world_mut().insert_resource(DeltaTime(Seconds(0.2)));
    app.world_mut()
        .insert_resource(FixedTimestep::with_step_size(Seconds(0.1)));
    app.add_systems(Phase::FixedUpdate, increment);

    // Act
    app.handle_redraw();

    // Assert
    assert_eq!(app.world().resource::<Counter>().0, 2);
}

/// @doc: `FixedUpdate` must not run when delta is zero — a zero-delta frame produces zero accumulator
/// steps, so physics and game logic that depend on deterministic fixed ticks must not fire.
#[test]
fn when_delta_is_zero_then_fixed_update_system_does_not_run() {
    // Arrange
    let mut app = App::new();
    app.world_mut().insert_resource(Counter(0));
    app.world_mut().insert_resource(DeltaTime(Seconds(0.0)));
    app.world_mut()
        .insert_resource(FixedTimestep::with_step_size(Seconds(0.1)));
    app.add_systems(Phase::FixedUpdate, increment);

    // Act
    app.handle_redraw();

    // Assert
    assert_eq!(app.world().resource::<Counter>().0, 0);
}

/// @doc: `FixedUpdate` must not fire when the accumulated delta has not yet reached one full step.
/// A delta of 0.05s with a 0.1s step size produces zero steps; the remainder carries forward.
#[test]
fn when_delta_less_than_step_size_then_fixed_update_does_not_run() {
    // Arrange
    let mut app = App::new();
    app.world_mut().insert_resource(Counter(0));
    app.world_mut().insert_resource(DeltaTime(Seconds(0.05)));
    app.world_mut()
        .insert_resource(FixedTimestep::with_step_size(Seconds(0.1)));
    app.add_systems(Phase::FixedUpdate, increment);

    // Act
    app.handle_redraw();

    // Assert
    assert_eq!(app.world().resource::<Counter>().0, 0);
}

/// @doc: The `FixedUpdate` accumulator must carry its remainder across frames. Frame 1 adds 0.05s
/// (no step fires, remainder=0.05s). Frame 2 adds 0.07s (total=0.12s, one step fires,
/// remainder=0.02s). The counter must be exactly 1 after both frames.
#[test]
fn when_accumulated_remainder_carries_over_then_second_frame_fires_expected_steps() {
    // Arrange
    let mut app = App::new();
    app.world_mut().insert_resource(Counter(0));
    app.world_mut().insert_resource(DeltaTime(Seconds(0.05)));
    app.world_mut()
        .insert_resource(FixedTimestep::with_step_size(Seconds(0.1)));
    app.add_systems(Phase::FixedUpdate, increment);

    // Act — frame 1: delta=0.05s, accumulator reaches 0.05s → 0 steps
    app.handle_redraw();
    // Act — frame 2: delta=0.07s, accumulator reaches 0.12s → 1 step, remainder 0.02s
    app.world_mut().insert_resource(DeltaTime(Seconds(0.07)));
    app.handle_redraw();

    // Assert
    assert_eq!(app.world().resource::<Counter>().0, 1);
}

/// @doc: Startup must execute before Update within the same `handle_redraw` call, so that
/// resources or entities written by Startup are visible to Update on frame zero. A game that
/// spawns its player in Startup and processes them in Update must not stutter for one frame.
#[test]
fn when_startup_system_runs_then_update_system_observes_side_effects_same_frame() {
    #[derive(Resource)]
    struct Flag(bool);

    fn set_flag(mut flag: ResMut<Flag>) {
        flag.0 = true;
    }

    fn assert_flag_set(flag: Res<Flag>, mut counter: ResMut<Counter>) {
        if flag.0 {
            counter.0 += 1;
        }
    }

    // Arrange
    let mut app = App::new();
    app.world_mut().insert_resource(Flag(false));
    app.world_mut().insert_resource(Counter(0));
    app.add_systems(Phase::Startup, set_flag);
    app.add_systems(Phase::Update, assert_flag_set);

    // Act
    app.handle_redraw();

    // Assert — Update must see Flag(true) set by Startup in the same frame
    assert_eq!(app.world().resource::<Counter>().0, 1);
}
