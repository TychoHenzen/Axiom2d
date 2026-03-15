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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::float_cmp)]
mod tests {
    use std::sync::{Arc, Mutex};

    use glam::{Mat4, Vec2};

    use super::*;
    use crate::culling::aabb_intersects_view_rect;

    fn compute_view_matrix(camera: &Camera2D) -> Mat4 {
        let scale = Mat4::from_scale(glam::Vec3::new(camera.zoom, camera.zoom, 1.0));
        let translation =
            Mat4::from_translation(glam::Vec3::new(-camera.position.x, -camera.position.y, 0.0));
        scale * translation
    }
    use crate::renderer::RendererRes;
    use crate::testing::{SpyRenderer, insert_spy_with_viewport};

    #[test]
    fn when_camera2d_serialized_to_ron_then_deserializes_to_equal_value() {
        // Arrange
        let camera = Camera2D {
            position: Vec2::new(150.0, -75.0),
            zoom: 2.5,
        };

        // Act
        let ron = ron::to_string(&camera).unwrap();
        let back: Camera2D = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(camera, back);
    }

    #[test]
    fn when_camera2d_created_with_defaults_then_position_is_zero_and_zoom_is_one() {
        // Act
        let camera = Camera2D::default();

        // Assert
        assert_eq!(camera.position, Vec2::ZERO);
        assert_eq!(camera.zoom, 1.0);
    }

    #[test]
    fn when_camera_at_origin_with_zoom_one_then_view_matrix_is_identity() {
        // Arrange
        let camera = Camera2D::default();

        // Act
        let view = compute_view_matrix(&camera);

        // Assert
        assert_eq!(view, Mat4::IDENTITY);
    }

    #[test]
    fn when_camera_at_nonzero_position_then_view_matrix_translates_by_negative_position() {
        // Arrange
        let camera = Camera2D {
            position: Vec2::new(100.0, 200.0),
            zoom: 1.0,
        };

        // Act
        let view = compute_view_matrix(&camera);

        // Assert
        let translation = view.w_axis;
        assert!((translation.x - (-100.0)).abs() < 1e-6);
        assert!((translation.y - (-200.0)).abs() < 1e-6);
    }

    #[test]
    fn when_camera_at_origin_with_zoom_two_then_view_matrix_scales_by_two() {
        // Arrange
        let camera = Camera2D {
            position: Vec2::ZERO,
            zoom: 2.0,
        };

        // Act
        let view = compute_view_matrix(&camera);

        // Assert
        assert!((view.x_axis.x - 2.0).abs() < 1e-6);
        assert!((view.y_axis.y - 2.0).abs() < 1e-6);
    }

    #[test]
    fn when_camera_at_origin_with_zoom_half_then_view_matrix_scales_by_half() {
        // Arrange
        let camera = Camera2D {
            position: Vec2::ZERO,
            zoom: 0.5,
        };

        // Act
        let view = compute_view_matrix(&camera);

        // Assert
        assert!((view.x_axis.x - 0.5).abs() < 1e-6);
        assert!((view.y_axis.y - 0.5).abs() < 1e-6);
    }

    #[test]
    fn when_camera_at_position_with_nonunit_zoom_then_view_matrix_combines_translation_and_scale() {
        // Arrange
        let camera = Camera2D {
            position: Vec2::new(50.0, 100.0),
            zoom: 2.0,
        };

        // Act
        let view = compute_view_matrix(&camera);

        // Assert
        assert!((view.x_axis.x - 2.0).abs() < 1e-6);
        assert!((view.y_axis.y - 2.0).abs() < 1e-6);
        assert!((view.w_axis.x - (-100.0)).abs() < 1e-6);
        assert!((view.w_axis.y - (-200.0)).abs() < 1e-6);
    }

    /// @doc: Camera position defines the world point that appears at screen center
    #[test]
    fn when_world_point_matches_camera_center_then_world_to_screen_returns_screen_center() {
        // Arrange
        let camera = Camera2D {
            position: Vec2::new(400.0, 300.0),
            zoom: 1.0,
        };

        // Act
        let screen = world_to_screen(Vec2::new(400.0, 300.0), &camera, 800.0, 600.0);

        // Assert
        assert!((screen.x - 400.0).abs() < 1e-4);
        assert!((screen.y - 300.0).abs() < 1e-4);
    }

    #[test]
    fn when_world_point_at_viewport_corner_then_world_to_screen_returns_corner() {
        // Arrange
        let camera = Camera2D {
            position: Vec2::new(400.0, 300.0),
            zoom: 1.0,
        };

        // Act
        let screen = world_to_screen(Vec2::new(800.0, 600.0), &camera, 800.0, 600.0);

        // Assert
        assert!((screen.x - 800.0).abs() < 1e-4);
        assert!((screen.y - 600.0).abs() < 1e-4);
    }

    /// @doc: Zoom multiplies screen-space distances — zoom 2 means objects appear 2x larger
    #[test]
    fn when_world_point_at_zoom_two_then_world_to_screen_reflects_magnification() {
        // Arrange
        let camera = Camera2D {
            position: Vec2::new(400.0, 300.0),
            zoom: 2.0,
        };

        // Act
        let screen = world_to_screen(Vec2::new(450.0, 300.0), &camera, 800.0, 600.0);

        // Assert
        assert!((screen.x - 500.0).abs() < 1e-4);
        assert!((screen.y - 300.0).abs() < 1e-4);
    }

    #[test]
    fn when_screen_center_then_screen_to_world_returns_camera_position() {
        // Arrange
        let camera = Camera2D {
            position: Vec2::new(400.0, 300.0),
            zoom: 1.0,
        };

        // Act
        let world = screen_to_world(Vec2::new(400.0, 300.0), &camera, 800.0, 600.0);

        // Assert
        assert!((world.x - 400.0).abs() < 1e-4);
        assert!((world.y - 300.0).abs() < 1e-4);
    }

    /// @doc: `world_to_screen` and `screen_to_world` are exact inverses — roundtrip recovers the original point
    #[test]
    fn when_screen_to_world_after_world_to_screen_then_recovers_original_point() {
        // Arrange
        let camera = Camera2D {
            position: Vec2::new(150.0, 75.0),
            zoom: 1.5,
        };
        let original = Vec2::new(200.0, 100.0);

        // Act
        let screen = world_to_screen(original, &camera, 800.0, 600.0);
        let recovered = screen_to_world(screen, &camera, 800.0, 600.0);

        // Assert
        assert!((recovered.x - original.x).abs() < 1e-4);
        assert!((recovered.y - original.y).abs() < 1e-4);
    }

    proptest::proptest! {
        #[test]
        fn when_any_world_point_then_screen_to_world_of_world_to_screen_recovers_original(
            wx in -1000.0_f32..=1000.0,
            wy in -1000.0_f32..=1000.0,
            cx in -500.0_f32..=500.0,
            cy in -500.0_f32..=500.0,
            zoom in 0.1_f32..=10.0,
            vw in 1.0_f32..=2000.0,
            vh in 1.0_f32..=2000.0,
        ) {
            // Arrange
            let camera = Camera2D { position: Vec2::new(cx, cy), zoom };
            let point = Vec2::new(wx, wy);

            // Act
            let screen = world_to_screen(point, &camera, vw, vh);
            let recovered = screen_to_world(screen, &camera, vw, vh);

            // Assert
            assert!(
                (recovered.x - point.x).abs() < 1e-2,
                "x: expected {}, got {}",
                point.x,
                recovered.x
            );
            assert!(
                (recovered.y - point.y).abs() < 1e-2,
                "y: expected {}, got {}",
                point.y,
                recovered.y
            );
        }
    }

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

    /// @doc: Default camera produces pixel-perfect 1:1 mapping — world origin lands at NDC center
    #[test]
    fn when_camera_uniform_from_camera_at_origin_zoom_one_then_origin_maps_to_ndc_center() {
        // Arrange
        let camera = Camera2D::default();

        // Act
        let uniform = CameraUniform::from_camera(&camera, 800.0, 600.0);

        // Assert — camera at origin: world (0,0) is the view center → NDC (0,0)
        let vp = Mat4::from_cols_array_2d(&uniform.view_proj);
        let origin_ndc = vp * glam::Vec4::new(0.0, 0.0, 0.0, 1.0);
        assert!((origin_ndc.x).abs() < 1e-5);
        assert!((origin_ndc.y).abs() < 1e-5);
    }

    fn run_camera_prepare(world: &mut bevy_ecs::world::World) {
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(camera_prepare_system);
        schedule.run(world);
    }

    #[test]
    fn when_camera_uniform_from_camera_at_center_then_viewport_corners_map_to_ndc_corners() {
        // Arrange
        let camera = Camera2D {
            position: Vec2::new(400.0, 300.0),
            zoom: 1.0,
        };

        // Act
        let uniform = CameraUniform::from_camera(&camera, 800.0, 600.0);

        // Assert
        let vp = Mat4::from_cols_array_2d(&uniform.view_proj);
        let top_left = vp * glam::Vec4::new(0.0, 0.0, 0.0, 1.0);
        let bottom_right = vp * glam::Vec4::new(800.0, 600.0, 0.0, 1.0);
        assert!((top_left.x - (-1.0)).abs() < 1e-5);
        assert!((top_left.y - 1.0).abs() < 1e-5);
        assert!((bottom_right.x - 1.0).abs() < 1e-5);
        assert!((bottom_right.y - (-1.0)).abs() < 1e-5);
    }

    #[test]
    fn when_camera_prepare_system_runs_with_camera_then_set_view_projection_called() {
        // Arrange
        let mut world = bevy_ecs::world::World::new();
        let log = insert_spy_with_viewport(&mut world, 800, 600);
        world.spawn(Camera2D::default());

        // Act
        run_camera_prepare(&mut world);

        // Assert
        let log = log.lock().unwrap();
        assert!(log.contains(&"set_view_projection".to_string()));
    }

    /// @doc: `camera_prepare_system` always sets a projection — defaults to viewport-centered ortho when no `Camera2D` entity exists
    #[test]
    fn when_camera_prepare_system_runs_without_camera_then_default_ortho_set() {
        // Arrange
        let mut world = bevy_ecs::world::World::new();
        let log = insert_spy_with_viewport(&mut world, 800, 600);

        // Act
        run_camera_prepare(&mut world);

        // Assert
        let log = log.lock().unwrap();
        assert!(log.contains(&"set_view_projection".to_string()));
    }

    #[test]
    fn when_camera_uniform_y_flip_then_top_maps_to_positive_ndc_y() {
        // Arrange — camera centered on viewport, zoom 1
        let camera = Camera2D {
            position: Vec2::new(400.0, 300.0),
            zoom: 1.0,
        };

        // Act
        let uniform = CameraUniform::from_camera(&camera, 800.0, 600.0);

        // Assert — top of viewport (y=0) maps to NDC y=+1 (Y-flip)
        let vp = Mat4::from_cols_array_2d(&uniform.view_proj);
        let top_center = vp * glam::Vec4::new(400.0, 0.0, 0.0, 1.0);
        assert!(
            (top_center.y - 1.0).abs() < 1e-4,
            "top of viewport should map to NDC y=+1, got {}",
            top_center.y
        );
        // Bottom of viewport (y=600) maps to NDC y=-1
        let bottom_center = vp * glam::Vec4::new(400.0, 600.0, 0.0, 1.0);
        assert!(
            (bottom_center.y - (-1.0)).abs() < 1e-4,
            "bottom of viewport should map to NDC y=-1, got {}",
            bottom_center.y
        );
    }

    #[test]
    fn when_viewport_width_zero_then_camera_prepare_system_skips() {
        // Arrange
        let mut world = bevy_ecs::world::World::new();
        let log = insert_spy_with_viewport(&mut world, 0, 600);
        world.spawn(Camera2D::default());

        // Act
        run_camera_prepare(&mut world);

        // Assert
        let log = log.lock().unwrap();
        assert!(
            !log.contains(&"set_view_projection".to_string()),
            "should skip when viewport width is zero"
        );
    }

    #[test]
    fn when_viewport_height_zero_then_camera_prepare_system_skips() {
        // Arrange
        let mut world = bevy_ecs::world::World::new();
        let log = insert_spy_with_viewport(&mut world, 800, 0);
        world.spawn(Camera2D::default());

        // Act
        run_camera_prepare(&mut world);

        // Assert
        let log = log.lock().unwrap();
        assert!(
            !log.contains(&"set_view_projection".to_string()),
            "should skip when viewport height is zero"
        );
    }

    #[test]
    fn when_no_camera_then_system_uses_viewport_center() {
        // Arrange
        let mut world = bevy_ecs::world::World::new();
        let matrix: crate::testing::MatrixCapture = Arc::new(Mutex::new(None));
        let log = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log)
            .with_viewport(800, 600)
            .with_matrix_capture(matrix.clone());
        world.insert_resource(RendererRes::new(Box::new(spy)));

        // Act
        run_camera_prepare(&mut world);

        // Assert
        let actual = matrix.lock().unwrap().unwrap();
        let expected_cam = Camera2D {
            position: Vec2::new(400.0, 300.0),
            zoom: 1.0,
        };
        let expected = CameraUniform::from_camera(&expected_cam, 800.0, 600.0);
        assert_eq!(actual, expected.view_proj);
    }
}
