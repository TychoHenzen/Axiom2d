// EVOLVE-BLOCK-START
use engine_render::font::{bake_balanced_text_into_mesh, bake_wrapped_text_into_mesh};
use engine_render::prelude::{rect_polygon, rounded_rect_path, tessellate};
use engine_render::shape::{Shape, ShapeVariant, TessellatedColorMesh};
use glam::Vec2;

use super::face_layout::FRONT_FACE_REGIONS;
use super::spawn_table_card::{CARD_CORNER_RADIUS, TEXT_COLOR, fit_name_font_size};
use crate::card::art::tessellate_art_shapes;
use crate::card::art_selection::fit_art_mesh_to_region;
use crate::card::component::CardLabel;
use crate::card::identity::definition::rarity_border_color;
use crate::card::identity::gem_sockets::{
    MAX_GEM_RADIUS, gem_color, gem_desc_positions, hexagon_vertices,
};
use crate::card::identity::signature::{CardSignature, Element};

const TEXT_COLOR_ARRAY: [f32; 4] = [TEXT_COLOR.r, TEXT_COLOR.g, TEXT_COLOR.b, TEXT_COLOR.a];

fn color_to_array(c: engine_core::color::Color) -> [f32; 4] {
    [c.r, c.g, c.b, c.a]
}

/// Tessellate all front-face geometry into a single mesh.
/// Geometry is appended back-to-front (painter's order):
/// border → name strip → art area bg → desc strip → text → gems
pub fn bake_front_face(
    signature: &CardSignature,
    card_size: Vec2,
    label: &CardLabel,
    art_shapes: Option<&[Shape]>,
) -> TessellatedColorMesh {
    let mut mesh = TessellatedColorMesh::new();
    let (w, h) = (card_size.x, card_size.y);

    let rarity = signature.rarity();
    let border_color = rarity_border_color(rarity, signature);

    // --- Shapes (border, strips) — skip art region (drawn with shader) ---
    for (i, region) in FRONT_FACE_REGIONS.iter().enumerate() {
        if region.use_art_shader {
            continue;
        }
        let (reg_hw, reg_hh, offset_y) = region.resolve(w, h);
        let color = match i {
            0 => color_to_array(border_color),
            _ => color_to_array(region.color),
        };
        let variant = if i == 0 {
            rounded_rect_path(reg_hw, reg_hh, CARD_CORNER_RADIUS)
        } else {
            rect_polygon(reg_hw, reg_hh)
        };
        if let Ok(tess) = tessellate(&variant) {
            let offset: Vec<[f32; 2]> = tess
                .vertices
                .iter()
                .map(|&[x, y]| [x, y + offset_y])
                .collect();
            mesh.push_vertices(&offset, &tess.indices, color);
        }
    }

    // --- Art shapes (tessellated vector art, fitted to art region) ---
    if let Some(shapes) = art_shapes {
        let art_mesh = tessellate_art_shapes(shapes);
        if !art_mesh.is_empty() {
            let (art_hw, art_hh, art_oy) = FRONT_FACE_REGIONS[2].resolve(w, h);
            let fitted = fit_art_mesh_to_region(&art_mesh, art_hw, art_hh, art_oy);
            let vertex_base = mesh.vertices.len() as u32;
            mesh.vertices.extend_from_slice(&fitted.vertices);
            mesh.indices
                .extend(fitted.indices.iter().map(|&i| i + vertex_base));
        }
    }

    // --- Name text (wraps to max 2 lines, sized to fit name strip) ---
    let (name_half_w, name_half_h, name_offset_y) = FRONT_FACE_REGIONS[1].resolve(w, h);
    let name_max_width = name_half_w * 2.0 * 0.9;
    let name_max_height = name_half_h * 2.0 * 0.9;
    let name_font_size = fit_name_font_size(&label.name, h / 12.0, name_max_width, name_max_height);
    // Shift text down by ~35% of font size to visually center glyphs
    // (baseline sits at y, but most glyph mass is above the baseline)
    let name_text_y = name_offset_y + name_font_size * 0.35;
    bake_balanced_text_into_mesh(
        &mut mesh,
        &label.name,
        name_font_size,
        TEXT_COLOR_ARRAY,
        0.0,
        name_text_y,
        name_max_width,
    );

    // --- Description text (wrapped) ---
    let (desc_half_w, _, desc_offset_y) = FRONT_FACE_REGIONS[3].resolve(w, h);
    let desc_font_size = h / 20.0;
    let desc_max_width = desc_half_w * 2.0 * 0.9;
    bake_wrapped_text_into_mesh(
        &mut mesh,
        &label.description,
        desc_font_size,
        TEXT_COLOR_ARRAY,
        0.0,
        desc_offset_y,
        desc_max_width,
    );

    // --- Gems ---
    let positions = gem_desc_positions(card_size);
    for (i, element) in Element::ALL.iter().enumerate() {
        let intensity = signature.intensity(*element);
        let aspect = signature.dominant_aspect(*element);
        let gem_color = color_to_array(gem_color(aspect, intensity));
        let radius = MAX_GEM_RADIUS;
        let verts = hexagon_vertices(radius);
        let points: Vec<_> = verts.to_vec();
        let variant = ShapeVariant::Polygon { points };
        if let Ok(tess) = tessellate(&variant) {
            let pos = positions[i];
            let offset: Vec<[f32; 2]> = tess
                .vertices
                .iter()
                .map(|&[x, y]| [x + pos.x, y + pos.y])
                .collect();
            mesh.push_vertices(&offset, &tess.indices, gem_color);
        }
    }

    mesh
}

/// Tessellate back-face geometry into a single mesh.
///
/// Draws a rounded-rect border (preserving the card silhouette) then fits
/// the `card_back` vector art inside the card bounds.
pub fn bake_back_face(card_size: Vec2) -> TessellatedColorMesh {
    let mut mesh = TessellatedColorMesh::new();
    let (w, h) = (card_size.x, card_size.y);

    // Rounded border background — preserves the card's rounded-corner shape
    let outer = rounded_rect_path(w * 0.5, h * 0.5, CARD_CORNER_RADIUS);
    if let Ok(tess) = tessellate(&outer) {
        mesh.push_vertices(
            &tess.vertices,
            &tess.indices,
            color_to_array(engine_core::color::Color::from_u8(30, 60, 120, 255)),
        );
    }

    // Card-back art — fitted within the card bounds (slightly inset to show border)
    let art_shapes = crate::card::art::card_back::card_front2();
    let art_mesh = crate::card::art::tessellate_art_shapes(&art_shapes);
    if !art_mesh.is_empty() {
        let inset = CARD_CORNER_RADIUS;
        let art_hw = w * 0.5 - inset;
        let art_hh = h * 0.5 - inset;
        let fitted =
            crate::card::art_selection::fit_art_mesh_to_region(&art_mesh, art_hw, art_hh, 0.0);
        let vertex_base = mesh.vertices.len() as u32;
        mesh.vertices.extend_from_slice(&fitted.vertices);
        mesh.indices
            .extend(fitted.indices.iter().map(|&i| i + vertex_base));
    }

    mesh
}
// EVOLVE-BLOCK-END
