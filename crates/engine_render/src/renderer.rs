use engine_core::color::Color;

pub trait Renderer {
    fn clear(&mut self, color: Color);
    fn present(&mut self);
}

pub struct NullRenderer;

impl Renderer for NullRenderer {
    fn clear(&mut self, _color: Color) {}
    fn present(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_null_renderer_clears_then_does_not_panic() {
        // Arrange
        let mut renderer = NullRenderer;

        // Act
        renderer.clear(Color::BLACK);
    }

    #[test]
    fn when_null_renderer_presents_then_does_not_panic() {
        // Arrange
        let mut renderer = NullRenderer;

        // Act
        renderer.present();
    }

    #[test]
    fn when_null_renderer_boxed_as_dyn_renderer_then_can_be_held() {
        // Arrange
        let renderer = NullRenderer;

        // Act
        let _boxed: Box<dyn Renderer> = Box::new(renderer);
    }
}
