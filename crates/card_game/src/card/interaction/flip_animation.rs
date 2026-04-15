use bevy_ecs::prelude::{Commands, Component, Entity, Has, Query, Res};
use engine_core::prelude::{DeltaTime, Seconds, Transform2D};
use engine_core::scale_spring::ScaleSpring;

use crate::card::component::Card;

pub const FLIP_DURATION: Seconds = Seconds(0.3);

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct FlipAnimation {
    pub duration: Seconds,
    pub progress: f32,
    pub target_face_up: bool,
}

impl FlipAnimation {
    pub fn start(target_face_up: bool) -> Self {
        Self {
            duration: FLIP_DURATION,
            progress: 0.0,
            target_face_up,
        }
    }
}

pub fn flip_animation_system(
    dt: Res<DeltaTime>,
    mut query: Query<(Entity, &mut FlipAnimation, &mut Card, &mut Transform2D)>,
    mut commands: Commands,
) {
    for (entity, mut anim, mut card, mut transform) in &mut query {
        anim.progress += dt.0.0 / anim.duration.0;

        let base_scale = transform.scale.y;

        if anim.progress >= 1.0 {
            transform.scale.x = base_scale;
            card.face_up = anim.target_face_up;
            commands.entity(entity).remove::<FlipAnimation>();
            continue;
        }

        if anim.progress < 0.5 {
            transform.scale.x = base_scale * (1.0 - anim.progress * 2.0);
        } else {
            card.face_up = anim.target_face_up;
            transform.scale.x = base_scale * ((anim.progress - 0.5) * 2.0);
        }
    }
}

pub fn sync_scale_spring_lock_x(mut query: Query<(&mut ScaleSpring, Has<FlipAnimation>)>) {
    for (mut spring, has_flip) in &mut query {
        spring.lock_x = has_flip;
    }
}
