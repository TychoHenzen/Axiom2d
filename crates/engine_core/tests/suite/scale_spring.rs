#![allow(clippy::unwrap_used, clippy::float_cmp)]

use bevy_ecs::prelude::{Schedule, World};
use glam::Vec2;

use engine_core::scale_spring::{ScaleSpring, scale_spring_system};
use engine_core::time::DeltaTime;
use engine_core::transform::Transform2D;
use engine_core::types::Seconds;

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
    assert!(t.scale.x > 0.5, "expected scale.x > 0.5, got {}", t.scale.x);
    assert!(t.scale.x < 1.0, "expected scale.x < 1.0, got {}", t.scale.x);
}

#[test]
fn when_scale_spring_many_frames_then_both_axes_converge() {
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
        "expected scale.x≈1.0, got {}",
        t.scale.x
    );
    assert!(
        (t.scale.y - 1.0).abs() < 0.01,
        "expected scale.y≈1.0, got {}",
        t.scale.y
    );
}

/// @doc: Converged springs auto-remove — prevents wasted per-frame spring math on settled cards
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

/// @doc: Final scale snaps to exact target — avoids floating-point drift leaving cards at 0.999x scale
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

/// @doc: `lock_x` preserves horizontal scale during flip animation — only vertical axis participates in the squash
#[test]
fn when_lock_x_true_then_scale_y_springs_but_scale_x_unchanged() {
    // Arrange
    let mut world = make_world();
    let mut spring = ScaleSpring::new(1.0);
    spring.lock_x = true;
    let entity = world
        .spawn((
            Transform2D {
                position: Vec2::ZERO,
                rotation: 0.0,
                scale: Vec2::new(1.0, 0.5),
            },
            spring,
        ))
        .id();

    // Act
    run_system(&mut world);

    // Assert
    let t = world.get::<Transform2D>(entity).unwrap();
    assert_eq!(
        t.scale.x, 1.0,
        "scale.x should be unchanged when lock_x=true"
    );
    assert!(
        t.scale.y > 0.5,
        "scale.y should move toward target, got {}",
        t.scale.y
    );
}

// Convergence threshold (1e-4) and default stiffness/damping (200.0/20.0) are
// private constants in the source module. Tests below inline these values.

/// @doc: Spring removal requires BOTH position and velocity convergence — prevents premature removal during overshoot
#[test]
fn when_position_converged_but_velocity_above_threshold_then_spring_not_removed() {
    // Arrange
    let mut world = make_world();
    let entity = world
        .spawn((
            Transform2D {
                position: Vec2::ZERO,
                rotation: 0.0,
                scale: Vec2::splat(1.00001),
            },
            ScaleSpring {
                target: 1.0,
                velocity: 0.001,
                lock_x: false,
                stiffness: 200.0,
                damping: 20.0,
            },
        ))
        .id();

    // Act
    run_system(&mut world);

    // Assert
    assert!(
        world.get::<ScaleSpring>(entity).is_some(),
        "spring should not be removed when velocity is above threshold"
    );
}

/// @doc: Convergence threshold boundary — displacement exactly at the
/// threshold must NOT remove the spring. An off-by-one `<` vs `<=`
/// comparison would cause springs to be removed one frame too early,
/// leaving cards visibly short of their target scale.
#[test]
fn when_position_at_exact_threshold_boundary_then_spring_not_removed() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(DeltaTime(Seconds(0.0)));
    let entity = world
        .spawn((
            Transform2D {
                position: Vec2::ZERO,
                rotation: 0.0,
                scale: Vec2::splat(1.0 + 1e-4), // CONVERGE_THRESHOLD
            },
            ScaleSpring::new(1.0),
        ))
        .id();

    // Act
    run_system(&mut world);

    // Assert
    assert!(
        world.get::<ScaleSpring>(entity).is_some(),
        "spring should not be removed when displacement equals threshold exactly"
    );
}

#[test]
fn when_velocity_at_exact_threshold_boundary_then_spring_not_removed() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(DeltaTime(Seconds(0.0)));
    let entity = world
        .spawn((
            Transform2D {
                position: Vec2::ZERO,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
            ScaleSpring {
                target: 1.0,
                velocity: 1e-4, // CONVERGE_THRESHOLD
                lock_x: false,
                stiffness: 200.0,
                damping: 20.0,
            },
        ))
        .id();

    // Act
    run_system(&mut world);

    // Assert
    assert!(
        world.get::<ScaleSpring>(entity).is_some(),
        "spring should not be removed when velocity equals threshold exactly"
    );
}

/// @doc: Springs already at rest are removed on first tick — no wasted
/// computation. This happens when a card's current scale already matches
/// the spring target (e.g., releasing a card that wasn't resized).
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

/// @doc: Large dt (2 FPS) must not cause scale explosion — spring math stays stable under extreme frame drops
#[test]
fn when_large_dt_simulating_2fps_then_scale_stays_bounded() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(DeltaTime(Seconds(0.5)));
    let entity = world
        .spawn((
            Transform2D {
                position: Vec2::ZERO,
                rotation: 0.0,
                scale: Vec2::splat(1.0),
            },
            ScaleSpring::new(1.05),
        ))
        .id();

    // Act — run 5 frames at 2 FPS (2.5 simulated seconds)
    run_n_frames(&mut world, 5);

    // Assert
    let t = world.get::<Transform2D>(entity).unwrap();
    assert!(
        t.scale.x >= 0.0 && t.scale.x <= 10.0,
        "scale.x diverged at 2 FPS: {}",
        t.scale.x
    );
    assert!(
        t.scale.y >= 0.0 && t.scale.y <= 10.0,
        "scale.y diverged at 2 FPS: {}",
        t.scale.y
    );
}
