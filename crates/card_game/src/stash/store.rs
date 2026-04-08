use bevy_ecs::prelude::{Entity, Query, Res, ResMut, Resource, World};
use engine_core::color::Color;
use engine_core::prelude::EventBus;
use engine_core::prelude::Transform2D;
use engine_input::prelude::{MouseButton, MouseState};
use engine_physics::prelude::{Collider, PhysicsCommand, RigidBody};
use engine_render::font::measure_text;
use engine_render::prelude::{
    Camera2D, QUAD_INDICES, RendererRes, rect_vertices, resolve_viewport_camera, screen_to_world,
};
use engine_scene::render_order::RenderLayer;
use glam::Vec2;

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
use engine_scene::prelude::ChildOf;

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
        if !self.can_afford(cost) {
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
}

impl StoreItemKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Reader => "Card Reader",
            Self::Screen => "Screen",
            Self::Combiner => "Combiner",
        }
    }

    pub fn cost(self) -> u32 {
        match self {
            Self::Reader => 30,
            Self::Screen => 20,
            Self::Combiner => 25,
        }
    }

    pub fn refund_value(self) -> u32 {
        self.cost()
    }

    pub fn preview_color(self) -> Color {
        match self {
            Self::Reader => Color {
                r: 0.56,
                g: 0.44,
                b: 0.26,
                a: 1.0,
            },
            Self::Screen => Color {
                r: 0.14,
                g: 0.34,
                b: 0.54,
                a: 1.0,
            },
            Self::Combiner => Color {
                r: 0.36,
                g: 0.22,
                b: 0.50,
                a: 1.0,
            },
        }
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct StoreCatalog {
    items: Vec<StoreItemKind>,
}

impl Default for StoreCatalog {
    fn default() -> Self {
        Self {
            items: vec![
                StoreItemKind::Reader,
                StoreItemKind::Screen,
                StoreItemKind::Combiner,
            ],
        }
    }
}

impl StoreCatalog {
    pub fn items(&self) -> &[StoreItemKind] {
        &self.items
    }
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

fn item_columns(item_count: usize) -> u8 {
    STORE_COLUMNS.min(item_count.max(1) as u8)
}

fn store_item_layout(grid_width: u8, item_count: usize, index: usize) -> (f32, f32) {
    let columns = item_columns(item_count);
    let total_w = f32::from(columns) * STORE_ITEM_WIDTH
        + f32::from(columns.saturating_sub(1)) * STORE_ITEM_GAP_X;
    let grid_w = f32::from(grid_width) * SLOT_STRIDE_W - SLOT_GAP;
    let start_x = GRID_MARGIN + (grid_w - total_w).max(0.0) * 0.5;
    let start_y = GRID_MARGIN + STORE_HEADER_HEIGHT;
    let col = f32::from(index as u8 % columns);
    let row = f32::from(index as u8 / columns);
    let left = start_x + col * (STORE_ITEM_WIDTH + STORE_ITEM_GAP_X);
    let top = start_y + row * (STORE_ITEM_HEIGHT + STORE_ITEM_GAP_Y);
    (left, top)
}

fn draw_screen_rect(
    renderer: &mut dyn engine_render::renderer::Renderer,
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
    renderer.draw_shape(
        &verts,
        &QUAD_INDICES,
        color,
        engine_render::prelude::IDENTITY_MODEL,
    );
}

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

fn draw_reader_preview(
    renderer: &mut dyn engine_render::renderer::Renderer,
    camera: &Camera2D,
    viewport_w: f32,
    viewport_h: f32,
    left: f32,
    top: f32,
) {
    draw_screen_rect(
        renderer,
        camera,
        viewport_w,
        viewport_h,
        left + 38.0,
        top + 22.0,
        66.0,
        86.0,
        STORE_PREVIEW_DARK,
    );
    draw_screen_rect(
        renderer,
        camera,
        viewport_w,
        viewport_h,
        left + 50.0,
        top + 40.0,
        42.0,
        50.0,
        STORE_PREVIEW_MID,
    );
    draw_screen_rect(
        renderer,
        camera,
        viewport_w,
        viewport_h,
        left + 58.0,
        top + 12.0,
        26.0,
        6.0,
        STORE_PREVIEW_LIGHT,
    );
}

fn draw_screen_preview(
    renderer: &mut dyn engine_render::renderer::Renderer,
    camera: &Camera2D,
    viewport_w: f32,
    viewport_h: f32,
    left: f32,
    top: f32,
) {
    draw_screen_rect(
        renderer,
        camera,
        viewport_w,
        viewport_h,
        left + 32.0,
        top + 22.0,
        80.0,
        96.0,
        STORE_PREVIEW_DARK,
    );
    draw_screen_rect(
        renderer,
        camera,
        viewport_w,
        viewport_h,
        left + 40.0,
        top + 30.0,
        64.0,
        80.0,
        STORE_PREVIEW_MID,
    );
    draw_screen_rect(
        renderer,
        camera,
        viewport_w,
        viewport_h,
        left + 48.0,
        top + 38.0,
        18.0,
        18.0,
        STORE_PREVIEW_LIGHT,
    );
    draw_screen_rect(
        renderer,
        camera,
        viewport_w,
        viewport_h,
        left + 72.0,
        top + 38.0,
        18.0,
        18.0,
        STORE_PREVIEW_LIGHT,
    );
    draw_screen_rect(
        renderer,
        camera,
        viewport_w,
        viewport_h,
        left + 48.0,
        top + 62.0,
        18.0,
        18.0,
        STORE_PREVIEW_LIGHT,
    );
    draw_screen_rect(
        renderer,
        camera,
        viewport_w,
        viewport_h,
        left + 72.0,
        top + 62.0,
        18.0,
        18.0,
        STORE_PREVIEW_LIGHT,
    );
}

fn draw_combiner_preview(
    renderer: &mut dyn engine_render::renderer::Renderer,
    camera: &Camera2D,
    viewport_w: f32,
    viewport_h: f32,
    left: f32,
    top: f32,
) {
    draw_screen_rect(
        renderer,
        camera,
        viewport_w,
        viewport_h,
        left + 36.0,
        top + 32.0,
        72.0,
        56.0,
        STORE_PREVIEW_DARK,
    );
    draw_screen_rect(
        renderer,
        camera,
        viewport_w,
        viewport_h,
        left + 28.0,
        top + 36.0,
        10.0,
        10.0,
        STORE_PREVIEW_LIGHT,
    );
    draw_screen_rect(
        renderer,
        camera,
        viewport_w,
        viewport_h,
        left + 28.0,
        top + 54.0,
        10.0,
        10.0,
        STORE_PREVIEW_LIGHT,
    );
    draw_screen_rect(
        renderer,
        camera,
        viewport_w,
        viewport_h,
        left + 106.0,
        top + 45.0,
        10.0,
        10.0,
        STORE_PREVIEW_LIGHT,
    );
    draw_screen_rect(
        renderer,
        camera,
        viewport_w,
        viewport_h,
        left + 52.0,
        top + 48.0,
        40.0,
        4.0,
        STORE_PREVIEW_MID,
    );
}

fn draw_store_item(
    renderer: &mut dyn engine_render::renderer::Renderer,
    camera: &Camera2D,
    viewport_w: f32,
    viewport_h: f32,
    item: StoreItemKind,
    left: f32,
    top: f32,
) {
    draw_screen_rect(
        renderer,
        camera,
        viewport_w,
        viewport_h,
        left,
        top,
        STORE_ITEM_WIDTH,
        STORE_ITEM_HEIGHT,
        STORE_PANEL_FILL,
    );
    draw_screen_rect(
        renderer,
        camera,
        viewport_w,
        viewport_h,
        left + 2.0,
        top + 2.0,
        STORE_ITEM_WIDTH - 4.0,
        STORE_ITEM_HEIGHT - 4.0,
        STORE_HIGHLIGHT_COLOR,
    );
    draw_screen_rect(
        renderer,
        camera,
        viewport_w,
        viewport_h,
        left + 4.0,
        top + 4.0,
        STORE_ITEM_WIDTH - 8.0,
        STORE_ITEM_HEIGHT - 8.0,
        STORE_PANEL_FILL,
    );

    match item {
        StoreItemKind::Reader => {
            draw_reader_preview(renderer, camera, viewport_w, viewport_h, left, top);
        }
        StoreItemKind::Screen => {
            draw_screen_preview(renderer, camera, viewport_w, viewport_h, left, top);
        }
        StoreItemKind::Combiner => {
            draw_combiner_preview(renderer, camera, viewport_w, viewport_h, left, top);
        }
    }

    let center_x = left + STORE_ITEM_WIDTH * 0.5;
    draw_centered_screen_text(
        renderer,
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
        renderer,
        camera,
        viewport_w,
        viewport_h,
        &format!("{} coins", item.cost()),
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
    for (index, &item) in items.iter().enumerate() {
        let (left, top) = store_item_layout(grid.width(), items.len(), index);
        let right = left + STORE_ITEM_WIDTH;
        let bottom = top + STORE_ITEM_HEIGHT;
        if screen_pos.x >= left
            && screen_pos.x <= right
            && screen_pos.y >= top
            && screen_pos.y <= bottom
        {
            return Some(item);
        }
    }
    None
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

pub(crate) fn render_store_page(
    renderer: &mut dyn engine_render::renderer::Renderer,
    camera: &Camera2D,
    viewport_w: f32,
    viewport_h: f32,
    grid: &StashGrid,
    wallet: &StoreWallet,
    catalog: &StoreCatalog,
) {
    draw_centered_screen_text(
        renderer,
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
        renderer,
        camera,
        viewport_w,
        viewport_h,
        &format!("Coins: {}", wallet.coins()),
        GRID_MARGIN + 360.0,
        GRID_MARGIN + 4.0,
        STORE_TITLE_FONT,
        STORE_TEXT_COLOR,
    );
    draw_centered_screen_text(
        renderer,
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
        draw_store_item(renderer, camera, viewport_w, viewport_h, item, left, top);
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
    let children: Vec<Entity> = {
        let mut query = world.query::<(Entity, &ChildOf)>();
        query
            .iter(world)
            .filter_map(|(child, parent)| (parent.0 == entity).then_some(child))
            .collect()
    };

    for child in children {
        despawn_entity_tree(world, child);
    }

    let _ = world.despawn(entity);
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

    let drag_active = world
        .get_resource::<DragState>()
        .is_some_and(|drag| drag.dragging.is_some())
        || world
            .get_resource::<ReaderDragState>()
            .is_some_and(|drag| drag.dragging.is_some())
        || world
            .get_resource::<ScreenDragState>()
            .is_some_and(|drag| drag.dragging.is_some())
        || world
            .get_resource::<CombinerDragState>()
            .is_some_and(|drag| drag.dragging.is_some());
    if drag_active || !stash_ui_contains(mouse_screen_pos, &grid) {
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

    let reader_drag = world
        .get_resource::<ReaderDragState>()
        .and_then(|drag| drag.dragging.clone());
    let screen_drag = world
        .get_resource::<ScreenDragState>()
        .and_then(|drag| drag.dragging.clone());
    let combiner_drag = world
        .get_resource::<CombinerDragState>()
        .and_then(|drag| drag.dragging.clone());

    if let Some(dragged_reader) = reader_drag {
        sell_reader(world, dragged_reader.entity);
        world.resource_mut::<ReaderDragState>().dragging = None;
    } else if let Some(dragged_screen) = screen_drag {
        sell_screen(world, dragged_screen.entity);
        world.resource_mut::<ScreenDragState>().dragging = None;
    } else if let Some(dragged_combiner) = combiner_drag {
        sell_combiner(world, dragged_combiner.entity);
        world.resource_mut::<CombinerDragState>().dragging = None;
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

pub fn store_render_system(
    grid: Res<StashGrid>,
    visible: Res<StashVisible>,
    wallet: Res<StoreWallet>,
    catalog: Res<StoreCatalog>,
    camera_query: Query<&Camera2D>,
    mut renderer: ResMut<RendererRes>,
) {
    if !visible.0 || !grid.is_store_page() {
        return;
    }

    let Some((vw, vh, camera)) = resolve_viewport_camera(&renderer, &camera_query) else {
        return;
    };
    render_store_page(&mut **renderer, &camera, vw, vh, &grid, &wallet, &catalog);
}

pub fn store_item_screen_bounds(
    grid: &StashGrid,
    catalog: &StoreCatalog,
    index: usize,
) -> Option<(f32, f32, f32, f32)> {
    let items = catalog.items();
    if index >= items.len() {
        return None;
    }
    let (left, top) = store_item_layout(grid.width(), items.len(), index);
    Some((left, top, left + STORE_ITEM_WIDTH, top + STORE_ITEM_HEIGHT))
}
