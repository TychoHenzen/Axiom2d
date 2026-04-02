#![allow(clippy::unwrap_used, clippy::float_cmp)]

use engine_render::font::{
    FONT_BYTES, GlyphCache, bake_text_into_mesh, layout_text, measure_text, outline_glyph,
    render_text_glyphs, tessellate_glyph, wrap_text,
};
use engine_render::shape::{PathCommand, TessellatedColorMesh};
use engine_render::testing::{ShapeCallLog, SpyRenderer};

#[test]
fn when_parsing_embedded_font_then_valid_face_with_ascii_glyphs() {
    // Arrange / Act
    let face = ttf_parser::Face::parse(FONT_BYTES, 0).unwrap();

    // Assert
    assert!(face.number_of_glyphs() > 0);
    assert!(face.units_per_em() > 0);
    assert!(face.glyph_index('A').is_some());
}

/// @doc: Glyph outlining must produce path commands -- empty outline fails tessellation and renders nothing
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

/// @doc: Font size must linearly scale outline coordinates -- wrong scale causes text to render at incorrect size
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

/// @doc: Tessellation must produce non-empty vertex/index buffers -- empty mesh fails GPU submission
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

/// @doc: All tessellation indices must reference valid vertices -- out-of-bounds indices crash GPU or render garbage
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

/// @doc: Empty glyph outlines must tessellate to empty meshes -- prevents spurious GPU work and aliasing artifacts
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

/// @doc: Glyph cache returns same mesh on repeated queries -- caching eliminates redundant tessellation
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

/// @doc: Glyph cache key includes font size -- different sizes create separate entries to preserve fidelity
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

/// @doc: First glyph `x_offset` must be zero -- non-zero offset shifts text baseline and breaks alignment
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

/// @doc: Glyph advance width must determine next glyph position -- wrong advance causes overlapping or gapped text
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

/// @doc: Glyph offsets must strictly increase left-to-right -- non-monotonic layout causes character overlap
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

/// @doc: Space glyph advance must be respected -- ignoring space width collapses words into each other
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

fn make_spy() -> (SpyRenderer, ShapeCallLog) {
    let log = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let shape_calls: ShapeCallLog = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(log).with_shape_capture(shape_calls.clone());
    (spy, shape_calls)
}

/// @doc: Single character must produce exactly one `draw_shape` call -- renderer invocation must match character count
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

/// @doc: Character count must match `draw_shape` call count -- missing calls mean unrendered glyphs
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

/// @doc: Space character must not produce `draw_shape` calls -- rendering spaces wastes GPU work and is invisible
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

/// @doc: Glyph model matrices must translate rightward for subsequent characters -- wrong translation causes overlap or reversal
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

/// @doc: Text color must propagate to all `draw_shape` calls -- color mismatch renders wrong or monochrome text
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

/// @doc: Short text within width must not wrap -- unnecessary wrapping breaks layout calculations
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

/// @doc: Text exceeding `max_width` must split at word boundaries -- unbroken overflow causes render clipping
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

/// @doc: Multiple wraps must handle incrementally narrow widths -- wrapping logic must not deadlock or drop words
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

/// @doc: Empty string must produce single empty line -- null/missing output breaks iteration and layout
#[test]
fn when_wrap_text_empty_then_single_empty_line() {
    // Act
    let lines = wrap_text("", 16.0, 100.0);

    // Assert
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], "");
}

/// @doc: Baking single character must produce mesh vertices and propagate color -- empty mesh fails render
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

/// @doc: Baked glyph vertices must extend horizontally with character count -- insufficient extent suggests missing glyphs
#[test]
fn when_bake_text_two_chars_then_second_glyph_vertices_offset_right() {
    // Arrange
    let mut mesh = TessellatedColorMesh::new();
    let color = [1.0, 1.0, 1.0, 1.0];

    // Act
    bake_text_into_mesh(&mut mesh, "AB", 32.0, color, 0.0, 0.0);

    // Assert -- vertices should span a wider range than single-char
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

/// @doc: Wrapping never splits words mid-word -- overflow words placed alone on a line
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
