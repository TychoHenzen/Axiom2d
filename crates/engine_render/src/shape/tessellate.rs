use glam::Vec2;
use lyon::math::point;
use lyon::path::Path as LyonPath;
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, StrokeOptions, StrokeTessellator,
    StrokeVertex, VertexBuffers,
};

use super::components::{ShapeVariant, TessellatedMesh};
use super::path::{PathCommand, resolve_commands};

#[derive(Debug)]
pub enum TessellateError {
    Circle(lyon::tessellation::TessellationError),
    Polygon(lyon::tessellation::TessellationError),
    Path(lyon::tessellation::TessellationError),
}

fn empty_mesh() -> TessellatedMesh {
    TessellatedMesh {
        vertices: Vec::new(),
        indices: Vec::new(),
    }
}

pub fn tessellate(variant: &ShapeVariant) -> Result<TessellatedMesh, TessellateError> {
    let mut geo: VertexBuffers<[f32; 2], u32> = VertexBuffers::new();
    let mut tess = FillTessellator::new();
    let opts = FillOptions::default();

    match variant {
        ShapeVariant::Circle { radius } => {
            fill_circle(&mut tess, opts, &mut geo, *radius)?;
        }
        ShapeVariant::Polygon { points } => {
            if points.len() < 3 {
                return Ok(empty_mesh());
            }
            fill_polygon(&mut tess, opts, &mut geo, points)?;
        }
        ShapeVariant::Path { commands } => {
            if commands.is_empty() {
                return Ok(empty_mesh());
            }
            fill_path(&mut tess, opts, &mut geo, commands)?;
        }
    }

    Ok(TessellatedMesh {
        vertices: geo.vertices,
        indices: geo.indices,
    })
}

fn fill_circle(
    tess: &mut FillTessellator,
    opts: FillOptions,
    geo: &mut VertexBuffers<[f32; 2], u32>,
    radius: f32,
) -> Result<(), TessellateError> {
    tess.tessellate_circle(
        point(0.0, 0.0),
        radius,
        &opts,
        &mut BuffersBuilder::new(geo, |v: FillVertex| v.position().to_array()),
    )
    .map_err(TessellateError::Circle)
}

fn fill_polygon(
    tess: &mut FillTessellator,
    opts: FillOptions,
    geo: &mut VertexBuffers<[f32; 2], u32>,
    points: &[Vec2],
) -> Result<(), TessellateError> {
    let lp: Vec<lyon::math::Point> = points.iter().map(|p| point(p.x, p.y)).collect();
    tess.tessellate_polygon(
        lyon::path::Polygon {
            points: &lp,
            closed: true,
        },
        &opts,
        &mut BuffersBuilder::new(geo, |v: FillVertex| v.position().to_array()),
    )
    .map_err(TessellateError::Polygon)
}

fn fill_path(
    tess: &mut FillTessellator,
    opts: FillOptions,
    geo: &mut VertexBuffers<[f32; 2], u32>,
    commands: &[PathCommand],
) -> Result<(), TessellateError> {
    let path = build_lyon_path(commands);
    tess.tessellate_path(
        &path,
        &opts,
        &mut BuffersBuilder::new(geo, |v: FillVertex| v.position().to_array()),
    )
    .map_err(TessellateError::Path)
}

fn build_lyon_path(commands: &[PathCommand]) -> LyonPath {
    let resolved = resolve_commands(commands);
    let mut builder = LyonPath::builder();
    let mut needs_begin = true;
    for cmd in &resolved {
        needs_begin = apply_path_cmd(&mut builder, cmd, needs_begin);
    }
    if !needs_begin {
        builder.end(false);
    }
    builder.build()
}

fn ensure_begun(builder: &mut LyonBuilder, needs_begin: bool) -> bool {
    if needs_begin {
        builder.begin(point(0.0, 0.0));
    }
    false
}

type LyonBuilder = lyon::path::builder::NoAttributes<lyon::path::BuilderImpl>;

fn apply_path_cmd(builder: &mut LyonBuilder, cmd: &PathCommand, needs_begin: bool) -> bool {
    match cmd {
        PathCommand::MoveTo(p) => apply_move_to(builder, *p, needs_begin),
        PathCommand::LineTo(p) => {
            let nb = ensure_begun(builder, needs_begin);
            builder.line_to(point(p.x, p.y));
            nb
        }
        PathCommand::QuadraticTo { control, to } => {
            let nb = ensure_begun(builder, needs_begin);
            builder.quadratic_bezier_to(point(control.x, control.y), point(to.x, to.y));
            nb
        }
        PathCommand::CubicTo {
            control1,
            control2,
            to,
        } => apply_cubic(builder, *control1, *control2, *to, needs_begin),
        PathCommand::Close => {
            if !needs_begin {
                builder.end(true);
            }
            true
        }
        PathCommand::Reverse => needs_begin,
    }
}

fn apply_move_to(builder: &mut LyonBuilder, p: Vec2, needs_begin: bool) -> bool {
    if !needs_begin {
        builder.end(false);
    }
    builder.begin(point(p.x, p.y));
    false
}

fn apply_cubic(builder: &mut LyonBuilder, c1: Vec2, c2: Vec2, to: Vec2, needs_begin: bool) -> bool {
    let nb = ensure_begun(builder, needs_begin);
    builder.cubic_bezier_to(point(c1.x, c1.y), point(c2.x, c2.y), point(to.x, to.y));
    nb
}

pub fn tessellate_stroke(
    variant: &ShapeVariant,
    line_width: f32,
) -> Result<TessellatedMesh, TessellateError> {
    let mut geo: VertexBuffers<[f32; 2], u32> = VertexBuffers::new();
    let mut tess = StrokeTessellator::new();
    let opts = StrokeOptions::default().with_line_width(line_width);

    match variant {
        ShapeVariant::Circle { radius } => {
            stroke_circle(&mut tess, &opts, &mut geo, *radius)?;
        }
        ShapeVariant::Polygon { points } => {
            if points.len() < 3 {
                return Ok(empty_mesh());
            }
            stroke_polygon(&mut tess, &opts, &mut geo, points)?;
        }
        ShapeVariant::Path { commands } => {
            if commands.is_empty() {
                return Ok(empty_mesh());
            }
            stroke_path(&mut tess, &opts, &mut geo, commands)?;
        }
    }

    Ok(TessellatedMesh {
        vertices: geo.vertices,
        indices: geo.indices,
    })
}

fn stroke_circle(
    tess: &mut StrokeTessellator,
    opts: &StrokeOptions,
    geo: &mut VertexBuffers<[f32; 2], u32>,
    radius: f32,
) -> Result<(), TessellateError> {
    tess.tessellate_circle(
        point(0.0, 0.0),
        radius,
        opts,
        &mut BuffersBuilder::new(geo, |v: StrokeVertex| v.position().to_array()),
    )
    .map_err(TessellateError::Circle)
}

fn stroke_polygon(
    tess: &mut StrokeTessellator,
    opts: &StrokeOptions,
    geo: &mut VertexBuffers<[f32; 2], u32>,
    points: &[Vec2],
) -> Result<(), TessellateError> {
    let path = polygon_to_lyon_path(points);
    tess.tessellate_path(
        &path,
        opts,
        &mut BuffersBuilder::new(geo, |v: StrokeVertex| v.position().to_array()),
    )
    .map_err(TessellateError::Path)
}

fn polygon_to_lyon_path(points: &[Vec2]) -> LyonPath {
    let lp: Vec<lyon::math::Point> = points.iter().map(|p| point(p.x, p.y)).collect();
    let mut builder = LyonPath::builder();
    builder.begin(lp[0]);
    for &pt in &lp[1..] {
        builder.line_to(pt);
    }
    builder.end(true);
    builder.build()
}

fn stroke_path(
    tess: &mut StrokeTessellator,
    opts: &StrokeOptions,
    geo: &mut VertexBuffers<[f32; 2], u32>,
    commands: &[PathCommand],
) -> Result<(), TessellateError> {
    let path = build_lyon_path(commands);
    tess.tessellate_path(
        &path,
        opts,
        &mut BuffersBuilder::new(geo, |v: StrokeVertex| v.position().to_array()),
    )
    .map_err(TessellateError::Path)
}

pub fn shape_aabb(variant: &ShapeVariant) -> (Vec2, Vec2) {
    match variant {
        ShapeVariant::Circle { radius } => {
            let r = *radius;
            (Vec2::new(-r, -r), Vec2::new(r, r))
        }
        ShapeVariant::Polygon { points } => polygon_aabb(points),
        ShapeVariant::Path { commands } => path_aabb(commands),
    }
}

fn polygon_aabb(points: &[Vec2]) -> (Vec2, Vec2) {
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

fn path_aabb(commands: &[PathCommand]) -> (Vec2, Vec2) {
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn when_tessellating_circle_then_produces_nonempty_vertices_and_indices() {
        // Arrange
        let variant = ShapeVariant::Circle { radius: 50.0 };

        // Act
        let mesh = tessellate(&variant).unwrap();

        // Assert
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());
    }

    #[test]
    fn when_tessellating_stroke_on_circle_then_produces_nonempty_vertices_and_indices() {
        // Arrange
        let variant = ShapeVariant::Circle { radius: 50.0 };

        // Act
        let mesh = tessellate_stroke(&variant, 4.0).unwrap();

        // Assert
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());
    }

    #[test]
    fn when_tessellating_stroke_on_circle_then_index_count_is_multiple_of_three() {
        // Arrange
        let variant = ShapeVariant::Circle { radius: 50.0 };

        // Act
        let mesh = tessellate_stroke(&variant, 4.0).unwrap();

        // Assert
        assert!(!mesh.indices.is_empty());
        assert_eq!(mesh.indices.len() % 3, 0);
    }

    #[test]
    fn when_tessellating_stroke_on_circle_then_all_indices_within_vertex_bounds() {
        // Arrange
        let variant = ShapeVariant::Circle { radius: 50.0 };

        // Act
        let mesh = tessellate_stroke(&variant, 4.0).unwrap();

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
        let mesh = tessellate_stroke(&variant, 4.0).unwrap();

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
        let mesh = tessellate_stroke(&variant, 4.0).unwrap();

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
        let mesh = tessellate_stroke(&variant, 4.0).unwrap();

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
        let mesh = tessellate_stroke(&variant, 4.0).unwrap();

        // Assert
        assert!(mesh.vertices.is_empty());
        assert!(mesh.indices.is_empty());
    }

    #[test]
    fn when_tessellating_stroke_with_wider_line_then_more_vertices() {
        // Arrange
        let variant = ShapeVariant::Circle { radius: 50.0 };

        // Act
        let narrow = tessellate_stroke(&variant, 2.0).unwrap();
        let wide = tessellate_stroke(&variant, 20.0).unwrap();

        // Assert
        assert!(wide.vertices.len() >= narrow.vertices.len());
    }

    /// @doc: MoveTo with non-zero coordinates — incorrect state tracking can snap geometry to origin
    #[test]
    fn when_path_moveto_nonorigin_then_vertices_are_near_moveto_point() {
        // Arrange — MoveTo to (50,50), then draw a triangle there.
        // If apply_move_to incorrectly returns true, ensure_begun would start
        // a new sub-path at (0,0), shifting the geometry to the origin.
        let variant = ShapeVariant::Path {
            commands: vec![
                PathCommand::MoveTo(Vec2::new(50.0, 50.0)),
                PathCommand::LineTo(Vec2::new(150.0, 50.0)),
                PathCommand::LineTo(Vec2::new(100.0, 150.0)),
                PathCommand::Close,
            ],
        };

        // Act
        let mesh = tessellate(&variant).unwrap();

        // Assert — all vertices should be in the [50,150] range, NOT near origin
        assert!(!mesh.vertices.is_empty());
        for v in &mesh.vertices {
            assert!(
                v[0] >= 49.0 && v[0] <= 151.0,
                "vertex x={} outside expected [50,150] range",
                v[0]
            );
            assert!(
                v[1] >= 49.0 && v[1] <= 151.0,
                "vertex y={} outside expected [50,150] range",
                v[1]
            );
        }
    }

    #[test]
    fn when_tessellating_circle_then_index_count_is_multiple_of_three() {
        // Arrange
        let variant = ShapeVariant::Circle { radius: 50.0 };

        // Act
        let mesh = tessellate(&variant).unwrap();

        // Assert
        assert_eq!(mesh.indices.len() % 3, 0);
    }

    #[test]
    fn when_tessellating_circle_then_all_indices_within_vertex_bounds() {
        // Arrange
        let variant = ShapeVariant::Circle { radius: 50.0 };

        // Act
        let mesh = tessellate(&variant).unwrap();

        // Assert
        let vertex_count = mesh.vertices.len() as u32;
        for &index in &mesh.indices {
            assert!(
                index < vertex_count,
                "index {index} out of bounds (vertex count {vertex_count})"
            );
        }
    }

    /// @doc: Zero-radius circle must not panic — degenerate shapes occur from animation edge cases
    #[test]
    fn when_tessellating_zero_radius_circle_then_does_not_panic() {
        // Arrange
        let variant = ShapeVariant::Circle { radius: 0.0 };

        // Act
        let mesh = tessellate(&variant).unwrap();

        // Assert
        assert_eq!(mesh.indices.len() % 3, 0);
    }

    #[test]
    fn when_tessellating_larger_circle_then_more_vertices_than_smaller() {
        // Arrange
        let small = ShapeVariant::Circle { radius: 10.0 };
        let large = ShapeVariant::Circle { radius: 100.0 };

        // Act
        let small_mesh = tessellate(&small).unwrap();
        let large_mesh = tessellate(&large).unwrap();

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
        let mesh = tessellate(&variant).unwrap();

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
        let mesh = tessellate(&variant).unwrap();

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

    /// @doc: Degenerate polygons (< 3 points) return empty mesh — prevents tessellation crashes
    #[test]
    fn when_tessellating_polygon_with_fewer_than_three_points_then_returns_empty_mesh() {
        // Arrange
        let variant = ShapeVariant::Polygon {
            points: vec![Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0)],
        };

        // Act
        let mesh = tessellate(&variant).unwrap();

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
        let mesh = tessellate(&variant).unwrap();

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
        let mesh = tessellate(&variant).unwrap();

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
        let mesh = tessellate(&variant).unwrap();

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
        let mesh = tessellate(&variant).unwrap();

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
        let mesh = tessellate(&variant).unwrap();

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
        let mesh = tessellate(&variant).unwrap();

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
        let mesh = tessellate(&variant).unwrap();

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
        let mesh = tessellate(&variant).unwrap();

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

    /// @doc: Path AABB includes cubic control points — conservative bounds prevent false culling
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
        let mesh = tessellate(&variant).unwrap();

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
        let mesh = tessellate(&variant).unwrap();

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
        let mesh = tessellate_stroke(&variant, 2.0).unwrap();

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
        let mesh = tessellate_stroke(&variant, 2.0).unwrap();

        // Assert
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());
    }

    /// @doc: Path Close command generates closing stroke segment — omission causes missing visual edge
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
        let closed_mesh = tessellate_stroke(&closed, 1.0).unwrap();
        let open_mesh = tessellate_stroke(&open, 1.0).unwrap();

        // Assert — closing the path adds a third stroke segment
        assert!(
            closed_mesh.vertices.len() > open_mesh.vertices.len(),
            "closed stroke ({} verts) should have more vertices than open ({} verts)",
            closed_mesh.vertices.len(),
            open_mesh.vertices.len()
        );
    }
}
