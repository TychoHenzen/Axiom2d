use bevy_ecs::prelude::{Query, Res, ResMut};
use engine_core::color::Color;
use engine_render::font::measure_text;
use engine_render::prelude::{
    Camera2D, QUAD_INDICES, RendererRes, rect_vertices, resolve_viewport_camera, screen_to_world,
};
use engine_render::shape::TessellatedMesh;
use engine_scene::prelude::{RenderLayer, SortOrder};
use engine_ui::draw_command::{DrawCommand, DrawQueue};
use glam::Vec2;

use super::constants::GRID_MARGIN;
use super::grid::StashGrid;
use super::store::store_item_bounds;
use super::store::{
    STORE_BODY_FONT, STORE_COIN_FONT, STORE_HIGHLIGHT_COLOR, STORE_ITEM_HEIGHT, STORE_ITEM_WIDTH,
    STORE_PANEL_FILL, STORE_PREVIEW_DARK, STORE_PREVIEW_LIGHT, STORE_PREVIEW_MID,
    STORE_PRICE_COLOR, STORE_PRICE_FONT, STORE_TEXT_COLOR, STORE_TITLE_FONT, StoreCatalog,
    StoreItemKind, StoreWallet, storage_tab_purchase_cost, store_item_layout,
};
use super::toggle::StashVisible;

fn draw_screen_rect(
    queue: &mut DrawQueue,
    layer: RenderLayer,
    order: SortOrder,
    camera: &Camera2D,
    viewport_w: f32,
    viewport_h: f32,
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    color: Color,
) {
    let origin = screen_to_world(Vec2::new(left, top), camera, viewport_w, viewport_h);
    let verts = rect_vertices(
        origin.x,
        origin.y,
        width / camera.zoom,
        height / camera.zoom,
    );
    queue.push(
        layer,
        order,
        DrawCommand::Shape {
            mesh: TessellatedMesh {
                vertices: verts.to_vec(),
                indices: QUAD_INDICES.to_vec(),
            },
            color,
            model: engine_render::prelude::IDENTITY_MODEL,
            material: None,
            stroke: None,
        },
    );
}

fn draw_centered_screen_text(
    queue: &mut DrawQueue,
    layer: RenderLayer,
    order: SortOrder,
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
    queue.push(
        layer,
        order,
        DrawCommand::RawText {
            text: text.to_owned(),
            x: world_pos.x,
            y: world_pos.y,
            font_size: font_size / camera.zoom,
            color,
        },
    );
}

fn draw_preview_rects(
    queue: &mut DrawQueue,
    layer: RenderLayer,
    base: i32,
    camera: &Camera2D,
    viewport_w: f32,
    viewport_h: f32,
    left: f32,
    top: f32,
    rects: &[(i32, f32, f32, f32, f32, Color)],
) {
    for &(offset, x, y, width, height, color) in rects {
        draw_screen_rect(
            queue,
            layer,
            SortOrder::new(base + offset),
            camera,
            viewport_w,
            viewport_h,
            left + x,
            top + y,
            width,
            height,
            color,
        );
    }
}

fn preview_rects_for(item: StoreItemKind) -> &'static [(i32, f32, f32, f32, f32, Color)] {
    match item {
        StoreItemKind::Reader => &[
            (0, 38.0, 22.0, 66.0, 86.0, STORE_PREVIEW_DARK),
            (1, 50.0, 40.0, 42.0, 50.0, STORE_PREVIEW_MID),
            (2, 58.0, 12.0, 26.0, 6.0, STORE_PREVIEW_LIGHT),
        ],
        StoreItemKind::Screen => &[
            (0, 32.0, 22.0, 80.0, 96.0, STORE_PREVIEW_DARK),
            (1, 40.0, 30.0, 64.0, 80.0, STORE_PREVIEW_MID),
            (2, 48.0, 38.0, 18.0, 18.0, STORE_PREVIEW_LIGHT),
            (2, 72.0, 38.0, 18.0, 18.0, STORE_PREVIEW_LIGHT),
            (2, 48.0, 62.0, 18.0, 18.0, STORE_PREVIEW_LIGHT),
            (2, 72.0, 62.0, 18.0, 18.0, STORE_PREVIEW_LIGHT),
        ],
        StoreItemKind::Combiner => &[
            (0, 36.0, 32.0, 72.0, 56.0, STORE_PREVIEW_DARK),
            (1, 28.0, 36.0, 10.0, 10.0, STORE_PREVIEW_LIGHT),
            (1, 28.0, 54.0, 10.0, 10.0, STORE_PREVIEW_LIGHT),
            (1, 106.0, 45.0, 10.0, 10.0, STORE_PREVIEW_LIGHT),
            (1, 52.0, 48.0, 40.0, 4.0, STORE_PREVIEW_MID),
        ],
        StoreItemKind::BoosterMachine => &[
            (0, 40.0, 32.0, 64.0, 50.0, STORE_PREVIEW_DARK),
            (1, 30.0, 40.0, 10.0, 10.0, STORE_PREVIEW_LIGHT),
        ],
    }
}

/// Sub-order layout within each store item (10 slots per item):
/// +0: outer panel background
/// +1: highlight border
/// +2: inner panel fill
/// +3..+5: preview shapes (dark, mid, light layers)
/// +6: label text
/// +7: price text
fn draw_store_item(
    queue: &mut DrawQueue,
    layer: RenderLayer,
    base_order: i32,
    camera: &Camera2D,
    viewport_w: f32,
    viewport_h: f32,
    item: StoreItemKind,
    left: f32,
    top: f32,
) {
    for (offset, inset, color) in [
        (0_i32, 0.0, STORE_PANEL_FILL),
        (1_i32, 2.0, STORE_HIGHLIGHT_COLOR),
        (2_i32, 4.0, STORE_PANEL_FILL),
    ] {
        draw_screen_rect(
            queue,
            layer,
            SortOrder::new(base_order + offset),
            camera,
            viewport_w,
            viewport_h,
            left + inset,
            top + inset,
            STORE_ITEM_WIDTH - inset * 2.0,
            STORE_ITEM_HEIGHT - inset * 2.0,
            color,
        );
    }

    let preview_base = base_order + 3;
    draw_preview_rects(
        queue,
        layer,
        preview_base,
        camera,
        viewport_w,
        viewport_h,
        left,
        top,
        preview_rects_for(item),
    );

    let center_x = left + STORE_ITEM_WIDTH * 0.5;
    draw_centered_screen_text(
        queue,
        layer,
        SortOrder::new(base_order + 6),
        camera,
        viewport_w,
        viewport_h,
        item.label(),
        center_x,
        top + STORE_ITEM_HEIGHT - 38.0,
        STORE_BODY_FONT,
        STORE_TEXT_COLOR,
    );
    draw_centered_screen_text(
        queue,
        layer,
        SortOrder::new(base_order + 7),
        camera,
        viewport_w,
        viewport_h,
        item.price_text(),
        center_x,
        top + STORE_ITEM_HEIGHT - 20.0,
        STORE_PRICE_FONT,
        STORE_PRICE_COLOR,
    );
}
/// Each store item occupies 10 sort-order slots (see `draw_store_item`).
/// Items are assigned non-overlapping ranges starting at `base_order + 10`.
/// Header text uses `base_order`.
pub(crate) fn render_store_page(
    queue: &mut DrawQueue,
    layer: RenderLayer,
    base_order: SortOrder,
    camera: &Camera2D,
    viewport_w: f32,
    viewport_h: f32,
    grid: &StashGrid,
    wallet: &StoreWallet,
    catalog: &StoreCatalog,
) {
    let base = base_order.value();
    draw_centered_screen_text(
        queue,
        layer,
        SortOrder::new(base),
        camera,
        viewport_w,
        viewport_h,
        "Store",
        GRID_MARGIN + 96.0,
        GRID_MARGIN + 4.0,
        STORE_TITLE_FONT,
        STORE_TEXT_COLOR,
    );
    draw_centered_screen_text(
        queue,
        layer,
        SortOrder::new(base + 1),
        camera,
        viewport_w,
        viewport_h,
        &format!("Coins: {}", wallet.coins()),
        GRID_MARGIN + 360.0,
        GRID_MARGIN + 4.0,
        STORE_COIN_FONT,
        STORE_TEXT_COLOR,
    );
    draw_centered_screen_text(
        queue,
        layer,
        SortOrder::new(base + 2),
        camera,
        viewport_w,
        viewport_h,
        &format!("Next tab: {}", storage_tab_purchase_cost(grid.page_count())),
        GRID_MARGIN + 476.0,
        GRID_MARGIN + 4.0,
        STORE_TITLE_FONT,
        STORE_PRICE_COLOR,
    );

    let items = catalog.items();
    for (index, item) in items.iter().copied().enumerate() {
        let (left, top) = store_item_layout(grid.width(), items.len(), index);
        let item_base = base + 10 + (index as i32) * 10;
        draw_store_item(
            queue, layer, item_base, camera, viewport_w, viewport_h, item, left, top,
        );
    }
}
pub fn store_render_system(
    grid: Res<StashGrid>,
    visible: Res<StashVisible>,
    wallet: Res<StoreWallet>,
    catalog: Res<StoreCatalog>,
    camera_query: Query<&Camera2D>,
    renderer: Res<RendererRes>,
    mut draw_queue: ResMut<DrawQueue>,
) {
    if !visible.0 || !grid.is_store_page() {
        return;
    }

    let Some((vw, vh, camera)) = resolve_viewport_camera(&renderer, &camera_query) else {
        return;
    };
    render_store_page(
        &mut draw_queue,
        RenderLayer::UI,
        SortOrder::new(200),
        &camera,
        vw,
        vh,
        &grid,
        &wallet,
        &catalog,
    );
}

pub fn store_item_screen_bounds(
    grid: &StashGrid,
    catalog: &StoreCatalog,
    index: usize,
) -> Option<(f32, f32, f32, f32)> {
    let items = catalog.items();
    items
        .get(index)
        .map(|_| store_item_bounds(grid.width(), items.len(), index))
}
