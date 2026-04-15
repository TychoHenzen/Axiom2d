use std::collections::HashMap;

use glam::Vec2;
use ttf_parser::GlyphId;

use crate::shape::{PathCommand, ShapeVariant, TessellatedColorMesh, TessellatedMesh, tessellate};

pub const FONT_BYTES: &[u8] = include_bytes!("../assets/JetBrainsMono-Regular.ttf");

fn embedded_font_face() -> ttf_parser::Face<'static> {
    ttf_parser::Face::parse(FONT_BYTES, 0).expect("embedded font is valid")
}

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
    entries: HashMap<(GlyphId, u32), TessellatedMesh>,
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
        let key = (glyph_id, font_size.to_bits());
        self.entries.entry(key).or_insert_with(|| {
            let commands = outline_glyph(face, glyph_id, font_size);
            tessellate_glyph(&commands)
        })
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

pub struct LayoutGlyph {
    pub glyph_id: GlyphId,
    pub x_offset: f32,
}

pub fn layout_text(face: &ttf_parser::Face, text: &str, font_size: f32) -> Vec<LayoutGlyph> {
    let scale = font_size / f32::from(face.units_per_em());
    let mut x = 0.0_f32;
    let mut glyphs = Vec::with_capacity(text.len());
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
    let face = embedded_font_face();
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
    let face = embedded_font_face();
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

/// Balanced wrapping: splits text into at most 2 lines, trying to keep
/// line lengths roughly equal. Falls back to greedy `wrap_text` if
/// the text fits on 1 line or has only 1 word.
pub fn balanced_wrap_text(text: &str, font_size: f32, max_width: f32) -> Vec<String> {
    let face = embedded_font_face();
    let full_width = measure_text_with_face(&face, text, font_size);
    if full_width <= max_width {
        return vec![text.to_string()];
    }
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return vec![String::new()];
    }
    if words.len() == 1 {
        return vec![words[0].to_string()];
    }
    let space_width = measure_text_with_face(&face, " ", font_size);
    let mut prefix_widths = Vec::with_capacity(words.len() + 1);
    prefix_widths.push(0.0_f32);
    for word in &words {
        let width = measure_text_with_face(&face, word, font_size);
        let previous_width = prefix_widths
            .last()
            .copied()
            .expect("prefix widths always starts with a zero entry");
        prefix_widths.push(previous_width + width);
    }
    let total_word_width = prefix_widths
        .last()
        .copied()
        .expect("prefix widths always contains a total width");
    let mut best_split = None;
    let mut best_diff = f32::MAX;
    for (split, _) in prefix_widths.iter().enumerate().take(words.len()).skip(1) {
        let left_width = prefix_widths[split] + space_width * (split - 1) as f32;
        let right_width = (total_word_width - prefix_widths[split])
            + space_width * (words.len() - split - 1) as f32;
        if left_width <= max_width && right_width <= max_width {
            let diff = (left_width - right_width).abs();
            if diff < best_diff {
                best_diff = diff;
                best_split = Some(split);
            }
        }
    }
    let Some(best_split) = best_split else {
        return wrap_text(text, font_size, max_width);
    };
    vec![words[..best_split].join(" "), words[best_split..].join(" ")]
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
    let face = embedded_font_face();
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
#[allow(clippy::too_many_arguments)]
pub fn bake_text_into_mesh(
    mesh: &mut TessellatedColorMesh,
    text: &str,
    font_size: f32,
    color: [f32; 4],
    base_x: f32,
    base_y: f32,
) {
    let face = embedded_font_face();
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
#[allow(clippy::too_many_arguments)]
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
    bake_lines_into_mesh(mesh, &lines, font_size, color, base_x, base_y);
}

/// Same as `bake_wrapped_text_into_mesh` but uses balanced line-splitting
/// (roughly equal line widths) capped at 2 lines.
#[allow(clippy::too_many_arguments)]
pub fn bake_balanced_text_into_mesh(
    mesh: &mut TessellatedColorMesh,
    text: &str,
    font_size: f32,
    color: [f32; 4],
    base_x: f32,
    base_y: f32,
    max_width: f32,
) {
    let lines = balanced_wrap_text(text, font_size, max_width);
    bake_lines_into_mesh(mesh, &lines, font_size, color, base_x, base_y);
}

#[allow(clippy::too_many_arguments)]
fn bake_lines_into_mesh(
    mesh: &mut TessellatedColorMesh,
    lines: &[String],
    font_size: f32,
    color: [f32; 4],
    base_x: f32,
    base_y: f32,
) {
    if lines.len() == 1 {
        bake_text_into_mesh(mesh, &lines[0], font_size, color, base_x, base_y);
        return;
    }
    // Multi-line: space lines evenly around base_y.
    // Glyph Y is negated (ascenders go negative, descenders positive),
    // so positive Y offset moves text downward.
    let line_height = font_size * 1.3;
    let total_span = (lines.len() as f32 - 1.0) * line_height;
    let start_y = base_y - total_span * 0.5;
    for (i, line) in lines.iter().enumerate() {
        let y_offset = start_y + i as f32 * line_height;
        bake_text_into_mesh(mesh, line, font_size, color, base_x, y_offset);
    }
}
