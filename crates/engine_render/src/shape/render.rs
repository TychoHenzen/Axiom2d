use bevy_ecs::prelude::{Query, Res, ResMut, Resource};
use engine_scene::prelude::{EffectiveVisibility, GlobalTransform2D, RenderLayer, SortOrder};
use glam::Vec2;

use super::cache::CachedMesh;
use super::components::{Shape, ShapeVariant, Stroke};
use super::tessellate::{shape_aabb, tessellate, tessellate_stroke};
use crate::camera::{Camera2D, CameraRotation};
use crate::culling::{aabb_intersects_view_rect, compute_view_rect};
use crate::material::{Material2d, apply_material};
use crate::renderer::RendererRes;

/// Insert this resource to disable `shape_render_system`, allowing a unified
/// render system to take over shape drawing alongside other draw calls.
#[derive(Resource)]
pub struct ShapeRenderDisabled;

pub fn affine2_to_mat4(affine: &glam::Affine2) -> [[f32; 4]; 4] {
    let m = affine.matrix2;
    let t = affine.translation;
    [
        [m.x_axis.x, m.x_axis.y, 0.0, 0.0],
        [m.y_axis.x, m.y_axis.y, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [t.x, t.y, 0.0, 1.0],
    ]
}

pub fn is_shape_culled(pos: Vec2, variant: &ShapeVariant, view_rect: Option<(Vec2, Vec2)>) -> bool {
    let Some((view_min, view_max)) = view_rect else {
        return false;
    };
    let (local_min, local_max) = shape_aabb(variant);
    let r = local_min.abs().max(local_max.abs()).length();
    let entity_min = Vec2::new(pos.x - r, pos.y - r);
    let entity_max = Vec2::new(pos.x + r, pos.y + r);
    !aabb_intersects_view_rect(entity_min, entity_max, view_min, view_max)
}

type ShapeQuery<'w> = (
    &'w Shape,
    &'w GlobalTransform2D,
    Option<&'w RenderLayer>,
    Option<&'w SortOrder>,
    Option<&'w EffectiveVisibility>,
    Option<&'w Material2d>,
    Option<&'w Stroke>,
    Option<&'w CachedMesh>,
);

pub fn shape_render_system(
    disabled: Option<Res<ShapeRenderDisabled>>,
    query: Query<ShapeQuery>,
    camera_query: Query<(&Camera2D, Option<&CameraRotation>)>,
    mut renderer: ResMut<RendererRes>,
) {
    if disabled.is_some() {
        return;
    }
    let view_rect = compute_view_rect(&camera_query, &renderer);
    let mut shapes: Vec<_> = query.iter().filter(|t| t.4.is_none_or(|v| v.0)).collect();
    shapes.sort_by_key(|t| {
        (
            t.2.copied().unwrap_or(RenderLayer::World),
            t.3.copied().unwrap_or_default(),
        )
    });
    let mut last_shader = None;
    let mut last_blend_mode = None;
    for (shape, transform, _, _, _, mat, stroke, cached) in shapes {
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
}
