use bevy_ecs::prelude::{Query, Res, ResMut};
use engine_core::color::Color;
use engine_input::prelude::MouseState;
use engine_render::prelude::{Camera2D, RendererRes, screen_to_world};
use glam::Vec2;

use crate::stash::constants::{
    BACKGROUND_COLOR, GRID_MARGIN, SLOT_GAP, SLOT_STRIDE_H, SLOT_STRIDE_W,
};
use crate::stash::grid::StashGrid;
use crate::stash::toggle::StashVisible;
use engine_render::prelude::resolve_viewport_camera;
use engine_render::prelude::{QUAD_INDICES, rect_vertices};

pub const TAB_WIDTH: f32 = 30.0;
pub const TAB_HEIGHT: f32 = 16.0;
pub const TAB_GAP: f32 = 4.0;
pub const TAB_ACTIVE: Color = Color {
    r: 0.35,
    g: 0.35,
    b: 0.35,
    a: 1.0,
};
pub const TAB_INACTIVE: Color = BACKGROUND_COLOR;

/// Returns the screen-space Y position of the top edge of the tab row.
pub fn tab_row_top_y(grid_height: u8) -> f32 {
    GRID_MARGIN + f32::from(grid_height) * SLOT_STRIDE_H - SLOT_GAP
}

/// Returns the screen-space X position of the left edge of tab `i` in a row of `page_count` tabs,
/// centered under a grid of width `grid_width`.
pub fn tab_left_x(grid_width: u8, page_count: u8, tab_index: u8) -> f32 {
    let tab_stride = TAB_WIDTH + TAB_GAP;
    let total_width = f32::from(page_count) * tab_stride - TAB_GAP;
    let grid_screen_w = f32::from(grid_width) * SLOT_STRIDE_W - SLOT_GAP;
    let grid_center_x = GRID_MARGIN + grid_screen_w / 2.0;
    let tabs_start_x = grid_center_x - total_width / 2.0;
    tabs_start_x + f32::from(tab_index) * tab_stride
}

pub fn stash_tab_click_system(
    mouse: Res<MouseState>,
    visible: Res<StashVisible>,
    mut grid: ResMut<StashGrid>,
) {
    if !visible.0 {
        return;
    }
    if !mouse.just_pressed(engine_input::prelude::MouseButton::Left) {
        return;
    }

    let screen = mouse.screen_pos();
    let top_y = tab_row_top_y(grid.height());
    let bottom_y = top_y + TAB_HEIGHT;

    if screen.y < top_y || screen.y > bottom_y {
        return;
    }

    let page_count = grid.page_count();
    for i in 0..page_count {
        let left = tab_left_x(grid.width(), page_count, i);
        let right = left + TAB_WIDTH;
        if screen.x >= left && screen.x <= right {
            grid.set_current_page(i);
            return;
        }
    }
}

pub fn stash_tab_render_system(
    grid: Res<StashGrid>,
    visible: Res<StashVisible>,
    camera_query: Query<&Camera2D>,
    mut renderer: ResMut<RendererRes>,
) {
    if !visible.0 {
        return;
    }
    let Some((vw, vh, camera)) = resolve_viewport_camera(&renderer, &camera_query) else {
        return;
    };

    let page_count = grid.page_count();
    let current = grid.current_page();
    let top_y = tab_row_top_y(grid.height());

    for i in 0..page_count {
        let left_x = tab_left_x(grid.width(), page_count, i);
        let color = if i == current {
            TAB_ACTIVE
        } else {
            TAB_INACTIVE
        };
        let origin = screen_to_world(Vec2::new(left_x, top_y), &camera, vw, vh);
        let tab_w = TAB_WIDTH / camera.zoom;
        let tab_h = TAB_HEIGHT / camera.zoom;
        let verts = rect_vertices(origin.x, origin.y, tab_w, tab_h);
        renderer.draw_shape(
            &verts,
            &QUAD_INDICES,
            color,
            engine_render::prelude::IDENTITY_MODEL,
        );
    }
}
