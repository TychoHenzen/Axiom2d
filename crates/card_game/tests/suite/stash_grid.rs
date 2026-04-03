#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::World;
use glam::Vec2;

use card_game::stash::grid::{StashGrid, find_stash_slot_at};

#[test]
fn when_placing_entity_in_empty_slot_then_get_returns_that_entity() {
    // Arrange
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut grid = StashGrid::new(10, 10, 1);

    // Act
    let place_result = grid.place(0, 3, 5, entity);
    let get_result = grid.get(0, 3, 5);

    // Assert
    assert_eq!(place_result.unwrap(), ());
    assert_eq!(get_result, Some(&entity));
}

#[test]
fn when_taking_an_empty_slot_then_returns_none() {
    // Arrange
    let grid = &mut StashGrid::new(10, 10, 1);

    // Act
    let result = grid.take(0, 0, 0);

    // Assert
    assert_eq!(result, None);
}

#[test]
fn when_getting_an_empty_slot_then_returns_none() {
    // Arrange
    let grid = StashGrid::new(10, 10, 1);

    // Act
    let result = grid.get(0, 0, 0);

    // Assert
    assert_eq!(result, None);
}

/// @doc: Occupied slot placement is rejected without overwriting — prevents silent card loss during drag-and-drop
#[test]
fn when_placing_on_occupied_slot_then_returns_error_and_preserves_original() {
    // Arrange
    let mut world = World::new();
    let first = world.spawn_empty().id();
    let second = world.spawn_empty().id();
    let mut grid = StashGrid::new(10, 10, 1);
    grid.place(0, 2, 2, first).unwrap();

    // Act
    let result = grid.place(0, 2, 2, second);

    // Assert
    assert!(result.is_err());
    assert_eq!(grid.get(0, 2, 2), Some(&first));
}

#[test]
fn when_taking_occupied_slot_then_returns_entity_and_slot_becomes_empty() {
    // Arrange
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut grid = StashGrid::new(10, 10, 1);
    grid.place(0, 1, 1, entity).unwrap();

    // Act
    let result = grid.take(0, 1, 1);

    // Assert
    assert_eq!(result, Some(entity));
    assert_eq!(grid.get(0, 1, 1), None);
}

#[test]
fn when_slot_taken_then_new_entity_can_be_placed_there() {
    // Arrange
    let mut world = World::new();
    let first = world.spawn_empty().id();
    let second = world.spawn_empty().id();
    let mut grid = StashGrid::new(10, 10, 1);
    grid.place(0, 0, 0, first).unwrap();
    grid.take(0, 0, 0);

    // Act
    let result = grid.place(0, 0, 0, second);

    // Assert
    assert_eq!(result.unwrap(), ());
    assert_eq!(grid.get(0, 0, 0), Some(&second));
}

#[test]
fn when_page_has_gap_then_first_empty_returns_first_unoccupied_slot() {
    // Arrange
    let mut world = World::new();
    let mut grid = StashGrid::new(3, 3, 1);
    grid.place(0, 0, 0, world.spawn_empty().id()).unwrap();
    grid.place(0, 0, 1, world.spawn_empty().id()).unwrap();

    // Act
    let result = grid.first_empty(0);

    // Assert
    assert_eq!(result, Some((0, 2)));
}

/// @doc: Column-major fill order — `first_empty` scans down columns then across, matching visual grid layout
#[test]
fn when_first_column_full_then_first_empty_wraps_to_next_column() {
    // Arrange
    let mut world = World::new();
    let mut grid = StashGrid::new(3, 2, 1);
    grid.place(0, 0, 0, world.spawn_empty().id()).unwrap();
    grid.place(0, 0, 1, world.spawn_empty().id()).unwrap();

    // Act
    let result = grid.first_empty(0);

    // Assert
    assert_eq!(result, Some((1, 0)));
}

#[test]
fn when_page_completely_full_then_first_empty_returns_none() {
    // Arrange
    let mut world = World::new();
    let mut grid = StashGrid::new(2, 2, 1);
    grid.place(0, 0, 0, world.spawn_empty().id()).unwrap();
    grid.place(0, 0, 1, world.spawn_empty().id()).unwrap();
    grid.place(0, 1, 0, world.spawn_empty().id()).unwrap();
    grid.place(0, 1, 1, world.spawn_empty().id()).unwrap();

    // Act
    let result = grid.first_empty(0);

    // Assert
    assert_eq!(result, None);
}

/// @doc: Pages are independent — `first_empty` only scans the requested page, not the entire grid
#[test]
fn when_first_empty_on_second_page_then_ignores_first_page() {
    // Arrange
    let mut world = World::new();
    let mut grid = StashGrid::new(2, 2, 2);
    grid.place(0, 0, 0, world.spawn_empty().id()).unwrap();
    grid.place(0, 0, 1, world.spawn_empty().id()).unwrap();
    grid.place(0, 1, 0, world.spawn_empty().id()).unwrap();
    grid.place(0, 1, 1, world.spawn_empty().id()).unwrap();

    // Act
    let result = grid.first_empty(1);

    // Assert
    assert_eq!(result, Some((0, 0)));
}

/// @doc: Grid bounds checking prevents placement outside the stash — guards against integer overflow in slot lookup.
#[test]
fn when_placing_with_col_out_of_bounds_then_slot_not_stored() {
    // Arrange
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut grid = StashGrid::new(10, 10, 1);

    // Act
    let _ = grid.place(0, 10, 0, entity);

    // Assert
    assert_eq!(grid.get(0, 10, 0), None);
}

/// @doc: Y-boundary validation is independent of X — prevents off-by-one bugs in row-vs-column bounds checking.
#[test]
fn when_placing_with_row_out_of_bounds_then_slot_not_stored() {
    // Arrange
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut grid = StashGrid::new(10, 10, 1);

    // Act
    let _ = grid.place(0, 0, 10, entity);

    // Assert
    assert_eq!(grid.get(0, 0, 10), None);
}

/// @doc: Page bounds checking is strict — invalid pages silently reject placements to prevent cross-page slot pollution.
#[test]
fn when_placing_on_invalid_page_then_slot_not_stored() {
    // Arrange
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut grid = StashGrid::new(10, 10, 2);

    // Act
    let _ = grid.place(2, 0, 0, entity);

    // Assert
    assert_eq!(grid.get(2, 0, 0), None);
}

#[test]
fn when_set_current_page_within_range_then_current_page_returns_it() {
    // Arrange
    let mut grid = StashGrid::new(10, 10, 3);

    // Act
    grid.set_current_page(2);

    // Assert
    assert_eq!(grid.current_page(), 2);
}

/// @doc: Page index clamping prevents out-of-bounds access — UI tab clicks can't create invalid page state
#[test]
fn when_set_current_page_beyond_count_then_clamped_to_last_page() {
    // Arrange
    let mut grid = StashGrid::new(10, 10, 3);

    // Act
    grid.set_current_page(5);

    // Assert
    assert_eq!(grid.current_page(), 3);
}

#[test]
fn when_set_current_page_on_single_page_grid_then_clamped_to_one() {
    // Arrange
    let mut grid = StashGrid::new(10, 10, 1);

    // Act
    grid.set_current_page(5);

    // Assert
    assert_eq!(grid.current_page(), 1);
}

/// @doc: Pages are truly isolated in coordinate space — allows full reuse of same (col, row) across page tabs without collision.
#[test]
fn when_placing_on_different_pages_then_same_coordinates_are_independent() {
    // Arrange
    let mut world = World::new();
    let entity_a = world.spawn_empty().id();
    let entity_b = world.spawn_empty().id();
    let mut grid = StashGrid::new(10, 10, 2);

    // Act
    grid.place(0, 5, 5, entity_a).unwrap();
    grid.place(1, 5, 5, entity_b).unwrap();

    // Assert
    assert_eq!(grid.get(0, 5, 5), Some(&entity_a));
    assert_eq!(grid.get(1, 5, 5), Some(&entity_b));
}

/// @doc: Cursor-to-grid mapping uses `GRID_MARGIN` offset and stride constants — ensures drag-and-drop hits the intended slot.
#[test]
fn when_cursor_at_slot_center_then_returns_correct_col_row() {
    // Arrange
    // col=1, row=2 center: x = 20 + 1*54 + 25 = 99.0, y = 20 + 2*79 + 37 = 195.0
    let screen_pos = Vec2::new(99.0, 195.0);

    // Act
    let result = find_stash_slot_at(screen_pos, 4, 5);

    // Assert
    assert_eq!(result, Some((1, 2)));
}

#[test]
fn when_cursor_inside_slot_but_not_at_center_then_returns_that_slot() {
    // Arrange
    let screen_pos = Vec2::new(130.0, 260.0);

    // Act
    let result = find_stash_slot_at(screen_pos, 5, 6);

    // Assert
    assert_eq!(result, Some((2, 3)));
}

#[test]
fn when_cursor_at_top_left_boundary_of_first_slot_then_returns_zero_zero() {
    // Arrange
    let screen_pos = Vec2::new(20.0, 20.0);

    // Act
    let result = find_stash_slot_at(screen_pos, 3, 4);

    // Assert
    assert_eq!(result, Some((0, 0)));
}

#[test]
fn when_cursor_in_lower_half_of_tall_slot_then_slot_is_hit() {
    // Arrange
    let screen_pos = Vec2::new(80.0, 238.0);

    // Act
    let result = find_stash_slot_at(screen_pos, 4, 5);

    // Assert
    assert_eq!(result, Some((1, 2)));
}

/// @doc: Gap pixels between slots still resolve to a slot — integer division absorbs the gap, so drops never miss
#[test]
fn when_cursor_in_gap_between_slots_then_snaps_to_adjacent_slot() {
    // Arrange
    let screen_pos = Vec2::new(80.0, 253.0);

    // Act
    let result = find_stash_slot_at(screen_pos, 4, 5);

    // Assert
    assert_eq!(result, Some((1, 2)));
}
