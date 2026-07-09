#![allow(clippy::unwrap_used)]

use engine_input::action_map::ActionMap;
use engine_input::key_code::KeyCode;

/// @doc: Verifies that binding multiple keys to an action returns all bound keys from bindings_for.
#[test]
fn when_multiple_keys_bound_to_same_action_then_all_keys_returned() {
    // Arrange
    let mut map = ActionMap::default();

    // Act
    map.bind("move_right", vec![KeyCode::ArrowRight, KeyCode::KeyD]);

    // Assert
    assert_eq!(
        map.bindings_for("move_right"),
        &[KeyCode::ArrowRight, KeyCode::KeyD],
        "bindings_for should return all keys bound to an action"
    );
}

/// @doc: Verifies that binding a single key to an action returns it from bindings_for.
#[test]
fn when_single_key_bound_to_action_then_bindings_for_returns_that_key() {
    // Arrange
    let mut map = ActionMap::default();

    // Act
    map.bind("jump", vec![KeyCode::Space]);

    // Assert
    assert_eq!(map.bindings_for("jump"), &[KeyCode::Space], "bindings_for should return the single bound key");
}
