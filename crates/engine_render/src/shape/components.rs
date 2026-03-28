use std::ops::{Deref, DerefMut};

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

/// Vertex with baked position, RGBA color, and per-shape UV coordinates.
/// Layout matches `ShapeVertex` in the wgpu renderer (32 bytes).
/// UV encodes normalized position within the shape's bounding box \[0,1\],
/// giving shaders geometric hints about shape structure (edges, gradients).
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColorVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
    pub uv: [f32; 2],
}

/// Pre-tessellated mesh with per-vertex color.
/// Used by `BakedCardMesh` to store card geometry that never changes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TessellatedColorMesh {
    pub vertices: Vec<ColorVertex>,
    pub indices: Vec<u32>,
}

impl TessellatedColorMesh {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Append position-only vertices with a uniform color, offsetting indices.
    pub fn push_vertices(&mut self, positions: &[[f32; 2]], indices: &[u32], color: [f32; 4]) {
        let base = self.vertices.len() as u32;
        self.vertices
            .extend(positions.iter().map(|&position| ColorVertex {
                position,
                color,
                uv: [0.0, 0.0],
            }));
        self.indices.extend(indices.iter().map(|&i| i + base));
    }

    /// Append position-only vertices with a uniform color and per-vertex UV, offsetting indices.
    pub fn push_vertices_with_uv(
        &mut self,
        positions: &[[f32; 2]],
        uvs: &[[f32; 2]],
        indices: &[u32],
        color: [f32; 4],
    ) {
        let base = self.vertices.len() as u32;
        self.vertices
            .extend(
                positions
                    .iter()
                    .zip(uvs.iter())
                    .map(|(&position, &uv)| ColorVertex {
                        position,
                        color,
                        uv,
                    }),
            );
        self.indices.extend(indices.iter().map(|&i| i + base));
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }
}

/// A shape overlay drawn on top of the entity's primary mesh.
/// Used for shader-driven effects (art areas, foil, etc.) that can't be baked.
#[derive(Clone, Debug)]
pub struct OverlayEntry {
    pub mesh: TessellatedColorMesh,
    pub material: crate::material::Material2d,
    pub visible: bool,
}

/// Overlay quads drawn immediately after the entity's `ColorMesh`.
/// Each entry gets its own shader/material application and draw call,
/// but shares the entity's model transform and sort order.
#[derive(Component, Clone, Debug, Default)]
pub struct MeshOverlays(pub Vec<OverlayEntry>);

/// ECS component wrapping a pre-tessellated colored mesh for direct rendering.
/// The unified render system draws this via `draw_colored_mesh`, bypassing
/// per-frame tessellation. Game code sets this component to control what is drawn.
#[derive(Component, Clone, Debug, Default)]
pub struct ColorMesh(pub TessellatedColorMesh);

impl Deref for ColorMesh {
    type Target = TessellatedColorMesh;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ColorMesh {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::float_cmp)]
mod tests {
    use super::*;

    #[test]
    fn when_color_vertex_size_checked_then_exactly_32_bytes() {
        // Act
        let size = std::mem::size_of::<ColorVertex>();

        // Assert — position [f32;2] + color [f32;4] + uv [f32;2] = 32
        assert_eq!(size, 32);
    }

    #[test]
    fn when_push_vertices_called_then_uv_defaults_to_zero() {
        // Arrange
        let mut mesh = TessellatedColorMesh::new();

        // Act
        mesh.push_vertices(
            &[[0.0, 0.0], [10.0, 0.0], [5.0, 10.0]],
            &[0, 1, 2],
            [1.0, 0.0, 0.0, 1.0],
        );

        // Assert
        for v in &mesh.vertices {
            assert_eq!(v.uv, [0.0, 0.0]);
        }
    }

    #[test]
    fn when_push_vertices_with_uv_called_then_uv_preserved_per_vertex() {
        // Arrange
        let mut mesh = TessellatedColorMesh::new();
        let positions = [[0.0, 0.0], [10.0, 0.0], [5.0, 10.0]];
        let uvs = [[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]];
        let color = [1.0, 0.0, 0.0, 1.0];

        // Act
        mesh.push_vertices_with_uv(&positions, &uvs, &[0, 1, 2], color);

        // Assert
        assert_eq!(mesh.vertices.len(), 3);
        assert_eq!(mesh.vertices[0].uv, [0.0, 0.0]);
        assert_eq!(mesh.vertices[1].uv, [1.0, 0.0]);
        assert_eq!(mesh.vertices[2].uv, [0.5, 1.0]);
    }

    #[test]
    fn when_colored_mesh_merge_two_triangles_then_indices_offset_correctly() {
        // Arrange
        let mut mesh = TessellatedColorMesh::new();
        let white = [1.0, 1.0, 1.0, 1.0];
        let red = [1.0, 0.0, 0.0, 1.0];
        mesh.push_vertices(&[[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]], &[0, 1, 2], white);

        // Act
        mesh.push_vertices(&[[2.0, 0.0], [3.0, 0.0], [2.5, 1.0]], &[0, 1, 2], red);

        // Assert
        assert_eq!(mesh.vertices.len(), 6);
        assert_eq!(mesh.indices.len(), 6);
        assert_eq!(mesh.indices[3..], [3, 4, 5]);
        assert_eq!(mesh.vertices[0].color, white);
        assert_eq!(mesh.vertices[3].color, red);
    }

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
