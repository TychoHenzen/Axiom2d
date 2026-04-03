use bevy_ecs::prelude::{Query, Res, ResMut, With, Without};
use engine_core::prelude::Transform2D;
use engine_input::mouse_button::MouseButton;
use engine_input::prelude::MouseState;
use engine_physics::prelude::PhysicsRes;

use crate::card::interaction::physics_helpers::warn_on_physics_result;
use crate::card::reader::components::{CardReader, ReaderDragState};
use crate::card::reader::spawn::READER_JACK_OFFSET;

pub fn reader_drag_system(
    mouse: Res<MouseState>,
    reader_drag: Res<ReaderDragState>,
    mut reader_transforms: Query<&mut Transform2D, With<CardReader>>,
    mut card_transforms: Query<&mut Transform2D, Without<CardReader>>,
    readers: Query<&CardReader>,
    mut physics: ResMut<PhysicsRes>,
) {
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
    if let Ok(reader) = readers.get(info.entity) {
        if let Ok(mut jack_t) = card_transforms.get_mut(reader.jack_entity) {
            jack_t.position = target + READER_JACK_OFFSET;
        }
        if let Some(card_entity) = reader.loaded
            && let Ok(mut card_transform) = card_transforms.get_mut(card_entity)
        {
            card_transform.position = target;
        }
    }
}

pub fn reader_release_system(mouse: Res<MouseState>, mut reader_drag: ResMut<ReaderDragState>) {
    if reader_drag.dragging.is_some() && !mouse.pressed(MouseButton::Left) {
        reader_drag.dragging = None;
    }
}
