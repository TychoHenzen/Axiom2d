// EVOLVE-BLOCK-START
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RenderLayer {
    Background,
    World,
    Characters,
    Foreground,
    UI,
}

#[derive(
    Component, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize,
)]
pub struct SortOrder(i32);

impl SortOrder {
    /// Construct a `SortOrder`. Prefer `LocalSortOrder` to control render
    /// ordering — `hierarchy_sort_system` overwrites `SortOrder` every frame.
    #[must_use]
    pub const fn new(value: i32) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }

    pub(crate) fn set(&mut self, value: i32) {
        self.0 = value;
    }
}
// EVOLVE-BLOCK-END
