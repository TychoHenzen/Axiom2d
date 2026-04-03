use bevy_ecs::prelude::{Entity, Query, Res, ResMut};
use engine_core::prelude::Transform2D;
use engine_input::mouse_button::MouseButton;
use engine_input::prelude::MouseState;

use crate::card::interaction::drag_state::DragState;
use crate::card::jack_socket::PendingCable;
use crate::card::reader::components::{CardReader, ReaderDragInfo, ReaderDragState};
use crate::card::screen_device::ScreenDragState;

pub fn reader_pick_system(
    mouse: Res<MouseState>,
    drag_state: Res<DragState>,
    mut reader_drag: ResMut<ReaderDragState>,
    screen_drag: Res<ScreenDragState>,
    pending: Res<PendingCable>,
    readers: Query<(Entity, &Transform2D, &CardReader)>,
) {
    if drag_state.dragging.is_some()
        || reader_drag.dragging.is_some()
        || screen_drag.dragging.is_some()
        || pending.source.is_some()
    {
        return;
    }
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    let cursor = mouse.world_pos();
    for (entity, transform, reader) in &readers {
        let delta = (cursor - transform.position).abs();
        if delta.x <= reader.half_extents.x && delta.y <= reader.half_extents.y {
            reader_drag.dragging = Some(ReaderDragInfo {
                entity,
                grab_offset: cursor - transform.position,
            });
            return;
        }
    }
}
