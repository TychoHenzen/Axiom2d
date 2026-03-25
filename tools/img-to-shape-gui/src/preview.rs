use egui::epaint::{PathShape, PathStroke};
use egui::{Color32, Pos2, Vec2 as EguiVec2};
use engine_core::color::Color;
use engine_render::shape::{PathCommand, Shape, sample_cubic, sample_quadratic};
use glam::Vec2;
use lyon::math::point;
use lyon::path::Path as LyonPath;
use lyon::tessellation::{BuffersBuilder, FillOptions, FillTessellator, FillVertex, VertexBuffers};

const BEZIER_SAMPLES: usize = 16;

/// Convert engine Color (f32 0..1) to egui Color32 (u8 0..255).
fn color_to_egui(color: &Color) -> Color32 {
    Color32::from_rgba_premultiplied(
        (color.r * 255.0).round() as u8,
        (color.g * 255.0).round() as u8,
        (color.b * 255.0).round() as u8,
        (color.a * 255.0).round() as u8,
    )
}

/// Convert engine Vec2 (y-up) to egui Pos2 (y-down) given a canvas size.
fn to_egui_pos(v: Vec2, canvas: EguiVec2) -> Pos2 {
    Pos2::new(v.x + canvas.x / 2.0, canvas.y / 2.0 - v.y)
}

/// Convert a sequence of `PathCommand`s into egui `PathShape`s for rendering.
///
/// Each contour (starting at `MoveTo`) becomes a separate `PathShape`.
/// Cubic/quadratic beziers are flattened to line segments.
pub fn path_commands_to_egui_shapes(
    commands: &[PathCommand],
    canvas_size: EguiVec2,
    color: &Color,
) -> Vec<egui::Shape> {
    let fill = color_to_egui(color);
    let mut shapes: Vec<egui::Shape> = Vec::new();
    let mut points: Vec<Pos2> = Vec::new();
    let mut closed = false;
    let mut last_engine_pt = Vec2::ZERO;

    for cmd in commands {
        match cmd {
            PathCommand::MoveTo(v) => {
                if !points.is_empty() {
                    shapes.push(make_path_shape(&points, closed, fill));
                    points.clear();
                    closed = false;
                }
                last_engine_pt = *v;
                points.push(to_egui_pos(*v, canvas_size));
            }
            PathCommand::LineTo(v) => {
                last_engine_pt = *v;
                points.push(to_egui_pos(*v, canvas_size));
            }
            PathCommand::QuadraticTo { control, to } => {
                let sampled = sample_quadratic(last_engine_pt, *control, *to, BEZIER_SAMPLES);
                for pt in &sampled[1..] {
                    points.push(to_egui_pos(*pt, canvas_size));
                }
                last_engine_pt = *to;
            }
            PathCommand::CubicTo {
                control1,
                control2,
                to,
            } => {
                let sampled =
                    sample_cubic(last_engine_pt, *control1, *control2, *to, BEZIER_SAMPLES);
                for pt in &sampled[1..] {
                    points.push(to_egui_pos(*pt, canvas_size));
                }
                last_engine_pt = *to;
            }
            PathCommand::Close => {
                closed = true;
            }
            PathCommand::Reverse => {}
        }
    }

    if !points.is_empty() {
        shapes.push(make_path_shape(&points, closed, fill));
    }

    shapes
}

/// Convert a full Shape into egui shapes for preview rendering.
pub fn shape_to_egui_shapes(shape: &Shape, canvas_size: EguiVec2) -> Vec<egui::Shape> {
    match &shape.variant {
        engine_render::shape::ShapeVariant::Path { commands } => {
            path_commands_to_egui_shapes(commands, canvas_size, &shape.color)
        }
        engine_render::shape::ShapeVariant::Circle { radius } => {
            let center = to_egui_pos(Vec2::ZERO, canvas_size);
            let fill = color_to_egui(&shape.color);
            vec![egui::Shape::circle_filled(center, *radius, fill)]
        }
        engine_render::shape::ShapeVariant::Polygon { points } => {
            let egui_pts: Vec<Pos2> = points
                .iter()
                .map(|p| to_egui_pos(*p, canvas_size))
                .collect();
            let fill = color_to_egui(&shape.color);
            vec![make_path_shape(&egui_pts, true, fill)]
        }
    }
}

/// Tessellate a polygon using lyon for correct concave rendering.
///
/// egui's `PathShape` uses fan tessellation which only works for convex
/// polygons. Concave shapes (stars, L-shapes, etc.) need proper
/// triangulation to render correctly.
fn make_path_shape(points: &[Pos2], closed: bool, fill: Color32) -> egui::Shape {
    if points.len() < 3 || !closed {
        // Degenerate or open path — fall back to egui's path rendering.
        return egui::Shape::Path(PathShape {
            points: points.to_vec(),
            closed,
            fill,
            stroke: PathStroke::NONE,
        });
    }

    // Build lyon path from points.
    let mut builder = LyonPath::builder();
    builder.begin(point(points[0].x, points[0].y));
    for p in &points[1..] {
        builder.line_to(point(p.x, p.y));
    }
    builder.close();
    let path = builder.build();

    // Tessellate.
    let mut geometry: VertexBuffers<Pos2, u32> = VertexBuffers::new();
    let mut tessellator = FillTessellator::new();
    let result = tessellator.tessellate_path(
        &path,
        &FillOptions::default(),
        &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| {
            let p = vertex.position();
            Pos2::new(p.x, p.y)
        }),
    );

    if result.is_err() || geometry.indices.is_empty() {
        // Tessellation failed — fall back to egui path.
        return egui::Shape::Path(PathShape {
            points: points.to_vec(),
            closed,
            fill,
            stroke: PathStroke::NONE,
        });
    }

    // Build egui Mesh from tessellated triangles.
    let mut mesh = egui::Mesh::default();
    mesh.vertices = geometry
        .vertices
        .iter()
        .map(|p| egui::epaint::Vertex {
            pos: *p,
            uv: egui::pos2(0.0, 0.0),
            color: fill,
        })
        .collect();
    mesh.indices = geometry.indices;

    // Add a thin stroke outline in the same color to cover sub-pixel gaps
    // between adjacent tessellated regions.
    let outline = egui::Shape::Path(PathShape {
        points: points.to_vec(),
        closed: true,
        fill: Color32::TRANSPARENT,
        stroke: PathStroke::new(1.5, fill),
    });

    egui::Shape::Vec(vec![egui::Shape::mesh(mesh), outline])
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn canvas() -> EguiVec2 {
        EguiVec2::new(100.0, 100.0)
    }

    fn extract_path_shape(shape: &egui::Shape) -> &PathShape {
        match shape {
            egui::Shape::Path(ps) => ps,
            other => panic!("expected Path shape, got {other:?}"),
        }
    }

    // TC010
    #[test]
    fn when_moveto_command_then_egui_path_starts_at_correct_position() {
        // Arrange
        let commands = vec![PathCommand::MoveTo(Vec2::new(10.0, 20.0))];

        // Act
        let shapes = path_commands_to_egui_shapes(&commands, canvas(), &Color::WHITE);

        // Assert
        assert_eq!(shapes.len(), 1);
        let ps = extract_path_shape(&shapes[0]);
        let pos = ps.points[0];
        assert!((pos.x - 60.0).abs() < 0.01, "x: expected 60, got {}", pos.x);
        assert!((pos.y - 30.0).abs() < 0.01, "y: expected 30, got {}", pos.y);
    }

    // TC011
    #[test]
    fn when_lineto_command_then_egui_path_appends_point() {
        // Arrange
        let commands = vec![
            PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
            PathCommand::LineTo(Vec2::new(5.0, 0.0)),
        ];

        // Act
        let shapes = path_commands_to_egui_shapes(&commands, canvas(), &Color::WHITE);

        // Assert
        let ps = extract_path_shape(&shapes[0]);
        assert_eq!(ps.points.len(), 2);
    }

    // TC012
    #[test]
    fn when_close_command_then_egui_shape_is_filled_mesh() {
        // Arrange
        let commands = vec![
            PathCommand::MoveTo(Vec2::ZERO),
            PathCommand::LineTo(Vec2::new(10.0, 0.0)),
            PathCommand::LineTo(Vec2::new(5.0, 8.0)),
            PathCommand::Close,
        ];

        // Act
        let shapes = path_commands_to_egui_shapes(&commands, canvas(), &Color::WHITE);

        // Assert — closed polygons produce a Vec containing mesh + stroke outline
        assert_eq!(shapes.len(), 1);
        assert!(
            matches!(&shapes[0], egui::Shape::Vec(_)),
            "closed polygon should produce a Vec (mesh + outline)"
        );
    }

    // TC013
    #[test]
    fn when_cubic_bezier_command_then_egui_shape_contains_mesh() {
        // Arrange
        let commands = vec![
            PathCommand::MoveTo(Vec2::ZERO),
            PathCommand::CubicTo {
                control1: Vec2::new(0.0, 10.0),
                control2: Vec2::new(10.0, 10.0),
                to: Vec2::new(10.0, 0.0),
            },
            PathCommand::Close,
        ];

        // Act
        let shapes = path_commands_to_egui_shapes(&commands, canvas(), &Color::WHITE);

        // Assert — closed bezier produces Vec containing mesh + outline
        assert_eq!(shapes.len(), 1);
        let inner = match &shapes[0] {
            egui::Shape::Vec(v) => v,
            other => panic!("expected Vec, got {other:?}"),
        };
        assert!(
            inner.iter().any(|s| matches!(s, egui::Shape::Mesh(_))),
            "should contain a Mesh sub-shape"
        );
    }

    // TC014
    #[test]
    fn when_quadratic_bezier_command_then_egui_shape_contains_mesh() {
        // Arrange
        let commands = vec![
            PathCommand::MoveTo(Vec2::ZERO),
            PathCommand::QuadraticTo {
                control: Vec2::new(5.0, 10.0),
                to: Vec2::new(10.0, 0.0),
            },
            PathCommand::Close,
        ];

        // Act
        let shapes = path_commands_to_egui_shapes(&commands, canvas(), &Color::WHITE);

        // Assert — closed quadratic produces Vec containing mesh + outline
        assert_eq!(shapes.len(), 1);
        let inner = match &shapes[0] {
            egui::Shape::Vec(v) => v,
            other => panic!("expected Vec, got {other:?}"),
        };
        assert!(
            inner.iter().any(|s| matches!(s, egui::Shape::Mesh(_))),
            "should contain a Mesh sub-shape"
        );
    }

    // TC015
    #[test]
    fn when_engine_coords_converted_then_y_axis_is_flipped() {
        // Arrange — positive engine y (up) should map to smaller screen y (up in egui)
        let commands = vec![PathCommand::MoveTo(Vec2::new(0.0, 1.0))];
        let canvas = EguiVec2::new(100.0, 100.0);

        // Act
        let shapes = path_commands_to_egui_shapes(&commands, canvas, &Color::WHITE);

        // Assert
        let ps = extract_path_shape(&shapes[0]);
        let y = ps.points[0].y;
        assert!(
            y < canvas.y / 2.0,
            "positive engine y should map above center (y < 50), got {y}"
        );
    }

    // TC016
    #[test]
    fn when_multiple_moveto_commands_then_returns_multiple_shapes() {
        // Arrange
        let commands = vec![
            PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
            PathCommand::LineTo(Vec2::new(5.0, 0.0)),
            PathCommand::Close,
            PathCommand::MoveTo(Vec2::new(20.0, 0.0)),
            PathCommand::LineTo(Vec2::new(25.0, 0.0)),
            PathCommand::Close,
        ];

        // Act
        let shapes = path_commands_to_egui_shapes(&commands, canvas(), &Color::WHITE);

        // Assert
        assert_eq!(shapes.len(), 2, "two contours should produce two shapes");
    }

    // TC017
    #[test]
    fn when_shape_color_converted_to_egui_then_rgba_preserved() {
        // Arrange
        let color = Color::new(0.2, 0.4, 0.6, 0.8);

        // Act
        let egui_color = color_to_egui(&color);

        // Assert
        let [r, g, b, a] = egui_color.to_array();
        assert!(
            (i16::from(r) - 51).unsigned_abs() <= 1,
            "r: expected ~51, got {r}"
        );
        assert!(
            (i16::from(g) - 102).unsigned_abs() <= 1,
            "g: expected ~102, got {g}"
        );
        assert!(
            (i16::from(b) - 153).unsigned_abs() <= 1,
            "b: expected ~153, got {b}"
        );
        assert!(
            (i16::from(a) - 204).unsigned_abs() <= 1,
            "a: expected ~204, got {a}"
        );
    }
}
