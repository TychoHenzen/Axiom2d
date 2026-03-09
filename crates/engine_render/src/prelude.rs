pub use crate::rect::Rect;
pub use crate::renderer::{NullRenderer, Renderer};
pub use crate::window::WindowConfig;
pub use crate::create_renderer;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_prelude_imported_then_window_config_and_renderer_types_resolve() {
        // Act
        let _cfg = WindowConfig::default();
        let _renderer: Box<dyn Renderer> = Box::new(NullRenderer);
        let _rect = Rect::default();
    }
}
