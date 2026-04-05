#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::{Schedule, World};
use card_game::card::jack_cable::Jack;
use card_game::card::reader::{CardReader, ReaderDragState};
use card_game::card::reader::{ReaderAccent, ReaderRecess, ReaderRune, SignatureSpace};
use card_game::card::screen_device::{ScreenDevice, ScreenDragState, ScreenSignalDot};
use card_game::stash::grid::StashGrid;
use card_game::stash::pages::{stash_tab_click_system, tab_left_x, tab_row_top_y};
use card_game::stash::store::{
    STORE_TAB_BASE_COST, StoreCatalog, StoreWallet, storage_tab_purchase_cost, store_buy_system,
    store_item_screen_bounds, store_sell_system,
};
use card_game::stash::toggle::StashVisible;
use engine_input::prelude::{MouseButton, MouseState};
use engine_render::font::measure_text;
use engine_render::prelude::{Camera2D, RendererRes, world_to_screen};
use engine_render::testing::{SpyRenderer, TextCallLog};
use glam::Vec2;
use std::sync::{Arc, Mutex};

fn run_buy_system(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(store_buy_system);
    schedule.run(world);
}

fn run_sell_system(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(store_sell_system);
    schedule.run(world);
}

fn run_tab_click_system(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(stash_tab_click_system);
    schedule.run(world);
}

fn run_store_render_system(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(card_game::stash::store::store_render_system);
    schedule.run(world);
}

fn make_store_world() -> World {
    let mut world = World::new();
    world.insert_resource(StashGrid::new(5, 5, 1));
    world.resource_mut::<StashGrid>().set_current_page(0);
    world.insert_resource(StashVisible(true));
    world.insert_resource(StoreWallet::default());
    world.insert_resource(StoreCatalog::default());
    world.insert_resource(MouseState::default());
    world.insert_resource(ReaderDragState::default());
    world.insert_resource(ScreenDragState::default());
    world
}

fn make_store_render_world() -> (World, TextCallLog) {
    let mut world = make_store_world();
    let log = Arc::new(Mutex::new(Vec::new()));
    let text_calls: TextCallLog = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(log)
        .with_text_capture(text_calls.clone())
        .with_viewport(1024, 768);
    world.insert_resource(RendererRes::new(Box::new(spy)));
    world.spawn(Camera2D {
        position: Vec2::ZERO,
        zoom: 1.0,
    });
    (world, text_calls)
}

fn click_at(world: &mut World, pos: Vec2, pressed: bool) {
    let mut mouse = MouseState::default();
    mouse.set_screen_pos(pos);
    mouse.set_world_pos(pos);
    if pressed {
        mouse.press(MouseButton::Left);
    } else {
        mouse.release(MouseButton::Left);
    }
    world.insert_resource(mouse);
}

#[test]
fn when_plus_tab_clicked_then_adds_storage_tab_and_selects_it() {
    // Arrange
    let mut world = make_store_world();
    let tab_count = world.resource::<StashGrid>().tab_count();
    let plus_index = tab_count - 1;
    let left = tab_left_x(world.resource::<StashGrid>().width(), tab_count, plus_index);
    let top = tab_row_top_y(world.resource::<StashGrid>().height());
    click_at(&mut world, Vec2::new(left + 1.0, top + 1.0), true);

    // Act
    run_tab_click_system(&mut world);

    // Assert
    let grid = world.resource::<StashGrid>();
    assert_eq!(grid.page_count(), 2);
    assert_eq!(grid.current_page(), 2);
    assert_eq!(
        world.resource::<StoreWallet>().coins(),
        100 - storage_tab_purchase_cost(1)
    );
}

#[test]
fn when_click_reader_tile_then_spawns_reader_copy_and_spends_coins() {
    // Arrange
    let mut world = make_store_world();
    let catalog = StoreCatalog::default();
    let bounds = store_item_screen_bounds(world.resource::<StashGrid>(), &catalog, 0).unwrap();
    let center = Vec2::new((bounds.0 + bounds.2) * 0.5, (bounds.1 + bounds.3) * 0.5);
    click_at(&mut world, center, true);

    // Act
    run_buy_system(&mut world);

    // Assert
    assert_eq!(world.resource::<StoreWallet>().coins(), 70);
    assert!(world.resource::<ReaderDragState>().dragging.is_some());
    assert_eq!(world.query::<&CardReader>().iter(&world).count(), 1);
}

#[test]
fn when_click_screen_tile_then_spawns_screen_copy_and_spends_coins() {
    // Arrange
    let mut world = make_store_world();
    let catalog = StoreCatalog::default();
    let bounds = store_item_screen_bounds(world.resource::<StashGrid>(), &catalog, 1).unwrap();
    let center = Vec2::new((bounds.0 + bounds.2) * 0.5, (bounds.1 + bounds.3) * 0.5);
    click_at(&mut world, center, true);

    // Act
    run_buy_system(&mut world);

    // Assert
    assert_eq!(world.resource::<StoreWallet>().coins(), 80);
    assert!(world.resource::<ScreenDragState>().dragging.is_some());
    assert_eq!(world.query::<&ScreenDevice>().iter(&world).count(), 1);
}

#[test]
fn when_reader_dragged_back_over_store_then_reader_is_sold_and_refunded() {
    // Arrange
    let mut world = make_store_world();
    let catalog = StoreCatalog::default();
    let bounds = store_item_screen_bounds(world.resource::<StashGrid>(), &catalog, 0).unwrap();
    let center = Vec2::new((bounds.0 + bounds.2) * 0.5, (bounds.1 + bounds.3) * 0.5);
    click_at(&mut world, center, true);
    run_buy_system(&mut world);
    assert!(world.resource::<ReaderDragState>().dragging.is_some());

    click_at(&mut world, center, false);

    // Act
    run_sell_system(&mut world);

    // Assert
    assert_eq!(world.resource::<StoreWallet>().coins(), 100);
    assert!(world.resource::<ReaderDragState>().dragging.is_none());
    assert_eq!(world.query::<&CardReader>().iter(&world).count(), 0);
}

#[test]
fn when_store_page_renders_then_item_prices_land_with_the_item_cards() {
    // Arrange
    let (mut world, text_calls) = make_store_render_world();

    // Act
    run_store_render_system(&mut world);

    // Assert
    let calls = text_calls.lock().unwrap();
    assert_eq!(calls.len(), 7);

    let catalog = StoreCatalog::default();
    let grid = world.resource::<StashGrid>();
    let camera = Camera2D::default();

    let reader_bounds = store_item_screen_bounds(grid, &catalog, 0).unwrap();
    let reader_label_screen =
        world_to_screen(Vec2::new(calls[3].1, calls[3].2), &camera, 1024.0, 768.0);
    let reader_price_screen =
        world_to_screen(Vec2::new(calls[4].1, calls[4].2), &camera, 1024.0, 768.0);
    let reader_width = reader_bounds.2 - reader_bounds.0;
    let reader_height = reader_bounds.3 - reader_bounds.1;
    let expected_reader_label_x =
        reader_bounds.0 + reader_width * 0.5 - measure_text("Card Reader", 15.0) * 0.5;
    let expected_reader_label_y = reader_bounds.1 + reader_height - 38.0;
    let expected_reader_price_x =
        reader_bounds.0 + reader_width * 0.5 - measure_text("30 coins", 14.0) * 0.5;
    let expected_reader_price_y = reader_bounds.1 + reader_height - 20.0;
    assert_eq!(calls[3].0, "Card Reader");
    assert_eq!(calls[4].0, "30 coins");
    assert!(
        (reader_label_screen.x - expected_reader_label_x).abs() < 1.0,
        "reader label x={} expected near {}",
        reader_label_screen.x,
        expected_reader_label_x
    );
    assert!(
        (reader_label_screen.y - expected_reader_label_y).abs() < 1.0,
        "reader label y={} expected near {}",
        reader_label_screen.y,
        expected_reader_label_y
    );
    assert!(
        (reader_price_screen.x - expected_reader_price_x).abs() < 1.0,
        "reader price x={} expected near {}",
        reader_price_screen.x,
        expected_reader_price_x
    );
    assert!(
        (reader_price_screen.y - expected_reader_price_y).abs() < 1.0,
        "reader price y={} expected near {}",
        reader_price_screen.y,
        expected_reader_price_y
    );

    let screen_bounds = store_item_screen_bounds(grid, &catalog, 1).unwrap();
    let screen_label_screen =
        world_to_screen(Vec2::new(calls[5].1, calls[5].2), &camera, 1024.0, 768.0);
    let screen_price_screen =
        world_to_screen(Vec2::new(calls[6].1, calls[6].2), &camera, 1024.0, 768.0);
    let screen_width = screen_bounds.2 - screen_bounds.0;
    let screen_height = screen_bounds.3 - screen_bounds.1;
    let expected_screen_label_x =
        screen_bounds.0 + screen_width * 0.5 - measure_text("Screen", 15.0) * 0.5;
    let expected_screen_label_y = screen_bounds.1 + screen_height - 38.0;
    let expected_screen_price_x =
        screen_bounds.0 + screen_width * 0.5 - measure_text("20 coins", 14.0) * 0.5;
    let expected_screen_price_y = screen_bounds.1 + screen_height - 20.0;
    assert_eq!(calls[5].0, "Screen");
    assert_eq!(calls[6].0, "20 coins");
    assert!(
        (screen_label_screen.x - expected_screen_label_x).abs() < 1.0,
        "screen label x={} expected near {}",
        screen_label_screen.x,
        expected_screen_label_x
    );
    assert!(
        (screen_label_screen.y - expected_screen_label_y).abs() < 1.0,
        "screen label y={} expected near {}",
        screen_label_screen.y,
        expected_screen_label_y
    );
    assert!(
        (screen_price_screen.x - expected_screen_price_x).abs() < 1.0,
        "screen price x={} expected near {}",
        screen_price_screen.x,
        expected_screen_price_x
    );
    assert!(
        (screen_price_screen.y - expected_screen_price_y).abs() < 1.0,
        "screen price y={} expected near {}",
        screen_price_screen.y,
        expected_screen_price_y
    );
}

#[test]
fn when_selling_reader_then_reader_tree_and_jack_are_removed() {
    // Arrange
    let mut world = make_store_world();
    let catalog = StoreCatalog::default();
    let bounds = store_item_screen_bounds(world.resource::<StashGrid>(), &catalog, 0).unwrap();
    let center = Vec2::new((bounds.0 + bounds.2) * 0.5, (bounds.1 + bounds.3) * 0.5);
    click_at(&mut world, center, true);
    run_buy_system(&mut world);
    click_at(&mut world, center, false);

    // Act
    run_sell_system(&mut world);

    // Assert
    assert_eq!(world.query::<&CardReader>().iter(&world).count(), 0);
    assert_eq!(
        world.query::<&Jack<SignatureSpace>>().iter(&world).count(),
        0
    );
    assert_eq!(world.query::<&ReaderRecess>().iter(&world).count(), 0);
    assert_eq!(world.query::<&ReaderAccent>().iter(&world).count(), 0);
    assert_eq!(world.query::<&ReaderRune>().iter(&world).count(), 0);
}

#[test]
fn when_selling_screen_then_screen_tree_and_jack_are_removed() {
    // Arrange
    let mut world = make_store_world();
    let catalog = StoreCatalog::default();
    let bounds = store_item_screen_bounds(world.resource::<StashGrid>(), &catalog, 1).unwrap();
    let center = Vec2::new((bounds.0 + bounds.2) * 0.5, (bounds.1 + bounds.3) * 0.5);
    click_at(&mut world, center, true);
    run_buy_system(&mut world);
    click_at(&mut world, center, false);

    // Act
    run_sell_system(&mut world);

    // Assert
    assert_eq!(world.query::<&ScreenDevice>().iter(&world).count(), 0);
    assert_eq!(
        world.query::<&Jack<SignatureSpace>>().iter(&world).count(),
        0
    );
    assert_eq!(world.query::<&ScreenSignalDot>().iter(&world).count(), 0);
}

// ---------------------------------------------------------------------------
// StoreWallet::can_afford boundary tests
// ---------------------------------------------------------------------------

#[test]
fn when_wallet_coins_less_than_cost_then_can_afford_is_false() {
    // Arrange
    let wallet = StoreWallet::new(10);

    // Act / Assert
    assert!(!wallet.can_afford(11));
}

#[test]
fn when_wallet_coins_equal_cost_then_can_afford_is_true() {
    // Arrange
    let wallet = StoreWallet::new(10);

    // Act / Assert
    assert!(wallet.can_afford(10));
}

#[test]
fn when_wallet_empty_and_spend_called_then_returns_false_and_coins_unchanged() {
    // Arrange
    let mut wallet = StoreWallet::new(0);

    // Act
    let result = wallet.spend(1);

    // Assert
    assert!(!result);
    assert_eq!(wallet.coins(), 0);
}

// ---------------------------------------------------------------------------
// storage_tab_purchase_cost — hardcoded-value tests (catch << vs >> mutation)
// ---------------------------------------------------------------------------

#[test]
fn when_one_storage_tab_exists_then_purchase_cost_equals_base_cost() {
    // exponent = 0, cost = 25 * 1 = 25
    assert_eq!(storage_tab_purchase_cost(1), STORE_TAB_BASE_COST);
}

#[test]
fn when_two_storage_tabs_exist_then_purchase_cost_is_twice_base() {
    // exponent = 1, cost = 25 * 2 = 50
    assert_eq!(storage_tab_purchase_cost(2), STORE_TAB_BASE_COST * 2);
}

#[test]
fn when_three_storage_tabs_exist_then_purchase_cost_is_four_times_base() {
    // exponent = 2, cost = 25 * 4 = 100
    assert_eq!(storage_tab_purchase_cost(3), STORE_TAB_BASE_COST * 4);
}

// ---------------------------------------------------------------------------
// Behavioral: insufficient coins blocks tab purchase
// ---------------------------------------------------------------------------

#[test]
fn when_wallet_empty_and_plus_tab_clicked_then_no_storage_tab_added() {
    // Arrange
    let mut world = make_store_world();
    world.insert_resource(StoreWallet::new(0));
    let (left, top) = {
        let grid = world.resource::<StashGrid>();
        let tab_count = grid.tab_count();
        let plus_index = tab_count - 1;
        let left = tab_left_x(grid.width(), tab_count, plus_index);
        let top = tab_row_top_y(grid.height());
        (left, top)
    };
    click_at(&mut world, Vec2::new(left + 1.0, top + 1.0), true);

    // Act
    run_tab_click_system(&mut world);

    // Assert
    assert_eq!(world.resource::<StashGrid>().page_count(), 1);
}
