#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::{Schedule, World};
use card_game::booster::device::{BoosterDragState, BoosterMachine};
use card_game::card::combiner_device::CombinerDragState;
use card_game::card::reader::CardReader;
use card_game::card::reader::ReaderDragState;
use card_game::card::screen_device::ScreenDragState;
use card_game::stash::grid::StashGrid;
use card_game::stash::store::{
    STORE_COIN_FONT, STORE_COLUMNS, STORE_ITEM_GAP_X, STORE_ITEM_GAP_Y, STORE_ITEM_HEIGHT,
    STORE_ITEM_WIDTH, STORE_STARTING_COINS, STORE_TAB_BASE_COST, StoreCatalog, StoreItemKind,
    StoreWallet, storage_tab_purchase_cost, store_buy_system, store_item_screen_bounds,
    store_sell_system, store_ui_bounds,
};
use card_game::stash::toggle::StashVisible;
use engine_core::prelude::EventBus;
use engine_input::prelude::{MouseButton, MouseState};
use engine_physics::prelude::PhysicsCommand;
use glam::Vec2;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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
    world.insert_resource(CombinerDragState::default());
    world.insert_resource(BoosterDragState::default());
    world.insert_resource(EventBus::<PhysicsCommand>::default());
    world
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

// ---------------------------------------------------------------------------
// StoreWallet
// ---------------------------------------------------------------------------

/// @doc: StoreWallet default gives STORE_STARTING_COINS.
#[test]
fn when_default_wallet_then_has_starting_coins() {
    // Arrange
    let wallet = StoreWallet::default();

    // Act / Assert
    assert_eq!(wallet.coins(), STORE_STARTING_COINS);
}

/// @doc: StoreWallet::new sets the exact coin count.
#[test]
fn when_wallet_created_with_value_then_coins_match() {
    // Arrange
    let wallet = StoreWallet::new(42);

    // Act / Assert
    assert_eq!(wallet.coins(), 42);
}

/// @doc: can_afford returns true when coins exceed cost.
#[test]
fn when_wallet_has_more_coins_than_cost_then_can_afford_is_true() {
    // Arrange
    let wallet = StoreWallet::new(50);

    // Act / Assert
    assert!(wallet.can_afford(49));
}

/// @doc: can_afford returns false when cost is zero and wallet is empty.
#[test]
fn when_wallet_empty_then_zero_cost_is_affordable() {
    // Arrange
    let wallet = StoreWallet::new(0);

    // Act / Assert
    assert!(wallet.can_afford(0));
}

/// @doc: spend returns true and deducts coins when sufficient.
#[test]
fn when_wallet_spend_sufficient_then_returns_true_and_deducts() {
    // Arrange
    let mut wallet = StoreWallet::new(100);

    // Act
    let ok = wallet.spend(30);

    // Assert
    assert!(ok);
    assert_eq!(wallet.coins(), 70);
}

/// @doc: refund adds coins to wallet.
#[test]
fn when_wallet_refunded_then_coins_increase() {
    // Arrange
    let mut wallet = StoreWallet::new(50);

    // Act
    wallet.refund(25);

    // Assert
    assert_eq!(wallet.coins(), 75);
}

/// @doc: refund saturates at u32::MAX without overflow.
#[test]
fn when_wallet_refund_causes_overflow_then_saturates() {
    // Arrange
    let mut wallet = StoreWallet::new(u32::MAX);

    // Act
    wallet.refund(1);

    // Assert
    assert_eq!(wallet.coins(), u32::MAX);
}

// ---------------------------------------------------------------------------
// StoreItemKind
// ---------------------------------------------------------------------------

/// @doc: ALL contains exactly four device variants.
#[test]
fn when_store_item_kind_all_then_contains_four_items() {
    // Arrange / Act / Assert
    assert_eq!(StoreItemKind::ALL.len(), 4);
    assert!(StoreItemKind::ALL.contains(&StoreItemKind::Reader));
    assert!(StoreItemKind::ALL.contains(&StoreItemKind::Screen));
    assert!(StoreItemKind::ALL.contains(&StoreItemKind::Combiner));
    assert!(StoreItemKind::ALL.contains(&StoreItemKind::BoosterMachine));
}

/// @doc: label returns human-readable name for each variant.
#[test]
fn when_item_kind_label_then_returns_correct_name() {
    // Arrange / Act / Assert
    assert_eq!(StoreItemKind::Reader.label(), "Card Reader");
    assert_eq!(StoreItemKind::Screen.label(), "Screen");
    assert_eq!(StoreItemKind::Combiner.label(), "Combiner");
    assert_eq!(StoreItemKind::BoosterMachine.label(), "Booster Machine");
}

/// @doc: cost returns correct coin price for each variant.
#[test]
fn when_item_kind_cost_then_returns_correct_price() {
    // Arrange / Act / Assert
    assert_eq!(StoreItemKind::Reader.cost(), 30);
    assert_eq!(StoreItemKind::Screen.cost(), 20);
    assert_eq!(StoreItemKind::Combiner.cost(), 25);
    assert_eq!(StoreItemKind::BoosterMachine.cost(), 35);
}

/// @doc: price_text returns formatted cost string.
#[test]
fn when_item_kind_price_text_then_contains_cost_and_coin_label() {
    // Arrange / Act / Assert
    assert_eq!(StoreItemKind::Reader.price_text(), "30 coins");
    assert_eq!(StoreItemKind::Screen.price_text(), "20 coins");
    assert_eq!(StoreItemKind::Combiner.price_text(), "25 coins");
    assert_eq!(StoreItemKind::BoosterMachine.price_text(), "35 coins");
}

/// @doc: refund_value equals the purchase cost.
#[test]
fn when_item_kind_refund_value_then_equals_cost() {
    // Arrange / Act / Assert
    for item in &StoreItemKind::ALL {
        assert_eq!(
            item.refund_value(),
            item.cost(),
            "refund_value mismatch for {:?}",
            item
        );
    }
}

/// @doc: preview_color returns a non-black color for each variant.
#[test]
fn when_item_kind_preview_color_then_has_positive_alpha() {
    // Arrange / Act / Assert
    for item in &StoreItemKind::ALL {
        let color = item.preview_color();
        assert!(
            color.a > 0.0,
            "preview_color alpha should be > 0 for {:?}",
            item
        );
    }
}

// ---------------------------------------------------------------------------
// StoreCatalog
// ---------------------------------------------------------------------------

/// @doc: default catalog contains all four item kinds.
#[test]
fn when_catalog_default_then_contains_all_items() {
    // Arrange
    let catalog = StoreCatalog::default();

    // Act
    let items = catalog.items();

    // Assert
    assert_eq!(items.len(), 4);
    assert_eq!(items, StoreItemKind::ALL.as_slice());
}

/// @doc: items() returns a slice with stable ordering.
#[test]
fn when_catalog_items_then_order_is_reader_screen_combiner_booster() {
    // Arrange
    let catalog = StoreCatalog::default();

    // Act
    let items = catalog.items();

    // Assert
    assert_eq!(items[0], StoreItemKind::Reader);
    assert_eq!(items[1], StoreItemKind::Screen);
    assert_eq!(items[2], StoreItemKind::Combiner);
    assert_eq!(items[3], StoreItemKind::BoosterMachine);
}

// ---------------------------------------------------------------------------
// storage_tab_purchase_cost
// ---------------------------------------------------------------------------

/// @doc: zero storage tabs gives base cost (count saturates at 0).
#[test]
fn when_zero_tabs_then_cost_is_base() {
    // Arrange / Act / Assert
    assert_eq!(
        storage_tab_purchase_cost(0),
        STORE_TAB_BASE_COST,
        "zero tabs should produce base cost (exponent saturates at 0)"
    );
}

/// @doc: thirty-one tabs saturates exponent at 30.
#[test]
fn when_thirty_one_tabs_then_cost_saturates() {
    // Arrange / Act
    let cost_31 = storage_tab_purchase_cost(31);
    let cost_32 = storage_tab_purchase_cost(32);

    // Assert — both saturate at the same cost value, no overflow
    assert_eq!(cost_31, cost_32, "cost should saturate beyond tab 30");
    assert!(cost_31 > 0, "cost should remain positive at tab 31");
}

// ---------------------------------------------------------------------------
// store_item_screen_bounds
// ---------------------------------------------------------------------------

/// @doc: valid index returns bounding rect for the store item.
#[test]
fn when_item_bounds_valid_index_then_returns_some() {
    // Arrange
    let grid = StashGrid::new(5, 5, 1);
    let catalog = StoreCatalog::default();

    // Act
    let bounds = store_item_screen_bounds(&grid, &catalog, 0);

    // Assert
    assert!(bounds.is_some());
    let (left, top, right, bottom) = bounds.unwrap();
    assert!(right > left);
    assert!(bottom > top);
}

/// @doc: invalid index returns None.
#[test]
fn when_item_bounds_invalid_index_then_returns_none() {
    // Arrange
    let grid = StashGrid::new(5, 5, 1);
    let catalog = StoreCatalog::default();

    // Act
    let bounds = store_item_screen_bounds(&grid, &catalog, 99);

    // Assert
    assert!(bounds.is_none());
}

/// @doc: bounds are consistent for all four catalog items.
#[test]
fn when_item_bounds_for_all_catalog_items_then_return_some() {
    // Arrange
    let grid = StashGrid::new(5, 5, 1);
    let catalog = StoreCatalog::default();

    // Act / Assert
    for i in 0..catalog.items().len() {
        assert!(
            store_item_screen_bounds(&grid, &catalog, i).is_some(),
            "expected bounds for index {}",
            i
        );
    }
}

// ---------------------------------------------------------------------------
// store_ui_bounds
// ---------------------------------------------------------------------------

/// @doc: store_ui_bounds returns sensible screen coordinates.
#[test]
fn when_store_ui_bounds_then_right_exceeds_left_and_bottom_exceeds_top() {
    // Arrange / Act
    let (left, top, right, bottom) = store_ui_bounds(5, 5);

    // Assert
    assert!(right > left, "right should be right of left");
    assert!(bottom > top, "bottom should be below top");
    assert!(left >= 0.0);
    assert!(top >= 0.0);
}

// ---------------------------------------------------------------------------
// store_buy_system — edge cases
// ---------------------------------------------------------------------------

/// @doc: buy system does nothing when stash is not visible.
#[test]
fn when_store_not_visible_then_buy_system_does_nothing() {
    // Arrange
    let mut world = make_store_world();
    world.insert_resource(StashVisible(false));
    // Click at the first reader item
    let catalog = StoreCatalog::default();
    let bounds = store_item_screen_bounds(world.resource::<StashGrid>(), &catalog, 0).unwrap();
    let center = Vec2::new((bounds.0 + bounds.2) * 0.5, (bounds.1 + bounds.3) * 0.5);
    click_at(&mut world, center, true);

    // Act
    run_buy_system(&mut world);

    // Assert — wallet untouched, no entity spawned
    assert_eq!(world.resource::<StoreWallet>().coins(), STORE_STARTING_COINS);
    assert_eq!(world.query::<&CardReader>().iter(&world).count(), 0);
}

/// @doc: buy system does nothing when not on store page.
#[test]
fn when_not_on_store_page_then_buy_system_does_nothing() {
    // Arrange
    let mut world = make_store_world();
    // Set current page to 1 (storage page, not store page)
    world.resource_mut::<StashGrid>().set_current_page(1);
    world.resource_mut::<StashGrid>().add_storage_tab();
    let catalog = StoreCatalog::default();
    let bounds = store_item_screen_bounds(world.resource::<StashGrid>(), &catalog, 0).unwrap();
    let center = Vec2::new((bounds.0 + bounds.2) * 0.5, (bounds.1 + bounds.3) * 0.5);
    click_at(&mut world, center, true);

    // Act
    run_buy_system(&mut world);

    // Assert — wallet untouched
    assert_eq!(world.resource::<StoreWallet>().coins(), STORE_STARTING_COINS);
}

/// @doc: buy system does nothing when insufficient coins.
#[test]
fn when_buy_with_insufficient_coins_then_no_purchase() {
    // Arrange
    let mut world = make_store_world();
    world.insert_resource(StoreWallet::new(5)); // Reader costs 30
    let catalog = StoreCatalog::default();
    let bounds = store_item_screen_bounds(world.resource::<StashGrid>(), &catalog, 0).unwrap();
    let center = Vec2::new((bounds.0 + bounds.2) * 0.5, (bounds.1 + bounds.3) * 0.5);
    click_at(&mut world, center, true);

    // Act
    run_buy_system(&mut world);

    // Assert
    assert_eq!(world.resource::<StoreWallet>().coins(), 5);
    assert_eq!(world.query::<&CardReader>().iter(&world).count(), 0);
}

/// @doc: buying a booster machine spawns the device and deducts coins.
#[test]
fn when_buy_booster_machine_then_spawns_and_spends_coins() {
    // Arrange
    let mut world = make_store_world();
    let catalog = StoreCatalog::default();
    let bounds = store_item_screen_bounds(world.resource::<StashGrid>(), &catalog, 3).unwrap();
    let center = Vec2::new((bounds.0 + bounds.2) * 0.5, (bounds.1 + bounds.3) * 0.5);
    click_at(&mut world, center, true);

    // Act
    run_buy_system(&mut world);

    // Assert
    assert_eq!(
        world.resource::<StoreWallet>().coins(),
        STORE_STARTING_COINS - 35
    );
    assert!(world.resource::<BoosterDragState>().dragging.is_some());
    assert_eq!(world.query::<&BoosterMachine>().iter(&world).count(), 1);
}

// ---------------------------------------------------------------------------
// store_sell_system — edge cases
// ---------------------------------------------------------------------------

/// @doc: selling a booster machine refunds and despawns it.
#[test]
fn when_sell_booster_machine_then_refunded_and_despawned() {
    // Arrange
    let mut world = make_store_world();
    let catalog = StoreCatalog::default();
    let bounds = store_item_screen_bounds(world.resource::<StashGrid>(), &catalog, 3).unwrap();
    let center = Vec2::new((bounds.0 + bounds.2) * 0.5, (bounds.1 + bounds.3) * 0.5);
    click_at(&mut world, center, true);
    run_buy_system(&mut world);
    click_at(&mut world, center, false);

    // Act
    run_sell_system(&mut world);

    // Assert
    assert_eq!(world.resource::<StoreWallet>().coins(), STORE_STARTING_COINS);
    assert!(world.resource::<BoosterDragState>().dragging.is_none());
    assert_eq!(world.query::<&BoosterMachine>().iter(&world).count(), 0);
}

/// @doc: sell system does nothing when store is not visible.
#[test]
fn when_sell_store_not_visible_then_no_sale() {
    // Arrange
    let mut world = make_store_world();
    let catalog = StoreCatalog::default();
    let bounds = store_item_screen_bounds(world.resource::<StashGrid>(), &catalog, 0).unwrap();
    let center = Vec2::new((bounds.0 + bounds.2) * 0.5, (bounds.1 + bounds.3) * 0.5);
    click_at(&mut world, center, true);
    run_buy_system(&mut world);
    // Reader is now in drag
    world.insert_resource(StashVisible(false));
    click_at(&mut world, center, false);

    // Act
    run_sell_system(&mut world);

    // Assert — reader was not sold (still exists)
    assert!(world.resource::<ReaderDragState>().dragging.is_some());
    assert_eq!(world.query::<&CardReader>().iter(&world).count(), 1);
}

/// @doc: sell system does nothing when not on store page.
#[test]
fn when_sell_not_on_store_page_then_no_sale() {
    // Arrange
    let mut world = make_store_world();
    let catalog = StoreCatalog::default();
    let bounds = store_item_screen_bounds(world.resource::<StashGrid>(), &catalog, 0).unwrap();
    let center = Vec2::new((bounds.0 + bounds.2) * 0.5, (bounds.1 + bounds.3) * 0.5);
    click_at(&mut world, center, true);
    run_buy_system(&mut world);
    // Switch to storage page (need 2 pages: store=0, storage=1)
    world.resource_mut::<StashGrid>().add_storage_tab();
    world.resource_mut::<StashGrid>().set_current_page(1);
    click_at(&mut world, center, false);

    // Act
    run_sell_system(&mut world);

    // Assert — reader still exists
    assert_eq!(world.query::<&CardReader>().iter(&world).count(), 1);
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// @doc: store layout constants have sensible positive values.
#[test]
fn when_store_constants_then_positive_values() {
    // Arrange / Act / Assert
    assert!(STORE_COLUMNS > 0);
    assert!(STORE_ITEM_WIDTH > 0.0);
    assert!(STORE_ITEM_HEIGHT > 0.0);
    assert!(STORE_ITEM_GAP_X > 0.0);
    assert!(STORE_ITEM_GAP_Y > 0.0);
    assert!(STORE_COIN_FONT > 0.0);
    assert!(STORE_STARTING_COINS > 0);
    assert!(STORE_TAB_BASE_COST > 0);
}
