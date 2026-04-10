// EVOLVE-BLOCK-START
use bevy_ecs::prelude::{Query, Res, ResMut, With, Without};
use engine_core::prelude::{EventBus, Transform2D};
use engine_input::mouse_button::MouseButton;
use engine_input::prelude::MouseState;
use engine_physics::prelude::PhysicsCommand;

use crate::card::reader::components::{
    CardReader, READER_COLLISION_FILTER, READER_COLLISION_GROUP, ReaderDragState,
};
use crate::card::reader::spawn::READER_JACK_OFFSET;

pub fn reader_drag_system(
    mouse: Res<MouseState>,
    reader_drag: Res<ReaderDragState>,
    mut reader_transforms: Query<&mut Transform2D, With<CardReader>>,
    mut card_transforms: Query<&mut Transform2D, Without<CardReader>>,
    readers: Query<&CardReader>,
    mut commands: ResMut<EventBus<PhysicsCommand>>,
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
    commands.push(PhysicsCommand::SetBodyPosition {
        entity: info.entity,
        position: target,
    });
    // Suppress collision while dragging so the kinematic velocity (inferred from
    // position delta) cannot launch dynamic card bodies.
    commands.push(PhysicsCommand::SetCollisionGroup {
        entity: info.entity,
        membership: 0,
        filter: 0,
    });
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

pub fn reader_release_system(
    mouse: Res<MouseState>,
    mut reader_drag: ResMut<ReaderDragState>,
    mut commands: ResMut<EventBus<PhysicsCommand>>,
) {
    let Some(info) = &reader_drag.dragging else {
        return;
    };
    if !mouse.pressed(MouseButton::Left) {
        // Restore collision so the placed reader blocks cards normally.
        commands.push(PhysicsCommand::SetCollisionGroup {
            entity: info.entity,
            membership: READER_COLLISION_GROUP,
            filter: READER_COLLISION_FILTER,
        });
        reader_drag.dragging = None;
    }
}
// EVOLVE-BLOCK-END
