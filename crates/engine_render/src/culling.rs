use bevy_ecs::prelude::Query;
use glam::Vec2;

use crate::camera::Camera2D;
use crate::renderer::RendererRes;

pub fn camera_view_rect(
    camera: &Camera2D,
    viewport_width: f32,
    viewport_height: f32,
) -> (Vec2, Vec2) {
    let half_w = viewport_width / (2.0 * camera.zoom);
    let half_h = viewport_height / (2.0 * camera.zoom);
    let min = Vec2::new(camera.position.x - half_w, camera.position.y - half_h);
    let max = Vec2::new(camera.position.x + half_w, camera.position.y + half_h);
    (min, max)
}

pub fn compute_view_rect(
    camera_query: &Query<&Camera2D>,
    renderer: &RendererRes,
) -> Option<(Vec2, Vec2)> {
    camera_query.iter().next().map(|cam| {
        let (vw, vh) = renderer.viewport_size();
        camera_view_rect(cam, vw as f32, vh as f32)
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
