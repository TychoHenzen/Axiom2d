#![allow(clippy::unwrap_used, clippy::float_cmp)]

use std::sync::{Arc, Mutex};

use engine_render::camera::{
    Camera2D, CameraUniform, camera_prepare_system, screen_to_world, world_to_screen,
};
use engine_render::prelude::*;
use engine_render::renderer::RendererRes;
use engine_render::testing::{SpyRenderer, insert_spy_with_viewport};

use glam::{Mat4, Vec2};

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
fn when_camera_at_origin_zoom_one_then_ndc_center_is_zero() {
    // Arrange
    let camera = Camera2D::default();

    // Act
    let uniform = CameraUniform::from_camera(&camera, 800.0, 600.0);

    // Assert — world origin (camera position) maps to NDC (0,0)
    let vp = Mat4::from_cols_array_2d(&uniform.view_proj);
    let origin = vp * glam::Vec4::new(0.0, 0.0, 0.0, 1.0);
    assert!((origin.x).abs() < 1e-5);
    assert!((origin.y).abs() < 1e-5);
}

#[test]
fn when_camera_at_offset_then_offset_point_maps_to_ndc_center() {
    // Arrange
    let camera = Camera2D {
        position: Vec2::new(100.0, 200.0),
        zoom: 1.0,
    };

    // Act
    let uniform = CameraUniform::from_camera(&camera, 800.0, 600.0);

    // Assert — camera position maps to NDC (0,0)
    let vp = Mat4::from_cols_array_2d(&uniform.view_proj);
    let center = vp * glam::Vec4::new(100.0, 200.0, 0.0, 1.0);
    assert!((center.x).abs() < 1e-5);
    assert!((center.y).abs() < 1e-5);
}

#[test]
fn when_camera_zoom_two_then_half_viewport_in_world_maps_to_ndc_edge() {
    // Arrange — zoom 2 means half the world extent fits in the viewport
    let camera = Camera2D {
        position: Vec2::ZERO,
        zoom: 2.0,
    };

    // Act
    let uniform = CameraUniform::from_camera(&camera, 800.0, 600.0);

    // Assert — at zoom 2, world point at (200, 0) maps to NDC x=1
    let vp = Mat4::from_cols_array_2d(&uniform.view_proj);
    let edge = vp * glam::Vec4::new(200.0, 0.0, 0.0, 1.0);
    assert!((edge.x - 1.0).abs() < 1e-4);
}

#[test]
fn when_camera_zoom_half_then_double_viewport_in_world_maps_to_ndc_edge() {
    // Arrange — zoom 0.5 means double the world extent fits in the viewport
    let camera = Camera2D {
        position: Vec2::ZERO,
        zoom: 0.5,
    };

    // Act
    let uniform = CameraUniform::from_camera(&camera, 800.0, 600.0);

    // Assert — at zoom 0.5, world point at (800, 0) maps to NDC x=1
    let vp = Mat4::from_cols_array_2d(&uniform.view_proj);
    let edge = vp * glam::Vec4::new(800.0, 0.0, 0.0, 1.0);
    assert!((edge.x - 1.0).abs() < 1e-4);
}

#[test]
fn when_camera_at_position_with_zoom_then_ndc_combines_offset_and_scale() {
    // Arrange
    let camera = Camera2D {
        position: Vec2::new(50.0, 100.0),
        zoom: 2.0,
    };

    // Act
    let uniform = CameraUniform::from_camera(&camera, 800.0, 600.0);

    // Assert — camera center maps to NDC (0,0), edge point maps correctly
    let vp = Mat4::from_cols_array_2d(&uniform.view_proj);
    let center = vp * glam::Vec4::new(50.0, 100.0, 0.0, 1.0);
    assert!((center.x).abs() < 1e-4);
    assert!((center.y).abs() < 1e-4);
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
    let matrix: engine_render::testing::MatrixCapture = Arc::new(Mutex::new(None));
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
