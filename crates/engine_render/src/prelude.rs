pub use crate::rect::Rect;
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
        let _rect = Rect {
            x: engine_core::types::Pixels(0.0),
            y: engine_core::types::Pixels(0.0),
            width: engine_core::types::Pixels(100.0),
            height: engine_core::types::Pixels(50.0),
            color: engine_core::color::Color::WHITE,
        };

        // Assert — compilation is the assertion
    }
}
