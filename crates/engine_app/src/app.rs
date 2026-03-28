use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowId};

use engine_input::button_state::ButtonState;
use engine_input::key_code::KeyCode;
use engine_input::mouse_button::MouseButton;

use engine_core::types::Pixels;
use engine_ecs::prelude::{
    IntoScheduleConfigs, PHASE_COUNT, Phase, Schedule, ScheduleSystem, World,
};
use engine_input::prelude::{InputEventBuffer, MouseEventBuffer, MouseState};
use engine_render::prelude::RendererRes;
use engine_render::window::WindowConfig;

use crate::window_size::WindowSize;

pub trait Plugin {
    fn build(&self, app: &mut App);
}

pub struct App {
    plugin_count: usize,
    window_config: WindowConfig,
    window: Option<Arc<Window>>,
    world: World,
    schedules: [Schedule; PHASE_COUNT],
}

impl App {
    pub fn new() -> Self {
        let mut world = World::new();
        world.insert_resource(WindowSize::default());
        world.insert_resource(engine_core::time::DeltaTime::default());
        Self {
            plugin_count: 0,
            window_config: WindowConfig::default(),
            window: None,
            world,
            schedules: Phase::ALL.map(Schedule::new),
        }
    }

    pub fn set_window_config(&mut self, config: WindowConfig) -> &mut Self {
        self.update_window_size(config.width, config.height);
        self.window_config = config;
        self
    }

    pub fn add_plugin(&mut self, plugin: impl Plugin) -> &mut Self {
        plugin.build(self);
        self.plugin_count += 1;
        self
    }

    pub fn plugin_count(&self) -> usize {
        self.plugin_count
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    pub fn schedule_count(&self) -> usize {
        PHASE_COUNT
    }

    pub fn add_systems<M>(
        &mut self,
        phase: Phase,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self {
        self.schedules[phase.index()].add_systems(systems);
        self
    }

    #[cfg(test)]
    pub(crate) fn set_renderer(
        &mut self,
        renderer: Box<dyn engine_render::renderer::Renderer + Send + Sync>,
    ) {
        self.world.insert_resource(RendererRes::new(renderer));
    }

    pub(crate) fn handle_key_event(
        &mut self,
        physical_key: PhysicalKey,
        state: winit::event::ElementState,
    ) {
        if let PhysicalKey::Code(winit_key) = physical_key
            && let Some(mut buffer) = self.world.get_resource_mut::<InputEventBuffer>()
        {
            buffer.push(KeyCode::from(winit_key), ButtonState::from(state));
        }
    }

    pub(crate) fn handle_cursor_moved(&mut self, position: glam::Vec2) {
        if let Some(mut mouse) = self.world.get_resource_mut::<MouseState>() {
            mouse.set_screen_pos(position);
        }
    }

    pub(crate) fn handle_mouse_button(
        &mut self,
        button: winit::event::MouseButton,
        state: winit::event::ElementState,
    ) {
        if let Some(mut buffer) = self.world.get_resource_mut::<MouseEventBuffer>() {
            buffer.push(MouseButton::from(button), ButtonState::from(state));
        }
    }

    pub(crate) fn handle_mouse_wheel(&mut self, delta: glam::Vec2) {
        if let Some(mut mouse) = self.world.get_resource_mut::<MouseState>() {
            mouse.add_scroll_delta(delta);
        }
    }

    fn update_window_size(&mut self, width: u32, height: u32) {
        self.world.insert_resource(WindowSize {
            width: Pixels(width as f32),
            height: Pixels(height as f32),
        });
    }

    pub(crate) fn handle_resize(&mut self, width: u32, height: u32) {
        self.update_window_size(width, height);
        if let Some(mut renderer) = self.world.get_resource_mut::<RendererRes>() {
            renderer.resize(width, height);
        }
    }

    pub fn handle_redraw(&mut self) {
        for schedule in &mut self.schedules {
            schedule.run(&mut self.world);
        }
        if let Some(mut renderer) = self.world.get_resource_mut::<RendererRes>() {
            renderer.present();
        }
    }

    pub fn run(&mut self) {
        // INVARIANT: EventLoop::new() only fails if the OS windowing system is
        // unavailable (e.g. no display server). No recovery is possible.
        let event_loop = EventLoop::new().expect("failed to create event loop");
        // INVARIANT: run_app() only fails on OS-level event loop errors.
        // The game cannot continue without an event loop.
        event_loop
            .run_app(self)
            .expect("event loop exited with error");
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = Window::default_attributes()
            .with_title(self.window_config.title)
            .with_inner_size(winit::dpi::LogicalSize::new(
                f64::from(self.window_config.width),
                f64::from(self.window_config.height),
            ))
            .with_resizable(self.window_config.resizable)
            .with_visible(false);
        // INVARIANT: create_window() only fails if the OS cannot allocate a
        // window (out of resources, no display). No recovery is possible.
        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .expect("failed to create window"),
        );
        let renderer = engine_render::create_renderer(window.clone(), &self.window_config);

        self.world.insert_resource(RendererRes::new(renderer));
        self.window = Some(window.clone());

        // Reset the clock so the first DeltaTime is near-zero, not the
        // seconds spent on GPU initialization.
        self.world
            .insert_resource(engine_core::prelude::ClockRes::new(Box::new(
                engine_core::time::SystemClock::default(),
            )));

        // Render a few frames while the window is still hidden to ensure
        // the GPU has fully presented before the window becomes visible.
        // This avoids the white flash from the OS compositor.
        for _ in 0..3 {
            self.handle_redraw();
        }
        window.set_visible(true);
        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => self.handle_resize(size.width, size.height),
            WindowEvent::RedrawRequested => self.handle_redraw(),
            WindowEvent::KeyboardInput { event, .. } => {
                self.handle_key_event(event.physical_key, event.state);
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.handle_cursor_moved(glam::Vec2::new(position.x as f32, position.y as f32));
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.handle_mouse_button(button, state);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll = match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => glam::Vec2::new(x, y),
                    winit::event::MouseScrollDelta::PixelDelta(pos) => {
                        glam::Vec2::new(pos.x as f32, pos.y as f32)
                    }
                };
                self.handle_mouse_wheel(scroll);
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;
    use std::sync::{Arc, Mutex};

    use engine_core::color::Color;
    use engine_core::types::Pixels;
    use engine_ecs::prelude::{Phase, ResMut, Resource};
    use engine_render::prelude::RendererRes;
    use engine_render::testing::SpyRenderer;

    use crate::window_size::WindowSize;
    use engine_input::prelude::{ButtonState, InputEventBuffer, KeyCode};

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

        fn capture_delta(
            dt: engine_ecs::prelude::Res<DeltaTime>,
            mut captured: ResMut<CapturedDelta>,
        ) {
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

    /// @doc: Keyboard press events must reach `InputEventBuffer` — missing event prevents key input recognition
    #[test]
    fn when_app_receives_keyboard_press_then_event_pushed_to_buffer() {
        // Arrange
        let mut app = App::new();
        app.world_mut().insert_resource(InputEventBuffer::default());

        // Act
        app.handle_key_event(
            winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ArrowLeft),
            winit::event::ElementState::Pressed,
        );

        // Assert
        let mut buffer = app.world_mut().resource_mut::<InputEventBuffer>();
        let events: Vec<_> = buffer.drain().collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], (KeyCode::ArrowLeft, ButtonState::Pressed));
    }

    /// @doc: Keyboard release events must reach `InputEventBuffer` — missing releases trap keys pressed
    #[test]
    fn when_app_receives_keyboard_release_then_release_event_pushed_to_buffer() {
        // Arrange
        let mut app = App::new();
        app.world_mut().insert_resource(InputEventBuffer::default());

        // Act
        app.handle_key_event(
            winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ArrowLeft),
            winit::event::ElementState::Released,
        );

        // Assert
        let mut buffer = app.world_mut().resource_mut::<InputEventBuffer>();
        let events: Vec<_> = buffer.drain().collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], (KeyCode::ArrowLeft, ButtonState::Released));
    }

    /// @doc: Unidentified keys must not create events — unidentified key events would confuse input consumers
    #[test]
    fn when_app_receives_unidentified_physical_key_then_buffer_remains_empty() {
        use winit::keyboard::NativeKeyCode;

        // Arrange
        let mut app = App::new();
        app.world_mut().insert_resource(InputEventBuffer::default());

        // Act
        app.handle_key_event(
            winit::keyboard::PhysicalKey::Unidentified(NativeKeyCode::Unidentified),
            winit::event::ElementState::Pressed,
        );

        // Assert
        let mut buffer = app.world_mut().resource_mut::<InputEventBuffer>();
        assert_eq!(buffer.drain().count(), 0);
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

    /// @doc: Mouse button events must reach `MouseEventBuffer` — missing events break click/drag input
    #[test]
    fn when_mouse_button_event_received_by_app_then_event_pushed_to_buffer() {
        // Arrange
        let mut app = App::new();
        app.world_mut()
            .insert_resource(engine_input::prelude::MouseEventBuffer::default());

        // Act
        app.handle_mouse_button(
            winit::event::MouseButton::Right,
            winit::event::ElementState::Pressed,
        );

        // Assert
        let mut buffer = app
            .world_mut()
            .resource_mut::<engine_input::prelude::MouseEventBuffer>();
        let events: Vec<_> = buffer.drain().collect();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            (
                engine_input::prelude::MouseButton::Right,
                ButtonState::Pressed
            )
        );
    }

    /// @doc: Missing `MouseEventBuffer` must not panic — handles optional input buffer gracefully
    #[test]
    fn when_mouse_button_event_received_without_buffer_resource_then_does_not_panic() {
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
}
