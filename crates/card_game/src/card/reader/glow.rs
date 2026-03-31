use bevy_ecs::prelude::*;
use engine_render::prelude::{Shape, Stroke};
use engine_scene::prelude::Children;

use super::components::CardReader;
use super::spawn::{
    ACCENT_COLOR_DIM, ACCENT_COLOR_LIT, RECESS_STROKE_DIM, RECESS_STROKE_LIT, RUNE_COLOR_DIM,
    RUNE_COLOR_LIT,
};

/// Marker for the inner recess child entity.
#[derive(Component, Debug)]
pub struct ReaderRecess;

/// Marker for the accent line child entity.
#[derive(Component, Debug)]
pub struct ReaderAccent;

/// Marker for a corner rune child entity.
#[derive(Component, Debug)]
pub struct ReaderRune;

/// Toggles reader child entity colors between dim and lit based on whether
/// a card is loaded.
#[allow(clippy::type_complexity)]
pub fn reader_glow_system(
    readers: Query<(&CardReader, &Children)>,
    mut runes: Query<&mut Shape, With<ReaderRune>>,
    mut accents: Query<&mut Shape, (With<ReaderAccent>, Without<ReaderRune>)>,
    mut recesses: Query<
        &mut Stroke,
        (
            With<ReaderRecess>,
            Without<ReaderRune>,
            Without<ReaderAccent>,
        ),
    >,
) {
    for (reader, children) in &readers {
        let is_loaded = reader.loaded.is_some();

        for &child in &children.0 {
            if let Ok(mut shape) = runes.get_mut(child) {
                shape.color = if is_loaded {
                    RUNE_COLOR_LIT
                } else {
                    RUNE_COLOR_DIM
                };
            }
            if let Ok(mut shape) = accents.get_mut(child) {
                shape.color = if is_loaded {
                    ACCENT_COLOR_LIT
                } else {
                    ACCENT_COLOR_DIM
                };
            }
            if let Ok(mut stroke) = recesses.get_mut(child) {
                stroke.color = if is_loaded {
                    RECESS_STROKE_LIT
                } else {
                    RECESS_STROKE_DIM
                };
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use bevy_ecs::schedule::Schedule;
    use engine_core::prelude::Transform2D;
    use engine_scene::prelude::hierarchy_maintenance_system;
    use glam::Vec2;

    use super::*;
    use crate::card::reader::spawn::spawn_reader;

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
}
