use bevy_ecs::prelude::{Commands, Res};
use engine_input::prelude::MouseState;

use crate::card_item_form::CardItemForm;
use crate::drag_state::DragState;
use crate::stash_grid::StashGrid;
use crate::stash_grid::find_stash_slot_at;
use crate::stash_toggle::StashVisible;

pub fn stash_drag_hover_system(
    drag_state: Res<DragState>,
    mouse: Res<MouseState>,
    stash_visible: Res<StashVisible>,
    grid: Res<StashGrid>,
    mut commands: Commands,
) {
    let Some(info) = drag_state.dragging else {
        return;
    };

    let over_stash = stash_visible.0
        && find_stash_slot_at(mouse.screen_pos(), grid.width(), grid.height()).is_some();

    if over_stash {
        commands.entity(info.entity).insert(CardItemForm);
    } else {
        commands.entity(info.entity).remove::<CardItemForm>();
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use bevy_ecs::prelude::*;
    use engine_input::prelude::MouseState;
    use glam::Vec2;

    use super::stash_drag_hover_system;
    use crate::card_item_form::CardItemForm;
    use crate::card_zone::CardZone;
    use crate::drag_state::{DragInfo, DragState};
    use crate::stash_grid::StashGrid;
    use crate::stash_toggle::StashVisible;

    fn run_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(stash_drag_hover_system);
        schedule.run(world);
    }

    fn make_drag_state(entity: Entity) -> DragState {
        DragState {
            dragging: Some(DragInfo {
                entity,
                local_grab_offset: Vec2::ZERO,
                origin_zone: CardZone::Table,
                stash_cursor_follow: false,
            }),
        }
    }

    fn mouse_at(pos: Vec2) -> MouseState {
        let mut mouse = MouseState::default();
        mouse.set_screen_pos(pos);
        mouse
    }

    // -----------------------------------------------------------------------
    // TC015 – dragging AND cursor over stash AND stash visible
    //         → dragged entity gains CardItemForm
    // -----------------------------------------------------------------------

    #[test]
    fn when_dragging_and_cursor_over_stash_and_stash_visible_then_entity_gains_card_item_form() {
        // Arrange
        let mut world = World::new();
        let card = world.spawn_empty().id();
        world.insert_resource(make_drag_state(card));
        world.insert_resource(mouse_at(Vec2::new(45.0, 45.0)));
        world.insert_resource(StashVisible(true));
        world.insert_resource(StashGrid::new(10, 10, 1));

        // Act
        run_system(&mut world);

        // Assert
        assert!(
            world.get::<CardItemForm>(card).is_some(),
            "dragged entity should gain CardItemForm when cursor is over a visible stash slot"
        );
    }

    // -----------------------------------------------------------------------
    // TC016 – dragging AND cursor outside stash AND stash visible
    //         AND entity already has CardItemForm → entity loses CardItemForm
    // -----------------------------------------------------------------------

    #[test]
    fn when_dragging_and_cursor_outside_stash_and_stash_visible_then_entity_loses_card_item_form() {
        // Arrange
        let mut world = World::new();
        let card = world.spawn(CardItemForm).id();
        world.insert_resource(make_drag_state(card));
        world.insert_resource(mouse_at(Vec2::new(800.0, 400.0)));
        world.insert_resource(StashVisible(true));
        world.insert_resource(StashGrid::new(10, 10, 1));

        // Act
        run_system(&mut world);

        // Assert
        assert!(
            world.get::<CardItemForm>(card).is_none(),
            "stash-origin dragged entity should lose CardItemForm when cursor is outside stash area"
        );
    }

    // -----------------------------------------------------------------------
    // TC017 – no drag in progress AND cursor over stash
    //         → existing CardItemForm on a bystander entity is preserved
    //
    // A buggy implementation that clears CardItemForm from all entities
    // unconditionally would break this. A correct no-drag guard only acts
    // on the dragged entity.
    // -----------------------------------------------------------------------

    #[test]
    fn when_no_drag_and_cursor_over_stash_then_existing_card_item_form_is_preserved() {
        // Arrange — bystander card already has CardItemForm; no drag is active
        let mut world = World::new();
        let card = world.spawn(CardItemForm).id();
        world.insert_resource(DragState::default()); // no drag
        world.insert_resource(mouse_at(Vec2::new(45.0, 45.0)));
        world.insert_resource(StashVisible(true));
        world.insert_resource(StashGrid::new(10, 10, 1));

        // Act
        run_system(&mut world);

        // Assert
        assert!(
            world.get::<CardItemForm>(card).is_some(),
            "CardItemForm on a non-dragged entity must not be removed when there is no active drag"
        );
    }

    // -----------------------------------------------------------------------
    // TC018 – dragging AND cursor over stash area AND stash HIDDEN
    //         AND entity already has CardItemForm → entity loses CardItemForm
    //
    // Stash hidden means "not a valid drop target"; any existing CardItemForm
    // should be cleared regardless of cursor position.
    // -----------------------------------------------------------------------

    #[test]
    fn when_dragging_and_cursor_over_stash_and_stash_hidden_then_entity_loses_card_item_form() {
        // Arrange
        let mut world = World::new();
        let card = world.spawn(CardItemForm).id();
        world.insert_resource(make_drag_state(card));
        world.insert_resource(mouse_at(Vec2::new(45.0, 45.0)));
        world.insert_resource(StashVisible(false));
        world.insert_resource(StashGrid::new(10, 10, 1));

        // Act
        run_system(&mut world);

        // Assert
        assert!(
            world.get::<CardItemForm>(card).is_none(),
            "dragged entity must lose CardItemForm when stash is hidden"
        );
    }

    #[test]
    fn when_dragging_and_stash_hidden_and_entity_has_card_item_form_then_entity_loses_card_item_form()
     {
        // Arrange
        let mut world = World::new();
        let card = world.spawn(CardItemForm).id();
        world.insert_resource(make_drag_state(card));
        world.insert_resource(mouse_at(Vec2::new(45.0, 45.0)));
        world.insert_resource(StashVisible(false));
        world.insert_resource(StashGrid::new(10, 10, 1));

        // Act
        run_system(&mut world);

        // Assert
        assert!(
            world.get::<CardItemForm>(card).is_none(),
            "dragged entity should lose CardItemForm when stash becomes hidden"
        );
    }
}
