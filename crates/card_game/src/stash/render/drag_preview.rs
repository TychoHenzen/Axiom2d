use std::collections::HashMap;

use engine_core::color::Color;
use engine_render::prelude::{Camera2D, QUAD_INDICES, ShaderHandle, UNIT_QUAD, unit_quad_model};

use crate::card::interaction::drag_state::DragInfo;
use crate::card::rendering::baked_mesh::BakedCardMesh;
use crate::card::rendering::geometry::{ART_QUAD, art_quad_model};
use crate::stash::constants::{GRID_MARGIN, SLOT_COLOR, SLOT_GAP, SLOT_STRIDE_H, SLOT_STRIDE_W};
use crate::stash::grid::StashGrid;
use bevy_ecs::prelude::Entity;
use glam::Vec2;

pub(super) fn draw_drag_preview(
    renderer: &mut dyn engine_render::prelude::Renderer,
    camera: &Camera2D,
    grid: &StashGrid,
    drag_info: DragInfo,
    mouse_screen_pos: Vec2,
    mouse_world_pos: Vec2,
    icon_colors: &HashMap<Entity, Color>,
    baked_meshes: &HashMap<Entity, &BakedCardMesh>,
    renderer_art_shader: Option<ShaderHandle>,
) {
    let world_slot_w = crate::stash::constants::SLOT_WIDTH / camera.zoom;
    let world_slot_h = crate::stash::constants::SLOT_HEIGHT / camera.zoom;

    let screen = mouse_screen_pos;
    let bg_x_max = GRID_MARGIN + f32::from(grid.width()) * SLOT_STRIDE_W - SLOT_GAP;
    let bg_y_max = GRID_MARGIN + f32::from(grid.height()) * SLOT_STRIDE_H - SLOT_GAP;
    let over_stash_area = screen.x >= GRID_MARGIN
        && screen.x < bg_x_max
        && screen.y >= GRID_MARGIN
        && screen.y < bg_y_max;

    if over_stash_area {
        let cursor_world = mouse_world_pos;
        if let Some(baked) = baked_meshes.get(&drag_info.entity) {
            let model =
                super::models::miniature_card_model(camera.zoom, cursor_world.x, cursor_world.y);
            renderer.set_shader(ShaderHandle(0));
            renderer.draw_colored_mesh(&baked.front.vertices, &baked.front.indices, model);
        } else {
            let color = icon_colors
                .get(&drag_info.entity)
                .copied()
                .unwrap_or(SLOT_COLOR);
            if let Some(art) = renderer_art_shader {
                renderer.set_shader(art);
                let model =
                    art_quad_model(world_slot_w, world_slot_h, cursor_world.x, cursor_world.y);
                renderer.draw_shape(&ART_QUAD, &QUAD_INDICES, color, model);
            } else {
                let model =
                    unit_quad_model(world_slot_w, world_slot_h, cursor_world.x, cursor_world.y);
                renderer.draw_shape(&UNIT_QUAD, &QUAD_INDICES, color, model);
            }
            renderer.set_shader(ShaderHandle(0));
        }
    }
}
