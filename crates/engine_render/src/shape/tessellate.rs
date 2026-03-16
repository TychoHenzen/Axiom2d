use glam::Vec2;
use lyon::math::point;
use lyon::path::Path as LyonPath;
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, StrokeOptions, StrokeTessellator,
    StrokeVertex, VertexBuffers,
};

use super::components::{ShapeVariant, TessellatedMesh};
use super::path::{PathCommand, resolve_commands};

pub fn tessellate(variant: &ShapeVariant) -> TessellatedMesh {
    let mut geometry: VertexBuffers<[f32; 2], u32> = VertexBuffers::new();
    let mut tessellator = FillTessellator::new();
    let options = FillOptions::default();

    match variant {
        ShapeVariant::Circle { radius } => {
            tessellator
                .tessellate_circle(
                    point(0.0, 0.0),
                    *radius,
                    &options,
                    &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| {
                        vertex.position().to_array()
                    }),
                )
                .expect("circle tessellation failed");
        }
        ShapeVariant::Polygon { points } => {
            if points.len() < 3 {
                return TessellatedMesh {
                    vertices: Vec::new(),
                    indices: Vec::new(),
                };
            }
            let lyon_points: Vec<lyon::math::Point> =
                points.iter().map(|p| point(p.x, p.y)).collect();
            tessellator
                .tessellate_polygon(
                    lyon::path::Polygon {
                        points: &lyon_points,
                        closed: true,
                    },
                    &options,
                    &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| {
                        vertex.position().to_array()
                    }),
                )
                .expect("polygon tessellation failed");
        }
        ShapeVariant::Path { commands } => {
            if commands.is_empty() {
                return TessellatedMesh {
                    vertices: Vec::new(),
                    indices: Vec::new(),
                };
            }
            let path = build_lyon_path(commands);
            tessellator
                .tessellate_path(
                    &path,
                    &options,
                    &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| {
                        vertex.position().to_array()
                    }),
                )
                .expect("path tessellation failed");
        }
    }

    TessellatedMesh {
        vertices: geometry.vertices,
        indices: geometry.indices,
    }
}

fn build_lyon_path(commands: &[PathCommand]) -> LyonPath {
    let resolved = resolve_commands(commands);
    let mut builder = LyonPath::builder();
    let mut needs_begin = true;
    for cmd in &resolved {
        match cmd {
            PathCommand::MoveTo(p) => {
                if !needs_begin {
                    builder.end(false);
                }
                builder.begin(point(p.x, p.y));
                needs_begin = false;
            }
            PathCommand::LineTo(p) => {
                if needs_begin {
                    builder.begin(point(0.0, 0.0));
                    needs_begin = false;
                }
                builder.line_to(point(p.x, p.y));
            }
            PathCommand::QuadraticTo { control, to } => {
                if needs_begin {
                    builder.begin(point(0.0, 0.0));
                    needs_begin = false;
                }
                builder.quadratic_bezier_to(point(control.x, control.y), point(to.x, to.y));
            }
            PathCommand::CubicTo {
                control1,
                control2,
                to,
            } => {
                if needs_begin {
                    builder.begin(point(0.0, 0.0));
                    needs_begin = false;
                }
                builder.cubic_bezier_to(
                    point(control1.x, control1.y),
                    point(control2.x, control2.y),
                    point(to.x, to.y),
                );
            }
            PathCommand::Close => {
                if !needs_begin {
                    builder.end(true);
                    needs_begin = true;
                }
            }
            PathCommand::Reverse => {} // already resolved by resolve_commands
        }
    }
    if !needs_begin {
        builder.end(false);
    }
    builder.build()
}

pub fn tessellate_stroke(variant: &ShapeVariant, line_width: f32) -> TessellatedMesh {
    let mut geometry: VertexBuffers<[f32; 2], u32> = VertexBuffers::new();
    let mut tessellator = StrokeTessellator::new();
    let options = StrokeOptions::default().with_line_width(line_width);

    match variant {
        ShapeVariant::Circle { radius } => {
            tessellator
                .tessellate_circle(
                    point(0.0, 0.0),
                    *radius,
                    &options,
                    &mut BuffersBuilder::new(&mut geometry, |vertex: StrokeVertex| {
                        vertex.position().to_array()
                    }),
                )
                .expect("circle stroke tessellation failed");
        }
        ShapeVariant::Polygon { points } => {
            if points.len() < 3 {
                return TessellatedMesh {
                    vertices: Vec::new(),
                    indices: Vec::new(),
                };
            }
            let lyon_points: Vec<lyon::math::Point> =
                points.iter().map(|p| point(p.x, p.y)).collect();
            let mut builder = LyonPath::builder();
            builder.begin(lyon_points[0]);
            for &pt in &lyon_points[1..] {
                builder.line_to(pt);
            }
            builder.end(true);
            let path = builder.build();
            tessellator
                .tessellate_path(
                    &path,
                    &options,
                    &mut BuffersBuilder::new(&mut geometry, |vertex: StrokeVertex| {
                        vertex.position().to_array()
                    }),
                )
                .expect("polygon stroke tessellation failed");
        }
        ShapeVariant::Path { commands } => {
            if commands.is_empty() {
                return TessellatedMesh {
                    vertices: Vec::new(),
                    indices: Vec::new(),
                };
            }
            let path = build_lyon_path(commands);
            tessellator
                .tessellate_path(
                    &path,
                    &options,
                    &mut BuffersBuilder::new(&mut geometry, |vertex: StrokeVertex| {
                        vertex.position().to_array()
                    }),
                )
                .expect("path stroke tessellation failed");
        }
    }

    TessellatedMesh {
        vertices: geometry.vertices,
        indices: geometry.indices,
    }
}

pub(crate) fn shape_aabb(variant: &ShapeVariant) -> (Vec2, Vec2) {
    match variant {
        ShapeVariant::Circle { radius } => {
            let r = *radius;
            (Vec2::new(-r, -r), Vec2::new(r, r))
        }
        ShapeVariant::Polygon { points } => {
            if points.is_empty() {
                return (Vec2::ZERO, Vec2::ZERO);
            }
            let mut min = points[0];
            let mut max = points[0];
            for &p in &points[1..] {
                min = min.min(p);
                max = max.max(p);
            }
            (min, max)
        }
        ShapeVariant::Path { commands } => {
            let mut iter = commands.iter().filter_map(|cmd| match cmd {
                PathCommand::MoveTo(p) | PathCommand::LineTo(p) => Some(*p),
                PathCommand::QuadraticTo { to, .. } | PathCommand::CubicTo { to, .. } => Some(*to),
                PathCommand::Close | PathCommand::Reverse => None,
            });
            let Some(first) = iter.next() else {
                return (Vec2::ZERO, Vec2::ZERO);
            };
            iter.fold((first, first), |(min, max), p| (min.min(p), max.max(p)))
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn when_tessellating_circle_then_produces_nonempty_vertices_and_indices() {
        // Arrange
        let variant = ShapeVariant::Circle { radius: 50.0 };

        // Act
        let mesh = tessellate(&variant);

        // Assert
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());
    }

    #[test]
    fn when_tessellating_stroke_on_circle_then_produces_nonempty_vertices_and_indices() {
        // Arrange
        let variant = ShapeVariant::Circle { radius: 50.0 };

        // Act
        let mesh = tessellate_stroke(&variant, 4.0);

        // Assert
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());
    }

    #[test]
    fn when_tessellating_stroke_on_circle_then_index_count_is_multiple_of_three() {
        // Arrange
        let variant = ShapeVariant::Circle { radius: 50.0 };

        // Act
        let mesh = tessellate_stroke(&variant, 4.0);

        // Assert
        assert!(!mesh.indices.is_empty());
        assert_eq!(mesh.indices.len() % 3, 0);
    }

    #[test]
    fn when_tessellating_stroke_on_circle_then_all_indices_within_vertex_bounds() {
        // Arrange
        let variant = ShapeVariant::Circle { radius: 50.0 };

        // Act
        let mesh = tessellate_stroke(&variant, 4.0);

        // Assert
        let vertex_count = mesh.vertices.len() as u32;
        for &index in &mesh.indices {
            assert!(
                index < vertex_count,
                "index {index} out of bounds (vertex count {vertex_count})"
            );
        }
    }

    #[test]
    fn when_tessellating_stroke_on_polygon_then_produces_valid_mesh() {
        // Arrange
        let variant = ShapeVariant::Polygon {
            points: vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(100.0, 0.0),
                Vec2::new(100.0, 100.0),
                Vec2::new(0.0, 100.0),
            ],
        };

        // Act
        let mesh = tessellate_stroke(&variant, 4.0);

        // Assert
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());
        assert_eq!(mesh.indices.len() % 3, 0);
        let vertex_count = mesh.vertices.len() as u32;
        for &index in &mesh.indices {
            assert!(index < vertex_count);
        }
    }

    #[test]
    fn when_tessellating_stroke_on_path_then_produces_valid_mesh() {
        // Arrange
        let variant = ShapeVariant::Path {
            commands: vec![
                PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
                PathCommand::LineTo(Vec2::new(100.0, 0.0)),
                PathCommand::LineTo(Vec2::new(50.0, 100.0)),
                PathCommand::Close,
            ],
        };

        // Act
        let mesh = tessellate_stroke(&variant, 4.0);

        // Assert
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());
        assert_eq!(mesh.indices.len() % 3, 0);
        let vertex_count = mesh.vertices.len() as u32;
        for &index in &mesh.indices {
            assert!(index < vertex_count);
        }
    }

    #[test]
    fn when_tessellating_stroke_on_degenerate_polygon_then_returns_empty_mesh() {
        // Arrange
        let variant = ShapeVariant::Polygon {
            points: vec![Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0)],
        };

        // Act
        let mesh = tessellate_stroke(&variant, 4.0);

        // Assert
        assert!(mesh.vertices.is_empty());
        assert!(mesh.indices.is_empty());
    }

    #[test]
    fn when_tessellating_stroke_on_empty_path_then_returns_empty_mesh() {
        // Arrange
        let variant = ShapeVariant::Path {
            commands: Vec::new(),
        };

        // Act
        let mesh = tessellate_stroke(&variant, 4.0);

        // Assert
        assert!(mesh.vertices.is_empty());
        assert!(mesh.indices.is_empty());
    }

    #[test]
    fn when_tessellating_stroke_with_wider_line_then_more_vertices() {
        // Arrange
        let variant = ShapeVariant::Circle { radius: 50.0 };

        // Act
        let narrow = tessellate_stroke(&variant, 2.0);
        let wide = tessellate_stroke(&variant, 20.0);

        // Assert
        assert!(wide.vertices.len() >= narrow.vertices.len());
    }

    #[test]
    fn when_tessellating_circle_then_index_count_is_multiple_of_three() {
        // Arrange
        let variant = ShapeVariant::Circle { radius: 50.0 };

        // Act
        let mesh = tessellate(&variant);

        // Assert
        assert_eq!(mesh.indices.len() % 3, 0);
    }

    #[test]
    fn when_tessellating_circle_then_all_indices_within_vertex_bounds() {
        // Arrange
        let variant = ShapeVariant::Circle { radius: 50.0 };

        // Act
        let mesh = tessellate(&variant);

        // Assert
        let vertex_count = mesh.vertices.len() as u32;
        for &index in &mesh.indices {
            assert!(
                index < vertex_count,
                "index {index} out of bounds (vertex count {vertex_count})"
            );
        }
    }

    #[test]
    fn when_tessellating_zero_radius_circle_then_does_not_panic() {
        // Arrange
        let variant = ShapeVariant::Circle { radius: 0.0 };

        // Act
        let mesh = tessellate(&variant);

        // Assert
        assert_eq!(mesh.indices.len() % 3, 0);
    }

    #[test]
    fn when_tessellating_larger_circle_then_more_vertices_than_smaller() {
        // Arrange
        let small = ShapeVariant::Circle { radius: 10.0 };
        let large = ShapeVariant::Circle { radius: 100.0 };

        // Act
        let small_mesh = tessellate(&small);
        let large_mesh = tessellate(&large);

        // Assert
        assert!(large_mesh.vertices.len() >= small_mesh.vertices.len());
    }

    #[test]
    fn when_tessellating_triangle_polygon_then_produces_three_vertices_and_three_indices() {
        // Arrange
        let variant = ShapeVariant::Polygon {
            points: vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(100.0, 0.0),
                Vec2::new(50.0, 86.6),
            ],
        };

        // Act
        let mesh = tessellate(&variant);

        // Assert
        assert_eq!(mesh.vertices.len(), 3);
        assert_eq!(mesh.indices.len(), 3);
    }

    #[test]
    fn when_tessellating_quad_polygon_then_valid_triangulated_mesh() {
        // Arrange
        let variant = ShapeVariant::Polygon {
            points: vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(100.0, 0.0),
                Vec2::new(100.0, 100.0),
                Vec2::new(0.0, 100.0),
            ],
        };

        // Act
        let mesh = tessellate(&variant);

        // Assert
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());
        assert_eq!(mesh.indices.len() % 3, 0);
        let vertex_count = mesh.vertices.len() as u32;
        for &index in &mesh.indices {
            assert!(index < vertex_count);
        }
    }

    #[test]
    fn when_circle_aabb_then_width_and_height_equal_double_radius() {
        // Arrange
        let variant = ShapeVariant::Circle { radius: 50.0 };

        // Act
        let (min, max) = shape_aabb(&variant);

        // Assert
        assert_eq!(min, Vec2::new(-50.0, -50.0));
        assert_eq!(max, Vec2::new(50.0, 50.0));
    }

    #[test]
    fn when_polygon_aabb_then_matches_point_extents() {
        // Arrange
        let variant = ShapeVariant::Polygon {
            points: vec![
                Vec2::new(-10.0, -20.0),
                Vec2::new(30.0, 40.0),
                Vec2::new(5.0, -5.0),
            ],
        };

        // Act
        let (min, max) = shape_aabb(&variant);

        // Assert
        assert_eq!(min, Vec2::new(-10.0, -20.0));
        assert_eq!(max, Vec2::new(30.0, 40.0));
    }

    #[test]
    fn when_tessellating_polygon_with_fewer_than_three_points_then_returns_empty_mesh() {
        // Arrange
        let variant = ShapeVariant::Polygon {
            points: vec![Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0)],
        };

        // Act
        let mesh = tessellate(&variant);

        // Assert
        assert!(mesh.vertices.is_empty());
        assert!(mesh.indices.is_empty());
    }

    #[test]
    fn when_tessellating_path_with_empty_commands_then_returns_empty_mesh() {
        // Arrange
        let variant = ShapeVariant::Path {
            commands: Vec::new(),
        };

        // Act
        let mesh = tessellate(&variant);

        // Assert
        assert!(mesh.vertices.is_empty());
        assert!(mesh.indices.is_empty());
    }

    #[test]
    fn when_tessellating_path_with_no_moveto_then_implicitly_begins_at_origin() {
        // Arrange
        let variant = ShapeVariant::Path {
            commands: vec![
                PathCommand::LineTo(Vec2::new(100.0, 0.0)),
                PathCommand::LineTo(Vec2::new(50.0, 100.0)),
                PathCommand::Close,
            ],
        };

        // Act
        let mesh = tessellate(&variant);

        // Assert — implicit begin at (0,0) forms a valid triangle
        assert!(!mesh.vertices.is_empty());
        assert_eq!(mesh.indices.len() % 3, 0);
    }

    #[test]
    fn when_tessellating_path_triangle_then_produces_nonempty_mesh() {
        // Arrange
        let variant = ShapeVariant::Path {
            commands: vec![
                PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
                PathCommand::LineTo(Vec2::new(100.0, 0.0)),
                PathCommand::LineTo(Vec2::new(50.0, 100.0)),
                PathCommand::Close,
            ],
        };

        // Act
        let mesh = tessellate(&variant);

        // Assert
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());
    }

    #[test]
    fn when_tessellating_path_triangle_then_index_count_is_multiple_of_three() {
        // Arrange
        let variant = ShapeVariant::Path {
            commands: vec![
                PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
                PathCommand::LineTo(Vec2::new(100.0, 0.0)),
                PathCommand::LineTo(Vec2::new(50.0, 100.0)),
                PathCommand::Close,
            ],
        };

        // Act
        let mesh = tessellate(&variant);

        // Assert
        assert_eq!(mesh.indices.len() % 3, 0);
    }

    #[test]
    fn when_tessellating_path_triangle_then_all_indices_within_vertex_bounds() {
        // Arrange
        let variant = ShapeVariant::Path {
            commands: vec![
                PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
                PathCommand::LineTo(Vec2::new(100.0, 0.0)),
                PathCommand::LineTo(Vec2::new(50.0, 100.0)),
                PathCommand::Close,
            ],
        };

        // Act
        let mesh = tessellate(&variant);

        // Assert
        for &idx in &mesh.indices {
            assert!(
                (idx as usize) < mesh.vertices.len(),
                "index {idx} out of bounds for {} vertices",
                mesh.vertices.len()
            );
        }
    }

    #[test]
    fn when_tessellating_path_with_unclosed_subpath_then_produces_nonempty_mesh() {
        // Arrange
        let variant = ShapeVariant::Path {
            commands: vec![
                PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
                PathCommand::LineTo(Vec2::new(100.0, 0.0)),
                PathCommand::LineTo(Vec2::new(50.0, 100.0)),
            ],
        };

        // Act
        let mesh = tessellate(&variant);

        // Assert
        assert!(!mesh.vertices.is_empty());
        assert_eq!(mesh.indices.len() % 3, 0);
    }

    #[test]
    fn when_tessellating_path_with_quadratic_bezier_then_produces_triangulated_mesh() {
        // Arrange
        let variant = ShapeVariant::Path {
            commands: vec![
                PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
                PathCommand::QuadraticTo {
                    control: Vec2::new(50.0, 100.0),
                    to: Vec2::new(100.0, 0.0),
                },
                PathCommand::Close,
            ],
        };

        // Act
        let mesh = tessellate(&variant);

        // Assert
        assert!(mesh.vertices.len() > 2);
        assert!(!mesh.indices.is_empty());
        assert_eq!(mesh.indices.len() % 3, 0);
    }

    #[test]
    fn when_tessellating_path_with_cubic_bezier_then_produces_nonempty_triangulated_mesh() {
        // Arrange
        let variant = ShapeVariant::Path {
            commands: vec![
                PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
                PathCommand::CubicTo {
                    control1: Vec2::new(25.0, 100.0),
                    control2: Vec2::new(75.0, 100.0),
                    to: Vec2::new(100.0, 0.0),
                },
                PathCommand::Close,
            ],
        };

        // Act
        let mesh = tessellate(&variant);

        // Assert
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());
        assert_eq!(mesh.indices.len() % 3, 0);
    }

    #[test]
    fn when_path_aabb_with_triangle_then_returns_tight_bounding_box() {
        // Arrange
        let variant = ShapeVariant::Path {
            commands: vec![
                PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
                PathCommand::LineTo(Vec2::new(100.0, 0.0)),
                PathCommand::LineTo(Vec2::new(50.0, 100.0)),
                PathCommand::Close,
            ],
        };

        // Act
        let (min, max) = shape_aabb(&variant);

        // Assert
        assert_eq!(min, Vec2::new(0.0, 0.0));
        assert_eq!(max, Vec2::new(100.0, 100.0));
    }

    #[test]
    fn when_path_aabb_with_negative_coordinates_then_min_is_negative() {
        // Arrange
        let variant = ShapeVariant::Path {
            commands: vec![
                PathCommand::MoveTo(Vec2::new(-50.0, -30.0)),
                PathCommand::LineTo(Vec2::new(80.0, 0.0)),
                PathCommand::LineTo(Vec2::new(0.0, 60.0)),
                PathCommand::Close,
            ],
        };

        // Act
        let (min, max) = shape_aabb(&variant);

        // Assert
        assert_eq!(min, Vec2::new(-50.0, -30.0));
        assert_eq!(max, Vec2::new(80.0, 60.0));
    }

    #[test]
    fn when_path_aabb_with_empty_commands_then_returns_zero_aabb() {
        // Arrange
        let variant = ShapeVariant::Path {
            commands: Vec::new(),
        };

        // Act
        let (min, max) = shape_aabb(&variant);

        // Assert
        assert_eq!(min, Vec2::ZERO);
        assert_eq!(max, Vec2::ZERO);
    }

    #[test]
    fn when_path_aabb_with_only_close_command_then_returns_zero_aabb() {
        // Arrange
        let variant = ShapeVariant::Path {
            commands: vec![PathCommand::Close],
        };

        // Act
        let (min, max) = shape_aabb(&variant);

        // Assert
        assert_eq!(min, Vec2::ZERO);
        assert_eq!(max, Vec2::ZERO);
    }

    #[test]
    fn when_path_aabb_with_cubic_control_points_then_covers_endpoints() {
        // Arrange
        let variant = ShapeVariant::Path {
            commands: vec![
                PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
                PathCommand::CubicTo {
                    control1: Vec2::new(200.0, 50.0),
                    control2: Vec2::new(200.0, 100.0),
                    to: Vec2::new(100.0, 100.0),
                },
                PathCommand::Close,
            ],
        };

        // Act
        let (min, max) = shape_aabb(&variant);

        // Assert
        assert!(max.x >= 100.0);
        assert!(max.y >= 100.0);
        assert!(min.x <= 0.0);
        assert!(min.y <= 0.0);
    }

    #[test]
    fn when_build_lyon_path_with_close_then_produces_closed_path() {
        // Arrange
        let variant = ShapeVariant::Path {
            commands: vec![
                PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
                PathCommand::LineTo(Vec2::new(10.0, 0.0)),
                PathCommand::LineTo(Vec2::new(5.0, 10.0)),
                PathCommand::Close,
            ],
        };

        // Act
        let mesh = tessellate(&variant);

        // Assert — closed triangle should produce vertices and indices
        assert!(!mesh.vertices.is_empty(), "should produce vertices");
        assert!(!mesh.indices.is_empty(), "should produce indices");
    }

    #[test]
    fn when_build_lyon_path_without_close_then_still_produces_geometry() {
        // Arrange — open path (no Close command)
        let variant = ShapeVariant::Path {
            commands: vec![
                PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
                PathCommand::LineTo(Vec2::new(10.0, 0.0)),
                PathCommand::LineTo(Vec2::new(10.0, 10.0)),
                PathCommand::LineTo(Vec2::new(0.0, 10.0)),
            ],
        };

        // Act
        let mesh = tessellate(&variant);

        // Assert — open path should still be tessellatable
        assert!(!mesh.vertices.is_empty());
    }

    #[test]
    fn when_tessellate_stroke_polygon_with_fewer_than_3_points_then_empty() {
        // Arrange
        let variant = ShapeVariant::Polygon {
            points: vec![Vec2::ZERO, Vec2::X],
        };

        // Act
        let mesh = tessellate_stroke(&variant, 2.0);

        // Assert
        assert!(mesh.vertices.is_empty());
        assert!(mesh.indices.is_empty());
    }

    #[test]
    fn when_tessellate_stroke_polygon_with_3_points_then_produces_geometry() {
        // Arrange
        let variant = ShapeVariant::Polygon {
            points: vec![Vec2::ZERO, Vec2::new(10.0, 0.0), Vec2::new(5.0, 10.0)],
        };

        // Act
        let mesh = tessellate_stroke(&variant, 2.0);

        // Assert
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());
    }

    // Mutant 2: delete `!` in build_lyon_path Close arm (line 305).
    // `if !needs_begin` becomes `if needs_begin`, so Close never ends the path
    // builder — the contour is left open.  For stroke tessellation this omits
    // the closing segment, producing fewer vertices than a properly closed path.
    #[test]
    fn when_path_close_command_then_stroke_has_more_vertices_than_open_path() {
        // Arrange
        let closed = ShapeVariant::Path {
            commands: vec![
                PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
                PathCommand::LineTo(Vec2::new(10.0, 0.0)),
                PathCommand::LineTo(Vec2::new(5.0, 10.0)),
                PathCommand::Close,
            ],
        };
        let open = ShapeVariant::Path {
            commands: vec![
                PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
                PathCommand::LineTo(Vec2::new(10.0, 0.0)),
                PathCommand::LineTo(Vec2::new(5.0, 10.0)),
            ],
        };

        // Act
        let closed_mesh = tessellate_stroke(&closed, 1.0);
        let open_mesh = tessellate_stroke(&open, 1.0);

        // Assert — closing the path adds a third stroke segment
        assert!(
            closed_mesh.vertices.len() > open_mesh.vertices.len(),
            "closed stroke ({} verts) should have more vertices than open ({} verts)",
            closed_mesh.vertices.len(),
            open_mesh.vertices.len()
        );
    }
}
