use bevy_ecs::prelude::{Has, Query};
use engine_core::scale_spring::ScaleSpring;

use crate::flip_animation::FlipAnimation;

pub fn sync_scale_spring_lock_x(mut query: Query<(&mut ScaleSpring, Has<FlipAnimation>)>) {
    for (mut spring, has_flip) in &mut query {
        spring.lock_x = has_flip;
    }
}
