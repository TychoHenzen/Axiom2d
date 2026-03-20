use bevy_ecs::prelude::{Entity, Resource};
use glam::Vec2;
use std::collections::HashMap;

use crate::stash::constants::{GRID_MARGIN, SLOT_STRIDE_H, SLOT_STRIDE_W};

#[derive(Debug, thiserror::Error)]
#[error("slot is already occupied")]
pub struct SlotOccupied;

#[derive(Resource, Debug, Clone, PartialEq)]
pub struct StashGrid {
    slots: HashMap<(u8, u8, u8), Entity>,
    width: u8,
    height: u8,
    page_count: u8,
    current_page: u8,
}

impl StashGrid {
    pub fn new(width: u8, height: u8, page_count: u8) -> Self {
        Self {
            slots: HashMap::new(),
            width,
            height,
            page_count,
            current_page: 0,
        }
    }

    pub fn place(
        &mut self,
        page: u8,
        col: u8,
        row: u8,
        entity: Entity,
    ) -> Result<(), SlotOccupied> {
        if col >= self.width || row >= self.height || page >= self.page_count {
            return Err(SlotOccupied);
        }
        let key = (page, col, row);
        if self.slots.contains_key(&key) {
            return Err(SlotOccupied);
        }
        self.slots.insert(key, entity);
        Ok(())
    }

    pub fn take(&mut self, page: u8, col: u8, row: u8) -> Option<Entity> {
        self.slots.remove(&(page, col, row))
    }

    pub fn get(&self, page: u8, col: u8, row: u8) -> Option<&Entity> {
        self.slots.get(&(page, col, row))
    }

    pub fn current_page(&self) -> u8 {
        self.current_page
    }

    pub fn set_current_page(&mut self, page: u8) {
        self.current_page = page.min(self.page_count.saturating_sub(1));
    }

    pub fn width(&self) -> u8 {
        self.width
    }

    pub fn height(&self) -> u8 {
        self.height
    }

    pub fn page_count(&self) -> u8 {
        self.page_count
    }

    pub fn first_empty(&self, page: u8) -> Option<(u8, u8)> {
        for col in 0..self.width {
            for row in 0..self.height {
                if !self.slots.contains_key(&(page, col, row)) {
                    return Some((col, row));
                }
            }
        }
        None
    }
}

pub(crate) fn cursor_over_stash(
    mouse: &engine_input::prelude::MouseState,
    visible: &super::toggle::StashVisible,
    grid: &StashGrid,
) -> bool {
    visible.0 && find_stash_slot_at(mouse.screen_pos(), grid.width(), grid.height()).is_some()
}

pub(crate) fn find_stash_slot_at(
    screen_pos: Vec2,
    grid_width: u8,
    grid_height: u8,
) -> Option<(u8, u8)> {
    let rel_x = screen_pos.x - GRID_MARGIN;
    let rel_y = screen_pos.y - GRID_MARGIN;
    if rel_x < 0.0 || rel_y < 0.0 {
        return None;
    }
    let col = (rel_x / SLOT_STRIDE_W) as u8;
    let row = (rel_y / SLOT_STRIDE_H) as u8;
    if col >= grid_width || row >= grid_height {
        return None;
    }
    Some((col, row))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::World;

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

    #[test]
    fn when_slot_occupied_error_displayed_then_message_is_human_readable() {
        // Arrange
        let err = SlotOccupied;

        // Act
        let message = format!("{err}");

        // Assert
        assert!(!message.is_empty());
    }

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
    fn when_width_called_then_returns_constructed_width() {
        // Arrange
        let grid = StashGrid::new(10, 8, 3);

        // Act / Assert
        assert_eq!(grid.width(), 10);
    }

    #[test]
    fn when_height_called_then_returns_constructed_height() {
        // Arrange
        let grid = StashGrid::new(10, 8, 3);

        // Act / Assert
        assert_eq!(grid.height(), 8);
    }

    #[test]
    fn when_page_count_called_then_returns_constructed_page_count() {
        // Arrange
        let grid = StashGrid::new(10, 8, 3);

        // Act / Assert
        assert_eq!(grid.page_count(), 3);
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

    #[test]
    fn when_set_current_page_beyond_count_then_clamped_to_last_page() {
        // Arrange
        let mut grid = StashGrid::new(10, 10, 3);

        // Act
        grid.set_current_page(5);

        // Assert
        assert_eq!(grid.current_page(), 2);
    }

    #[test]
    fn when_set_current_page_on_single_page_grid_then_stays_zero() {
        // Arrange
        let mut grid = StashGrid::new(10, 10, 1);

        // Act
        grid.set_current_page(1);

        // Assert
        assert_eq!(grid.current_page(), 0);
    }

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

    #[test]
    fn when_cursor_in_gap_between_slots_then_snaps_to_adjacent_slot() {
        // Arrange
        let screen_pos = Vec2::new(80.0, 253.0);

        // Act
        let result = find_stash_slot_at(screen_pos, 4, 5);

        // Assert
        assert_eq!(result, Some((1, 2)));
    }
}
