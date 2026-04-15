use glam::Vec2;
use lyon::math::point;
use lyon::path::Path as LyonPath;
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillRule, FillTessellator, FillVertex, StrokeOptions,
    StrokeTessellator, StrokeVertex, VertexBuffers,
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
            // NonZero keeps self-intersecting band polygons solid instead of XOR-ing.
            let poly_opts = opts.with_fill_rule(FillRule::NonZero);
            fill_polygon(&mut tess, poly_opts, &mut geo, points)?;
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
            PathCommand::Close if !needs_begin => {
                builder.end(true);
                needs_begin = true;
            }
            PathCommand::Close | PathCommand::Reverse => {}
        }
    }

    if !needs_begin {
        builder.end(false);
    }
    builder.build()
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
    let Some((first, rest)) = points.split_first() else {
        return LyonPath::builder().build();
    };

    let mut builder = LyonPath::builder();
    builder.begin(point(first.x, first.y));
    for &p in rest {
        builder.line_to(point(p.x, p.y));
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
    bounds_from_points(points.iter().copied())
}

fn path_aabb(commands: &[PathCommand]) -> (Vec2, Vec2) {
    bounds_from_points(commands.iter().filter_map(|cmd| match cmd {
        PathCommand::MoveTo(p) | PathCommand::LineTo(p) => Some(*p),
        PathCommand::QuadraticTo { to, .. } | PathCommand::CubicTo { to, .. } => Some(*to),
        PathCommand::Close | PathCommand::Reverse => None,
    }))
}

fn bounds_from_points(mut points: impl Iterator<Item = Vec2>) -> (Vec2, Vec2) {
    let Some(first) = points.next() else {
        return (Vec2::ZERO, Vec2::ZERO);
    };
    points.fold((first, first), |(min, max), p| (min.min(p), max.max(p)))
}
