use axiom2d::prelude::*;

use crate::types::{
    CAMERA_PAN_SPEED, CAMERA_ZOOM_SPEED, FrameCount, OrbitalSpeed, ZOOM_MIN, action,
};

pub(crate) fn count_frames(mut count: ResMut<FrameCount>) {
    count.0 += 1;
}

pub(crate) fn camera_pan_system(
    mut query: Query<&mut Camera2D>,
    input: Res<InputState>,
    action_map: Res<ActionMap>,
    dt: Res<DeltaTime>,
) {
    let mut dx = 0.0;
    let mut dy = 0.0;
    if input.action_pressed(&action_map, action::MOVE_RIGHT) {
        dx += 1.0;
    }
    if input.action_pressed(&action_map, action::MOVE_LEFT) {
        dx -= 1.0;
    }
    if input.action_pressed(&action_map, action::MOVE_DOWN) {
        dy += 1.0;
    }
    if input.action_pressed(&action_map, action::MOVE_UP) {
        dy -= 1.0;
    }
    let displacement = CAMERA_PAN_SPEED * dt.0.0;
    for mut camera in &mut query {
        camera.position.x += dx * displacement;
        camera.position.y += dy * displacement;
    }
}

pub(crate) fn camera_zoom_system(
    mut query: Query<&mut Camera2D>,
    input: Res<InputState>,
    action_map: Res<ActionMap>,
    dt: Res<DeltaTime>,
) {
    let mut zoom_dir = 0.0;
    if input.action_pressed(&action_map, action::ZOOM_IN) {
        zoom_dir += 1.0;
    }
    if input.action_pressed(&action_map, action::ZOOM_OUT) {
        zoom_dir -= 1.0;
    }
    let zoom_delta = CAMERA_ZOOM_SPEED * dt.0.0 * zoom_dir;
    for mut camera in &mut query {
        camera.zoom = (camera.zoom + zoom_delta).max(ZOOM_MIN);
    }
}

pub(crate) fn orbit_system(
    mut query: Query<(&mut Transform2D, &OrbitalSpeed)>,
    dt: Res<DeltaTime>,
) {
    for (mut transform, speed) in &mut query {
        transform.rotation += speed.0 * dt.0.0;
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::float_cmp)]
mod tests {
    use super::*;
    use crate::types::action;

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
}
