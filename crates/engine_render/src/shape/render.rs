use glam::Vec2;

use super::components::ShapeVariant;
use super::tessellate::shape_aabb;
use crate::culling::aabb_intersects_view_rect;

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

pub fn is_shape_culled(pos: Vec2, variant: &ShapeVariant, view_rect: Option<(Vec2, Vec2)>) -> bool {
    let Some((view_min, view_max)) = view_rect else {
        return false;
    };
    let (local_min, local_max) = shape_aabb(variant);
    let r = local_min.abs().max(local_max.abs()).length();
    let entity_min = Vec2::new(pos.x - r, pos.y - r);
    let entity_max = Vec2::new(pos.x + r, pos.y + r);
    !aabb_intersects_view_rect(entity_min, entity_max, view_min, view_max)
}
