use bevy_ecs::prelude::{Query, Res, ResMut};
use engine_input::mouse_state::MouseState;
use engine_render::camera::{Camera2D, screen_to_world};

use crate::window_size::WindowSize;

pub fn mouse_world_pos_system(
    window_size: Res<WindowSize>,
    camera_query: Query<&Camera2D>,
    mut mouse: ResMut<MouseState>,
) {
    let camera = camera_query.iter().next().cloned().unwrap_or_default();
    let world_pos = screen_to_world(
        mouse.screen_pos(),
        &camera,
        window_size.width.0,
        window_size.height.0,
    );
    mouse.set_world_pos(world_pos);
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use bevy_ecs::prelude::{Schedule, World};
    use engine_input::mouse_state::MouseState;
    use engine_render::camera::Camera2D;
    use glam::Vec2;

    use crate::window_size::WindowSize;
    use engine_core::types::Pixels;

    use super::*;

    fn setup_world(screen_pos: Vec2, width: u32, height: u32) -> World {
        let mut world = World::new();
        let mut mouse = MouseState::default();
        mouse.set_screen_pos(screen_pos);
        world.insert_resource(mouse);
        world.insert_resource(WindowSize {
            width: Pixels(width as f32),
            height: Pixels(height as f32),
        });
        world
    }

    fn run_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(mouse_world_pos_system);
        schedule.run(world);
    }

    #[test]
    fn when_world_pos_system_runs_with_camera_then_world_pos_is_screen_to_world_converted() {
        // Arrange
        let mut world = setup_world(Vec2::new(400.0, 300.0), 800, 600);
        world.spawn(Camera2D::default());

        // Act
        run_system(&mut world);

        // Assert
        let mouse = world.resource::<MouseState>();
        assert_eq!(mouse.world_pos(), Vec2::new(0.0, 0.0));
    }

    #[test]
    fn when_world_pos_system_runs_with_no_camera_then_uses_default_camera() {
        // Arrange
        let mut world = setup_world(Vec2::new(400.0, 300.0), 800, 600);

        // Act
        run_system(&mut world);

        // Assert
        let mouse = world.resource::<MouseState>();
        assert_eq!(mouse.world_pos(), Vec2::new(0.0, 0.0));
    }

    #[test]
    fn when_world_pos_system_runs_with_zoomed_camera_then_center_still_maps_to_camera_pos() {
        // Arrange
        let mut world = setup_world(Vec2::new(400.0, 300.0), 800, 600);
        world.spawn(Camera2D {
            zoom: 2.0,
            ..Camera2D::default()
        });

        // Act
        run_system(&mut world);

        // Assert
        let mouse = world.resource::<MouseState>();
        assert_eq!(mouse.world_pos(), Vec2::new(0.0, 0.0));
    }

    #[test]
    fn when_world_pos_system_runs_with_offset_cursor_and_zoom_then_world_pos_is_scaled() {
        // Arrange
        let mut world = setup_world(Vec2::new(600.0, 300.0), 800, 600);
        world.spawn(Camera2D {
            zoom: 2.0,
            ..Camera2D::default()
        });

        // Act
        run_system(&mut world);

        // Assert
        let mouse = world.resource::<MouseState>();
        assert!((mouse.world_pos().x - 100.0).abs() < 1e-4);
        assert!(mouse.world_pos().y.abs() < 1e-4);
    }
}
