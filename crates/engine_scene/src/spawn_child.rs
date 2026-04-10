// EVOLVE-BLOCK-START
use bevy_ecs::prelude::*;

use crate::hierarchy::ChildOf;

pub trait SpawnChildExt {
    fn spawn_child(&mut self, parent: Entity, bundle: impl Bundle) -> Entity;
}

impl SpawnChildExt for World {
    fn spawn_child(&mut self, parent: Entity, bundle: impl Bundle) -> Entity {
        self.spawn((ChildOf(parent), bundle)).id()
    }
}
// EVOLVE-BLOCK-END
