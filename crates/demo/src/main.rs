use axiom2d::prelude::*;

#[derive(Resource, Default)]
struct FrameCount(u64);

fn count_frames(mut count: ResMut<FrameCount>) {
    count.0 += 1;
}

const PLAYER_SPEED: f32 = 300.0;

fn player_control(
    mut query: Query<&mut Transform2D>,
    input: Res<InputState>,
    action_map: Res<ActionMap>,
    dt: Res<DeltaTime>,
) {
    let mut dx = 0.0;
    let mut dy = 0.0;
    if input.action_pressed(&action_map, "move_right") {
        dx += 1.0;
    }
    if input.action_pressed(&action_map, "move_left") {
        dx -= 1.0;
    }
    if input.action_pressed(&action_map, "move_down") {
        dy += 1.0;
    }
    if input.action_pressed(&action_map, "move_up") {
        dy -= 1.0;
    }
    let displacement = PLAYER_SPEED * dt.0.0;
    for mut t in &mut query {
        t.position.x += dx * displacement;
        t.position.y += dy * displacement;
    }
}

fn setup(app: &mut App) {
    let config = WindowConfig {
        title: "Axiom2d Demo",
        ..Default::default()
    };
    let mut action_map = ActionMap::default();
    action_map.bind("move_right", vec![KeyCode::ArrowRight]);
    action_map.bind("move_left", vec![KeyCode::ArrowLeft]);
    action_map.bind("move_up", vec![KeyCode::ArrowUp]);
    action_map.bind("move_down", vec![KeyCode::ArrowDown]);
    app.world_mut().insert_resource(action_map);
    app.world_mut().insert_resource(FrameCount::default());
    app.world_mut().insert_resource(ClearColor::default());
    app.world_mut().insert_resource(InputState::default());
    app.world_mut().insert_resource(InputEventBuffer::default());
    app.world_mut()
        .insert_resource(ClockRes::new(Box::new(SystemClock::new())));
    app.world_mut().spawn((
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
    app.world_mut().spawn(Camera2D {
        position: Vec2::new(640.0, 360.0),
        zoom: 1.0,
    });
    app.set_window_config(config)
        .add_systems(Phase::Input, input_system)
        .add_systems(Phase::PreUpdate, time_system)
        .add_systems(Phase::Update, (count_frames, player_control))
        .add_systems(
            Phase::PostUpdate,
            (
                hierarchy_maintenance_system,
                transform_propagation_system,
                visibility_system,
            )
                .chain(),
        )
        .add_systems(
            Phase::Render,
            (clear_system, camera_prepare_system, sprite_render_system).chain(),
        );
}

fn main() {
    let mut app = App::new();
    setup(&mut app);
    app.run();
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use engine_render::testing::SpyRenderer;

    use super::*;

    fn setup_action_control_world() -> World {
        let mut action_map = ActionMap::default();
        action_map.bind("move_right", vec![KeyCode::ArrowRight]);
        action_map.bind("move_left", vec![KeyCode::ArrowLeft]);
        action_map.bind("move_up", vec![KeyCode::ArrowUp]);
        action_map.bind("move_down", vec![KeyCode::ArrowDown]);
        let mut world = World::new();
        world.insert_resource(action_map);
        world.insert_resource(InputState::default());
        world.insert_resource(DeltaTime(Seconds(1.0)));
        world.spawn(Transform2D {
            position: Vec2::new(400.0, 300.0),
            ..Transform2D::default()
        });
        world
    }

    fn run_player_control(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(player_control);
        schedule.run(world);
    }

    #[test]
    fn when_move_right_action_pressed_then_player_translates_right() {
        // Arrange
        let mut world = setup_action_control_world();
        world
            .resource_mut::<InputState>()
            .press(KeyCode::ArrowRight);

        // Act
        run_player_control(&mut world);

        // Assert
        let mut query = world.query::<&Transform2D>();
        let transform = query.single(&world).unwrap();
        assert_eq!(transform.position.x, 400.0 + PLAYER_SPEED);
        assert_eq!(transform.position.y, 300.0);
    }

    #[test]
    fn when_move_left_action_pressed_then_player_translates_left() {
        // Arrange
        let mut world = setup_action_control_world();
        world.resource_mut::<InputState>().press(KeyCode::ArrowLeft);

        // Act
        run_player_control(&mut world);

        // Assert
        let mut query = world.query::<&Transform2D>();
        let transform = query.single(&world).unwrap();
        assert_eq!(transform.position.x, 400.0 - PLAYER_SPEED);
        assert_eq!(transform.position.y, 300.0);
    }

    #[test]
    fn when_move_up_action_pressed_then_player_translates_up() {
        // Arrange
        let mut world = setup_action_control_world();
        world.resource_mut::<InputState>().press(KeyCode::ArrowUp);

        // Act
        run_player_control(&mut world);

        // Assert
        let mut query = world.query::<&Transform2D>();
        let transform = query.single(&world).unwrap();
        assert_eq!(transform.position.x, 400.0);
        assert_eq!(transform.position.y, 300.0 - PLAYER_SPEED);
    }

    #[test]
    fn when_move_down_action_pressed_then_player_translates_down() {
        // Arrange
        let mut world = setup_action_control_world();
        world.resource_mut::<InputState>().press(KeyCode::ArrowDown);

        // Act
        run_player_control(&mut world);

        // Assert
        let mut query = world.query::<&Transform2D>();
        let transform = query.single(&world).unwrap();
        assert_eq!(transform.position.x, 400.0);
        assert_eq!(transform.position.y, 300.0 + PLAYER_SPEED);
    }

    #[test]
    fn when_no_actions_pressed_then_player_position_unchanged() {
        // Arrange
        let mut world = setup_action_control_world();

        // Act
        run_player_control(&mut world);

        // Assert
        let mut query = world.query::<&Transform2D>();
        let transform = query.single(&world).unwrap();
        assert_eq!(transform.position.x, 400.0);
        assert_eq!(transform.position.y, 300.0);
    }

    #[test]
    fn when_opposite_horizontal_actions_pressed_then_player_x_unchanged() {
        // Arrange
        let mut world = setup_action_control_world();
        world.resource_mut::<InputState>().press(KeyCode::ArrowLeft);
        world
            .resource_mut::<InputState>()
            .press(KeyCode::ArrowRight);

        // Act
        run_player_control(&mut world);

        // Assert
        let mut query = world.query::<&Transform2D>();
        let transform = query.single(&world).unwrap();
        assert_eq!(transform.position.x, 400.0);
    }

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
        assert_eq!(calls[2], "draw_sprite");
    }

    #[test]
    fn when_setup_called_then_player_entity_has_transform2d_and_sprite() {
        // Arrange
        let mut app = App::new();

        // Act
        setup(&mut app);

        // Assert
        let world = app.world_mut();
        let mut query = world.query::<(&Transform2D, &Sprite)>();
        assert_eq!(query.iter(world).count(), 1);
    }

    #[test]
    fn when_setup_called_then_camera2d_entity_exists() {
        // Arrange
        let mut app = App::new();

        // Act
        setup(&mut app);

        // Assert
        let world = app.world_mut();
        let mut query = world.query::<&Camera2D>();
        assert_eq!(query.iter(world).count(), 1);
    }

    #[test]
    fn when_setup_called_then_action_map_has_four_move_bindings() {
        // Arrange
        let mut app = App::new();

        // Act
        setup(&mut app);

        // Assert
        let world = app.world_mut();
        let action_map = world.get_resource::<ActionMap>().unwrap();
        assert!(!action_map.bindings_for("move_right").is_empty());
        assert!(!action_map.bindings_for("move_left").is_empty());
        assert!(!action_map.bindings_for("move_up").is_empty());
        assert!(!action_map.bindings_for("move_down").is_empty());
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
