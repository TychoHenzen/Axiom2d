#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::{Schedule, World};
use card_game::stash::toggle::{StashVisible, stash_toggle_system};
use engine_input::prelude::{InputState, KeyCode};

fn run_system(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(stash_toggle_system);
    schedule.run(world);
}

#[test]
fn when_tab_just_pressed_and_hidden_then_becomes_visible() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(StashVisible(false));
    let mut input = InputState::default();
    input.press(KeyCode::Tab);
    world.insert_resource(input);

    // Act
    run_system(&mut world);

    // Assert
    assert!(world.resource::<StashVisible>().0);
}

#[test]
fn when_tab_just_pressed_and_visible_then_becomes_hidden() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(StashVisible(true));
    let mut input = InputState::default();
    input.press(KeyCode::Tab);
    world.insert_resource(input);

    // Act
    run_system(&mut world);

    // Assert
    assert!(!world.resource::<StashVisible>().0);
}
