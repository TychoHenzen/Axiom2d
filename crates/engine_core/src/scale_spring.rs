use bevy_ecs::prelude::{Commands, Component, Entity, Query, Res};
use serde::{Deserialize, Serialize};

use crate::spring::spring_step;
use crate::time::DeltaTime;
use crate::transform::Transform2D;
use crate::types::Seconds;

const CONVERGE_THRESHOLD: f32 = 1e-4;
const DEFAULT_STIFFNESS: f32 = 200.0;
const DEFAULT_DAMPING: f32 = 20.0;
// Largest single integration step that stays well inside the stability region
// for stiffness=200: stable limit ≈ 2/sqrt(200) ≈ 0.141 s; 1/60 s gives a safe margin.
const MAX_SUBSTEP_DT: f32 = 1.0 / 60.0;

#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ScaleSpring {
    pub target: f32,
    pub velocity: f32,
    pub lock_x: bool,
    pub stiffness: f32,
    pub damping: f32,
}

impl ScaleSpring {
    pub fn new(target: f32) -> Self {
        Self {
            target,
            velocity: 0.0,
            lock_x: false,
            stiffness: DEFAULT_STIFFNESS,
            damping: DEFAULT_DAMPING,
        }
    }
}

pub fn scale_spring_system(
    dt: Res<DeltaTime>,
    mut query: Query<(Entity, &mut Transform2D, &mut ScaleSpring)>,
    mut commands: Commands,
) {
    let Seconds(dt_secs) = dt.0;

    for (entity, mut transform, mut spring) in &mut query {
        let mut remaining = dt_secs;
        let mut sc = transform.scale.y;
        let mut sv = spring.velocity;
        while remaining > 0.0 {
            let step = remaining.min(MAX_SUBSTEP_DT);
            (sc, sv) = spring_step(
                sc,
                spring.target,
                sv,
                step,
                spring.stiffness,
                spring.damping,
            );
            remaining -= step;
        }
        transform.scale.y = sc;
        if !spring.lock_x {
            transform.scale.x = sc;
        }
        spring.velocity = sv;

        if (sc - spring.target).abs() < CONVERGE_THRESHOLD && sv.abs() < CONVERGE_THRESHOLD {
            transform.scale.y = spring.target;
            if !spring.lock_x {
                transform.scale.x = spring.target;
            }
            commands.entity(entity).remove::<ScaleSpring>();
        }
    }
}
