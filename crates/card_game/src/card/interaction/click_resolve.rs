use bevy_ecs::prelude::{Commands, Component, Entity, Query, Res, ResMut, Trigger};
use engine_core::prelude::EventBus;
use engine_input::prelude::{MouseButton, MouseState};
use engine_physics::hit_test::{collider_half_extents, local_space_hit};
use engine_physics::prelude::Collider;
use engine_scene::prelude::{GlobalTransform2D, SortOrder};
use glam::Vec2;

use crate::card::component::CardZone;
use crate::card::interaction::drag_state::DragState;
use crate::card::interaction::intent::InteractionIntent;
use crate::card::jack_socket::PendingCable;
use crate::card::reader::ReaderDragState;
use crate::card::screen_device::ScreenDragState;
use crate::stash::grid::{StashGrid, find_stash_slot_at};
use crate::stash::toggle::StashVisible;

/// Hit shape for a `Clickable` entity.
#[derive(Clone)]
pub enum ClickHitShape {
    Aabb(Vec2),
    Circle(f32),
}

/// Add this component to any entity that should receive `ClickedEntity` triggers.
#[derive(Component)]
pub struct Clickable(pub ClickHitShape);

/// Trigger event delivered to the topmost clicked entity.
#[derive(bevy_ecs::prelude::Event)]
pub struct ClickedEntity {
    pub world_cursor: Vec2,
}

/// Raycasts all `Clickable` entities, picks the topmost by `SortOrder`,
/// and delivers a `ClickedEntity` observer trigger to that entity.
///
/// Runs in `Phase::Input`.
#[allow(clippy::too_many_arguments)]
pub fn click_resolve_system(
    mouse: Res<MouseState>,
    drag_state: Res<DragState>,
    reader_drag: Res<ReaderDragState>,
    screen_drag: Option<Res<ScreenDragState>>,
    pending: Res<PendingCable>,
    stash_visible: Option<Res<StashVisible>>,
    grid: Option<Res<StashGrid>>,
    mut intents: ResMut<EventBus<InteractionIntent>>,
    mut commands: Commands,
    query: Query<(Entity, &Clickable, &GlobalTransform2D, &SortOrder)>,
) {
    // Guard: any active drag/cable suppresses new click resolution
    if drag_state.dragging.is_some()
        || reader_drag.dragging.is_some()
        || screen_drag.is_some_and(|d| d.dragging.is_some())
        || pending.source.is_some()
    {
        return;
    }

    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    // Stash UI check (screen-space) — handled before world raycast
    if let (Some(stash_visible), Some(grid)) = (stash_visible.as_deref(), grid.as_deref()) {
        if stash_visible.0 && crate::stash::pages::stash_ui_contains(mouse.screen_pos(), grid) {
            if !grid.is_store_page() {
                if let Some((col, row)) =
                    find_stash_slot_at(mouse.screen_pos(), grid.width(), grid.height())
                {
                    let page = grid.current_storage_page().unwrap_or(0);
                    if let Some(entity) = grid.get(page, col, row) {
                        intents.push(InteractionIntent::PickFromStash {
                            entity: *entity,
                            page,
                            col,
                            row,
                        });
                    }
                }
            }
            return;
        }
    }

    // World raycast: find topmost Clickable under cursor
    let cursor = mouse.world_pos();
    let hit = query
        .iter()
        .filter(|(_, clickable, global, _)| hit_test(cursor, clickable, global))
        .max_by_key(|(_, _, _, sort)| sort.value());

    if let Some((entity, _, _, _)) = hit {
        commands.trigger_targets(
            ClickedEntity {
                world_cursor: cursor,
            },
            entity,
        );
    }
}

fn hit_test(cursor: Vec2, clickable: &Clickable, global: &GlobalTransform2D) -> bool {
    match &clickable.0 {
        ClickHitShape::Aabb(half) => {
            let cursor_local = global.0.inverse().transform_point2(cursor);
            local_space_hit(cursor_local, *half)
        }
        ClickHitShape::Circle(radius) => {
            let cursor_local = global.0.inverse().transform_point2(cursor);
            cursor_local.length() <= *radius
        }
    }
}

/// Observer registered on each card entity at spawn time.
///
/// Converts a `ClickedEntity` trigger into a `PickCard` intent.
pub fn on_card_clicked(
    trigger: Trigger<ClickedEntity>,
    query: Query<(&CardZone, &GlobalTransform2D, &Collider)>,
    mut intents: ResMut<EventBus<InteractionIntent>>,
) {
    let entity = trigger.target();
    let cursor = trigger.event().world_cursor;

    let Ok((zone, global, collider)) = query.get(entity) else {
        return;
    };

    let grab_offset = global
        .0
        .matrix2
        .inverse()
        .mul_vec2(cursor - global.0.translation);

    intents.push(InteractionIntent::PickCard {
        entity,
        zone: *zone,
        collider: collider.clone(),
        grab_offset,
    });
}

/// Convenience: compute the `ClickHitShape::Aabb` for a card from its `Collider`.
pub fn aabb_hit_shape_from_collider(collider: &Collider) -> Option<ClickHitShape> {
    collider_half_extents(collider).map(ClickHitShape::Aabb)
}
