use bevy_ecs::prelude::Resource;
use engine_core::color::Color;

use crate::atlas::TextureAtlas;
use crate::material::BlendMode;
use crate::rect::Rect;
use crate::shader::ShaderHandle;
use engine_core::types::TextureId;

pub const IDENTITY_MODEL: [[f32; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

pub trait Renderer {
    fn clear(&mut self, color: Color);
    fn draw_rect(&mut self, rect: Rect);
    fn draw_sprite(&mut self, rect: Rect, uv_rect: [f32; 4]);
    fn draw_shape(
        &mut self,
        vertices: &[[f32; 2]],
        indices: &[u32],
        color: Color,
        model: [[f32; 4]; 4],
    );
    fn set_view_projection(&mut self, matrix: [[f32; 4]; 4]);
    fn set_blend_mode(&mut self, mode: BlendMode);
    fn set_shader(&mut self, shader: ShaderHandle);
    fn set_material_uniforms(&mut self, data: &[u8]);
    fn bind_material_texture(&mut self, texture: TextureId, binding: u32);
    fn compile_shader(&mut self, handle: ShaderHandle, source: &str);
    fn upload_atlas(&mut self, atlas: &TextureAtlas);
    fn viewport_size(&self) -> (u32, u32);
    fn apply_post_process(&mut self);
    fn present(&mut self);
    fn resize(&mut self, width: u32, height: u32);
}

#[derive(Resource)]
pub struct RendererRes(Box<dyn Renderer + Send + Sync>);

impl RendererRes {
    #[must_use]
    pub fn new(renderer: Box<dyn Renderer + Send + Sync>) -> Self {
        Self(renderer)
    }
}

impl std::ops::Deref for RendererRes {
    type Target = dyn Renderer;
    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl std::ops::DerefMut for RendererRes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.0
    }
}

pub struct NullRenderer;

impl Renderer for NullRenderer {
    fn clear(&mut self, _color: Color) {}
    fn draw_rect(&mut self, _rect: Rect) {}
    fn draw_sprite(&mut self, _rect: Rect, _uv_rect: [f32; 4]) {}
    fn draw_shape(
        &mut self,
        _vertices: &[[f32; 2]],
        _indices: &[u32],
        _color: Color,
        _model: [[f32; 4]; 4],
    ) {
    }
    fn set_view_projection(&mut self, _matrix: [[f32; 4]; 4]) {}
    fn set_blend_mode(&mut self, _mode: BlendMode) {}
    fn set_shader(&mut self, _shader: ShaderHandle) {}
    fn set_material_uniforms(&mut self, _data: &[u8]) {}
    fn bind_material_texture(&mut self, _texture: TextureId, _binding: u32) {}
    fn compile_shader(&mut self, _handle: ShaderHandle, _source: &str) {}
    fn upload_atlas(&mut self, _atlas: &TextureAtlas) {}
    fn viewport_size(&self) -> (u32, u32) {
        (0, 0)
    }
    fn apply_post_process(&mut self) {}
    fn present(&mut self) {}
    fn resize(&mut self, _width: u32, _height: u32) {}
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {

    use std::sync::{Arc, Mutex};

    use engine_core::types::Pixels;

    use super::*;
    use crate::rect::Rect;
    use crate::testing::SpyRenderer;

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
    fn when_null_renderer_all_methods_called_then_none_panic() {
        // Arrange
        let mut renderer = NullRenderer;

        // Act
        renderer.clear(Color::BLACK);
        renderer.draw_rect(sample_rect());
        renderer.draw_sprite(sample_rect(), [0.0, 0.0, 1.0, 1.0]);
        renderer.draw_shape(
            &[[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]],
            &[0, 1, 2],
            Color::WHITE,
            IDENTITY_MODEL,
        );
        renderer.set_view_projection([[0.0f32; 4]; 4]);
        renderer.set_blend_mode(BlendMode::Alpha);
        renderer.set_shader(crate::shader::ShaderHandle(0));
        renderer.set_material_uniforms(&[1, 2, 3]);
        renderer.bind_material_texture(engine_core::types::TextureId(0), 2);
        renderer.upload_atlas(&crate::test_helpers::minimal_atlas());
        renderer.compile_shader(crate::shader::ShaderHandle(1), "@vertex fn vs_shape() {}");
        renderer.apply_post_process();
        renderer.resize(800, 600);
        renderer.present();
        assert_eq!(renderer.viewport_size(), (0, 0));
    }

    #[test]
    fn when_renderer_res_in_world_then_system_can_call_clear_via_resmut() {
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log.clone());
        let mut world = bevy_ecs::world::World::new();
        world.insert_resource(RendererRes::new(Box::new(spy)));
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(|mut renderer: bevy_ecs::prelude::ResMut<RendererRes>| {
            renderer.clear(Color::BLACK);
        });

        // Act
        schedule.run(&mut world);

        // Assert
        assert_eq!(log.lock().unwrap().as_slice(), &["clear"]);
    }
}
