#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::*;
use engine_core::prelude::Color;
use engine_render::testing::{ShapeCallLog, insert_spy_with_shape_capture};
use engine_scene::prelude::EffectiveVisibility;
use engine_scene::transform_propagation::GlobalTransform2D;
use engine_ui::prelude::Text;
use engine_ui::text_render::text_render_system;
use glam::Affine2;

fn spawn_text_entity(world: &mut World, text: Text, translation: glam::Vec2) -> Entity {
    world
        .spawn((
            text,
            GlobalTransform2D(Affine2::from_translation(translation)),
        ))
        .id()
}

fn run_system(world: &mut World) -> ShapeCallLog {
    let shape_calls = insert_spy_with_shape_capture(world);
    let mut schedule = Schedule::default();
    schedule.add_systems(text_render_system);
    schedule.run(world);
    shape_calls
}

#[test]
fn when_one_text_entity_then_draw_shape_called() {
    // Arrange
    let mut world = World::new();
    spawn_text_entity(
        &mut world,
        Text {
            content: "A".to_owned(),
            font_size: 12.0,
            color: Color::WHITE,
            max_width: None,
        },
        glam::Vec2::ZERO,
    );

    // Act
    let shape_calls = run_system(&mut world);

    // Assert
    let calls = shape_calls.lock().unwrap();
    assert!(
        !calls.is_empty(),
        "should produce draw_shape calls for text"
    );
}

#[test]
fn when_effective_visibility_false_then_no_draw_shape_calls() {
    // Arrange
    let mut world = World::new();
    world.spawn((
        Text {
            content: "Hidden".to_owned(),
            font_size: 12.0,
            color: Color::WHITE,
            max_width: None,
        },
        GlobalTransform2D(Affine2::IDENTITY),
        EffectiveVisibility(false),
    ));

    // Act
    let shape_calls = run_system(&mut world);

    // Assert
    let calls = shape_calls.lock().unwrap();
    assert!(calls.is_empty());
}

#[test]
fn when_no_text_entities_then_no_draw_shape_calls() {
    // Arrange
    let mut world = World::new();

    // Act
    let shape_calls = run_system(&mut world);

    // Assert
    let calls = shape_calls.lock().unwrap();
    assert!(calls.is_empty());
}

#[test]
fn when_text_entity_rotated_then_model_matrix_has_rotation() {
    // Arrange
    let mut world = World::new();
    let rotation = std::f32::consts::FRAC_PI_4;
    world.spawn((
        Text {
            content: "A".to_owned(),
            font_size: 12.0,
            color: Color::WHITE,
            max_width: None,
        },
        GlobalTransform2D(Affine2::from_angle(rotation)),
    ));

    // Act
    let shape_calls = run_system(&mut world);

    // Assert
    let calls = shape_calls.lock().unwrap();
    assert!(!calls.is_empty());
    let model = calls[0].3;
    assert!(
        (model[0][0] - rotation.cos()).abs() < 1e-4,
        "model[0][0]={} expected cos(pi/4)={}",
        model[0][0],
        rotation.cos()
    );
}

#[test]
fn when_text_has_max_width_then_wraps_into_multiple_lines() {
    // Arrange
    let mut world = World::new();
    let font_size = 12.0;
    let word_width = engine_render::font::measure_text("Deal", font_size);
    spawn_text_entity(
        &mut world,
        Text {
            content: "Deal 3 damage to a target".to_owned(),
            font_size,
            color: Color::WHITE,
            max_width: Some(word_width * 3.0),
        },
        glam::Vec2::ZERO,
    );

    // Act
    let shape_calls = run_system(&mut world);

    // Assert
    let calls = shape_calls.lock().unwrap();
    assert!(!calls.is_empty());
    let y_positions: Vec<f32> = calls.iter().map(|c| c.3[3][1]).collect();
    let min_y = y_positions.iter().copied().fold(f32::INFINITY, f32::min);
    let max_y = y_positions
        .iter()
        .copied()
        .fold(f32::NEG_INFINITY, f32::max);
    assert!(
        (max_y - min_y).abs() > 1.0,
        "wrapped text should have glyphs at different y positions, got min={min_y} max={max_y}"
    );
}
