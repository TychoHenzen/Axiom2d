use bevy_ecs::prelude::{Commands, Query, Res, ResMut};
use engine_input::prelude::MouseState;
use engine_physics::prelude::{Collider, PhysicsRes, RigidBody};
use engine_scene::prelude::{GlobalTransform2D, RenderLayer};

use crate::card::component::CardItemForm;
use crate::card::interaction::drag_state::DragState;
use crate::card::interaction::physics_helpers::{activate_physics_body, warn_on_physics_result};
use crate::card::interaction::pick::{
    DRAG_SCALE, DRAGGED_COLLISION_FILTER, DRAGGED_COLLISION_GROUP,
};
use crate::card::rendering::geometry::TABLE_CARD_WIDTH as CARD_WIDTH;
use crate::stash::constants::SLOT_WIDTH;
use crate::stash::grid::{StashGrid, find_stash_slot_at};
use crate::stash::toggle::StashVisible;
use engine_core::scale_spring::ScaleSpring;

pub fn stash_boundary_system(
    mouse: Res<MouseState>,
    mut drag_state: ResMut<DragState>,
    mut physics: ResMut<PhysicsRes>,
    stash_visible: Res<StashVisible>,
    grid: Res<StashGrid>,
    mut commands: Commands,
    transform_query: Query<(&GlobalTransform2D, &Collider)>,
) {
    let Some(info) = drag_state.dragging else {
        return;
    };

    let over_stash = stash_visible.0
        && !grid.is_store_page()
        && find_stash_slot_at(mouse.screen_pos(), grid.width(), grid.height()).is_some();

    if info.stash_cursor_follow && !over_stash {
        // Exit stash: add physics body, switch to spring drag
        if let Ok((transform, collider)) = transform_query.get(info.entity) {
            activate_physics_body(
                info.entity,
                transform.0.translation,
                collider,
                &mut physics,
                DRAGGED_COLLISION_GROUP,
                DRAGGED_COLLISION_FILTER,
            );
        }
        commands
            .entity(info.entity)
            .insert(RigidBody::Dynamic)
            .insert(RenderLayer::World)
            .remove::<CardItemForm>()
            .insert(ScaleSpring::new(DRAG_SCALE));
        drag_state.dragging = Some(crate::card::interaction::drag_state::DragInfo {
            stash_cursor_follow: false,
            ..info
        });
    } else if !info.stash_cursor_follow && over_stash {
        // Enter stash: remove physics body, switch to cursor-follow
        warn_on_physics_result("remove_body", info.entity, physics.remove_body(info.entity));
        commands
            .entity(info.entity)
            .remove::<RigidBody>()
            .insert(RenderLayer::UI)
            .insert(CardItemForm)
            .insert(ScaleSpring::new(SLOT_WIDTH / CARD_WIDTH));
        drag_state.dragging = Some(crate::card::interaction::drag_state::DragInfo {
            stash_cursor_follow: true,
            ..info
        });
    }
}
