use bevy_ecs::prelude::{Entity, World};

pub(crate) fn spawn_entity() -> Entity {
    World::new().spawn_empty().id()
}
