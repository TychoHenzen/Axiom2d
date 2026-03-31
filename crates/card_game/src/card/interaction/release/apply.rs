use bevy_ecs::prelude::{Commands, Entity, Query, ResMut};
use engine_core::prelude::{Event, EventBus, Transform2D};
use engine_physics::prelude::{Collider, PhysicsRes, RigidBody};
use engine_scene::prelude::RenderLayer;
use glam::Vec2;

use super::target::DropTarget;
use crate::card::component::Card;
use crate::card::component::CardItemForm;
use crate::card::component::CardZone;
use crate::card::interaction::flip_animation::FlipAnimation;
use crate::card::interaction::game_state_param::CardGameState;
use crate::card::interaction::physics_helpers::{activate_physics_body, warn_on_physics_result};
use crate::card::interaction::pick::{CARD_COLLISION_FILTER, CARD_COLLISION_GROUP};
use crate::hand::cards::Hand;
use crate::hand::layout::HandSpring;
use crate::stash::grid::StashGrid;
use engine_core::scale_spring::ScaleSpring;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CardDropIntent {
    pub(crate) entity: Entity,
    pub(crate) target: DropTarget,
    pub(crate) origin_position: Vec2,
}

impl Event for CardDropIntent {}

pub fn apply_card_drop_intents_system(
    mut intents: ResMut<EventBus<CardDropIntent>>,
    mut state: CardGameState,
    mut commands: Commands,
    transform_query: Query<(&Transform2D, &Collider)>,
    card_query: Query<&Card>,
) {
    for intent in intents.drain() {
        let Some(info) = state.drag_state.dragging else {
            continue;
        };
        if info.entity != intent.entity {
            continue;
        }

        match intent.target {
            DropTarget::Stash { page, col, row } => {
                let current_pos = transform_query
                    .get(intent.entity)
                    .ok()
                    .map(|(t, _)| t.position);
                drop_on_stash(
                    intent.entity,
                    page,
                    col,
                    row,
                    current_pos,
                    &mut state.grid,
                    &mut state.physics,
                    &mut commands,
                );
            }
            DropTarget::Hand => {
                let face_up = card_query.get(intent.entity).is_ok_and(|c| c.face_up);
                drop_on_hand(
                    intent.entity,
                    face_up,
                    intent.origin_position,
                    &mut state.hand,
                    &mut state.physics,
                    &mut commands,
                );
            }
            DropTarget::Table => {
                drop_on_table(
                    intent.entity,
                    None,
                    &mut state.physics,
                    &mut commands,
                    &transform_query,
                );
            }
            DropTarget::TableSnapBack => {
                drop_on_table(
                    intent.entity,
                    Some(intent.origin_position),
                    &mut state.physics,
                    &mut commands,
                    &transform_query,
                );
            }
        }

        state.drag_state.dragging = None;
    }
}

fn drop_on_hand(
    entity: Entity,
    face_up: bool,
    origin_position: Vec2,
    hand: &mut Hand,
    physics: &mut PhysicsRes,
    commands: &mut Commands,
) {
    warn_on_physics_result("remove_body", entity, physics.remove_body(entity));
    let zone = if let Ok(index) = hand.add(entity) {
        CardZone::Hand(index)
    } else {
        commands.entity(entity).insert(Transform2D {
            position: origin_position,
            rotation: 0.0,
            scale: Vec2::ONE,
        });
        CardZone::Table
    };
    let mut ec = commands.entity(entity);
    ec.remove::<RigidBody>()
        .insert(zone)
        .insert(RenderLayer::UI)
        .remove::<CardItemForm>()
        .insert(HandSpring::new());
    if !face_up {
        ec.insert(FlipAnimation::start(true));
    }
}

fn drop_on_stash(
    entity: Entity,
    page: u8,
    col: u8,
    row: u8,
    current_pos: Option<Vec2>,
    grid: &mut StashGrid,
    physics: &mut PhysicsRes,
    commands: &mut Commands,
) {
    warn_on_physics_result("remove_body", entity, physics.remove_body(entity));
    grid.place(page, col, row, entity)
        .expect("slot should be empty: guarded by is_none check above");
    let mut ec = commands.entity(entity);
    ec.remove::<RigidBody>()
        .insert(CardZone::Stash { page, col, row })
        .insert(RenderLayer::UI)
        .insert(CardItemForm);
    if let Some(pos) = current_pos {
        ec.insert(Transform2D {
            position: pos,
            rotation: 0.0,
            scale: Vec2::ONE,
        });
    }
}

fn drop_on_table(
    entity: Entity,
    snap_back: Option<Vec2>,
    physics: &mut PhysicsRes,
    commands: &mut Commands,
    transform_query: &Query<(&Transform2D, &Collider)>,
) {
    let position = if let Some(origin) = snap_back {
        origin
    } else {
        transform_query
            .get(entity)
            .map(|(t, _)| t.position)
            .unwrap_or(Vec2::ZERO)
    };

    if let Ok((_, collider)) = transform_query.get(entity) {
        activate_physics_body(
            entity,
            position,
            collider,
            physics,
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
