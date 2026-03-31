use bevy_ecs::prelude::{Query, Res, ResMut};
use engine_core::prelude::{EventBus, Transform2D};
use engine_input::prelude::MouseState;

use crate::card::interaction::drag_state::DragState;
use crate::card::reader::components::{
    CardReader, ReaderDragInfo, ReaderDragState, ReaderPickIntent,
};

pub fn reader_pick_intent_system(
    mouse: Res<MouseState>,
    drag_state: Res<DragState>,
    reader_drag: Res<ReaderDragState>,
    readers: Query<(bevy_ecs::prelude::Entity, &Transform2D, &CardReader)>,
    mut intents: ResMut<EventBus<ReaderPickIntent>>,
) {
    use engine_input::mouse_button::MouseButton;

    if drag_state.dragging.is_some() || reader_drag.dragging.is_some() {
        return;
    }
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    let cursor = mouse.world_pos();
    for (entity, transform, reader) in &readers {
        let delta = (cursor - transform.position).abs();
        if delta.x <= reader.half_extents.x && delta.y <= reader.half_extents.y {
            intents.push(ReaderPickIntent {
                entity,
                grab_offset: cursor - transform.position,
            });
            return;
        }
    }
}

pub fn apply_reader_pick_intents_system(
    drag_state: Res<DragState>,
    mut reader_drag: ResMut<ReaderDragState>,
    mut intents: ResMut<EventBus<ReaderPickIntent>>,
) {
    if drag_state.dragging.is_some() || reader_drag.dragging.is_some() {
        intents.drain();
        return;
    }

    if let Some(intent) = intents.drain().next() {
        reader_drag.dragging = Some(ReaderDragInfo {
            entity: intent.entity,
            grab_offset: intent.grab_offset,
        });
    }
}
