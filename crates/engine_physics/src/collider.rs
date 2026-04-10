// EVOLVE-BLOCK-START
use bevy_ecs::prelude::Component;
use glam::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Collider {
    Circle(f32),
    Aabb(Vec2),
    ConvexPolygon(Vec<Vec2>),
}
// EVOLVE-BLOCK-END
