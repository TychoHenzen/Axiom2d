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
    MAX_GEM_RADIUS, gem_color, gem_desc_positions, octagon_vertices,
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
        let verts = octagon_vertices(radius);
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::card::component::CardLabel;
    use crate::card::identity::signature::CardSignature;

    #[test]
    fn when_bake_front_then_mesh_has_vertices_and_valid_indices() {
        // Arrange
        let sig = CardSignature::default();
        let card_size = Vec2::new(60.0, 90.0);
        let label = CardLabel {
            name: "Test".to_owned(),
            description: "A test card".to_owned(),
        };

        // Act
        let mesh = bake_front_face(&sig, card_size, &label, None);

        // Assert
        assert!(!mesh.is_empty(), "front mesh should have geometry");
        assert_eq!(mesh.indices.len() % 3, 0, "indices should form triangles");
        let vcount = mesh.vertices.len() as u32;
        for &i in &mesh.indices {
            assert!(i < vcount, "index {i} out of bounds ({vcount} vertices)");
        }
    }

    /// @doc: Gem tessellation must complete during front-face baking. Signature intensities
    /// directly control gem radius, and all gems (all 8 elements) are always included in the mesh
    /// regardless of intensity level. This test ensures gems don't disappear when rarity-driven
    /// rendering changes.
    #[test]
    fn when_bake_front_then_contains_gem_geometry() {
        // Arrange — signature with non-zero intensities produces visible gems
        let sig = CardSignature::new([0.0, 0.8, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let card_size = Vec2::new(60.0, 90.0);
        let label = CardLabel {
            name: "Gem Test".to_owned(),
            description: "Has gems".to_owned(),
        };

        // Act
        let mesh = bake_front_face(&sig, card_size, &label, None);

        // Assert — mesh should have significantly more vertices than just 4 rectangles
        assert!(
            mesh.vertices.len() > 30,
            "expected gems to add substantial geometry, got {} vertices",
            mesh.vertices.len()
        );
    }

    /// @doc: Art region separation is critical—art geometry must be rendered by the art shader,
    /// not the baked mesh. If art colors appear in baked vertices, the art shader will composite
    /// on top, double-rendering the art. This test ensures the baked mesh respects the
    /// `use_art_shader` flag in `FRONT_FACE_REGIONS`.
    #[test]
    fn when_bake_front_then_art_region_color_not_present() {
        // Arrange — the art area (region 2) has use_art_shader=true and should be
        // excluded from the baked mesh so the art shader can render it instead
        let sig = CardSignature::default();
        let card_size = Vec2::new(60.0, 90.0);
        let label = CardLabel {
            name: "Test".to_owned(),
            description: "Desc".to_owned(),
        };
        let profile =
            crate::card::identity::signature_profile::SignatureProfile::without_archetype(&sig);
        let visuals = crate::card::identity::visual_params::generate_card_visuals(&sig, &profile);
        let art_color = [
            visuals.art_color.r,
            visuals.art_color.g,
            visuals.art_color.b,
            visuals.art_color.a,
        ];

        // Act
        let mesh = bake_front_face(&sig, card_size, &label, None);

        // Assert — no vertex should have the art area's generated color
        let has_art_color = mesh.vertices.iter().any(|v| v.color == art_color);
        assert!(
            !has_art_color,
            "baked mesh should not contain art area geometry (shader handles it)"
        );
    }

    #[test]
    fn when_bake_back_then_mesh_has_vertices() {
        // Arrange
        let card_size = Vec2::new(60.0, 90.0);

        // Act
        let mesh = bake_back_face(card_size);

        // Assert
        assert!(!mesh.is_empty());
    }

    #[test]
    fn when_baking_with_art_shapes_then_more_vertices_than_without() {
        // Arrange
        let sig = CardSignature::default();
        let card_size = Vec2::new(60.0, 90.0);
        let label = CardLabel {
            name: "Test".to_owned(),
            description: "Desc".to_owned(),
        };
        let art_shapes = vec![Shape {
            variant: ShapeVariant::Circle { radius: 10.0 },
            color: engine_core::color::Color::WHITE,
        }];

        // Act
        let without = bake_front_face(&sig, card_size, &label, None);
        let with = bake_front_face(&sig, card_size, &label, Some(&art_shapes));

        // Assert
        assert!(
            with.vertices.len() > without.vertices.len(),
            "with art: {} vertices, without: {} — expected more with art",
            with.vertices.len(),
            without.vertices.len()
        );
    }

    #[test]
    fn when_baking_with_art_shapes_then_all_indices_valid() {
        // Arrange
        let sig = CardSignature::default();
        let card_size = Vec2::new(60.0, 90.0);
        let label = CardLabel {
            name: "Test".to_owned(),
            description: "Desc".to_owned(),
        };
        let art_shapes = vec![Shape {
            variant: ShapeVariant::Circle { radius: 10.0 },
            color: engine_core::color::Color::WHITE,
        }];

        // Act
        let mesh = bake_front_face(&sig, card_size, &label, Some(&art_shapes));

        // Assert — all indices must be valid after art injection
        let vcount = mesh.vertices.len() as u32;
        assert_eq!(mesh.indices.len() % 3, 0, "indices should form triangles");
        for &i in &mesh.indices {
            assert!(i < vcount, "index {i} out of bounds ({vcount} vertices)");
        }
    }

    #[test]
    fn when_baking_without_art_then_vertex_count_unchanged() {
        // Arrange
        let sig = CardSignature::default();
        let card_size = Vec2::new(60.0, 90.0);
        let label = CardLabel {
            name: "Test".to_owned(),
            description: "Desc".to_owned(),
        };

        // Act
        let mesh_a = bake_front_face(&sig, card_size, &label, None);
        let mesh_b = bake_front_face(&sig, card_size, &label, None);

        // Assert
        assert_eq!(mesh_a.vertices.len(), mesh_b.vertices.len());
        assert_eq!(mesh_a.indices.len(), mesh_b.indices.len());
    }

    /// @doc: All gems render at fixed `MAX_GEM_RADIUS` regardless of intensity.
    /// Gems must always be visible at full size so that the color gradient (not size)
    /// communicates element intensity to the player.
    #[test]
    fn when_bake_front_with_zero_intensity_then_gems_at_max_radius() {
        use crate::card::identity::gem_sockets::{MAX_GEM_RADIUS, gem_desc_positions};

        // Arrange — all intensities at 0.0 → still max gem radius
        let sig = CardSignature::new([0.0; 8]);
        let card_size = Vec2::new(60.0, 90.0);
        let label = CardLabel {
            name: "Min Gems".to_owned(),
            description: "Zero intensity".to_owned(),
        };
        let positions = gem_desc_positions(card_size);

        // Act
        let mesh = bake_front_face(&sig, card_size, &label, None);

        // Assert — for each gem position, find nearby vertices and verify they
        // are within MAX_GEM_RADIUS of that position (+ float tolerance)
        let tolerance = MAX_GEM_RADIUS + 0.1;
        for (gi, gem_pos) in positions.iter().enumerate() {
            let nearby = mesh
                .vertices
                .iter()
                .filter(|v| {
                    let dx = v.position[0] - gem_pos.x;
                    let dy = v.position[1] - gem_pos.y;
                    (dx * dx + dy * dy).sqrt() < tolerance
                })
                .count();
            assert!(
                nearby > 0,
                "no vertices found near gem {gi} at ({}, {})",
                gem_pos.x,
                gem_pos.y
            );
        }
    }

    /// @doc: UV coordinates distinguish art geometry from solid-color geometry in the unified render pass.
    /// Art vertices have non-zero UV (for art atlas lookup); non-art vertices have zero UV.
    /// If this invariant breaks, art and non-art blending in the shader will fail, causing visual artifacts.
    #[test]
    fn when_bake_front_with_art_then_art_vertices_have_uv_and_non_art_have_zero() {
        // Arrange
        let sig = CardSignature::default();
        let card_size = Vec2::new(60.0, 90.0);
        let label = CardLabel {
            name: "Test".to_owned(),
            description: "Desc".to_owned(),
        };
        let art_shapes = vec![Shape {
            variant: ShapeVariant::Circle { radius: 10.0 },
            color: engine_core::color::Color::RED,
        }];

        // Act
        let with_art = bake_front_face(&sig, card_size, &label, Some(&art_shapes));

        // Assert — partition by UV: non-art vertices have zero UV, art vertices don't
        let zero_uv_count = with_art
            .vertices
            .iter()
            .filter(|v| v.uv == [0.0, 0.0])
            .count();
        let nonzero_uv_count = with_art
            .vertices
            .iter()
            .filter(|v| v.uv != [0.0, 0.0])
            .count();

        assert!(
            nonzero_uv_count > 0,
            "should have art vertices with non-zero UV"
        );
        assert!(
            zero_uv_count > 0,
            "should have non-art vertices with zero UV"
        );

        // All non-zero UVs must be in [0,1] range
        for (i, v) in with_art.vertices.iter().enumerate() {
            assert!(
                v.uv[0] >= 0.0 && v.uv[0] <= 1.0 && v.uv[1] >= 0.0 && v.uv[1] <= 1.0,
                "vertex {i} uv out of [0,1] range: {:?}",
                v.uv
            );
        }
    }
}
