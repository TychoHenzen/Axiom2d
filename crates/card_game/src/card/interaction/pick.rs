use bevy_ecs::prelude::{Commands, Entity, Query, Res};
use engine_input::prelude::{MouseButton, MouseState};
use engine_physics::prelude::Collider;
use engine_scene::prelude::{GlobalTransform2D, SortOrder};

use crate::card::component::Card;
use crate::card::component::CardZone;
use crate::card::interaction::game_state_param::CardGameState;
use crate::card::reader::ReaderDragState;

mod apply;
mod hit_test;
mod source;

pub const CARD_COLLISION_GROUP: u32 = 0b0001;
pub const CARD_COLLISION_FILTER: u32 = 0b0010;
pub(crate) const DRAGGED_COLLISION_GROUP: u32 = 0;
pub(crate) const DRAGGED_COLLISION_FILTER: u32 = 0;
pub const DRAG_SCALE: f32 = 1.05;

pub(crate) use source::PickSource;

pub fn card_pick_system(
    mouse: Res<MouseState>,
    mut state: CardGameState,
    reader_drag: Res<ReaderDragState>,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &Card,
        &CardZone,
        &GlobalTransform2D,
        &Collider,
        &SortOrder,
    )>,
) {
    if state.drag_state.dragging.is_some() || reader_drag.dragging.is_some() {
        return;
    }
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(source) =
        source::identify_pick_source(&mouse, &state.stash_visible, &mut state.grid, &query)
    else {
        return;
    };

    match source {
        PickSource::Stash {
            entity,
            page,
            col,
            row,
        } => {
            state.grid.take(page, col, row);
            apply::pick_from_stash(entity, page, col, row, &mut state.drag_state, &mut commands);
        }
        PickSource::Card {
            entity,
            zone,
            collider,
            grab_offset,
        } => {
            apply::pick_from_card(
                entity,
                zone,
                collider,
                grab_offset,
                &mut state.hand,
                &mut state.physics,
                &mut state.drag_state,
                &mut state.grid,
                &mut commands,
                &mut query,
            );
        }
    }
}
