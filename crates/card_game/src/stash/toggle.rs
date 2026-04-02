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
