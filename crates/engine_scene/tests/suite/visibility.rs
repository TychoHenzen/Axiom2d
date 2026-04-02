#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::*;
use bevy_ecs::schedule::IntoScheduleConfigs;
use engine_scene::hierarchy::{ChildOf, Children, hierarchy_maintenance_system};
use engine_scene::visibility::{EffectiveVisibility, Visible, visibility_system};

fn run_hierarchy_system(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(hierarchy_maintenance_system);
    schedule.run(world);
}

fn run_visibility_system(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(visibility_system);
    schedule.run(world);
}

#[derive(Resource, Default)]
struct ChangedEffectiveVisibilityCapture(usize);

fn capture_changed_effective_visibility(
    mut capture: ResMut<ChangedEffectiveVisibilityCapture>,
    query: Query<Entity, Changed<EffectiveVisibility>>,
) {
    capture.0 = query.iter().count();
}

#[test]
fn when_root_entity_has_default_visible_then_visibility_system_inserts_effective_visibility_true() {
    // Arrange
    let mut world = World::new();
    let entity = world.spawn(Visible::default()).id();

    // Act
    run_visibility_system(&mut world);

    // Assert
    let effective = world.get::<EffectiveVisibility>(entity).unwrap();
    assert_eq!(*effective, EffectiveVisibility(true));
}

#[test]
fn when_root_entity_has_visible_false_then_visibility_system_inserts_effective_visibility_false() {
    // Arrange
    let mut world = World::new();
    let entity = world.spawn(Visible(false)).id();

    // Act
    run_visibility_system(&mut world);

    // Assert
    let effective = world.get::<EffectiveVisibility>(entity).unwrap();
    assert_eq!(*effective, EffectiveVisibility(false));
}

/// @doc: Visible is opt-in — entities without it default to visible (no component = no hiding)
#[test]
fn when_root_entity_has_no_visible_component_then_visibility_system_inserts_effective_visibility_true()
 {
    // Arrange
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    // Act
    run_visibility_system(&mut world);

    // Assert
    let effective = world.get::<EffectiveVisibility>(entity).unwrap();
    assert_eq!(*effective, EffectiveVisibility(true));
}

#[test]
fn when_visible_parent_has_visible_child_then_child_effective_visibility_is_true() {
    // Arrange
    let mut world = World::new();
    let parent = world.spawn(Visible(true)).id();
    let child = world.spawn((Visible(true), ChildOf(parent))).id();
    world.entity_mut(parent).insert(Children(vec![child]));

    // Act
    run_visibility_system(&mut world);

    // Assert
    let effective = world.get::<EffectiveVisibility>(child).unwrap();
    assert_eq!(*effective, EffectiveVisibility(true));
}

/// @doc: AND-logic propagation: `EffectiveVisibility` = `parent_effective` AND `child_visible`
#[test]
fn when_parent_is_hidden_and_child_is_visible_then_child_effective_visibility_is_false() {
    // Arrange
    let mut world = World::new();
    let parent = world.spawn(Visible(false)).id();
    let child = world.spawn((Visible(true), ChildOf(parent))).id();
    world.entity_mut(parent).insert(Children(vec![child]));

    // Act
    run_visibility_system(&mut world);

    // Assert
    let effective = world.get::<EffectiveVisibility>(child).unwrap();
    assert_eq!(*effective, EffectiveVisibility(false));
}

#[test]
fn when_parent_is_visible_and_child_is_hidden_then_child_effective_visibility_is_false() {
    // Arrange
    let mut world = World::new();
    let parent = world.spawn(Visible(true)).id();
    let child = world.spawn((Visible(false), ChildOf(parent))).id();
    world.entity_mut(parent).insert(Children(vec![child]));

    // Act
    run_visibility_system(&mut world);

    // Assert
    let effective = world.get::<EffectiveVisibility>(child).unwrap();
    assert_eq!(*effective, EffectiveVisibility(false));
}

#[test]
fn when_three_level_hierarchy_with_hidden_root_then_grandchild_effective_visibility_is_false() {
    // Arrange
    let mut world = World::new();
    let root = world.spawn(Visible(false)).id();
    let child = world.spawn((Visible(true), ChildOf(root))).id();
    let grandchild = world.spawn((Visible(true), ChildOf(child))).id();
    world.entity_mut(root).insert(Children(vec![child]));
    world.entity_mut(child).insert(Children(vec![grandchild]));

    // Act
    run_visibility_system(&mut world);

    // Assert
    let effective = world.get::<EffectiveVisibility>(grandchild).unwrap();
    assert_eq!(*effective, EffectiveVisibility(false));
}

#[test]
fn when_two_siblings_one_hidden_then_each_gets_independent_effective_visibility() {
    // Arrange
    let mut world = World::new();
    let parent = world.spawn(Visible(true)).id();
    let child_a = world.spawn((Visible(true), ChildOf(parent))).id();
    let child_b = world.spawn((Visible(false), ChildOf(parent))).id();
    world
        .entity_mut(parent)
        .insert(Children(vec![child_a, child_b]));

    // Act
    run_visibility_system(&mut world);

    // Assert
    let a_effective = world.get::<EffectiveVisibility>(child_a).unwrap();
    assert_eq!(*a_effective, EffectiveVisibility(true));
    let b_effective = world.get::<EffectiveVisibility>(child_b).unwrap();
    assert_eq!(*b_effective, EffectiveVisibility(false));
}

#[test]
fn when_hierarchy_system_runs_before_visibility_system_then_children_receive_effective_visibility()
{
    // Arrange
    let mut world = World::new();
    let parent = world.spawn(Visible(true)).id();
    let child = world.spawn((Visible(true), ChildOf(parent))).id();

    // Act
    run_hierarchy_system(&mut world);
    run_visibility_system(&mut world);

    // Assert
    let effective = world.get::<EffectiveVisibility>(child).unwrap();
    assert_eq!(*effective, EffectiveVisibility(true));
}

#[test]
fn when_parent_visibility_changed_and_system_reruns_then_child_effective_visibility_updates() {
    // Arrange
    let mut world = World::new();
    let parent = world.spawn(Visible(false)).id();
    let child = world.spawn((Visible(true), ChildOf(parent))).id();
    world.entity_mut(parent).insert(Children(vec![child]));
    run_visibility_system(&mut world);
    assert_eq!(
        *world.get::<EffectiveVisibility>(child).unwrap(),
        EffectiveVisibility(false)
    );
    world.entity_mut(parent).insert(Visible(true));

    // Act
    run_visibility_system(&mut world);

    // Assert
    let effective = world.get::<EffectiveVisibility>(child).unwrap();
    assert_eq!(*effective, EffectiveVisibility(true));
}

#[test]
fn when_child_has_no_visible_component_and_parent_is_visible_then_child_effective_visibility_is_true()
 {
    // Arrange
    let mut world = World::new();
    let parent = world.spawn(Visible(true)).id();
    let child = world.spawn(ChildOf(parent)).id();
    world.entity_mut(parent).insert(Children(vec![child]));

    // Act
    run_visibility_system(&mut world);

    // Assert
    let effective = world.get::<EffectiveVisibility>(child).unwrap();
    assert_eq!(*effective, EffectiveVisibility(true));
}

#[test]
fn when_child_has_no_visible_component_and_parent_is_hidden_then_child_effective_visibility_is_false()
 {
    // Arrange
    let mut world = World::new();
    let parent = world.spawn(Visible(false)).id();
    let child = world.spawn(ChildOf(parent)).id();
    world.entity_mut(parent).insert(Children(vec![child]));

    // Act
    run_visibility_system(&mut world);

    // Assert
    let effective = world.get::<EffectiveVisibility>(child).unwrap();
    assert_eq!(*effective, EffectiveVisibility(false));
}

#[test]
fn when_visibility_system_reruns_without_changes_then_effective_visibility_is_not_marked_changed() {
    // Arrange
    let mut world = World::new();
    let parent = world.spawn(Visible(true)).id();
    let child = world.spawn((Visible(true), ChildOf(parent))).id();
    world.entity_mut(parent).insert(Children(vec![child]));
    world.insert_resource(ChangedEffectiveVisibilityCapture::default());

    let mut schedule = Schedule::default();
    schedule.add_systems((visibility_system, capture_changed_effective_visibility).chain());
    schedule.run(&mut world);

    // Act
    schedule.run(&mut world);

    // Assert
    let changed = world.resource::<ChangedEffectiveVisibilityCapture>().0;
    assert!(
        changed == 0,
        "unchanged hierarchy should not rewrite EffectiveVisibility, but {changed} entities were marked changed"
    );

    let parent_effective = world.get::<EffectiveVisibility>(parent).unwrap();
    assert_eq!(*parent_effective, EffectiveVisibility(true));
    let child_effective = world.get::<EffectiveVisibility>(child).unwrap();
    assert_eq!(*child_effective, EffectiveVisibility(true));
}
