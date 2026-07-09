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

/// @doc: Verifies that input_system marks a key as pressed from a press event in the bus.
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
    assert!(
        world.resource::<InputState>().pressed(KeyCode::ArrowRight),
        "key should be marked pressed after a Press event"
    );
}

/// @doc: Verifies that input_system marks a key as just pressed from a press event.
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
            .just_pressed(KeyCode::ArrowRight),
        "key should be marked just_pressed after a Press event"
    );
}

/// @doc: Verifies that input_system clears pressed state from a release event.
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
    assert!(
        !world.resource::<InputState>().pressed(KeyCode::Space),
        "key should not be pressed after a Release event"
    );
}

/// @doc: Verifies that input_system marks a key as just released from a release event.
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
    assert!(
        world.resource::<InputState>().just_released(KeyCode::Space),
        "key should be marked just_released after a Release event"
    );
}

/// @doc: Verifies that input_system drains the event bus after processing.
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
    assert!(
        world.resource::<EventBus<KeyInputEvent>>().is_empty(),
        "event bus should be drained after input_system processes events"
    );
}

/// @doc: Verifies that just-pressed state clears on the second frame while held state persists.
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
    assert!(
        !state.just_pressed(KeyCode::ArrowDown),
        "just_pressed should clear after the first frame"
    );
    assert!(
        state.pressed(KeyCode::ArrowDown),
        "pressed state should persist beyond the first frame"
    );
}

/// @doc: Verifies that just-released state clears on the second frame after a release event.
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
    assert!(
        !world.resource::<InputState>().just_released(KeyCode::Space),
        "just_released should clear after the second frame"
    );
}

/// @doc: Releasing a key that was never pressed does not mark it as pressed, but still
/// records just_released for the release event itself.
#[test]
fn when_release_unpressed_key_then_not_pressed() {
    // Arrange
    let mut world = setup_world();
    world
        .resource_mut::<EventBus<KeyInputEvent>>()
        .push(KeyInputEvent {
            key: KeyCode::Space,
            state: ButtonState::Released,
        });

    // Act
    run_input_system(&mut world);

    // Assert
    let state = world.resource::<InputState>();
    assert!(
        !state.pressed(KeyCode::Space),
        "releasing an unpressed key should not mark it pressed"
    );
}

/// @doc: Pressing two different keys in one frame registers both as pressed and just_pressed.
#[test]
fn when_two_keys_pressed_in_one_frame_then_both_registered() {
    // Arrange
    let mut world = setup_world();
    world
        .resource_mut::<EventBus<KeyInputEvent>>()
        .push(KeyInputEvent {
            key: KeyCode::ArrowLeft,
            state: ButtonState::Pressed,
        });
    world
        .resource_mut::<EventBus<KeyInputEvent>>()
        .push(KeyInputEvent {
            key: KeyCode::ArrowRight,
            state: ButtonState::Pressed,
        });

    // Act
    run_input_system(&mut world);

    // Assert
    let state = world.resource::<InputState>();
    assert!(
        state.pressed(KeyCode::ArrowLeft),
        "ArrowLeft should be pressed after its press event"
    );
    assert!(
        state.pressed(KeyCode::ArrowRight),
        "ArrowRight should be pressed after its press event"
    );
    assert!(
        state.just_pressed(KeyCode::ArrowLeft),
        "ArrowLeft should be just_pressed after its press event"
    );
    assert!(
        state.just_pressed(KeyCode::ArrowRight),
        "ArrowRight should be just_pressed after its press event"
    );
}

/// @doc: An empty event bus is a no-op — state is cleared of frame state but otherwise unchanged.
#[test]
fn when_empty_event_bus_then_state_unchanged() {
    // Arrange
    let mut world = setup_world();
    world.resource_mut::<InputState>().press(KeyCode::Tab);
    world.resource_mut::<InputState>().clear_frame_state();

    // Act
    run_input_system(&mut world);

    // Assert
    let state = world.resource::<InputState>();
    assert!(
        state.pressed(KeyCode::Tab),
        "pressed state should persist when bus is empty"
    );
    assert!(
        !state.just_pressed(KeyCode::Tab),
        "just_pressed should be cleared when bus is empty"
    );
}
