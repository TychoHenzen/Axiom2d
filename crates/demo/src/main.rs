use axiom2d::prelude::*;

#[derive(Resource)]
struct RectSize {
    width: Pixels,
    height: Pixels,
}

#[derive(Resource, Default)]
struct FrameCount(u64);

fn count_frames(mut count: ResMut<FrameCount>) {
    count.0 += 1;
}

fn render_rect(
    mut renderer: ResMut<RendererRes>,
    query: Query<&Position>,
    rect_size: Res<RectSize>,
) {
    for pos in &query {
        renderer.draw_rect(Rect {
            x: pos.x,
            y: pos.y,
            width: rect_size.width,
            height: rect_size.height,
            color: Color::WHITE,
        });
    }
}

const PLAYER_SPEED: f32 = 300.0;

fn player_control(mut query: Query<&mut Position>, input: Res<InputState>, dt: Res<DeltaTime>) {
    let mut dx = 0.0;
    let mut dy = 0.0;

    if input.pressed(KeyCode::ArrowRight) {
        dx += 1.0;
    }
    if input.pressed(KeyCode::ArrowLeft) {
        dx -= 1.0;
    }
    if input.pressed(KeyCode::ArrowDown) {
        dy += 1.0;
    }
    if input.pressed(KeyCode::ArrowUp) {
        dy -= 1.0;
    }

    let displacement = PLAYER_SPEED * dt.0.0;
    for mut pos in &mut query {
        pos.x = pos.x + Pixels(dx * displacement);
        pos.y = pos.y + Pixels(dy * displacement);
    }
}

fn setup(app: &mut App) {
    let config = WindowConfig {
        title: "Axiom2d Demo",
        ..Default::default()
    };
    app.world_mut().insert_resource(FrameCount::default());
    app.world_mut().insert_resource(ClearColor::default());
    app.world_mut().insert_resource(InputState::default());
    app.world_mut().insert_resource(InputEventBuffer::default());
    app.world_mut()
        .insert_resource(ClockRes::new(Box::new(SystemClock::new())));
    app.world_mut().insert_resource(RectSize {
        width: Pixels(300.0),
        height: Pixels(200.0),
    });
    app.world_mut().spawn(Position {
        x: Pixels(490.0),
        y: Pixels(260.0),
    });
    app.set_window_config(config)
        .add_systems(Phase::Input, input_system)
        .add_systems(Phase::PreUpdate, time_system)
        .add_systems(Phase::Update, (count_frames, player_control))
        .add_systems(Phase::Render, (clear_system, render_rect).chain());
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

    fn setup_control_world() -> World {
        let mut world = World::new();
        world.insert_resource(InputState::default());
        world.insert_resource(DeltaTime(Seconds(1.0)));
        world.spawn(Position {
            x: Pixels(400.0),
            y: Pixels(300.0),
        });
        world
    }

    #[test]
    fn when_right_held_then_position_moves_right() {
        // Arrange
        let mut world = setup_control_world();
        world
            .resource_mut::<InputState>()
            .press(KeyCode::ArrowRight);
        let mut schedule = Schedule::default();
        schedule.add_systems(player_control);

        // Act
        schedule.run(&mut world);

        // Assert
        let mut query = world.query::<&Position>();
        let pos = query.single(&world).unwrap();
        assert_eq!(pos.x, Pixels(400.0 + PLAYER_SPEED));
    }

    #[test]
    fn when_left_held_then_position_moves_left() {
        // Arrange
        let mut world = setup_control_world();
        world.resource_mut::<InputState>().press(KeyCode::ArrowLeft);
        let mut schedule = Schedule::default();
        schedule.add_systems(player_control);

        // Act
        schedule.run(&mut world);

        // Assert
        let mut query = world.query::<&Position>();
        let pos = query.single(&world).unwrap();
        assert_eq!(pos.x, Pixels(400.0 - PLAYER_SPEED));
    }

    #[test]
    fn when_up_held_then_position_moves_up() {
        // Arrange
        let mut world = setup_control_world();
        world.resource_mut::<InputState>().press(KeyCode::ArrowUp);
        let mut schedule = Schedule::default();
        schedule.add_systems(player_control);

        // Act
        schedule.run(&mut world);

        // Assert
        let mut query = world.query::<&Position>();
        let pos = query.single(&world).unwrap();
        assert_eq!(pos.y, Pixels(300.0 - PLAYER_SPEED));
    }

    #[test]
    fn when_down_held_then_position_moves_down() {
        // Arrange
        let mut world = setup_control_world();
        world.resource_mut::<InputState>().press(KeyCode::ArrowDown);
        let mut schedule = Schedule::default();
        schedule.add_systems(player_control);

        // Act
        schedule.run(&mut world);

        // Assert
        let mut query = world.query::<&Position>();
        let pos = query.single(&world).unwrap();
        assert_eq!(pos.y, Pixels(300.0 + PLAYER_SPEED));
    }

    #[test]
    fn when_no_keys_held_then_position_unchanged() {
        // Arrange
        let mut world = setup_control_world();
        let mut schedule = Schedule::default();
        schedule.add_systems(player_control);

        // Act
        schedule.run(&mut world);

        // Assert
        let mut query = world.query::<&Position>();
        let pos = query.single(&world).unwrap();
        assert_eq!(pos.x, Pixels(400.0));
        assert_eq!(pos.y, Pixels(300.0));
    }

    #[test]
    fn when_opposite_keys_held_then_position_unchanged() {
        // Arrange
        let mut world = setup_control_world();
        world.resource_mut::<InputState>().press(KeyCode::ArrowLeft);
        world
            .resource_mut::<InputState>()
            .press(KeyCode::ArrowRight);
        let mut schedule = Schedule::default();
        schedule.add_systems(player_control);

        // Act
        schedule.run(&mut world);

        // Assert
        let mut query = world.query::<&Position>();
        let pos = query.single(&world).unwrap();
        assert_eq!(pos.x, Pixels(400.0));
    }

    #[test]
    fn when_render_rect_system_runs_then_draw_rect_called_with_position() {
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log.clone());
        let mut world = World::new();
        world.insert_resource(RendererRes::new(Box::new(spy)));
        world.insert_resource(RectSize {
            width: Pixels(300.0),
            height: Pixels(200.0),
        });
        world.spawn(Position {
            x: Pixels(100.0),
            y: Pixels(50.0),
        });
        let mut schedule = Schedule::default();
        schedule.add_systems(render_rect);

        // Act
        schedule.run(&mut world);

        // Assert
        assert!(log.lock().unwrap().contains(&"draw_rect".to_string()));
    }

    #[test]
    fn when_demo_setup_called_then_world_contains_expected_resources_and_entity() {
        // Arrange
        let mut app = App::new();

        // Act
        setup(&mut app);

        // Assert
        let world = app.world_mut();
        assert!(world.get_resource::<WindowSize>().is_some());
        assert!(world.get_resource::<ClearColor>().is_some());
        assert!(world.get_resource::<RectSize>().is_some());
        assert!(world.get_resource::<InputState>().is_some());
        let mut query = world.query::<&Position>();
        assert_eq!(query.iter(world).count(), 1);
    }

    #[test]
    fn when_render_phase_runs_then_clear_happens_before_draw_rect() {
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log.clone());
        let mut world = World::new();
        world.insert_resource(RendererRes::new(Box::new(spy)));
        world.insert_resource(ClearColor::default());
        world.insert_resource(RectSize {
            width: Pixels(300.0),
            height: Pixels(200.0),
        });
        world.spawn(Position {
            x: Pixels(100.0),
            y: Pixels(50.0),
        });
        let mut schedule = Schedule::default();
        schedule.add_systems((clear_system, render_rect).chain());

        // Act
        schedule.run(&mut world);

        // Assert
        let calls = log.lock().unwrap();
        assert_eq!(calls[0], "clear");
        assert_eq!(calls[1], "draw_rect");
    }
}
