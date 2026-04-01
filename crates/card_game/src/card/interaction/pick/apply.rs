use bevy_ecs::prelude::{Commands, Entity, Query};
use engine_physics::prelude::{Collider, PhysicsRes, RigidBody};
use engine_scene::prelude::{GlobalTransform2D, LocalSortOrder, RenderLayer, SortOrder};
use glam::Vec2;

use crate::card::component::Card;
use crate::card::component::CardItemForm;
use crate::card::component::CardZone;
use crate::card::interaction::drag_state::{DragInfo, DragState};
use crate::card::interaction::physics_helpers::activate_physics_body;
use crate::hand::cards::Hand;
use crate::hand::layout::HandSpring;
use engine_core::scale_spring::ScaleSpring;

use super::{DRAG_SCALE, DRAGGED_COLLISION_FILTER, DRAGGED_COLLISION_GROUP};

pub(super) fn pick_from_stash(
    entity: Entity,
    page: u8,
    col: u8,
    row: u8,
    drag_state: &mut DragState,
    commands: &mut Commands,
) {
    commands.entity(entity).insert(CardZone::Table);
    commands.entity(entity).remove::<CardItemForm>();
    commands.entity(entity).insert(ScaleSpring::new(DRAG_SCALE));
    drag_state.dragging = Some(DragInfo {
        entity,
        local_grab_offset: Vec2::ZERO,
        origin_zone: CardZone::Stash { page, col, row },
        stash_cursor_follow: true,
        origin_position: Vec2::ZERO,
    });
}

pub(super) fn pick_from_card(
    entity: Entity,
    zone: CardZone,
    collider: Collider,
    grab_offset: Vec2,
    hand: &mut Hand,
    physics: &mut PhysicsRes,
    drag_state: &mut DragState,
    grid: &mut crate::stash::grid::StashGrid,
    commands: &mut Commands,
    query: &mut Query<(
        Entity,
        &Card,
        &CardZone,
        &GlobalTransform2D,
        &Collider,
        &SortOrder,
    )>,
) {
    let max_sort = max_table_sort_order(query);

    if let CardZone::Hand(_) = zone {
        transition_hand_to_table(entity, hand, physics, commands, query, &collider);
    }

    if let CardZone::Stash { page, col, row } = zone {
        grid.take(page, col, row);
        commands.entity(entity).insert(CardZone::Table);
        commands.entity(entity).remove::<CardItemForm>();
    }

    if matches!(zone, CardZone::Table) {
        physics
            .set_collision_group(entity, DRAGGED_COLLISION_GROUP, DRAGGED_COLLISION_FILTER)
            .expect("picked entity should have physics body");
    }

    let origin_position = query
        .get(entity)
        .map(|(_, _, _, t, _, _)| t.0.translation)
        .unwrap_or(Vec2::ZERO);
    drag_state.dragging = Some(DragInfo {
        entity,
        local_grab_offset: grab_offset,
        origin_zone: zone,
        stash_cursor_follow: false,
        origin_position,
    });
    commands.entity(entity).insert(LocalSortOrder(max_sort + 1));
    commands.entity(entity).insert(ScaleSpring::new(DRAG_SCALE));
}

pub(super) fn max_table_sort_order(
    query: &Query<(
        Entity,
        &Card,
        &CardZone,
        &GlobalTransform2D,
        &Collider,
        &SortOrder,
    )>,
) -> i32 {
    query
        .iter()
        .filter(|(_, _, zone, _, _, _)| **zone == CardZone::Table)
        .map(|(_, _, _, _, _, sort)| sort.value())
        .max()
        .unwrap_or(0)
}

pub(super) fn transition_hand_to_table(
    entity: Entity,
    hand: &mut Hand,
    physics: &mut PhysicsRes,
    commands: &mut Commands,
    query: &Query<(
        Entity,
        &Card,
        &CardZone,
        &GlobalTransform2D,
        &Collider,
        &SortOrder,
    )>,
    collider: &Collider,
) {
    hand.remove(entity);
    let position = query
        .get(entity)
        .map(|(_, _, _, t, _, _)| t.0.translation)
        .unwrap_or(Vec2::ZERO);
    activate_physics_body(
        entity,
        position,
        collider,
        physics,
        DRAGGED_COLLISION_GROUP,
        DRAGGED_COLLISION_FILTER,
    );
    commands.entity(entity).insert(RigidBody::Dynamic);
    commands.entity(entity).insert(RenderLayer::World);
    commands.entity(entity).remove::<HandSpring>();
    commands.entity(entity).insert(ScaleSpring::new(1.0));
}
