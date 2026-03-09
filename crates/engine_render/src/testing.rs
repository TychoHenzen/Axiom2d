use std::sync::{Arc, Mutex};

use engine_core::color::Color;

use crate::rect::Rect;
use crate::renderer::Renderer;

pub struct SpyRenderer {
    log: Arc<Mutex<Vec<String>>>,
    color_capture: Option<Arc<Mutex<Option<Color>>>>,
}

impl SpyRenderer {
    pub fn new(log: Arc<Mutex<Vec<String>>>) -> Self {
        Self {
            log,
            color_capture: None,
        }
    }

    pub fn with_color_capture(
        log: Arc<Mutex<Vec<String>>>,
        color_capture: Arc<Mutex<Option<Color>>>,
    ) -> Self {
        Self {
            log,
            color_capture: Some(color_capture),
        }
    }
}

impl Renderer for SpyRenderer {
    fn clear(&mut self, color: Color) {
        self.log.lock().unwrap().push("clear".into());
        if let Some(capture) = &self.color_capture {
            *capture.lock().unwrap() = Some(color);
        }
    }

    fn draw_rect(&mut self, _rect: Rect) {
        self.log.lock().unwrap().push("draw_rect".into());
    }

    fn present(&mut self) {
        self.log.lock().unwrap().push("present".into());
    }

    fn resize(&mut self, _width: u32, _height: u32) {
        self.log.lock().unwrap().push("resize".into());
    }
}

#[cfg(test)]
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
    fn when_clear_called_with_color_capture_then_color_is_stored() {
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let color_capture = Arc::new(Mutex::new(None));
        let mut spy = SpyRenderer::with_color_capture(log.clone(), color_capture.clone());
        let expected = Color::new(1.0, 0.0, 0.5, 1.0);

        // Act
        spy.clear(expected);

        // Assert
        assert_eq!(*color_capture.lock().unwrap(), Some(expected));
    }

}
