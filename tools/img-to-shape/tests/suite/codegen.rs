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
