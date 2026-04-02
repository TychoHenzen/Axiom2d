use bevy_ecs::prelude::{Commands, Entity, Has, Query, Res};
use engine_input::prelude::{MouseButton, MouseState};
use engine_physics::hit_test::{collider_half_extents, local_space_hit};
use engine_physics::prelude::Collider;
use engine_scene::prelude::{GlobalTransform2D, SortOrder};

use crate::card::component::Card;
use crate::card::component::CardZone;
use crate::card::interaction::drag_state::DragState;
use crate::card::interaction::flip_animation::FlipAnimation;

#[allow(clippy::type_complexity)]
pub fn card_flip_system(
    mut commands: Commands,
    mouse: Res<MouseState>,
    drag_state: Res<DragState>,
    query: Query<(
        Entity,
        &Card,
        &CardZone,
        &GlobalTransform2D,
        &Collider,
        &SortOrder,
        Has<FlipAnimation>,
    )>,
) {
    if drag_state.dragging.is_some() {
        return;
    }
    if !mouse.just_pressed(MouseButton::Right) {
        return;
    }

    let cursor = mouse.world_pos();

    let best = query
        .iter()
        .filter(|(_, _, zone, _, _, _, has_anim)| **zone == CardZone::Table && !has_anim)
        .filter(|(_, _, _, transform, collider, _, _)| {
            let Some(half) = collider_half_extents(collider) else {
                return false;
            };
            let cursor_local = transform.0.inverse().transform_point2(cursor);
            local_space_hit(cursor_local, half)
        })
        .max_by_key(|(_, _, _, _, _, sort, _)| sort.value())
        .map(|(entity, card, _, _, _, _, _)| (entity, card.face_up));

    if let Some((entity, face_up)) = best {
        commands
            .entity(entity)
            .insert(FlipAnimation::start(!face_up));
    }
}
