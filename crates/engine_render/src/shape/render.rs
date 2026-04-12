// EVOLVE-BLOCK-START
use glam::Vec2;

use super::components::ShapeVariant;
use super::tessellate::shape_aabb;

#[inline]
pub fn affine2_to_mat4(affine: &glam::Affine2) -> [[f32; 4]; 4] {
    let m = affine.matrix2;
    let t = affine.translation;
    [
        [m.x_axis.x, m.x_axis.y, 0.0, 0.0],
        [m.y_axis.x, m.y_axis.y, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [t.x, t.y, 0.0, 1.0],
    ]
}

#[inline]
pub fn is_shape_culled(pos: Vec2, variant: &ShapeVariant, view_rect: Option<(Vec2, Vec2)>) -> bool {
    let Some((view_min, view_max)) = view_rect else {
        return false;
    };

    let (local_min, local_max) = shape_aabb(variant);
    let min = local_min + pos;
    let max = local_max + pos;

    max.x < view_min.x || min.x > view_max.x || max.y < view_min.y || min.y > view_max.y
}
// EVOLVE-BLOCK-END
