use bevy_ecs::prelude::{Res, ResMut};
use engine_input::prelude::{MouseButton, MouseState};

use crate::drag_state::DragState;

pub fn card_release_system(mouse: Res<MouseState>, mut drag_state: ResMut<DragState>) {
    if drag_state.dragging.is_none() {
        return;
    }
    if !mouse.just_released(MouseButton::Left) {
        return;
    }
    drag_state.dragging = None;
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use bevy_ecs::prelude::*;
    use engine_input::prelude::{MouseButton, MouseState};
    use glam::Vec2;

    use super::card_release_system;
    use crate::card::Card;
    use crate::card_zone::CardZone;
    use crate::drag_state::{DragInfo, DragState};

    fn run_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(card_release_system);
        schedule.run(world);
    }

    fn spawn_entity() -> Entity {
        World::new().spawn_empty().id()
    }

    #[test]
    fn when_mouse_released_while_dragging_then_drag_state_cleared() {
        // Arrange
        let mut world = World::new();
        let entity = spawn_entity();
        world.insert_resource(DragState {
            dragging: Some(DragInfo {
                entity,
                local_grab_offset: Vec2::ZERO,
                origin_zone: CardZone::Table,
            }),
        });
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.release(MouseButton::Left);
        world.insert_resource(mouse);

        // Act
        run_system(&mut world);

        // Assert
        assert!(world.resource::<DragState>().dragging.is_none());
    }

    #[test]
    fn when_mouse_released_while_not_dragging_then_no_panic_and_stays_none() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(DragState::default());
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.release(MouseButton::Left);
        world.insert_resource(mouse);

        // Act
        run_system(&mut world);

        // Assert
        assert!(world.resource::<DragState>().dragging.is_none());
    }

    #[test]
    fn when_mouse_not_released_then_drag_state_not_cleared() {
        // Arrange
        let mut world = World::new();
        let entity = spawn_entity();
        world.insert_resource(DragState {
            dragging: Some(DragInfo {
                entity,
                local_grab_offset: Vec2::ZERO,
                origin_zone: CardZone::Table,
            }),
        });
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        world.insert_resource(mouse);

        // Act
        run_system(&mut world);

        // Assert
        assert!(world.resource::<DragState>().dragging.is_some());
    }

    #[test]
    fn when_card_released_on_table_then_zone_unchanged() {
        // Arrange
        let mut world = World::new();
        let entity = world
            .spawn((
                Card::face_down(
                    engine_core::prelude::TextureId(1),
                    engine_core::prelude::TextureId(2),
                ),
                CardZone::Table,
            ))
            .id();
        world.insert_resource(DragState {
            dragging: Some(DragInfo {
                entity,
                local_grab_offset: Vec2::ZERO,
                origin_zone: CardZone::Table,
            }),
        });
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.release(MouseButton::Left);
        world.insert_resource(mouse);

        // Act
        run_system(&mut world);

        // Assert
        let zone = world.entity(entity).get::<CardZone>().unwrap();
        assert_eq!(*zone, CardZone::Table);
    }
}
