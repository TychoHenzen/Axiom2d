use bevy_ecs::prelude::{Commands, Query, Res};
use engine_core::prelude::Transform2D;
use engine_input::prelude::{MouseButton, MouseState};
use engine_physics::prelude::Collider;
use engine_render::prelude::RendererRes;
use glam::Vec2;

use crate::card::component::{Card, CardZone};
use crate::card::interaction::game_state_param::CardGameState;
use crate::card::rendering::drop_zone_glow::HAND_DROP_ZONE_HEIGHT;
use crate::stash::grid::StashGrid;
use crate::stash::grid::find_stash_slot_at;

use super::apply::{drop_on_hand, drop_on_stash, drop_on_table};

#[derive(Debug, Clone, Copy, PartialEq)]
enum DropTarget {
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
    if stash_visible {
        if grid.is_store_page() && crate::stash::pages::stash_ui_contains(screen_pos, grid) {
            return DropTarget::TableSnapBack;
        }
        if !grid.is_store_page()
            && let Some((col, row)) = find_stash_slot_at(screen_pos, grid.width(), grid.height())
        {
            let page = grid.current_storage_page().unwrap_or(0);
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
    }

    if viewport_height > 0.0 && is_hand_drop_zone(screen_pos.y, viewport_height) {
        DropTarget::Hand
    } else {
        DropTarget::Table
    }
}

pub fn card_release_system(
    mouse: Res<MouseState>,
    mut state: CardGameState,
    renderer: Res<RendererRes>,
    mut commands: Commands,
    transform_query: Query<(&Transform2D, &Collider)>,
    card_query: Query<&Card>,
) {
    let Some(info) = state.drag_state.dragging else {
        return;
    };
    if !mouse.just_released(MouseButton::Left) {
        return;
    }

    let screen_pos = mouse.screen_pos();
    let (_, vh) = renderer.viewport_size();
    let vh = vh as f32;

    let target = resolve_drop_target(
        screen_pos,
        vh,
        state.stash_visible.0,
        &state.grid,
        &info.origin_zone,
    );

    match target {
        DropTarget::Stash { page, col, row } => {
            let current_pos = transform_query
                .get(info.entity)
                .ok()
                .map(|(t, _)| t.position);
            drop_on_stash(
                info.entity,
                page,
                col,
                row,
                current_pos,
                &mut state.grid,
                &mut *state.physics_commands,
                &mut commands,
            );
        }
        DropTarget::Hand => {
            let face_up = card_query.get(info.entity).is_ok_and(|c| c.face_up);
            drop_on_hand(
                info.entity,
                face_up,
                info.origin_position,
                &mut state.hand,
                &mut *state.physics_commands,
                &mut commands,
            );
        }
        DropTarget::Table => {
            drop_on_table(
                info.entity,
                None,
                &mut *state.physics_commands,
                &mut commands,
                &transform_query,
            );
        }
        DropTarget::TableSnapBack => {
            drop_on_table(
                info.entity,
                Some(info.origin_position),
                &mut *state.physics_commands,
                &mut commands,
                &transform_query,
            );
        }
    }

    state.drag_state.dragging = None;
}
