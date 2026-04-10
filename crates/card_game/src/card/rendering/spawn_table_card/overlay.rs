// EVOLVE-BLOCK-START
use bevy_ecs::prelude::World;
use glam::Vec2;

use crate::card::identity::signature::CardSignature;
use crate::card::identity::signature::compute_seed;
use crate::card::identity::signature_profile::SignatureProfile;
use crate::card::identity::visual_params::generate_card_visuals;
use crate::card::rendering::art_shader::CardArtShader;
use crate::card::rendering::art_shader::{
    ArtRegionParams, ConditionEffect, ShaderVariant, TierShaders, VariantShaders,
};
use crate::card::rendering::face_layout::FRONT_FACE_REGIONS;
use engine_render::shape::{ColorVertex, MeshOverlays, OverlayEntry, TessellatedColorMesh};

pub(crate) fn build_mesh_overlays(
    world: &World,
    card_size: Vec2,
    signature: &CardSignature,
    face_up: bool,
    front_mesh: &TessellatedColorMesh,
) -> MeshOverlays {
    use crate::card::identity::gem_sockets::{
        MAX_GEM_RADIUS, gem_color, gem_desc_positions, gem_specular_intensity,
    };
    use crate::card::identity::signature::Element;

    let mut entries = Vec::new();
    let profile = SignatureProfile::without_archetype(signature);
    let visuals = generate_card_visuals(signature, &profile);

    let art_region = &FRONT_FACE_REGIONS[2];
    let (half_w, half_h, offset_y) = art_region.resolve(card_size.x, card_size.y);
    let art_params = ArtRegionParams {
        half_w,
        half_h,
        pointer_x: 0.0,
        pointer_y: 0.0,
        offset_y,
        extra0: 0.0,
        extra1: 0.0,
        extra2: 0.0,
    };
    let art_uniforms = bytemuck::bytes_of(&art_params).to_vec();

    if let Some(art_shader) = world.get_resource::<CardArtShader>().map(|s| s.0) {
        let c = [
            visuals.art_color.r,
            visuals.art_color.g,
            visuals.art_color.b,
            visuals.art_color.a,
        ];
        let quad_mesh = TessellatedColorMesh {
            vertices: vec![
                ColorVertex {
                    position: [-half_w, -half_h + offset_y],
                    color: c,
                    uv: [0.0, 0.0],
                },
                ColorVertex {
                    position: [half_w, -half_h + offset_y],
                    color: c,
                    uv: [1.0, 0.0],
                },
                ColorVertex {
                    position: [half_w, half_h + offset_y],
                    color: c,
                    uv: [1.0, 1.0],
                },
                ColorVertex {
                    position: [-half_w, half_h + offset_y],
                    color: c,
                    uv: [0.0, 1.0],
                },
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
        };
        entries.push(OverlayEntry {
            mesh: quad_mesh,
            material: engine_render::material::Material2d {
                shader: art_shader,
                uniforms: art_uniforms.clone(),
                ..engine_render::material::Material2d::default()
            },
            visible: face_up,
            front_only: true,
        });
    }

    let variant_shader = match visuals.shader_variant {
        ShaderVariant::None => None,
        variant => world
            .get_resource::<VariantShaders>()
            .map(|vs| match variant {
                ShaderVariant::Embossed => vs.embossed,
                ShaderVariant::Glow => vs.glow,
                ShaderVariant::Glossy => vs.glossy,
                ShaderVariant::Foil => vs.foil,
                ShaderVariant::None => unreachable!(),
            }),
    };

    if let Some(shader) = variant_shader {
        entries.push(OverlayEntry {
            mesh: front_mesh.clone(),
            material: engine_render::material::Material2d {
                shader,
                uniforms: art_uniforms.clone(),
                blend_mode: engine_render::material::BlendMode::Alpha,
                ..engine_render::material::Material2d::default()
            },
            visible: face_up,
            front_only: true,
        });
    }

    let condition = ConditionEffect::from_tier(visuals.tier_detail);
    let tier_shader = match condition {
        ConditionEffect::None => None,
        ConditionEffect::Worn => world.get_resource::<TierShaders>().map(|ts| ts.dormant),
        ConditionEffect::Shiny => world.get_resource::<TierShaders>().map(|ts| ts.intense),
    };

    if let Some(shader) = tier_shader {
        let (card_hw, card_hh) = (card_size.x * 0.5, card_size.y * 0.5);
        let white = [1.0, 1.0, 1.0, 1.0];
        let tier_quad = TessellatedColorMesh {
            vertices: vec![
                ColorVertex {
                    position: [-card_hw, -card_hh],
                    color: white,
                    uv: [0.0, 0.0],
                },
                ColorVertex {
                    position: [card_hw, -card_hh],
                    color: white,
                    uv: [1.0, 0.0],
                },
                ColorVertex {
                    position: [card_hw, card_hh],
                    color: white,
                    uv: [1.0, 1.0],
                },
                ColorVertex {
                    position: [-card_hw, card_hh],
                    color: white,
                    uv: [0.0, 1.0],
                },
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
        };
        let seed = compute_seed(signature);
        let seed_f32 = (seed & 0xFFFF_FFFF) as f32;
        let mut tier_uniforms = art_uniforms.clone();
        tier_uniforms[20..24].copy_from_slice(&seed_f32.to_le_bytes());
        entries.push(OverlayEntry {
            mesh: tier_quad,
            material: engine_render::material::Material2d {
                shader,
                uniforms: tier_uniforms,
                blend_mode: engine_render::material::BlendMode::Alpha,
                ..engine_render::material::Material2d::default()
            },
            visible: true,
            front_only: false,
        });
    }

    if let Some(gem_shader) = world
        .get_resource::<crate::card::rendering::art_shader::GemShader>()
        .map(|gs| gs.0)
    {
        let positions = gem_desc_positions(card_size);
        for (i, element) in Element::ALL.iter().enumerate() {
            let intensity = signature.intensity(*element);
            let aspect = signature.dominant_aspect(*element);
            let color = gem_color(aspect, intensity);
            let gem_color = [color.r, color.g, color.b, color.a];
            let radius = MAX_GEM_RADIUS;
            let specular = gem_specular_intensity(intensity);
            let mut overlay =
                build_gem_overlay(positions[i], radius, gem_color, specular, gem_shader);
            overlay.visible = face_up;
            entries.push(overlay);
        }
    }

    MeshOverlays(entries)
}

/// Build an overlay entry for a single faceted gem.
pub(crate) fn build_gem_overlay(
    center: glam::Vec2,
    radius: f32,
    color: [f32; 4],
    specular_intensity: f32,
    gem_shader: engine_render::prelude::ShaderHandle,
) -> engine_render::shape::OverlayEntry {
    use crate::card::identity::gem_sockets::{hexagon_uvs, hexagon_vertices};
    use engine_render::shape::{ColorVertex, OverlayEntry, TessellatedColorMesh};

    let verts = hexagon_vertices(radius);
    let uvs = hexagon_uvs(&verts);

    let vertices: Vec<ColorVertex> = verts
        .iter()
        .zip(uvs.iter())
        .map(|(v, uv)| ColorVertex {
            position: [v.x + center.x, v.y + center.y],
            color,
            uv: *uv,
        })
        .collect();

    let mut indices = Vec::with_capacity(12);
    for i in 1..5u32 {
        indices.push(0);
        indices.push(i);
        indices.push(i + 1);
    }

    let mesh = TessellatedColorMesh { vertices, indices };

    let params = ArtRegionParams {
        half_w: radius,
        half_h: radius,
        pointer_x: 0.0,
        pointer_y: 0.0,
        offset_y: center.y,
        extra0: specular_intensity,
        extra1: 0.0,
        extra2: 0.0,
    };

    OverlayEntry {
        mesh,
        material: engine_render::material::Material2d {
            shader: gem_shader,
            uniforms: bytemuck::bytes_of(&params).to_vec(),
            blend_mode: engine_render::material::BlendMode::Additive,
            ..engine_render::material::Material2d::default()
        },
        visible: true,
        front_only: true,
    }
}
// EVOLVE-BLOCK-END
