#![allow(clippy::unwrap_used)]

use engine_core::color::Color;
use engine_render::shape::{PathCommand, Shape, ShapeVariant};
use glam::Vec2;
use img_to_shape::codegen::{
    ArtMetadata, ExportOptimizationConfig, RepositoryEntry, build_shared_palette,
    decode_shapes_from_compact_data, decode_shapes_from_compact_palette_data,
    encode_shapes_to_compact_data, encode_shapes_to_compact_data_with_shared_palette,
    generate_art_mod, generate_hydrate_module, generate_repository_module,
    optimize_shapes_for_export, shapes_to_art_file, shapes_to_compact_art_file,
    shapes_to_compact_art_file_with_shared_palette, shapes_to_vec_literal,
};

fn default_metadata() -> ArtMetadata<'static> {
    ArtMetadata {
        element: "Solidum",
        aspect: "Solid",
        signature_axes: [0.0; 8],
    }
}

fn triangle_shape(color: Color) -> Shape {
    Shape {
        variant: ShapeVariant::Path {
            commands: vec![
                PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
                PathCommand::LineTo(Vec2::new(10.0, 0.0)),
                PathCommand::LineTo(Vec2::new(5.0, 10.0)),
                PathCommand::Close,
            ],
        },
        color,
    }
}

#[test]
fn when_shapes_empty_then_file_contains_empty_vec() {
    // Arrange
    let shapes: Vec<Shape> = vec![];
    let meta = default_metadata();

    // Act
    let code = shapes_to_art_file(&shapes, &meta, "test_art");

    // Assert
    assert!(code.contains("vec![]"), "expected empty vec:\n{code}");
}

#[test]
fn when_single_shape_then_file_starts_with_use_imports() {
    // Arrange
    let shapes = vec![triangle_shape(Color::RED)];
    let meta = default_metadata();

    // Act
    let code = shapes_to_art_file(&shapes, &meta, "test_art");

    // Assert
    assert!(
        code.contains("use engine_core::color::Color;"),
        "missing Color import:\n{code}"
    );
    assert!(
        code.contains("use engine_render::shape::{PathCommand, Shape, ShapeVariant};"),
        "missing shape imports:\n{code}"
    );
    assert!(
        code.contains("use glam::Vec2;"),
        "missing Vec2 import:\n{code}"
    );
}

#[test]
fn when_single_shape_then_file_contains_pub_fn_returning_vec_shape() {
    // Arrange
    let shapes = vec![triangle_shape(Color::RED)];
    let meta = default_metadata();

    // Act
    let code = shapes_to_art_file(&shapes, &meta, "test_art");

    // Assert
    assert!(
        code.contains("pub fn test_art() -> Vec<Shape>"),
        "missing fn sig:\n{code}"
    );
}

#[test]
fn when_single_path_shape_then_file_contains_path_commands() {
    // Arrange
    let shapes = vec![triangle_shape(Color::RED)];
    let meta = default_metadata();

    // Act
    let code = shapes_to_art_file(&shapes, &meta, "test_art");

    // Assert
    assert!(code.contains("MoveTo"), "missing MoveTo:\n{code}");
    assert!(code.contains("LineTo"), "missing LineTo:\n{code}");
    assert!(code.contains("Close"), "missing Close:\n{code}");
}

#[test]
fn when_red_shape_then_file_contains_color_components() {
    // Arrange
    let shapes = vec![triangle_shape(Color::RED)];
    let meta = default_metadata();

    // Act
    let code = shapes_to_art_file(&shapes, &meta, "test_art");

    // Assert
    assert!(code.contains("Color::new(1"), "missing red color:\n{code}");
}

#[test]
fn when_two_shapes_then_file_contains_both() {
    // Arrange
    let shapes = vec![triangle_shape(Color::RED), triangle_shape(Color::BLUE)];
    let meta = default_metadata();

    // Act
    let code = shapes_to_art_file(&shapes, &meta, "test_art");

    // Assert
    let count = code.matches("Shape {").count();
    assert_eq!(count, 2, "expected 2 shapes:\n{code}");
}

#[test]
fn when_metadata_has_element_then_file_contains_it() {
    // Arrange
    let shapes = vec![triangle_shape(Color::RED)];
    let meta = ArtMetadata {
        element: "Febris",
        aspect: "Heat",
        signature_axes: [0.5, -0.3, 0.0, 0.8, 0.0, 0.0, 0.0, 0.0],
    };

    // Act
    let code = shapes_to_art_file(&shapes, &meta, "test_art");

    // Assert
    assert!(code.contains("Febris"), "missing element:\n{code}");
    assert!(code.contains("Heat"), "missing aspect:\n{code}");
    assert!(code.contains("0.5"), "missing axis value:\n{code}");
}

#[test]
fn when_fn_name_provided_then_file_uses_it() {
    // Arrange
    let shapes = vec![triangle_shape(Color::RED)];
    let meta = default_metadata();

    // Act
    let code = shapes_to_art_file(&shapes, &meta, "solidum_flame");

    // Assert
    assert!(
        code.contains("pub fn solidum_flame"),
        "missing fn name:\n{code}"
    );
}

#[test]
fn when_fn_name_empty_then_file_uses_fallback() {
    // Arrange
    let shapes = vec![triangle_shape(Color::RED)];
    let meta = default_metadata();

    // Act
    let code = shapes_to_art_file(&shapes, &meta, "");

    // Assert
    assert!(
        code.contains("pub fn art_mesh"),
        "missing fallback name:\n{code}"
    );
}

#[test]
fn when_single_shape_then_file_has_balanced_braces_and_brackets() {
    // Arrange
    let shapes = vec![triangle_shape(Color::RED)];
    let meta = default_metadata();

    // Act
    let code = shapes_to_art_file(&shapes, &meta, "test_art");

    // Assert
    let open_braces = code.matches('{').count();
    let close_braces = code.matches('}').count();
    let open_brackets = code.matches('[').count();
    let close_brackets = code.matches(']').count();
    assert_eq!(open_braces, close_braces, "unbalanced braces");
    assert_eq!(open_brackets, close_brackets, "unbalanced brackets");
}

#[test]
fn when_cubic_command_then_file_contains_control_points() {
    // Arrange
    let shapes = vec![Shape {
        variant: ShapeVariant::Path {
            commands: vec![
                PathCommand::MoveTo(Vec2::ZERO),
                PathCommand::CubicTo {
                    control1: Vec2::new(1.0, 2.0),
                    control2: Vec2::new(3.0, 4.0),
                    to: Vec2::new(5.0, 0.0),
                },
                PathCommand::Close,
            ],
        },
        color: Color::WHITE,
    }];
    let meta = default_metadata();

    // Act
    let code = shapes_to_art_file(&shapes, &meta, "test_art");

    // Assert
    assert!(code.contains("CubicTo"), "missing CubicTo:\n{code}");
    assert!(code.contains("control1"), "missing control1:\n{code}");
    assert!(code.contains("control2"), "missing control2:\n{code}");
}

// --- Compact encoding tests ---

#[test]
fn when_compact_encoding_then_output_contains_data_array_and_hydrate_import() {
    // Arrange
    let shapes = vec![triangle_shape(Color::RED)];
    let meta = default_metadata();

    // Act
    let code = shapes_to_compact_art_file(&shapes, &meta, "test_art").unwrap();

    // Assert
    assert!(
        code.contains("const DATA: &[i16]"),
        "missing DATA array:\n{code}"
    );
    assert!(
        code.contains("const COLORS: &[u8]"),
        "missing COLORS array:\n{code}"
    );
    assert!(
        code.contains("use super::hydrate::hydrate_shapes_compact"),
        "missing hydrate import:\n{code}"
    );
    assert!(
        code.contains("hydrate_shapes_compact(COLORS, DATA)"),
        "missing hydrate call:\n{code}"
    );
    assert!(code.contains("pub fn test_art"), "missing pub fn:\n{code}");
}

#[test]
fn when_compact_encoding_then_data_contains_shape_tag() {
    // Arrange
    let shapes = vec![triangle_shape(Color::RED)];

    // Act
    let compact = encode_shapes_to_compact_data(&shapes).unwrap();

    // Assert
    assert_eq!(compact.colors[0], u8::MAX, "r should be 255");
    assert_eq!(compact.colors[1], 0, "g should be 0");
    assert_eq!(compact.colors[2], 0, "b should be 0");
    assert_eq!(compact.data[0], 4, "first tag should be 4");
    assert_eq!(compact.data[1], 0, "start x should be 0");
    assert_eq!(compact.data[2], 0, "start y should be 0");
}

#[test]
fn when_compact_encoding_roundtripped_then_shapes_match() {
    // Arrange
    let shapes = vec![triangle_shape(Color::RED), triangle_shape(Color::BLUE)];

    // Act
    let compact = encode_shapes_to_compact_data(&shapes).unwrap();
    let decoded = decode_shapes_from_compact_data(&compact.colors, &compact.data);

    // Assert
    assert_eq!(decoded, shapes);
}

#[test]
fn when_shared_palette_compact_encoding_then_shapes_roundtrip_via_indexes() {
    // Arrange
    let shapes = vec![triangle_shape(Color::RED), triangle_shape(Color::BLUE)];
    let shape_sets = vec![shapes.as_slice()];
    let shared_palette = build_shared_palette(&shape_sets, 2);

    // Act
    let compact =
        encode_shapes_to_compact_data_with_shared_palette(&shapes, &shared_palette).unwrap();
    let decoded = decode_shapes_from_compact_palette_data(
        &shared_palette,
        &compact.color_indexes,
        &compact.data,
    );

    // Assert
    assert_eq!(decoded, shapes);
}

#[test]
fn when_shared_palette_compact_art_generated_then_output_uses_color_indexes() {
    // Arrange
    let shapes = vec![triangle_shape(Color::RED), triangle_shape(Color::BLUE)];
    let meta = default_metadata();
    let shape_sets = vec![shapes.as_slice()];
    let shared_palette = build_shared_palette(&shape_sets, 2);

    // Act
    let code =
        shapes_to_compact_art_file_with_shared_palette(&shapes, &meta, "test_art", &shared_palette)
            .unwrap();

    // Assert
    assert!(
        code.contains("const COLOR_INDEXES: &[u8]"),
        "missing COLOR_INDEXES array:\n{code}"
    );
    assert!(
        code.contains("use super::hydrate::hydrate_shapes_compact_indexed"),
        "missing indexed hydrate import:\n{code}"
    );
    assert!(
        code.contains("hydrate_shapes_compact_indexed(COLOR_INDEXES, DATA)"),
        "missing indexed hydrate call:\n{code}"
    );
}

#[test]
fn when_compact_encoding_overflows_i16_then_encoding_fails() {
    // Arrange
    let shapes = vec![Shape {
        variant: ShapeVariant::Path {
            commands: vec![
                PathCommand::MoveTo(Vec2::new(400.0, 0.0)),
                PathCommand::LineTo(Vec2::new(401.0, 1.0)),
                PathCommand::Close,
            ],
        },
        color: Color::WHITE,
    }];

    // Act
    let error = encode_shapes_to_compact_data(&shapes).unwrap_err();

    // Assert
    assert!(
        error
            .to_string()
            .contains("compact i16 encoding overflow for shape start x"),
        "unexpected error: {error}"
    );
}

#[test]
fn when_compact_encoding_with_many_commands_then_fewer_lines_than_verbose() {
    // Arrange — shape with 200 LineTo commands (simulates real art output)
    let commands: Vec<PathCommand> = std::iter::once(PathCommand::MoveTo(Vec2::ZERO))
        .chain((1..200).map(|i| PathCommand::LineTo(Vec2::new(i as f32, (i * 2) as f32))))
        .chain(std::iter::once(PathCommand::Close))
        .collect();
    let shapes = vec![Shape {
        variant: ShapeVariant::Path { commands },
        color: Color::RED,
    }];
    let meta = default_metadata();

    // Act
    let verbose = shapes_to_art_file(&shapes, &meta, "test");
    let compact = shapes_to_compact_art_file(&shapes, &meta, "test").unwrap();

    // Assert — compact should have fewer lines once data volume exceeds hydrate fn overhead
    let verbose_lines = verbose.lines().count();
    let compact_lines = compact.lines().count();
    assert!(
        compact_lines < verbose_lines,
        "compact ({compact_lines} lines) should be fewer than verbose ({verbose_lines} lines)"
    );
}

// --- Generated module tests ---

#[test]
fn when_hydrate_module_generated_then_contains_pub_fn_and_imports() {
    // Act
    let code = generate_hydrate_module();

    // Assert
    assert!(
        code.contains("pub fn hydrate_shapes"),
        "missing pub fn:\n{code}"
    );
    assert!(
        code.contains("pub fn hydrate_shapes_compact"),
        "missing compact hydrate fn:\n{code}"
    );
    assert!(
        code.contains("pub fn hydrate_shapes_compact_indexed"),
        "missing indexed hydrate fn:\n{code}"
    );
    assert!(
        code.contains("const SHARED_PALETTE: &[u8]"),
        "missing shared palette const:\n{code}"
    );
    assert!(
        code.contains("use engine_core::color::Color"),
        "missing Color import:\n{code}"
    );
    assert!(
        code.contains("use engine_render::shape"),
        "missing shape import:\n{code}"
    );
}

#[test]
fn when_coordinate_decimals_reduced_then_geometry_rounds_for_preview_export() {
    // Arrange
    let shapes = vec![Shape {
        variant: ShapeVariant::Path {
            commands: vec![
                PathCommand::MoveTo(Vec2::new(0.04, 0.06)),
                PathCommand::LineTo(Vec2::new(10.06, 0.04)),
                PathCommand::CubicTo {
                    control1: Vec2::new(4.44, 3.36),
                    control2: Vec2::new(6.61, 7.78),
                    to: Vec2::new(5.06, 10.04),
                },
                PathCommand::Close,
            ],
        },
        color: Color::WHITE,
    }];

    // Act
    let optimized = optimize_shapes_for_export(
        &shapes,
        &ExportOptimizationConfig {
            coordinate_decimals: 1,
            palette_size: 0,
        },
    );

    // Assert
    let ShapeVariant::Path { commands } = &optimized[0].variant else {
        panic!("expected path shape");
    };
    assert_eq!(commands[0], PathCommand::MoveTo(Vec2::new(0.0, 0.1)));
    assert_eq!(commands[1], PathCommand::LineTo(Vec2::new(10.1, 0.0)));
}

#[test]
fn when_shared_palette_built_then_target_size_respected() {
    // Arrange
    let first = vec![
        triangle_shape(Color::new(1.0, 0.0, 0.0, 1.0)),
        triangle_shape(Color::new(0.9, 0.1, 0.0, 1.0)),
    ];
    let second = vec![triangle_shape(Color::new(0.0, 0.0, 1.0, 1.0))];
    let shape_sets = vec![first.as_slice(), second.as_slice()];

    // Act
    let palette = build_shared_palette(&shape_sets, 2);

    // Assert
    assert_eq!(palette.len(), 2);
}

#[test]
fn when_repository_generated_then_contains_hydrate_all_and_get() {
    // Arrange
    let entries = vec![
        RepositoryEntry {
            fn_name: "armor1",
            element_index: 0,
            aspect_pole: 0,
            signature_axes: [0.0; 8],
        },
        RepositoryEntry {
            fn_name: "sword2",
            element_index: 1,
            aspect_pole: 0,
            signature_axes: [0.0; 8],
        },
    ];

    // Act
    let code = generate_repository_module(&entries);

    // Assert
    assert!(
        code.contains("pub fn hydrate_all"),
        "missing hydrate_all:\n{code}"
    );
    assert!(code.contains("pub fn get"), "missing get:\n{code}");
    assert!(
        code.contains("armor1::armor1()"),
        "missing armor1 call:\n{code}"
    );
    assert!(
        code.contains("sword2::sword2()"),
        "missing sword2 call:\n{code}"
    );
    assert!(
        code.contains("use super::armor1"),
        "missing armor1 import:\n{code}"
    );
}

#[test]
fn when_repository_generated_then_contains_art_entry_constructor() {
    // Arrange
    let entries = vec![RepositoryEntry {
        fn_name: "armor1",
        element_index: 0,
        aspect_pole: 0,
        signature_axes: [0.0; 8],
    }];

    // Act
    let code = generate_repository_module(&entries);

    // Assert
    assert!(
        code.contains("ArtEntry::new("),
        "missing ArtEntry constructor:\n{code}"
    );
    assert!(
        code.contains("Element::Solidum"),
        "missing Element::Solidum:\n{code}"
    );
    assert!(
        code.contains("Aspect::Solid"),
        "missing Aspect::Solid:\n{code}"
    );
    assert!(
        code.contains("CardSignature::new("),
        "missing CardSignature::new:\n{code}"
    );
}

#[test]
fn when_repository_generated_for_two_different_elements_then_both_variants_present() {
    // Arrange
    let entries = vec![
        RepositoryEntry {
            fn_name: "armor1",
            element_index: 0,
            aspect_pole: 0,
            signature_axes: [0.0; 8],
        },
        RepositoryEntry {
            fn_name: "fire_card",
            element_index: 1,
            aspect_pole: 0,
            signature_axes: [0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        },
    ];

    // Act
    let code = generate_repository_module(&entries);

    // Assert
    assert!(
        code.contains("Element::Solidum"),
        "missing Solidum:\n{code}"
    );
    assert!(code.contains("Element::Febris"), "missing Febris:\n{code}");
    assert!(code.contains("Aspect::Solid"), "missing Solid:\n{code}");
    assert!(code.contains("Aspect::Heat"), "missing Heat:\n{code}");
}

#[test]
fn when_repository_generated_then_contains_all_query_method_signatures() {
    // Arrange
    let entries = vec![RepositoryEntry {
        fn_name: "armor1",
        element_index: 0,
        aspect_pole: 0,
        signature_axes: [0.0; 8],
    }];

    // Act
    let code = generate_repository_module(&entries);

    // Assert
    assert!(
        code.contains("pub fn get_entry"),
        "missing get_entry:\n{code}"
    );
    assert!(
        code.contains("pub fn by_element"),
        "missing by_element:\n{code}"
    );
    assert!(
        code.contains("pub fn by_aspect"),
        "missing by_aspect:\n{code}"
    );
    assert!(
        code.contains("pub fn closest_to"),
        "missing closest_to:\n{code}"
    );
}

#[test]
fn when_art_mod_generated_then_declares_all_modules() {
    // Arrange
    let names = vec!["armor1", "sword2"];

    // Act
    let code = generate_art_mod(&names);

    // Assert — only art modules, no support modules or utility code.
    assert!(
        code.contains("pub mod armor1;"),
        "missing armor1 mod:\n{code}"
    );
    assert!(
        code.contains("pub mod sword2;"),
        "missing sword2 mod:\n{code}"
    );
    assert!(
        !code.contains("pub mod hydrate;"),
        "should not declare hydrate:\n{code}"
    );
    assert!(
        !code.contains("pub mod repository;"),
        "should not declare repository:\n{code}"
    );
}

// --- Vec literal tests ---

#[test]
fn when_vec_literal_empty_shapes_then_returns_empty_vec() {
    // Arrange
    let shapes: Vec<Shape> = vec![];

    // Act
    let code = shapes_to_vec_literal(&shapes);

    // Assert
    assert_eq!(code, "vec![]");
}

#[test]
fn when_vec_literal_single_shape_then_contains_shape_struct() {
    // Arrange
    let shapes = vec![triangle_shape(Color::RED)];

    // Act
    let code = shapes_to_vec_literal(&shapes);

    // Assert
    assert!(code.starts_with("vec!["), "should start with vec![: {code}");
    assert!(code.ends_with(']'), "should end with ]: {code}");
    assert!(code.contains("Shape {"), "missing Shape:\n{code}");
    assert!(code.contains("MoveTo"), "missing MoveTo:\n{code}");
    assert!(code.contains("Close"), "missing Close:\n{code}");
}

#[test]
fn when_vec_literal_multiple_shapes_then_contains_all() {
    // Arrange
    let shapes = vec![triangle_shape(Color::RED), triangle_shape(Color::BLUE)];

    // Act
    let code = shapes_to_vec_literal(&shapes);

    // Assert
    let count = code.matches("Shape {").count();
    assert_eq!(count, 2, "expected 2 shapes:\n{code}");
}

// --- Signature test helpers ---

/// Extract the axes string from the first `CardSignature::new([...])` literal in `code`.
/// Returns the comma-separated content between `[` and `]`, with surrounding whitespace
/// stripped. Panics with a clear message if the literal is not present.
#[allow(dead_code)]
fn extract_first_signature_axes(code: &str) -> &str {
    let marker = "CardSignature::new([";
    let sig_start = code
        .find(marker)
        .expect("CardSignature::new not found in generated output");
    let after = &code[sig_start + marker.len()..];
    let sig_end = after
        .find("])")
        .expect("closing ]) not found after CardSignature::new([");
    after[..sig_end].trim()
}

// --- Signature determinism tests ---

/// @doc: `generate_repository_module` must be a pure function: calling it twice
/// with the same `RepositoryEntry` must produce byte-for-byte identical output.
/// Without this guarantee, re-running the codegen tool would produce spurious diffs
/// in the committed `repository.rs`, causing unnecessary churn in code review and
/// making it impossible to detect whether a real change has occurred by diffing.
/// The seeded-from-name path is the only code path that touches internal state
/// (the hash), so it is the critical path to guard.
#[test]
fn when_zero_axes_then_repeated_codegen_emits_identical_signature_literal() {
    // Arrange
    let make_entry = || RepositoryEntry {
        fn_name: "armor1",
        element_index: 0,
        aspect_pole: 0,
        signature_axes: [0.0; 8],
    };

    // Act
    let first = generate_repository_module(&[make_entry()]);
    let second = generate_repository_module(&[make_entry()]);

    // Assert
    assert_eq!(
        first, second,
        "generate_repository_module must be deterministic: \
         re-running codegen with the same entry must not change the emitted signature"
    );
}

/// @doc: When `signature_axes` are all zero, `generate_repository_module` seeds
/// the signature from the art name so that each card gets a unique position in the
/// 8-dimensional signature space. A degenerate signature where all 8 axes collapse
/// to the same rounded value (e.g. `[0.37, 0.37, ...]`) provides zero discriminating
/// power — `closest_to` queries would return arbitrary results among cards with the
/// same degenerate signature. The per-axis hash seed must produce enough spread that
/// after 2-decimal rounding no two axes share the same value.
#[test]
fn when_zero_axes_single_char_name_then_seeded_signature_axes_are_all_distinct() {
    // Arrange — single-character name exposes weak per-axis differentiation in the
    // FNV-inspired hash (the axis seed offset is too small for 1-byte inputs)
    let entry = RepositoryEntry {
        fn_name: "a",
        element_index: 0,
        aspect_pole: 0,
        signature_axes: [0.0; 8],
    };

    // Act
    let code = generate_repository_module(&[entry]);

    // Assert — extract the CardSignature literal and verify all 8 values differ
    let axes_str = extract_first_signature_axes(&code);
    let values: Vec<&str> = axes_str.split(',').map(str::trim).collect();

    assert_eq!(values.len(), 8, "expected 8 axis values, got: {values:?}");

    let mut seen = std::collections::BTreeSet::new();
    for v in &values {
        assert!(
            seen.insert(*v),
            "duplicate axis value '{v}' in seeded signature for name 'a' — \
             hash produces degenerate signature: {axes_str}"
        );
    }
}

/// @doc: The `fmt_f32` helper rounds axis values to 2 decimal places before
/// emitting them into the `CardSignature::new([...])` literal. When a seeded axis
/// is a tiny negative value (e.g. -0.0017), the rounding produces negative zero
/// (`-0.0` in Rust's f32 Display). While `-0.0` is valid Rust, it introduces
/// invisible semantic ambiguity — `-0.0_f32 == 0.0_f32` is true at runtime but
/// `-0.0` and `0.0` differ in `to_bits()` comparisons and serialization, meaning
/// the emitted source could silently diverge from its intent on re-parse.
/// Generated code must never contain `-0.0` as a signature axis literal.
#[test]
fn when_zero_axes_name_with_near_zero_axis_then_emitted_literal_contains_no_negative_zero() {
    // Arrange — "ad" seeds one axis to ~-0.0017, which rounds to -0.0 without canonicalization
    let entry = RepositoryEntry {
        fn_name: "ad",
        element_index: 0,
        aspect_pole: 0,
        signature_axes: [0.0; 8],
    };

    // Act
    let code = generate_repository_module(&[entry]);

    // Assert — extract each axis token and check none equals the string "-0.0"
    let axes_str = extract_first_signature_axes(&code);
    for token in axes_str.split(',').map(str::trim) {
        assert!(
            token != "-0.0",
            "generated signature literal contains the negative-zero token '-0.0' — \
             use '0.0' instead to avoid ambiguity in serialization and bit-level comparisons:\n\
             CardSignature::new([{axes_str}])"
        );
    }
}

// --- Signature uniqueness tests ---

/// @doc: `CardSignature.distance_to` computes Euclidean distance across all 8 axes.
/// If any axis exceeds [-1.0, 1.0] the distance metric becomes asymmetric: a single
/// out-of-range axis can dominate the sum and make unrelated cards appear closer than
/// related ones, breaking `closest_to` matching. The seeded hash must therefore clamp
/// or scale its output to the unit hypercube before the axes are formatted and emitted.
/// This test is a regression guard — the splitmix64 implementation already satisfies
/// this constraint; it ensures no future refactor silently breaks the invariant.
#[test]
fn when_zero_axes_then_seeded_signature_axes_are_in_range() {
    // Arrange
    let entry = RepositoryEntry {
        fn_name: "some_art_name",
        element_index: 0,
        aspect_pole: 0,
        signature_axes: [0.0; 8],
    };

    // Act
    let code = generate_repository_module(&[entry]);

    // Assert
    let axes_str = extract_first_signature_axes(&code);
    let values: Vec<f32> = axes_str
        .split(',')
        .map(|s| {
            s.trim()
                .parse::<f32>()
                .unwrap_or_else(|e| panic!("failed to parse axis '{s}': {e}"))
        })
        .collect();

    assert_eq!(values.len(), 8, "expected 8 axis values, got: {values:?}");
    for v in &values {
        assert!(
            (-1.0..=1.0).contains(v),
            "seeded axis {v} is outside [-1.0, 1.0] — \
             unbounded axes skew Euclidean distance and break closest_to matching"
        );
    }
}

/// @doc: The art repository maps each piece to a unique point in 8-dimensional
/// signature space. If two arts share the same `CardSignature` literal, they become
/// indistinguishable to `closest_to`: a query positioned equidistant from both will
/// return one arbitrarily, and re-running the query may return the other. Uniqueness
/// is therefore a correctness requirement, not just a nice-to-have.
/// This test is a regression guard — the splitmix64 implementation already satisfies
/// this constraint; it ensures no future refactor accidentally collapses the seeds.
#[test]
fn when_two_different_names_then_signatures_differ() {
    // Arrange
    let entries = vec![
        RepositoryEntry {
            fn_name: "fire_blade",
            element_index: 0,
            aspect_pole: 0,
            signature_axes: [0.0; 8],
        },
        RepositoryEntry {
            fn_name: "ice_shield",
            element_index: 0,
            aspect_pole: 0,
            signature_axes: [0.0; 8],
        },
    ];

    // Act
    let code = generate_repository_module(&entries);

    // Assert — find each entry's insert block and extract its CardSignature literal
    fn extract_sig_for_name<'a>(code: &'a str, name: &str) -> &'a str {
        // Locate the insert call for this entry: `self.cache.insert("name", ...`
        let marker = format!("insert(\"{name}\"");
        let insert_pos = code
            .find(&marker)
            .unwrap_or_else(|| panic!("insert block for '{name}' not found in output"));
        let from_insert = &code[insert_pos..];
        let sig_start = from_insert
            .find("CardSignature::new([")
            .unwrap_or_else(|| panic!("CardSignature for '{name}' not found"));
        let from_sig = &from_insert[sig_start + "CardSignature::new([".len()..];
        let sig_end = from_sig
            .find("])")
            .unwrap_or_else(|| panic!("closing ]) for '{name}' not found"));
        &from_sig[..sig_end]
    }

    let fire_axes = extract_sig_for_name(&code, "fire_blade");
    let ice_axes = extract_sig_for_name(&code, "ice_shield");

    assert_ne!(
        fire_axes, ice_axes,
        "\"fire_blade\" and \"ice_shield\" produced the same CardSignature literal \
         '{fire_axes}' — distinct names must map to distinct points in signature space"
    );
}

/// @doc: During production codegen for the barbarian icon set, 89 of 130 entries
/// shared an identical `CardSignature` because the old FNV hash produced too little
/// spread across the 8 axes after 2-decimal rounding. These 89 collisions made the
/// affected cards indistinguishable to `closest_to` queries. The splitmix64-based
/// replacement was introduced specifically to eliminate this collision class.
/// This test reproduces the exact production input (names `barbarian_icons_01_t`
/// through `barbarian_icons_130_t`) and asserts that all 130 signatures are unique,
/// making it the canonical regression guard for the FNV collision bug.
#[test]
fn when_130_similar_names_then_all_signatures_are_unique() {
    // Arrange
    let names: Vec<String> = (1..=130)
        .map(|i| format!("barbarian_icons_{i:02}_t"))
        .collect();
    let entries: Vec<RepositoryEntry<'_>> = names
        .iter()
        .map(|n| RepositoryEntry {
            fn_name: n.as_str(),
            element_index: 0,
            aspect_pole: 0,
            signature_axes: [0.0; 8],
        })
        .collect();

    // Act
    let code = generate_repository_module(&entries);

    // Assert — collect every CardSignature literal and verify all 130 are distinct
    let mut signatures: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut search = code.as_str();
    while let Some(start) = search.find("CardSignature::new([") {
        let after = &search[start + "CardSignature::new([".len()..];
        let end = after.find("])").expect("closing ]) not found");
        let axes_str = after[..end].trim().to_owned();
        signatures.insert(axes_str);
        search = &after[end + 2..];
    }

    assert_eq!(
        signatures.len(),
        130,
        "expected 130 unique CardSignature literals but got {} — \
         hash collision detected among barbarian_icons entries",
        signatures.len()
    );
}
