use axiom2d::prelude::*;

use crate::types::{
    CAMERA_PAN_SPEED, CAMERA_ZOOM_SPEED, Earth, FrameCount, Moon, OrbitalSpeed, ZOOM_MIN, action,
};

pub fn count_frames(mut count: ResMut<FrameCount>) {
    count.0 += 1;
}

pub fn camera_pan_system(
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

pub fn camera_zoom_system(
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

pub fn orbit_system(mut query: Query<(&mut Transform2D, &OrbitalSpeed)>, dt: Res<DeltaTime>) {
    for (mut transform, speed) in &mut query {
        transform.rotation += speed.0 * dt.0.0;
    }
}

pub fn synodic_camera_system(
    mut camera_query: Query<(&mut Camera2D, &mut CameraRotation)>,
    earth_query: Query<&GlobalTransform2D, With<Earth>>,
    moon_query: Query<&GlobalTransform2D, With<Moon>>,
) {
    let earth = earth_query
        .single()
        .expect("expected exactly one Earth entity");
    let moon = moon_query
        .single()
        .expect("expected exactly one Moon entity");
    let (mut camera, mut camera_rotation) = camera_query
        .single_mut()
        .expect("expected exactly one camera entity");
    let (_, _, translation) = earth.0.to_scale_angle_translation();
    let (_, rotation, _) = moon.0.to_scale_angle_translation();
    camera.position = translation;
    camera_rotation.0 = rotation;
}
