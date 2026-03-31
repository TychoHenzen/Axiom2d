use bevy_ecs::prelude::{Query, Res, ResMut, With, Without};
use engine_core::prelude::{EventBus, Transform2D};
use engine_input::prelude::MouseState;
use engine_physics::prelude::PhysicsRes;

use crate::card::interaction::physics_helpers::warn_on_physics_result;
use crate::card::reader::components::{CardReader, ReaderDragState, ReaderReleaseIntent};

pub fn reader_drag_system(
    mouse: Res<MouseState>,
    reader_drag: Res<ReaderDragState>,
    mut reader_transforms: Query<&mut Transform2D, With<CardReader>>,
    mut card_transforms: Query<&mut Transform2D, Without<CardReader>>,
    readers: Query<&CardReader>,
    mut physics: ResMut<PhysicsRes>,
) {
    use engine_input::mouse_button::MouseButton;

    let Some(info) = &reader_drag.dragging else {
        return;
    };
    if !mouse.pressed(MouseButton::Left) {
        return;
    }
    let target = mouse.world_pos() - info.grab_offset;
    if let Ok(mut transform) = reader_transforms.get_mut(info.entity) {
        transform.position = target;
    }
    warn_on_physics_result(
        "set_body_position",
        info.entity,
        physics.set_body_position(info.entity, target),
    );
    if let Ok(reader) = readers.get(info.entity)
        && let Some(card_entity) = reader.loaded
        && let Ok(mut card_transform) = card_transforms.get_mut(card_entity)
    {
        card_transform.position = target;
    }
}

pub fn reader_release_intent_system(
    mouse: Res<MouseState>,
    reader_drag: Res<ReaderDragState>,
    mut intents: ResMut<EventBus<ReaderReleaseIntent>>,
) {
    use engine_input::mouse_button::MouseButton;

    if reader_drag.dragging.is_some() && !mouse.pressed(MouseButton::Left) {
        intents.push(ReaderReleaseIntent);
    }
}

pub fn apply_reader_release_intents_system(
    mut reader_drag: ResMut<ReaderDragState>,
    mut intents: ResMut<EventBus<ReaderReleaseIntent>>,
) {
    if intents.drain().next().is_some() {
        reader_drag.dragging = None;
    }
}
