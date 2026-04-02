#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::Schedule;
use bevy_ecs::world::World;
use engine_core::prelude::EventBus;

use engine_input::button_state::ButtonState;
use engine_input::key_code::KeyCode;
use engine_input::keyboard::{InputState, KeyInputEvent, input_system};

fn setup_world() -> World {
    let mut world = World::new();
    world.insert_resource(InputState::default());
    world.insert_resource(EventBus::<KeyInputEvent>::default());
    world
}

fn run_input_system(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(input_system);
    schedule.run(world);
}

#[test]
fn when_press_event_in_bus_then_key_is_pressed() {
    // Arrange
    let mut world = setup_world();
    world
        .resource_mut::<EventBus<KeyInputEvent>>()
        .push(KeyInputEvent {
            key: KeyCode::ArrowRight,
            state: ButtonState::Pressed,
        });

    // Act
    run_input_system(&mut world);

    // Assert
    assert!(world.resource::<InputState>().pressed(KeyCode::ArrowRight));
}

#[test]
fn when_press_event_in_bus_then_key_is_just_pressed() {
    // Arrange
    let mut world = setup_world();
    world
        .resource_mut::<EventBus<KeyInputEvent>>()
        .push(KeyInputEvent {
            key: KeyCode::ArrowRight,
            state: ButtonState::Pressed,
        });

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
fn when_release_event_in_bus_then_key_is_not_pressed() {
    // Arrange
    let mut world = setup_world();
    world.resource_mut::<InputState>().press(KeyCode::Space);
    world.resource_mut::<InputState>().clear_frame_state();
    world
        .resource_mut::<EventBus<KeyInputEvent>>()
        .push(KeyInputEvent {
            key: KeyCode::Space,
            state: ButtonState::Released,
        });

    // Act
    run_input_system(&mut world);

    // Assert
    assert!(!world.resource::<InputState>().pressed(KeyCode::Space));
}

#[test]
fn when_release_event_in_bus_then_key_is_just_released() {
    // Arrange
    let mut world = setup_world();
    world.resource_mut::<InputState>().press(KeyCode::Space);
    world.resource_mut::<InputState>().clear_frame_state();
    world
        .resource_mut::<EventBus<KeyInputEvent>>()
        .push(KeyInputEvent {
            key: KeyCode::Space,
            state: ButtonState::Released,
        });

    // Act
    run_input_system(&mut world);

    // Assert
    assert!(world.resource::<InputState>().just_released(KeyCode::Space));
}

#[test]
fn when_system_runs_then_bus_is_drained() {
    // Arrange
    let mut world = setup_world();
    world
        .resource_mut::<EventBus<KeyInputEvent>>()
        .push(KeyInputEvent {
            key: KeyCode::ArrowRight,
            state: ButtonState::Pressed,
        });

    // Act
    run_input_system(&mut world);

    // Assert
    assert!(world.resource::<EventBus<KeyInputEvent>>().is_empty());
}

#[test]
fn when_system_runs_second_frame_then_just_pressed_is_cleared() {
    // Arrange
    let mut world = setup_world();
    world
        .resource_mut::<EventBus<KeyInputEvent>>()
        .push(KeyInputEvent {
            key: KeyCode::ArrowDown,
            state: ButtonState::Pressed,
        });
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
        .resource_mut::<EventBus<KeyInputEvent>>()
        .push(KeyInputEvent {
            key: KeyCode::Space,
            state: ButtonState::Released,
        });
    run_input_system(&mut world);

    // Act
    run_input_system(&mut world);

    // Assert
    assert!(!world.resource::<InputState>().just_released(KeyCode::Space));
}
