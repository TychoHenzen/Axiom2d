use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CardZone {
    Table,
    Hand(usize),
    Stash { page: u8, col: u8, row: u8 },
}
