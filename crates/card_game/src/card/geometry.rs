use engine_render::prelude::ShapeVariant;
use glam::Vec2;

pub const TABLE_CARD_WIDTH: f32 = 60.0;
pub const TABLE_CARD_HEIGHT: f32 = 90.0;
pub const TABLE_CARD_SIZE: Vec2 = Vec2::new(TABLE_CARD_WIDTH, TABLE_CARD_HEIGHT);

/// Normalized unit quad with vertices in \[-0.5, 0.5\].
///
/// When drawn with a model matrix from [`unit_quad_model`], the model scale
/// directly encodes the world-space width and height — no intermediate
/// coordinate space. UV mapping in the shader is `local_pos + 0.5`.
pub(crate) const UNIT_QUAD: [[f32; 2]; 4] = [[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]];

pub(crate) const QUAD_INDICES: [u32; 6] = [0, 1, 2, 0, 2, 3];

/// Half-extents of the card art shader's fixed coordinate space.
/// The WGSL shader (`uv_gradient.wgsl`) hardcodes these values for UV mapping:
/// `uv = local_pos / (half_size * 2) + 0.5`. All vertices passed to `draw_shape`
/// when the art shader is active MUST be in this range.
pub(crate) const ART_HALF_W: f32 = 27.0;
pub(crate) const ART_HALF_H: f32 = 22.5;

/// Quad vertices matching the art shader's expected coordinate space.
/// Use this (not `UNIT_QUAD`) when the card art shader is active.
pub(crate) const ART_QUAD: [[f32; 2]; 4] = [
    [-ART_HALF_W, -ART_HALF_H],
    [ART_HALF_W, -ART_HALF_H],
    [ART_HALF_W, ART_HALF_H],
    [-ART_HALF_W, ART_HALF_H],
];

/// Build a model matrix that scales an [`ART_QUAD`] to the given world-space
/// width × height and translates to (cx, cy).
pub(crate) fn art_quad_model(world_w: f32, world_h: f32, cx: f32, cy: f32) -> [[f32; 4]; 4] {
    let sx = world_w / (ART_HALF_W * 2.0);
    let sy = world_h / (ART_HALF_H * 2.0);
    [
        [sx, 0.0, 0.0, 0.0],
        [0.0, sy, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [cx, cy, 0.0, 1.0],
    ]
}

/// Create a rectangular polygon centered at origin with given half-extents.
pub(crate) fn rect_polygon(half_w: f32, half_h: f32) -> ShapeVariant {
    ShapeVariant::Polygon {
        points: vec![
            Vec2::new(-half_w, -half_h),
            Vec2::new(half_w, -half_h),
            Vec2::new(half_w, half_h),
            Vec2::new(-half_w, half_h),
        ],
    }
}

/// Build a model matrix that scales a \[-0.5, 0.5\] unit quad to the given
/// width × height and translates to (cx, cy).
pub(crate) fn unit_quad_model(width: f32, height: f32, cx: f32, cy: f32) -> [[f32; 4]; 4] {
    [
        [width, 0.0, 0.0, 0.0],
        [0.0, height, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [cx, cy, 0.0, 1.0],
    ]
}

/// Generate rect vertices with top-left corner at (x, y) and size w × h.
/// For non-shader shapes drawn with `IDENTITY_MODEL`.
pub(crate) fn rect_vertices(x: f32, y: f32, w: f32, h: f32) -> [[f32; 2]; 4] {
    [[x, y], [x + w, y], [x + w, y + h], [x, y + h]]
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn when_unit_quad_then_vertices_span_one() {
        let w = UNIT_QUAD[1][0] - UNIT_QUAD[0][0];
        let h = UNIT_QUAD[3][1] - UNIT_QUAD[0][1];
        assert!((w - 1.0).abs() < 1e-6, "width={w}");
        assert!((h - 1.0).abs() < 1e-6, "height={h}");
    }

    #[test]
    fn when_unit_quad_model_then_matrix_scales_and_translates() {
        // Act
        let m = unit_quad_model(100.0, 200.0, 10.0, 20.0);

        // Assert
        assert_eq!(m[0][0], 100.0, "scale x");
        assert_eq!(m[1][1], 200.0, "scale y");
        assert_eq!(m[3][0], 10.0, "translate x");
        assert_eq!(m[3][1], 20.0, "translate y");
        assert_eq!(m[2][2], 1.0, "z identity");
        assert_eq!(m[3][3], 1.0, "w");
    }

    #[test]
    fn when_rect_polygon_then_half_extents_match() {
        let ShapeVariant::Polygon { ref points } = rect_polygon(30.0, 45.0) else {
            panic!("expected Polygon");
        };
        let max_x = points.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
        let max_y = points.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
        assert!((max_x - 30.0).abs() < 1e-6);
        assert!((max_y - 45.0).abs() < 1e-6);
    }

    #[test]
    fn when_rect_vertices_then_corners_at_expected_positions() {
        let verts = rect_vertices(10.0, 20.0, 30.0, 40.0);
        assert_eq!(verts[0], [10.0, 20.0]);
        assert_eq!(verts[1], [40.0, 20.0]);
        assert_eq!(verts[2], [40.0, 60.0]);
        assert_eq!(verts[3], [10.0, 60.0]);
    }

    #[test]
    fn when_table_card_size_then_matches_width_height() {
        assert_eq!(TABLE_CARD_SIZE.x, TABLE_CARD_WIDTH);
        assert_eq!(TABLE_CARD_SIZE.y, TABLE_CARD_HEIGHT);
    }
}
