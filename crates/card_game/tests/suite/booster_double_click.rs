#![allow(clippy::unwrap_used)]

use bevy_ecs::entity::Entity;
use card_game::booster::double_click::{DOUBLE_CLICK_WINDOW, DoubleClickState};

#[test]
fn when_same_entity_clicked_within_window_then_double_click_detected() {
    // Arrange
    let mut state = DoubleClickState::default();
    let entity = Entity::from_raw(1);

    // Act
    let first = state.register_click(entity, 1.0);
    let second = state.register_click(entity, 1.1);

    // Assert
    assert!(first.is_none());
    assert_eq!(second, Some(entity));
}

#[test]
fn when_same_entity_clicked_outside_window_then_no_double_click() {
    // Arrange
    let mut state = DoubleClickState::default();
    let entity = Entity::from_raw(2);

    // Act
    let first = state.register_click(entity, 1.0);
    // 0.5 > DOUBLE_CLICK_WINDOW (0.3), so no double-click
    let second = state.register_click(entity, 1.0 + DOUBLE_CLICK_WINDOW + 0.2);

    // Assert
    assert!(first.is_none());
    assert!(second.is_none());
}

#[test]
fn when_different_entities_clicked_then_no_double_click() {
    // Arrange
    let mut state = DoubleClickState::default();
    let e1 = Entity::from_raw(3);
    let e2 = Entity::from_raw(4);

    // Act
    let first = state.register_click(e1, 1.0);
    let second = state.register_click(e2, 1.1);

    // Assert
    assert!(first.is_none());
    assert!(second.is_none());
}
