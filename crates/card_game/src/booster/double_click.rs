// Double-click detection system
use bevy_ecs::prelude::{Entity, Resource, World};
use engine_core::prelude::EventBus;
use engine_core::time::DeltaTime;
use engine_input::mouse_button::MouseButton;
use engine_input::prelude::MouseState;
use engine_physics::prelude::PhysicsCommand;
use engine_render::camera::Camera2D;
use glam::Vec2;

use crate::booster::opening::BoosterOpening;
use crate::booster::pack::BoosterPack;
use crate::card::interaction::drag_state::DragState;

pub const DOUBLE_CLICK_WINDOW: f32 = 0.3;

#[derive(Resource, Debug, Default)]
pub struct DoubleClickState {
    last_click: Option<(Entity, f32)>,
    elapsed: f32,
}

impl DoubleClickState {
    /// Advance internal clock by dt seconds.
    pub fn tick(&mut self, dt: f32) {
        self.elapsed += dt;
    }

    /// Register a click on an entity at the given timestamp.
    /// Returns Some(entity) if this constitutes a double-click.
    pub fn register_click(&mut self, entity: Entity, time: f32) -> Option<Entity> {
        if let Some((prev_entity, prev_time)) = self.last_click
            && prev_entity == entity
            && (time - prev_time) <= DOUBLE_CLICK_WINDOW
        {
            self.last_click = None;
            return Some(entity);
        }
        self.last_click = Some((entity, time));
        None
    }
}

pub fn double_click_detect_system(world: &mut World) {
    // 1. Always tick the elapsed timer so timing works between clicks
    let dt = world.get_resource::<DeltaTime>().map_or(0.0, |dt| dt.0.0);
    {
        let mut state = world
            .get_resource_mut::<DoubleClickState>()
            .expect("DoubleClickState must be inserted");
        state.tick(dt);
    }

    // 2. Don't allow a new opening while one is already running
    if world.contains_resource::<BoosterOpening>() {
        return;
    }

    // 3. Check mouse just pressed (left button)
    let just_pressed = world
        .get_resource::<MouseState>()
        .is_some_and(|m| m.just_pressed(MouseButton::Left));
    if !just_pressed {
        return;
    }

    // 4. Check if a BoosterPack entity is being dragged
    let drag_entity = world
        .get_resource::<DragState>()
        .and_then(|d| d.dragging.as_ref())
        .map(|d| d.entity);
    let Some(entity) = drag_entity else { return };

    if world.get::<BoosterPack>(entity).is_none() {
        return;
    }

    // 5. Get current elapsed time and register the click
    let double_clicked = {
        let mut state = world
            .get_resource_mut::<DoubleClickState>()
            .expect("DoubleClickState must be inserted");
        let now = state.elapsed;
        state.register_click(entity, now)
    };

    let Some(pack_entity) = double_clicked else {
        return;
    };

    // 6. Double-click confirmed — cancel the drag
    if let Some(mut drag_state) = world.get_resource_mut::<DragState>() {
        drag_state.dragging = None;
    }

    // 7. Remove physics from the pack entity
    if let Some(mut bus) = world.get_resource_mut::<EventBus<PhysicsCommand>>() {
        bus.push(PhysicsCommand::RemoveBody {
            entity: pack_entity,
        });
    }

    // 8. Gather pack position, rotation, and cards
    let (pack_position, pack_rotation) = world
        .get::<engine_core::prelude::Transform2D>(pack_entity)
        .map_or((Vec2::ZERO, 0.0), |t| (t.position, t.rotation));

    let cards = world
        .get::<BoosterPack>(pack_entity)
        .map_or_else(Vec::new, |bp| bp.cards.clone());

    // 9. Capture current camera position — the camera will pan from here
    //    to center on the pack.
    let camera_start_pos = world
        .query::<&Camera2D>()
        .iter(world)
        .next()
        .map_or(Vec2::ZERO, |cam| cam.position);

    // 10. Insert the BoosterOpening resource to start the animation.
    //     screen_center is the pack position — the camera pans to it.
    world.insert_resource(BoosterOpening::new(
        pack_entity,
        cards,
        pack_position,
        pack_position,
        pack_rotation,
        camera_start_pos,
    ));
}
