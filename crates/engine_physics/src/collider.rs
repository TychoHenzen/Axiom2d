use bevy_ecs::prelude::Component;
use glam::Vec2;

#[derive(Component, Debug, Clone, PartialEq)]
pub enum Collider {
    Circle(f32),
    Aabb(Vec2),
    ConvexPolygon(Vec<Vec2>),
}
