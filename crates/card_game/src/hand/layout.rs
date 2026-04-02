use bevy_ecs::prelude::{Commands, Component, Entity, Query, Res, ResMut};
use bevy_ecs::system::SystemParam;
use engine_core::prelude::{DeltaTime, Seconds, Transform2D};
use engine_core::profiler::FrameProfiler;
use engine_core::spring::spring_step;
use engine_render::prelude::{Camera2D, RendererRes, screen_to_world};
use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::hand::cards::Hand;
use engine_core::scale_spring::ScaleSpring;
use engine_render::prelude::resolve_viewport_camera;

#[derive(SystemParam)]
pub struct HandLayoutParams<'w> {
    hand: Res<'w, Hand>,
    dt: Res<'w, DeltaTime>,
    renderer: Res<'w, RendererRes>,
}

const FAN_ARC_DEGREES: f32 = 45.0;
const FAN_CARD_SPACING_DEGREES: f32 = 8.0;
const FAN_RADIUS: f32 = 400.0;
const FAN_BOTTOM_OFFSET: f32 = 80.0;

const FAN_SCALE: f32 = 3.0;

const SPRING_STIFFNESS: f32 = 200.0;
const SPRING_DAMPING: f32 = 20.0;

#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HandSpring {
    pub velocity: Vec2,
    pub angular_velocity: f32,
}

impl Default for HandSpring {
    fn default() -> Self {
        Self::new()
    }
}

impl HandSpring {
    pub fn new() -> Self {
        Self {
            velocity: Vec2::ZERO,
            angular_velocity: 0.0,
        }
    }
}

fn fan_angle(index: usize, count: usize) -> f32 {
    if count <= 1 {
        return 0.0;
    }
    let max_half_arc = (FAN_ARC_DEGREES / 2.0).to_radians();
    let desired_step = FAN_CARD_SPACING_DEGREES.to_radians();
    let step = desired_step.min(2.0 * max_half_arc / (count - 1) as f32);
    let half_spread = step * (count - 1) as f32 / 2.0;
    -half_spread + step * index as f32
}

fn fan_screen_position(angle: f32, viewport_width: f32, viewport_height: f32) -> Vec2 {
    let radius = FAN_RADIUS * FAN_SCALE;
    let pivot_x = viewport_width / 2.0;
    let pivot_y = viewport_height - FAN_BOTTOM_OFFSET + radius;
    Vec2::new(
        pivot_x + angle.sin() * radius,
        pivot_y - angle.cos() * radius,
    )
}

pub fn hand_layout_system(
    params: HandLayoutParams,
    camera_query: Query<&Camera2D>,
    mut cards: Query<(Entity, &mut Transform2D, Option<&mut HandSpring>)>,
    mut commands: Commands,
    mut profiler: Option<ResMut<FrameProfiler>>,
) {
    let _span = profiler.as_deref_mut().map(|p| p.span("hand_layout"));
    if params.hand.is_empty() {
        return;
    }

    let Some((vw, vh, camera)) = resolve_viewport_camera(&params.renderer, &camera_query) else {
        return;
    };

    let n = params.hand.len();
    let Seconds(dt_secs) = params.dt.0;
    let target_scale = FAN_SCALE / camera.zoom;

    for (i, &card_entity) in params.hand.cards().iter().enumerate() {
        let angle = fan_angle(i, n);
        let screen_pos = fan_screen_position(angle, vw, vh);
        let target_pos = screen_to_world(screen_pos, &camera, vw, vh);

        if let Ok((_, mut transform, spring)) = cards.get_mut(card_entity) {
            if let Some(mut spring) = spring {
                apply_spring_motion(&mut transform, &mut spring, target_pos, angle, dt_secs);
                commands
                    .entity(card_entity)
                    .insert(ScaleSpring::new(target_scale));
            } else {
                transform.position = target_pos;
                transform.rotation = angle;
                transform.scale = Vec2::splat(target_scale);
            }
        }
    }
}

fn apply_spring_motion(
    transform: &mut Transform2D,
    spring: &mut HandSpring,
    target_pos: Vec2,
    target_angle: f32,
    dt: f32,
) {
    let (px, vx) = spring_step(
        transform.position.x,
        target_pos.x,
        spring.velocity.x,
        dt,
        SPRING_STIFFNESS,
        SPRING_DAMPING,
    );
    let (py, vy) = spring_step(
        transform.position.y,
        target_pos.y,
        spring.velocity.y,
        dt,
        SPRING_STIFFNESS,
        SPRING_DAMPING,
    );
    let (rot, av) = spring_step(
        transform.rotation,
        target_angle,
        spring.angular_velocity,
        dt,
        SPRING_STIFFNESS,
        SPRING_DAMPING,
    );
    transform.position = Vec2::new(px, py);
    transform.rotation = rot;
    spring.velocity = Vec2::new(vx, vy);
    spring.angular_velocity = av;
}
