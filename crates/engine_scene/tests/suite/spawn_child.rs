#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::*;
use engine_scene::hierarchy::{ChildOf, Children, hierarchy_maintenance_system};
use engine_scene::spawn_child::SpawnChildExt;

fn run_hierarchy_system(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(hierarchy_maintenance_system);
    schedule.run(world);
}

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
