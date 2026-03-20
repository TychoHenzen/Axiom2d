mod scene;
mod systems;
mod types;

use axiom2d::prelude::*;

use scene::{spawn_camera, spawn_nebula, spawn_planets, spawn_sun};
use systems::{camera_pan_system, camera_zoom_system, count_frames, orbit_system};
use types::{FrameCount, action};

fn setup(app: &mut App) {
    app.add_plugin(DefaultPlugins);

    let config = WindowConfig {
        title: "Axiom2d Solar System",
        ..Default::default()
    };
    let mut action_map = ActionMap::default();
    action_map.bind(action::MOVE_RIGHT, vec![KeyCode::ArrowRight]);
    action_map.bind(action::MOVE_LEFT, vec![KeyCode::ArrowLeft]);
    action_map.bind(action::MOVE_UP, vec![KeyCode::ArrowUp]);
    action_map.bind(action::MOVE_DOWN, vec![KeyCode::ArrowDown]);
    action_map.bind(action::ZOOM_IN, vec![KeyCode::Equal]);
    action_map.bind(action::ZOOM_OUT, vec![KeyCode::Minus]);
    app.world_mut().insert_resource(action_map);
    app.world_mut().insert_resource(FrameCount::default());
    app.world_mut().insert_resource(ClearColor(Color::BLACK));
    app.world_mut().insert_resource(BloomSettings {
        enabled: true,
        threshold: 0.6,
        intensity: 0.4,
        blur_radius: 6,
    });

    spawn_sun(app.world_mut());
    spawn_planets(app.world_mut());
    spawn_nebula(app.world_mut());
    spawn_camera(app.world_mut());

    app.set_window_config(config).add_systems(
        Phase::Update,
        (
            count_frames,
            orbit_system,
            camera_pan_system,
            camera_zoom_system,
        ),
    );
}

fn main() {
    let mut app = App::new();
    setup(&mut app);
    app.run();
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::float_cmp)]
mod tests {
    use std::sync::{Arc, Mutex};

    use engine_render::testing::SpyRenderer;

    use super::*;

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
    fn when_sprite_render_system_runs_then_draw_sprite_called_for_player() {
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log.clone());
        let mut world = World::new();
        world.insert_resource(RendererRes::new(Box::new(spy)));
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
        schedule.add_systems(sprite_render_system);

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
    fn when_render_phase_runs_then_clear_before_camera_before_sprite() {
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log.clone()).with_viewport(800, 600);
        let mut world = World::new();
        world.insert_resource(RendererRes::new(Box::new(spy)));
        world.insert_resource(ClearColor::default());
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
        schedule.add_systems((clear_system, camera_prepare_system, sprite_render_system).chain());

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
}
