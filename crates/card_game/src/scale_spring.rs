use bevy_ecs::prelude::{Commands, Component, Entity, Has, Query, Res};
use engine_core::prelude::{DeltaTime, Seconds, Transform2D};
use serde::{Deserialize, Serialize};

use crate::flip_animation::FlipAnimation;
use crate::hand_layout::spring_step;

const CONVERGE_THRESHOLD: f32 = 1e-4;

#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ScaleSpring {
    pub target: f32,
    pub velocity: f32,
}

impl ScaleSpring {
    pub fn new(target: f32) -> Self {
        Self {
            target,
            velocity: 0.0,
        }
    }
}

pub fn scale_spring_system(
    dt: Res<DeltaTime>,
    mut query: Query<(Entity, &mut Transform2D, &mut ScaleSpring, Has<FlipAnimation>)>,
    mut commands: Commands,
) {
    let Seconds(dt_secs) = dt.0;

    for (entity, mut transform, mut spring, has_flip) in &mut query {
        let (sc, sv) =
            spring_step(transform.scale.y, spring.target, spring.velocity, dt_secs);
        transform.scale.y = sc;
        if !has_flip {
            transform.scale.x = sc;
        }
        spring.velocity = sv;

        if (sc - spring.target).abs() < CONVERGE_THRESHOLD
            && sv.abs() < CONVERGE_THRESHOLD
        {
            transform.scale.y = spring.target;
            if !has_flip {
                transform.scale.x = spring.target;
            }
            commands.entity(entity).remove::<ScaleSpring>();
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::{Schedule, World};
    use glam::Vec2;

    fn make_world() -> World {
        let mut world = World::new();
        world.insert_resource(DeltaTime(Seconds(0.016)));
        world
    }

    fn run_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(scale_spring_system);
        schedule.run(world);
    }

    fn run_n_frames(world: &mut World, n: usize) {
        let mut schedule = Schedule::default();
        schedule.add_systems(scale_spring_system);
        for _ in 0..n {
            schedule.run(world);
        }
    }

    #[test]
    fn when_scale_spring_one_frame_then_moves_toward_target() {
        // Arrange
        let mut world = make_world();
        let entity = world
            .spawn((
                Transform2D {
                    position: Vec2::ZERO,
                    rotation: 0.0,
                    scale: Vec2::splat(0.5),
                },
                ScaleSpring::new(1.0),
            ))
            .id();

        // Act
        run_system(&mut world);

        // Assert
        let t = world.get::<Transform2D>(entity).unwrap();
        assert!(
            t.scale.x > 0.5,
            "expected scale.x to increase from 0.5 toward 1.0, got {}",
            t.scale.x
        );
        assert!(
            t.scale.x < 1.0,
            "expected scale.x not to reach 1.0 in one frame, got {}",
            t.scale.x
        );
    }

    #[test]
    fn when_scale_spring_many_frames_then_converges() {
        // Arrange
        let mut world = make_world();
        let entity = world
            .spawn((
                Transform2D {
                    position: Vec2::ZERO,
                    rotation: 0.0,
                    scale: Vec2::splat(0.5),
                },
                ScaleSpring::new(1.0),
            ))
            .id();

        // Act
        run_n_frames(&mut world, 300);

        // Assert
        let t = world.get::<Transform2D>(entity).unwrap();
        assert!(
            (t.scale.x - 1.0).abs() < 0.01,
            "expected scale.x≈1.0 after convergence, got {}",
            t.scale.x
        );
    }

    #[test]
    fn when_scale_spring_converged_then_component_removed() {
        // Arrange
        let mut world = make_world();
        let entity = world
            .spawn((
                Transform2D {
                    position: Vec2::ZERO,
                    rotation: 0.0,
                    scale: Vec2::splat(0.5),
                },
                ScaleSpring::new(1.0),
            ))
            .id();

        // Act
        run_n_frames(&mut world, 300);

        // Assert
        assert!(
            world.get::<ScaleSpring>(entity).is_none(),
            "ScaleSpring should be removed after convergence"
        );
    }

    #[test]
    fn when_scale_spring_converged_then_scale_snapped_to_exact_target() {
        // Arrange
        let mut world = make_world();
        let entity = world
            .spawn((
                Transform2D {
                    position: Vec2::ZERO,
                    rotation: 0.0,
                    scale: Vec2::splat(0.5),
                },
                ScaleSpring::new(1.0),
            ))
            .id();

        // Act
        run_n_frames(&mut world, 300);

        // Assert
        let t = world.get::<Transform2D>(entity).unwrap();
        assert_eq!(t.scale, Vec2::ONE);
    }

    #[test]
    fn when_scale_at_target_with_zero_velocity_then_immediately_removed() {
        // Arrange
        let mut world = make_world();
        let entity = world
            .spawn((
                Transform2D {
                    position: Vec2::ZERO,
                    rotation: 0.0,
                    scale: Vec2::ONE,
                },
                ScaleSpring::new(1.0),
            ))
            .id();

        // Act
        run_system(&mut world);

        // Assert
        assert!(
            world.get::<ScaleSpring>(entity).is_none(),
            "ScaleSpring should be removed when already at target"
        );
    }
}
