pub mod scene;
pub mod systems;
pub mod types;

use axiom2d::prelude::*;

pub use scene::{spawn_camera, spawn_nebula, spawn_planets, spawn_sun};
pub use systems::orbit_system;
pub use types::{FrameCount, action};

pub fn setup(app: &mut App) {
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
            crate::systems::count_frames,
            crate::systems::orbit_system,
            crate::systems::camera_pan_system,
            crate::systems::camera_zoom_system,
        ),
    );
}
