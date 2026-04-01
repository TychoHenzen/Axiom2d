use std::collections::HashMap;

use engine_core::color::Color;
use engine_render::prelude::{
    Camera2D, QUAD_INDICES, ShaderHandle, UNIT_QUAD, screen_to_world, unit_quad_model,
};
use glam::Vec2;

use crate::card::rendering::baked_mesh::BakedCardMesh;
use crate::card::rendering::geometry::{ART_QUAD, art_quad_model};
use crate::stash::constants::{
    GRID_MARGIN, SLOT_COLOR, SLOT_HEIGHT, SLOT_HIGHLIGHT_COLOR, SLOT_STRIDE_H, SLOT_STRIDE_W,
    SLOT_WIDTH,
};
use crate::stash::grid::StashGrid;
use bevy_ecs::prelude::Entity;

pub(super) fn draw_slots(
    renderer: &mut dyn engine_render::prelude::Renderer,
    camera: &Camera2D,
    vw: f32,
    vh: f32,
    grid: &StashGrid,
    page: u8,
    icon_colors: &HashMap<Entity, Color>,
    baked_meshes: &HashMap<Entity, &BakedCardMesh>,
    highlight_slot: Option<(u8, u8)>,
    renderer_art_shader: Option<ShaderHandle>,
) {
    let world_slot_w = SLOT_WIDTH / camera.zoom;
    let world_slot_h = SLOT_HEIGHT / camera.zoom;

    for col in 0..grid.width() {
        for row in 0..grid.height() {
            let screen_x = GRID_MARGIN + f32::from(col) * SLOT_STRIDE_W;
            let screen_y = GRID_MARGIN + f32::from(row) * SLOT_STRIDE_H;
            let center = screen_to_world(
                Vec2::new(screen_x + SLOT_WIDTH * 0.5, screen_y + SLOT_HEIGHT * 0.5),
                camera,
                vw,
                vh,
            );

            if let Some(&entity) = grid.get(page, col, row) {
                if let Some(baked) = baked_meshes.get(&entity) {
                    let model =
                        super::models::miniature_card_model(camera.zoom, center.x, center.y);
                    renderer.set_shader(ShaderHandle(0));
                    renderer.draw_colored_mesh(&baked.front.vertices, &baked.front.indices, model);
                } else {
                    let color = icon_colors.get(&entity).copied().unwrap_or(SLOT_COLOR);
                    if let Some(art) = renderer_art_shader {
                        renderer.set_shader(art);
                        let model = art_quad_model(world_slot_w, world_slot_h, center.x, center.y);
                        renderer.draw_shape(&ART_QUAD, &QUAD_INDICES, color, model);
                    } else {
                        let model = unit_quad_model(world_slot_w, world_slot_h, center.x, center.y);
                        renderer.draw_shape(&UNIT_QUAD, &QUAD_INDICES, color, model);
                    }
                    renderer.set_shader(ShaderHandle(0));
                }
            } else {
                let slot_color = if highlight_slot == Some((col, row)) {
                    SLOT_HIGHLIGHT_COLOR
                } else {
                    SLOT_COLOR
                };
                let model = unit_quad_model(world_slot_w, world_slot_h, center.x, center.y);
                renderer.draw_shape(&UNIT_QUAD, &QUAD_INDICES, slot_color, model);
            }
        }
    }
}
