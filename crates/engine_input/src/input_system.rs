use bevy_ecs::prelude::ResMut;
use winit::event::ElementState;

use crate::input_event_buffer::InputEventBuffer;
use crate::input_state::InputState;

pub fn input_system(mut buffer: ResMut<InputEventBuffer>, mut state: ResMut<InputState>) {
    state.clear_frame_state();
    for (key, element_state) in buffer.drain() {
        match element_state {
            ElementState::Pressed => state.press(key),
            ElementState::Released => state.release(key),
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::Schedule;
    use bevy_ecs::world::World;
    use winit::event::ElementState;
    use winit::keyboard::KeyCode;

    use crate::input_event_buffer::InputEventBuffer;
    use crate::input_state::InputState;

    use super::*;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(InputState::default());
        world.insert_resource(InputEventBuffer::default());
        world
    }

    fn run_input_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(input_system);
        schedule.run(world);
    }

    #[test]
    fn when_press_event_in_buffer_then_key_is_pressed() {
        // Arrange
        let mut world = setup_world();
        world
            .resource_mut::<InputEventBuffer>()
            .push(KeyCode::ArrowRight, ElementState::Pressed);

        // Act
        run_input_system(&mut world);

        // Assert
        assert!(world.resource::<InputState>().pressed(KeyCode::ArrowRight));
    }

    #[test]
    fn when_press_event_in_buffer_then_key_is_just_pressed() {
        // Arrange
        let mut world = setup_world();
        world
            .resource_mut::<InputEventBuffer>()
            .push(KeyCode::ArrowRight, ElementState::Pressed);

        // Act
        run_input_system(&mut world);

        // Assert
        assert!(
            world
                .resource::<InputState>()
                .just_pressed(KeyCode::ArrowRight)
        );
    }

    #[test]
    fn when_release_event_in_buffer_then_key_is_not_pressed() {
        // Arrange
        let mut world = setup_world();
        world.resource_mut::<InputState>().press(KeyCode::Space);
        world.resource_mut::<InputState>().clear_frame_state();
        world
            .resource_mut::<InputEventBuffer>()
            .push(KeyCode::Space, ElementState::Released);

        // Act
        run_input_system(&mut world);

        // Assert
        assert!(!world.resource::<InputState>().pressed(KeyCode::Space));
    }

    #[test]
    fn when_release_event_in_buffer_then_key_is_just_released() {
        // Arrange
        let mut world = setup_world();
        world.resource_mut::<InputState>().press(KeyCode::Space);
        world.resource_mut::<InputState>().clear_frame_state();
        world
            .resource_mut::<InputEventBuffer>()
            .push(KeyCode::Space, ElementState::Released);

        // Act
        run_input_system(&mut world);

        // Assert
        assert!(world.resource::<InputState>().just_released(KeyCode::Space));
    }

    #[test]
    fn when_system_runs_then_buffer_is_drained() {
        // Arrange
        let mut world = setup_world();
        world
            .resource_mut::<InputEventBuffer>()
            .push(KeyCode::ArrowRight, ElementState::Pressed);

        // Act
        run_input_system(&mut world);

        // Assert
        assert!(world.resource_mut::<InputEventBuffer>().drain().is_empty());
    }

    #[test]
    fn when_system_runs_second_frame_then_just_pressed_is_cleared() {
        // Arrange
        let mut world = setup_world();
        world
            .resource_mut::<InputEventBuffer>()
            .push(KeyCode::ArrowDown, ElementState::Pressed);
        run_input_system(&mut world);

        // Act
        run_input_system(&mut world);

        // Assert
        let state = world.resource::<InputState>();
        assert!(!state.just_pressed(KeyCode::ArrowDown));
        assert!(state.pressed(KeyCode::ArrowDown));
    }

    #[test]
    fn when_system_runs_second_frame_then_just_released_is_cleared() {
        // Arrange
        let mut world = setup_world();
        world.resource_mut::<InputState>().press(KeyCode::Space);
        world.resource_mut::<InputState>().clear_frame_state();
        world
            .resource_mut::<InputEventBuffer>()
            .push(KeyCode::Space, ElementState::Released);
        run_input_system(&mut world);

        // Act
        run_input_system(&mut world);

        // Assert
        assert!(!world.resource::<InputState>().just_released(KeyCode::Space));
    }
}
