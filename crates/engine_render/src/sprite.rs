use bevy_ecs::prelude::Component;
use engine_core::color::Color;
use engine_core::types::{Pixels, TextureId};
use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::culling::aabb_intersects_view_rect;

#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Sprite {
    pub texture: TextureId,
    pub uv_rect: [f32; 4],
    pub color: Color,
    pub width: Pixels,
    pub height: Pixels,
}

pub fn is_sprite_culled(sprite: &Sprite, pos: Vec2, view_rect: Option<(Vec2, Vec2)>) -> bool {
    let Some((view_min, view_max)) = view_rect else {
        return false;
    };
    let entity_min = Vec2::new(pos.x, pos.y);
    let entity_max = Vec2::new(pos.x + sprite.width.0, pos.y + sprite.height.0);
    !aabb_intersects_view_rect(entity_min, entity_max, view_min, view_max)
}
