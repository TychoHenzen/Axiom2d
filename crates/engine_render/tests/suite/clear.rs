#![allow(clippy::unwrap_used)]

use std::sync::{Arc, Mutex};

use engine_core::color::Color;
use engine_render::clear::{ClearColor, clear_system};
use engine_render::renderer::RendererRes;
use engine_render::testing::SpyRenderer;

#[test]
fn when_clear_system_runs_then_renderer_clear_receives_clear_color_value() {
    // Arrange
    let expected_color = Color::new(0.1, 0.2, 0.3, 1.0);
    let log = Arc::new(Mutex::new(Vec::new()));
    let color_capture = Arc::new(Mutex::new(None));
    let spy = SpyRenderer::new(log.clone()).with_color_capture(color_capture.clone());

    let mut world = bevy_ecs::world::World::new();
    world.insert_resource(RendererRes::new(Box::new(spy)));
    world.insert_resource(ClearColor(expected_color));

    let mut schedule = bevy_ecs::schedule::Schedule::default();
    schedule.add_systems(clear_system);

    // Act
    schedule.run(&mut world);

    // Assert
    assert_eq!(*color_capture.lock().unwrap(), Some(expected_color));
}
