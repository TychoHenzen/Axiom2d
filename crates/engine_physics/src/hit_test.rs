use glam::Vec2;

use crate::collider::Collider;

pub fn collider_half_extents(collider: &Collider) -> Option<Vec2> {
    match collider {
        Collider::Aabb(half) => Some(*half),
        _ => None,
    }
}

pub fn local_space_hit(cursor_local: Vec2, half: Vec2) -> bool {
    cursor_local.x.abs() <= half.x && cursor_local.y.abs() <= half.y
}
