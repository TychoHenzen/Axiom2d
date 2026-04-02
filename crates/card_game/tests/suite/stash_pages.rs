#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::{Query, Res, ResMut, Schedule, World};
use card_game::stash::constants::{
    BACKGROUND_COLOR, GRID_MARGIN, SLOT_GAP, SLOT_STRIDE_H, SLOT_STRIDE_W,
};
use card_game::stash::grid::StashGrid;
use card_game::stash::pages::{
    TAB_ACTIVE, TAB_GAP, TAB_HEIGHT, TAB_INACTIVE, TAB_WIDTH, stash_tab_click_system,
    stash_tab_render_system, tab_left_x, tab_row_top_y,
};
use card_game::stash::toggle::StashVisible;
use engine_input::prelude::MouseButton;
use engine_render::prelude::{Camera2D, RendererRes};
use engine_render::testing::{ShapeCallLog, SpyRenderer};
use glam::Vec2;
use std::sync::{Arc, Mutex};

fn make_click_world(page_count: u8, visible: bool) -> World {
    let mut world = World::new();
    world.insert_resource(StashGrid::new(5, 5, page_count));
    world.insert_resource(StashVisible(visible));
    world.insert_resource(engine_input::prelude::MouseState::default());
    world
}

fn run_click_system(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(stash_tab_click_system);
    schedule.run(world);
}

/// Grid params (width=5, height=5) used by `make_click_world`.
const GRID_W: u8 = 5;
const GRID_H: u8 = 5;

fn click_at(world: &mut World, x: f32, y: f32) {
    let mut mouse = engine_input::prelude::MouseState::default();
    mouse.set_screen_pos(Vec2::new(x, y));
    mouse.press(MouseButton::Left);
    world.insert_resource(mouse);
}

fn tab_center(tab_index: u8, page_count: u8) -> (f32, f32) {
    let left = tab_left_x(GRID_W, page_count, tab_index);
    let top = tab_row_top_y(GRID_H);
    (left + TAB_WIDTH / 2.0, top + TAB_HEIGHT / 2.0)
}

#[test]
fn when_click_on_second_tab_then_switches_to_page_one() {
    // Arrange
    let mut world = make_click_world(3, true);
    let (cx, cy) = tab_center(1, 3);
    click_at(&mut world, cx, cy);

    // Act
    run_click_system(&mut world);

    // Assert
    assert_eq!(world.resource::<StashGrid>().current_page(), 1);
}

#[test]
fn when_click_on_third_tab_then_switches_to_page_two() {
    // Arrange
    let mut world = make_click_world(3, true);
    let (cx, cy) = tab_center(2, 3);
    click_at(&mut world, cx, cy);

    // Act
    run_click_system(&mut world);

    // Assert
    assert_eq!(world.resource::<StashGrid>().current_page(), 2);
}

#[test]
fn when_click_on_first_tab_then_stays_on_page_zero() {
    // Arrange
    let mut world = make_click_world(3, true);
    let (cx, cy) = tab_center(0, 3);
    click_at(&mut world, cx, cy);

    // Act
    run_click_system(&mut world);

    // Assert
    assert_eq!(world.resource::<StashGrid>().current_page(), 0);
}

/// @doc: Tab gap pixels don't change pages — only tab rectangles are clickable, preventing accidental page flips.
#[test]
fn when_click_between_tabs_then_page_unchanged() {
    // Arrange
    let mut world = make_click_world(3, true);
    let left0 = tab_left_x(GRID_W, 3, 0);
    let top = tab_row_top_y(GRID_H);
    click_at(
        &mut world,
        left0 + TAB_WIDTH + TAB_GAP / 2.0,
        top + TAB_HEIGHT / 2.0,
    );

    // Act
    run_click_system(&mut world);

    // Assert
    assert_eq!(world.resource::<StashGrid>().current_page(), 0);
}

#[test]
fn when_click_above_tabs_then_page_unchanged() {
    // Arrange
    let mut world = make_click_world(3, true);
    let left = tab_left_x(GRID_W, 3, 1);
    let top = tab_row_top_y(GRID_H);
    click_at(&mut world, left + TAB_WIDTH / 2.0, top - 2.0);

    // Act
    run_click_system(&mut world);

    // Assert
    assert_eq!(world.resource::<StashGrid>().current_page(), 0);
}

#[test]
fn when_click_below_tabs_then_page_unchanged() {
    // Arrange
    let mut world = make_click_world(3, true);
    let left = tab_left_x(GRID_W, 3, 1);
    let top = tab_row_top_y(GRID_H);
    click_at(&mut world, left + TAB_WIDTH / 2.0, top + TAB_HEIGHT + 2.0);

    // Act
    run_click_system(&mut world);

    // Assert
    assert_eq!(world.resource::<StashGrid>().current_page(), 0);
}

/// @doc: Hidden stash blocks tab clicks — prevents page switching when UI is not visible.
#[test]
fn when_stash_hidden_and_click_on_tab_then_page_unchanged() {
    // Arrange
    let mut world = make_click_world(3, false);
    let (cx, cy) = tab_center(1, 3);
    click_at(&mut world, cx, cy);

    // Act
    run_click_system(&mut world);

    // Assert
    assert_eq!(world.resource::<StashGrid>().current_page(), 0);
}

#[test]
fn when_no_click_then_page_unchanged() {
    // Arrange
    let mut world = make_click_world(3, true);
    world.resource_mut::<StashGrid>().set_current_page(1);

    // Act
    run_click_system(&mut world);

    // Assert
    assert_eq!(world.resource::<StashGrid>().current_page(), 1);
}

#[test]
fn when_right_click_on_tab_then_page_unchanged() {
    // Arrange
    let mut world = make_click_world(3, true);
    let (cx, cy) = tab_center(1, 3);
    let mut mouse = engine_input::prelude::MouseState::default();
    mouse.set_screen_pos(Vec2::new(cx, cy));
    mouse.press(MouseButton::Right);
    world.insert_resource(mouse);

    // Act
    run_click_system(&mut world);

    // Assert
    assert_eq!(world.resource::<StashGrid>().current_page(), 0);
}

fn make_render_world(
    grid: StashGrid,
    visible: bool,
    viewport: (u32, u32),
) -> (World, ShapeCallLog) {
    let mut world = World::new();
    world.insert_resource(grid);
    world.insert_resource(StashVisible(visible));

    let log = Arc::new(Mutex::new(Vec::new()));
    let shape_calls: ShapeCallLog = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(log)
        .with_shape_capture(shape_calls.clone())
        .with_viewport(viewport.0, viewport.1);
    world.insert_resource(RendererRes::new(Box::new(spy)));

    world.spawn(Camera2D {
        position: Vec2::ZERO,
        zoom: 1.0,
    });

    (world, shape_calls)
}

fn run_render_system(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(stash_tab_render_system);
    schedule.run(world);
}

#[test]
fn when_stash_hidden_then_no_tab_shapes_drawn() {
    // Arrange
    let (mut world, shape_calls) = make_render_world(StashGrid::new(5, 5, 3), false, (1024, 768));

    // Act
    run_render_system(&mut world);

    // Assert
    assert!(shape_calls.lock().unwrap().is_empty());
}

#[test]
fn when_viewport_zero_then_no_tab_shapes_drawn() {
    // Arrange
    let (mut world, shape_calls) = make_render_world(StashGrid::new(5, 5, 3), true, (0, 768));

    // Act
    run_render_system(&mut world);

    // Assert
    assert!(shape_calls.lock().unwrap().is_empty());
}

#[test]
fn when_three_pages_then_three_tab_shapes_drawn() {
    // Arrange
    let (mut world, shape_calls) = make_render_world(StashGrid::new(5, 5, 3), true, (1024, 768));

    // Act
    run_render_system(&mut world);

    // Assert
    assert_eq!(shape_calls.lock().unwrap().len(), 3);
}

#[test]
fn when_single_page_then_one_tab_shape_drawn() {
    // Arrange
    let (mut world, shape_calls) = make_render_world(StashGrid::new(5, 5, 1), true, (1024, 768));

    // Act
    run_render_system(&mut world);

    // Assert
    assert_eq!(shape_calls.lock().unwrap().len(), 1);
}

/// @doc: Active tab visually distinct from inactive — users see which page they're on.
#[test]
fn when_on_page_zero_then_first_tab_is_active_color() {
    // Arrange
    let (mut world, shape_calls) = make_render_world(StashGrid::new(5, 5, 3), true, (1024, 768));

    // Act
    run_render_system(&mut world);

    // Assert
    let calls = shape_calls.lock().unwrap();
    assert_eq!(calls[0].2, TAB_ACTIVE);
    assert_eq!(calls[1].2, TAB_INACTIVE);
    assert_eq!(calls[2].2, TAB_INACTIVE);
}

#[test]
fn when_on_page_one_then_middle_tab_is_active_color() {
    // Arrange
    let mut grid = StashGrid::new(5, 5, 3);
    grid.set_current_page(1);
    let (mut world, shape_calls) = make_render_world(grid, true, (1024, 768));

    // Act
    run_render_system(&mut world);

    // Assert
    let calls = shape_calls.lock().unwrap();
    assert_eq!(calls[0].2, TAB_INACTIVE);
    assert_eq!(calls[1].2, TAB_ACTIVE);
    assert_eq!(calls[2].2, TAB_INACTIVE);
}

#[test]
fn when_on_last_page_then_last_tab_is_active_color() {
    // Arrange
    let mut grid = StashGrid::new(5, 5, 3);
    grid.set_current_page(2);
    let (mut world, shape_calls) = make_render_world(grid, true, (1024, 768));

    // Act
    run_render_system(&mut world);

    // Assert
    let calls = shape_calls.lock().unwrap();
    assert_eq!(calls[0].2, TAB_INACTIVE);
    assert_eq!(calls[1].2, TAB_INACTIVE);
    assert_eq!(calls[2].2, TAB_ACTIVE);
}

#[test]
fn when_tabs_rendered_then_positioned_below_grid_bottom() {
    // Arrange
    let grid = StashGrid::new(5, 4, 3);
    let expected_top = tab_row_top_y(grid.height());
    let (mut world, shape_calls) = make_render_world(grid, true, (1024, 768));

    // Act
    run_render_system(&mut world);

    // Assert
    let calls = shape_calls.lock().unwrap();
    let camera = Camera2D::default();
    for (i, call) in calls.iter().enumerate() {
        let screen = engine_render::prelude::world_to_screen(
            Vec2::new(call.0[0][0], call.0[0][1]),
            &camera,
            1024.0,
            768.0,
        );
        assert!(
            (screen.y - expected_top).abs() < 1.0,
            "tab {i} screen_y={}, expected near {expected_top}",
            screen.y
        );
    }
}

/// @doc: Tab group centered under grid — ensures tab UI stays visually aligned with stash regardless of grid width.
#[test]
fn when_tabs_rendered_then_centered_under_grid() {
    // Arrange
    let grid = StashGrid::new(5, 4, 3);
    let grid_screen_w = f32::from(grid.width()) * SLOT_STRIDE_W - SLOT_GAP;
    let grid_center_x = GRID_MARGIN + grid_screen_w / 2.0;
    let (mut world, shape_calls) = make_render_world(grid, true, (1024, 768));

    // Act
    run_render_system(&mut world);

    // Assert
    let calls = shape_calls.lock().unwrap();
    let camera = Camera2D::default();
    let first_screen = engine_render::prelude::world_to_screen(
        Vec2::new(calls[0].0[0][0], calls[0].0[0][1]),
        &camera,
        1024.0,
        768.0,
    );
    let last_screen = engine_render::prelude::world_to_screen(
        Vec2::new(calls[2].0[0][0], calls[2].0[0][1]),
        &camera,
        1024.0,
        768.0,
    );
    let tab_group_mid_x = (first_screen.x + last_screen.x + TAB_WIDTH) / 2.0;
    assert!(
        (tab_group_mid_x - grid_center_x).abs() < 1.0,
        "tab midpoint {tab_group_mid_x} should be near grid center {grid_center_x}"
    );
}

/// @doc: Tab spacing uniform — prevents compression or stretching artifacts when pages > 2.
#[test]
fn when_tabs_rendered_then_adjacent_tabs_evenly_spaced() {
    // Arrange
    let (mut world, shape_calls) = make_render_world(StashGrid::new(5, 5, 3), true, (1024, 768));

    // Act
    run_render_system(&mut world);

    // Assert
    let calls = shape_calls.lock().unwrap();
    let x0 = calls[0].0[0][0];
    let x1 = calls[1].0[0][0];
    let x2 = calls[2].0[0][0];
    let dx01 = x1 - x0;
    let dx12 = x2 - x1;
    assert!(
        (dx01 - dx12).abs() < 0.01,
        "spacing should be uniform: dx01={dx01}, dx12={dx12}"
    );
}
