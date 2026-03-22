use bevy_ecs::prelude::{Entity, Local, Query, ResMut};
use engine_render::camera::Camera2D;
use engine_render::culling::{aabb_intersects_view_rect, compute_view_rect};
use engine_render::font::{GlyphCache, measure_text, render_text_transformed, wrap_text};
use engine_render::material::{Material2d, apply_material};
use engine_render::prelude::RendererRes;
use engine_render::shape::{
    ColorMesh, Shape, ShapeVariant, Stroke, affine2_to_mat4, shape_aabb, tessellate,
    tessellate_stroke,
};
use engine_scene::prelude::{EffectiveVisibility, GlobalTransform2D, RenderLayer, SortOrder};
use glam::Vec2;

use crate::widget::Text;

const LINE_HEIGHT_FACTOR: f32 = 1.3;

type ShapeItem<'w> = (
    Entity,
    &'w Shape,
    &'w GlobalTransform2D,
    Option<&'w RenderLayer>,
    Option<&'w SortOrder>,
    Option<&'w EffectiveVisibility>,
    Option<&'w Material2d>,
    Option<&'w Stroke>,
);

type TextItem<'w> = (
    Entity,
    &'w Text,
    &'w GlobalTransform2D,
    Option<&'w RenderLayer>,
    Option<&'w SortOrder>,
    Option<&'w EffectiveVisibility>,
);

type ColorMeshItem<'w> = (
    Entity,
    &'w ColorMesh,
    &'w GlobalTransform2D,
    Option<&'w RenderLayer>,
    Option<&'w SortOrder>,
    Option<&'w EffectiveVisibility>,
);

#[derive(Clone, Copy)]
enum DrawKind {
    Shape,
    Text,
    ColorMesh,
}

struct SortedDrawItem {
    entity: Entity,
    sort_key: (RenderLayer, SortOrder),
    kind: DrawKind,
}

fn is_shape_culled(pos: Vec2, variant: &ShapeVariant, view_rect: Option<(Vec2, Vec2)>) -> bool {
    let Some((view_min, view_max)) = view_rect else {
        return false;
    };
    let (local_min, local_max) = shape_aabb(variant);
    let r = local_min.abs().max(local_max.abs()).length();
    let entity_min = Vec2::new(pos.x - r, pos.y - r);
    let entity_max = Vec2::new(pos.x + r, pos.y + r);
    !aabb_intersects_view_rect(entity_min, entity_max, view_min, view_max)
}

fn collect_draw_items(
    shape_query: &Query<ShapeItem>,
    text_query: &Query<TextItem>,
    color_mesh_query: &Query<ColorMeshItem>,
) -> Vec<SortedDrawItem> {
    let mut items: Vec<SortedDrawItem> = Vec::new();
    for (entity, _, _, layer, sort, vis, _, _) in shape_query.iter() {
        if vis.is_some_and(|v| !v.0) {
            continue;
        }
        items.push(SortedDrawItem {
            entity,
            sort_key: (
                layer.copied().unwrap_or(RenderLayer::World),
                sort.copied().unwrap_or_default(),
            ),
            kind: DrawKind::Shape,
        });
    }
    for (entity, _, _, layer, sort, vis) in text_query.iter() {
        if vis.is_some_and(|v| !v.0) {
            continue;
        }
        items.push(SortedDrawItem {
            entity,
            sort_key: (
                layer.copied().unwrap_or(RenderLayer::World),
                sort.copied().unwrap_or_default(),
            ),
            kind: DrawKind::Text,
        });
    }
    for (entity, _, _, layer, sort, vis) in color_mesh_query.iter() {
        if vis.is_some_and(|v| !v.0) {
            continue;
        }
        items.push(SortedDrawItem {
            entity,
            sort_key: (
                layer.copied().unwrap_or(RenderLayer::World),
                sort.copied().unwrap_or_default(),
            ),
            kind: DrawKind::ColorMesh,
        });
    }
    items.sort_by_key(|item| item.sort_key);
    items
}

/// Unified render system that draws both shapes and text in a single sorted
/// pass, preventing text from rendering on top of shapes that should occlude it.
pub fn unified_render_system(
    shape_query: Query<ShapeItem>,
    text_query: Query<TextItem>,
    color_mesh_query: Query<ColorMeshItem>,
    camera_query: Query<&Camera2D>,
    mut renderer: ResMut<RendererRes>,
    mut cache: Local<GlyphCache>,
) {
    let view_rect = compute_view_rect(&camera_query, &renderer);
    let items = collect_draw_items(&shape_query, &text_query, &color_mesh_query);

    let mut last_shader = None;
    let mut last_blend_mode = None;

    for item in &items {
        match item.kind {
            DrawKind::Shape => {
                let Ok((_, shape, transform, _, _, _, mat, stroke)) = shape_query.get(item.entity)
                else {
                    continue;
                };
                if is_shape_culled(transform.0.translation, &shape.variant, view_rect) {
                    continue;
                }
                apply_material(&mut **renderer, mat, &mut last_shader, &mut last_blend_mode);
                let model = affine2_to_mat4(&transform.0);
                let Ok(mesh) = tessellate(&shape.variant) else {
                    continue;
                };
                renderer.draw_shape(&mesh.vertices, &mesh.indices, shape.color, model);
                if let Some(stroke) = stroke
                    && let Ok(sm) = tessellate_stroke(&shape.variant, stroke.width)
                {
                    renderer.draw_shape(&sm.vertices, &sm.indices, stroke.color, model);
                }
            }
            DrawKind::Text => {
                let Ok((_, text, global_transform, _, _, _)) = text_query.get(item.entity) else {
                    continue;
                };
                draw_text(&mut **renderer, &mut cache, text, global_transform);
            }
            DrawKind::ColorMesh => {
                let Ok((_, mesh, transform, _, _, _)) = color_mesh_query.get(item.entity) else {
                    continue;
                };
                if mesh.is_empty() {
                    continue;
                }
                let model = affine2_to_mat4(&transform.0);
                renderer.draw_colored_mesh(&mesh.vertices, &mesh.indices, model);
            }
        }
    }
}

fn draw_text(
    renderer: &mut dyn engine_render::renderer::Renderer,
    cache: &mut GlyphCache,
    text: &Text,
    global_transform: &GlobalTransform2D,
) {
    if let Some(max_width) = text.max_width {
        let lines = wrap_text(&text.content, text.font_size, max_width);
        let line_height = text.font_size * LINE_HEIGHT_FACTOR;
        let total_height = (lines.len() as f32 - 1.0) * line_height;
        let start_y = -total_height * 0.5;
        for (i, line) in lines.iter().enumerate() {
            let line_width = measure_text(line, text.font_size);
            let y_offset = start_y + i as f32 * line_height;
            let offset = glam::Affine2::from_translation(Vec2::new(-line_width * 0.5, y_offset));
            let line_transform = global_transform.0 * offset;
            let model = affine2_to_mat4(&line_transform);
            render_text_transformed(renderer, cache, line, &model, text.font_size, text.color);
        }
    } else {
        let text_width = measure_text(&text.content, text.font_size);
        let center_offset = glam::Affine2::from_translation(Vec2::new(-text_width * 0.5, 0.0));
        let centered_transform = global_transform.0 * center_offset;
        let model = affine2_to_mat4(&centered_transform);
        render_text_transformed(
            renderer,
            cache,
            &text.content,
            &model,
            text.font_size,
            text.color,
        );
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use bevy_ecs::prelude::*;
    use engine_core::prelude::Color;
    use engine_render::prelude::{Shape, ShapeVariant};
    use engine_render::shape::{ColorMesh, TessellatedColorMesh};
    use engine_render::testing::{
        ColoredMeshCallLog, ShapeCallLog, insert_spy_with_colored_mesh_capture,
        insert_spy_with_shape_capture,
    };
    use engine_scene::prelude::{EffectiveVisibility, RenderLayer, SortOrder};
    use engine_scene::transform_propagation::GlobalTransform2D;
    use glam::Affine2;

    use super::*;

    fn run_system(world: &mut World) -> ShapeCallLog {
        let shape_calls = insert_spy_with_shape_capture(world);
        let mut schedule = Schedule::default();
        schedule.add_systems(unified_render_system);
        schedule.run(world);
        shape_calls
    }

    #[test]
    fn when_shape_and_text_then_both_drawn() {
        // Arrange
        let mut world = World::new();
        world.spawn((
            Shape {
                variant: ShapeVariant::Circle { radius: 10.0 },
                color: Color::RED,
            },
            GlobalTransform2D(Affine2::IDENTITY),
            SortOrder(0),
        ));
        world.spawn((
            Text {
                content: "A".to_owned(),
                font_size: 12.0,
                color: Color::WHITE,
                max_width: None,
            },
            GlobalTransform2D(Affine2::IDENTITY),
            SortOrder(1),
        ));

        // Act
        let shape_calls = run_system(&mut world);

        // Assert
        let calls = shape_calls.lock().unwrap();
        assert!(
            calls.len() >= 2,
            "should have draw calls for both shape and text, got {}",
            calls.len()
        );
    }

    #[test]
    fn when_text_has_lower_sort_order_then_drawn_before_shape() {
        // Arrange
        let mut world = World::new();
        let shape_y = 100.0;
        let text_y = -100.0;
        world.spawn((
            Shape {
                variant: ShapeVariant::Circle { radius: 10.0 },
                color: Color::RED,
            },
            GlobalTransform2D(Affine2::from_translation(Vec2::new(0.0, shape_y))),
            SortOrder(5),
            RenderLayer::World,
        ));
        world.spawn((
            Text {
                content: "A".to_owned(),
                font_size: 12.0,
                color: Color::WHITE,
                max_width: None,
            },
            GlobalTransform2D(Affine2::from_translation(Vec2::new(0.0, text_y))),
            SortOrder(1),
            RenderLayer::World,
        ));

        // Act
        let shape_calls = run_system(&mut world);

        // Assert — text (SortOrder 1) should draw before shape (SortOrder 5).
        // Text glyphs have y near text_y, shape vertices have y near shape_y.
        let calls = shape_calls.lock().unwrap();
        assert!(calls.len() >= 2, "expected at least 2 draw calls");

        // Find first call from text (model translation y near text_y) and
        // first call from shape (model translation y near shape_y).
        let first_text_idx = calls.iter().position(|c| (c.3[3][1] - text_y).abs() < 50.0);
        let first_shape_idx = calls
            .iter()
            .position(|c| (c.3[3][1] - shape_y).abs() < 50.0);
        assert!(
            first_text_idx.is_some() && first_shape_idx.is_some(),
            "should find both text and shape draw calls"
        );
        assert!(
            first_text_idx.unwrap() < first_shape_idx.unwrap(),
            "text (SortOrder 1) should draw before shape (SortOrder 5)"
        );
    }

    fn run_system_colored(world: &mut World) -> ColoredMeshCallLog {
        let calls = insert_spy_with_colored_mesh_capture(world);
        let mut schedule = Schedule::default();
        schedule.add_systems(unified_render_system);
        schedule.run(world);
        calls
    }

    #[test]
    fn when_color_mesh_entity_then_draw_colored_mesh_called() {
        // Arrange
        let mut world = World::new();
        let mut mesh = TessellatedColorMesh::new();
        mesh.push_vertices(
            &[[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]],
            &[0, 1, 2],
            [1.0, 0.0, 0.0, 1.0],
        );
        world.spawn((
            ColorMesh(mesh),
            GlobalTransform2D(Affine2::IDENTITY),
            SortOrder(0),
        ));

        // Act
        let calls = run_system_colored(&mut world);

        // Assert
        let calls = calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0[0].color, [1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn when_invisible_then_not_drawn() {
        // Arrange
        let mut world = World::new();
        world.spawn((
            Shape {
                variant: ShapeVariant::Circle { radius: 10.0 },
                color: Color::RED,
            },
            GlobalTransform2D(Affine2::IDENTITY),
            SortOrder(0),
            EffectiveVisibility(false),
        ));
        world.spawn((
            Text {
                content: "A".to_owned(),
                font_size: 12.0,
                color: Color::WHITE,
                max_width: None,
            },
            GlobalTransform2D(Affine2::IDENTITY),
            SortOrder(1),
            EffectiveVisibility(false),
        ));

        // Act
        let shape_calls = run_system(&mut world);

        // Assert
        let calls = shape_calls.lock().unwrap();
        assert!(calls.is_empty());
    }
}
