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

fn bounce_rect(
    mut query: Query<(&Position, &mut Velocity)>,
    bounds: Res<WindowSize>,
    rect_size: Res<RectSize>,
) {
    for (pos, mut vel) in &mut query {
        if pos.x + rect_size.width >= bounds.width || pos.x <= Pixels(0.0) {
            vel.dx = Pixels(-vel.dx.0);
        }
        if pos.y + rect_size.height >= bounds.height || pos.y <= Pixels(0.0) {
            vel.dy = Pixels(-vel.dy.0);
        }
    }
}

fn move_rect(mut query: Query<(&mut Position, &Velocity)>) {
    for (mut pos, vel) in &mut query {
        pos.x = pos.x + vel.dx;
        pos.y = pos.y + vel.dy;
    }
}

fn setup(app: &mut App) {
    let config = WindowConfig {
        title: "Axiom2d Demo",
        ..Default::default()
    };
    app.world_mut().insert_resource(FrameCount::default());
    app.world_mut().insert_resource(ClearColor::default());
    app.world_mut().insert_resource(RectSize {
        width: Pixels(300.0),
        height: Pixels(200.0),
    });
    app.world_mut().spawn((
        Position { x: Pixels(490.0), y: Pixels(260.0) },
        Velocity { dx: Pixels(4.0), dy: Pixels(3.0) },
    ));
    app.set_window_config(config)
        .add_systems(Phase::Update, (count_frames, move_rect, bounce_rect))
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

    fn spawn_entity(world: &mut World, pos: Position, vel: Velocity) {
        world.spawn((pos, vel));
        world.insert_resource(WindowSize { width: Pixels(1280.0), height: Pixels(720.0) });
        world.insert_resource(RectSize { width: Pixels(300.0), height: Pixels(200.0) });
    }

    #[test]
    fn when_move_system_runs_then_position_advances_by_velocity() {
        // Arrange
        let mut world = World::new();
        world.spawn((
            Position { x: Pixels(100.0), y: Pixels(50.0) },
            Velocity { dx: Pixels(5.0), dy: Pixels(-3.0) },
        ));
        let mut schedule = Schedule::default();
        schedule.add_systems(move_rect);

        // Act
        schedule.run(&mut world);

        // Assert
        let mut query = world.query::<&Position>();
        let pos = query.single(&world).unwrap();
        assert_eq!(pos.x, Pixels(105.0));
        assert_eq!(pos.y, Pixels(47.0));
    }

    #[test]
    fn when_position_reaches_right_edge_then_velocity_dx_reverses() {
        // Arrange
        let mut world = World::new();
        spawn_entity(
            &mut world,
            Position { x: Pixels(990.0), y: Pixels(100.0) },
            Velocity { dx: Pixels(10.0), dy: Pixels(0.0) },
        );
        let mut schedule = Schedule::default();
        schedule.add_systems(bounce_rect);

        // Act
        schedule.run(&mut world);

        // Assert
        let mut query = world.query::<&Velocity>();
        let vel = query.single(&world).unwrap();
        assert_eq!(vel.dx, Pixels(-10.0));
    }

    #[test]
    fn when_position_reaches_left_edge_then_velocity_dx_reverses() {
        // Arrange
        let mut world = World::new();
        spawn_entity(
            &mut world,
            Position { x: Pixels(-5.0), y: Pixels(100.0) },
            Velocity { dx: Pixels(-10.0), dy: Pixels(0.0) },
        );
        let mut schedule = Schedule::default();
        schedule.add_systems(bounce_rect);

        // Act
        schedule.run(&mut world);

        // Assert
        let mut query = world.query::<&Velocity>();
        let vel = query.single(&world).unwrap();
        assert_eq!(vel.dx, Pixels(10.0));
    }

    #[test]
    fn when_position_reaches_bottom_edge_then_velocity_dy_reverses() {
        // Arrange
        let mut world = World::new();
        spawn_entity(
            &mut world,
            Position { x: Pixels(100.0), y: Pixels(525.0) },
            Velocity { dx: Pixels(0.0), dy: Pixels(10.0) },
        );
        let mut schedule = Schedule::default();
        schedule.add_systems(bounce_rect);

        // Act
        schedule.run(&mut world);

        // Assert
        let mut query = world.query::<&Velocity>();
        let vel = query.single(&world).unwrap();
        assert_eq!(vel.dy, Pixels(-10.0));
    }

    #[test]
    fn when_position_reaches_top_edge_then_velocity_dy_reverses() {
        // Arrange
        let mut world = World::new();
        spawn_entity(
            &mut world,
            Position { x: Pixels(100.0), y: Pixels(-5.0) },
            Velocity { dx: Pixels(0.0), dy: Pixels(-10.0) },
        );
        let mut schedule = Schedule::default();
        schedule.add_systems(bounce_rect);

        // Act
        schedule.run(&mut world);

        // Assert
        let mut query = world.query::<&Velocity>();
        let vel = query.single(&world).unwrap();
        assert_eq!(vel.dy, Pixels(10.0));
    }

    #[test]
    fn when_position_is_mid_screen_then_velocity_unchanged() {
        // Arrange
        let mut world = World::new();
        spawn_entity(
            &mut world,
            Position { x: Pixels(400.0), y: Pixels(300.0) },
            Velocity { dx: Pixels(5.0), dy: Pixels(3.0) },
        );
        let mut schedule = Schedule::default();
        schedule.add_systems(bounce_rect);

        // Act
        schedule.run(&mut world);

        // Assert
        let mut query = world.query::<&Velocity>();
        let vel = query.single(&world).unwrap();
        assert_eq!(vel.dx, Pixels(5.0));
        assert_eq!(vel.dy, Pixels(3.0));
    }

    #[test]
    fn when_render_rect_system_runs_then_draw_rect_called_with_position() {
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log.clone());
        let mut world = World::new();
        world.insert_resource(RendererRes::new(Box::new(spy)));
        world.insert_resource(RectSize { width: Pixels(300.0), height: Pixels(200.0) });
        world.spawn(Position { x: Pixels(100.0), y: Pixels(50.0) });
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
        let mut query = world.query::<(&Position, &Velocity)>();
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

    #[test]
    fn when_demo_runs_one_frame_then_position_changes() {
        // Arrange
        let mut world = World::new();
        spawn_entity(
            &mut world,
            Position { x: Pixels(490.0), y: Pixels(260.0) },
            Velocity { dx: Pixels(4.0), dy: Pixels(3.0) },
        );
        let mut schedule = Schedule::default();
        schedule.add_systems((move_rect, bounce_rect));

        // Act
        schedule.run(&mut world);

        // Assert
        let mut query = world.query::<&Position>();
        let pos = query.single(&world).unwrap();
        assert_ne!(pos.x, Pixels(490.0));
    }
}
