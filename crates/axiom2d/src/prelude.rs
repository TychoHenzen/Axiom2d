pub use engine_app::prelude::*;
pub use engine_core::prelude::*;
pub use engine_render::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_facade_prelude_imported_then_app_window_config_and_null_renderer_resolve() {
        // Act
        let _app = App::new();
        let _cfg = WindowConfig::default();
        let _renderer: Box<dyn Renderer> = Box::new(NullRenderer);

        // Assert — compilation is the assertion
    }
}
