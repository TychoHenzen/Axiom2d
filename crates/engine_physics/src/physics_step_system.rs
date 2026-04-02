use bevy_ecs::prelude::{Res, ResMut};
use engine_core::prelude::{DeltaTime, EventBus};

use crate::collision_event::CollisionEvent;
use crate::physics_res::PhysicsRes;

pub fn physics_step_system(
    dt: Res<DeltaTime>,
    mut physics: ResMut<PhysicsRes>,
    mut events: ResMut<EventBus<CollisionEvent>>,
) {
    physics.step(dt.0);
    for event in physics.drain_collision_events() {
        events.push(event);
    }
}
