use bevy_ecs::prelude::{Entity, Query};
use engine_input::prelude::MouseState;
use engine_physics::prelude::Collider;
use engine_scene::prelude::{GlobalTransform2D, SortOrder};
use glam::Vec2;

use crate::card::component::Card;
use crate::card::component::CardZone;
use crate::stash::grid::{StashGrid, find_stash_slot_at};
use crate::stash::toggle::StashVisible;

use super::hit_test::find_card_under_cursor;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum PickSource {
    Stash {
        entity: Entity,
        page: u8,
        col: u8,
        row: u8,
    },
    Card {
        entity: Entity,
        zone: CardZone,
        collider: Collider,
        grab_offset: Vec2,
    },
}

pub(crate) fn identify_pick_source(
    mouse: &MouseState,
    stash_visible: &StashVisible,
    grid: &mut StashGrid,
    query: &Query<(
        Entity,
        &Card,
        &CardZone,
        &GlobalTransform2D,
        &Collider,
        &SortOrder,
    )>,
) -> Option<PickSource> {
    if stash_visible.0 {
        let screen = mouse.screen_pos();
        if let Some((col, row)) = find_stash_slot_at(screen, grid.width(), grid.height()) {
            let page = grid.current_page();
            if let Some(entity) = grid.get(page, col, row) {
                return Some(PickSource::Stash {
                    entity: *entity,
                    page,
                    col,
                    row,
                });
            }
        }
    }

    let cursor = mouse.world_pos();
    find_card_under_cursor(query, cursor).map(|(entity, zone, grab_offset, collider)| {
        PickSource::Card {
            entity,
            zone,
            collider,
            grab_offset,
        }
    })
}
