use engine_render::renderer::Renderer;

pub trait Plugin {
    fn build(&self, app: &mut App);
}

pub struct App {
    plugin_count: usize,
    renderer: Option<Box<dyn Renderer>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            plugin_count: 0,
            renderer: None,
        }
    }

    pub fn set_renderer(&mut self, renderer: Box<dyn Renderer>) -> &mut Self {
        self.renderer = Some(renderer);
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
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

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
    fn when_set_renderer_called_with_null_renderer_then_does_not_panic() {
        // Arrange
        let mut app = App::new();

        // Act
        app.set_renderer(Box::new(engine_render::renderer::NullRenderer));
    }

    #[test]
    fn when_app_default_called_then_plugin_count_is_zero() {
        // Act
        let app = App::default();

        // Assert
        assert_eq!(app.plugin_count(), 0);
    }
}
