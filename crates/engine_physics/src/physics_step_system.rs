use bevy_ecs::prelude::{Res, ResMut};
use engine_core::prelude::{DeltaTime, EventBus};
use engine_core::profiler::FrameProfiler;

use crate::collision_event::CollisionEvent;
use crate::physics_res::PhysicsRes;

pub fn physics_step_system(
    dt: Res<DeltaTime>,
    mut physics: ResMut<PhysicsRes>,
    mut events: ResMut<EventBus<CollisionEvent>>,
    mut profiler: Option<ResMut<FrameProfiler>>,
) {
    let t = std::time::Instant::now();
    physics.step(dt.0);
    for event in physics.drain_collision_events() {
        events.push(event);
    }
    if let Some(p) = profiler.as_deref_mut() {
        p.record_phase("physics_step", t.elapsed().as_micros() as u64);
    }
}
