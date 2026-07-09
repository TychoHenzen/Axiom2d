#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::{Schedule, World};
use card_game::stash::toggle::{StashVisible, stash_toggle_system};
use engine_input::prelude::{InputState, KeyCode};

fn run_system(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(stash_toggle_system);
    schedule.run(world);
}

/// @doc: Verifies that the stash toggle system makes StashVisible true when Tab is pressed while hidden.
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
    assert!(world.resource::<StashVisible>().0, "stash should become visible when Tab pressed while hidden");
}

/// @doc: Verifies that the stash toggle system hides the stash when Tab is pressed while visible.
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
    assert!(!world.resource::<StashVisible>().0, "stash should become hidden when Tab pressed while visible");
}

/// @doc: When Tab is not pressed, hidden stash stays hidden (no spurious toggle).
#[test]
fn when_tab_not_pressed_and_hidden_then_stays_hidden() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(StashVisible(false));
    world.insert_resource(InputState::default());

    // Act
    run_system(&mut world);

    // Assert
    assert!(!world.resource::<StashVisible>().0, "stash should remain hidden when no Tab press");
}

/// @doc: When Tab is not pressed, visible stash stays visible (no auto-hide).
#[test]
fn when_tab_not_pressed_and_visible_then_stays_visible() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(StashVisible(true));
    world.insert_resource(InputState::default());

    // Act
    run_system(&mut world);

    // Assert
    assert!(world.resource::<StashVisible>().0, "stash should remain visible when no Tab press");
}

/// @doc: Pressing a non-Tab key does not toggle stash visibility.
#[test]
fn when_non_tab_key_pressed_then_stash_unchanged() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(StashVisible(false));
    let mut input = InputState::default();
    input.press(KeyCode::Enter);
    world.insert_resource(input);

    // Act
    run_system(&mut world);

    // Assert
    assert!(!world.resource::<StashVisible>().0, "stash should remain hidden when Enter pressed");
}
