pub use crate::renderer::{NullRenderer, Renderer};
pub use crate::window::WindowConfig;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_prelude_imported_then_window_config_and_renderer_types_resolve() {
        // Act
        let _cfg = WindowConfig::default();
        let _renderer: Box<dyn Renderer> = Box::new(NullRenderer);

        // Assert — compilation is the assertion
    }
}
