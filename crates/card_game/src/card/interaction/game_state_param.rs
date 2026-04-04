use bevy_ecs::prelude::{Res, ResMut};
use bevy_ecs::system::SystemParam;
use engine_core::prelude::EventBus;
use engine_physics::prelude::PhysicsCommand;

use crate::card::interaction::drag_state::DragState;
use crate::hand::cards::Hand;
use crate::stash::grid::StashGrid;
use crate::stash::toggle::StashVisible;

#[derive(SystemParam)]
pub struct CardGameState<'w> {
    pub drag_state: ResMut<'w, DragState>,
    pub hand: ResMut<'w, Hand>,
    pub physics_commands: ResMut<'w, EventBus<PhysicsCommand>>,
    pub stash_visible: Res<'w, StashVisible>,
    pub grid: ResMut<'w, StashGrid>,
}
