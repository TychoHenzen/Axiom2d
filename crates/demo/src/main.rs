use axiom2d::prelude::*;

#[derive(Resource, Default)]
struct FrameCount(u64);

#[derive(Component)]
struct Sun;

#[derive(Component)]
struct Moon;

#[derive(Component)]
struct OrbitalSpeed(pub f32);

const SUN_POSITION: Vec2 = Vec2::ZERO;
const SUN_COLOR: Color = Color { r: 1.0, g: 0.85, b: 0.0, a: 1.0 };
const CAMERA_PAN_SPEED: f32 = 300.0;
const CAMERA_ZOOM_SPEED: f32 = 1.0;
const ZOOM_MIN: f32 = 0.1;

fn count_frames(mut count: ResMut<FrameCount>) {
    count.0 += 1;
}

fn camera_pan_system(
    mut query: Query<&mut Camera2D>,
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
    let displacement = CAMERA_PAN_SPEED * dt.0.0;
    for mut camera in &mut query {
        camera.position.x += dx * displacement;
        camera.position.y += dy * displacement;
    }
}

fn camera_zoom_system(
    mut query: Query<&mut Camera2D>,
    input: Res<InputState>,
    action_map: Res<ActionMap>,
    dt: Res<DeltaTime>,
) {
    let mut zoom_dir = 0.0;
    if input.action_pressed(&action_map, "zoom_in") {
        zoom_dir += 1.0;
    }
    if input.action_pressed(&action_map, "zoom_out") {
        zoom_dir -= 1.0;
    }
    let zoom_delta = CAMERA_ZOOM_SPEED * dt.0.0 * zoom_dir;
    for mut camera in &mut query {
        camera.zoom = (camera.zoom + zoom_delta).max(ZOOM_MIN);
    }
}

fn orbit_system(mut query: Query<(&mut Transform2D, &OrbitalSpeed)>, dt: Res<DeltaTime>) {
    for (mut transform, speed) in &mut query {
        transform.rotation += speed.0 * dt.0.0;
    }
}

fn setup(app: &mut App) {
    let config = WindowConfig {
        title: "Axiom2d Solar System",
        ..Default::default()
    };
    let mut action_map = ActionMap::default();
    action_map.bind("move_right", vec![KeyCode::ArrowRight]);
    action_map.bind("move_left", vec![KeyCode::ArrowLeft]);
    action_map.bind("move_up", vec![KeyCode::ArrowUp]);
    action_map.bind("move_down", vec![KeyCode::ArrowDown]);
    action_map.bind("zoom_in", vec![KeyCode::Equal]);
    action_map.bind("zoom_out", vec![KeyCode::Minus]);
    app.world_mut().insert_resource(action_map);
    app.world_mut().insert_resource(FrameCount::default());
    app.world_mut()
        .insert_resource(ClearColor(Color::BLACK));
    app.world_mut().insert_resource(InputState::default());
    app.world_mut().insert_resource(InputEventBuffer::default());
    app.world_mut()
        .insert_resource(ClockRes::new(Box::new(SystemClock::new())));

    // Sun
    app.world_mut().spawn((
        Transform2D {
            position: SUN_POSITION,
            ..Transform2D::default()
        },
        Sprite {
            texture: TextureId(0),
            uv_rect: [0.0, 0.0, 1.0, 1.0],
            color: SUN_COLOR,
            width: Pixels(80.0),
            height: Pixels(80.0),
        },
        Sun,
        RenderLayer::World,
    ));

    // Planets: (orbit_radius, speed_rad_per_sec, color, size, moon)
    // moon: Option<(moon_orbit_radius, moon_speed, moon_color, moon_size)>
    let planets: [(f32, f32, Color, f32, Option<(f32, f32, Color, f32)>); 4] = [
        (120.0, 1.5, Color::from_u8(180, 120, 60, 255), 20.0, None),
        (200.0, 1.0, Color::from_u8(60, 130, 200, 255), 30.0,
            Some((40.0, 3.0, Color::from_u8(200, 200, 200, 255), 8.0))),
        (300.0, 0.6, Color::from_u8(50, 180, 80, 255), 35.0,
            Some((50.0, 2.0, Color::from_u8(180, 160, 140, 255), 10.0))),
        (420.0, 0.35, Color::RED, 25.0, None),
    ];
    for (radius, speed, color, size, moon) in planets {
        let pivot = app
            .world_mut()
            .spawn((
                Transform2D {
                    position: SUN_POSITION,
                    ..Transform2D::default()
                },
                OrbitalSpeed(speed),
            ))
            .id();
        let planet = app.world_mut().spawn_child(pivot, (
            Transform2D {
                position: Vec2::new(radius, 0.0),
                ..Transform2D::default()
            },
            Sprite {
                texture: TextureId(0),
                uv_rect: [0.0, 0.0, 1.0, 1.0],
                color,
                width: Pixels(size),
                height: Pixels(size),
            },
            RenderLayer::World,
        ));
        if let Some((moon_radius, moon_speed, moon_color, moon_size)) = moon {
            let moon_pivot = app.world_mut().spawn_child(planet, (
                Transform2D::default(),
                OrbitalSpeed(moon_speed),
            ));
            app.world_mut().spawn_child(moon_pivot, (
                Transform2D {
                    position: Vec2::new(moon_radius, 0.0),
                    ..Transform2D::default()
                },
                Sprite {
                    texture: TextureId(0),
                    uv_rect: [0.0, 0.0, 1.0, 1.0],
                    color: moon_color,
                    width: Pixels(moon_size),
                    height: Pixels(moon_size),
                },
                Moon,
                RenderLayer::World,
            ));
        }
    }

    // Camera
    app.world_mut().spawn(Camera2D {
        position: SUN_POSITION,
        zoom: 0.5,
    });

    app.set_window_config(config)
        .add_systems(Phase::Input, input_system)
        .add_systems(Phase::PreUpdate, time_system)
        .add_systems(Phase::Update, (count_frames, orbit_system, camera_pan_system, camera_zoom_system))
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

    const PLANET_COUNT: usize = 4;
    const MOON_COUNT: usize = 2;

    fn setup_camera_pan_world() -> World {
        let mut action_map = ActionMap::default();
        action_map.bind("move_right", vec![KeyCode::ArrowRight]);
        action_map.bind("move_left", vec![KeyCode::ArrowLeft]);
        action_map.bind("move_up", vec![KeyCode::ArrowUp]);
        action_map.bind("move_down", vec![KeyCode::ArrowDown]);
        let mut world = World::new();
        world.insert_resource(action_map);
        world.insert_resource(InputState::default());
        world.insert_resource(DeltaTime(Seconds(1.0)));
        world.spawn(Camera2D {
            position: Vec2::new(400.0, 300.0),
            zoom: 1.0,
        });
        world
    }

    fn run_camera_pan(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(camera_pan_system);
        schedule.run(world);
    }

    #[test]
    fn when_pan_right_then_camera_moves_right() {
        // Arrange
        let mut world = setup_camera_pan_world();
        world
            .resource_mut::<InputState>()
            .press(KeyCode::ArrowRight);

        // Act
        run_camera_pan(&mut world);

        // Assert
        let mut query = world.query::<&Camera2D>();
        let camera = query.single(&world).unwrap();
        assert_eq!(camera.position.x, 400.0 + CAMERA_PAN_SPEED);
        assert_eq!(camera.position.y, 300.0);
    }

    #[test]
    fn when_pan_left_then_camera_moves_left() {
        // Arrange
        let mut world = setup_camera_pan_world();
        world.resource_mut::<InputState>().press(KeyCode::ArrowLeft);

        // Act
        run_camera_pan(&mut world);

        // Assert
        let mut query = world.query::<&Camera2D>();
        let camera = query.single(&world).unwrap();
        assert_eq!(camera.position.x, 400.0 - CAMERA_PAN_SPEED);
        assert_eq!(camera.position.y, 300.0);
    }

    #[test]
    fn when_pan_up_then_camera_moves_up() {
        // Arrange
        let mut world = setup_camera_pan_world();
        world.resource_mut::<InputState>().press(KeyCode::ArrowUp);

        // Act
        run_camera_pan(&mut world);

        // Assert
        let mut query = world.query::<&Camera2D>();
        let camera = query.single(&world).unwrap();
        assert_eq!(camera.position.x, 400.0);
        assert_eq!(camera.position.y, 300.0 - CAMERA_PAN_SPEED);
    }

    #[test]
    fn when_pan_down_then_camera_moves_down() {
        // Arrange
        let mut world = setup_camera_pan_world();
        world.resource_mut::<InputState>().press(KeyCode::ArrowDown);

        // Act
        run_camera_pan(&mut world);

        // Assert
        let mut query = world.query::<&Camera2D>();
        let camera = query.single(&world).unwrap();
        assert_eq!(camera.position.x, 400.0);
        assert_eq!(camera.position.y, 300.0 + CAMERA_PAN_SPEED);
    }

    #[test]
    fn when_no_pan_input_then_camera_position_unchanged() {
        // Arrange
        let mut world = setup_camera_pan_world();

        // Act
        run_camera_pan(&mut world);

        // Assert
        let mut query = world.query::<&Camera2D>();
        let camera = query.single(&world).unwrap();
        assert_eq!(camera.position.x, 400.0);
        assert_eq!(camera.position.y, 300.0);
    }

    #[test]
    fn when_opposite_pan_directions_then_camera_x_unchanged() {
        // Arrange
        let mut world = setup_camera_pan_world();
        world.resource_mut::<InputState>().press(KeyCode::ArrowLeft);
        world
            .resource_mut::<InputState>()
            .press(KeyCode::ArrowRight);

        // Act
        run_camera_pan(&mut world);

        // Assert
        let mut query = world.query::<&Camera2D>();
        let camera = query.single(&world).unwrap();
        assert_eq!(camera.position.x, 400.0);
    }

    fn setup_zoom_world(initial_zoom: f32) -> World {
        let mut action_map = ActionMap::default();
        action_map.bind("zoom_in", vec![KeyCode::Equal]);
        action_map.bind("zoom_out", vec![KeyCode::Minus]);
        let mut world = World::new();
        world.insert_resource(action_map);
        world.insert_resource(InputState::default());
        world.insert_resource(DeltaTime(Seconds(1.0)));
        world.spawn(Camera2D {
            position: Vec2::ZERO,
            zoom: initial_zoom,
        });
        world
    }

    fn run_camera_zoom(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(camera_zoom_system);
        schedule.run(world);
    }

    #[test]
    fn when_zoom_in_pressed_then_camera_zoom_increases() {
        // Arrange
        let mut world = setup_zoom_world(1.0);
        world.resource_mut::<InputState>().press(KeyCode::Equal);

        // Act
        run_camera_zoom(&mut world);

        // Assert
        let mut query = world.query::<&Camera2D>();
        let camera = query.single(&world).unwrap();
        assert!(camera.zoom > 1.0);
    }

    #[test]
    fn when_zoom_out_pressed_then_camera_zoom_decreases() {
        // Arrange
        let mut world = setup_zoom_world(1.0);
        world.resource_mut::<InputState>().press(KeyCode::Minus);

        // Act
        run_camera_zoom(&mut world);

        // Assert
        let mut query = world.query::<&Camera2D>();
        let camera = query.single(&world).unwrap();
        assert!(camera.zoom < 1.0);
    }

    #[test]
    fn when_zoom_out_at_minimum_then_zoom_does_not_go_below_floor() {
        // Arrange
        let mut world = setup_zoom_world(ZOOM_MIN);
        world.resource_mut::<InputState>().press(KeyCode::Minus);

        // Act
        run_camera_zoom(&mut world);

        // Assert
        let mut query = world.query::<&Camera2D>();
        let camera = query.single(&world).unwrap();
        assert!(camera.zoom >= ZOOM_MIN);
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
    fn when_setup_called_then_all_sprite_entities_exist() {
        // Arrange
        let mut app = App::new();

        // Act
        setup(&mut app);

        // Assert — 1 sun + 4 planets + 2 moons
        let world = app.world_mut();
        let mut query = world.query::<(&Transform2D, &Sprite)>();
        assert_eq!(query.iter(world).count(), 1 + PLANET_COUNT + MOON_COUNT);
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
    fn when_orbit_system_runs_then_rotation_increments_by_speed_times_delta() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(DeltaTime(Seconds(0.5)));
        world.spawn((Transform2D::default(), OrbitalSpeed(2.0)));
        let mut schedule = Schedule::default();
        schedule.add_systems(orbit_system);

        // Act
        schedule.run(&mut world);

        // Assert
        let mut query = world.query::<&Transform2D>();
        let transform = query.single(&world).unwrap();
        assert_eq!(transform.rotation, 1.0);
    }

    #[test]
    fn when_orbit_system_runs_twice_then_rotation_accumulates() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(DeltaTime(Seconds(0.25)));
        world.spawn((Transform2D::default(), OrbitalSpeed(1.0)));
        let mut schedule = Schedule::default();
        schedule.add_systems(orbit_system);

        // Act
        schedule.run(&mut world);
        schedule.run(&mut world);

        // Assert
        let mut query = world.query::<&Transform2D>();
        let transform = query.single(&world).unwrap();
        assert_eq!(transform.rotation, 0.5);
    }

    #[test]
    fn when_delta_time_is_zero_then_rotation_unchanged() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(DeltaTime(Seconds(0.0)));
        world.spawn((
            Transform2D { rotation: 1.5, ..Transform2D::default() },
            OrbitalSpeed(5.0),
        ));
        let mut schedule = Schedule::default();
        schedule.add_systems(orbit_system);

        // Act
        schedule.run(&mut world);

        // Assert
        let mut query = world.query::<&Transform2D>();
        let transform = query.single(&world).unwrap();
        assert_eq!(transform.rotation, 1.5);
    }

    #[test]
    fn when_entity_has_no_orbital_speed_then_rotation_unchanged() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(DeltaTime(Seconds(1.0)));
        let unaffected = world.spawn(Transform2D::default()).id();
        world.spawn((Transform2D::default(), OrbitalSpeed(1.0)));
        let mut schedule = Schedule::default();
        schedule.add_systems(orbit_system);

        // Act
        schedule.run(&mut world);

        // Assert
        let transform = world.get::<Transform2D>(unaffected).unwrap();
        assert_eq!(transform.rotation, 0.0);
    }

    #[test]
    fn when_multiple_pivots_then_each_rotates_at_own_speed() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(DeltaTime(Seconds(1.0)));
        let a = world.spawn((Transform2D::default(), OrbitalSpeed(1.0))).id();
        let b = world.spawn((Transform2D::default(), OrbitalSpeed(3.0))).id();
        let mut schedule = Schedule::default();
        schedule.add_systems(orbit_system);

        // Act
        schedule.run(&mut world);

        // Assert
        assert_eq!(world.get::<Transform2D>(a).unwrap().rotation, 1.0);
        assert_eq!(world.get::<Transform2D>(b).unwrap().rotation, 3.0);
    }

    #[test]
    fn when_setup_called_then_action_map_has_all_bindings() {
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
        assert!(!action_map.bindings_for("zoom_in").is_empty());
        assert!(!action_map.bindings_for("zoom_out").is_empty());
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

    #[test]
    fn when_setup_called_then_exactly_one_sun_entity_exists() {
        // Arrange
        let mut app = App::new();

        // Act
        setup(&mut app);

        // Assert
        let world = app.world_mut();
        let mut query = world.query::<&Sun>();
        assert_eq!(query.iter(world).count(), 1);
    }

    #[test]
    fn when_setup_called_then_sun_has_yellow_color() {
        // Arrange
        let mut app = App::new();
        setup(&mut app);

        // Act
        let world = app.world_mut();
        let mut query = world.query::<(&Sun, &Sprite)>();
        let (_, sprite) = query.single(world).unwrap();

        // Assert
        assert_eq!(sprite.color, SUN_COLOR);
    }

    #[test]
    fn when_setup_called_then_correct_number_of_planet_pivots() {
        // Arrange
        let mut app = App::new();

        // Act
        setup(&mut app);

        // Assert — planet pivots have OrbitalSpeed but no ChildOf (they're roots)
        let world = app.world_mut();
        let mut query = world.query_filtered::<&OrbitalSpeed, Without<ChildOf>>();
        assert_eq!(query.iter(world).count(), PLANET_COUNT);
    }

    #[test]
    fn when_setup_called_then_each_pivot_has_one_planet_child() {
        // Arrange
        let mut app = App::new();
        setup(&mut app);
        let world = app.world_mut();
        let mut schedule = Schedule::default();
        schedule.add_systems(hierarchy_maintenance_system);
        schedule.run(world);

        // Act / Assert — planet pivots are root OrbitalSpeed entities (no ChildOf)
        let mut pivot_query = world.query_filtered::<(Entity, &OrbitalSpeed), Without<ChildOf>>();
        let pivots: Vec<Entity> = pivot_query.iter(world).map(|(e, _)| e).collect();
        assert_eq!(pivots.len(), PLANET_COUNT);
        for pivot in pivots {
            let children = world.get::<Children>(pivot).unwrap();
            assert_eq!(children.0.len(), 1);
            let child = children.0[0];
            assert!(world.get::<Sprite>(child).is_some());
        }
    }

    #[test]
    fn when_setup_called_then_each_planet_has_distinct_color() {
        // Arrange
        let mut app = App::new();
        setup(&mut app);
        let world = app.world_mut();
        let mut schedule = Schedule::default();
        schedule.add_systems(hierarchy_maintenance_system);
        schedule.run(world);

        // Act
        let mut pivot_query = world.query_filtered::<(Entity, &OrbitalSpeed), Without<ChildOf>>();
        let pivots: Vec<Entity> = pivot_query.iter(world).map(|(e, _)| e).collect();
        let mut colors = Vec::new();
        for pivot in pivots {
            let children = world.get::<Children>(pivot).unwrap();
            let sprite = world.get::<Sprite>(children.0[0]).unwrap();
            colors.push(sprite.color);
        }

        // Assert
        let unique: std::collections::HashSet<u32> = colors
            .iter()
            .map(|c| {
                let r = (c.r * 255.0) as u32;
                let g = (c.g * 255.0) as u32;
                let b = (c.b * 255.0) as u32;
                (r << 16) | (g << 8) | b
            })
            .collect();
        assert_eq!(unique.len(), PLANET_COUNT);
    }

    #[test]
    fn when_setup_called_then_all_planets_on_world_render_layer() {
        // Arrange
        let mut app = App::new();
        setup(&mut app);
        let world = app.world_mut();
        let mut schedule = Schedule::default();
        schedule.add_systems(hierarchy_maintenance_system);
        schedule.run(world);

        // Act / Assert
        let mut pivot_query = world.query_filtered::<(Entity, &OrbitalSpeed), Without<ChildOf>>();
        let pivots: Vec<Entity> = pivot_query.iter(world).map(|(e, _)| e).collect();
        for pivot in pivots {
            let children = world.get::<Children>(pivot).unwrap();
            let layer = world.get::<RenderLayer>(children.0[0]).unwrap();
            assert_eq!(*layer, RenderLayer::World);
        }
    }

    #[test]
    fn when_setup_called_then_camera_centered_on_sun() {
        // Arrange
        let mut app = App::new();
        setup(&mut app);

        // Act
        let world = app.world_mut();
        let mut query = world.query::<&Camera2D>();
        let camera = query.single(world).unwrap();

        // Assert
        assert_eq!(camera.position, SUN_POSITION);
    }

    #[test]
    fn when_orbit_and_propagation_run_then_planet_position_on_circle() {
        // Arrange
        let mut world = World::new();
        let orbit_radius = 100.0;
        world.insert_resource(DeltaTime(Seconds(std::f32::consts::FRAC_PI_2)));
        let pivot = world
            .spawn((
                Transform2D::default(),
                OrbitalSpeed(1.0),
            ))
            .id();
        world.spawn_child(pivot, Transform2D {
            position: Vec2::new(orbit_radius, 0.0),
            ..Transform2D::default()
        });
        let mut schedule = Schedule::default();
        schedule.add_systems(
            (
                orbit_system,
                hierarchy_maintenance_system,
                transform_propagation_system,
            )
                .chain(),
        );

        // Act
        schedule.run(&mut world);

        // Assert
        let mut query = world.query::<(&GlobalTransform2D, &ChildOf)>();
        let (global, _) = query.single(&world).unwrap();
        let pos = global.0.translation;
        assert!(
            (pos - Vec2::new(0.0, orbit_radius)).length() < 1e-4,
            "expected planet near (0, {orbit_radius}), got ({}, {})",
            pos.x,
            pos.y
        );
    }

    #[test]
    fn when_setup_called_then_moons_exist_with_moon_marker() {
        // Arrange
        let mut app = App::new();

        // Act
        setup(&mut app);

        // Assert
        let world = app.world_mut();
        let mut query = world.query::<&Moon>();
        assert!(query.iter(world).count() >= 1, "expected at least one moon");
    }

    #[test]
    fn when_setup_called_then_moons_are_grandchildren_of_orbit_pivots() {
        // Arrange
        let mut app = App::new();
        setup(&mut app);
        let world = app.world_mut();
        let mut schedule = Schedule::default();
        schedule.add_systems(hierarchy_maintenance_system);
        schedule.run(world);
        schedule.run(world);

        // Act — each moon should have a ChildOf pointing to a moon pivot,
        // which itself has a ChildOf pointing to a planet entity
        let mut moon_query = world.query::<(Entity, &Moon, &ChildOf)>();
        let moons: Vec<(Entity, Entity)> = moon_query
            .iter(world)
            .map(|(e, _, parent)| (e, parent.0))
            .collect();

        // Assert
        assert!(!moons.is_empty());
        for (_moon, moon_pivot) in &moons {
            let pivot_parent = world.get::<ChildOf>(*moon_pivot);
            assert!(pivot_parent.is_some(), "moon pivot should be a child of the planet");
        }
    }

    #[test]
    fn when_setup_called_then_moon_pivots_have_orbital_speed() {
        // Arrange
        let mut app = App::new();
        setup(&mut app);
        let world = app.world_mut();

        // Act — find all Moon entities, check their parents have OrbitalSpeed
        let mut moon_query = world.query::<(&Moon, &ChildOf)>();
        let moon_pivots: Vec<Entity> = moon_query
            .iter(world)
            .map(|(_, parent)| parent.0)
            .collect();

        // Assert
        assert!(!moon_pivots.is_empty());
        for pivot in moon_pivots {
            assert!(
                world.get::<OrbitalSpeed>(pivot).is_some(),
                "moon pivot should have OrbitalSpeed"
            );
        }
    }
}
