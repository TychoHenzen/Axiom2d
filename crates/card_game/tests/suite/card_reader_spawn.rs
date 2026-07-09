#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::*;
use engine_render::prelude::ShapeVariant;
use engine_render::shape::Shape;
use engine_scene::prelude::{ChildOf, Children, hierarchy_maintenance_system};
use glam::Vec2;

use card_game::card::reader::glow::{ReaderAccent, ReaderRecess, ReaderRune};
use card_game::card::reader::spawn::spawn_reader;

fn run_hierarchy(world: &mut World) {
    let mut schedule = bevy_ecs::schedule::Schedule::default();
    schedule.add_systems(hierarchy_maintenance_system);
    schedule.run(world);
}

/// @doc: Verifies that spawning a reader produces a root entity with a Path shape (rounded rect).
#[test]
fn when_spawn_reader_then_root_has_rounded_rect_shape() {
    // Arrange
    let mut world = World::new();

    // Act
    let (reader, _) = spawn_reader(&mut world, Vec2::ZERO);

    // Assert
    let shape = world.get::<Shape>(reader).unwrap();
    assert!(
        matches!(shape.variant, ShapeVariant::Path { .. }),
        "expected rounded rect (Path variant)"
    );
}

/// @doc: Verifies that the reader entity has exactly six child entities after hierarchy maintenance.
#[test]
fn when_spawn_reader_then_six_children_exist() {
    // Arrange
    let mut world = World::new();
    let (reader, _) = spawn_reader(&mut world, Vec2::ZERO);

    // Act
    run_hierarchy(&mut world);

    // Assert — 1 recess + 1 accent + 4 runes = 6 children
    let children = world.get::<Children>(reader).unwrap();
    assert_eq!(
        children.0.len(),
        6,
        "expected 6 children (1 recess + 1 accent + 4 runes)"
    );
}

/// @doc: Verifies that the reader has exactly four rune children.
#[test]
fn when_spawn_reader_then_four_rune_children_exist() {
    // Arrange
    let mut world = World::new();
    let (reader, _) = spawn_reader(&mut world, Vec2::ZERO);
    run_hierarchy(&mut world);

    // Act
    let children = &world.get::<Children>(reader).unwrap().0;
    let rune_count = children
        .iter()
        .filter(|e| world.get::<ReaderRune>(**e).is_some())
        .count();

    // Assert
    assert_eq!(rune_count, 4, "expected exactly 4 rune children");
}

/// @doc: Verifies that the reader has exactly one recess child.
#[test]
fn when_spawn_reader_then_recess_child_exists() {
    // Arrange
    let mut world = World::new();
    let (reader, _) = spawn_reader(&mut world, Vec2::ZERO);
    run_hierarchy(&mut world);

    // Act
    let children = &world.get::<Children>(reader).unwrap().0;
    let recess_count = children
        .iter()
        .filter(|e| world.get::<ReaderRecess>(**e).is_some())
        .count();

    // Assert
    assert_eq!(recess_count, 1, "expected exactly 1 recess child");
}

/// @doc: Verifies that the reader has exactly one accent child.
#[test]
fn when_spawn_reader_then_accent_child_exists() {
    // Arrange
    let mut world = World::new();
    let (reader, _) = spawn_reader(&mut world, Vec2::ZERO);
    run_hierarchy(&mut world);

    // Act
    let children = &world.get::<Children>(reader).unwrap().0;
    let accent_count = children
        .iter()
        .filter(|e| world.get::<ReaderAccent>(**e).is_some())
        .count();

    // Assert
    assert_eq!(accent_count, 1, "expected exactly 1 accent child");
}

/// @doc: Verifies that all reader children have a ChildOf component pointing back to the reader root.
#[test]
fn when_spawn_reader_then_children_have_child_of_pointing_to_reader() {
    // Arrange
    let mut world = World::new();
    let (reader, _) = spawn_reader(&mut world, Vec2::ZERO);
    run_hierarchy(&mut world);

    // Act / Assert
    let children = &world.get::<Children>(reader).unwrap().0;
    for &child in children {
        let child_of = world.get::<ChildOf>(child).unwrap();
        assert_eq!(
            child_of.0, reader,
            "all children should have ChildOf pointing to reader root"
        );
    }
}
