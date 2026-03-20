use super::ShapeVariant;
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

/// Generate rect vertices with top-left corner at (x, y) and size w × h.
/// For non-shader shapes drawn with `IDENTITY_MODEL`.
pub fn rect_vertices(x: f32, y: f32, w: f32, h: f32) -> [[f32; 2]; 4] {
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
}
