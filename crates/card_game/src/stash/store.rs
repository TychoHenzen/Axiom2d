use bevy_ecs::prelude::{Entity, Query, Res, ResMut, Resource, World};
use engine_core::color::Color;
use engine_core::prelude::EventBus;
use engine_core::prelude::Transform2D;
use engine_input::prelude::{MouseButton, MouseState};
use engine_render::font::measure_text;
use engine_render::prelude::{
    Camera2D, QUAD_INDICES, RendererRes, rect_vertices, resolve_viewport_camera, screen_to_world,
};
use engine_render::shape::TessellatedMesh;
use engine_scene::prelude::{RenderLayer, SortOrder};
use engine_ui::draw_command::{DrawCommand, DrawQueue};
use glam::Vec2;

use crate::booster::device::{
    BoosterDragInfo, BoosterDragState, BoosterMachine, spawn_booster_machine,
};
use crate::card::combiner_device::{
    CombinerDevice, CombinerDragInfo, CombinerDragState, spawn_combiner_device,
};
use crate::card::component::CardZone;
use crate::card::interaction::drag_state::DragState;
use crate::card::reader::{
    READER_COLLISION_FILTER, READER_COLLISION_GROUP, READER_HALF_EXTENTS, ReaderDragInfo,
    ReaderDragState, spawn_reader,
};
use crate::card::screen_device::{ScreenDragInfo, ScreenDragState, spawn_screen_device};
use crate::stash::constants::{GRID_MARGIN, SLOT_GAP, SLOT_STRIDE_W};
use crate::stash::grid::StashGrid;
use crate::stash::pages::stash_ui_contains;
use crate::stash::toggle::StashVisible;
use engine_core::scale_spring::ScaleSpring;
use engine_physics::prelude::{Collider, PhysicsCommand, RigidBody};
use engine_scene::prelude::{ChildOf, Children};

pub const STORE_STARTING_COINS: u32 = 100;
pub const STORE_TAB_BASE_COST: u32 = 25;

pub const STORE_COLUMNS: u8 = 2;
pub const STORE_ITEM_WIDTH: f32 = 152.0;
pub const STORE_ITEM_HEIGHT: f32 = 172.0;
pub const STORE_ITEM_GAP_X: f32 = 20.0;
pub const STORE_ITEM_GAP_Y: f32 = 18.0;
pub const STORE_HEADER_HEIGHT: f32 = 34.0;
pub const STORE_TITLE_FONT: f32 = 18.0;
pub const STORE_BODY_FONT: f32 = 15.0;
pub const STORE_PRICE_FONT: f32 = 14.0;
pub const STORE_COIN_FONT: f32 = 18.0;

const STORE_PANEL_FILL: Color = Color {
    r: 0.18,
    g: 0.16,
    b: 0.14,
    a: 1.0,
};
const STORE_PREVIEW_DARK: Color = Color {
    r: 0.12,
    g: 0.12,
    b: 0.14,
    a: 1.0,
};
const STORE_PREVIEW_MID: Color = Color {
    r: 0.28,
    g: 0.28,
    b: 0.30,
    a: 1.0,
};
const STORE_PREVIEW_LIGHT: Color = Color {
    r: 0.56,
    g: 0.56,
    b: 0.58,
    a: 1.0,
};
const STORE_TEXT_COLOR: Color = Color {
    r: 0.94,
    g: 0.90,
    b: 0.80,
    a: 1.0,
};
const STORE_PRICE_COLOR: Color = Color {
    r: 0.90,
    g: 0.75,
    b: 0.35,
    a: 1.0,
};
const STORE_HIGHLIGHT_COLOR: Color = Color {
    r: 0.36,
    g: 0.30,
    b: 0.24,
    a: 1.0,
};

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoreWallet {
    coins: u32,
}

impl Default for StoreWallet {
    fn default() -> Self {
        Self {
            coins: STORE_STARTING_COINS,
        }
    }
}

impl StoreWallet {
    pub fn new(coins: u32) -> Self {
        Self { coins }
    }

    pub fn coins(&self) -> u32 {
        self.coins
    }

    pub fn can_afford(&self, cost: u32) -> bool {
        self.coins >= cost
    }

    pub fn spend(&mut self, cost: u32) -> bool {
        if self.coins < cost {
            return false;
        }
        self.coins -= cost;
        true
    }

    pub fn refund(&mut self, value: u32) {
        self.coins = self.coins.saturating_add(value);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreItemKind {
    Reader,
    Screen,
    Combiner,
    BoosterMachine,
}

impl StoreItemKind {
    pub const ALL: [Self; 4] = [
        Self::Reader,
        Self::Screen,
        Self::Combiner,
        Self::BoosterMachine,
    ];

    const LABELS: [&'static str; 4] = ["Card Reader", "Screen", "Combiner", "Booster Machine"];
    const COSTS: [u32; 4] = [30, 20, 25, 35];
    const PRICE_TEXTS: [&'static str; 4] = ["30 coins", "20 coins", "25 coins", "35 coins"];
    const PREVIEW_COLORS: [Color; 4] = [
        Color {
            r: 0.56,
            g: 0.44,
            b: 0.26,
            a: 1.0,
        },
        Color {
            r: 0.14,
            g: 0.34,
            b: 0.54,
            a: 1.0,
        },
        Color {
            r: 0.36,
            g: 0.22,
            b: 0.50,
            a: 1.0,
        },
        Color {
            r: 0.56,
            g: 0.44,
            b: 0.12,
            a: 1.0,
        },
    ];

    fn index(self) -> usize {
        self as usize
    }

    pub fn label(self) -> &'static str {
        Self::LABELS[self.index()]
    }

    pub fn cost(self) -> u32 {
        Self::COSTS[self.index()]
    }

    pub fn price_text(self) -> &'static str {
        Self::PRICE_TEXTS[self.index()]
    }

    pub fn refund_value(self) -> u32 {
        self.cost()
    }

    pub fn preview_color(self) -> Color {
        Self::PREVIEW_COLORS[self.index()]
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct StoreCatalog {
    items: [StoreItemKind; 4],
}

impl Default for StoreCatalog {
    fn default() -> Self {
        Self {
            items: StoreItemKind::ALL,
        }
    }
}

impl StoreCatalog {
    pub fn items(&self) -> &[StoreItemKind] {
        &self.items
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StoreDragTarget {
    Any,
    Reader(Entity),
    Screen(Entity),
    Combiner(Entity),
    Booster(Entity),
}

fn active_store_drag_target(world: &World) -> Option<StoreDragTarget> {
    if world
        .get_resource::<DragState>()
        .is_some_and(|drag| drag.dragging.is_some())
    {
        return Some(StoreDragTarget::Any);
    }

    if let Some(entity) = world
        .get_resource::<ReaderDragState>()
        .and_then(|state| state.dragging.as_ref().map(|drag| drag.entity))
    {
        return Some(StoreDragTarget::Reader(entity));
    }

    if let Some(entity) = world
        .get_resource::<ScreenDragState>()
        .and_then(|state| state.dragging.as_ref().map(|drag| drag.entity))
    {
        return Some(StoreDragTarget::Screen(entity));
    }

    if let Some(entity) = world
        .get_resource::<CombinerDragState>()
        .and_then(|state| state.dragging.as_ref().map(|drag| drag.entity))
    {
        return Some(StoreDragTarget::Combiner(entity));
    }

    world.get_resource::<BoosterDragState>().and_then(|state| {
        state
            .dragging
            .as_ref()
            .map(|drag| StoreDragTarget::Booster(drag.entity))
    })
}

pub fn storage_tab_purchase_cost(storage_tab_count: u8) -> u32 {
    let exponent = u32::from(storage_tab_count.saturating_sub(1).min(30));
    STORE_TAB_BASE_COST.saturating_mul(1u32 << exponent)
}

pub fn store_ui_bounds(grid_width: u8, grid_height: u8) -> (f32, f32, f32, f32) {
    let left = GRID_MARGIN;
    let right = GRID_MARGIN + f32::from(grid_width) * SLOT_STRIDE_W - SLOT_GAP;
    let top = GRID_MARGIN;
    let bottom = crate::stash::pages::tab_row_top_y(grid_height) + crate::stash::pages::TAB_HEIGHT;
    (left, top, right, bottom)
}

fn item_columns(item_count: usize) -> usize {
    (STORE_COLUMNS as usize).min(item_count.max(1))
}

fn store_item_origin(grid_width: u8, item_count: usize) -> (f32, f32) {
    let columns = item_columns(item_count);
    let total_w =
        columns as f32 * STORE_ITEM_WIDTH + columns.saturating_sub(1) as f32 * STORE_ITEM_GAP_X;
    let grid_w = f32::from(grid_width) * SLOT_STRIDE_W - SLOT_GAP;
    let start_x = GRID_MARGIN + (grid_w - total_w).max(0.0) * 0.5;
    (start_x, GRID_MARGIN + STORE_HEADER_HEIGHT)
}

fn store_item_layout(grid_width: u8, item_count: usize, index: usize) -> (f32, f32) {
    let columns = item_columns(item_count);
    let (start_x, start_y) = store_item_origin(grid_width, item_count);
    let col = (index % columns) as f32;
    let row = (index / columns) as f32;
    let left = start_x + col * (STORE_ITEM_WIDTH + STORE_ITEM_GAP_X);
    let top = start_y + row * (STORE_ITEM_HEIGHT + STORE_ITEM_GAP_Y);
    (left, top)
}

fn store_item_bounds(grid_width: u8, item_count: usize, index: usize) -> (f32, f32, f32, f32) {
    let (left, top) = store_item_layout(grid_width, item_count, index);
    (left, top, left + STORE_ITEM_WIDTH, top + STORE_ITEM_HEIGHT)
}

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

fn store_item_at(
    screen_pos: Vec2,
    grid: &StashGrid,
    catalog: &StoreCatalog,
) -> Option<StoreItemKind> {
    let items = catalog.items();
    let item_count = items.len();
    if item_count == 0 {
        return None;
    }

    items.iter().copied().enumerate().find_map(|(index, item)| {
        let (left, top, right, bottom) = store_item_bounds(grid.width(), item_count, index);
        (screen_pos.x >= left
            && screen_pos.x < right
            && screen_pos.y >= top
            && screen_pos.y < bottom)
            .then_some(item)
    })
}

fn spawn_reader_purchase(world: &mut World, position: Vec2) -> Entity {
    let (reader_entity, _jack_entity) = spawn_reader(world, position);
    if let Some(mut bus) = world.get_resource_mut::<EventBus<PhysicsCommand>>() {
        bus.push(PhysicsCommand::AddBody {
            entity: reader_entity,
            body_type: RigidBody::Kinematic,
            position,
        });
        bus.push(PhysicsCommand::AddCollider {
            entity: reader_entity,
            collider: Collider::Aabb(READER_HALF_EXTENTS),
        });
        bus.push(PhysicsCommand::SetCollisionGroup {
            entity: reader_entity,
            membership: READER_COLLISION_GROUP,
            filter: READER_COLLISION_FILTER,
        });
    }
    reader_entity
}

fn spawn_screen_purchase(world: &mut World, position: Vec2) -> Entity {
    let (screen_entity, _jack_entity) = spawn_screen_device(world, position);
    screen_entity
}

fn spawn_combiner_purchase(world: &mut World, position: Vec2) -> Entity {
    let (device_entity, _in_a, _in_b, _out) = spawn_combiner_device(world, position);
    device_entity
}

fn spawn_booster_purchase(world: &mut World, position: Vec2) -> Entity {
    let (device_entity, _jack_entity) = spawn_booster_machine(world, position);
    device_entity
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

fn despawn_connected_cables(world: &mut World, endpoints: &[Entity]) {
    let cable_entities: Vec<Entity> = {
        let mut query = world.query::<(Entity, &crate::card::jack_cable::Cable)>();
        query
            .iter(world)
            .filter_map(|(entity, cable)| {
                if endpoints.contains(&cable.source) || endpoints.contains(&cable.dest) {
                    Some(entity)
                } else {
                    None
                }
            })
            .collect()
    };
    for cable in cable_entities {
        world.despawn(cable);
    }
}

fn despawn_entity_tree(world: &mut World, entity: Entity) {
    let mut stack = vec![(entity, false)];

    while let Some((current, expanded)) = stack.pop() {
        if expanded {
            let _ = world.despawn(current);
            continue;
        }

        stack.push((current, true));
        let children = if let Some(children) = world.get::<Children>(current) {
            children.0.clone()
        } else {
            let mut query = world.query::<(Entity, &ChildOf)>();
            query
                .iter(world)
                .filter_map(|(child, parent)| (parent.0 == current).then_some(child))
                .collect()
        };
        stack.extend(children.into_iter().map(|child| (child, false)));
    }
}

pub fn store_buy_system(world: &mut World) {
    let (visible, mouse_screen_pos, mouse_world_pos, just_pressed, grid) = {
        let Some(visible) = world.get_resource::<StashVisible>() else {
            return;
        };
        let Some(mouse) = world.get_resource::<MouseState>() else {
            return;
        };
        let grid = world.resource::<StashGrid>().clone();
        (
            visible.0,
            mouse.screen_pos(),
            mouse.world_pos(),
            mouse.just_pressed(MouseButton::Left),
            grid,
        )
    };
    if !visible || !just_pressed || !grid.is_store_page() {
        return;
    }

    if active_store_drag_target(world).is_some() || !stash_ui_contains(mouse_screen_pos, &grid) {
        return;
    }

    let Some(catalog) = world.get_resource::<StoreCatalog>() else {
        return;
    };
    let Some(item) = store_item_at(mouse_screen_pos, &grid, catalog) else {
        return;
    };

    let cost = item.cost();
    {
        let mut wallet = world.resource_mut::<StoreWallet>();
        if !wallet.spend(cost) {
            return;
        }
    }

    let spawn_pos = mouse_world_pos;
    match item {
        StoreItemKind::Reader => {
            let entity = spawn_reader_purchase(world, spawn_pos);
            world.resource_mut::<ReaderDragState>().dragging = Some(ReaderDragInfo {
                entity,
                grab_offset: Vec2::ZERO,
            });
        }
        StoreItemKind::Screen => {
            let entity = spawn_screen_purchase(world, spawn_pos);
            world.resource_mut::<ScreenDragState>().dragging = Some(ScreenDragInfo {
                entity,
                grab_offset: Vec2::ZERO,
            });
        }
        StoreItemKind::Combiner => {
            let entity = spawn_combiner_purchase(world, spawn_pos);
            world.resource_mut::<CombinerDragState>().dragging = Some(CombinerDragInfo {
                entity,
                grab_offset: Vec2::ZERO,
            });
        }
        StoreItemKind::BoosterMachine => {
            let entity = spawn_booster_purchase(world, spawn_pos);
            world.resource_mut::<BoosterDragState>().dragging = Some(BoosterDragInfo {
                entity,
                grab_offset: Vec2::ZERO,
            });
        }
    }
}

pub fn store_sell_system(world: &mut World) {
    let (visible, mouse_screen_pos, just_released, grid) = {
        let Some(visible) = world.get_resource::<StashVisible>() else {
            return;
        };
        let Some(mouse) = world.get_resource::<MouseState>() else {
            return;
        };
        let grid = world.resource::<StashGrid>().clone();
        (
            visible.0,
            mouse.screen_pos(),
            mouse.just_released(MouseButton::Left),
            grid,
        )
    };
    if !visible || !just_released || !grid.is_store_page() {
        return;
    }
    if !stash_ui_contains(mouse_screen_pos, &grid) {
        return;
    }

    match active_store_drag_target(world) {
        Some(StoreDragTarget::Reader(entity)) => {
            sell_reader(world, entity);
            world.resource_mut::<ReaderDragState>().dragging = None;
        }
        Some(StoreDragTarget::Screen(entity)) => {
            sell_screen(world, entity);
            world.resource_mut::<ScreenDragState>().dragging = None;
        }
        Some(StoreDragTarget::Combiner(entity)) => {
            sell_combiner(world, entity);
            world.resource_mut::<CombinerDragState>().dragging = None;
        }
        Some(StoreDragTarget::Booster(entity)) => {
            sell_booster(world, entity);
            world.resource_mut::<BoosterDragState>().dragging = None;
        }
        Some(StoreDragTarget::Any) | None => {}
    }
}

fn sell_screen(world: &mut World, entity: Entity) {
    let (jack_entity, refund) = {
        let Some(screen) = world.get::<crate::card::screen_device::ScreenDevice>(entity) else {
            return;
        };
        (screen.signature_input, StoreItemKind::Screen.refund_value())
    };
    despawn_connected_cables(world, &[entity, jack_entity]);
    despawn_entity_tree(world, jack_entity);
    despawn_entity_tree(world, entity);
    world.resource_mut::<StoreWallet>().refund(refund);
}

fn sell_combiner(world: &mut World, entity: Entity) {
    let (jack_entities, refund) = {
        let Some(device) = world.get::<CombinerDevice>(entity) else {
            return;
        };
        (
            [device.input_a, device.input_b, device.output],
            StoreItemKind::Combiner.refund_value(),
        )
    };
    despawn_connected_cables(world, &jack_entities);
    for jack in jack_entities {
        despawn_entity_tree(world, jack);
    }
    despawn_entity_tree(world, entity);
    world.resource_mut::<StoreWallet>().refund(refund);
}

fn sell_reader(world: &mut World, entity: Entity) {
    let (jack_entity, loaded_card, refund) = {
        let Some(reader) = world.get::<crate::card::reader::CardReader>(entity) else {
            return;
        };
        (
            reader.jack_entity,
            reader.loaded,
            StoreItemKind::Reader.refund_value(),
        )
    };

    if let Some(card_entity) = loaded_card {
        let reader_pos = world
            .get::<Transform2D>(entity)
            .map_or(Vec2::ZERO, |t| t.position);
        if let Some(mut zone) = world.get_mut::<CardZone>(card_entity) {
            *zone = CardZone::Table;
        }
        if let Some(mut transform) = world.get_mut::<Transform2D>(card_entity) {
            transform.position = reader_pos;
            transform.rotation = 0.0;
            transform.scale = Vec2::ONE;
        }
        world
            .entity_mut(card_entity)
            .insert(RigidBody::Dynamic)
            .insert(RenderLayer::World)
            .insert(ScaleSpring::new(1.0));
        if let Some(collider) = world.get::<Collider>(card_entity).cloned()
            && let Some(mut bus) = world.get_resource_mut::<EventBus<PhysicsCommand>>()
        {
            bus.push(PhysicsCommand::RemoveBody {
                entity: card_entity,
            });
            bus.push(PhysicsCommand::AddBody {
                entity: card_entity,
                body_type: RigidBody::Dynamic,
                position: reader_pos,
            });
            bus.push(PhysicsCommand::AddCollider {
                entity: card_entity,
                collider,
            });
        }
    }

    despawn_connected_cables(world, &[entity, jack_entity]);
    despawn_entity_tree(world, jack_entity);
    despawn_entity_tree(world, entity);
    world.resource_mut::<StoreWallet>().refund(refund);
}

fn sell_booster(world: &mut World, entity: Entity) {
    let (jack_entity, button_entity, output_pack, refund) = {
        let Some(machine) = world.get::<BoosterMachine>(entity) else {
            return;
        };
        (
            machine.signal_input,
            machine.button_entity,
            machine.output_pack,
            StoreItemKind::BoosterMachine.refund_value(),
        )
    };
    despawn_connected_cables(world, &[entity, jack_entity]);
    despawn_entity_tree(world, jack_entity);
    despawn_entity_tree(world, button_entity);
    if let Some(pack) = output_pack {
        despawn_entity_tree(world, pack);
    }
    despawn_entity_tree(world, entity);
    world.resource_mut::<StoreWallet>().refund(refund);
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
