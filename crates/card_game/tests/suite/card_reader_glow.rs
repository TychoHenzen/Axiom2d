#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::*;
use bevy_ecs::schedule::Schedule;
use engine_core::prelude::Transform2D;
use engine_render::prelude::{Shape, Stroke};
use engine_scene::prelude::{Children, hierarchy_maintenance_system};
use glam::Vec2;

use card_game::card::reader::CardReader;
use card_game::card::reader::glow::{ReaderAccent, ReaderRecess, ReaderRune, reader_glow_system};
use card_game::card::reader::spawn::{
    ACCENT_COLOR_LIT, RECESS_STROKE_LIT, RUNE_COLOR_DIM, RUNE_COLOR_LIT, spawn_reader,
};

fn run_glow(world: &mut World) {
    // Hierarchy must run first so Children is populated.
    let mut hierarchy = Schedule::default();
    hierarchy.add_systems(hierarchy_maintenance_system);
    hierarchy.run(world);

    let mut schedule = Schedule::default();
    schedule.add_systems(reader_glow_system);
    schedule.run(world);
}

#[test]
fn when_reader_empty_then_runes_are_dim() {
    // Arrange
    let mut world = World::new();
    let (reader, _) = spawn_reader(&mut world, Vec2::ZERO);

    // Act
    run_glow(&mut world);

    // Assert
    let children = &world.get::<Children>(reader).unwrap().0;
    for &child in children {
        if world.get::<ReaderRune>(child).is_some() {
            let shape = world.get::<Shape>(child).unwrap();
            assert!(
                (shape.color.a - RUNE_COLOR_DIM.a).abs() < f32::EPSILON,
                "expected dim alpha {}, got {}",
                RUNE_COLOR_DIM.a,
                shape.color.a
            );
        }
    }
}

#[test]
fn when_reader_loaded_then_runes_are_lit() {
    // Arrange
    let mut world = World::new();
    let (reader, _) = spawn_reader(&mut world, Vec2::ZERO);
    let card_entity = world.spawn(Transform2D::default()).id();
    world.get_mut::<CardReader>(reader).unwrap().loaded = Some(card_entity);

    // Act
    run_glow(&mut world);

    // Assert
    let children = &world.get::<Children>(reader).unwrap().0;
    for &child in children {
        if world.get::<ReaderRune>(child).is_some() {
            let shape = world.get::<Shape>(child).unwrap();
            assert!(
                (shape.color.a - RUNE_COLOR_LIT.a).abs() < f32::EPSILON,
                "expected lit alpha {}, got {}",
                RUNE_COLOR_LIT.a,
                shape.color.a
            );
        }
    }
}

#[test]
fn when_reader_loaded_then_accent_is_lit() {
    // Arrange
    let mut world = World::new();
    let (reader, _) = spawn_reader(&mut world, Vec2::ZERO);
    let card_entity = world.spawn(Transform2D::default()).id();
    world.get_mut::<CardReader>(reader).unwrap().loaded = Some(card_entity);

    // Act
    run_glow(&mut world);

    // Assert
    let children = &world.get::<Children>(reader).unwrap().0;
    for &child in children {
        if world.get::<ReaderAccent>(child).is_some() {
            let shape = world.get::<Shape>(child).unwrap();
            assert!(
                (shape.color.a - ACCENT_COLOR_LIT.a).abs() < f32::EPSILON,
                "expected lit alpha {}, got {}",
                ACCENT_COLOR_LIT.a,
                shape.color.a
            );
        }
    }
}

#[test]
fn when_reader_loaded_then_recess_stroke_is_lit() {
    // Arrange
    let mut world = World::new();
    let (reader, _) = spawn_reader(&mut world, Vec2::ZERO);
    let card_entity = world.spawn(Transform2D::default()).id();
    world.get_mut::<CardReader>(reader).unwrap().loaded = Some(card_entity);

    // Act
    run_glow(&mut world);

    // Assert
    let children = &world.get::<Children>(reader).unwrap().0;
    for &child in children {
        if world.get::<ReaderRecess>(child).is_some() {
            let stroke = world.get::<Stroke>(child).unwrap();
            assert!(
                (stroke.color.a - RECESS_STROKE_LIT.a).abs() < f32::EPSILON,
                "expected lit alpha {}, got {}",
                RECESS_STROKE_LIT.a,
                stroke.color.a
            );
        }
    }
}

#[test]
fn when_card_ejected_then_runes_return_to_dim() {
    // Arrange
    let mut world = World::new();
    let (reader, _) = spawn_reader(&mut world, Vec2::ZERO);
    let card_entity = world.spawn(Transform2D::default()).id();

    // Load card
    world.get_mut::<CardReader>(reader).unwrap().loaded = Some(card_entity);
    run_glow(&mut world);

    // Eject card
    world.get_mut::<CardReader>(reader).unwrap().loaded = None;

    // Act
    run_glow(&mut world);

    // Assert
    let children = &world.get::<Children>(reader).unwrap().0;
    for &child in children {
        if world.get::<ReaderRune>(child).is_some() {
            let shape = world.get::<Shape>(child).unwrap();
            assert!(
                (shape.color.a - RUNE_COLOR_DIM.a).abs() < f32::EPSILON,
                "expected dim alpha after eject, got {}",
                shape.color.a
            );
        }
    }
}
