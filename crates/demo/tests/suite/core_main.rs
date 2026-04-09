#![allow(clippy::unwrap_used)]

use std::sync::{Arc, Mutex};

use axiom2d::prelude::*;
use engine_render::testing::SpyRenderer;

#[test]
fn when_transform_propagation_runs_then_root_entity_gets_global_transform() {
    // Arrange
    let mut world = World::new();
    world.spawn(Transform2D {
        position: Vec2::new(490.0, 260.0),
        ..Transform2D::default()
    });
    let mut schedule = Schedule::default();
    schedule.add_systems(transform_propagation_system);

    // Act
    schedule.run(&mut world);

    // Assert
    let mut query = world.query::<&GlobalTransform2D>();
    let global = query.single(&world).unwrap();
    assert_eq!(global.0.translation, Vec2::new(490.0, 260.0));
}

#[test]
fn when_unified_render_system_runs_then_draw_sprite_called_for_player() {
    // Arrange
    let log = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(log.clone());
    let mut world = World::new();
    world.insert_resource(RendererRes::new(Box::new(spy)));
    world.insert_resource(DrawQueue::default());
    world.spawn((
        Sprite {
            texture: TextureId(0),
            uv_rect: [0.0, 0.0, 1.0, 1.0],
            color: Color::WHITE,
            width: Pixels(300.0),
            height: Pixels(200.0),
        },
        GlobalTransform2D(Affine2::from_translation(Vec2::new(490.0, 260.0))),
    ));
    let mut schedule = Schedule::default();
    schedule.add_systems(unified_render_system);

    // Act
    schedule.run(&mut world);

    // Assert
    let count = log
        .lock()
        .unwrap()
        .iter()
        .filter(|s| s.as_str() == "draw_sprite")
        .count();
    assert_eq!(count, 1);
}

#[test]
fn when_render_phase_runs_then_clear_before_camera_before_draw() {
    // Arrange
    let log = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(log.clone()).with_viewport(800, 600);
    let mut world = World::new();
    world.insert_resource(RendererRes::new(Box::new(spy)));
    world.insert_resource(ClearColor::default());
    world.insert_resource(DrawQueue::default());
    world.spawn(Camera2D::default());
    world.spawn((
        Sprite {
            texture: TextureId(0),
            uv_rect: [0.0, 0.0, 1.0, 1.0],
            color: Color::WHITE,
            width: Pixels(300.0),
            height: Pixels(200.0),
        },
        GlobalTransform2D(Affine2::from_translation(Vec2::new(400.0, 300.0))),
    ));
    let mut schedule = Schedule::default();
    schedule.add_systems((clear_system, camera_prepare_system, unified_render_system).chain());

    // Act
    schedule.run(&mut world);

    // Assert
    let calls = log.lock().unwrap();
    assert_eq!(calls[0], "clear");
    assert_eq!(calls[1], "set_view_projection");
    assert_eq!(calls[2], "set_shader");
    assert_eq!(calls[3], "set_blend_mode");
    assert_eq!(calls[4], "draw_sprite");
}

#[test]
fn when_post_update_systems_run_then_player_entity_gains_global_transform() {
    // Arrange
    let mut world = World::new();
    world.spawn((
        Transform2D {
            position: Vec2::new(490.0, 260.0),
            ..Transform2D::default()
        },
        Sprite {
            texture: TextureId(0),
            uv_rect: [0.0, 0.0, 1.0, 1.0],
            color: Color::WHITE,
            width: Pixels(300.0),
            height: Pixels(200.0),
        },
    ));
    let mut schedule = Schedule::default();
    schedule.add_systems(
        (
            hierarchy_maintenance_system,
            transform_propagation_system,
            visibility_system,
        )
            .chain(),
    );

    // Act
    schedule.run(&mut world);

    // Assert
    let mut query = world.query::<(&Transform2D, &GlobalTransform2D)>();
    assert_eq!(query.iter(&world).count(), 1);
    let (_, global) = query.single(&world).unwrap();
    assert_eq!(global.0.translation, Vec2::new(490.0, 260.0));
}
