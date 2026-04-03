use bevy_ecs::prelude::{Query, Res, ResMut};
use engine_core::color::Color;
use engine_input::prelude::MouseState;
use engine_render::font::measure_text;
use engine_render::prelude::{Camera2D, RendererRes, screen_to_world};
use glam::Vec2;

use crate::stash::constants::{
    BACKGROUND_COLOR, GRID_MARGIN, SLOT_GAP, SLOT_STRIDE_H, SLOT_STRIDE_W,
};
use crate::stash::grid::StashGrid;
use crate::stash::store::{StoreWallet, storage_tab_purchase_cost};
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
pub const TAB_ACTION: Color = Color {
    r: 0.34,
    g: 0.30,
    b: 0.24,
    a: 1.0,
};
const TAB_LABEL_COLOR: Color = Color {
    r: 0.94,
    g: 0.90,
    b: 0.80,
    a: 1.0,
};

fn draw_centered_screen_text(
    renderer: &mut dyn engine_render::renderer::Renderer,
    camera: &Camera2D,
    viewport_w: f32,
    viewport_h: f32,
    text: &str,
    center_x: f32,
    y: f32,
    font_size: f32,
    color: Color,
) {
    let width = measure_text(text, font_size);
    let world_pos = screen_to_world(
        Vec2::new(center_x - width * 0.5, y),
        camera,
        viewport_w,
        viewport_h,
    );
    renderer.draw_text(
        text,
        world_pos.x,
        world_pos.y,
        font_size / camera.zoom,
        color,
    );
}

/// Returns the screen-space Y position of the top edge of the tab row.
pub fn tab_row_top_y(grid_height: u8) -> f32 {
    GRID_MARGIN + f32::from(grid_height) * SLOT_STRIDE_H - SLOT_GAP
}

/// Returns the screen-space X position of the left edge of tab `i` in a row of `tab_count` tabs,
/// centered under a grid of width `grid_width`.
pub fn tab_left_x(grid_width: u8, tab_count: u8, tab_index: u8) -> f32 {
    let tab_stride = TAB_WIDTH + TAB_GAP;
    let total_width = f32::from(tab_count) * tab_stride - TAB_GAP;
    let grid_screen_w = f32::from(grid_width) * SLOT_STRIDE_W - SLOT_GAP;
    let grid_center_x = GRID_MARGIN + grid_screen_w / 2.0;
    let tabs_start_x = grid_center_x - total_width / 2.0;
    tabs_start_x + f32::from(tab_index) * tab_stride
}

/// Returns `true` if the cursor is within the stash UI region, including the tab row.
pub fn stash_ui_contains(screen_pos: Vec2, grid: &StashGrid) -> bool {
    let left = GRID_MARGIN;
    let right = GRID_MARGIN + f32::from(grid.width()) * SLOT_STRIDE_W - SLOT_GAP;
    let top = GRID_MARGIN;
    let bottom = tab_row_top_y(grid.height()) + TAB_HEIGHT;
    screen_pos.x >= left && screen_pos.x <= right && screen_pos.y >= top && screen_pos.y <= bottom
}

pub fn stash_tab_click_system(
    mouse: Res<MouseState>,
    visible: Res<StashVisible>,
    mut grid: ResMut<StashGrid>,
    mut wallet: ResMut<StoreWallet>,
) {
    if !visible.0 {
        return;
    }
    if !mouse.just_pressed(engine_input::prelude::MouseButton::Left) {
        return;
    }

    let screen = mouse.screen_pos();
    if !stash_ui_contains(screen, &grid) {
        return;
    }

    let top_y = tab_row_top_y(grid.height());
    let bottom_y = top_y + TAB_HEIGHT;
    if screen.y < top_y || screen.y > bottom_y {
        return;
    }

    let tab_count = grid.tab_count();
    for i in 0..tab_count {
        let left = tab_left_x(grid.width(), tab_count, i);
        let right = left + TAB_WIDTH;
        if screen.x < left || screen.x > right {
            continue;
        }

        if i == 0 {
            grid.set_current_page(0);
            return;
        }

        if i == tab_count - 1 {
            let cost = storage_tab_purchase_cost(grid.page_count());
            if wallet.spend(cost) {
                let new_storage_count = grid.add_storage_tab();
                grid.set_current_page(new_storage_count);
            }
            return;
        }

        grid.set_current_page(i);
        return;
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

    let tab_count = grid.tab_count();
    let current = grid.current_page();
    let top_y = tab_row_top_y(grid.height());

    for i in 0..tab_count {
        let left_x = tab_left_x(grid.width(), tab_count, i);
        let color = if i == current {
            TAB_ACTIVE
        } else if i == tab_count - 1 {
            TAB_ACTION
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

        let label = match i {
            0 => "$".to_owned(),
            _ if i == tab_count - 1 => "+".to_owned(),
            _ => i.to_string(),
        };
        let label_font = 12.0;
        draw_centered_screen_text(
            &mut **renderer,
            &camera,
            vw,
            vh,
            &label,
            left_x + TAB_WIDTH * 0.5,
            top_y + TAB_HEIGHT * 0.5 - label_font * 0.45,
            label_font,
            TAB_LABEL_COLOR,
        );
    }
}
