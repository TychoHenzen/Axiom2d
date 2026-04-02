use bevy_ecs::prelude::{Component, Query, ResMut};
use engine_core::color::Color;
use engine_core::types::{Pixels, TextureId};
use engine_scene::prelude::{EffectiveVisibility, GlobalTransform2D, RenderLayer, SortOrder};
use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::camera::Camera2D;
use crate::culling::{aabb_intersects_view_rect, compute_view_rect};
use crate::material::{Material2d, apply_material};
use crate::rect::Rect;
use crate::renderer::RendererRes;

#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Sprite {
    pub texture: TextureId,
    pub uv_rect: [f32; 4],
    pub color: Color,
    pub width: Pixels,
    pub height: Pixels,
}

fn is_sprite_culled(sprite: &Sprite, pos: Vec2, view_rect: Option<(Vec2, Vec2)>) -> bool {
    let Some((view_min, view_max)) = view_rect else {
        return false;
    };
    let entity_min = Vec2::new(pos.x, pos.y);
    let entity_max = Vec2::new(pos.x + sprite.width.0, pos.y + sprite.height.0);
    !aabb_intersects_view_rect(entity_min, entity_max, view_min, view_max)
}

#[allow(clippy::type_complexity)]
pub fn sprite_render_system(
    query: Query<(
        &Sprite,
        &GlobalTransform2D,
        Option<&RenderLayer>,
        Option<&SortOrder>,
        Option<&EffectiveVisibility>,
        Option<&Material2d>,
    )>,
    camera_query: Query<&Camera2D>,
    mut renderer: ResMut<RendererRes>,
) {
    let view_rect = compute_view_rect(&camera_query, &renderer);
    let mut sprites: Vec<_> = query.iter().filter(|t| t.4.is_none_or(|v| v.0)).collect();
    sprites.sort_by_key(|t| {
        (
            t.2.copied().unwrap_or(RenderLayer::World),
            t.3.copied().unwrap_or_default(),
        )
    });
    let mut last_shader = None;
    let mut last_blend_mode = None;
    for (sprite, transform, _, _, _, mat) in sprites {
        let pos = transform.0.translation;
        if is_sprite_culled(sprite, pos, view_rect) {
            continue;
        }
        apply_material(&mut **renderer, mat, &mut last_shader, &mut last_blend_mode);
        let rect = Rect {
            x: Pixels(pos.x),
            y: Pixels(pos.y),
            width: sprite.width,
            height: sprite.height,
            color: sprite.color,
        };
        renderer.draw_sprite(rect, sprite.uv_rect);
    }
}
