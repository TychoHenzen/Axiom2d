pub mod scene;
pub mod systems;
pub mod types;

use axiom2d::prelude::*;

pub use scene::{spawn_camera, spawn_nebula, spawn_planets, spawn_sun, spawn_synodic_frame};
pub use systems::{orbit_system, synodic_camera_system};
pub use types::{FrameCount, action};

pub fn setup(app: &mut App) {
    app.add_plugin(DefaultPlugins);

    let config = WindowConfig {
        title: "Axiom2d Synodic Solar System",
        ..Default::default()
    };
    let mut action_map = ActionMap::default();
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

    let frame = spawn_synodic_frame(app.world_mut());
    spawn_sun(app.world_mut(), frame);
    spawn_planets(app.world_mut(), frame);
    spawn_nebula(app.world_mut());
    spawn_camera(app.world_mut());

    app.set_window_config(config).add_systems(
        Phase::Update,
        (
            crate::systems::count_frames,
            crate::systems::orbit_system,
            crate::systems::camera_zoom_system,
        ),
    );
    app.add_systems(
        Phase::PostUpdate,
        crate::systems::synodic_camera_system.after(transform_propagation_system),
    );
}
