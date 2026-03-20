use bevy_ecs::prelude::{Res, ResMut, Resource};
use engine_input::prelude::{InputState, KeyCode};
use serde::{Deserialize, Serialize};

#[derive(Default, Resource, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StashVisible(pub bool);

pub fn stash_toggle_system(input: Res<InputState>, mut visible: ResMut<StashVisible>) {
    if input.just_pressed(KeyCode::Tab) {
        visible.0 = !visible.0;
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::{Schedule, World};

    fn run_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(super::stash_toggle_system);
        schedule.run(world);
    }

    #[test]
    fn when_defaulted_then_stash_is_hidden() {
        // Arrange / Act
        let vis = StashVisible::default();

        // Assert
        assert!(!vis.0);
    }

    #[test]
    fn when_tab_just_pressed_and_hidden_then_becomes_visible() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(StashVisible(false));
        let mut input = InputState::default();
        input.press(KeyCode::Tab);
        world.insert_resource(input);

        // Act
        run_system(&mut world);

        // Assert
        assert!(world.resource::<StashVisible>().0);
    }

    #[test]
    fn when_tab_just_pressed_and_visible_then_becomes_hidden() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(StashVisible(true));
        let mut input = InputState::default();
        input.press(KeyCode::Tab);
        world.insert_resource(input);

        // Act
        run_system(&mut world);

        // Assert
        assert!(!world.resource::<StashVisible>().0);
    }

    #[test]
    fn when_tab_not_pressed_then_visibility_unchanged() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(StashVisible(false));
        world.insert_resource(InputState::default());

        // Act
        run_system(&mut world);

        // Assert
        assert!(!world.resource::<StashVisible>().0);
    }
}
