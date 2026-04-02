#![allow(clippy::unwrap_used)]

use engine_core::color::Color;
use engine_render::shape::{PathCommand, Shape, ShapeVariant};
use glam::Vec2;
use img_to_shape::codegen::{
    ArtMetadata, RepositoryEntry, encode_shapes_to_floats, generate_art_mod,
    generate_hydrate_module, generate_repository_module, shapes_to_art_file,
    shapes_to_compact_art_file, shapes_to_vec_literal,
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
    let code = shapes_to_compact_art_file(&shapes, &meta, "test_art");

    // Assert
    assert!(
        code.contains("const DATA: &[f32]"),
        "missing DATA array:\n{code}"
    );
    assert!(
        code.contains("use super::hydrate::hydrate_shapes"),
        "missing hydrate import:\n{code}"
    );
    assert!(
        code.contains("hydrate_shapes(DATA)"),
        "missing hydrate call:\n{code}"
    );
    assert!(code.contains("pub fn test_art"), "missing pub fn:\n{code}");
}

#[test]
fn when_compact_encoding_then_data_contains_shape_tag() {
    // Arrange
    let shapes = vec![triangle_shape(Color::RED)];

    // Act
    let data = encode_shapes_to_floats(&shapes);

    // Assert — first float should be TAG_SHAPE (4.0)
    assert!(
        (data[0] - 4.0).abs() < f32::EPSILON,
        "first tag should be 4.0 (shape)"
    );
    // Then 4 color floats (r=1.0, g=0.0, b=0.0, a=1.0)
    assert!((data[1] - 1.0).abs() < f32::EPSILON, "r should be 1.0");
    assert!((data[2]).abs() < f32::EPSILON, "g should be 0.0");
    assert!((data[3]).abs() < f32::EPSILON, "b should be 0.0");
    assert!((data[4] - 1.0).abs() < f32::EPSILON, "a should be 1.0");
}

#[test]
fn when_compact_encoding_roundtripped_then_shape_count_matches() {
    // Arrange — encode then paste the data into a quick decode check
    let shapes = vec![triangle_shape(Color::RED), triangle_shape(Color::BLUE)];

    // Act
    let data = encode_shapes_to_floats(&shapes);

    // Assert — count shape tags (4.0 values at positions where a tag is expected)
    let shape_tags = data
        .iter()
        .filter(|&&v| (v - 4.0).abs() < f32::EPSILON)
        .count();
    // There should be at least 2 shape tags (could be more if a float happens to be 4.0,
    // but with our test colors that won't happen)
    assert_eq!(shape_tags, 2, "expected 2 shape tags in data");
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
    let compact = shapes_to_compact_art_file(&shapes, &meta, "test");

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
        code.contains("use engine_core::color::Color"),
        "missing Color import:\n{code}"
    );
    assert!(
        code.contains("use engine_render::shape"),
        "missing shape import:\n{code}"
    );
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
