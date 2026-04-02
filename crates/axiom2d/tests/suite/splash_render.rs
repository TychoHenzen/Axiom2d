#![allow(clippy::unwrap_used)]

use axiom2d::prelude::*;
use bevy_ecs::schedule::Schedule;
use bevy_ecs::world::World;
use engine_render::prelude::{PathCommand, split_contours};
use glam::Vec2;

fn run_splash_render(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(splash_render_system);
    schedule.run(world);
}

#[test]
fn when_splash_done_then_render_system_does_not_set_projection() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(SplashScreen {
        elapsed: 3.0,
        duration: 2.0,
        done: true,
    });
    let log = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let spy = engine_render::testing::SpyRenderer::new(log.clone()).with_viewport(800, 600);
    world.insert_resource(engine_render::prelude::RendererRes::new(Box::new(spy)));

    // Act
    run_splash_render(&mut world);

    // Assert
    let calls = log.lock().unwrap();
    assert!(
        !calls.iter().any(|c| c == "set_view_projection"),
        "should not set projection when done"
    );
}

#[test]
fn when_viewport_zero_width_then_render_system_does_not_set_projection() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(SplashScreen::new(2.0));
    let log = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let spy = engine_render::testing::SpyRenderer::new(log.clone()).with_viewport(0, 600);
    world.insert_resource(engine_render::prelude::RendererRes::new(Box::new(spy)));

    // Act
    run_splash_render(&mut world);

    // Assert
    let calls = log.lock().unwrap();
    assert!(
        !calls.iter().any(|c| c == "set_view_projection"),
        "should not set projection when width=0"
    );
}

#[test]
fn when_viewport_zero_height_then_render_system_does_not_set_projection() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(SplashScreen::new(2.0));
    let log = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let spy = engine_render::testing::SpyRenderer::new(log.clone()).with_viewport(800, 0);
    world.insert_resource(engine_render::prelude::RendererRes::new(Box::new(spy)));

    // Act
    run_splash_render(&mut world);

    // Assert
    let calls = log.lock().unwrap();
    assert!(
        !calls.iter().any(|c| c == "set_view_projection"),
        "should not set projection when height=0"
    );
}

#[test]
fn when_splash_active_and_viewport_valid_then_sets_projection() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(SplashScreen::new(2.0));
    let log = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let spy = engine_render::testing::SpyRenderer::new(log.clone()).with_viewport(800, 600);
    world.insert_resource(engine_render::prelude::RendererRes::new(Box::new(spy)));

    // Act
    run_splash_render(&mut world);

    // Assert
    let calls = log.lock().unwrap();
    assert!(
        calls.iter().any(|c| c == "set_view_projection"),
        "should set projection when active with valid viewport"
    );
}

#[test]
fn when_splash_render_zoom_computed_then_uses_min_of_width_and_height_ratios() {
    // Arrange — 1000x300 viewport: vw/500=2.0, vh/300=1.0 → zoom=1.0
    let mut world = World::new();
    world.insert_resource(SplashScreen::new(2.0));
    let matrix = std::sync::Arc::new(std::sync::Mutex::new(None));
    let log = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let spy = engine_render::testing::SpyRenderer::new(log)
        .with_viewport(1000, 300)
        .with_matrix_capture(matrix.clone());
    world.insert_resource(engine_render::prelude::RendererRes::new(Box::new(spy)));

    // Act
    run_splash_render(&mut world);

    // Assert — verify projection was set (matrix captured)
    let mat = matrix.lock().unwrap();
    assert!(mat.is_some(), "projection matrix should be captured");
}

#[test]
fn when_splash_render_wide_viewport_then_zoom_limited_by_height() {
    // Arrange — 2000x300: vw/500=4.0, vh/300=1.0 → zoom=1.0
    // vs 2000x600: vw/500=4.0, vh/300=2.0 → zoom=2.0
    // Different zoom → different projection matrix
    let mut world_a = World::new();
    world_a.insert_resource(SplashScreen::new(2.0));
    let matrix_a = std::sync::Arc::new(std::sync::Mutex::new(None));
    let log_a = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let spy_a = engine_render::testing::SpyRenderer::new(log_a)
        .with_viewport(2000, 300)
        .with_matrix_capture(matrix_a.clone());
    world_a.insert_resource(engine_render::prelude::RendererRes::new(Box::new(spy_a)));

    let mut world_b = World::new();
    world_b.insert_resource(SplashScreen::new(2.0));
    let matrix_b = std::sync::Arc::new(std::sync::Mutex::new(None));
    let log_b = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let spy_b = engine_render::testing::SpyRenderer::new(log_b)
        .with_viewport(2000, 600)
        .with_matrix_capture(matrix_b.clone());
    world_b.insert_resource(engine_render::prelude::RendererRes::new(Box::new(spy_b)));

    // Act
    run_splash_render(&mut world_a);
    run_splash_render(&mut world_b);

    // Assert — different viewports → different matrices (zoom differs)
    let mat_a = matrix_a.lock().unwrap().unwrap();
    let mat_b = matrix_b.lock().unwrap().unwrap();
    assert_ne!(
        mat_a, mat_b,
        "different viewport heights should produce different projections"
    );
}

#[test]
fn when_split_contours_called_then_splits_on_moveto() {
    // Arrange
    let commands = vec![
        PathCommand::MoveTo(Vec2::ZERO),
        PathCommand::LineTo(Vec2::X),
        PathCommand::Close,
        PathCommand::MoveTo(Vec2::Y),
        PathCommand::LineTo(Vec2::ONE),
        PathCommand::Close,
    ];

    // Act
    let contours = split_contours(&commands);

    // Assert
    assert_eq!(contours.len(), 2);
    assert_eq!(contours[0].len(), 3);
    assert_eq!(contours[1].len(), 3);
}

#[test]
#[allow(clippy::float_cmp)]
fn when_splash_render_square_viewport_then_zoom_matches_expected_matrix() {
    // Arrange — 1000x600: vw/500=2.0, vh/300=2.0 → zoom=2.0
    let mut world = World::new();
    world.insert_resource(SplashScreen::new(2.0));
    let matrix = std::sync::Arc::new(std::sync::Mutex::new(None));
    let log = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let spy = engine_render::testing::SpyRenderer::new(log)
        .with_viewport(1000, 600)
        .with_matrix_capture(matrix.clone());
    world.insert_resource(engine_render::prelude::RendererRes::new(Box::new(spy)));

    let expected_camera = engine_render::prelude::Camera2D {
        position: Vec2::new(0.0, 15.0),
        zoom: 2.0,
    };
    let expected =
        engine_render::prelude::CameraUniform::from_camera(&expected_camera, 1000.0, 600.0);

    // Act
    run_splash_render(&mut world);

    // Assert
    let mat = matrix.lock().unwrap().expect("matrix should be captured");
    assert_eq!(
        mat, expected.view_proj,
        "zoom=2.0 produces the wrong projection matrix"
    );
}

#[test]
#[allow(clippy::float_cmp)]
fn when_splash_render_height_constrained_viewport_then_zoom_limited_by_height_ratio() {
    // Arrange — 2000x300: vw/500=4.0, vh/300=1.0 → zoom=1.0
    let mut world = World::new();
    world.insert_resource(SplashScreen::new(2.0));
    let matrix = std::sync::Arc::new(std::sync::Mutex::new(None));
    let log = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let spy = engine_render::testing::SpyRenderer::new(log)
        .with_viewport(2000, 300)
        .with_matrix_capture(matrix.clone());
    world.insert_resource(engine_render::prelude::RendererRes::new(Box::new(spy)));

    let expected_camera = engine_render::prelude::Camera2D {
        position: Vec2::new(0.0, 15.0),
        zoom: 1.0,
    };
    let expected =
        engine_render::prelude::CameraUniform::from_camera(&expected_camera, 2000.0, 300.0);

    // Act
    run_splash_render(&mut world);

    // Assert
    let mat = matrix.lock().unwrap().expect("matrix should be captured");
    assert_eq!(
        mat, expected.view_proj,
        "zoom=1.0 (height-limited) produces the wrong projection matrix"
    );
}

#[test]
#[allow(clippy::float_cmp)]
fn when_splash_render_width_constrained_then_zoom_limited_by_width_ratio() {
    // Arrange — 250x600: vw/500=0.5, vh/300=2.0 → zoom=0.5
    let mut world = World::new();
    world.insert_resource(SplashScreen::new(2.0));
    let matrix = std::sync::Arc::new(std::sync::Mutex::new(None));
    let log = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let spy = engine_render::testing::SpyRenderer::new(log)
        .with_viewport(250, 600)
        .with_matrix_capture(matrix.clone());
    world.insert_resource(engine_render::prelude::RendererRes::new(Box::new(spy)));

    let expected_camera = engine_render::prelude::Camera2D {
        position: Vec2::new(0.0, 15.0),
        zoom: 0.5,
    };
    let expected =
        engine_render::prelude::CameraUniform::from_camera(&expected_camera, 250.0, 600.0);

    // Act
    run_splash_render(&mut world);

    // Assert
    let mat = matrix.lock().unwrap().expect("matrix should be captured");
    assert_eq!(
        mat, expected.view_proj,
        "zoom=0.5 (width-limited) produces the wrong projection matrix"
    );
}
