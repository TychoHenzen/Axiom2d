// Double-click detection system
use bevy_ecs::prelude::{Entity, Resource};

pub const DOUBLE_CLICK_WINDOW: f32 = 0.3;

#[derive(Resource, Debug, Default)]
pub struct DoubleClickState {
    last_click: Option<(Entity, f32)>,
}

impl DoubleClickState {
    /// Register a click on an entity at the given timestamp.
    /// Returns Some(entity) if this constitutes a double-click.
    pub fn register_click(&mut self, entity: Entity, time: f32) -> Option<Entity> {
        if let Some((prev_entity, prev_time)) = self.last_click {
            if prev_entity == entity && (time - prev_time) <= DOUBLE_CLICK_WINDOW {
                self.last_click = None;
                return Some(entity);
            }
        }
        self.last_click = Some((entity, time));
        None
    }
}

pub fn double_click_detect_system() {
    // Will be fully implemented when wired to the opening animation
}
