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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hierarchy::Children;
    use crate::test_helpers::run_hierarchy_system;

    #[test]
    fn when_spawn_child_called_then_new_entity_has_child_of_pointing_to_parent() {
        // Arrange
        let mut world = World::new();
        let parent = world.spawn_empty().id();

        // Act
        let child = world.spawn_child(parent, ());

        // Assert
        let child_of = world
            .get::<ChildOf>(child)
            .expect("child should have ChildOf");
        assert_eq!(child_of.0, parent);
    }

    #[derive(Component)]
    struct Marker;

    #[test]
    fn when_spawn_child_called_then_new_entity_also_contains_the_provided_bundle() {
        // Arrange
        let mut world = World::new();
        let parent = world.spawn_empty().id();

        // Act
        let child = world.spawn_child(parent, Marker);

        // Assert
        assert!(world.get::<ChildOf>(child).is_some());
        assert!(world.get::<Marker>(child).is_some());
    }

    #[test]
    fn when_spawn_child_used_then_hierarchy_system_picks_up_the_new_child() {
        // Arrange
        let mut world = World::new();
        let parent = world.spawn_empty().id();
        let child = world.spawn_child(parent, ());

        // Act
        run_hierarchy_system(&mut world);

        // Assert
        let children = world
            .get::<Children>(parent)
            .expect("parent should have Children");
        assert_eq!(children.0, vec![child]);
    }
}
