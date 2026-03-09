use std::collections::HashMap;
use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

use engine_core::types::Pixels;
use engine_ecs::prelude::{IntoScheduleConfigs, Phase, Schedule, ScheduleSystem, World};
use engine_render::prelude::RendererRes;
use engine_render::window::WindowConfig;

use crate::window_size::WindowSize;

pub trait Plugin {
    fn build(&self, app: &mut App);
}

const PHASE_ORDER: [Phase; 5] = [
    Phase::Input,
    Phase::PreUpdate,
    Phase::Update,
    Phase::PostUpdate,
    Phase::Render,
];

pub struct App {
    plugin_count: usize,
    window_config: WindowConfig,
    window: Option<Arc<Window>>,
    world: World,
    schedules: HashMap<Phase, Schedule>,
}

impl App {
    pub fn new() -> Self {
        let mut world = World::new();
        world.insert_resource(WindowSize::default());
        Self {
            plugin_count: 0,
            window_config: WindowConfig::default(),
            window: None,
            world,
            schedules: PHASE_ORDER
                .iter()
                .map(|&phase| (phase, Schedule::new(phase)))
                .collect(),
        }
    }

    pub fn set_window_config(&mut self, config: WindowConfig) -> &mut Self {
        self.world.insert_resource(WindowSize {
            width: Pixels(config.width as f32),
            height: Pixels(config.height as f32),
        });
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
        self.schedules.len()
    }

    pub fn add_systems<M>(&mut self, phase: Phase, systems: impl IntoScheduleConfigs<ScheduleSystem, M>) -> &mut Self {
        self.schedules.get_mut(&phase).unwrap().add_systems(systems);
        self
    }

    #[cfg(test)]
    pub(crate) fn set_renderer(&mut self, renderer: Box<dyn engine_render::renderer::Renderer + Send + Sync>) {
        self.world.insert_resource(RendererRes::new(renderer));
    }

    pub(crate) fn handle_resize(&mut self, width: u32, height: u32) {
        self.world.insert_resource(WindowSize {
            width: Pixels(width as f32),
            height: Pixels(height as f32),
        });
        if let Some(mut renderer) = self.world.get_resource_mut::<RendererRes>() {
            renderer.resize(width, height);
        }
    }

    pub(crate) fn handle_redraw(&mut self) {
        for phase in PHASE_ORDER {
            self.schedules.get_mut(&phase).unwrap().run(&mut self.world);
        }
        if let Some(mut renderer) = self.world.get_resource_mut::<RendererRes>() {
            renderer.present();
        }
    }

    pub fn run(&mut self) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.run_app(self).unwrap();
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
                self.window_config.width as f64,
                self.window_config.height as f64,
            ))
            .with_resizable(self.window_config.resizable);
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        let renderer = engine_render::create_renderer(window.clone(), &self.window_config);

        self.window = Some(window);
        self.world.insert_resource(RendererRes::new(renderer));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => self.handle_resize(size.width, size.height),
            WindowEvent::RedrawRequested => self.handle_redraw(),
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

    #[derive(Resource)]
    struct Counter(u32);

    fn increment(mut counter: ResMut<Counter>) {
        counter.0 += 1;
    }

    #[test]
    fn when_app_new_called_then_plugin_count_is_zero() {
        // Act
        let app = App::new();

        // Assert
        assert_eq!(app.plugin_count(), 0);
    }

    struct NoOpPlugin;
    impl Plugin for NoOpPlugin {
        fn build(&self, _app: &mut App) {}
    }

    struct AnotherNoOpPlugin;
    impl Plugin for AnotherNoOpPlugin {
        fn build(&self, _app: &mut App) {}
    }

    #[test]
    fn when_add_plugin_chained_twice_then_does_not_panic() {
        // Arrange
        let mut app = App::new();

        // Act
        app.add_plugin(NoOpPlugin).add_plugin(AnotherNoOpPlugin);
    }

    #[test]
    fn when_one_plugin_added_then_plugin_count_is_one() {
        // Arrange
        let mut app = App::new();

        // Act
        app.add_plugin(NoOpPlugin);

        // Assert
        assert_eq!(app.plugin_count(), 1);
    }

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

    #[test]
    fn when_plugin_added_then_build_called_exactly_once() {
        // Arrange
        let counter = Rc::new(Cell::new(0u32));
        let plugin = CountingPlugin { counter: Rc::clone(&counter) };

        // Act
        App::new().add_plugin(plugin);

        // Assert
        assert_eq!(counter.get(), 1);
    }

    #[test]
    fn when_app_default_called_then_plugin_count_is_zero() {
        // Act
        let app = App::default();

        // Assert
        assert_eq!(app.plugin_count(), 0);
    }

    #[test]
    fn when_set_window_config_called_then_config_is_stored() {
        // Arrange
        let mut app = App::new();
        let config = WindowConfig {
            title: "Test",
            width: 800,
            height: 600,
            vsync: false,
            resizable: false,
        };

        // Act
        app.set_window_config(config);

        // Assert
        assert_eq!(app.window_config, config);
    }

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

    #[test]
    fn when_handle_redraw_called_without_renderer_res_then_does_not_panic() {
        // Arrange
        let mut app = App::new();

        // Act
        app.handle_redraw();
    }

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

    #[test]
    fn when_systems_in_all_phases_then_run_in_canonical_order() {
        #[derive(Resource, Default)]
        struct Log(Vec<&'static str>);

        fn log_input(mut log: ResMut<Log>) { log.0.push("input"); }
        fn log_pre_update(mut log: ResMut<Log>) { log.0.push("pre_update"); }
        fn log_update(mut log: ResMut<Log>) { log.0.push("update"); }
        fn log_post_update(mut log: ResMut<Log>) { log.0.push("post_update"); }
        fn log_render(mut log: ResMut<Log>) { log.0.push("render"); }

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

    #[test]
    fn when_add_systems_chained_then_builder_pattern_works() {
        fn noop() {}

        // Act
        let mut app = App::new();
        app.set_window_config(WindowConfig::default())
            .add_systems(Phase::Update, noop);
    }

    #[test]
    fn when_new_app_created_then_five_schedules_exist() {
        // Act
        let app = App::new();

        // Assert
        assert_eq!(app.schedule_count(), 5);
    }

    #[test]
    fn when_resource_inserted_into_app_world_then_value_is_readable() {
        #[derive(Resource)]
        struct Score(u32);

        // Arrange
        let mut app = App::new();

        // Act
        app.world_mut().insert_resource(Score(7));
        let result = app.world().resource::<Score>().0;

        // Assert
        assert_eq!(result, 7);
    }

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

    #[test]
    fn when_plugin_inserts_resource_then_resource_persists_after_build() {
        #[derive(Resource)]
        struct Gravity(f32);

        struct GravityPlugin;
        impl Plugin for GravityPlugin {
            fn build(&self, app: &mut App) {
                app.world_mut().insert_resource(Gravity(9.81));
            }
        }

        // Arrange
        let mut app = App::new();

        // Act
        app.add_plugin(GravityPlugin);

        // Assert
        let g = app.world().resource::<Gravity>().0;
        assert!((g - 9.81).abs() < f32::EPSILON);
    }

    #[test]
    fn when_set_renderer_called_then_renderer_res_present_in_world() {
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(Arc::clone(&log));
        let mut app = App::new();

        // Act
        app.set_renderer(Box::new(spy));

        // Assert
        assert!(app.world().get_resource::<RendererRes>().is_some());
    }

    #[test]
    fn when_app_created_then_renderer_res_not_yet_in_world() {
        // Act
        let app = App::new();

        // Assert
        assert!(app.world().get_resource::<RendererRes>().is_none());
    }

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

    #[test]
    fn when_app_new_called_then_window_size_resource_is_present() {
        // Act
        let app = App::new();

        // Assert
        let size = app.world().get_resource::<WindowSize>();
        assert!(size.is_some());
    }

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
