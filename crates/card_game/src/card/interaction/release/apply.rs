use bevy_ecs::prelude::{Commands, Entity, Query};
use engine_core::prelude::Transform2D;
use engine_physics::prelude::{Collider, PhysicsRes, RigidBody};
use engine_scene::render_order::RenderLayer;
use glam::Vec2;

use crate::card::component::CardItemForm;
use crate::card::component::CardZone;
use crate::card::interaction::flip_animation::FlipAnimation;
use crate::card::interaction::physics_helpers::{activate_physics_body, warn_on_physics_result};
use crate::card::interaction::pick::{CARD_COLLISION_FILTER, CARD_COLLISION_GROUP};
use crate::hand::cards::Hand;
use crate::hand::layout::HandSpring;
use crate::stash::grid::StashGrid;
use engine_core::scale_spring::ScaleSpring;

pub(super) fn drop_on_hand(
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

pub(super) fn drop_on_stash(
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

pub(super) fn drop_on_table(
    entity: Entity,
    snap_back: Option<Vec2>,
    physics: &mut PhysicsRes,
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
