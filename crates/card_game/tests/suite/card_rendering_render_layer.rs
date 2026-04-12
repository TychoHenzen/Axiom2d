#![allow(clippy::unwrap_used)]

use crate::test_helpers::make_test_card;
use bevy_ecs::prelude::*;
use card_game::card::component::CardFaceSide;
use card_game::card::rendering::render_layer::card_render_layer_system;
use engine_scene::prelude::{ChildOf, RenderLayer, SortOrder, hierarchy_maintenance_system};

fn run_system(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems((hierarchy_maintenance_system, card_render_layer_system).chain());
    schedule.run(world);
}

fn spawn_card_with_face(world: &mut World, parent_layer: RenderLayer) -> (Entity, Entity) {
    let card = make_test_card();
    let root = world.spawn((card, parent_layer, SortOrder::new(0))).id();
    let face = world
        .spawn((
            ChildOf(root),
            CardFaceSide::Front,
            RenderLayer::World,
            SortOrder::new(0),
        ))
        .id();
    (root, face)
}

/// @doc: Card `RenderLayer` must propagate to children (`CardFaceSide` entities) to keep face meshes on the correct layer.
/// If a card moves from Table (World) to Hand (UI), its child faces must follow, or they render behind UI instead of on top.
#[test]
fn when_parent_ui_and_child_world_then_child_becomes_ui() {
    // Arrange
    let mut world = World::new();
    let (_, face) = spawn_card_with_face(&mut world, RenderLayer::UI);

    // Act
    run_system(&mut world);

    // Assert
    assert_eq!(*world.get::<RenderLayer>(face).unwrap(), RenderLayer::UI);
}

#[test]
fn when_parent_world_and_child_world_then_child_stays_world() {
    // Arrange
    let mut world = World::new();
    let (_, face) = spawn_card_with_face(&mut world, RenderLayer::World);

    // Act
    run_system(&mut world);

    // Assert
    assert_eq!(*world.get::<RenderLayer>(face).unwrap(), RenderLayer::World);
}

/// @doc: Dynamic layer changes (zone transitions: Table -> Hand -> Stash) must be reflected in children.
#[test]
fn when_parent_changes_layer_then_child_follows() {
    // Arrange
    let mut world = World::new();
    let (root, face) = spawn_card_with_face(&mut world, RenderLayer::World);
    run_system(&mut world);
    assert_eq!(*world.get::<RenderLayer>(face).unwrap(), RenderLayer::World);

    // Act
    *world.get_mut::<RenderLayer>(root).unwrap() = RenderLayer::UI;
    run_system(&mut world);

    // Assert
    assert_eq!(*world.get::<RenderLayer>(face).unwrap(), RenderLayer::UI);
}

/// @doc: Only `CardFaceSide` children should have their layer synced. Non-face children (e.g., `StashIcon` entities)
/// must retain independent layer control.
#[test]
fn when_child_has_no_card_face_side_then_not_affected() {
    // Arrange — non-face child (e.g., StashIcon) keeps its own layer
    let mut world = World::new();
    let card = make_test_card();
    let root = world
        .spawn((card, RenderLayer::World, SortOrder::new(0)))
        .id();
    let non_face = world
        .spawn((ChildOf(root), RenderLayer::UI, SortOrder::new(0)))
        .id();

    // Act
    run_system(&mut world);

    // Assert — no CardFaceSide, so layer unchanged
    assert_eq!(
        *world.get::<RenderLayer>(non_face).unwrap(),
        RenderLayer::UI
    );
}
