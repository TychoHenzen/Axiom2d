use bevy_ecs::prelude::Resource;
use engine_core::color::Color;

use crate::atlas::TextureAtlas;
use crate::material::BlendMode;
use crate::rect::Rect;
use crate::shader::ShaderHandle;
use crate::shape::ColorVertex;
use engine_core::types::TextureId;

/// Opaque handle to a persistent GPU mesh buffer.
/// Created by [`Renderer::upload_persistent_colored_mesh`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GpuMeshHandle(pub u32);

#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("atlas upload failed: {0}")]
    AtlasUpload(String),
    #[error("shader compilation failed: {0}")]
    ShaderCompilation(String),
    #[error("surface error: {0}")]
    Surface(String),
}

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
    fn draw_colored_mesh(
        &mut self,
        vertices: &[ColorVertex],
        indices: &[u32],
        model: [[f32; 4]; 4],
    );
    /// Upload a colored mesh to a persistent GPU buffer. Returns a handle valid for the
    /// lifetime of the renderer. The mesh data is NOT copied per-frame.
    fn upload_persistent_colored_mesh(
        &mut self,
        vertices: &[ColorVertex],
        indices: &[u32],
    ) -> GpuMeshHandle;

    /// Record a draw call that reads from a previously uploaded persistent mesh.
    /// No vertex copy occurs; the GPU reads from the persistent buffer directly.
    fn draw_persistent_colored_mesh(
        &mut self,
        handle: GpuMeshHandle,
        model: [[f32; 4]; 4],
    );

    /// Release a persistent GPU mesh buffer. Safe to call with an already-freed or
    /// invalid handle (no-op). Does not affect in-flight draw calls.
    fn free_persistent_colored_mesh(&mut self, handle: GpuMeshHandle);
    #[allow(clippy::too_many_arguments)]
    fn draw_text(&mut self, text: &str, x: f32, y: f32, font_size: f32, color: Color);
    fn set_view_projection(&mut self, matrix: [[f32; 4]; 4]);
    fn set_blend_mode(&mut self, mode: BlendMode);
    fn set_shader(&mut self, shader: ShaderHandle);
    fn set_material_uniforms(&mut self, data: &[u8]);
    fn bind_material_texture(&mut self, texture: TextureId, binding: u32);
    fn compile_shader(&mut self, handle: ShaderHandle, source: &str) -> Result<(), RenderError>;
    fn upload_atlas(&mut self, atlas: &TextureAtlas) -> Result<(), RenderError>;
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
    fn draw_colored_mesh(
        &mut self,
        _vertices: &[ColorVertex],
        _indices: &[u32],
        _model: [[f32; 4]; 4],
    ) {
    }
    fn upload_persistent_colored_mesh(
        &mut self,
        _vertices: &[ColorVertex],
        _indices: &[u32],
    ) -> GpuMeshHandle {
        GpuMeshHandle(0)
    }
    fn draw_persistent_colored_mesh(&mut self, _handle: GpuMeshHandle, _model: [[f32; 4]; 4]) {}
    fn free_persistent_colored_mesh(&mut self, _handle: GpuMeshHandle) {}
    fn draw_text(&mut self, _text: &str, _x: f32, _y: f32, _font_size: f32, _color: Color) {}
    fn set_view_projection(&mut self, _matrix: [[f32; 4]; 4]) {}
    fn set_blend_mode(&mut self, _mode: BlendMode) {}
    fn set_shader(&mut self, _shader: ShaderHandle) {}
    fn set_material_uniforms(&mut self, _data: &[u8]) {}
    fn bind_material_texture(&mut self, _texture: TextureId, _binding: u32) {}
    fn compile_shader(&mut self, _handle: ShaderHandle, _source: &str) -> Result<(), RenderError> {
        Ok(())
    }
    fn upload_atlas(&mut self, _atlas: &TextureAtlas) -> Result<(), RenderError> {
        Ok(())
    }
    fn viewport_size(&self) -> (u32, u32) {
        (0, 0)
    }
    fn apply_post_process(&mut self) {}
    fn present(&mut self) {}
    fn resize(&mut self, _width: u32, _height: u32) {}
}
