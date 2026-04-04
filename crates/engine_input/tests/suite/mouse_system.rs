#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::Schedule;
use bevy_ecs::world::World;
use engine_core::prelude::EventBus;

use engine_input::button_state::ButtonState;
use engine_input::mouse::{
    MouseButton, MouseInputEvent, MouseState, mouse_input_system, scroll_clear_system,
};

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
        .push(MouseInputEvent::Button {
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
        .push(MouseInputEvent::Button {
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
        .push(MouseInputEvent::Button {
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
        .push(MouseInputEvent::Button {
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
        .push(MouseInputEvent::Button {
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

#[test]
fn when_move_event_in_bus_then_mouse_input_system_sets_screen_pos() {
    // Arrange
    let mut world = setup_world();
    world
        .resource_mut::<EventBus<MouseInputEvent>>()
        .push(MouseInputEvent::Move {
            screen_pos: glam::Vec2::new(320.0, 240.0),
        });

    // Act
    run_mouse_system(&mut world);

    // Assert
    assert_eq!(
        world.resource::<MouseState>().screen_pos(),
        glam::Vec2::new(320.0, 240.0)
    );
}

#[test]
fn when_scroll_event_in_bus_then_mouse_input_system_accumulates_scroll_delta() {
    // Arrange
    let mut world = setup_world();
    world
        .resource_mut::<EventBus<MouseInputEvent>>()
        .push(MouseInputEvent::Scroll {
            delta: glam::Vec2::new(0.0, 3.0),
        });

    // Act
    run_mouse_system(&mut world);

    // Assert
    assert_eq!(
        world.resource::<MouseState>().scroll_delta(),
        glam::Vec2::new(0.0, 3.0)
    );
}
