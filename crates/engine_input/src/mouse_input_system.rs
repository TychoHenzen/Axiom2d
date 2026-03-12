use bevy_ecs::prelude::ResMut;
use winit::event::ElementState;

use crate::mouse_event_buffer::MouseEventBuffer;
use crate::mouse_state::MouseState;

pub fn mouse_input_system(mut buffer: ResMut<MouseEventBuffer>, mut state: ResMut<MouseState>) {
    state.clear_frame_state();
    for (button, element_state) in buffer.drain() {
        match element_state {
            ElementState::Pressed => state.press(button),
            ElementState::Released => state.release(button),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use bevy_ecs::prelude::Schedule;
    use bevy_ecs::world::World;
    use winit::event::ElementState;
    use winit::event::MouseButton;

    use crate::mouse_event_buffer::MouseEventBuffer;
    use crate::mouse_state::MouseState;

    use super::*;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(MouseState::default());
        world.insert_resource(MouseEventBuffer::default());
        world
    }

    fn run_mouse_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(mouse_input_system);
        schedule.run(world);
    }

    #[test]
    fn when_press_event_in_buffer_then_mouse_input_system_sets_button_pressed() {
        // Arrange
        let mut world = setup_world();
        world
            .resource_mut::<MouseEventBuffer>()
            .push(MouseButton::Left, ElementState::Pressed);

        // Act
        run_mouse_system(&mut world);

        // Assert
        assert!(world.resource::<MouseState>().pressed(MouseButton::Left));
    }

    #[test]
    fn when_press_event_in_buffer_then_mouse_input_system_sets_just_pressed() {
        // Arrange
        let mut world = setup_world();
        world
            .resource_mut::<MouseEventBuffer>()
            .push(MouseButton::Right, ElementState::Pressed);

        // Act
        run_mouse_system(&mut world);

        // Assert
        assert!(
            world
                .resource::<MouseState>()
                .just_pressed(MouseButton::Right)
        );
    }

    #[test]
    fn when_release_event_in_buffer_then_mouse_input_system_sets_just_released() {
        // Arrange
        let mut world = setup_world();
        world.resource_mut::<MouseState>().press(MouseButton::Left);
        world.resource_mut::<MouseState>().clear_frame_state();
        world
            .resource_mut::<MouseEventBuffer>()
            .push(MouseButton::Left, ElementState::Released);

        // Act
        run_mouse_system(&mut world);

        // Assert
        let state = world.resource::<MouseState>();
        assert!(state.just_released(MouseButton::Left));
        assert!(!state.pressed(MouseButton::Left));
    }

    #[test]
    fn when_mouse_input_system_runs_then_buffer_is_drained() {
        // Arrange
        let mut world = setup_world();
        world
            .resource_mut::<MouseEventBuffer>()
            .push(MouseButton::Left, ElementState::Pressed);

        // Act
        run_mouse_system(&mut world);

        // Assert
        assert_eq!(world.resource_mut::<MouseEventBuffer>().drain().count(), 0);
    }

    #[test]
    fn when_mouse_input_system_runs_second_frame_then_just_pressed_is_cleared() {
        // Arrange
        let mut world = setup_world();
        world
            .resource_mut::<MouseEventBuffer>()
            .push(MouseButton::Left, ElementState::Pressed);
        run_mouse_system(&mut world);

        // Act
        run_mouse_system(&mut world);

        // Assert
        let state = world.resource::<MouseState>();
        assert!(!state.just_pressed(MouseButton::Left));
        assert!(state.pressed(MouseButton::Left));
    }
}
