// EVOLVE-BLOCK-START
use bevy_ecs::prelude::{Local, Query, ResMut};
use engine_core::color::Color;
use engine_core::profiler::FrameProfiler;
use engine_core::types::Pixels;
use engine_render::camera::{Camera2D, CameraRotation};
use engine_render::culling::{aabb_intersects_view_rect, compute_view_rect};
use engine_render::font::{GlyphCache, measure_text, render_text_transformed, wrap_text};
use engine_render::material::{BlendMode, Material2d, apply_material};
use engine_render::prelude::RendererRes;
use engine_render::rect::Rect;
use engine_render::shader::ShaderHandle;
use engine_render::shape::{
    CachedMesh, ColorMesh, MeshOverlays, PersistentColorMesh, Shape, Stroke, affine2_to_mat4,
    is_shape_culled, tessellate, tessellate_stroke,
};
use engine_render::sprite::Sprite;
use engine_scene::prelude::{EffectiveVisibility, GlobalTransform2D, RenderLayer, SortOrder};
use glam::Vec2;

use crate::draw_command::{
    DrawCommand, DrawQueue, OverlayCommand, SortedDrawCommand, StrokeCommand,
};
use crate::widget::Text;

const LINE_HEIGHT_FACTOR: f32 = 1.3;

type ShapeItem<'w> = (
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
    &'w Text,
    &'w GlobalTransform2D,
    Option<&'w RenderLayer>,
    Option<&'w SortOrder>,
    Option<&'w EffectiveVisibility>,
);

type ColorMeshItem<'w> = (
    &'w ColorMesh,
    &'w GlobalTransform2D,
    Option<&'w RenderLayer>,
    Option<&'w SortOrder>,
    Option<&'w EffectiveVisibility>,
    Option<&'w MeshOverlays>,
    Option<&'w Material2d>,
);

type PersistentMeshItem<'w> = (
    &'w PersistentColorMesh,
    &'w GlobalTransform2D,
    Option<&'w RenderLayer>,
    Option<&'w SortOrder>,
    Option<&'w EffectiveVisibility>,
    Option<&'w MeshOverlays>,
    Option<&'w Material2d>,
);

type SpriteItem<'w> = (
    &'w Sprite,
    &'w GlobalTransform2D,
    Option<&'w RenderLayer>,
    Option<&'w SortOrder>,
    Option<&'w EffectiveVisibility>,
    Option<&'w Material2d>,
);

fn key(layer: Option<&RenderLayer>, order: Option<&SortOrder>) -> (RenderLayer, SortOrder) {
    (
        layer.copied().unwrap_or(RenderLayer::World),
        order.copied().unwrap_or_default(),
    )
}

fn is_hidden(vis: Option<&EffectiveVisibility>) -> bool {
    vis.is_some_and(|v| !v.0)
}

fn push_sorted_command(
    commands: &mut Vec<SortedDrawCommand>,
    layer: Option<&RenderLayer>,
    order: Option<&SortOrder>,
    command: DrawCommand,
) {
    commands.push(SortedDrawCommand {
        sort_key: key(layer, order),
        command,
    });
}

fn sprite_intersects_view(
    sprite: &Sprite,
    transform: &GlobalTransform2D,
    view_rect: Option<(Vec2, Vec2)>,
) -> bool {
    view_rect.is_none_or(|(view_min, view_max)| {
        let pos = transform.0.translation;
        let entity_min = pos;
        let entity_max = Vec2::new(pos.x + sprite.width.0, pos.y + sprite.height.0);
        aabb_intersects_view_rect(entity_min, entity_max, view_min, view_max)
    })
}

fn collect_overlays(overlays: Option<&MeshOverlays>) -> Vec<OverlayCommand> {
    overlays.map_or_else(Vec::new, |o| {
        let mut collected = Vec::with_capacity(o.0.len());
        for entry in &o.0 {
            if entry.visible {
                collected.push(OverlayCommand {
                    mesh: entry.mesh.clone(),
                    material: entry.material.clone(),
                });
            }
        }
        collected
    })
}

#[allow(clippy::too_many_arguments)]
fn collect_draw_commands(
    shape_query: &Query<ShapeItem>,
    text_query: &Query<TextItem>,
    color_mesh_query: &Query<ColorMeshItem>,
    persistent_mesh_query: &Query<PersistentMeshItem>,
    sprite_query: &Query<SpriteItem>,
    draw_queue: &mut DrawQueue,
    view_rect: Option<(Vec2, Vec2)>,
) -> Vec<SortedDrawCommand> {
    let mut commands = draw_queue.drain();

    for (shape, transform, layer, order, vis, mat, stroke, cached) in shape_query.iter() {
        if is_hidden(vis) {
            continue;
        }
        if is_shape_culled(transform.0.translation, &shape.variant, view_rect) {
            continue;
        }
        let mesh = if let Some(cached) = cached {
            cached.0.clone()
        } else if let Ok(m) = tessellate(&shape.variant) {
            m
        } else {
            continue;
        };
        let stroke_cmd = stroke.and_then(|s| {
            tessellate_stroke(&shape.variant, s.width)
                .ok()
                .map(|sm| StrokeCommand {
                    mesh: sm,
                    color: s.color,
                })
        });
        push_sorted_command(
            &mut commands,
            layer,
            order,
            DrawCommand::Shape {
                mesh,
                color: shape.color,
                model: affine2_to_mat4(&transform.0),
                material: mat.cloned(),
                stroke: stroke_cmd,
            },
        );
    }

    for (text, transform, layer, order, vis) in text_query.iter() {
        if is_hidden(vis) {
            continue;
        }
        push_sorted_command(
            &mut commands,
            layer,
            order,
            DrawCommand::Text {
                content: text.content.clone(),
                font_size: text.font_size,
                color: text.color,
                max_width: text.max_width,
                transform: transform.0,
            },
        );
    }

    for (mesh, transform, layer, order, vis, overlays, mat) in color_mesh_query.iter() {
        if is_hidden(vis) || mesh.is_empty() {
            continue;
        }
        push_sorted_command(
            &mut commands,
            layer,
            order,
            DrawCommand::ColorMesh {
                mesh: mesh.0.clone(),
                model: affine2_to_mat4(&transform.0),
                material: mat.cloned(),
                overlays: collect_overlays(overlays),
            },
        );
    }

    for (pcm, transform, layer, order, vis, overlays, mat) in persistent_mesh_query.iter() {
        if is_hidden(vis) {
            continue;
        }
        push_sorted_command(
            &mut commands,
            layer,
            order,
            DrawCommand::PersistentMesh {
                handle: pcm.0,
                model: affine2_to_mat4(&transform.0),
                material: mat.cloned(),
                overlays: collect_overlays(overlays),
            },
        );
    }

    for (sprite, transform, layer, order, vis, mat) in sprite_query.iter() {
        if is_hidden(vis) || !sprite_intersects_view(sprite, transform, view_rect) {
            continue;
        }
        let pos = transform.0.translation;
        push_sorted_command(
            &mut commands,
            layer,
            order,
            DrawCommand::Sprite {
                rect: Rect {
                    x: Pixels(pos.x),
                    y: Pixels(pos.y),
                    width: sprite.width,
                    height: sprite.height,
                    color: sprite.color,
                },
                uv_rect: sprite.uv_rect,
                material: mat.cloned(),
            },
        );
    }

    commands.sort_by_key(|cmd| cmd.sort_key);
    commands
}

fn draw_commands(
    renderer: &mut dyn engine_render::renderer::Renderer,
    cache: &mut GlyphCache,
    commands: &[SortedDrawCommand],
) {
    let mut last_shader = None;
    let mut last_blend_mode = None;

    for cmd in commands {
        match &cmd.command {
            DrawCommand::Shape {
                mesh,
                color,
                model,
                material,
                stroke,
            } => {
                apply_material(
                    renderer,
                    material.as_ref(),
                    &mut last_shader,
                    &mut last_blend_mode,
                );
                renderer.draw_shape(&mesh.vertices, &mesh.indices, *color, *model);
                if let Some(s) = stroke {
                    renderer.draw_shape(&s.mesh.vertices, &s.mesh.indices, s.color, *model);
                }
            }
            DrawCommand::Text {
                content,
                font_size,
                color,
                max_width,
                transform,
            } => {
                draw_text(
                    renderer, cache, content, *font_size, *color, *max_width, transform,
                );
            }
            DrawCommand::ColorMesh {
                mesh,
                model,
                material,
                overlays,
            } => {
                apply_material(
                    renderer,
                    material.as_ref(),
                    &mut last_shader,
                    &mut last_blend_mode,
                );
                renderer.draw_colored_mesh(&mesh.vertices, &mesh.indices, *model);
                draw_overlays(
                    renderer,
                    overlays,
                    *model,
                    &mut last_shader,
                    &mut last_blend_mode,
                );
            }
            DrawCommand::PersistentMesh {
                handle,
                model,
                material,
                overlays,
            } => {
                apply_material(
                    renderer,
                    material.as_ref(),
                    &mut last_shader,
                    &mut last_blend_mode,
                );
                renderer.draw_persistent_colored_mesh(*handle, *model);
                draw_overlays(
                    renderer,
                    overlays,
                    *model,
                    &mut last_shader,
                    &mut last_blend_mode,
                );
            }
            DrawCommand::Sprite {
                rect,
                uv_rect,
                material,
            } => {
                apply_material(
                    renderer,
                    material.as_ref(),
                    &mut last_shader,
                    &mut last_blend_mode,
                );
                renderer.draw_sprite(*rect, *uv_rect);
            }
            DrawCommand::RawText {
                text,
                x,
                y,
                font_size,
                color,
            } => {
                renderer.draw_text(text, *x, *y, *font_size, *color);
            }
        }
    }
}

fn draw_overlays(
    renderer: &mut dyn engine_render::renderer::Renderer,
    overlays: &[OverlayCommand],
    model: [[f32; 4]; 4],
    last_shader: &mut Option<ShaderHandle>,
    last_blend_mode: &mut Option<BlendMode>,
) {
    for entry in overlays {
        apply_material(
            renderer,
            Some(&entry.material),
            last_shader,
            last_blend_mode,
        );
        renderer.draw_colored_mesh(&entry.mesh.vertices, &entry.mesh.indices, model);
    }
}

fn draw_text(
    renderer: &mut dyn engine_render::renderer::Renderer,
    cache: &mut GlyphCache,
    content: &str,
    font_size: f32,
    color: Color,
    max_width: Option<f32>,
    transform: &glam::Affine2,
) {
    if let Some(max_w) = max_width {
        let lines = wrap_text(content, font_size, max_w);
        let line_height = font_size * LINE_HEIGHT_FACTOR;
        let total_height = lines.len().saturating_sub(1) as f32 * line_height;
        let start_y = -total_height * 0.5;
        for (i, line) in lines.iter().enumerate() {
            let line_width = measure_text(line, font_size);
            let y_offset = start_y + i as f32 * line_height;
            let offset = glam::Affine2::from_translation(Vec2::new(-line_width * 0.5, y_offset));
            let line_transform = *transform * offset;
            let model = affine2_to_mat4(&line_transform);
            render_text_transformed(renderer, cache, line, &model, font_size, color);
        }
    } else {
        let text_width = measure_text(content, font_size);
        let center_offset = glam::Affine2::from_translation(Vec2::new(-text_width * 0.5, 0.0));
        let centered_transform = *transform * center_offset;
        let model = affine2_to_mat4(&centered_transform);
        render_text_transformed(renderer, cache, content, &model, font_size, color);
    }
}

/// Unified render system that draws shapes, text, sprites, color meshes, and
/// persistent meshes in a single sorted pass. Draw order is determined by
/// `(RenderLayer, SortOrder)` with insertion order used as a deterministic tie
/// breaker.
///
/// Also drains the `DrawQueue` resource, merging immediate-mode commands into
/// the same sorted pass. Systems that push to `DrawQueue` in `Phase::Render`
/// are guaranteed to complete before this system runs in `Phase::PostRender`.
#[allow(clippy::too_many_arguments)]
pub fn unified_render_system(
    shape_query: Query<ShapeItem>,
    text_query: Query<TextItem>,
    color_mesh_query: Query<ColorMeshItem>,
    persistent_mesh_query: Query<PersistentMeshItem>,
    sprite_query: Query<SpriteItem>,
    camera_query: Query<(&Camera2D, Option<&CameraRotation>)>,
    mut renderer: ResMut<RendererRes>,
    mut draw_queue: ResMut<DrawQueue>,
    mut cache: Local<GlyphCache>,
    mut profiler: Option<ResMut<FrameProfiler>>,
) {
    let view_rect = compute_view_rect(&camera_query, &renderer);

    let t_sort = std::time::Instant::now();
    let commands = collect_draw_commands(
        &shape_query,
        &text_query,
        &color_mesh_query,
        &persistent_mesh_query,
        &sprite_query,
        &mut draw_queue,
        view_rect,
    );
    let sort_us = t_sort.elapsed().as_micros() as u64;

    let t_draw = std::time::Instant::now();
    draw_commands(&mut **renderer, &mut cache, &commands);
    let draw_us = t_draw.elapsed().as_micros() as u64;

    if let Some(p) = profiler.as_deref_mut() {
        p.record_phase("render_sort", sort_us);
        p.record_phase("render_draw", draw_us);
    }
}
// EVOLVE-BLOCK-END
