use bevy_ecs::prelude::{Commands, Entity, Query, ResMut};
use engine_core::prelude::{EventBus, Transform2D};
use engine_physics::prelude::{Collider, PhysicsCommand, RigidBody};
use engine_scene::prelude::{GlobalTransform2D, LocalSortOrder, RenderLayer, SortOrder};
use glam::Vec2;

use crate::card::component::{CardItemForm, CardZone};
use crate::card::interaction::drag_state::{DragInfo, DragState};
use crate::card::interaction::flip_animation::FlipAnimation;
use crate::card::interaction::intent::InteractionIntent;
use crate::card::interaction::physics_helpers::activate_physics_body;
use crate::card::interaction::pick::{
    CARD_COLLISION_FILTER, CARD_COLLISION_GROUP, DRAG_SCALE, DRAGGED_COLLISION_FILTER,
    DRAGGED_COLLISION_GROUP,
};
use crate::hand::cards::Hand;
use crate::hand::layout::HandSpring;
use crate::stash::grid::StashGrid;
use engine_core::scale_spring::ScaleSpring;

fn entity_position(transforms: &Query<&GlobalTransform2D>, entity: Entity) -> Vec2 {
    transforms
        .get(entity)
        .expect("picked entity must have GlobalTransform2D")
        .0
        .translation
}

fn table_sort_ceiling(sort_query: &Query<(&CardZone, &SortOrder)>) -> i32 {
    sort_query
        .iter()
        .filter_map(|(zone, sort)| (*zone == CardZone::Table).then_some(sort.value()))
        .max()
        .unwrap_or_default()
}

#[allow(clippy::too_many_arguments)]
pub fn interaction_apply_system(
    mut intents: ResMut<EventBus<InteractionIntent>>,
    mut drag_state: ResMut<DragState>,
    mut physics_commands: ResMut<EventBus<PhysicsCommand>>,
    mut hand: Option<ResMut<Hand>>,
    mut commands: Commands,
    transforms: Query<&GlobalTransform2D>,
    sort_query: Query<(&CardZone, &SortOrder)>,
    mut grid: Option<ResMut<StashGrid>>,
    transform_collider_query: Query<(&Transform2D, &Collider)>,
    rigid_body_query: Query<&RigidBody>,
) {
    for intent in intents.drain() {
        match intent {
            InteractionIntent::PickCard {
                entity,
                zone,
                collider,
                grab_offset,
            } => {
                let has_rigid_body = rigid_body_query.get(entity).is_ok();
                apply_pick_card(
                    entity,
                    zone,
                    &collider,
                    grab_offset,
                    has_rigid_body,
                    &mut drag_state,
                    &mut physics_commands,
                    &mut hand,
                    &mut commands,
                    &transforms,
                    &sort_query,
                );
            }
            InteractionIntent::PickFromStash {
                entity,
                page,
                col,
                row,
            } => {
                if let Some(grid) = &mut grid {
                    grid.take(page, col, row);
                }
                commands
                    .entity(entity)
                    .insert(CardZone::Table)
                    .remove::<CardItemForm>()
                    .insert(ScaleSpring::new(DRAG_SCALE));
                drag_state.dragging = Some(DragInfo {
                    entity,
                    local_grab_offset: Vec2::ZERO,
                    origin_zone: CardZone::Stash { page, col, row },
                    stash_cursor_follow: true,
                    origin_position: Vec2::ZERO,
                });
            }
            InteractionIntent::ReleaseOnTable { entity, snap_back } => {
                let origin_position = drag_state
                    .dragging
                    .map_or(Vec2::ZERO, |d| d.origin_position);
                apply_release_on_table(
                    entity,
                    if snap_back {
                        Some(origin_position)
                    } else {
                        None
                    },
                    &mut physics_commands,
                    &mut commands,
                    &transform_collider_query,
                );
                drag_state.dragging = None;
            }
            InteractionIntent::ReleaseOnHand {
                entity,
                face_up,
                origin_position: position,
            } => {
                apply_release_on_hand(
                    entity,
                    face_up,
                    position,
                    &mut hand,
                    &mut physics_commands,
                    &mut commands,
                );
                drag_state.dragging = None;
            }
            InteractionIntent::ReleaseOnStash {
                entity,
                page,
                col,
                row,
                current_position,
            } => {
                apply_release_on_stash(
                    entity,
                    page,
                    col,
                    row,
                    current_position,
                    &mut grid,
                    &mut physics_commands,
                    &mut commands,
                );
                drag_state.dragging = None;
            }
            InteractionIntent::OpenBoosterPack { .. } => {
                // handled by booster opening system
            }
        }
    }
}

fn apply_pick_card(
    entity: Entity,
    zone: CardZone,
    collider: &Collider,
    grab_offset: Vec2,
    has_rigid_body: bool,
    drag_state: &mut DragState,
    physics_commands: &mut EventBus<PhysicsCommand>,
    hand: &mut Option<ResMut<Hand>>,
    commands: &mut Commands,
    transforms: &Query<&GlobalTransform2D>,
    sort_query: &Query<(&CardZone, &SortOrder)>,
) {
    let position = entity_position(transforms, entity);

    if let CardZone::Hand(_) = zone {
        if let Some(hand) = hand {
            hand.remove(entity);
        }
        activate_physics_body(
            entity,
            position,
            collider,
            physics_commands,
            DRAGGED_COLLISION_GROUP,
            DRAGGED_COLLISION_FILTER,
        );
        let mut ec = commands.entity(entity);
        ec.insert(RigidBody::Dynamic)
            .insert(RenderLayer::World)
            .remove::<HandSpring>()
            .insert(ScaleSpring::new(1.0));
    }

    if matches!(zone, CardZone::Table) {
        if has_rigid_body {
            physics_commands.push(PhysicsCommand::SetCollisionGroup {
                entity,
                membership: DRAGGED_COLLISION_GROUP,
                filter: DRAGGED_COLLISION_FILTER,
            });
        } else {
            activate_physics_body(
                entity,
                position,
                collider,
                physics_commands,
                DRAGGED_COLLISION_GROUP,
                DRAGGED_COLLISION_FILTER,
            );
            commands
                .entity(entity)
                .insert(RigidBody::Dynamic)
                .insert(ScaleSpring::new(1.0));
        }
    }

    let max_sort = table_sort_ceiling(sort_query);

    drag_state.dragging = Some(DragInfo {
        entity,
        local_grab_offset: grab_offset,
        origin_zone: zone,
        stash_cursor_follow: false,
        origin_position: position,
    });

    commands
        .entity(entity)
        .insert(LocalSortOrder(max_sort + 1))
        .insert(ScaleSpring::new(DRAG_SCALE));
}

fn apply_release_on_table(
    entity: Entity,
    snap_back: Option<Vec2>,
    physics_commands: &mut EventBus<PhysicsCommand>,
    commands: &mut Commands,
    transform_query: &Query<(&Transform2D, &Collider)>,
) {
    let (position, collider) = if let Ok((t, c)) = transform_query.get(entity) {
        (snap_back.unwrap_or(t.position), Some(c))
    } else {
        (snap_back.unwrap_or(Vec2::ZERO), None)
    };

    if let Some(collider) = collider {
        activate_physics_body(
            entity,
            position,
            collider,
            physics_commands,
            CARD_COLLISION_GROUP,
            CARD_COLLISION_FILTER,
        );
    }

    let mut ec = commands.entity(entity);
    ec.insert(RigidBody::Dynamic)
        .insert(CardZone::Table)
        .insert(RenderLayer::World)
        .remove::<CardItemForm>()
        .insert(ScaleSpring::new(1.0));
    if snap_back.is_some() {
        ec.insert(Transform2D {
            position,
            rotation: 0.0,
            scale: Vec2::ONE,
        });
    }
}

fn apply_release_on_hand(
    entity: Entity,
    face_up: bool,
    origin_position: Vec2,
    hand: &mut Option<ResMut<Hand>>,
    physics_commands: &mut EventBus<PhysicsCommand>,
    commands: &mut Commands,
) {
    physics_commands.push(PhysicsCommand::RemoveBody { entity });

    let zone = if let Some(hand) = hand {
        if let Ok(index) = hand.add(entity) {
            CardZone::Hand(index)
        } else {
            commands.entity(entity).insert(Transform2D {
                position: origin_position,
                rotation: 0.0,
                scale: Vec2::ONE,
            });
            CardZone::Table
        }
    } else {
        CardZone::Table
    };
    let is_hand = matches!(zone, CardZone::Hand(_));

    let mut ec = commands.entity(entity);
    ec.remove::<RigidBody>()
        .insert(zone)
        .remove::<CardItemForm>();
    if is_hand {
        ec.insert(RenderLayer::UI).insert(HandSpring::new());
        if !face_up {
            ec.insert(FlipAnimation::start(true));
        }
    } else {
        ec.insert(RenderLayer::World).insert(ScaleSpring::new(1.0));
    }
}

fn apply_release_on_stash(
    entity: Entity,
    page: u8,
    col: u8,
    row: u8,
    current_pos: Vec2,
    grid: &mut Option<ResMut<StashGrid>>,
    physics_commands: &mut EventBus<PhysicsCommand>,
    commands: &mut Commands,
) {
    physics_commands.push(PhysicsCommand::RemoveBody { entity });
    if let Some(grid) = grid {
        grid.place(page, col, row, entity)
            .expect("slot should be empty: guarded by resolve_drop_target");
    }
    let mut ec = commands.entity(entity);
    ec.remove::<RigidBody>()
        .insert(CardZone::Stash { page, col, row })
        .insert(RenderLayer::UI)
        .insert(CardItemForm)
        .insert(Transform2D {
            position: current_pos,
            rotation: 0.0,
            scale: Vec2::ONE,
        });
}
