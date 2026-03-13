use bevy_ecs::prelude::Resource;
use engine_core::color::Color;

use crate::atlas::TextureAtlas;
use crate::material::{BlendMode, ShaderHandle};
use crate::rect::Rect;
use engine_core::types::TextureId;

pub trait Renderer {
    fn clear(&mut self, color: Color);
    fn draw_rect(&mut self, rect: Rect);
    fn draw_sprite(&mut self, rect: Rect, uv_rect: [f32; 4]);
    fn draw_shape(&mut self, vertices: &[[f32; 2]], indices: &[u32], color: Color);
    fn set_view_projection(&mut self, matrix: [[f32; 4]; 4]);
    fn set_blend_mode(&mut self, mode: BlendMode);
    fn set_shader(&mut self, shader: ShaderHandle);
    fn set_material_uniforms(&mut self, data: &[u8]);
    fn bind_material_texture(&mut self, texture: TextureId, binding: u32);
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
    fn draw_shape(&mut self, _vertices: &[[f32; 2]], _indices: &[u32], _color: Color) {}
    fn set_view_projection(&mut self, _matrix: [[f32; 4]; 4]) {}
    fn set_blend_mode(&mut self, _mode: BlendMode) {}
    fn set_shader(&mut self, _shader: ShaderHandle) {}
    fn set_material_uniforms(&mut self, _data: &[u8]) {}
    fn bind_material_texture(&mut self, _texture: TextureId, _binding: u32) {}
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
    use std::collections::HashMap;
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
    fn when_null_renderer_draws_sprite_then_does_not_panic() {
        // Arrange
        let mut renderer = NullRenderer;

        // Act
        renderer.draw_sprite(sample_rect(), [0.0, 0.0, 1.0, 1.0]);
    }

    #[test]
    fn when_null_renderer_draws_shape_then_does_not_panic() {
        // Arrange
        let mut renderer = NullRenderer;
        let vertices = [[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]];
        let indices = [0u32, 1, 2];

        // Act
        renderer.draw_shape(&vertices, &indices, Color::WHITE);
    }

    #[test]
    fn when_null_renderer_set_view_projection_then_does_not_panic() {
        // Arrange
        let mut renderer = NullRenderer;

        // Act
        renderer.set_view_projection([[0.0f32; 4]; 4]);
    }

    #[test]
    fn when_null_renderer_resizes_then_does_not_panic() {
        // Arrange
        let mut renderer = NullRenderer;

        // Act
        renderer.resize(800, 600);
    }

    fn minimal_atlas() -> crate::atlas::TextureAtlas {
        crate::atlas::TextureAtlas {
            data: vec![255; 4],
            width: 1,
            height: 1,
            lookups: HashMap::default(),
        }
    }

    #[test]
    fn when_null_renderer_upload_atlas_then_does_not_panic() {
        // Arrange
        let mut renderer = NullRenderer;

        // Act
        renderer.upload_atlas(&minimal_atlas());
    }

    #[test]
    fn when_null_renderer_set_blend_mode_then_does_not_panic() {
        // Arrange
        let mut renderer = NullRenderer;

        // Act
        renderer.set_blend_mode(BlendMode::Alpha);
        renderer.set_blend_mode(BlendMode::Additive);
        renderer.set_blend_mode(BlendMode::Multiply);
    }

    #[test]
    fn when_null_renderer_set_shader_and_uniforms_and_texture_then_does_not_panic() {
        // Arrange
        let mut renderer = NullRenderer;

        // Act
        renderer.set_shader(crate::material::ShaderHandle(0));
        renderer.set_material_uniforms(&[1, 2, 3]);
        renderer.bind_material_texture(engine_core::types::TextureId(0), 2);
    }

    #[test]
    fn when_null_renderer_applies_post_process_then_does_not_panic() {
        // Arrange
        let mut renderer = NullRenderer;

        // Act
        renderer.apply_post_process();
    }

    #[test]
    fn when_null_renderer_viewport_size_then_returns_zero_zero() {
        // Arrange
        let renderer = NullRenderer;

        // Act
        let (w, h) = renderer.viewport_size();

        // Assert
        assert_eq!(w, 0);
        assert_eq!(h, 0);
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
