use std::sync::{Arc, Mutex};

use engine_core::color::Color;

use crate::rect::Rect;
use crate::renderer::Renderer;

pub type SpriteCallLog = Arc<Mutex<Vec<(Rect, [f32; 4])>>>;
pub type ShapeCallLog = Arc<Mutex<Vec<(Vec<[f32; 2]>, Vec<u32>, Color)>>>;
pub type MatrixCapture = Arc<Mutex<Option<[[f32; 4]; 4]>>>;

pub struct SpyRenderer {
    log: Arc<Mutex<Vec<String>>>,
    color_capture: Option<Arc<Mutex<Option<Color>>>>,
    sprite_calls: Option<SpriteCallLog>,
    shape_calls: Option<ShapeCallLog>,
    matrix_capture: Option<MatrixCapture>,
    viewport: (u32, u32),
}

impl SpyRenderer {
    pub fn new(log: Arc<Mutex<Vec<String>>>) -> Self {
        Self {
            log,
            color_capture: None,
            sprite_calls: None,
            shape_calls: None,
            matrix_capture: None,
            viewport: (0, 0),
        }
    }

    pub fn with_color_capture(mut self, color_capture: Arc<Mutex<Option<Color>>>) -> Self {
        self.color_capture = Some(color_capture);
        self
    }

    pub fn with_sprite_capture(mut self, sprite_calls: SpriteCallLog) -> Self {
        self.sprite_calls = Some(sprite_calls);
        self
    }

    pub fn with_shape_capture(mut self, shape_calls: ShapeCallLog) -> Self {
        self.shape_calls = Some(shape_calls);
        self
    }

    pub fn with_matrix_capture(mut self, matrix_capture: MatrixCapture) -> Self {
        self.matrix_capture = Some(matrix_capture);
        self
    }

    pub fn with_viewport(mut self, width: u32, height: u32) -> Self {
        self.viewport = (width, height);
        self
    }

    fn log_call(&self, name: &str) {
        self.log.lock().expect("spy log poisoned").push(name.into());
    }
}

impl Renderer for SpyRenderer {
    fn clear(&mut self, color: Color) {
        self.log_call("clear");
        if let Some(capture) = &self.color_capture {
            *capture.lock().expect("color capture poisoned") = Some(color);
        }
    }

    fn draw_rect(&mut self, _rect: Rect) {
        self.log_call("draw_rect");
    }

    fn draw_sprite(&mut self, rect: Rect, uv_rect: [f32; 4]) {
        self.log_call("draw_sprite");
        if let Some(capture) = &self.sprite_calls {
            capture
                .lock()
                .expect("sprite capture poisoned")
                .push((rect, uv_rect));
        }
    }

    fn draw_shape(&mut self, vertices: &[[f32; 2]], indices: &[u32], color: Color) {
        self.log_call("draw_shape");
        if let Some(capture) = &self.shape_calls {
            capture.lock().expect("shape capture poisoned").push((
                vertices.to_vec(),
                indices.to_vec(),
                color,
            ));
        }
    }

    fn set_view_projection(&mut self, matrix: [[f32; 4]; 4]) {
        self.log_call("set_view_projection");
        if let Some(capture) = &self.matrix_capture {
            *capture.lock().expect("matrix capture poisoned") = Some(matrix);
        }
    }

    fn viewport_size(&self) -> (u32, u32) {
        self.viewport
    }

    fn apply_post_process(&mut self) {
        self.log_call("apply_post_process");
    }

    fn present(&mut self) {
        self.log_call("present");
    }

    fn resize(&mut self, _width: u32, _height: u32) {
        self.log_call("resize");
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::{Arc, Mutex};

    use engine_core::color::Color;

    use super::*;
    use crate::renderer::Renderer;

    #[test]
    fn when_clear_called_then_log_records_clear_string() {
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let mut spy = SpyRenderer::new(log.clone());

        // Act
        spy.clear(Color::WHITE);

        // Assert
        assert_eq!(log.lock().unwrap().as_slice(), &["clear"]);
    }

    #[test]
    fn when_draw_rect_called_then_log_records_draw_rect_string() {
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let mut spy = SpyRenderer::new(log.clone());
        let rect = Rect::default();

        // Act
        spy.draw_rect(rect);

        // Assert
        assert_eq!(log.lock().unwrap().as_slice(), &["draw_rect"]);
    }

    #[test]
    fn when_present_called_then_log_records_present_string() {
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let mut spy = SpyRenderer::new(log.clone());

        // Act
        spy.present();

        // Assert
        assert_eq!(log.lock().unwrap().as_slice(), &["present"]);
    }

    #[test]
    fn when_resize_called_then_log_records_resize_string() {
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let mut spy = SpyRenderer::new(log.clone());

        // Act
        spy.resize(800, 600);

        // Assert
        assert_eq!(log.lock().unwrap().as_slice(), &["resize"]);
    }

    #[test]
    fn when_draw_sprite_called_then_log_records_draw_sprite_string() {
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let mut spy = SpyRenderer::new(log.clone());

        // Act
        spy.draw_sprite(Rect::default(), [0.0, 0.0, 1.0, 1.0]);

        // Assert
        assert_eq!(log.lock().unwrap().as_slice(), &["draw_sprite"]);
    }

    #[test]
    fn when_set_view_projection_called_then_log_records_set_view_projection() {
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let mut spy = SpyRenderer::new(log.clone());

        // Act
        spy.set_view_projection([[0.0f32; 4]; 4]);

        // Assert
        assert_eq!(log.lock().unwrap().as_slice(), &["set_view_projection"]);
    }

    #[test]
    fn when_draw_shape_called_then_log_records_draw_shape_string() {
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let mut spy = SpyRenderer::new(log.clone());

        // Act
        spy.draw_shape(
            &[[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]],
            &[0, 1, 2],
            Color::WHITE,
        );

        // Assert
        assert_eq!(log.lock().unwrap().as_slice(), &["draw_shape"]);
    }

    #[test]
    fn when_draw_shape_called_with_capture_then_color_matches() {
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let shape_calls: ShapeCallLog = Arc::new(Mutex::new(Vec::new()));
        let mut spy = SpyRenderer::new(log).with_shape_capture(shape_calls.clone());
        let color = Color::new(1.0, 0.0, 0.0, 1.0);

        // Act
        spy.draw_shape(&[[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]], &[0, 1, 2], color);

        // Assert
        let calls = shape_calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].2, color);
    }

    #[test]
    fn when_apply_post_process_called_then_log_records_apply_post_process() {
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let mut spy = SpyRenderer::new(log.clone());

        // Act
        spy.apply_post_process();

        // Assert
        assert_eq!(log.lock().unwrap().as_slice(), &["apply_post_process"]);
    }

    #[test]
    fn when_clear_called_with_color_capture_then_color_is_stored() {
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let color_capture = Arc::new(Mutex::new(None));
        let mut spy = SpyRenderer::new(log.clone()).with_color_capture(color_capture.clone());
        let expected = Color::new(1.0, 0.0, 0.5, 1.0);

        // Act
        spy.clear(expected);

        // Assert
        assert_eq!(*color_capture.lock().unwrap(), Some(expected));
    }
}
