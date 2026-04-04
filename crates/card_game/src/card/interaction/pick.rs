use bevy_ecs::prelude::{Entity, Query, Res, ResMut};
use engine_core::prelude::EventBus;
use engine_input::prelude::{MouseButton, MouseState};
use engine_physics::prelude::Collider;
use engine_scene::prelude::{GlobalTransform2D, SortOrder};

use crate::card::component::Card;
use crate::card::component::CardZone;
use crate::card::interaction::game_state_param::CardGameState;
use crate::card::interaction::intent::InteractionIntent;
use crate::card::reader::ReaderDragState;
use crate::card::screen_device::ScreenDragState;

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
    screen_drag: Option<Res<ScreenDragState>>,
    mut intents: ResMut<EventBus<InteractionIntent>>,
    query: Query<(
        Entity,
        &Card,
        &CardZone,
        &GlobalTransform2D,
        &Collider,
        &SortOrder,
    )>,
) {
    if state.drag_state.dragging.is_some()
        || reader_drag.dragging.is_some()
        || screen_drag.is_some_and(|drag| drag.dragging.is_some())
    {
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
            intents.push(InteractionIntent::PickFromStash {
                entity,
                page,
                col,
                row,
            });
        }
        PickSource::Card {
            entity,
            zone,
            collider,
            grab_offset,
        } => {
            intents.push(InteractionIntent::PickCard {
                entity,
                zone,
                collider,
                grab_offset,
            });
        }
    }
}
