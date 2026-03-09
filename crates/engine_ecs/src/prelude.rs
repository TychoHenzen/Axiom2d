pub use bevy_ecs::prelude::{
    Added, Changed, Commands, Component, Entity, Query, Res, ResMut, Resource, Schedule, SystemSet,
    With, Without, World,
};

pub use crate::schedule::Phase;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_prelude_imported_then_world_entity_schedule_and_phase_resolve() {
        let _world = World::new();
        let _schedule = Schedule::new(Phase::Update);
    }

    #[test]
    fn when_prelude_imported_then_component_and_resource_derive_macros_are_accessible() {
        #[derive(Component)]
        struct Marker;

        #[derive(Resource)]
        struct Config(u32);

        // Arrange
        let mut world = World::new();

        // Act
        world.spawn(Marker);
        world.insert_resource(Config(1));

        // Assert
        assert_eq!(world.resource::<Config>().0, 1);
    }
}
