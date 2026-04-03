use bevy_ecs::prelude::{Entity, Local, Query, ResMut};
use engine_core::profiler::FrameProfiler;
use engine_render::camera::{Camera2D, CameraRotation};
use engine_render::culling::compute_view_rect;
use engine_render::font::{GlyphCache, measure_text, render_text_transformed, wrap_text};
use engine_render::material::{Material2d, apply_material};
use engine_render::prelude::RendererRes;
use engine_render::shape::{
    CachedMesh, ColorMesh, MeshOverlays, Shape, Stroke, affine2_to_mat4, is_shape_culled,
    tessellate, tessellate_stroke,
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
    Option<&'w CachedMesh>,
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
    Option<&'w MeshOverlays>,
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

fn collect_draw_items(
    shape_query: &Query<ShapeItem>,
    text_query: &Query<TextItem>,
    color_mesh_query: &Query<ColorMeshItem>,
) -> Vec<SortedDrawItem> {
    let mut items: Vec<SortedDrawItem> = Vec::new();
    for (entity, _, _, layer, sort, vis, _, _, _) in shape_query.iter() {
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
    for (entity, _, _, layer, sort, vis, _) in color_mesh_query.iter() {
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
    camera_query: Query<(&Camera2D, Option<&CameraRotation>)>,
    mut renderer: ResMut<RendererRes>,
    mut cache: Local<GlyphCache>,
    mut profiler: Option<ResMut<FrameProfiler>>,
) {
    let view_rect = compute_view_rect(&camera_query, &renderer);

    let t_sort = std::time::Instant::now();
    let items = collect_draw_items(&shape_query, &text_query, &color_mesh_query);
    let sort_us = t_sort.elapsed().as_micros() as u64;

    let t_draw = std::time::Instant::now();

    let mut last_shader = None;
    let mut last_blend_mode = None;

    for item in &items {
        match item.kind {
            DrawKind::Shape => {
                let Ok((_, shape, transform, _, _, _, mat, stroke, cached)) =
                    shape_query.get(item.entity)
                else {
                    continue;
                };
                if is_shape_culled(transform.0.translation, &shape.variant, view_rect) {
                    continue;
                }
                apply_material(&mut **renderer, mat, &mut last_shader, &mut last_blend_mode);
                let model = affine2_to_mat4(&transform.0);
                if let Some(cached) = cached {
                    renderer.draw_shape(&cached.0.vertices, &cached.0.indices, shape.color, model);
                } else {
                    let Ok(mesh) = tessellate(&shape.variant) else {
                        continue;
                    };
                    renderer.draw_shape(&mesh.vertices, &mesh.indices, shape.color, model);
                }
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
                let Ok((_, mesh, transform, _, _, _, overlays)) = color_mesh_query.get(item.entity)
                else {
                    continue;
                };
                if mesh.is_empty() {
                    continue;
                }
                apply_material(
                    &mut **renderer,
                    None,
                    &mut last_shader,
                    &mut last_blend_mode,
                );
                let model = affine2_to_mat4(&transform.0);
                renderer.draw_colored_mesh(&mesh.vertices, &mesh.indices, model);
                if let Some(overlays) = overlays {
                    for entry in overlays.0.iter().filter(|e| e.visible) {
                        apply_material(
                            &mut **renderer,
                            Some(&entry.material),
                            &mut last_shader,
                            &mut last_blend_mode,
                        );
                        renderer.draw_colored_mesh(
                            &entry.mesh.vertices,
                            &entry.mesh.indices,
                            model,
                        );
                    }
                }
            }
        }
    }

    let draw_us = t_draw.elapsed().as_micros() as u64;

    if let Some(p) = profiler.as_deref_mut() {
        p.record_phase("render_sort", sort_us);
        p.record_phase("render_draw", draw_us);
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
