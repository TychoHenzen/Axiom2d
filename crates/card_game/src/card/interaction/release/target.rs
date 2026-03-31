use bevy_ecs::prelude::{Res, ResMut};
use engine_core::prelude::EventBus;
use engine_input::prelude::{MouseButton, MouseState};
use engine_render::prelude::RendererRes;
use glam::Vec2;

use crate::card::component::CardZone;
use crate::card::interaction::drag_state::DragState;
use crate::card::rendering::drop_zone_glow::HAND_DROP_ZONE_HEIGHT;
use crate::stash::grid::StashGrid;
use crate::stash::grid::find_stash_slot_at;

use super::apply::CardDropIntent;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum DropTarget {
    Stash { page: u8, col: u8, row: u8 },
    Hand,
    Table,
    TableSnapBack,
}

fn is_hand_drop_zone(screen_y: f32, viewport_height: f32) -> bool {
    screen_y >= viewport_height - HAND_DROP_ZONE_HEIGHT
}

fn resolve_drop_target(
    screen_pos: Vec2,
    viewport_height: f32,
    stash_visible: bool,
    grid: &StashGrid,
    origin_zone: &CardZone,
) -> DropTarget {
    if stash_visible
        && let Some((col, row)) = find_stash_slot_at(screen_pos, grid.width(), grid.height())
    {
        let page = grid.current_page();
        if grid.get(page, col, row).is_none() {
            return DropTarget::Stash { page, col, row };
        }
        if let CardZone::Stash {
            page: op,
            col: oc,
            row: orow,
        } = *origin_zone
            && grid.get(op, oc, orow).is_none()
        {
            return DropTarget::Stash {
                page: op,
                col: oc,
                row: orow,
            };
        }
        return DropTarget::TableSnapBack;
    }

    if viewport_height > 0.0 && is_hand_drop_zone(screen_pos.y, viewport_height) {
        DropTarget::Hand
    } else {
        DropTarget::Table
    }
}

pub fn card_drop_intent_system(
    mouse: Res<MouseState>,
    drag_state: Res<DragState>,
    stash_visible: Res<crate::stash::toggle::StashVisible>,
    grid: Res<StashGrid>,
    renderer: Res<RendererRes>,
    mut intents: ResMut<EventBus<CardDropIntent>>,
) {
    let Some(info) = drag_state.dragging else {
        return;
    };
    if !mouse.just_released(MouseButton::Left) {
        return;
    }

    let screen_pos = mouse.screen_pos();
    let (_, vh) = renderer.viewport_size();
    let vh = vh as f32;

    let target = resolve_drop_target(screen_pos, vh, stash_visible.0, &grid, &info.origin_zone);

    intents.push(CardDropIntent {
        entity: info.entity,
        target,
        origin_position: info.origin_position,
    });
}
