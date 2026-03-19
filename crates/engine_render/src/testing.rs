use std::sync::{Arc, Mutex};

use bevy_ecs::world::World;
use engine_core::color::Color;

use engine_core::types::TextureId;

use crate::material::BlendMode;
use crate::rect::Rect;
use crate::renderer::{Renderer, RendererRes};
use crate::shader::ShaderHandle;

pub type RectCallLog = Arc<Mutex<Vec<Rect>>>;
pub type SpriteCallLog = Arc<Mutex<Vec<(Rect, [f32; 4])>>>;
pub type ShapeCallLog = Arc<Mutex<Vec<(Vec<[f32; 2]>, Vec<u32>, Color, [[f32; 4]; 4])>>>;
pub type MatrixCapture = Arc<Mutex<Option<[[f32; 4]; 4]>>>;
pub type BlendCapture = Arc<Mutex<Vec<BlendMode>>>;
pub type ShaderCapture = Arc<Mutex<Vec<ShaderHandle>>>;
pub type UniformCapture = Arc<Mutex<Vec<Vec<u8>>>>;
pub type TextureBindCapture = Arc<Mutex<Vec<(TextureId, u32)>>>;
pub type CompileShaderCapture = Arc<Mutex<Vec<(ShaderHandle, String)>>>;
pub type TextCallLog = Arc<Mutex<Vec<(String, f32, f32, f32, Color)>>>;

pub struct SpyRenderer {
    log: Arc<Mutex<Vec<String>>>,
    color_capture: Option<Arc<Mutex<Option<Color>>>>,
    rect_calls: Option<RectCallLog>,
    sprite_calls: Option<SpriteCallLog>,
    shape_calls: Option<ShapeCallLog>,
    matrix_capture: Option<MatrixCapture>,
    blend_calls: Option<BlendCapture>,
    shader_calls: Option<ShaderCapture>,
    uniform_calls: Option<UniformCapture>,
    texture_bind_calls: Option<TextureBindCapture>,
    compile_shader_calls: Option<CompileShaderCapture>,
    text_calls: Option<TextCallLog>,
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
            compile_shader_calls: None,
            text_calls: None,
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

    pub fn with_blend_capture(mut self, blend_calls: BlendCapture) -> Self {
        self.blend_calls = Some(blend_calls);
        self
    }

    pub fn with_shader_capture(mut self, shader_calls: ShaderCapture) -> Self {
        self.shader_calls = Some(shader_calls);
        self
    }

    pub fn with_uniform_capture(mut self, uniform_calls: UniformCapture) -> Self {
        self.uniform_calls = Some(uniform_calls);
        self
    }

    pub fn with_texture_bind_capture(mut self, texture_bind_calls: TextureBindCapture) -> Self {
        self.texture_bind_calls = Some(texture_bind_calls);
        self
    }

    pub fn with_compile_shader_capture(
        mut self,
        compile_shader_calls: CompileShaderCapture,
    ) -> Self {
        self.compile_shader_calls = Some(compile_shader_calls);
        self
    }

    pub fn with_text_capture(mut self, text_calls: TextCallLog) -> Self {
        self.text_calls = Some(text_calls);
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

    fn draw_shape(
        &mut self,
        vertices: &[[f32; 2]],
        indices: &[u32],
        color: Color,
        model: [[f32; 4]; 4],
    ) {
        self.log_call("draw_shape");
        if let Some(capture) = &self.shape_calls {
            capture.lock().expect("shape capture poisoned").push((
                vertices.to_vec(),
                indices.to_vec(),
                color,
                model,
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

    fn compile_shader(&mut self, handle: ShaderHandle, source: &str) {
        self.log_call("compile_shader");
        if let Some(capture) = &self.compile_shader_calls {
            capture
                .lock()
                .expect("compile_shader capture poisoned")
                .push((handle, source.to_owned()));
        }
    }

    fn draw_text(&mut self, text: &str, x: f32, y: f32, font_size: f32, color: Color) {
        self.log_call("draw_text");
        if let Some(capture) = &self.text_calls {
            capture.lock().expect("text capture poisoned").push((
                text.to_owned(),
                x,
                y,
                font_size,
                color,
            ));
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

fn insert_spy_capturing<T: 'static>(
    world: &mut World,
    attach: fn(SpyRenderer, Arc<Mutex<Vec<T>>>) -> SpyRenderer,
) -> Arc<Mutex<Vec<T>>> {
    let log = Arc::new(Mutex::new(Vec::new()));
    let capture: Arc<Mutex<Vec<T>>> = Arc::new(Mutex::new(Vec::new()));
    let spy = attach(SpyRenderer::new(log), capture.clone());
    world.insert_resource(RendererRes::new(Box::new(spy)));
    capture
}

pub fn insert_spy_with_blend_capture(world: &mut World) -> BlendCapture {
    insert_spy_capturing(world, SpyRenderer::with_blend_capture)
}

pub fn insert_spy_with_shader_capture(world: &mut World) -> ShaderCapture {
    insert_spy_capturing(world, SpyRenderer::with_shader_capture)
}

pub fn insert_spy_with_uniform_capture(world: &mut World) -> UniformCapture {
    insert_spy_capturing(world, SpyRenderer::with_uniform_capture)
}

pub fn insert_spy_with_texture_bind_capture(world: &mut World) -> TextureBindCapture {
    insert_spy_capturing(world, SpyRenderer::with_texture_bind_capture)
}

pub fn insert_spy_with_sprite_capture(world: &mut World) -> SpriteCallLog {
    insert_spy_capturing(world, SpyRenderer::with_sprite_capture)
}

pub fn insert_spy_with_shape_capture(world: &mut World) -> ShapeCallLog {
    insert_spy_capturing(world, SpyRenderer::with_shape_capture)
}

pub fn insert_spy_with_shape_and_viewport(
    world: &mut World,
    width: u32,
    height: u32,
) -> ShapeCallLog {
    let log = Arc::new(Mutex::new(Vec::new()));
    let calls: ShapeCallLog = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(log)
        .with_shape_capture(calls.clone())
        .with_viewport(width, height);
    world.insert_resource(RendererRes::new(Box::new(spy)));
    calls
}

pub fn insert_spy_with_compile_shader_capture(world: &mut World) -> CompileShaderCapture {
    let log = Arc::new(Mutex::new(Vec::new()));
    let capture: CompileShaderCapture = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(log).with_compile_shader_capture(capture.clone());
    world.insert_resource(RendererRes::new(Box::new(spy)));
    capture
}

pub fn insert_spy_with_text_capture(world: &mut World) -> TextCallLog {
    insert_spy_capturing(world, SpyRenderer::with_text_capture)
}

pub fn insert_spy_with_text_and_viewport(
    world: &mut World,
    width: u32,
    height: u32,
) -> TextCallLog {
    let log = Arc::new(Mutex::new(Vec::new()));
    let calls: TextCallLog = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(log)
        .with_text_capture(calls.clone())
        .with_viewport(width, height);
    world.insert_resource(RendererRes::new(Box::new(spy)));
    calls
}

pub fn insert_spy_with_blend_and_sprite_capture(
    world: &mut World,
) -> (BlendCapture, SpriteCallLog) {
    let log = Arc::new(Mutex::new(Vec::new()));
    let blend_calls: BlendCapture = Arc::new(Mutex::new(Vec::new()));
    let sprite_calls = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(log)
        .with_blend_capture(blend_calls.clone())
        .with_sprite_capture(sprite_calls.clone());
    world.insert_resource(RendererRes::new(Box::new(spy)));
    (blend_calls, sprite_calls)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::{Arc, Mutex};

    use engine_core::color::Color;

    use super::*;
    use crate::renderer::Renderer;

    fn call_every_renderer_method(spy: &mut SpyRenderer) {
        spy.clear(Color::WHITE);
        spy.draw_rect(Rect::default());
        spy.draw_sprite(Rect::default(), [0.0, 0.0, 1.0, 1.0]);
        spy.draw_shape(
            &[[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]],
            &[0, 1, 2],
            Color::WHITE,
            crate::renderer::IDENTITY_MODEL,
        );
        spy.set_view_projection([[0.0f32; 4]; 4]);
        spy.set_blend_mode(BlendMode::Additive);
        spy.set_shader(crate::shader::ShaderHandle(0));
        spy.set_material_uniforms(&[1]);
        spy.bind_material_texture(engine_core::types::TextureId(0), 0);
        spy.draw_text("Test", 0.0, 0.0, 12.0, Color::WHITE);
        spy.compile_shader(crate::shader::ShaderHandle(1), "source");
        spy.upload_atlas(&crate::atlas::TextureAtlas {
            data: vec![255; 4],
            width: 1,
            height: 1,
            lookups: std::collections::HashMap::default(),
        });
        spy.apply_post_process();
        spy.resize(800, 600);
        spy.present();
    }

    const ALL_METHOD_NAMES: &[&str] = &[
        "clear",
        "draw_rect",
        "draw_sprite",
        "draw_shape",
        "set_view_projection",
        "set_blend_mode",
        "set_shader",
        "set_material_uniforms",
        "bind_material_texture",
        "draw_text",
        "compile_shader",
        "upload_atlas",
        "apply_post_process",
        "resize",
        "present",
    ];

    #[test]
    fn when_each_method_called_then_log_records_matching_string() {
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let mut spy = SpyRenderer::new(log.clone());

        // Act
        call_every_renderer_method(&mut spy);

        // Assert
        let entries = log.lock().unwrap();
        assert_eq!(entries.as_slice(), ALL_METHOD_NAMES);
    }

    #[test]
    fn when_draw_shape_called_with_capture_then_color_matches() {
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let shape_calls: ShapeCallLog = Arc::new(Mutex::new(Vec::new()));
        let mut spy = SpyRenderer::new(log).with_shape_capture(shape_calls.clone());
        let color = Color::new(1.0, 0.0, 0.0, 1.0);

        // Act
        spy.draw_shape(
            &[[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]],
            &[0, 1, 2],
            color,
            crate::renderer::IDENTITY_MODEL,
        );

        // Assert
        let calls = shape_calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].2, color);
    }

    #[test]
    fn when_set_blend_mode_called_twice_with_capture_then_both_calls_recorded_in_order() {
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let blend_calls: BlendCapture = Arc::new(Mutex::new(Vec::new()));
        let mut spy = SpyRenderer::new(log).with_blend_capture(blend_calls.clone());

        // Act
        spy.set_blend_mode(BlendMode::Alpha);
        spy.set_blend_mode(BlendMode::Additive);

        // Assert
        let calls = blend_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[BlendMode::Alpha, BlendMode::Additive]);
    }

    #[test]
    fn when_set_shader_called_with_capture_then_handle_matches() {
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let shader_calls: ShaderCapture = Arc::new(Mutex::new(Vec::new()));
        let mut spy = SpyRenderer::new(log).with_shader_capture(shader_calls.clone());

        // Act
        spy.set_shader(crate::shader::ShaderHandle(7));

        // Assert
        let calls = shader_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[crate::shader::ShaderHandle(7)]);
    }

    #[test]
    fn when_set_material_uniforms_called_with_capture_then_bytes_match() {
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let uniform_calls: UniformCapture = Arc::new(Mutex::new(Vec::new()));
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
        let texture_bind_calls: TextureBindCapture = Arc::new(Mutex::new(Vec::new()));
        let mut spy = SpyRenderer::new(log).with_texture_bind_capture(texture_bind_calls.clone());

        // Act
        spy.bind_material_texture(engine_core::types::TextureId(3), 1);

        // Assert
        let calls = texture_bind_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[(engine_core::types::TextureId(3), 1)]);
    }

    #[test]
    fn when_compile_shader_called_with_capture_then_handle_and_source_recorded() {
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let capture: CompileShaderCapture = Arc::new(Mutex::new(Vec::new()));
        let mut spy = SpyRenderer::new(log).with_compile_shader_capture(capture.clone());

        // Act
        spy.compile_shader(crate::shader::ShaderHandle(7), "fn vs_shape() {}");

        // Assert
        let calls = capture.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, crate::shader::ShaderHandle(7));
        assert_eq!(calls[0].1, "fn vs_shape() {}");
    }

    #[test]
    fn when_draw_text_called_with_capture_then_arguments_match() {
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let text_calls: TextCallLog = Arc::new(Mutex::new(Vec::new()));
        let mut spy = SpyRenderer::new(log).with_text_capture(text_calls.clone());
        let color = Color::WHITE;

        // Act
        spy.draw_text("Hello", 10.0, 20.0, 12.0, color);

        // Assert
        let calls = text_calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, "Hello");
        assert_eq!(calls[0].1, 10.0);
        assert_eq!(calls[0].2, 20.0);
        assert_eq!(calls[0].3, 12.0);
        assert_eq!(calls[0].4, color);
    }

    #[test]
    fn when_draw_text_called_twice_then_both_captured_in_order() {
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let text_calls: TextCallLog = Arc::new(Mutex::new(Vec::new()));
        let mut spy = SpyRenderer::new(log).with_text_capture(text_calls.clone());

        // Act
        spy.draw_text("Name", 0.0, 0.0, 10.0, Color::WHITE);
        spy.draw_text("Description", 0.0, 30.0, 8.0, Color::BLACK);

        // Assert
        let calls = text_calls.lock().unwrap();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].0, "Name");
        assert_eq!(calls[1].0, "Description");
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
