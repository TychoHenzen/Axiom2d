use bevy_ecs::prelude::{Query, ResMut, Trigger, With};
use engine_core::prelude::Transform2D;

use crate::card::interaction::click_resolve::ClickedEntity;
use crate::card::interaction::drag_state::DeviceDragInfo;
use crate::card::reader::components::{CardReader, ReaderDragState};

/// Observer registered on each `CardReader` entity at spawn time.
pub fn on_reader_clicked(
    trigger: Trigger<ClickedEntity>,
    readers: Query<&Transform2D, With<CardReader>>,
    mut reader_drag: ResMut<ReaderDragState>,
) {
    let entity = trigger.target();
    let cursor = trigger.event().world_cursor;
    let Ok(transform) = readers.get(entity) else {
        return;
    };
    reader_drag.dragging = Some(DeviceDragInfo {
        entity,
        grab_offset: cursor - transform.position,
    });
}
