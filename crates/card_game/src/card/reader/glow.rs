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
