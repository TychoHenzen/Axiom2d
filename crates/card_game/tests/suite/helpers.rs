use engine_physics::prelude::Collider;
use glam::Vec2;

pub fn default_card_collider() -> Collider {
    Collider::Aabb(Vec2::new(30.0, 45.0))
}
