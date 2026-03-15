use std::sync::{Arc, Mutex};

use bevy_ecs::world::World;
use engine_core::color::Color;

use engine_core::types::TextureId;

use crate::material::{BlendMode, ShaderHandle};
use crate::rect::Rect;
use crate::renderer::{Renderer, RendererRes};

pub type RectCallLog = Arc<Mutex<Vec<Rect>>>;
pub type SpriteCallLog = Arc<Mutex<Vec<(Rect, [f32; 4])>>>;
pub type ShapeCallLog = Arc<Mutex<Vec<(Vec<[f32; 2]>, Vec<u32>, Color)>>>;
pub type MatrixCapture = Arc<Mutex<Option<[[f32; 4]; 4]>>>;
pub type BlendCallLog = Arc<Mutex<Vec<BlendMode>>>;
pub type ShaderCallLog = Arc<Mutex<Vec<ShaderHandle>>>;
pub type UniformCallLog = Arc<Mutex<Vec<Vec<u8>>>>;
pub type TextureBindCallLog = Arc<Mutex<Vec<(TextureId, u32)>>>;

pub struct SpyRenderer {
    log: Arc<Mutex<Vec<String>>>,
    color_capture: Option<Arc<Mutex<Option<Color>>>>,
    rect_calls: Option<RectCallLog>,
    sprite_calls: Option<SpriteCallLog>,
    shape_calls: Option<ShapeCallLog>,
    matrix_capture: Option<MatrixCapture>,
    blend_calls: Option<BlendCallLog>,
    shader_calls: Option<ShaderCallLog>,
    uniform_calls: Option<UniformCallLog>,
    texture_bind_calls: Option<TextureBindCallLog>,
    viewport: (u32, u32),
}

impl SpyRenderer {
    pub fn new(log: Arc<Mutex<Vec<String>>>) -> Self {
        Self {
            log,
            color_capture: None,
            rect_calls: None,
            sprite_calls: None,
            shape_calls: None,
            matrix_capture: None,
            blend_calls: None,
            shader_calls: None,
            uniform_calls: None,
            texture_bind_calls: None,
            viewport: (0, 0),
        }
    }

    pub fn with_color_capture(mut self, color_capture: Arc<Mutex<Option<Color>>>) -> Self {
        self.color_capture = Some(color_capture);
        self
    }

    pub fn with_rect_capture(mut self, rect_calls: RectCallLog) -> Self {
        self.rect_calls = Some(rect_calls);
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

    pub fn with_blend_capture(mut self, blend_calls: BlendCallLog) -> Self {
        self.blend_calls = Some(blend_calls);
        self
    }

    pub fn with_shader_capture(mut self, shader_calls: ShaderCallLog) -> Self {
        self.shader_calls = Some(shader_calls);
        self
    }

    pub fn with_uniform_capture(mut self, uniform_calls: UniformCallLog) -> Self {
        self.uniform_calls = Some(uniform_calls);
        self
    }

    pub fn with_texture_bind_capture(mut self, texture_bind_calls: TextureBindCallLog) -> Self {
        self.texture_bind_calls = Some(texture_bind_calls);
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

    fn draw_rect(&mut self, rect: Rect) {
        self.log_call("draw_rect");
        if let Some(capture) = &self.rect_calls {
            capture.lock().expect("rect capture poisoned").push(rect);
        }
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

    fn set_blend_mode(&mut self, mode: BlendMode) {
        self.log_call("set_blend_mode");
        if let Some(capture) = &self.blend_calls {
            capture.lock().expect("blend capture poisoned").push(mode);
        }
    }

    fn set_shader(&mut self, shader: ShaderHandle) {
        self.log_call("set_shader");
        if let Some(capture) = &self.shader_calls {
            capture
                .lock()
                .expect("shader capture poisoned")
                .push(shader);
        }
    }

    fn set_material_uniforms(&mut self, data: &[u8]) {
        self.log_call("set_material_uniforms");
        if let Some(capture) = &self.uniform_calls {
            capture
                .lock()
                .expect("uniform capture poisoned")
                .push(data.to_vec());
        }
    }

    fn bind_material_texture(&mut self, texture: TextureId, binding: u32) {
        self.log_call("bind_material_texture");
        if let Some(capture) = &self.texture_bind_calls {
            capture
                .lock()
                .expect("texture bind capture poisoned")
                .push((texture, binding));
        }
    }

    fn upload_atlas(&mut self, _atlas: &crate::atlas::TextureAtlas) {
        self.log_call("upload_atlas");
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

pub fn insert_spy(world: &mut World) -> Arc<Mutex<Vec<String>>> {
    let log = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(log.clone());
    world.insert_resource(RendererRes::new(Box::new(spy)));
    log
}

pub fn insert_spy_with_viewport(
    world: &mut World,
    width: u32,
    height: u32,
) -> Arc<Mutex<Vec<String>>> {
    let log = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(log.clone()).with_viewport(width, height);
    world.insert_resource(RendererRes::new(Box::new(spy)));
    log
}

pub fn insert_spy_with_blend_capture(world: &mut World) -> BlendCallLog {
    let log = Arc::new(Mutex::new(Vec::new()));
    let blend_calls: BlendCallLog = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(log).with_blend_capture(blend_calls.clone());
    world.insert_resource(RendererRes::new(Box::new(spy)));
    blend_calls
}

pub fn insert_spy_with_shader_capture(world: &mut World) -> ShaderCallLog {
    let log = Arc::new(Mutex::new(Vec::new()));
    let shader_calls: ShaderCallLog = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(log).with_shader_capture(shader_calls.clone());
    world.insert_resource(RendererRes::new(Box::new(spy)));
    shader_calls
}

pub fn insert_spy_with_uniform_capture(world: &mut World) -> UniformCallLog {
    let log = Arc::new(Mutex::new(Vec::new()));
    let uniform_calls: UniformCallLog = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(log).with_uniform_capture(uniform_calls.clone());
    world.insert_resource(RendererRes::new(Box::new(spy)));
    uniform_calls
}

pub fn insert_spy_with_texture_bind_capture(world: &mut World) -> TextureBindCallLog {
    let log = Arc::new(Mutex::new(Vec::new()));
    let texture_bind_calls: TextureBindCallLog = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(log).with_texture_bind_capture(texture_bind_calls.clone());
    world.insert_resource(RendererRes::new(Box::new(spy)));
    texture_bind_calls
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
    fn when_set_blend_mode_called_then_log_records_set_blend_mode() {
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let mut spy = SpyRenderer::new(log.clone());

        // Act
        spy.set_blend_mode(BlendMode::Additive);

        // Assert
        assert_eq!(log.lock().unwrap().as_slice(), &["set_blend_mode"]);
    }

    #[test]
    fn when_set_blend_mode_called_twice_with_capture_then_both_calls_recorded_in_order() {
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let blend_calls: BlendCallLog = Arc::new(Mutex::new(Vec::new()));
        let mut spy = SpyRenderer::new(log).with_blend_capture(blend_calls.clone());

        // Act
        spy.set_blend_mode(BlendMode::Alpha);
        spy.set_blend_mode(BlendMode::Additive);

        // Assert
        let calls = blend_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[BlendMode::Alpha, BlendMode::Additive]);
    }

    #[test]
    fn when_upload_atlas_called_then_log_records_upload_atlas() {
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let mut spy = SpyRenderer::new(log.clone());
        let atlas = crate::atlas::TextureAtlas {
            data: vec![255; 4],
            width: 1,
            height: 1,
            lookups: std::collections::HashMap::default(),
        };

        // Act
        spy.upload_atlas(&atlas);

        // Assert
        assert_eq!(log.lock().unwrap().as_slice(), &["upload_atlas"]);
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
    fn when_set_shader_called_then_log_records_set_shader() {
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let mut spy = SpyRenderer::new(log.clone());

        // Act
        spy.set_shader(crate::material::ShaderHandle(42));

        // Assert
        assert_eq!(log.lock().unwrap().as_slice(), &["set_shader"]);
    }

    #[test]
    fn when_set_shader_called_with_capture_then_handle_matches() {
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let shader_calls: ShaderCallLog = Arc::new(Mutex::new(Vec::new()));
        let mut spy = SpyRenderer::new(log).with_shader_capture(shader_calls.clone());

        // Act
        spy.set_shader(crate::material::ShaderHandle(7));

        // Assert
        let calls = shader_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[crate::material::ShaderHandle(7)]);
    }

    #[test]
    fn when_set_material_uniforms_called_with_capture_then_bytes_match() {
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let uniform_calls: UniformCallLog = Arc::new(Mutex::new(Vec::new()));
        let mut spy = SpyRenderer::new(log).with_uniform_capture(uniform_calls.clone());

        // Act
        spy.set_material_uniforms(&[10, 20]);

        // Assert
        let calls = uniform_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[vec![10u8, 20]]);
    }

    #[test]
    fn when_bind_material_texture_called_with_capture_then_entry_matches() {
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let texture_bind_calls: TextureBindCallLog = Arc::new(Mutex::new(Vec::new()));
        let mut spy = SpyRenderer::new(log).with_texture_bind_capture(texture_bind_calls.clone());

        // Act
        spy.bind_material_texture(engine_core::types::TextureId(3), 1);

        // Assert
        let calls = texture_bind_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[(engine_core::types::TextureId(3), 1)]);
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
