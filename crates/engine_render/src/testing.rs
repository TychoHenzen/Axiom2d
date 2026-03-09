use std::sync::{Arc, Mutex};

use engine_core::color::Color;

use crate::rect::Rect;
use crate::renderer::Renderer;

pub struct SpyRenderer {
    log: Arc<Mutex<Vec<String>>>,
}

impl SpyRenderer {
    pub fn new(log: Arc<Mutex<Vec<String>>>) -> Self {
        Self { log }
    }
}

impl Renderer for SpyRenderer {
    fn clear(&mut self, _color: Color) {
        self.log.lock().unwrap().push("clear".into());
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
