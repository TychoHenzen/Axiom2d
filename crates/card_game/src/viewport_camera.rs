use bevy_ecs::prelude::Query;
use engine_render::prelude::{Camera2D, RendererRes};

/// Resolves the viewport size and first camera, returning `None` if the viewport is zero.
pub(crate) fn resolve_viewport_camera(
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
