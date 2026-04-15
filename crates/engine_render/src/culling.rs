use bevy_ecs::prelude::Query;
use glam::Vec2;

use crate::camera::{Camera2D, CameraRotation};
use crate::renderer::RendererRes;

pub fn camera_view_rect(
    camera: &Camera2D,
    viewport_width: f32,
    viewport_height: f32,
) -> (Vec2, Vec2) {
    camera_view_rect_with_rotation(camera, 0.0, viewport_width, viewport_height)
}

pub fn camera_view_rect_with_rotation(
    camera: &Camera2D,
    rotation: f32,
    viewport_width: f32,
    viewport_height: f32,
) -> (Vec2, Vec2) {
    let half_w = viewport_width / (2.0 * camera.zoom);
    let half_h = viewport_height / (2.0 * camera.zoom);
    let sin = rotation.sin().abs();
    let cos = rotation.cos().abs();
    let bound_half_w = half_w * cos + half_h * sin;
    let bound_half_h = half_w * sin + half_h * cos;
    let min = Vec2::new(
        camera.position.x - bound_half_w,
        camera.position.y - bound_half_h,
    );
    let max = Vec2::new(
        camera.position.x + bound_half_w,
        camera.position.y + bound_half_h,
    );
    (min, max)
}

pub fn compute_view_rect(
    camera_query: &Query<(&Camera2D, Option<&CameraRotation>)>,
    renderer: &RendererRes,
) -> Option<(Vec2, Vec2)> {
    camera_query.iter().next().map(|(cam, rotation)| {
        let (vw, vh) = renderer.viewport_size();
        camera_view_rect_with_rotation(
            cam,
            rotation.map_or(0.0, |rotation| rotation.0),
            vw as f32,
            vh as f32,
        )
    })
}

pub fn aabb_intersects_view_rect(
    entity_min: Vec2,
    entity_max: Vec2,
    view_min: Vec2,
    view_max: Vec2,
) -> bool {
    entity_max.x >= view_min.x
        && entity_min.x <= view_max.x
        && entity_max.y >= view_min.y
        && entity_min.y <= view_max.y
}
