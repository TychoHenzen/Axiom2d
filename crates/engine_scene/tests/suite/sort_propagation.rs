#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::*;
use engine_scene::hierarchy::{ChildOf, hierarchy_maintenance_system};
use engine_scene::render_order::SortOrder;
use engine_scene::sort_propagation::{LocalSortOrder, hierarchy_sort_system};

fn run_hierarchy_system(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(hierarchy_maintenance_system);
    schedule.run(world);
}

fn run_hierarchy_sort_system(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(hierarchy_sort_system);
    schedule.run(world);
}

fn run_with_hierarchy(world: &mut World) {
    run_hierarchy_system(world);
    run_hierarchy_sort_system(world);
}

#[test]
fn when_single_root_then_sort_order_is_zero() {
    // Arrange
    let mut world = World::new();
    let root = world.spawn(SortOrder::new(99)).id();

    // Act
    run_with_hierarchy(&mut world);

    // Assert
    assert_eq!(world.entity(root).get::<SortOrder>().unwrap().value(), 0);
}

#[test]
fn when_two_roots_then_sorted_by_local_sort_order() {
    // Arrange
    let mut world = World::new();
    let a = world.spawn((SortOrder::new(0), LocalSortOrder(1))).id();
    let b = world.spawn((SortOrder::new(0), LocalSortOrder(0))).id();

    // Act
    run_with_hierarchy(&mut world);

    // Assert — b has lower LocalSortOrder, so gets 0; a gets 1
    assert_eq!(world.entity(b).get::<SortOrder>().unwrap().value(), 0);
    assert_eq!(world.entity(a).get::<SortOrder>().unwrap().value(), 1);
}

#[test]
fn when_parent_with_children_then_dfs_order_parent_child_a_child_b() {
    // Arrange
    let mut world = World::new();
    let parent = world.spawn(SortOrder::new(0)).id();
    let child_a = world
        .spawn((ChildOf(parent), LocalSortOrder(0), SortOrder::new(0)))
        .id();
    let child_b = world
        .spawn((ChildOf(parent), LocalSortOrder(1), SortOrder::new(0)))
        .id();

    // Act
    run_with_hierarchy(&mut world);

    // Assert
    let p = world.entity(parent).get::<SortOrder>().unwrap().value();
    let a = world.entity(child_a).get::<SortOrder>().unwrap().value();
    let b = world.entity(child_b).get::<SortOrder>().unwrap().value();
    assert_eq!(p, 0);
    assert_eq!(a, 1);
    assert_eq!(b, 2);
}

#[test]
fn when_two_parents_with_children_then_no_interleaving() {
    // Arrange
    let mut world = World::new();
    let card_a = world.spawn((SortOrder::new(0), LocalSortOrder(0))).id();
    let a_border = world
        .spawn((ChildOf(card_a), LocalSortOrder(1), SortOrder::new(0)))
        .id();
    let a_art = world
        .spawn((ChildOf(card_a), LocalSortOrder(2), SortOrder::new(0)))
        .id();

    let card_b = world.spawn((SortOrder::new(0), LocalSortOrder(1))).id();
    let b_border = world
        .spawn((ChildOf(card_b), LocalSortOrder(1), SortOrder::new(0)))
        .id();
    let b_art = world
        .spawn((ChildOf(card_b), LocalSortOrder(2), SortOrder::new(0)))
        .id();

    // Act
    run_with_hierarchy(&mut world);

    // Assert — DFS: card_a(0), a_border(1), a_art(2), card_b(3), b_border(4), b_art(5)
    let sorts: Vec<i32> = [card_a, a_border, a_art, card_b, b_border, b_art]
        .iter()
        .map(|&e| world.entity(e).get::<SortOrder>().unwrap().value())
        .collect();
    assert_eq!(sorts, vec![0, 1, 2, 3, 4, 5]);
}

#[test]
fn when_grandchildren_then_dfs_visits_recursively() {
    // Arrange
    let mut world = World::new();
    let root = world.spawn(SortOrder::new(0)).id();
    let child = world
        .spawn((ChildOf(root), LocalSortOrder(0), SortOrder::new(0)))
        .id();
    let grandchild = world
        .spawn((ChildOf(child), LocalSortOrder(0), SortOrder::new(0)))
        .id();

    // Act
    run_with_hierarchy(&mut world);

    // Assert
    let r = world.entity(root).get::<SortOrder>().unwrap().value();
    let c = world.entity(child).get::<SortOrder>().unwrap().value();
    let g = world.entity(grandchild).get::<SortOrder>().unwrap().value();
    assert_eq!((r, c, g), (0, 1, 2));
}

#[test]
fn when_children_reordered_by_local_sort_then_sort_order_reflects_new_order() {
    // Arrange
    let mut world = World::new();
    let parent = world.spawn(SortOrder::new(0)).id();
    let child_a = world
        .spawn((ChildOf(parent), LocalSortOrder(1), SortOrder::new(0)))
        .id();
    let child_b = world
        .spawn((ChildOf(parent), LocalSortOrder(0), SortOrder::new(0)))
        .id();
    run_with_hierarchy(&mut world);

    // Act — swap order: a goes first
    world
        .entity_mut(child_a)
        .get_mut::<LocalSortOrder>()
        .unwrap()
        .0 = 0;
    world
        .entity_mut(child_b)
        .get_mut::<LocalSortOrder>()
        .unwrap()
        .0 = 1;
    run_with_hierarchy(&mut world);

    // Assert
    let a = world.entity(child_a).get::<SortOrder>().unwrap().value();
    let b = world.entity(child_b).get::<SortOrder>().unwrap().value();
    assert!(a < b, "a ({a}) should sort before b ({b})");
}

#[test]
fn when_entity_has_no_local_sort_then_treated_as_zero() {
    // Arrange
    let mut world = World::new();
    let parent = world.spawn(SortOrder::new(0)).id();
    let child_no_local = world.spawn((ChildOf(parent), SortOrder::new(0))).id();
    let child_with_local = world
        .spawn((ChildOf(parent), LocalSortOrder(1), SortOrder::new(0)))
        .id();

    // Act
    run_with_hierarchy(&mut world);

    // Assert — no LocalSortOrder = 0, so renders before LocalSortOrder(1)
    let no_local = world
        .entity(child_no_local)
        .get::<SortOrder>()
        .unwrap()
        .value();
    let with_local = world
        .entity(child_with_local)
        .get::<SortOrder>()
        .unwrap()
        .value();
    assert!(no_local < with_local);
}

#[test]
fn when_root_has_no_sort_order_then_not_visited() {
    // Arrange — entity without SortOrder should be ignored
    let mut world = World::new();
    let _no_sort = world.spawn_empty().id();
    let with_sort = world.spawn(SortOrder::new(99)).id();

    // Act
    run_with_hierarchy(&mut world);

    // Assert
    assert_eq!(
        world.entity(with_sort).get::<SortOrder>().unwrap().value(),
        0
    );
}
