use bevy_ecs::prelude::ResMut;
use engine_core::prelude::EventBus;

use super::buffer::MouseInputEvent;
use super::state::MouseState;
use crate::button_state::ButtonState;

pub fn mouse_input_system(
    mut bus: ResMut<EventBus<MouseInputEvent>>,
    mut state: ResMut<MouseState>,
) {
    state.clear_frame_state();
    for MouseInputEvent { button, state: bs } in bus.drain() {
        match bs {
            ButtonState::Pressed => state.press(button),
            ButtonState::Released => state.release(button),
        }
    }
}

pub fn scroll_clear_system(mut state: ResMut<MouseState>) {
    state.clear_scroll_delta();
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use bevy_ecs::prelude::Schedule;
    use bevy_ecs::world::World;

    use crate::button_state::ButtonState;
    use crate::mouse_button::MouseButton;

    use super::*;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(MouseState::default());
        world.insert_resource(EventBus::<MouseInputEvent>::default());
        world
    }

    fn run_mouse_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(mouse_input_system);
        schedule.run(world);
    }

    #[test]
    fn when_press_event_in_bus_then_mouse_input_system_sets_button_pressed() {
        // Arrange
        let mut world = setup_world();
        world
            .resource_mut::<EventBus<MouseInputEvent>>()
            .push(MouseInputEvent {
                button: MouseButton::Left,
                state: ButtonState::Pressed,
            });

        // Act
        run_mouse_system(&mut world);

        // Assert
        assert!(world.resource::<MouseState>().pressed(MouseButton::Left));
    }

    #[test]
    fn when_press_event_in_bus_then_mouse_input_system_sets_just_pressed() {
        // Arrange
        let mut world = setup_world();
        world
            .resource_mut::<EventBus<MouseInputEvent>>()
            .push(MouseInputEvent {
                button: MouseButton::Right,
                state: ButtonState::Pressed,
            });

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
    fn when_release_event_in_bus_then_mouse_input_system_sets_just_released() {
        // Arrange
        let mut world = setup_world();
        world.resource_mut::<MouseState>().press(MouseButton::Left);
        world.resource_mut::<MouseState>().clear_frame_state();
        world
            .resource_mut::<EventBus<MouseInputEvent>>()
            .push(MouseInputEvent {
                button: MouseButton::Left,
                state: ButtonState::Released,
            });

        // Act
        run_mouse_system(&mut world);

        // Assert
        let state = world.resource::<MouseState>();
        assert!(state.just_released(MouseButton::Left));
        assert!(!state.pressed(MouseButton::Left));
    }

    #[test]
    fn when_mouse_input_system_runs_then_bus_is_drained() {
        // Arrange
        let mut world = setup_world();
        world
            .resource_mut::<EventBus<MouseInputEvent>>()
            .push(MouseInputEvent {
                button: MouseButton::Left,
                state: ButtonState::Pressed,
            });

        // Act
        run_mouse_system(&mut world);

        // Assert
        assert!(world.resource::<EventBus<MouseInputEvent>>().is_empty());
    }

    #[test]
    fn when_scroll_clear_system_runs_then_scroll_delta_zeroed() {
        // Arrange
        let mut world = setup_world();
        world
            .resource_mut::<MouseState>()
            .add_scroll_delta(glam::Vec2::new(1.0, 2.0));

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(scroll_clear_system);
        schedule.run(&mut world);

        // Assert
        assert_eq!(
            world.resource::<MouseState>().scroll_delta(),
            glam::Vec2::ZERO
        );
    }

    #[test]
    fn when_mouse_input_system_runs_second_frame_then_just_pressed_is_cleared() {
        // Arrange
        let mut world = setup_world();
        world
            .resource_mut::<EventBus<MouseInputEvent>>()
            .push(MouseInputEvent {
                button: MouseButton::Left,
                state: ButtonState::Pressed,
            });
        run_mouse_system(&mut world);

        // Act
        run_mouse_system(&mut world);

        // Assert
        let state = world.resource::<MouseState>();
        assert!(!state.just_pressed(MouseButton::Left));
        assert!(state.pressed(MouseButton::Left));
    }
}
