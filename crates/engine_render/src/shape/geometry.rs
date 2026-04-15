use super::ShapeVariant;
use super::path::PathCommand;
use glam::Vec2;

/// Normalized unit quad with vertices in [-0.5, 0.5].
///
/// When drawn with a model matrix from [`unit_quad_model`], the model scale
/// directly encodes the world-space width and height — no intermediate
/// coordinate space. UV mapping in the shader is `local_pos + 0.5`.
pub const UNIT_QUAD: [[f32; 2]; 4] = [[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]];

pub const QUAD_INDICES: [u32; 6] = [0, 1, 2, 0, 2, 3];

/// Build a model matrix that scales a [-0.5, 0.5] unit quad to the given
/// width × height and translates to (cx, cy).
pub fn unit_quad_model(width: f32, height: f32, cx: f32, cy: f32) -> [[f32; 4]; 4] {
    [
        [width, 0.0, 0.0, 0.0],
        [0.0, height, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [cx, cy, 0.0, 1.0],
    ]
}

/// Create a rectangular polygon centered at origin with given half-extents.
pub fn rect_polygon(half_w: f32, half_h: f32) -> ShapeVariant {
    ShapeVariant::Polygon {
        points: vec![
            Vec2::new(-half_w, -half_h),
            Vec2::new(half_w, -half_h),
            Vec2::new(half_w, half_h),
            Vec2::new(-half_w, half_h),
        ],
    }
}

/// Create a rounded rectangle path centered at origin with given half-extents and corner radius.
///
/// Traces the outline clockwise starting from the bottom-left arc end point.
/// Each corner uses a single `QuadraticTo` with the sharp corner as control point.
pub fn rounded_rect_path(half_w: f32, half_h: f32, radius: f32) -> ShapeVariant {
    let r = radius;
    let (l, ri, b, t) = (-half_w, half_w, -half_h, half_h);

    ShapeVariant::Path {
        commands: vec![
            // Start at bottom-left arc end (moving right along bottom edge)
            PathCommand::MoveTo(Vec2::new(l + r, b)),
            // Bottom edge → bottom-right corner arc
            PathCommand::LineTo(Vec2::new(ri - r, b)),
            PathCommand::QuadraticTo {
                control: Vec2::new(ri, b),
                to: Vec2::new(ri, b + r),
            },
            // Right edge → top-right corner arc
            PathCommand::LineTo(Vec2::new(ri, t - r)),
            PathCommand::QuadraticTo {
                control: Vec2::new(ri, t),
                to: Vec2::new(ri - r, t),
            },
            // Top edge → top-left corner arc
            PathCommand::LineTo(Vec2::new(l + r, t)),
            PathCommand::QuadraticTo {
                control: Vec2::new(l, t),
                to: Vec2::new(l, t - r),
            },
            // Left edge → bottom-left corner arc
            PathCommand::LineTo(Vec2::new(l, b + r)),
            PathCommand::QuadraticTo {
                control: Vec2::new(l, b),
                to: Vec2::new(l + r, b),
            },
            PathCommand::Close,
        ],
    }
}

/// Generate rect vertices with top-left corner at (x, y) and size w × h.
/// For non-shader shapes drawn with `IDENTITY_MODEL`.
pub fn rect_vertices(x: f32, y: f32, w: f32, h: f32) -> [[f32; 2]; 4] {
    [[x, y], [x + w, y], [x + w, y + h], [x, y + h]]
}
