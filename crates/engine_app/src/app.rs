use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

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
}

impl App {
    pub fn new() -> Self {
        Self {
            plugin_count: 0,
            window_config: WindowConfig::default(),
            render_fn: None,
            window: None,
            renderer: None,
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

    pub(crate) fn set_renderer(&mut self, renderer: Box<dyn Renderer>) {
        self.renderer = Some(renderer);
    }

    pub(crate) fn handle_resize(&mut self, width: u32, height: u32) {
        if let Some(renderer) = &mut self.renderer {
            renderer.resize(width, height);
        }
    }

    pub(crate) fn handle_redraw(&mut self) {
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
    use engine_render::rect::Rect;

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
