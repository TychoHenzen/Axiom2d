use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

use engine_ecs::prelude::{IntoScheduleConfigs, Phase, Schedule, ScheduleSystem, World};
use engine_render::renderer::Renderer;
use engine_render::window::WindowConfig;

pub trait Plugin {
    fn build(&self, app: &mut App);
}

pub struct App {
    plugin_count: usize,
    window_config: WindowConfig,
    render_fn: Option<Box<dyn FnMut(&mut dyn Renderer)>>,
    window: Option<Arc<Window>>,
    renderer: Option<Box<dyn Renderer>>,
    world: World,
    schedules: Vec<Schedule>,
}

impl App {
    pub fn new() -> Self {
        Self {
            plugin_count: 0,
            window_config: WindowConfig::default(),
            render_fn: None,
            window: None,
            renderer: None,
            world: World::new(),
            schedules: vec![
                Schedule::new(Phase::Input),
                Schedule::new(Phase::PreUpdate),
                Schedule::new(Phase::Update),
                Schedule::new(Phase::PostUpdate),
                Schedule::new(Phase::Render),
            ],
        }
    }

    pub fn set_window_config(&mut self, config: WindowConfig) -> &mut Self {
        self.window_config = config;
        self
    }

    pub fn on_render(&mut self, f: impl FnMut(&mut dyn Renderer) + 'static) -> &mut Self {
        self.render_fn = Some(Box::new(f));
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
        let idx = match phase {
            Phase::Input => 0,
            Phase::PreUpdate => 1,
            Phase::Update => 2,
            Phase::PostUpdate => 3,
            Phase::Render => 4,
        };
        self.schedules[idx].add_systems(systems);
        self
    }

    #[cfg(test)]
    pub(crate) fn set_renderer(&mut self, renderer: Box<dyn Renderer>) {
        self.renderer = Some(renderer);
    }

    pub(crate) fn handle_resize(&mut self, width: u32, height: u32) {
        if let Some(renderer) = &mut self.renderer {
            renderer.resize(width, height);
        }
    }

    pub(crate) fn handle_redraw(&mut self) {
        for schedule in &mut self.schedules {
            schedule.run(&mut self.world);
        }
        let Self { renderer, render_fn, .. } = self;
        if let Some(renderer) = renderer {
            if let Some(f) = render_fn {
                f(renderer.as_mut());
            }
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
        self.renderer = Some(renderer);
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
    use std::cell::RefCell;
    use std::rc::Rc;

    use engine_core::color::Color;
    use engine_ecs::prelude::{Phase, ResMut, Resource};
    use engine_render::rect::Rect;

    #[derive(Resource)]
    struct Counter(u32);

    fn increment(mut counter: ResMut<Counter>) {
        counter.0 += 1;
    }

    struct SpyRenderer {
        log: Rc<RefCell<Vec<String>>>,
    }

    impl SpyRenderer {
        fn new(log: Rc<RefCell<Vec<String>>>) -> Self {
            Self { log }
        }
    }

    impl engine_render::renderer::Renderer for SpyRenderer {
        fn clear(&mut self, _color: Color) {
            self.log.borrow_mut().push("clear".into());
        }

        fn draw_rect(&mut self, _rect: Rect) {
            self.log.borrow_mut().push("draw_rect".into());
        }

        fn present(&mut self) {
            self.log.borrow_mut().push("present".into());
        }

        fn resize(&mut self, _width: u32, _height: u32) {
            self.log.borrow_mut().push("resize".into());
        }
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
    fn when_on_render_called_then_callback_receives_renderer_on_redraw() {
        // Arrange
        let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let spy = SpyRenderer::new(Rc::clone(&log));

        let mut app = App::new();
        app.on_render(move |r| {
            r.clear(Color::BLACK);
        });
        app.set_renderer(Box::new(spy));

        // Act
        app.handle_redraw();

        // Assert
        assert!(log.borrow().contains(&"clear".to_string()));
    }

    #[test]
    fn when_handle_redraw_called_without_render_fn_then_present_still_called() {
        // Arrange
        let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let spy = SpyRenderer::new(Rc::clone(&log));

        let mut app = App::new();
        app.set_renderer(Box::new(spy));

        // Act
        app.handle_redraw();

        // Assert
        assert_eq!(log.borrow().as_slice(), &["present".to_string()]);
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
            .add_systems(Phase::Update, noop)
            .on_render(|_| {});
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
    fn when_render_phase_and_render_fn_both_set_then_phases_run_before_render_fn() {
        #[derive(Resource, Default)]
        struct Log(Vec<String>);

        fn ecs_render(mut log: ResMut<Log>) {
            log.0.push("ecs_render".into());
        }

        // Arrange
        let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let spy = SpyRenderer::new(Rc::clone(&log));

        let mut app = App::new();
        app.world_mut().insert_resource(Log::default());
        app.add_systems(Phase::Render, ecs_render);
        app.on_render(move |r| {
            r.draw_rect(Rect::default());
        });
        app.set_renderer(Box::new(spy));

        // Act
        app.handle_redraw();

        // Assert
        assert!(app.world().resource::<Log>().0.contains(&"ecs_render".into()));
        let renderer_log = log.borrow();
        assert_eq!(renderer_log.as_slice(), &["draw_rect", "present"]);
    }

    #[test]
    fn when_no_render_fn_but_systems_exist_then_schedules_run_and_present_called() {
        // Arrange
        let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let spy = SpyRenderer::new(Rc::clone(&log));

        let mut app = App::new();
        app.world_mut().insert_resource(Counter(0));
        app.add_systems(Phase::Update, increment);
        app.set_renderer(Box::new(spy));

        // Act
        app.handle_redraw();

        // Assert
        assert_eq!(app.world().resource::<Counter>().0, 1);
        assert_eq!(log.borrow().as_slice(), &["present".to_string()]);
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
    fn when_handle_resize_called_then_renderer_resize_is_called() {
        // Arrange
        let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let spy = SpyRenderer::new(Rc::clone(&log));

        let mut app = App::new();
        app.set_renderer(Box::new(spy));

        // Act
        app.handle_resize(1024, 768);

        // Assert
        assert!(log.borrow().contains(&"resize".to_string()));
    }
}
