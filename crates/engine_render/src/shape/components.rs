use bevy_ecs::prelude::Component;
use engine_core::color::Color;
use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::path::PathCommand;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ShapeVariant {
    Circle { radius: f32 },
    Polygon { points: Vec<Vec2> },
    Path { commands: Vec<PathCommand> },
}

#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stroke {
    pub color: Color,
    pub width: f32,
}

#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Shape {
    pub variant: ShapeVariant,
    pub color: Color,
}

pub struct TessellatedMesh {
    pub vertices: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn when_polygon_shape_variant_debug_formatted_then_snapshot_matches() {
        // Arrange
        let variant = ShapeVariant::Polygon {
            points: vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(100.0, 0.0),
                Vec2::new(80.0, 60.0),
                Vec2::new(20.0, 60.0),
            ],
        };

        // Act
        let debug = format!("{variant:#?}");

        // Assert
        insta::assert_snapshot!(debug);
    }

    #[test]
    fn when_shape_circle_serialized_to_ron_then_deserializes_to_equal_value() {
        // Arrange
        let shape = Shape {
            variant: ShapeVariant::Circle { radius: 25.0 },
            color: Color::new(0.0, 1.0, 0.0, 1.0),
        };

        // Act
        let ron = ron::to_string(&shape).unwrap();
        let back: Shape = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(shape, back);
    }

    #[test]
    fn when_shape_polygon_serialized_to_ron_then_deserializes_with_point_order_preserved() {
        // Arrange
        let shape = Shape {
            variant: ShapeVariant::Polygon {
                points: vec![
                    Vec2::new(0.0, 0.0),
                    Vec2::new(100.0, 0.0),
                    Vec2::new(50.0, 86.6),
                ],
            },
            color: Color::RED,
        };

        // Act
        let ron = ron::to_string(&shape).unwrap();
        let back: Shape = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(shape, back);
    }

    #[test]
    fn when_stroke_serialized_to_ron_then_roundtrips() {
        // Arrange
        let stroke = Stroke {
            color: Color::new(0.0, 0.0, 0.0, 1.0),
            width: 2.5,
        };

        // Act
        let ron_str = ron::to_string(&stroke).unwrap();
        let back: Stroke = ron::from_str(&ron_str).unwrap();

        // Assert
        assert_eq!(stroke, back);
    }

    #[test]
    fn when_path_shape_serialized_to_ron_then_deserializes_to_equal_value() {
        // Arrange
        let shape = Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
                    PathCommand::LineTo(Vec2::new(100.0, 0.0)),
                    PathCommand::Close,
                ],
            },
            color: Color::new(1.0, 0.0, 0.0, 1.0),
        };

        // Act
        let ron_str = ron::to_string(&shape).expect("serialize");
        let deserialized: Shape = ron::from_str(&ron_str).expect("deserialize");

        // Assert
        assert_eq!(shape, deserialized);
    }

    #[test]
    fn when_path_with_all_command_types_serialized_to_ron_then_roundtrips() {
        // Arrange
        let variant = ShapeVariant::Path {
            commands: vec![
                PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
                PathCommand::LineTo(Vec2::new(50.0, 0.0)),
                PathCommand::QuadraticTo {
                    control: Vec2::new(75.0, 50.0),
                    to: Vec2::new(100.0, 0.0),
                },
                PathCommand::CubicTo {
                    control1: Vec2::new(25.0, 80.0),
                    control2: Vec2::new(75.0, 80.0),
                    to: Vec2::new(50.0, 100.0),
                },
                PathCommand::Close,
            ],
        };

        // Act
        let ron_str = ron::to_string(&variant).expect("serialize");
        let deserialized: ShapeVariant = ron::from_str(&ron_str).expect("deserialize");

        // Assert
        assert_eq!(variant, deserialized);
    }
}
