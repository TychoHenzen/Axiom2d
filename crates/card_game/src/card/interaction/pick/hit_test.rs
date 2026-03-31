use bevy_ecs::prelude::{Entity, Query};
use engine_physics::hit_test::{collider_half_extents, local_space_hit};
use engine_physics::prelude::Collider;
use engine_scene::prelude::{GlobalTransform2D, SortOrder};
use glam::Vec2;

use crate::card::component::Card;
use crate::card::component::CardZone;

pub(crate) fn find_card_under_cursor(
    query: &Query<(
        Entity,
        &Card,
        &CardZone,
        &GlobalTransform2D,
        &Collider,
        &SortOrder,
    )>,
    cursor: Vec2,
) -> Option<(Entity, CardZone, Vec2, Collider)> {
    query
        .iter()
        .filter(|(_, _, _, transform, collider, _)| {
            let Some(half) = collider_half_extents(collider) else {
                return false;
            };
            let cursor_local = transform.0.inverse().transform_point2(cursor);
            local_space_hit(cursor_local, half)
        })
        .max_by_key(|(_, _, _, _, _, sort)| sort.value())
        .map(|(entity, _, zone, transform, collider, _)| {
            let cursor_delta = cursor - transform.0.translation;
            let local_grab_offset = transform.0.matrix2.inverse().mul_vec2(cursor_delta);
            (entity, *zone, local_grab_offset, collider.clone())
        })
}
