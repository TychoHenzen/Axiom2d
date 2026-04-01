use std::collections::HashMap;

use bevy_ecs::prelude::{Entity, Query, Res, ResMut};
use bevy_ecs::system::SystemParam;
use engine_core::color::Color;
use engine_render::prelude::{Camera2D, QUAD_INDICES, RendererRes, rect_vertices, screen_to_world};
use glam::Vec2;

use crate::card::component::Card;
use crate::card::identity::signature_profile::SignatureProfile;
use crate::card::identity::visual_params::generate_card_visuals;
use crate::card::rendering::baked_mesh::BakedCardMesh;
use crate::stash::constants::{
    BACKGROUND_COLOR, GRID_MARGIN, SLOT_GAP, SLOT_STRIDE_H, SLOT_STRIDE_W,
};
use crate::stash::grid::{StashGrid, find_stash_slot_at};
use crate::stash::toggle::StashVisible;
use engine_render::prelude::resolve_viewport_camera;

mod drag_preview;
mod helpers;
mod models;
mod slots;

#[derive(SystemParam)]
pub struct StashRenderParams<'w> {
    grid: Res<'w, StashGrid>,
    visible: Res<'w, StashVisible>,
    drag_state: Res<'w, crate::card::interaction::drag_state::DragState>,
    mouse: Res<'w, engine_input::prelude::MouseState>,
    art_shader: Option<Res<'w, crate::card::rendering::art_shader::CardArtShader>>,
}

pub(crate) use helpers::reset_default_shader;

pub fn stash_render_system(
    params: StashRenderParams,
    card_query: Query<(Entity, &Card, Option<&BakedCardMesh>)>,
    camera_query: Query<&Camera2D>,
    mut renderer: ResMut<RendererRes>,
) {
    let renderer_art_shader = params.art_shader.map(|s| s.0);
    if !params.visible.0 {
        return;
    }

    let Some((vw, vh, camera)) = resolve_viewport_camera(&renderer, &camera_query) else {
        return;
    };

    reset_default_shader(&mut **renderer);

    let bg_screen_w = f32::from(params.grid.width()) * SLOT_STRIDE_W - SLOT_GAP;
    let bg_screen_h = f32::from(params.grid.height()) * SLOT_STRIDE_H - SLOT_GAP;
    let bg_origin = screen_to_world(Vec2::new(GRID_MARGIN, GRID_MARGIN), &camera, vw, vh);
    let bg_verts = rect_vertices(
        bg_origin.x,
        bg_origin.y,
        bg_screen_w / camera.zoom,
        bg_screen_h / camera.zoom,
    );
    renderer.draw_shape(
        &bg_verts,
        &QUAD_INDICES,
        BACKGROUND_COLOR,
        engine_render::prelude::IDENTITY_MODEL,
    );

    let icon_colors: HashMap<Entity, Color> = card_query
        .iter()
        .map(|(entity, card, _baked)| {
            let profile = SignatureProfile::without_archetype(&card.signature);
            (
                entity,
                generate_card_visuals(&card.signature, &profile).art_color,
            )
        })
        .collect();

    let baked_meshes: HashMap<Entity, &BakedCardMesh> = card_query
        .iter()
        .filter_map(|(entity, _card, baked)| baked.map(|b| (entity, b)))
        .collect();

    let page = params.grid.current_page();

    let highlight_slot = if params.drag_state.dragging.is_some() {
        find_stash_slot_at(
            params.mouse.screen_pos(),
            params.grid.width(),
            params.grid.height(),
        )
    } else {
        None
    };

    slots::draw_slots(
        &mut **renderer,
        &camera,
        vw,
        vh,
        &params.grid,
        page,
        &icon_colors,
        &baked_meshes,
        highlight_slot,
        renderer_art_shader,
    );

    // Draw the dragged card's icon on top of the stash grid at the cursor position
    if let Some(info) = params.drag_state.dragging {
        drag_preview::draw_drag_preview(
            &mut **renderer,
            &camera,
            &params.grid,
            info,
            params.mouse.screen_pos(),
            params.mouse.world_pos(),
            &icon_colors,
            &baked_meshes,
            renderer_art_shader,
        );
    }
}
