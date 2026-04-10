// EVOLVE-BLOCK-START
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
// EVOLVE-BLOCK-END
