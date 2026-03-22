use std::collections::HashMap;

use glam::Vec2;
use ttf_parser::GlyphId;

use crate::shape::{PathCommand, ShapeVariant, TessellatedColorMesh, TessellatedMesh, tessellate};

pub const FONT_BYTES: &[u8] = include_bytes!("../assets/JetBrainsMono-Regular.ttf");

struct OutlineBuilder {
    commands: Vec<PathCommand>,
    scale: f32,
}

impl ttf_parser::OutlineBuilder for OutlineBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.commands.push(PathCommand::MoveTo(Vec2::new(
            x * self.scale,
            -y * self.scale,
        )));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.commands.push(PathCommand::LineTo(Vec2::new(
            x * self.scale,
            -y * self.scale,
        )));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.commands.push(PathCommand::QuadraticTo {
            control: Vec2::new(x1 * self.scale, -y1 * self.scale),
            to: Vec2::new(x * self.scale, -y * self.scale),
        });
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.commands.push(PathCommand::CubicTo {
            control1: Vec2::new(x1 * self.scale, -y1 * self.scale),
            control2: Vec2::new(x2 * self.scale, -y2 * self.scale),
            to: Vec2::new(x * self.scale, -y * self.scale),
        });
    }

    fn close(&mut self) {
        self.commands.push(PathCommand::Close);
    }
}

pub fn outline_glyph(
    face: &ttf_parser::Face,
    glyph_id: GlyphId,
    font_size: f32,
) -> Vec<PathCommand> {
    let scale = font_size / f32::from(face.units_per_em());
    let mut builder = OutlineBuilder {
        commands: Vec::new(),
        scale,
    };
    face.outline_glyph(glyph_id, &mut builder);
    builder.commands
}

#[derive(Default)]
pub struct GlyphCache {
    entries: HashMap<(GlyphId, u16), TessellatedMesh>,
}

impl GlyphCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_or_tessellate(
        &mut self,
        face: &ttf_parser::Face,
        glyph_id: GlyphId,
        font_size: f32,
    ) -> &TessellatedMesh {
        let key = (glyph_id, font_size.round() as u16);
        self.entries.entry(key).or_insert_with(|| {
            let commands = outline_glyph(face, glyph_id, font_size);
            tessellate_glyph(&commands)
        })
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

pub struct LayoutGlyph {
    pub glyph_id: GlyphId,
    pub x_offset: f32,
}

pub fn layout_text(face: &ttf_parser::Face, text: &str, font_size: f32) -> Vec<LayoutGlyph> {
    let scale = font_size / f32::from(face.units_per_em());
    let mut x = 0.0_f32;
    let mut glyphs = Vec::new();
    for ch in text.chars() {
        let Some(glyph_id) = face.glyph_index(ch) else {
            continue;
        };
        glyphs.push(LayoutGlyph {
            glyph_id,
            x_offset: x,
        });
        if let Some(advance) = face.glyph_hor_advance(glyph_id) {
            x += f32::from(advance) * scale;
        }
    }
    glyphs
}

pub fn measure_text(text: &str, font_size: f32) -> f32 {
    // INVARIANT: FONT_BYTES is a compile-time embedded TTF file.
    // Parsing cannot fail unless the binary is corrupted.
    let face = ttf_parser::Face::parse(FONT_BYTES, 0).expect("embedded font is valid");
    measure_text_with_face(&face, text, font_size)
}

fn measure_text_with_face(face: &ttf_parser::Face, text: &str, font_size: f32) -> f32 {
    let scale = font_size / f32::from(face.units_per_em());
    let mut width = 0.0_f32;
    for ch in text.chars() {
        if let Some(glyph_id) = face.glyph_index(ch)
            && let Some(advance) = face.glyph_hor_advance(glyph_id)
        {
            width += f32::from(advance) * scale;
        }
    }
    width
}

/// Split text into lines that fit within `max_width` pixels at the given font size.
/// Wraps at word boundaries (spaces). Words that exceed `max_width` on their own
/// are placed on a line by themselves (no mid-word breaking).
pub fn wrap_text(text: &str, font_size: f32, max_width: f32) -> Vec<String> {
    // INVARIANT: FONT_BYTES is a compile-time embedded TTF file.
    let face = ttf_parser::Face::parse(FONT_BYTES, 0).expect("embedded font is valid");
    let words: Vec<&str> = text.split(' ').collect();
    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0.0_f32;
    let space_width = measure_text_with_face(&face, " ", font_size);

    for word in &words {
        let word_width = measure_text_with_face(&face, word, font_size);
        if current_line.is_empty() {
            current_line.push_str(word);
            current_width = word_width;
        } else {
            let width_with_word = current_width + space_width + word_width;
            if width_with_word <= max_width {
                current_line.push(' ');
                current_line.push_str(word);
                current_width = width_with_word;
            } else {
                lines.push(std::mem::take(&mut current_line));
                current_line.push_str(word);
                current_width = word_width;
            }
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

#[allow(clippy::too_many_arguments)]
pub fn render_text_glyphs(
    renderer: &mut dyn crate::renderer::Renderer,
    cache: &mut GlyphCache,
    text: &str,
    x: f32,
    y: f32,
    font_size: f32,
    color: engine_core::color::Color,
) {
    let base = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [x, y, 0.0, 1.0],
    ];
    render_text_transformed(renderer, cache, text, &base, font_size, color);
}

#[allow(clippy::too_many_arguments)]
pub fn render_text_transformed(
    renderer: &mut dyn crate::renderer::Renderer,
    cache: &mut GlyphCache,
    text: &str,
    base_model: &[[f32; 4]; 4],
    font_size: f32,
    color: engine_core::color::Color,
) {
    // INVARIANT: FONT_BYTES is a compile-time embedded TTF file.
    let face = ttf_parser::Face::parse(FONT_BYTES, 0).expect("embedded font is valid");
    let glyphs = layout_text(&face, text, font_size);
    for glyph in &glyphs {
        let mesh = cache.get_or_tessellate(&face, glyph.glyph_id, font_size);
        if mesh.vertices.is_empty() {
            continue;
        }
        let model = multiply_model_translate(base_model, glyph.x_offset, 0.0);
        renderer.draw_shape(&mesh.vertices, &mesh.indices, color, model);
    }
}

fn multiply_model_translate(base: &[[f32; 4]; 4], dx: f32, dy: f32) -> [[f32; 4]; 4] {
    let mut result = *base;
    result[3][0] += base[0][0] * dx + base[1][0] * dy;
    result[3][1] += base[0][1] * dx + base[1][1] * dy;
    result
}

pub fn tessellate_glyph(commands: &[PathCommand]) -> TessellatedMesh {
    if commands.is_empty() {
        return TessellatedMesh {
            vertices: Vec::new(),
            indices: Vec::new(),
        };
    }
    tessellate(&ShapeVariant::Path {
        commands: commands.to_vec(),
    })
    .unwrap_or_else(|_| TessellatedMesh {
        vertices: Vec::new(),
        indices: Vec::new(),
    })
}

/// Tessellate text glyphs and append them to an existing mesh with pre-applied
/// position offsets and uniform color. Text is centered horizontally around `base_x`.
pub fn bake_text_into_mesh(
    mesh: &mut TessellatedColorMesh,
    text: &str,
    font_size: f32,
    color: [f32; 4],
    base_x: f32,
    base_y: f32,
) {
    let face = ttf_parser::Face::parse(FONT_BYTES, 0).expect("embedded font is valid");
    let text_width = measure_text_with_face(&face, text, font_size);
    let center_x = base_x - text_width * 0.5;
    let glyphs = layout_text(&face, text, font_size);
    let mut cache = GlyphCache::new();
    for glyph in &glyphs {
        let glyph_mesh = cache.get_or_tessellate(&face, glyph.glyph_id, font_size);
        if glyph_mesh.vertices.is_empty() {
            continue;
        }
        let offset_x = center_x + glyph.x_offset;
        let transformed: Vec<[f32; 2]> = glyph_mesh
            .vertices
            .iter()
            .map(|&[x, y]| [x + offset_x, y + base_y])
            .collect();
        mesh.push_vertices(&transformed, &glyph_mesh.indices, color);
    }
}

/// Same as `bake_text_into_mesh` but with word wrapping.
pub fn bake_wrapped_text_into_mesh(
    mesh: &mut TessellatedColorMesh,
    text: &str,
    font_size: f32,
    color: [f32; 4],
    base_x: f32,
    base_y: f32,
    max_width: f32,
) {
    let lines = wrap_text(text, font_size, max_width);
    let line_height = font_size * 1.3;
    let total_height = (lines.len() as f32 - 1.0) * line_height;
    let start_y = base_y - total_height * 0.5;
    for (i, line) in lines.iter().enumerate() {
        let y_offset = start_y + i as f32 * line_height;
        bake_text_into_mesh(mesh, line, font_size, color, base_x, y_offset);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn when_parsing_embedded_font_then_valid_face_with_ascii_glyphs() {
        // Arrange / Act
        let face = ttf_parser::Face::parse(FONT_BYTES, 0).unwrap();

        // Assert
        assert!(face.number_of_glyphs() > 0);
        assert!(face.units_per_em() > 0);
        assert!(face.glyph_index('A').is_some());
    }

    #[test]
    fn when_outlining_glyph_a_then_path_commands_are_non_empty() {
        // Arrange
        let face = ttf_parser::Face::parse(FONT_BYTES, 0).unwrap();
        let glyph_id = face.glyph_index('A').unwrap();

        // Act
        let commands = outline_glyph(&face, glyph_id, 16.0);

        // Assert
        assert!(
            !commands.is_empty(),
            "outline of 'A' must produce path commands"
        );
        assert!(
            commands.iter().any(|c| matches!(c, PathCommand::MoveTo(_))),
            "outline must contain at least one MoveTo"
        );
    }

    #[test]
    fn when_outlining_glyph_a_at_double_size_then_coordinates_scale() {
        // Arrange
        let face = ttf_parser::Face::parse(FONT_BYTES, 0).unwrap();
        let glyph_id = face.glyph_index('A').unwrap();

        // Act
        let small = outline_glyph(&face, glyph_id, 16.0);
        let large = outline_glyph(&face, glyph_id, 32.0);

        // Assert
        let first_pt = |cmds: &[PathCommand]| {
            cmds.iter().find_map(|c| {
                if let PathCommand::MoveTo(p) = c {
                    Some(*p)
                } else {
                    None
                }
            })
        };
        let pt_small = first_pt(&small).expect("small outline has no MoveTo");
        let pt_large = first_pt(&large).expect("large outline has no MoveTo");
        assert!(
            (pt_large - pt_small * 2.0).length() < 1e-3,
            "doubling font_size must double coordinates; small={pt_small:?} large={pt_large:?}"
        );
    }

    #[test]
    fn when_tessellating_glyph_a_then_mesh_is_non_empty() {
        // Arrange
        let face = ttf_parser::Face::parse(FONT_BYTES, 0).unwrap();
        let glyph_id = face.glyph_index('A').unwrap();
        let commands = outline_glyph(&face, glyph_id, 32.0);

        // Act
        let mesh = tessellate_glyph(&commands);

        // Assert
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());
        assert_eq!(mesh.indices.len() % 3, 0);
    }

    #[test]
    fn when_tessellating_glyph_a_then_all_indices_within_bounds() {
        // Arrange
        let face = ttf_parser::Face::parse(FONT_BYTES, 0).unwrap();
        let glyph_id = face.glyph_index('A').unwrap();
        let commands = outline_glyph(&face, glyph_id, 32.0);

        // Act
        let mesh = tessellate_glyph(&commands);

        // Assert
        let vertex_count = mesh.vertices.len() as u32;
        for &index in &mesh.indices {
            assert!(
                index < vertex_count,
                "index {index} out of bounds (vertex count {vertex_count})"
            );
        }
    }

    #[test]
    fn when_tessellating_empty_commands_then_mesh_is_empty() {
        // Arrange
        let commands: Vec<PathCommand> = Vec::new();

        // Act
        let mesh = tessellate_glyph(&commands);

        // Assert
        assert!(mesh.vertices.is_empty());
        assert!(mesh.indices.is_empty());
    }

    #[test]
    fn when_cache_queried_twice_for_same_glyph_then_same_vertex_count() {
        // Arrange
        let face = ttf_parser::Face::parse(FONT_BYTES, 0).unwrap();
        let glyph_id = face.glyph_index('A').unwrap();
        let mut cache = GlyphCache::new();

        // Act
        let (verts1, idxs1) = {
            let m = cache.get_or_tessellate(&face, glyph_id, 32.0);
            (m.vertices.len(), m.indices.len())
        };
        let (verts2, idxs2) = {
            let m = cache.get_or_tessellate(&face, glyph_id, 32.0);
            (m.vertices.len(), m.indices.len())
        };

        // Assert
        assert_eq!(verts1, verts2);
        assert_eq!(idxs1, idxs2);
    }

    #[test]
    fn when_cache_queried_at_different_sizes_then_separate_entries() {
        // Arrange
        let face = ttf_parser::Face::parse(FONT_BYTES, 0).unwrap();
        let glyph_id = face.glyph_index('A').unwrap();
        let mut cache = GlyphCache::new();

        // Act
        let small_verts = cache
            .get_or_tessellate(&face, glyph_id, 16.0)
            .vertices
            .len();
        let large_verts = cache
            .get_or_tessellate(&face, glyph_id, 32.0)
            .vertices
            .len();

        // Assert
        assert!(
            large_verts >= small_verts,
            "32px mesh should have at least as many vertices as 16px"
        );
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn when_laying_out_single_char_then_x_offset_is_zero() {
        // Arrange
        let face = ttf_parser::Face::parse(FONT_BYTES, 0).unwrap();

        // Act
        let glyphs = layout_text(&face, "A", 32.0);

        // Assert
        assert_eq!(glyphs.len(), 1);
        assert!((glyphs[0].x_offset - 0.0).abs() < 1e-6);
    }

    #[test]
    fn when_laying_out_two_chars_then_second_offset_equals_first_advance() {
        // Arrange
        let face = ttf_parser::Face::parse(FONT_BYTES, 0).unwrap();
        let scale = 32.0 / f32::from(face.units_per_em());
        let a_id = face.glyph_index('A').unwrap();
        let expected_advance = f32::from(face.glyph_hor_advance(a_id).unwrap()) * scale;

        // Act
        let glyphs = layout_text(&face, "AB", 32.0);

        // Assert
        assert_eq!(glyphs.len(), 2);
        assert!(
            (glyphs[1].x_offset - expected_advance).abs() < 1e-3,
            "expected {expected_advance}, got {}",
            glyphs[1].x_offset
        );
    }

    #[test]
    fn when_laying_out_hello_then_x_offsets_monotonically_increase() {
        // Arrange
        let face = ttf_parser::Face::parse(FONT_BYTES, 0).unwrap();

        // Act
        let glyphs = layout_text(&face, "Hello", 24.0);

        // Assert
        assert_eq!(glyphs.len(), 5);
        for i in 1..glyphs.len() {
            assert!(
                glyphs[i].x_offset > glyphs[i - 1].x_offset,
                "offset[{}]={} should be > offset[{}]={}",
                i,
                glyphs[i].x_offset,
                i - 1,
                glyphs[i - 1].x_offset
            );
        }
    }

    #[test]
    fn when_laying_out_string_with_space_then_space_advances_cursor() {
        // Arrange
        let face = ttf_parser::Face::parse(FONT_BYTES, 0).unwrap();

        // Act
        let glyphs = layout_text(&face, "A B", 32.0);

        // Assert
        let a_only = layout_text(&face, "AB", 32.0);
        assert!(
            glyphs.last().unwrap().x_offset > a_only.last().unwrap().x_offset,
            "'A B' should place 'B' further right than 'AB'"
        );
    }

    fn make_spy() -> (crate::testing::SpyRenderer, crate::testing::ShapeCallLog) {
        let log = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let shape_calls: crate::testing::ShapeCallLog =
            std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let spy = crate::testing::SpyRenderer::new(log).with_shape_capture(shape_calls.clone());
        (spy, shape_calls)
    }

    #[test]
    fn when_render_text_single_char_then_one_draw_shape_call() {
        // Arrange
        let (mut spy, shape_calls) = make_spy();
        let mut cache = GlyphCache::new();

        // Act
        render_text_glyphs(
            &mut spy,
            &mut cache,
            "A",
            0.0,
            0.0,
            32.0,
            engine_core::color::Color::WHITE,
        );

        // Assert
        let calls = shape_calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
    }

    #[test]
    fn when_render_text_three_chars_then_three_draw_shape_calls() {
        // Arrange
        let (mut spy, shape_calls) = make_spy();
        let mut cache = GlyphCache::new();

        // Act
        render_text_glyphs(
            &mut spy,
            &mut cache,
            "Hi!",
            0.0,
            0.0,
            32.0,
            engine_core::color::Color::WHITE,
        );

        // Assert
        let calls = shape_calls.lock().unwrap();
        assert_eq!(calls.len(), 3);
    }

    #[test]
    fn when_render_text_with_space_then_space_not_drawn() {
        // Arrange
        let (mut spy, shape_calls) = make_spy();
        let mut cache = GlyphCache::new();

        // Act
        render_text_glyphs(
            &mut spy,
            &mut cache,
            "A B",
            0.0,
            0.0,
            32.0,
            engine_core::color::Color::WHITE,
        );

        // Assert
        let calls = shape_calls.lock().unwrap();
        assert_eq!(calls.len(), 2, "space should not produce a draw_shape call");
    }

    #[test]
    fn when_render_text_two_chars_then_second_model_translated_right() {
        // Arrange
        let (mut spy, shape_calls) = make_spy();
        let mut cache = GlyphCache::new();

        // Act
        render_text_glyphs(
            &mut spy,
            &mut cache,
            "AB",
            10.0,
            20.0,
            32.0,
            engine_core::color::Color::WHITE,
        );

        // Assert
        let calls = shape_calls.lock().unwrap();
        assert_eq!(calls.len(), 2);
        let x0 = calls[0].3[3][0]; // model[3][0] = x translation
        let x1 = calls[1].3[3][0];
        assert!(
            x1 > x0,
            "second glyph x={x1} should be right of first x={x0}"
        );
    }

    #[test]
    fn when_render_text_with_color_then_draw_shape_receives_color() {
        // Arrange
        let (mut spy, shape_calls) = make_spy();
        let mut cache = GlyphCache::new();
        let red = engine_core::color::Color::new(1.0, 0.0, 0.0, 1.0);

        // Act
        render_text_glyphs(&mut spy, &mut cache, "A", 0.0, 0.0, 32.0, red);

        // Assert
        let calls = shape_calls.lock().unwrap();
        assert_eq!(calls[0].2, red);
    }

    #[test]
    fn when_wrap_text_fits_in_one_line_then_single_line_returned() {
        // Arrange
        let text = "Hello";
        let font_size = 16.0;
        let max_width = measure_text(text, font_size) + 10.0;

        // Act
        let lines = wrap_text(text, font_size, max_width);

        // Assert
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "Hello");
    }

    #[test]
    fn when_wrap_text_exceeds_width_then_multiple_lines() {
        // Arrange
        let font_size = 16.0;
        let word_width = measure_text("Hello", font_size);
        let max_width = word_width * 1.5;

        // Act
        let lines = wrap_text("Hello World", font_size, max_width);

        // Assert
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "Hello");
        assert_eq!(lines[1], "World");
    }

    #[test]
    fn when_wrap_text_three_words_narrow_then_three_lines() {
        // Arrange
        let font_size = 16.0;
        let word_width = measure_text("Deal", font_size);
        let max_width = word_width * 1.2;

        // Act
        let lines = wrap_text("Deal 3 damage", font_size, max_width);

        // Assert
        assert!(
            lines.len() >= 2,
            "should wrap into multiple lines, got {lines:?}"
        );
    }

    #[test]
    fn when_wrap_text_empty_then_single_empty_line() {
        // Act
        let lines = wrap_text("", 16.0, 100.0);

        // Assert
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "");
    }

    #[test]
    fn when_bake_text_single_char_then_mesh_is_nonempty() {
        // Arrange
        let mut mesh = TessellatedColorMesh::new();
        let color = [0.1, 0.1, 0.1, 1.0];

        // Act
        bake_text_into_mesh(&mut mesh, "A", 16.0, color, 0.0, 0.0);

        // Assert
        assert!(!mesh.is_empty());
        assert!(mesh.vertices.iter().all(|v| v.color == color));
    }

    #[test]
    fn when_bake_text_two_chars_then_second_glyph_vertices_offset_right() {
        // Arrange
        let mut mesh = TessellatedColorMesh::new();
        let color = [1.0, 1.0, 1.0, 1.0];

        // Act
        bake_text_into_mesh(&mut mesh, "AB", 32.0, color, 0.0, 0.0);

        // Assert — vertices should span a wider range than single-char
        let mut single = TessellatedColorMesh::new();
        bake_text_into_mesh(&mut single, "A", 32.0, color, 0.0, 0.0);
        let max_x = mesh
            .vertices
            .iter()
            .map(|v| v.position[0])
            .fold(f32::NEG_INFINITY, f32::max);
        let single_max_x = single
            .vertices
            .iter()
            .map(|v| v.position[0])
            .fold(f32::NEG_INFINITY, f32::max);
        assert!(
            max_x > single_max_x,
            "two chars should extend further right"
        );
    }

    #[test]
    fn when_wrap_text_single_long_word_then_one_line() {
        // Arrange
        let font_size = 16.0;
        let word_width = measure_text("Supercalifragilistic", font_size);

        // Act
        let lines = wrap_text("Supercalifragilistic", font_size, word_width * 0.5);

        // Assert
        assert_eq!(lines.len(), 1, "single word should not be split");
        assert_eq!(lines[0], "Supercalifragilistic");
    }
}
