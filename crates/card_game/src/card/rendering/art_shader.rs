use bevy_ecs::prelude::{Query, Res, Resource};
use engine_input::prelude::MouseState;
use engine_render::prelude::{ShaderHandle, ShaderRegistry};
use engine_render::shape::MeshOverlays;

use crate::card::identity::signature::Rarity;

pub const UV_GRADIENT_WGSL: &str = include_str!("../../shaders/uv_gradient.wgsl");
pub const GLOSSY_WGSL: &str = include_str!("../../shaders/glossy.wgsl");
pub const EMBOSSED_WGSL: &str = include_str!("../../shaders/embossed.wgsl");
pub const FOIL_WGSL: &str = include_str!("../../shaders/foil.wgsl");
pub const GLOW_WGSL: &str = include_str!("../../shaders/glow.wgsl");

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
    pub _pad0: f32,
    pub _pad1: f32,
    pub _pad2: f32,
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

pub fn register_variant_shaders(registry: &mut ShaderRegistry) -> VariantShaders {
    VariantShaders {
        embossed: registry.register(EMBOSSED_WGSL),
        glow: registry.register(GLOW_WGSL),
        glossy: registry.register(GLOSSY_WGSL),
        foil: registry.register(FOIL_WGSL),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn when_registering_card_art_shader_then_handle_is_retrievable() {
        // Arrange
        let mut registry = ShaderRegistry::default();

        // Act
        let art_shader = register_card_art_shader(&mut registry);

        // Assert
        assert_eq!(registry.lookup(art_shader.0), Some(UV_GRADIENT_WGSL));
    }

    #[test]
    fn when_uv_gradient_shader_parsed_with_naga_then_no_error() {
        // Act
        let result = naga::front::wgsl::parse_str(UV_GRADIENT_WGSL);

        // Assert
        assert!(result.is_ok(), "WGSL parse error: {result:?}");
    }

    #[test]
    fn when_uv_gradient_shader_source_inspected_then_camera_and_model_uniforms_declared() {
        // Assert
        assert!(
            UV_GRADIENT_WGSL.contains("@group(0) @binding(0)"),
            "shader must declare camera uniform at group(0) binding(0)"
        );
        assert!(
            UV_GRADIENT_WGSL.contains("@group(1) @binding(0)"),
            "shader must declare model uniform at group(1) binding(0)"
        );
    }

    #[test]
    fn when_uv_gradient_shader_source_inspected_then_vertex_inputs_at_location0_and_location1() {
        // Assert
        assert!(
            UV_GRADIENT_WGSL.contains("@location(0)"),
            "shader must accept position at @location(0)"
        );
        assert!(
            UV_GRADIENT_WGSL.contains("@location(1)"),
            "shader must accept color at @location(1)"
        );
    }

    #[test]
    fn when_rarity_is_common_then_shader_variant_is_none() {
        // Act
        let variant = ShaderVariant::from_rarity(Rarity::Common);

        // Assert
        assert_eq!(variant, ShaderVariant::None);
    }

    #[test]
    fn when_rarity_is_uncommon_then_shader_variant_is_embossed() {
        // Act
        let variant = ShaderVariant::from_rarity(Rarity::Uncommon);

        // Assert
        assert_eq!(variant, ShaderVariant::Embossed);
    }

    #[test]
    fn when_rarity_is_rare_then_shader_variant_is_glow() {
        // Act
        let variant = ShaderVariant::from_rarity(Rarity::Rare);

        // Assert
        assert_eq!(variant, ShaderVariant::Glow);
    }

    #[test]
    fn when_rarity_is_epic_then_shader_variant_is_glossy() {
        // Act
        let variant = ShaderVariant::from_rarity(Rarity::Epic);

        // Assert
        assert_eq!(variant, ShaderVariant::Glossy);
    }

    #[test]
    fn when_rarity_is_legendary_then_shader_variant_is_foil() {
        // Act
        let variant = ShaderVariant::from_rarity(Rarity::Legendary);

        // Assert
        assert_eq!(variant, ShaderVariant::Foil);
    }

    #[test]
    fn when_registering_card_art_shader_twice_then_handles_differ() {
        // Arrange
        let mut registry = ShaderRegistry::default();

        // Act
        let first = register_card_art_shader(&mut registry);
        let second = register_card_art_shader(&mut registry);

        // Assert
        assert_ne!(first.0, second.0);
    }

    fn assert_variant_shader_valid(source: &str, name: &str) {
        // Naga parse
        let result = naga::front::wgsl::parse_str(source);
        assert!(result.is_ok(), "{name} WGSL parse error: {result:?}");

        // Binding layout: camera, model, art params
        assert!(
            source.contains("@group(0) @binding(0)"),
            "{name} must declare camera uniform at group(0) binding(0)"
        );
        assert!(
            source.contains("@group(1) @binding(0)"),
            "{name} must declare model uniform at group(1) binding(0)"
        );
        assert!(
            source.contains("@group(2) @binding(0)"),
            "{name} must declare art params uniform at group(2) binding(0)"
        );
    }

    #[test]
    fn when_glossy_shader_parsed_and_inspected_then_valid_with_correct_bindings() {
        assert_variant_shader_valid(GLOSSY_WGSL, "glossy");
    }

    #[test]
    fn when_embossed_shader_parsed_and_inspected_then_valid_with_correct_bindings() {
        assert_variant_shader_valid(EMBOSSED_WGSL, "embossed");
    }

    #[test]
    fn when_foil_shader_parsed_and_inspected_then_valid_with_correct_bindings() {
        assert_variant_shader_valid(FOIL_WGSL, "foil");
    }

    #[test]
    fn when_foil_shader_source_inspected_then_uses_in_uv_for_spatial_phase() {
        // Assert — foil uses per-shape UV for spatial phase variation
        assert!(
            FOIL_WGSL.contains("in.uv"),
            "foil shader must reference in.uv for geometry-aware spatial phase"
        );
    }

    #[test]
    fn when_glow_shader_parsed_and_inspected_then_valid_with_correct_bindings() {
        assert_variant_shader_valid(GLOW_WGSL, "glow");
    }

    #[test]
    fn when_glow_shader_inspected_then_uses_chromatic_shift() {
        // Assert — glow shader uses thin-film chromatic color shift
        assert!(
            GLOW_WGSL.contains("phase"),
            "glow shader should use phase-based color shifting"
        );
    }

    #[test]
    fn when_all_card_shaders_parsed_then_accept_uv_at_location2() {
        // Arrange
        let shaders: &[(&str, &str)] = &[
            (UV_GRADIENT_WGSL, "uv_gradient"),
            (GLOSSY_WGSL, "glossy"),
            (EMBOSSED_WGSL, "embossed"),
            (FOIL_WGSL, "foil"),
            (GLOW_WGSL, "glow"),
        ];

        for (source, name) in shaders {
            // Act
            let result = naga::front::wgsl::parse_str(source);

            // Assert
            assert!(result.is_ok(), "{name} WGSL parse error: {result:?}");
            assert!(
                source.contains("@location(2)"),
                "{name} must accept uv at @location(2)"
            );
        }
    }

    #[test]
    fn when_art_region_params_converted_to_bytes_then_size_is_thirty_two_bytes() {
        // Arrange
        let params = ArtRegionParams {
            half_w: 27.0,
            half_h: 22.5,
            pointer_x: 0.0,
            pointer_y: 0.0,
            offset_y: 5.0,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
        };

        // Act
        let bytes = bytemuck::bytes_of(&params);

        // Assert
        assert_eq!(bytes.len(), 32);
    }

    #[test]
    fn when_registering_variant_shaders_then_all_handles_are_retrievable() {
        // Arrange
        let mut registry = ShaderRegistry::default();

        // Act
        let variants = register_variant_shaders(&mut registry);

        // Assert
        assert_eq!(registry.lookup(variants.embossed), Some(EMBOSSED_WGSL));
        assert_eq!(registry.lookup(variants.glow), Some(GLOW_WGSL));
        assert_eq!(registry.lookup(variants.glossy), Some(GLOSSY_WGSL));
        assert_eq!(registry.lookup(variants.foil), Some(FOIL_WGSL));
    }
}
