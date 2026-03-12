pub mod collider;
pub mod collision_event;
pub mod physics_backend;
pub mod physics_res;
pub mod physics_step_system;
pub mod prelude;
pub mod rapier_backend;
pub mod rigid_body;

#[cfg(test)]
pub(crate) mod test_helpers {
    use bevy_ecs::prelude::{Entity, World};

    pub(crate) fn spawn_entity() -> Entity {
        World::new().spawn(()).id()
    }

    pub(crate) fn spawn_entities(count: usize) -> Vec<Entity> {
        let mut world = World::new();
        (0..count).map(|_| world.spawn(()).id()).collect()
    }
}
