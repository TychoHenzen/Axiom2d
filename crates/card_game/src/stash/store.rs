use bevy_ecs::prelude::{Entity, Resource, World};
use engine_core::color::Color;
use engine_core::prelude::EventBus;
use engine_core::prelude::Transform2D;
use engine_input::prelude::{MouseButton, MouseState};
use engine_scene::prelude::RenderLayer;
use glam::Vec2;

use crate::booster::device::{BoosterDragState, BoosterMachine, spawn_booster_machine};
use crate::card::combiner_device::{CombinerDevice, CombinerDragState, spawn_combiner_device};
use crate::card::component::CardZone;
use crate::card::interaction::drag_state::{DeviceDragInfo, DragState};
use crate::card::reader::{
    READER_COLLISION_FILTER, READER_COLLISION_GROUP, READER_HALF_EXTENTS, ReaderDragState,
    spawn_reader,
};
use crate::card::screen_device::{ScreenDragState, spawn_screen_device};
use crate::stash::constants::{GRID_MARGIN, SLOT_GAP, SLOT_STRIDE_W};
use crate::stash::grid::StashGrid;
use crate::stash::pages::stash_ui_contains;
use crate::stash::toggle::StashVisible;
use engine_core::scale_spring::ScaleSpring;
use engine_physics::prelude::{Collider, PhysicsCommand, RigidBody};
use engine_scene::prelude::{ChildOf, Children};

pub(crate) use super::store_render::render_store_page;
pub use super::store_render::{store_item_screen_bounds, store_render_system};

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

pub(super) const STORE_PANEL_FILL: Color = Color {
    r: 0.18,
    g: 0.16,
    b: 0.14,
    a: 1.0,
};
pub(super) const STORE_PREVIEW_DARK: Color = Color {
    r: 0.12,
    g: 0.12,
    b: 0.14,
    a: 1.0,
};
pub(super) const STORE_PREVIEW_MID: Color = Color {
    r: 0.28,
    g: 0.28,
    b: 0.30,
    a: 1.0,
};
pub(super) const STORE_PREVIEW_LIGHT: Color = Color {
    r: 0.56,
    g: 0.56,
    b: 0.58,
    a: 1.0,
};
pub(super) const STORE_TEXT_COLOR: Color = Color {
    r: 0.94,
    g: 0.90,
    b: 0.80,
    a: 1.0,
};
pub(super) const STORE_PRICE_COLOR: Color = Color {
    r: 0.90,
    g: 0.75,
    b: 0.35,
    a: 1.0,
};
pub(super) const STORE_HIGHLIGHT_COLOR: Color = Color {
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

pub(super) fn store_item_layout(grid_width: u8, item_count: usize, index: usize) -> (f32, f32) {
    let columns = item_columns(item_count);
    let (start_x, start_y) = store_item_origin(grid_width, item_count);
    let col = (index % columns) as f32;
    let row = (index / columns) as f32;
    let left = start_x + col * (STORE_ITEM_WIDTH + STORE_ITEM_GAP_X);
    let top = start_y + row * (STORE_ITEM_HEIGHT + STORE_ITEM_GAP_Y);
    (left, top)
}

pub(super) fn store_item_bounds(grid_width: u8, item_count: usize, index: usize) -> (f32, f32, f32, f32) {
    let (left, top) = store_item_layout(grid_width, item_count, index);
    (left, top, left + STORE_ITEM_WIDTH, top + STORE_ITEM_HEIGHT)
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
        let children = despawn_find_children(world, current);
        stack.extend(children.into_iter().map(|child| (child, false)));
    }
}

fn despawn_find_children(world: &mut World, parent: Entity) -> Vec<Entity> {
    if let Some(children) = world.get::<Children>(parent) {
        return children.0.clone();
    }
    let mut query = world.query::<(Entity, &ChildOf)>();
    query
        .iter(world)
        .filter_map(|(child, co)| (co.0 == parent).then_some(child))
        .collect()
}

fn buy_store_visibility_check(world: &World) -> Option<(bool, Vec2, Vec2, bool, StashGrid)> {
    let visible = world.get_resource::<StashVisible>()?;
    let mouse = world.get_resource::<MouseState>()?;
    let grid = world.resource::<StashGrid>().clone();
    Some((visible.0, mouse.screen_pos(), mouse.world_pos(), mouse.just_pressed(MouseButton::Left), grid))
}

fn spawn_purchased_device(world: &mut World, item: StoreItemKind, spawn_pos: Vec2) {
    let drag_info = |entity| DeviceDragInfo {
        entity,
        grab_offset: Vec2::ZERO,
    };
    match item {
        StoreItemKind::Reader => {
            let entity = spawn_reader_purchase(world, spawn_pos);
            world.resource_mut::<ReaderDragState>().dragging = Some(drag_info(entity));
        }
        StoreItemKind::Screen => {
            let entity = spawn_screen_purchase(world, spawn_pos);
            world.resource_mut::<ScreenDragState>().dragging = Some(drag_info(entity));
        }
        StoreItemKind::Combiner => {
            let entity = spawn_combiner_purchase(world, spawn_pos);
            world.resource_mut::<CombinerDragState>().dragging = Some(drag_info(entity));
        }
        StoreItemKind::BoosterMachine => {
            let entity = spawn_booster_purchase(world, spawn_pos);
            world.resource_mut::<BoosterDragState>().dragging = Some(drag_info(entity));
        }
    }
}

pub fn store_buy_system(world: &mut World) {
    let Some((visible, screen_pos, world_pos, pressed, grid)) = buy_store_visibility_check(world) else {
        return;
    };
    if !visible || !pressed || !grid.is_store_page() {
        return;
    }
    if active_store_drag_target(world).is_some() || !stash_ui_contains(screen_pos, &grid) {
        return;
    }

    let Some(catalog) = world.get_resource::<StoreCatalog>() else {
        return;
    };
    let Some(item) = store_item_at(screen_pos, &grid, catalog) else {
        return;
    };

    let cost = item.cost();
    if !world.resource_mut::<StoreWallet>().spend(cost) {
        return;
    }

    spawn_purchased_device(world, item, world_pos);
}

fn sell_visibility_check(world: &World) -> Option<(bool, Vec2, StashGrid)> {
    let visible = world.get_resource::<StashVisible>()?;
    let mouse = world.get_resource::<MouseState>()?;
    let grid = world.resource::<StashGrid>().clone();
    Some((visible.0, mouse.screen_pos(), grid))
}

fn sell_dispatch(world: &mut World, target: StoreDragTarget) {
    match target {
        StoreDragTarget::Reader(entity) => {
            sell_reader(world, entity);
            world.resource_mut::<ReaderDragState>().dragging = None;
        }
        StoreDragTarget::Screen(entity) => {
            sell_screen(world, entity);
            world.resource_mut::<ScreenDragState>().dragging = None;
        }
        StoreDragTarget::Combiner(entity) => {
            sell_combiner(world, entity);
            world.resource_mut::<CombinerDragState>().dragging = None;
        }
        StoreDragTarget::Booster(entity) => {
            sell_booster(world, entity);
            world.resource_mut::<BoosterDragState>().dragging = None;
        }
        StoreDragTarget::Any => {}
    }
}

pub fn store_sell_system(world: &mut World) {
    let Some((visible, screen_pos, grid)) = sell_visibility_check(world) else {
        return;
    };
    if !visible || !world.get_resource::<MouseState>().is_some_and(|m| m.just_released(MouseButton::Left)) {
        return;
    }
    if !grid.is_store_page() || !stash_ui_contains(screen_pos, &grid) {
        return;
    }

    if let Some(target) = active_store_drag_target(world) {
        sell_dispatch(world, target);
    }
}

fn sell_device_basic(world: &mut World, entity: Entity, jack_entities: &[Entity], refund: u32) {
    despawn_connected_cables(world, jack_entities);
    for &jack in jack_entities {
        despawn_entity_tree(world, jack);
    }
    despawn_entity_tree(world, entity);
    world.resource_mut::<StoreWallet>().refund(refund);
}

fn sell_screen(world: &mut World, entity: Entity) {
    let device = match world.get::<crate::card::screen_device::ScreenDevice>(entity) {
        Some(d) => d,
        None => return,
    };
    sell_device_basic(world, entity, &[entity, device.signature_input], StoreItemKind::Screen.refund_value());
}

fn sell_combiner(world: &mut World, entity: Entity) {
    let (in_a, in_b, out) = match world.get::<CombinerDevice>(entity) {
        Some(d) => (d.input_a, d.input_b, d.output),
        None => return,
    };
    let refund = StoreItemKind::Combiner.refund_value();
    despawn_connected_cables(world, &[in_a, in_b, out]);
    for jack in [in_a, in_b, out] {
        despawn_entity_tree(world, jack);
    }
    despawn_entity_tree(world, entity);
    world.resource_mut::<StoreWallet>().refund(refund);
}

fn sell_reader(world: &mut World, entity: Entity) {
    let (loaded, jack_entity) = match world.get::<crate::card::reader::CardReader>(entity) {
        Some(r) => (r.loaded, r.jack_entity),
        None => return,
    };
    let refund = StoreItemKind::Reader.refund_value();

    if let Some(card_entity) = loaded {
        eject_card_to_table(world, entity, card_entity);
    }

    sell_device_basic(world, entity, &[entity, jack_entity], refund);
}

fn eject_card_to_table(world: &mut World, reader_entity: Entity, card_entity: Entity) {
    let reader_pos = world
        .get::<Transform2D>(reader_entity)
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
        bus.push(PhysicsCommand::RemoveBody { entity: card_entity });
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

fn sell_booster(world: &mut World, entity: Entity) {
    let (input_jack, btn, output_pack) = match world.get::<BoosterMachine>(entity) {
        Some(m) => (m.signal_input, m.button_entity, m.output_pack),
        None => return,
    };
    let refund = StoreItemKind::BoosterMachine.refund_value();
    despawn_connected_cables(world, &[entity, input_jack]);
    despawn_entity_tree(world, input_jack);
    despawn_entity_tree(world, btn);
    if let Some(pack) = output_pack {
        despawn_entity_tree(world, pack);
    }
    despawn_entity_tree(world, entity);
    world.resource_mut::<StoreWallet>().refund(refund);
}

