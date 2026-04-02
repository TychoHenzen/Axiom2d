use bevy_ecs::prelude::Component;
use engine_core::types::TextureId;
use serde::{Deserialize, Serialize};

use crate::shader::ShaderHandle;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BlendMode {
    Alpha,
    Additive,
    Multiply,
}

impl BlendMode {
    pub const ALL: [Self; 3] = [Self::Alpha, Self::Additive, Self::Multiply];

    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextureBinding {
    pub texture: TextureId,
    pub binding: u32,
}

#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Material2d {
    pub blend_mode: BlendMode,
    pub shader: ShaderHandle,
    pub textures: Vec<TextureBinding>,
    pub uniforms: Vec<u8>,
}

impl Default for Material2d {
    fn default() -> Self {
        Self {
            blend_mode: BlendMode::Alpha,
            shader: ShaderHandle(0),
            textures: Vec::new(),
            uniforms: Vec::new(),
        }
    }
}

#[must_use]
pub fn effective_shader_handle(material: Option<&Material2d>) -> ShaderHandle {
    material.map_or(ShaderHandle(0), |m| m.shader)
}

#[must_use]
pub fn effective_blend_mode(material: Option<&Material2d>) -> BlendMode {
    material.map_or(BlendMode::Alpha, |m| m.blend_mode)
}

/// Applies per-entity material state to the renderer, deduplicating shader and blend-mode switches.
pub fn apply_material(
    renderer: &mut dyn crate::renderer::Renderer,
    material: Option<&Material2d>,
    last_shader: &mut Option<ShaderHandle>,
    last_blend_mode: &mut Option<BlendMode>,
) {
    let shader = effective_shader_handle(material);
    if *last_shader != Some(shader) {
        renderer.set_shader(shader);
        *last_shader = Some(shader);
    }

    let blend_mode = effective_blend_mode(material);
    if *last_blend_mode != Some(blend_mode) {
        renderer.set_blend_mode(blend_mode);
        *last_blend_mode = Some(blend_mode);
    }

    if let Some(mat) = material {
        if !mat.uniforms.is_empty() {
            renderer.set_material_uniforms(&mat.uniforms);
        }
        for binding in &mat.textures {
            renderer.bind_material_texture(binding.texture, binding.binding);
        }
    }
}
