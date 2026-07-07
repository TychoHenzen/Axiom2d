#![allow(clippy::unwrap_used)]

use engine_core::prelude::Color;
use engine_render::prelude::PathCommand;
use engine_render::shape::{Shape, ShapeVariant};
use img_to_shape_gui::preview::{path_commands_to_egui_shapes, shape_to_egui_shapes};

/// @doc: Empty path commands produce empty egui shapes
#[test]
fn when_empty_commands_then_returns_empty_shapes() {
    // Arrange
    let commands: Vec<PathCommand> = vec![];

    // Act
    let shapes = path_commands_to_egui_shapes(
        &commands,
        egui::vec2(100.0, 100.0),
        &Color::RED,
    );

    // Assert
    assert!(
        shapes.is_empty(),
        "empty path commands should produce no egui shapes"
    );
}

/// @doc: A simple closed rect path produces egui mesh shapes
#[test]
fn when_simple_rect_then_produces_egui_shapes() {
    // Arrange
    let commands = vec![
        PathCommand::MoveTo(glam::Vec2::new(0.0, 0.0)),
        PathCommand::LineTo(glam::Vec2::new(1.0, 0.0)),
        PathCommand::LineTo(glam::Vec2::new(1.0, 1.0)),
        PathCommand::LineTo(glam::Vec2::new(0.0, 1.0)),
        PathCommand::Close,
    ];

    // Act
    let shapes = path_commands_to_egui_shapes(
        &commands,
        egui::vec2(200.0, 200.0),
        &Color::GREEN,
    );

    // Assert — should produce at least one mesh shape for the filled rect
    assert!(
        !shapes.is_empty(),
        "simple rect should produce egui shapes"
    );
}

/// @doc: shape_to_egui_shapes handles Path variant
#[test]
fn when_shape_with_path_variant_then_returns_egui_shapes() {
    // Arrange
    let shape = Shape {
        variant: ShapeVariant::Path {
            commands: vec![
                PathCommand::MoveTo(glam::Vec2::new(0.0, 0.0)),
                PathCommand::LineTo(glam::Vec2::new(1.0, 0.0)),
                PathCommand::LineTo(glam::Vec2::new(1.0, 1.0)),
                PathCommand::LineTo(glam::Vec2::new(0.0, 1.0)),
                PathCommand::Close,
            ],
        },
        color: Color::BLUE,
    };

    // Act
    let shapes = shape_to_egui_shapes(&shape, egui::vec2(100.0, 100.0));

    // Assert
    assert!(
        !shapes.is_empty(),
        "Path variant shape should produce egui shapes"
    );
}
