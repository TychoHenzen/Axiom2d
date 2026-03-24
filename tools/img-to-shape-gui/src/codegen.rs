use engine_core::color::Color;
use engine_render::shape::{PathCommand, Shape, ShapeVariant};
use std::fmt::Write;

/// Generate Rust source code that constructs a `Vec<Shape>`.
pub fn shapes_to_rust_code(shapes: &[Shape]) -> String {
    if shapes.is_empty() {
        return "vec![]".to_string();
    }

    let mut out = String::from("vec![\n");
    for (i, shape) in shapes.iter().enumerate() {
        if i > 0 {
            out.push_str(",\n");
        }
        write_shape(&mut out, shape);
    }
    out.push_str("\n]");
    out
}

fn write_shape(out: &mut String, shape: &Shape) {
    out.push_str("    Shape {\n");
    out.push_str("        variant: ");
    write_variant(out, &shape.variant);
    out.push_str(",\n        color: ");
    write_color(out, &shape.color);
    out.push_str(",\n    }");
}

fn write_variant(out: &mut String, variant: &ShapeVariant) {
    match variant {
        ShapeVariant::Circle { radius } => {
            let _ = write!(out, "ShapeVariant::Circle {{ radius: {radius:.1} }}");
        }
        ShapeVariant::Polygon { points } => {
            out.push_str("ShapeVariant::Polygon {\n            points: vec![\n");
            for pt in points {
                let _ = writeln!(out, "                Vec2::new({}, {}),", pt.x, pt.y);
            }
            out.push_str("            ],\n        }");
        }
        ShapeVariant::Path { commands } => {
            out.push_str("ShapeVariant::Path {\n            commands: vec![\n");
            for cmd in commands {
                out.push_str("                ");
                write_command(out, cmd);
                out.push_str(",\n");
            }
            out.push_str("            ],\n        }");
        }
    }
}

fn write_command(out: &mut String, cmd: &PathCommand) {
    match cmd {
        PathCommand::MoveTo(v) => {
            let _ = write!(out, "PathCommand::MoveTo(Vec2::new({}, {}))", v.x, v.y);
        }
        PathCommand::LineTo(v) => {
            let _ = write!(out, "PathCommand::LineTo(Vec2::new({}, {}))", v.x, v.y);
        }
        PathCommand::QuadraticTo { control, to } => {
            let _ = write!(
                out,
                "PathCommand::QuadraticTo {{ control: Vec2::new({}, {}), to: Vec2::new({}, {}) }}",
                control.x, control.y, to.x, to.y
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
                control1.x, control1.y, control2.x, control2.y, to.x, to.y
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
        color.r, color.g, color.b, color.a
    );
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use glam::Vec2;

    // TC001
    #[test]
    fn when_shape_list_is_empty_then_codegen_returns_empty_vec_literal() {
        // Arrange
        let shapes: Vec<Shape> = vec![];

        // Act
        let code = shapes_to_rust_code(&shapes);

        // Assert
        assert_eq!(code, "vec![]");
    }

    // TC002
    #[test]
    fn when_single_circle_shape_then_codegen_contains_circle_radius_literal() {
        // Arrange
        let shapes = vec![Shape {
            variant: ShapeVariant::Circle { radius: 25.0 },
            color: Color::RED,
        }];

        // Act
        let code = shapes_to_rust_code(&shapes);

        // Assert
        assert!(code.contains("Circle"), "missing Circle: {code}");
        assert!(code.contains("25"), "missing radius 25: {code}");
        assert!(code.contains('1'), "missing red component: {code}");
    }

    // TC003
    #[test]
    fn when_single_polygon_shape_then_codegen_contains_all_point_coordinates() {
        // Arrange
        let shapes = vec![Shape {
            variant: ShapeVariant::Polygon {
                points: vec![Vec2::new(1.0, 2.0), Vec2::new(3.0, 4.0)],
            },
            color: Color::WHITE,
        }];

        // Act
        let code = shapes_to_rust_code(&shapes);

        // Assert
        assert!(code.contains("Polygon"), "missing Polygon: {code}");
        for val in ["1", "2", "3", "4"] {
            assert!(code.contains(val), "missing coordinate {val}: {code}");
        }
    }

    // TC004
    #[test]
    fn when_path_shape_with_moveto_and_lineto_then_codegen_contains_both_commands() {
        // Arrange
        let shapes = vec![Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::ZERO),
                    PathCommand::LineTo(Vec2::new(10.0, 0.0)),
                    PathCommand::Close,
                ],
            },
            color: Color::RED,
        }];

        // Act
        let code = shapes_to_rust_code(&shapes);

        // Assert
        assert!(code.contains("MoveTo"), "missing MoveTo: {code}");
        assert!(code.contains("LineTo"), "missing LineTo: {code}");
        assert!(code.contains("Close"), "missing Close: {code}");
    }

    // TC005
    #[test]
    fn when_path_shape_with_cubic_then_codegen_contains_control_point_coordinates() {
        // Arrange
        let shapes = vec![Shape {
            variant: ShapeVariant::Path {
                commands: vec![PathCommand::CubicTo {
                    control1: Vec2::new(1.0, 2.0),
                    control2: Vec2::new(3.0, 4.0),
                    to: Vec2::new(5.0, 0.0),
                }],
            },
            color: Color::WHITE,
        }];

        // Act
        let code = shapes_to_rust_code(&shapes);

        // Assert
        assert!(code.contains("CubicTo"), "missing CubicTo: {code}");
        assert!(code.contains("control1"), "missing control1: {code}");
        assert!(code.contains("control2"), "missing control2: {code}");
    }

    // TC006
    #[test]
    fn when_path_shape_with_quadratic_then_codegen_contains_control_and_to() {
        // Arrange
        let shapes = vec![Shape {
            variant: ShapeVariant::Path {
                commands: vec![PathCommand::QuadraticTo {
                    control: Vec2::new(5.0, 10.0),
                    to: Vec2::new(10.0, 0.0),
                }],
            },
            color: Color::WHITE,
        }];

        // Act
        let code = shapes_to_rust_code(&shapes);

        // Assert
        assert!(code.contains("QuadraticTo"), "missing QuadraticTo: {code}");
        assert!(code.contains("control"), "missing control: {code}");
    }

    // TC007
    #[test]
    fn when_multiple_shapes_then_codegen_output_has_correct_count() {
        // Arrange
        let shapes = vec![
            Shape {
                variant: ShapeVariant::Circle { radius: 5.0 },
                color: Color::RED,
            },
            Shape {
                variant: ShapeVariant::Polygon {
                    points: vec![Vec2::ZERO],
                },
                color: Color::GREEN,
            },
            Shape {
                variant: ShapeVariant::Path {
                    commands: vec![PathCommand::Close],
                },
                color: Color::BLUE,
            },
        ];

        // Act
        let code = shapes_to_rust_code(&shapes);

        // Assert
        let shape_count = code.matches("Shape {").count();
        assert_eq!(shape_count, 3, "expected 3 Shape expressions: {code}");
    }

    // TC008
    #[test]
    fn when_color_with_fractional_components_then_codegen_uses_float_literals() {
        // Arrange
        let shapes = vec![Shape {
            variant: ShapeVariant::Circle { radius: 1.0 },
            color: Color::new(0.502, 0.251, 0.753, 1.0),
        }];

        // Act
        let code = shapes_to_rust_code(&shapes);

        // Assert
        assert!(code.contains("0.502"), "missing 0.502: {code}");
        assert!(code.contains("0.251"), "missing 0.251: {code}");
        assert!(code.contains("0.753"), "missing 0.753: {code}");
    }

    // TC009
    #[test]
    fn when_codegen_output_then_structurally_valid_vec_literal() {
        // Arrange
        let shapes = vec![
            Shape {
                variant: ShapeVariant::Circle { radius: 5.0 },
                color: Color::RED,
            },
            Shape {
                variant: ShapeVariant::Path {
                    commands: vec![PathCommand::MoveTo(Vec2::ZERO), PathCommand::Close],
                },
                color: Color::BLUE,
            },
        ];

        // Act
        let code = shapes_to_rust_code(&shapes);

        // Assert
        assert!(code.starts_with("vec!["), "should start with vec![: {code}");
        assert!(code.ends_with(']'), "should end with ]: {code}");
        let shape_count = code.matches("Shape {").count();
        assert_eq!(shape_count, 2, "expected 2 shapes: {code}");
    }
}
