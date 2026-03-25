use std::fmt::Write;

use engine_core::color::Color;
use engine_render::shape::{PathCommand, Shape, ShapeVariant};

/// Metadata describing which card signature affinity this art represents.
pub struct ArtMetadata<'a> {
    pub element: &'a str,
    pub aspect: &'a str,
    pub signature_axes: [f32; 8],
}

/// Generate a complete `.rs` source file containing a `pub fn` that returns
/// `Vec<Shape>` — compact bezier path data suitable for runtime tessellation.
///
/// The output file includes `use` imports, signature metadata as doc comments,
/// and a function returning the shape vector.
pub fn shapes_to_art_file(shapes: &[Shape], metadata: &ArtMetadata<'_>, fn_name: &str) -> String {
    let name = if fn_name.is_empty() {
        "art_mesh"
    } else {
        fn_name
    };

    let mut out = String::new();

    // Imports
    out.push_str(
        "use engine_core::color::Color;\n\
         use engine_render::shape::{PathCommand, Shape, ShapeVariant};\n\
         use glam::Vec2;\n\n",
    );

    // Metadata doc comment
    let _ = writeln!(out, "/// Element: {}", metadata.element);
    let _ = writeln!(out, "/// Aspect: {}", metadata.aspect);
    let _ = write!(out, "/// Signature: [");
    for (i, &v) in metadata.signature_axes.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        let _ = write!(out, "{}", fmt_f32(v));
    }
    out.push_str("]\n");

    // Function
    let _ = writeln!(out, "pub fn {name}() -> Vec<Shape> {{");

    if shapes.is_empty() {
        out.push_str("    vec![]\n");
    } else {
        out.push_str("    vec![\n");
        for (i, shape) in shapes.iter().enumerate() {
            if i > 0 {
                out.push_str(",\n");
            }
            write_shape(&mut out, shape);
        }
        out.push_str("\n    ]\n");
    }
    out.push_str("}\n");

    out
}

fn fmt_f32(v: f32) -> String {
    // Round to 2 decimal places to keep generated code compact.
    let rounded = (v * 100.0).round() / 100.0;
    let s = format!("{rounded}");
    if s.contains('.') {
        s
    } else {
        format!("{rounded}.0")
    }
}

fn write_shape(out: &mut String, shape: &Shape) {
    out.push_str("        Shape {\n");
    out.push_str("            variant: ");
    write_variant(out, &shape.variant);
    out.push_str(",\n            color: ");
    write_color(out, &shape.color);
    out.push_str(",\n        }");
}

fn write_variant(out: &mut String, variant: &ShapeVariant) {
    match variant {
        ShapeVariant::Circle { radius } => {
            let _ = write!(out, "ShapeVariant::Circle {{ radius: {radius:.1} }}");
        }
        ShapeVariant::Polygon { points } => {
            out.push_str("ShapeVariant::Polygon {\n                points: vec![\n");
            for pt in points {
                let _ = writeln!(
                    out,
                    "                    Vec2::new({}, {}),",
                    fmt_f32(pt.x),
                    fmt_f32(pt.y)
                );
            }
            out.push_str("                ],\n            }");
        }
        ShapeVariant::Path { commands } => {
            out.push_str("ShapeVariant::Path {\n                commands: vec![\n");
            for cmd in commands {
                out.push_str("                    ");
                write_command(out, cmd);
                out.push_str(",\n");
            }
            out.push_str("                ],\n            }");
        }
    }
}

fn write_command(out: &mut String, cmd: &PathCommand) {
    match cmd {
        PathCommand::MoveTo(v) => {
            let _ = write!(
                out,
                "PathCommand::MoveTo(Vec2::new({}, {}))",
                fmt_f32(v.x),
                fmt_f32(v.y)
            );
        }
        PathCommand::LineTo(v) => {
            let _ = write!(
                out,
                "PathCommand::LineTo(Vec2::new({}, {}))",
                fmt_f32(v.x),
                fmt_f32(v.y)
            );
        }
        PathCommand::QuadraticTo { control, to } => {
            let _ = write!(
                out,
                "PathCommand::QuadraticTo {{ control: Vec2::new({}, {}), to: Vec2::new({}, {}) }}",
                fmt_f32(control.x),
                fmt_f32(control.y),
                fmt_f32(to.x),
                fmt_f32(to.y)
            );
        }
        PathCommand::CubicTo {
            control1,
            control2,
            to,
        } => {
            let _ = write!(
                out,
                "PathCommand::CubicTo {{ control1: Vec2::new({}, {}), control2: Vec2::new({}, {}), to: Vec2::new({}, {}) }}",
                fmt_f32(control1.x),
                fmt_f32(control1.y),
                fmt_f32(control2.x),
                fmt_f32(control2.y),
                fmt_f32(to.x),
                fmt_f32(to.y)
            );
        }
        PathCommand::Close => {
            out.push_str("PathCommand::Close");
        }
        PathCommand::Reverse => {
            out.push_str("PathCommand::Reverse");
        }
    }
}

fn write_color(out: &mut String, color: &Color) {
    let _ = write!(
        out,
        "Color::new({}, {}, {}, {})",
        fmt_f32(color.r),
        fmt_f32(color.g),
        fmt_f32(color.b),
        fmt_f32(color.a)
    );
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use glam::Vec2;

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
}
