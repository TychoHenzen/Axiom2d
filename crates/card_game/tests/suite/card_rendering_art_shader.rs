#![allow(clippy::unwrap_used)]

use card_game::card::identity::signature::Rarity;
use card_game::card::identity::signature_profile::Tier;
use card_game::card::rendering::art_shader::*;
use engine_render::prelude::ShaderRegistry;

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
fn when_foil_shader_source_inspected_then_uses_in_uv_for_surface_detail() {
    // Assert — foil uses per-shape UV for micro-detail surface variation
    assert!(
        FOIL_WGSL.contains("in.uv"),
        "foil shader must reference in.uv for per-fragment surface detail"
    );
}

#[test]
fn when_glow_shader_parsed_and_inspected_then_valid_with_correct_bindings() {
    assert_variant_shader_valid(GLOW_WGSL, "glow");
}

#[test]
fn when_glow_shader_inspected_then_uses_per_cell_phase() {
    // Assert — glow shader uses hashed cell phase for sparkle orientation
    assert!(
        GLOW_WGSL.contains("phase"),
        "glow shader should use phase for per-cell sparkle orientation"
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
        extra0: 0.0,
        extra1: 0.0,
        extra2: 0.0,
    };

    // Act
    let bytes = bytemuck::bytes_of(&params);

    // Assert
    assert_eq!(bytes.len(), 32);
}

/// @doc: Dormant tier cards have low signature intensity — the Worn effect
/// desaturates and fades the card art to convey age and weakness. Without
/// this mapping, dormant cards would look identical to active ones, removing
/// a key visual signal of card power level.
#[test]
fn when_tier_is_dormant_then_condition_effect_is_worn() {
    // Arrange
    let tier = Tier::Dormant;

    // Act
    let effect = ConditionEffect::from_tier(tier);

    // Assert
    assert_eq!(effect, ConditionEffect::Worn);
}

/// @doc: Active tier is the baseline condition — no visual distortion applied.
/// Most cards fall here (intensity 0.3–0.7), so "normal" means no shader overlay,
/// keeping the rendering pipeline lean for the common case.
#[test]
fn when_tier_is_active_then_condition_effect_is_none() {
    // Arrange
    let tier = Tier::Active;

    // Act
    let effect = ConditionEffect::from_tier(tier);

    // Assert
    assert_eq!(effect, ConditionEffect::None);
}

/// @doc: Intense tier cards have extreme signature values — the Shiny effect
/// adds shimmer and brightness to signal peak condition. Players should
/// immediately recognize these as exceptional cards from across the table.
#[test]
fn when_tier_is_intense_then_condition_effect_is_shiny() {
    // Arrange
    let tier = Tier::Intense;

    // Act
    let effect = ConditionEffect::from_tier(tier);

    // Assert
    assert_eq!(effect, ConditionEffect::Shiny);
}

// --- gem shader tests ---

/// @doc: The gem facet shader must parse without errors through naga (the WGSL validator
/// used by wgpu) and declare all three bind groups. A parse failure crashes the GPU pipeline
/// at startup; missing bind groups cause silent rendering failures for all gem overlays.
#[test]
fn when_gem_shader_parsed_and_inspected_then_valid_with_correct_bindings() {
    assert_variant_shader_valid(GEM_WGSL, "gem");
}

#[test]
fn when_gem_shader_inspected_then_accepts_position_color_and_uv_vertex_inputs() {
    // Assert
    assert!(
        GEM_WGSL.contains("@location(0)"),
        "gem shader must accept position at @location(0)"
    );
    assert!(
        GEM_WGSL.contains("@location(1)"),
        "gem shader must accept color at @location(1)"
    );
    assert!(
        GEM_WGSL.contains("@location(2)"),
        "gem shader must accept uv at @location(2)"
    );
}

/// @doc: The gem shader must implement Blinn-Phong specular highlights using a half-vector
/// between the light (pointer) and view directions. Without specular computation, the gem
/// would be a flat polygon with no light-catching effect — defeating the entire feature.
#[test]
fn when_gem_shader_inspected_then_contains_specular_computation() {
    // Assert
    assert!(
        GEM_WGSL.contains("half_vec") || GEM_WGSL.contains("half_dir"),
        "gem shader must compute a half-vector for Blinn-Phong specular"
    );
    assert!(
        GEM_WGSL.contains("spec"),
        "gem shader must compute a specular term"
    );
}

/// @doc: The gem shader must compute per-facet normals rather than using continuous bevel
/// normals. Discrete facet normals create the characteristic flat-face glint pattern of a
/// cut gemstone; without them the gem looks like a smooth embossed button instead.
#[test]
fn when_gem_shader_inspected_then_contains_facet_normal_logic() {
    // Assert
    assert!(
        GEM_WGSL.contains("facet") || GEM_WGSL.contains("normal"),
        "gem shader must compute per-facet normals for the cut-gemstone look"
    );
}

#[test]
fn when_registering_gem_shader_then_handle_is_retrievable() {
    // Arrange
    let mut registry = ShaderRegistry::default();

    // Act
    let gem = register_gem_shader(&mut registry);

    // Assert
    assert_eq!(registry.lookup(gem.0), Some(GEM_WGSL));
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

/// @doc: `TierShaders` resource must hold valid shader handles for both Dormant
/// (worn/faded) and Intense (shiny) effects. If registration fails or returns
/// stale handles, the spawn pipeline will silently skip tier overlays, making
/// all cards look Active regardless of their actual tier.
#[test]
fn when_registering_tier_shaders_then_both_handles_are_retrievable() {
    // Arrange
    let mut registry = ShaderRegistry::default();

    // Act
    let tiers = register_tier_shaders(&mut registry);

    // Assert
    assert_eq!(registry.lookup(tiers.dormant), Some(DORMANT_WGSL));
    assert_eq!(registry.lookup(tiers.intense), Some(INTENSE_WGSL));
}
