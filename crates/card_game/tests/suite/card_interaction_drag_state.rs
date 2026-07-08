#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::Entity;
use glam::Vec2;

use card_game::card::component::CardZone;
use card_game::card::interaction::drag_state::{DeviceDragInfo, DragInfo, DragState};

/// @doc: `DragState` defaults to no active drag.
#[test]
fn when_drag_state_default_then_not_dragging() {
    // Arrange
    let state = DragState::default();

    // Act & Assert
    assert!(
        state.dragging.is_none(),
        "DragState should start with no active drag"
    );
}

/// @doc: `DragState` can be assigned a `DragInfo` and reflects the active drag entity.
#[test]
fn when_drag_state_set_to_some_then_contains_drag_info() {
    // Arrange
    let entity = Entity::from_raw(1);
    let info = DragInfo {
        entity,
        local_grab_offset: Vec2::new(2.0, 3.0),
        origin_zone: CardZone::Hand(0),
        stash_cursor_follow: false,
        origin_position: Vec2::splat(50.0),
    };

    // Act
    let state = DragState {
        dragging: Some(info),
    };

    // Assert
    assert!(
        state.dragging.is_some(),
        "DragState should contain a drag after assignment"
    );
    assert_eq!(
        state.dragging.unwrap().entity,
        entity,
        "dragging entity should match the assigned value"
    );
}

/// @doc: `DragState` can be cleared back to None.
#[test]
fn when_drag_state_cleared_then_not_dragging() {
    // Arrange
    let mut state = DragState {
        dragging: Some(DragInfo {
            entity: Entity::from_raw(5),
            local_grab_offset: Vec2::ZERO,
            origin_zone: CardZone::Table,
            stash_cursor_follow: false,
            origin_position: Vec2::ZERO,
        }),
    };

    // Act
    state.dragging = None;

    // Assert
    assert!(
        state.dragging.is_none(),
        "DragState should have no active drag after clearing"
    );
}

/// @doc: `DragInfo` with Hand zone origin carries correct zone and position.
#[test]
fn when_drag_info_hand_zone_then_fields_match() {
    // Arrange
    let entity = Entity::from_raw(10);
    let offset = Vec2::new(15.0, -5.0);
    let pos = Vec2::new(200.0, 300.0);

    // Act
    let info = DragInfo {
        entity,
        local_grab_offset: offset,
        origin_zone: CardZone::Hand(2),
        stash_cursor_follow: false,
        origin_position: pos,
    };

    // Assert
    assert_eq!(info.entity, entity, "entity field mismatch");
    assert_eq!(info.local_grab_offset, offset, "local_grab_offset mismatch");
    assert!(
        matches!(info.origin_zone, CardZone::Hand(2)),
        "origin_zone should be Hand(2)"
    );
    assert!(
        !info.stash_cursor_follow,
        "stash_cursor_follow should be false"
    );
    assert_eq!(info.origin_position, pos, "origin_position mismatch");
}

/// @doc: `DragInfo` with Stash zone origin stores page, col, row.
#[test]
fn when_drag_info_stash_zone_then_grid_coords_match() {
    // Arrange
    let info = DragInfo {
        entity: Entity::from_raw(10),
        local_grab_offset: Vec2::ZERO,
        origin_zone: CardZone::Stash {
            page: 1,
            col: 3,
            row: 2,
        },
        stash_cursor_follow: true,
        origin_position: Vec2::ZERO,
    };

    // Act & Assert
    let CardZone::Stash { page, col, row } = info.origin_zone else {
        panic!("expected Stash zone");
    };
    assert_eq!(page, 1, "stash page mismatch");
    assert_eq!(col, 3, "stash col mismatch");
    assert_eq!(row, 2, "stash row mismatch");
    assert!(
        info.stash_cursor_follow,
        "stash_cursor_follow should be true for stash-origin drag"
    );
}

/// @doc: `DeviceDragInfo` stores entity and grab offset.
#[test]
fn when_device_drag_info_constructed_then_fields_match() {
    // Arrange
    let entity = Entity::from_raw(3);
    let offset = Vec2::new(5.0, -5.0);

    // Act
    let info = DeviceDragInfo {
        entity,
        grab_offset: offset,
    };

    // Assert
    assert_eq!(info.entity, entity, "entity field mismatch");
    assert_eq!(info.grab_offset, offset, "grab_offset mismatch");
}
