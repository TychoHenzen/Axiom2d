use crate::renderer::RendererRes;
use bevy_ecs::prelude::{Component, Query, ResMut, Resource};
use glam::{Mat2, Mat4, Vec2, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Camera2D {
    pub position: Vec2,
    pub zoom: f32,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct CameraRotation(pub f32);

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
    world_to_screen_with_rotation(world_point, camera, 0.0, viewport_width, viewport_height)
}

pub fn world_to_screen_with_rotation(
    world_point: Vec2,
    camera: &Camera2D,
    rotation: f32,
    viewport_width: f32,
    viewport_height: f32,
) -> Vec2 {
    let offset = world_point - camera.position;
    let rotated = Mat2::from_angle(-rotation) * offset;
    let scaled = rotated * camera.zoom;
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
    screen_to_world_with_rotation(screen_point, camera, 0.0, viewport_width, viewport_height)
}

pub fn screen_to_world_with_rotation(
    screen_point: Vec2,
    camera: &Camera2D,
    rotation: f32,
    viewport_width: f32,
    viewport_height: f32,
) -> Vec2 {
    let centered = Vec2::new(
        screen_point.x - viewport_width * 0.5,
        screen_point.y - viewport_height * 0.5,
    );
    let unscaled = centered / camera.zoom;
    let rotated = Mat2::from_angle(rotation) * unscaled;
    rotated + camera.position
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn from_camera(camera: &Camera2D, viewport_width: f32, viewport_height: f32) -> Self {
        Self::from_camera_with_rotation(camera, 0.0, viewport_width, viewport_height)
    }

    pub fn from_camera_with_rotation(
        camera: &Camera2D,
        rotation: f32,
        viewport_width: f32,
        viewport_height: f32,
    ) -> Self {
        let proj = Mat4::orthographic_rh(
            -viewport_width * 0.5,
            viewport_width * 0.5,
            viewport_height * 0.5,
            -viewport_height * 0.5,
            -1.0,
            1.0,
        );
        let view = Mat4::from_scale(Vec3::new(camera.zoom, camera.zoom, 1.0))
            * Mat4::from_rotation_z(-rotation)
            * Mat4::from_translation(Vec3::new(-camera.position.x, -camera.position.y, 0.0));
        Self {
            view_proj: (proj * view).to_cols_array_2d(),
        }
    }
}

pub fn camera_prepare_system(
    query: Query<(&Camera2D, Option<&CameraRotation>)>,
    mut renderer: ResMut<RendererRes>,
) {
    let (viewport_width, viewport_height) = renderer.viewport_size();
    if viewport_width == 0 || viewport_height == 0 {
        return;
    }
    let viewport_width = viewport_width as f32;
    let viewport_height = viewport_height as f32;
    let (camera, rotation) = query.iter().next().map_or(
        (
            Camera2D {
                position: Vec2::new(viewport_width / 2.0, viewport_height / 2.0),
                zoom: 1.0,
            },
            0.0,
        ),
        |(camera, rotation)| (*camera, rotation.map_or(0.0, |rotation| rotation.0)),
    );
    let uniform = CameraUniform::from_camera_with_rotation(
        &camera,
        rotation,
        viewport_width,
        viewport_height,
    );
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
