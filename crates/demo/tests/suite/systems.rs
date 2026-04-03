#![allow(clippy::float_cmp)]

use axiom2d::prelude::*;
use demo::setup;
use demo::systems::{camera_pan_system, camera_zoom_system, orbit_system, synodic_camera_system};
use demo::types::{CAMERA_PAN_SPEED, Earth, Moon, OrbitalSpeed, Sun, ZOOM_MIN, action};

fn setup_camera_world(bindings: &[(&str, Vec<KeyCode>)], camera: Camera2D) -> World {
    let mut action_map = ActionMap::default();
    for (name, keys) in bindings {
        action_map.bind(name, keys.clone());
    }
    let mut world = World::new();
    world.insert_resource(action_map);
    world.insert_resource(InputState::default());
    world.insert_resource(DeltaTime(Seconds(1.0)));
    world.spawn(camera);
    world
}

fn pan_bindings() -> Vec<(&'static str, Vec<KeyCode>)> {
    vec![
        (action::MOVE_RIGHT, vec![KeyCode::ArrowRight]),
        (action::MOVE_LEFT, vec![KeyCode::ArrowLeft]),
        (action::MOVE_UP, vec![KeyCode::ArrowUp]),
        (action::MOVE_DOWN, vec![KeyCode::ArrowDown]),
    ]
}

fn zoom_bindings() -> Vec<(&'static str, Vec<KeyCode>)> {
    vec![
        (action::ZOOM_IN, vec![KeyCode::Equal]),
        (action::ZOOM_OUT, vec![KeyCode::Minus]),
    ]
}

fn run_camera_pan(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(camera_pan_system);
    schedule.run(world);
}

fn run_camera_zoom(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(camera_zoom_system);
    schedule.run(world);
}

#[test]
fn when_pan_right_then_camera_moves_right() {
    // Arrange
    let camera = Camera2D {
        position: Vec2::new(400.0, 300.0),
        zoom: 1.0,
    };
    let mut world = setup_camera_world(&pan_bindings(), camera);
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
    let camera = Camera2D {
        position: Vec2::new(400.0, 300.0),
        zoom: 1.0,
    };
    let mut world = setup_camera_world(&pan_bindings(), camera);
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
    let camera = Camera2D {
        position: Vec2::new(400.0, 300.0),
        zoom: 1.0,
    };
    let mut world = setup_camera_world(&pan_bindings(), camera);
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
    let camera = Camera2D {
        position: Vec2::new(400.0, 300.0),
        zoom: 1.0,
    };
    let mut world = setup_camera_world(&pan_bindings(), camera);
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
    let camera = Camera2D {
        position: Vec2::new(400.0, 300.0),
        zoom: 1.0,
    };
    let mut world = setup_camera_world(&pan_bindings(), camera);

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
    let camera = Camera2D {
        position: Vec2::new(400.0, 300.0),
        zoom: 1.0,
    };
    let mut world = setup_camera_world(&pan_bindings(), camera);
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

#[test]
fn when_zoom_in_pressed_then_camera_zoom_increases() {
    // Arrange
    let camera = Camera2D {
        position: Vec2::ZERO,
        zoom: 1.0,
    };
    let mut world = setup_camera_world(&zoom_bindings(), camera);
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
    let camera = Camera2D {
        position: Vec2::ZERO,
        zoom: 1.0,
    };
    let mut world = setup_camera_world(&zoom_bindings(), camera);
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
    let camera = Camera2D {
        position: Vec2::ZERO,
        zoom: ZOOM_MIN,
    };
    let mut world = setup_camera_world(&zoom_bindings(), camera);
    world.resource_mut::<InputState>().press(KeyCode::Minus);

    // Act
    run_camera_zoom(&mut world);

    // Assert
    let mut query = world.query::<&Camera2D>();
    let camera = query.single(&world).unwrap();
    assert!(camera.zoom >= ZOOM_MIN);
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
        Transform2D {
            rotation: 1.5,
            ..Transform2D::default()
        },
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
    let a = world
        .spawn((Transform2D::default(), OrbitalSpeed(1.0)))
        .id();
    let b = world
        .spawn((Transform2D::default(), OrbitalSpeed(3.0)))
        .id();
    let mut schedule = Schedule::default();
    schedule.add_systems(orbit_system);

    // Act
    schedule.run(&mut world);

    // Assert
    assert_eq!(world.get::<Transform2D>(a).unwrap().rotation, 1.0);
    assert_eq!(world.get::<Transform2D>(b).unwrap().rotation, 3.0);
}

#[test]
fn when_synodic_frame_and_orbits_run_then_earth_and_moon_stay_centered() {
    // Arrange
    let mut app = App::new();
    setup(&mut app);
    let world = app.world_mut();
    let earth = world.query::<(Entity, &Earth)>().single(world).unwrap().0;
    let moon = world.query::<(Entity, &Moon)>().single(world).unwrap().0;
    let sun = world.query::<(Entity, &Sun)>().single(world).unwrap().0;
    let mut schedule = Schedule::default();
    schedule.add_systems(
        (
            orbit_system,
            hierarchy_maintenance_system,
            transform_propagation_system,
            synodic_camera_system,
        )
            .chain(),
    );

    world.insert_resource(DeltaTime(Seconds(0.5)));

    // Act
    schedule.run(world);
    let camera_first = *world.query::<&Camera2D>().single(world).unwrap();
    let camera_rotation_first = world.query::<&CameraRotation>().single(world).unwrap().0;
    let earth_first = world.get::<GlobalTransform2D>(earth).unwrap().0.translation;
    let moon_first = world.get::<GlobalTransform2D>(moon).unwrap().0.translation;
    let sun_first = world.get::<GlobalTransform2D>(sun).unwrap().0.translation;
    let moon_rotation_first = world
        .get::<GlobalTransform2D>(moon)
        .unwrap()
        .0
        .to_scale_angle_translation()
        .1;
    let earth_screen_first = world_to_screen_with_rotation(
        earth_first,
        &camera_first,
        camera_rotation_first,
        800.0,
        600.0,
    );
    let moon_screen_first = world_to_screen_with_rotation(
        moon_first,
        &camera_first,
        camera_rotation_first,
        800.0,
        600.0,
    );
    let sun_screen_first = world_to_screen_with_rotation(
        sun_first,
        &camera_first,
        camera_rotation_first,
        800.0,
        600.0,
    );

    world.insert_resource(DeltaTime(Seconds(0.5)));
    schedule.run(world);
    let camera_second = *world.query::<&Camera2D>().single(world).unwrap();
    let camera_rotation_second = world.query::<&CameraRotation>().single(world).unwrap().0;
    let earth_second = world.get::<GlobalTransform2D>(earth).unwrap().0.translation;
    let moon_second = world.get::<GlobalTransform2D>(moon).unwrap().0.translation;
    let sun_second = world.get::<GlobalTransform2D>(sun).unwrap().0.translation;
    let moon_rotation_second = world
        .get::<GlobalTransform2D>(moon)
        .unwrap()
        .0
        .to_scale_angle_translation()
        .1;
    let earth_screen_second = world_to_screen_with_rotation(
        earth_second,
        &camera_second,
        camera_rotation_second,
        800.0,
        600.0,
    );
    let moon_screen_second = world_to_screen_with_rotation(
        moon_second,
        &camera_second,
        camera_rotation_second,
        800.0,
        600.0,
    );
    let sun_screen_second = world_to_screen_with_rotation(
        sun_second,
        &camera_second,
        camera_rotation_second,
        800.0,
        600.0,
    );

    // Assert
    assert_eq!(earth_screen_first, Vec2::new(400.0, 300.0));
    assert_eq!(earth_screen_second, Vec2::new(400.0, 300.0));
    assert!((earth_screen_first - earth_screen_second).length() < 1e-4);
    assert!((moon_screen_first - moon_screen_second).length() < 1e-4);
    assert!((sun_screen_first - sun_screen_second).length() > 1e-4);
    assert!((camera_rotation_first - moon_rotation_first).abs() < 1e-4);
    assert!((camera_rotation_second - moon_rotation_second).abs() < 1e-4);
}
