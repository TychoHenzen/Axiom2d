use engine_core::color::Color;

use crate::rect::Rect;

pub trait Renderer {
    fn clear(&mut self, color: Color);
    fn draw_rect(&mut self, rect: Rect);
    fn present(&mut self);
    fn resize(&mut self, width: u32, height: u32);
}

pub struct NullRenderer;

impl Renderer for NullRenderer {
    fn clear(&mut self, _color: Color) {}
    fn draw_rect(&mut self, _rect: Rect) {}
    fn present(&mut self) {}
    fn resize(&mut self, _width: u32, _height: u32) {}
}

#[cfg(test)]
mod tests {
    use engine_core::types::Pixels;

    use super::*;
    use crate::rect::Rect;

    fn sample_rect() -> Rect {
        Rect {
            x: Pixels(10.0),
            y: Pixels(20.0),
            width: Pixels(100.0),
            height: Pixels(50.0),
            color: Color::WHITE,
        }
    }

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
    fn when_null_renderer_draws_rect_then_does_not_panic() {
        // Arrange
        let mut renderer = NullRenderer;

        // Act
        renderer.draw_rect(sample_rect());
    }

    #[test]
    fn when_null_renderer_resizes_then_does_not_panic() {
        // Arrange
        let mut renderer = NullRenderer;

        // Act
        renderer.resize(800, 600);
    }

    #[test]
    fn when_null_renderer_boxed_as_dyn_renderer_then_can_be_held() {
        // Act
        let _boxed: Box<dyn Renderer> = Box::new(NullRenderer);
    }
}
