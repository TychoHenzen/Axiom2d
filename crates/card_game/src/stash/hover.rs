use bevy_ecs::prelude::{Entity, Local, Query, Res, ResMut, Resource};
use engine_core::prelude::DeltaTime;
use engine_input::prelude::{InputState, KeyCode, MouseState};
use engine_render::prelude::{Camera2D, RendererRes, resolve_viewport_camera, screen_to_world};
use engine_scene::prelude::{RenderLayer, SortOrder};
use engine_ui::draw_command::{DrawCommand, DrawQueue, OverlayCommand};
use glam::Vec2;

use crate::card::interaction::drag_state::DragState;
use crate::card::rendering::baked_mesh::BakedCardMesh;
use crate::card::rendering::geometry::{TABLE_CARD_HEIGHT, TABLE_CARD_WIDTH};
use crate::stash::constants::{GRID_MARGIN, SLOT_STRIDE_H};
use crate::stash::grid::{StashGrid, find_stash_slot_at};
use crate::stash::toggle::StashVisible;
use engine_render::shape::MeshOverlays;

pub const ORBIT_PERIOD_X: f32 = 3.8;
pub const ORBIT_PERIOD_Y: f32 = 4.0;
pub const ORBIT_AMPLITUDE: f32 = 0.8;

/// Lissajous curve offset for the fake shader cursor on the hover preview.
/// X oscillates at 3.8s, Y at 4.0s — the 19:20 ratio creates a slowly
/// morphing pattern that cycles through N, V, and O-like shapes.
pub fn lissajous_offset(time: f32, half_w: f32, half_h: f32) -> Vec2 {
    let omega_x = std::f32::consts::TAU / ORBIT_PERIOD_X;
    let omega_y = std::f32::consts::TAU / ORBIT_PERIOD_Y;
    Vec2::new(
        half_w * ORBIT_AMPLITUDE * (omega_x * time).sin(),
        half_h * ORBIT_AMPLITUDE * (omega_y * time).sin(),
    )
}

#[derive(Resource, Debug, Default)]
pub struct StashHoverPreview {
    pub hovered_entity: Option<Entity>,
}

pub fn stash_hover_preview_system(
    stash_visible: Res<StashVisible>,
    input: Res<InputState>,
    mouse: Res<MouseState>,
    grid: Res<StashGrid>,
    drag_state: Res<DragState>,
    mut hover_preview: ResMut<StashHoverPreview>,
) {
    let ctrl_held = input.pressed(KeyCode::ControlLeft) || input.pressed(KeyCode::ControlRight);

    let hovered = stash_visible
        .0
        .then_some(())
        .filter(|()| ctrl_held)
        .filter(|()| drag_state.dragging.is_none())
        .filter(|()| grid.current_storage_page().is_some())
        .and_then(|()| find_stash_slot_at(mouse.screen_pos(), grid.width(), grid.height()))
        .and_then(|(col, row)| {
            grid.get(grid.current_storage_page().unwrap_or(0), col, row)
                .copied()
        });

    hover_preview.hovered_entity = hovered;
}

/// Renders a scaled preview of a hovered stash card using its baked front mesh.
/// Shader overlays receive a fake Lissajous cursor that slowly orbits the preview,
/// animating pointer-reactive effects (glossy glint, foil rainbow, embossed bevel).
pub fn stash_hover_preview_render_system(
    hover_preview: Res<StashHoverPreview>,
    grid: Res<StashGrid>,
    dt: Res<DeltaTime>,
    camera_query: Query<&Camera2D>,
    baked_query: Query<(&BakedCardMesh, Option<&MeshOverlays>)>,
    renderer: Res<RendererRes>,
    mut draw_queue: ResMut<DrawQueue>,
    mut elapsed: Local<f32>,
) {
    *elapsed += dt.0.0;

    let Some(hovered_entity) = hover_preview.hovered_entity else {
        return;
    };

    let Some((vw, vh, camera)) = resolve_viewport_camera(&renderer, &camera_query) else {
        return;
    };

    let Ok((baked, overlays)) = baked_query.get(hovered_entity) else {
        return;
    };

    let preview_screen_h = f32::from(grid.height()) * SLOT_STRIDE_H;
    let preview_screen_w = preview_screen_h * (TABLE_CARD_WIDTH / TABLE_CARD_HEIGHT);

    let preview_center_screen = Vec2::new(
        vw - GRID_MARGIN - preview_screen_w * 0.5,
        GRID_MARGIN + preview_screen_h * 0.5,
    );
    let preview_center = screen_to_world(preview_center_screen, &camera, vw, vh);

    let scale_x = (preview_screen_w / camera.zoom) / TABLE_CARD_WIDTH;
    let scale_y = (preview_screen_h / camera.zoom) / TABLE_CARD_HEIGHT;

    if !baked.front.is_empty() {
        let model = [
            [scale_x, 0.0, 0.0, 0.0],
            [0.0, scale_y, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [preview_center.x, preview_center.y, 0.0, 1.0],
        ];

        // Build overlay commands with modified uniforms for fake pointer
        let overlay_cmds = if let Some(overlays) = overlays {
            let half_w_world = preview_screen_w / (2.0 * camera.zoom);
            let half_h_world = preview_screen_h / (2.0 * camera.zoom);
            let orbit = lissajous_offset(*elapsed, half_w_world, half_h_world);
            let fake_ptr = preview_center + orbit;

            overlays
                .0
                .iter()
                .filter(|e| e.visible)
                .map(|entry| {
                    let mut mat = entry.material.clone();
                    if mat.uniforms.len() >= 16 {
                        mat.uniforms[8..12].copy_from_slice(&fake_ptr.x.to_le_bytes());
                        mat.uniforms[12..16].copy_from_slice(&fake_ptr.y.to_le_bytes());
                    }
                    OverlayCommand {
                        mesh: entry.mesh.clone(),
                        material: mat,
                    }
                })
                .collect()
        } else {
            vec![]
        };

        draw_queue.push(
            RenderLayer::UI,
            SortOrder::new(400),
            DrawCommand::ColorMesh {
                mesh: baked.front.clone(),
                model,
                material: None,
                overlays: overlay_cmds,
            },
        );
    }
}
