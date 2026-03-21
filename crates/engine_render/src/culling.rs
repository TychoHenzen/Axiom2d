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

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use glam::Vec2;

    use super::*;
    use crate::camera::Camera2D;

    #[test]
    fn when_view_rect_at_zoom_one_then_half_extents_equal_half_viewport() {
        // Arrange
        let camera = Camera2D {
            position: Vec2::new(400.0, 300.0),
            zoom: 1.0,
        };

        // Act
        let (min, max) = camera_view_rect(&camera, 800.0, 600.0);

        // Assert
        assert!((min.x - 0.0).abs() < 1e-4);
        assert!((min.y - 0.0).abs() < 1e-4);
        assert!((max.x - 800.0).abs() < 1e-4);
        assert!((max.y - 600.0).abs() < 1e-4);
    }

    #[test]
    fn when_view_rect_at_zoom_two_then_half_extents_are_halved() {
        // Arrange
        let camera = Camera2D {
            position: Vec2::new(400.0, 300.0),
            zoom: 2.0,
        };

        // Act
        let (min, max) = camera_view_rect(&camera, 800.0, 600.0);

        // Assert
        assert!((min.x - 200.0).abs() < 1e-4);
        assert!((min.y - 150.0).abs() < 1e-4);
        assert!((max.x - 600.0).abs() < 1e-4);
        assert!((max.y - 450.0).abs() < 1e-4);
    }

    #[test]
    fn when_entity_fully_inside_view_then_aabb_intersects_returns_true() {
        // Act / Assert
        assert!(aabb_intersects_view_rect(
            Vec2::new(100.0, 100.0),
            Vec2::new(200.0, 200.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(800.0, 600.0),
        ));
    }

    /// @doc: Frustum culling AABB test — entity fully outside on any axis means no intersection
    #[test]
    fn when_entity_completely_left_of_view_then_aabb_intersects_returns_false() {
        // Act / Assert
        assert!(!aabb_intersects_view_rect(
            Vec2::new(-200.0, 0.0),
            Vec2::new(-10.0, 100.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(800.0, 600.0),
        ));
    }

    #[test]
    fn when_entity_completely_right_of_view_then_aabb_intersects_returns_false() {
        // Act / Assert
        assert!(!aabb_intersects_view_rect(
            Vec2::new(850.0, 0.0),
            Vec2::new(1000.0, 100.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(800.0, 600.0),
        ));
    }

    #[test]
    fn when_entity_completely_above_view_then_aabb_intersects_returns_false() {
        // Act / Assert
        assert!(!aabb_intersects_view_rect(
            Vec2::new(0.0, -200.0),
            Vec2::new(100.0, -10.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(800.0, 600.0),
        ));
    }

    #[test]
    fn when_entity_completely_below_view_then_aabb_intersects_returns_false() {
        // Act / Assert
        assert!(!aabb_intersects_view_rect(
            Vec2::new(0.0, 650.0),
            Vec2::new(100.0, 800.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(800.0, 600.0),
        ));
    }

    #[test]
    fn when_entity_partially_overlaps_left_edge_then_aabb_intersects_returns_true() {
        // Act / Assert
        assert!(aabb_intersects_view_rect(
            Vec2::new(-50.0, 100.0),
            Vec2::new(50.0, 200.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(800.0, 600.0),
        ));
    }

    #[test]
    fn when_entity_exactly_touches_view_edge_then_aabb_intersects_returns_true() {
        // Act / Assert
        assert!(aabb_intersects_view_rect(
            Vec2::new(-10.0, 0.0),
            Vec2::new(0.0, 100.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(800.0, 600.0),
        ));
    }

    #[test]
    fn when_entity_contains_entire_view_then_aabb_intersects_returns_true() {
        // Act / Assert
        assert!(aabb_intersects_view_rect(
            Vec2::new(0.0, 0.0),
            Vec2::new(800.0, 600.0),
            Vec2::new(100.0, 100.0),
            Vec2::new(700.0, 500.0),
        ));
    }
}
