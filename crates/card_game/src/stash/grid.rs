use bevy_ecs::prelude::{Entity, Resource};
use glam::Vec2;
use std::collections::HashMap;

use crate::stash::constants::{GRID_MARGIN, SLOT_STRIDE_H, SLOT_STRIDE_W};

#[derive(Debug)]
pub struct SlotOccupied;

impl std::fmt::Display for SlotOccupied {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("slot is already occupied")
    }
}

impl std::error::Error for SlotOccupied {}

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
            page_count: page_count.max(1),
            current_page: 1,
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
        self.current_page = page.min(self.page_count);
    }

    /// Returns the zero-based storage page currently selected, or `None` on the store page.
    pub fn current_storage_page(&self) -> Option<u8> {
        self.current_page.checked_sub(1)
    }

    pub fn is_store_page(&self) -> bool {
        self.current_page == 0
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

    pub fn tab_count(&self) -> u8 {
        self.page_count.saturating_add(2)
    }

    pub fn add_storage_tab(&mut self) -> u8 {
        self.page_count = self.page_count.saturating_add(1);
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

pub fn find_stash_slot_at(screen_pos: Vec2, grid_width: u8, grid_height: u8) -> Option<(u8, u8)> {
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
