use bevy_ecs::prelude::{Local, Query, ResMut};
use engine_render::font::{GlyphCache, measure_text, render_text_transformed, wrap_text};
use engine_render::prelude::RendererRes;
use engine_render::shape::affine2_to_mat4;
use engine_scene::prelude::EffectiveVisibility;
use engine_scene::transform_propagation::GlobalTransform2D;

use crate::widget::Text;

const LINE_HEIGHT_FACTOR: f32 = 1.3;

pub fn text_render_system(
    query: Query<(&Text, &GlobalTransform2D, Option<&EffectiveVisibility>)>,
    mut renderer: ResMut<RendererRes>,
    mut cache: Local<GlyphCache>,
) {
    for (text, global_transform, effective_vis) in &query {
        if let Some(EffectiveVisibility(false)) = effective_vis {
            continue;
        }
        if let Some(max_width) = text.max_width {
            let lines = wrap_text(&text.content, text.font_size, max_width);
            let line_height = text.font_size * LINE_HEIGHT_FACTOR;
            let total_height = (lines.len() as f32 - 1.0) * line_height;
            let start_y = -total_height * 0.5;
            for (i, line) in lines.iter().enumerate() {
                let line_width = measure_text(line, text.font_size);
                let y_offset = start_y + i as f32 * line_height;
                let offset =
                    glam::Affine2::from_translation(glam::Vec2::new(-line_width * 0.5, y_offset));
                let line_transform = global_transform.0 * offset;
                let model = affine2_to_mat4(&line_transform);
                render_text_transformed(
                    &mut **renderer,
                    &mut cache,
                    line,
                    &model,
                    text.font_size,
                    text.color,
                );
            }
        } else {
            let text_width = measure_text(&text.content, text.font_size);
            let center_offset =
                glam::Affine2::from_translation(glam::Vec2::new(-text_width * 0.5, 0.0));
            let centered_transform = global_transform.0 * center_offset;
            let model = affine2_to_mat4(&centered_transform);
            render_text_transformed(
                &mut **renderer,
                &mut cache,
                &text.content,
                &model,
                text.font_size,
                text.color,
            );
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use bevy_ecs::prelude::*;
    use engine_core::prelude::Color;
    use engine_render::testing::{ShapeCallLog, insert_spy_with_shape_capture};
    use engine_scene::prelude::EffectiveVisibility;
    use engine_scene::transform_propagation::GlobalTransform2D;
    use glam::Affine2;

    use crate::prelude::Text;

    use super::text_render_system;

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
}
