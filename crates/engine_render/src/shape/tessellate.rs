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
