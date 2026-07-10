#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::{Schedule, World};
use card_game::stash::toggle::{StashVisible, stash_toggle_system};
use engine_input::prelude::{InputState, KeyCode};

fn run_system(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(stash_toggle_system);
    schedule.run(world);
}

/// @doc: Verifies that the stash toggle system makes `StashVisible` true when Tab is pressed while hidden.
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
    assert!(
        world.resource::<StashVisible>().0,
        "stash should become visible when Tab pressed while hidden"
    );
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
    assert!(
        !world.resource::<StashVisible>().0,
        "stash should become hidden when Tab pressed while visible"
    );
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
    assert!(
        !world.resource::<StashVisible>().0,
        "stash should remain hidden when no Tab press"
    );
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
    assert!(
        world.resource::<StashVisible>().0,
        "stash should remain visible when no Tab press"
    );
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
    assert!(
        !world.resource::<StashVisible>().0,
        "stash should remain hidden when Enter pressed"
    );
}

/// @doc: Tab held across two frames toggles only once — just_pressed clears after first frame.
#[test]
fn when_tab_held_two_frames_then_toggles_only_once() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(StashVisible(false));
    let mut input = InputState::default();
    input.press(KeyCode::Tab);
    world.insert_resource(input);

    // Act — first frame toggles hidden → visible
    run_system(&mut world);
    assert!(world.resource::<StashVisible>().0, "stash should be visible after first Tab press");

    // Advance frame state: clear just_pressed while keeping pressed=true (key held)
    world.resource_mut::<InputState>().clear_frame_state();
    // Second frame — Tab still held but just_pressed is cleared
    run_system(&mut world);

    // Assert — stays visible, no double-toggle
    assert!(world.resource::<StashVisible>().0, "stash should stay visible when Tab held across frames");
}

/// @doc: Press Tab, release, press Tab again — stash toggles twice and returns to original state.
#[test]
fn when_tab_release_tab_then_returns_to_original_state() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(StashVisible(false));
    let mut input = InputState::default();
    input.press(KeyCode::Tab);
    world.insert_resource(input);

    // Act — first press: hidden → visible
    run_system(&mut world);
    assert!(world.resource::<StashVisible>().0, "stash should be visible after first Tab press");

    // Release Tab and clear frame state
    world.resource_mut::<InputState>().clear_frame_state();
    // Second press
    world.resource_mut::<InputState>().press(KeyCode::Tab);
    run_system(&mut world);

    // Assert — back to hidden
    assert!(!world.resource::<StashVisible>().0, "stash should return to hidden after Tab press-release-press");
}

/// @doc: When resources are present in default state (StashVisible not inserted), the system panics.
/// This test verifies the system requires StashVisible to exist — the behavior is intentional.
#[test]
#[should_panic(expected = "StashVisible")]
fn when_stash_visible_missing_then_system_panics() {
    // Arrange — no StashVisible resource
    let mut world = World::new();
    let mut input = InputState::default();
    input.press(KeyCode::Tab);
    world.insert_resource(input);

    // Act — system expects StashVisible, should panic
    run_system(&mut world);
}
