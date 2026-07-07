#![allow(clippy::unwrap_used)]

use engine_core::prelude::Color;
use engine_render::prelude::PathCommand;
use engine_render::shape::{Shape, ShapeVariant};
use tiled_to_shapes::normalize::normalize_shapes;

fn make_rect_shape(x: f32, y: f32, w: f32, h: f32) -> Shape {
    Shape {
        variant: ShapeVariant::Path {
            commands: vec![
                PathCommand::MoveTo(glam::Vec2::new(x, y)),
                PathCommand::LineTo(glam::Vec2::new(x + w, y)),
                PathCommand::LineTo(glam::Vec2::new(x + w, y + h)),
                PathCommand::LineTo(glam::Vec2::new(x, y + h)),
                PathCommand::Close,
            ],
        },
        color: Color::RED,
    }
}

/// @doc: Empty input produces empty output
#[test]
fn when_empty_shapes_then_returns_empty() {
    // Arrange
    let shapes: Vec<Shape> = vec![];

    // Act
    let result = normalize_shapes(&shapes, 100.0, 100.0);

    // Assert
    assert!(
        result.is_empty(),
        "normalizing empty shapes should produce empty output, got {} entries",
        result.len()
    );
}

/// @doc: Normalize maps engine pixel coords to [0,1]² tile space
#[test]
fn when_single_shape_then_coordinates_normalized() {
    // Arrange — shape at engine origin, 200×200 image
    let shape = make_rect_shape(-50.0, -50.0, 100.0, 100.0);
    let shapes = vec![shape];

    // Act
    let result = normalize_shapes(&shapes, 200.0, 200.0);

    // Assert
    assert_eq!(result.len(), 1, "should produce one normalized path");
    let commands = &result[0];
    assert!(!commands.is_empty(), "normalized path should not be empty");
    // First MoveTo should be near (0.25, 0.25) since (-50+100)/200=0.25
    if let PathCommand::MoveTo(p) = commands[0] {
        assert!(
            (p.x - 0.25).abs() < 0.001,
            "expected x≈0.25 for engine x=-50 in 200px image, got x={}",
            p.x
        );
        assert!(
            (p.y - 0.25).abs() < 0.001,
            "expected y≈0.25 for engine y=-50 in 200px image, got y={}",
            p.y
        );
    } else {
        panic!("expected MoveTo as first command");
    }
}
