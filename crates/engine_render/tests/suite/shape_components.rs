#![allow(clippy::unwrap_used, clippy::float_cmp)]

use engine_core::color::Color;
use engine_render::shape::{
    ColorVertex, PathCommand, Shape, ShapeVariant, Stroke, TessellatedColorMesh,
};
use glam::Vec2;

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
