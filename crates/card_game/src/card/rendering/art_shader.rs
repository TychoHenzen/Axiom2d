// EVOLVE-BLOCK-START
use bevy_ecs::prelude::{Query, Res, Resource};
use engine_input::prelude::MouseState;
use engine_render::prelude::{ShaderHandle, ShaderRegistry};
use engine_render::shape::MeshOverlays;

use crate::card::identity::signature::Rarity;
use crate::card::identity::signature_profile::Tier;

pub const UV_GRADIENT_WGSL: &str = include_str!("../../shaders/uv_gradient.wgsl");
pub const DORMANT_WGSL: &str = include_str!("../../shaders/dormant.wgsl");
pub const INTENSE_WGSL: &str = include_str!("../../shaders/intense.wgsl");
pub const GLOSSY_WGSL: &str = include_str!("../../shaders/glossy.wgsl");
pub const EMBOSSED_WGSL: &str = include_str!("../../shaders/embossed.wgsl");
pub const FOIL_WGSL: &str = include_str!("../../shaders/foil.wgsl");
pub const GLOW_WGSL: &str = include_str!("../../shaders/glow.wgsl");
pub const GEM_WGSL: &str = include_str!("../../shaders/gem.wgsl");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderVariant {
    None,
    Embossed,
    Glow,
    Glossy,
    Foil,
}

impl ShaderVariant {
    pub fn from_rarity(rarity: Rarity) -> Self {
        match rarity {
            Rarity::Common => Self::None,
            Rarity::Uncommon => Self::Embossed,
            Rarity::Rare => Self::Glow,
            Rarity::Epic => Self::Glossy,
            Rarity::Legendary => Self::Foil,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionEffect {
    None,
    Worn,
    Shiny,
}

impl ConditionEffect {
    pub fn from_tier(tier: Tier) -> Self {
        match tier {
            Tier::Dormant => Self::Worn,
            Tier::Active => Self::None,
            Tier::Intense => Self::Shiny,
        }
    }
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct CardArtShader(pub ShaderHandle);

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ArtRegionParams {
    pub half_w: f32,
    pub half_h: f32,
    pub pointer_x: f32,
    pub pointer_y: f32,
    pub offset_y: f32,
    /// Shader-specific data slot. Usage varies by overlay:
    /// - Tier shaders: per-card seed (deterministic hash)
    /// - Gem shader: specular intensity (0.0–1.0)
    pub extra0: f32,
    pub extra1: f32,
    pub extra2: f32,
}

pub fn shader_pointer_system(mouse: Res<MouseState>, mut overlays_query: Query<&mut MeshOverlays>) {
    let world_pos = mouse.world_pos();
    let px_bytes = world_pos.x.to_le_bytes();
    let py_bytes = world_pos.y.to_le_bytes();
    for mut overlays in &mut overlays_query {
        for entry in &mut overlays.0 {
            let uniforms = &mut entry.material.uniforms;
            if uniforms.len() >= 16 {
                uniforms[8..12].copy_from_slice(&px_bytes);
                uniforms[12..16].copy_from_slice(&py_bytes);
            }
        }
    }
}

pub fn register_card_art_shader(registry: &mut ShaderRegistry) -> CardArtShader {
    CardArtShader(registry.register(UV_GRADIENT_WGSL))
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct VariantShaders {
    pub embossed: ShaderHandle,
    pub glow: ShaderHandle,
    pub glossy: ShaderHandle,
    pub foil: ShaderHandle,
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct TierShaders {
    pub dormant: ShaderHandle,
    pub intense: ShaderHandle,
}

pub fn register_tier_shaders(registry: &mut ShaderRegistry) -> TierShaders {
    TierShaders {
        dormant: registry.register(DORMANT_WGSL),
        intense: registry.register(INTENSE_WGSL),
    }
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct GemShader(pub ShaderHandle);

pub fn register_gem_shader(registry: &mut ShaderRegistry) -> GemShader {
    GemShader(registry.register(GEM_WGSL))
}

pub fn register_variant_shaders(registry: &mut ShaderRegistry) -> VariantShaders {
    VariantShaders {
        embossed: registry.register(EMBOSSED_WGSL),
        glow: registry.register(GLOW_WGSL),
        glossy: registry.register(GLOSSY_WGSL),
        foil: registry.register(FOIL_WGSL),
    }
}
// EVOLVE-BLOCK-END
