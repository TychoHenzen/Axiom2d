use bevy_ecs::prelude::{Component, Query, ResMut, Resource};
use glam::{Mat4, Vec2};
use serde::{Deserialize, Serialize};

use crate::culling::camera_view_rect;
use crate::renderer::RendererRes;

#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Camera2D {
    pub position: Vec2,
    pub zoom: f32,
}

impl Default for Camera2D {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            zoom: 1.0,
        }
    }
}

pub fn world_to_screen(
    world_point: Vec2,
    camera: &Camera2D,
    viewport_width: f32,
    viewport_height: f32,
) -> Vec2 {
    let offset = world_point - camera.position;
    let scaled = offset * camera.zoom;
    Vec2::new(
        scaled.x + viewport_width * 0.5,
        scaled.y + viewport_height * 0.5,
    )
}

pub fn screen_to_world(
    screen_point: Vec2,
    camera: &Camera2D,
    viewport_width: f32,
    viewport_height: f32,
) -> Vec2 {
    let centered = Vec2::new(
        screen_point.x - viewport_width * 0.5,
        screen_point.y - viewport_height * 0.5,
    );
    centered / camera.zoom + camera.position
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn from_camera(camera: &Camera2D, viewport_width: f32, viewport_height: f32) -> Self {
        let (view_min, view_max) = camera_view_rect(camera, viewport_width, viewport_height);
        // Orthographic projection: world x [view_min.x..view_max.x] → NDC [-1..1]
        // World y [view_min.y..view_max.y] → NDC [1..-1] (Y-flip for wgpu)
        let proj = Mat4::orthographic_rh(
            view_min.x, view_max.x, view_max.y, // bottom = max y (Y-flip)
            view_min.y, // top = min y (Y-flip)
            -1.0, 1.0,
        );
        Self {
            view_proj: proj.to_cols_array_2d(),
        }
    }
}

pub fn camera_prepare_system(query: Query<&Camera2D>, mut renderer: ResMut<RendererRes>) {
    let (viewport_width, viewport_height) = renderer.viewport_size();
    if viewport_width == 0 || viewport_height == 0 {
        return;
    }
    let viewport_width = viewport_width as f32;
    let viewport_height = viewport_height as f32;
    let camera = query.iter().next().copied().unwrap_or(Camera2D {
        position: Vec2::new(viewport_width / 2.0, viewport_height / 2.0),
        zoom: 1.0,
    });
    let uniform = CameraUniform::from_camera(&camera, viewport_width, viewport_height);
    renderer.set_view_projection(uniform.view_proj);
}

/// Resolves the viewport size and first camera, returning `None` if the viewport is zero.
pub fn resolve_viewport_camera(
    renderer: &RendererRes,
    camera_query: &Query<&Camera2D>,
) -> Option<(f32, f32, Camera2D)> {
    let (vw, vh) = renderer.viewport_size();
    if vw == 0 || vh == 0 {
        return None;
    }
    let camera = camera_query
        .iter()
        .next()
        .copied()
        .unwrap_or(Camera2D::default());
    Some((vw as f32, vh as f32, camera))
}
