use bevy_ecs::prelude::{Query, Res, ResMut};
use engine_core::prelude::Transform2D;
use engine_core::profiler::FrameProfiler;
use engine_render::prelude::{Camera2D, RendererRes, screen_to_world};
use glam::Vec2;

use crate::card::component::CardZone;
use crate::stash::constants::{GRID_MARGIN, SLOT_HEIGHT, SLOT_STRIDE_H, SLOT_STRIDE_W, SLOT_WIDTH};
use crate::stash::grid::StashGrid;
use engine_render::prelude::resolve_viewport_camera;

pub fn stash_layout_system(
    grid: Res<StashGrid>,
    camera_query: Query<&Camera2D>,
    renderer: Res<RendererRes>,
    mut card_query: Query<(&CardZone, &mut Transform2D)>,
    mut profiler: Option<ResMut<FrameProfiler>>,
) {
    let _span = profiler.as_deref_mut().map(|p| p.span("stash_layout"));
    let Some((vw, vh, camera)) = resolve_viewport_camera(&renderer, &camera_query) else {
        return;
    };

    let Some(page) = grid.current_storage_page() else {
        return;
    };
    for (zone, mut transform) in &mut card_query {
        if let CardZone::Stash {
            page: card_page,
            col,
            row,
        } = zone
        {
            if *card_page != page {
                continue;
            }
            let screen_x = GRID_MARGIN + f32::from(*col) * SLOT_STRIDE_W + SLOT_WIDTH * 0.5;
            let screen_y = GRID_MARGIN + f32::from(*row) * SLOT_STRIDE_H + SLOT_HEIGHT * 0.5;
            transform.position = screen_to_world(Vec2::new(screen_x, screen_y), &camera, vw, vh);
        }
    }
}
