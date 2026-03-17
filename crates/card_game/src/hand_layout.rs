use bevy_ecs::prelude::{Component, Query, Res};
use engine_core::prelude::{DeltaTime, Seconds, Transform2D};
use engine_render::prelude::{Camera2D, RendererRes, screen_to_world};
use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::hand::Hand;

pub const FAN_ARC_DEGREES: f32 = 30.0;
pub const FAN_CARD_SPACING_DEGREES: f32 = 5.0;
pub const FAN_RADIUS: f32 = 600.0;
pub const FAN_BOTTOM_OFFSET: f32 = 80.0;

pub const SPRING_STIFFNESS: f32 = 200.0;
pub const SPRING_DAMPING: f32 = 20.0;

#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HandSpring {
    pub velocity: Vec2,
    pub angular_velocity: f32,
}

impl HandSpring {
    pub fn new() -> Self {
        Self {
            velocity: Vec2::ZERO,
            angular_velocity: 0.0,
        }
    }
}

pub fn spring_step(current: f32, target: f32, velocity: f32, dt: f32) -> (f32, f32) {
    let displacement = target - current;
    let acceleration = displacement * SPRING_STIFFNESS - velocity * SPRING_DAMPING;
    let new_velocity = velocity + acceleration * dt;
    let new_position = current + new_velocity * dt;
    (new_position, new_velocity)
}

pub fn fan_angle(index: usize, count: usize) -> f32 {
    if count <= 1 {
        return 0.0;
    }
    let max_half_arc = (FAN_ARC_DEGREES / 2.0).to_radians();
    let desired_step = FAN_CARD_SPACING_DEGREES.to_radians();
    let step = desired_step.min(2.0 * max_half_arc / (count - 1) as f32);
    let half_spread = step * (count - 1) as f32 / 2.0;
    -half_spread + step * index as f32
}

pub fn fan_screen_position(angle: f32, viewport_width: f32, viewport_height: f32) -> Vec2 {
    let pivot_x = viewport_width / 2.0;
    let pivot_y = viewport_height - FAN_BOTTOM_OFFSET + FAN_RADIUS;
    Vec2::new(
        pivot_x + angle.sin() * FAN_RADIUS,
        pivot_y - angle.cos() * FAN_RADIUS,
    )
}

pub fn hand_layout_system(
    hand: Res<Hand>,
    dt: Res<DeltaTime>,
    camera_query: Query<&Camera2D>,
    renderer: Res<RendererRes>,
    mut cards: Query<(&mut Transform2D, Option<&mut HandSpring>)>,
) {
    if hand.is_empty() {
        return;
    }

    let (vw, vh) = renderer.viewport_size();
    if vw == 0 || vh == 0 {
        return;
    }
    let vw = vw as f32;
    let vh = vh as f32;

    let camera = camera_query
        .iter()
        .next()
        .copied()
        .unwrap_or(Camera2D::default());

    let n = hand.len();
    let Seconds(dt_secs) = dt.0;

    for (i, &entity) in hand.cards().iter().enumerate() {
        let angle = fan_angle(i, n);
        let screen_pos = fan_screen_position(angle, vw, vh);
        let target_pos = screen_to_world(screen_pos, &camera, vw, vh);

        if let Ok((mut transform, spring)) = cards.get_mut(entity) {
            if let Some(mut spring) = spring {
                let (px, vx) = spring_step(
                    transform.position.x,
                    target_pos.x,
                    spring.velocity.x,
                    dt_secs,
                );
                let (py, vy) = spring_step(
                    transform.position.y,
                    target_pos.y,
                    spring.velocity.y,
                    dt_secs,
                );
                let (rot, av) =
                    spring_step(transform.rotation, angle, spring.angular_velocity, dt_secs);
                transform.position = Vec2::new(px, py);
                transform.rotation = rot;
                spring.velocity = Vec2::new(vx, vy);
                spring.angular_velocity = av;
            } else {
                transform.position = target_pos;
                transform.rotation = angle;
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::{Entity, Schedule, World};
    use engine_render::testing::SpyRenderer;
    use std::sync::{Arc, Mutex};

    fn make_world(viewport_w: u32, viewport_h: u32) -> World {
        let mut world = World::new();
        world.insert_resource(Hand::new(10));
        world.insert_resource(DeltaTime(Seconds(0.016)));

        let log = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log).with_viewport(viewport_w, viewport_h);
        world.insert_resource(RendererRes::new(Box::new(spy)));

        world
    }

    fn run_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(hand_layout_system);
        schedule.run(world);
    }

    fn add_card_to_hand(world: &mut World) -> Entity {
        let entity = world
            .spawn(Transform2D {
                position: Vec2::ZERO,
                rotation: 999.0,
                scale: Vec2::ONE,
            })
            .id();
        world.resource_mut::<Hand>().add(entity).unwrap();
        entity
    }

    fn add_spring_card_to_hand(world: &mut World, position: Vec2) -> Entity {
        let entity = world
            .spawn((
                Transform2D {
                    position,
                    rotation: 0.0,
                    scale: Vec2::ONE,
                },
                HandSpring::new(),
            ))
            .id();
        world.resource_mut::<Hand>().add(entity).unwrap();
        entity
    }

    fn run_n_frames(world: &mut World, n: usize) {
        let mut schedule = Schedule::default();
        schedule.add_systems(hand_layout_system);
        for _ in 0..n {
            schedule.run(world);
        }
    }

    #[test]
    fn when_hand_is_empty_then_no_transform_is_mutated() {
        // Arrange
        let mut world = make_world(800, 600);
        let sentinel = Vec2::new(999.0, 999.0);
        let entity = world
            .spawn(Transform2D {
                position: sentinel,
                rotation: 0.0,
                scale: Vec2::ONE,
            })
            .id();

        // Act
        run_system(&mut world);

        // Assert
        let t = world.get::<Transform2D>(entity).unwrap();
        assert_eq!(t.position, sentinel);
    }

    #[test]
    fn when_viewport_width_is_zero_then_card_transform_is_not_mutated() {
        // Arrange
        let mut world = make_world(0, 768);
        let sentinel = Vec2::new(42.0, 42.0);
        let entity = world
            .spawn(Transform2D {
                position: sentinel,
                rotation: 0.0,
                scale: Vec2::ONE,
            })
            .id();
        world.resource_mut::<Hand>().add(entity).unwrap();

        // Act
        run_system(&mut world);

        // Assert
        let t = world.get::<Transform2D>(entity).unwrap();
        assert_eq!(t.position, sentinel);
    }

    #[test]
    fn when_viewport_height_is_zero_then_card_transform_is_not_mutated() {
        // Arrange
        let mut world = make_world(800, 0);
        let sentinel = Vec2::new(42.0, 42.0);
        let entity = world
            .spawn(Transform2D {
                position: sentinel,
                rotation: 0.0,
                scale: Vec2::ONE,
            })
            .id();
        world.resource_mut::<Hand>().add(entity).unwrap();

        // Act
        run_system(&mut world);

        // Assert
        let t = world.get::<Transform2D>(entity).unwrap();
        assert_eq!(t.position, sentinel);
    }

    #[test]
    fn when_entity_has_no_transform_then_no_panic() {
        // Arrange
        let mut world = make_world(800, 600);
        world.spawn(Camera2D::default());
        let entity = world.spawn_empty().id();
        world.resource_mut::<Hand>().add(entity).unwrap();

        // Act
        run_system(&mut world);
    }

    #[test]
    fn when_one_card_without_spring_then_rotation_is_zero() {
        // Arrange
        let mut world = make_world(800, 600);
        world.spawn(Camera2D::default());
        let entity = add_card_to_hand(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        let t = world.get::<Transform2D>(entity).unwrap();
        assert!(
            t.rotation.abs() < 1e-3,
            "expected rotation≈0.0, got {}",
            t.rotation
        );
    }

    #[test]
    fn when_one_card_without_spring_then_card_is_horizontally_centered() {
        // Arrange
        let mut world = make_world(800, 600);
        world.spawn(Camera2D::default());
        let entity = add_card_to_hand(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        let t = world.get::<Transform2D>(entity).unwrap();
        assert!(
            t.position.x.abs() < 1e-3,
            "expected x≈0.0, got {}",
            t.position.x
        );
    }

    #[test]
    fn when_one_card_without_spring_then_card_y_is_at_fan_bottom_offset() {
        // Arrange
        let mut world = make_world(800, 600);
        world.spawn(Camera2D::default());
        let entity = add_card_to_hand(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        let expected_y = 300.0 - FAN_BOTTOM_OFFSET;
        let t = world.get::<Transform2D>(entity).unwrap();
        assert!(
            (t.position.y - expected_y).abs() < 1e-3,
            "expected y≈{expected_y}, got {}",
            t.position.y
        );
    }

    #[test]
    fn when_two_cards_without_spring_then_left_has_negative_rotation_right_has_positive() {
        // Arrange
        let mut world = make_world(800, 600);
        world.spawn(Camera2D::default());
        let a = add_card_to_hand(&mut world);
        let b = add_card_to_hand(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        let ar = world.get::<Transform2D>(a).unwrap().rotation;
        let br = world.get::<Transform2D>(b).unwrap().rotation;
        assert!(ar < 0.0, "expected left rotation < 0, got {ar}");
        assert!(br > 0.0, "expected right rotation > 0, got {br}");
    }

    #[test]
    fn when_two_cards_without_spring_then_rotations_are_symmetric() {
        // Arrange
        let mut world = make_world(800, 600);
        world.spawn(Camera2D::default());
        let a = add_card_to_hand(&mut world);
        let b = add_card_to_hand(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        let ar = world.get::<Transform2D>(a).unwrap().rotation;
        let br = world.get::<Transform2D>(b).unwrap().rotation;
        assert!(
            (ar + br).abs() < 1e-3,
            "expected symmetric rotations, got {ar} and {br}"
        );
    }

    #[test]
    fn when_two_cards_without_spring_then_first_is_left_of_second() {
        // Arrange
        let mut world = make_world(800, 600);
        world.spawn(Camera2D::default());
        let a = add_card_to_hand(&mut world);
        let b = add_card_to_hand(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        let ax = world.get::<Transform2D>(a).unwrap().position.x;
        let bx = world.get::<Transform2D>(b).unwrap().position.x;
        assert!(ax < bx, "expected a.x < b.x, got {ax} vs {bx}");
    }

    #[test]
    fn when_two_cards_without_spring_then_fan_centered_around_screen_center() {
        // Arrange
        let mut world = make_world(800, 600);
        world.spawn(Camera2D::default());
        let a = add_card_to_hand(&mut world);
        let b = add_card_to_hand(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        let ax = world.get::<Transform2D>(a).unwrap().position.x;
        let bx = world.get::<Transform2D>(b).unwrap().position.x;
        let midpoint = (ax + bx) * 0.5;
        assert!(
            midpoint.abs() < 1e-3,
            "expected midpoint≈0.0, got {midpoint}"
        );
    }

    #[test]
    fn when_three_cards_without_spring_then_center_card_has_zero_rotation() {
        // Arrange
        let mut world = make_world(800, 600);
        world.spawn(Camera2D::default());
        let _a = add_card_to_hand(&mut world);
        let b = add_card_to_hand(&mut world);
        let _c = add_card_to_hand(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        let br = world.get::<Transform2D>(b).unwrap().rotation;
        assert!(
            br.abs() < 1e-3,
            "expected center card rotation≈0.0, got {br}"
        );
    }

    #[test]
    fn when_three_cards_without_spring_then_angular_spacing_is_uniform() {
        // Arrange
        let mut world = make_world(800, 600);
        world.spawn(Camera2D::default());
        let a = add_card_to_hand(&mut world);
        let b = add_card_to_hand(&mut world);
        let c = add_card_to_hand(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        let ar = world.get::<Transform2D>(a).unwrap().rotation;
        let br = world.get::<Transform2D>(b).unwrap().rotation;
        let cr = world.get::<Transform2D>(c).unwrap().rotation;
        let gap_ab = br - ar;
        let gap_bc = cr - br;
        assert!(
            (gap_ab - gap_bc).abs() < 1e-3,
            "expected uniform angular gap, got ab={gap_ab} bc={gap_bc}"
        );
    }

    #[test]
    fn when_three_cards_without_spring_then_center_card_is_horizontally_centered() {
        // Arrange
        let mut world = make_world(800, 600);
        world.spawn(Camera2D::default());
        let _a = add_card_to_hand(&mut world);
        let b = add_card_to_hand(&mut world);
        let _c = add_card_to_hand(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        let bx = world.get::<Transform2D>(b).unwrap().position.x;
        assert!(bx.abs() < 1e-3, "expected center card x≈0.0, got {bx}");
    }

    #[test]
    fn when_no_camera_then_uses_default_camera() {
        // Arrange
        let mut world = make_world(800, 600);
        let entity = add_card_to_hand(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        let t = world.get::<Transform2D>(entity).unwrap();
        assert!(
            t.position.x.abs() < 1e-3,
            "expected x≈0.0, got {}",
            t.position.x
        );
    }

    #[test]
    fn when_camera_offset_then_positions_shift() {
        // Arrange
        let mut world = make_world(800, 600);
        world.spawn(Camera2D {
            position: Vec2::new(100.0, 50.0),
            zoom: 1.0,
        });
        let entity = add_card_to_hand(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        let t = world.get::<Transform2D>(entity).unwrap();
        assert!(
            (t.position.x - 100.0).abs() < 1e-3,
            "expected x≈100.0, got {}",
            t.position.x
        );
    }

    #[test]
    fn when_camera_zoom_two_then_world_spread_halved() {
        // Arrange
        let mut world_z1 = make_world(800, 600);
        world_z1.spawn(Camera2D {
            position: Vec2::ZERO,
            zoom: 1.0,
        });
        let a1 = add_card_to_hand(&mut world_z1);
        let b1 = add_card_to_hand(&mut world_z1);
        run_system(&mut world_z1);
        let spread_z1 = world_z1.get::<Transform2D>(b1).unwrap().position.x
            - world_z1.get::<Transform2D>(a1).unwrap().position.x;

        let mut world_z2 = make_world(800, 600);
        world_z2.spawn(Camera2D {
            position: Vec2::ZERO,
            zoom: 2.0,
        });
        let a2 = add_card_to_hand(&mut world_z2);
        let b2 = add_card_to_hand(&mut world_z2);

        // Act
        run_system(&mut world_z2);

        // Assert
        let spread_z2 = world_z2.get::<Transform2D>(b2).unwrap().position.x
            - world_z2.get::<Transform2D>(a2).unwrap().position.x;
        let expected = spread_z1 / 2.0;
        assert!(
            (spread_z2 - expected).abs() < 1e-3,
            "expected spread≈{expected}, got {spread_z2}"
        );
    }

    #[test]
    fn when_fan_angle_with_count_one_then_angle_is_zero() {
        // Act
        let angle = fan_angle(0, 1);

        // Assert
        assert!(angle.abs() < 1e-6);
    }

    #[test]
    fn when_fan_angle_with_count_three_then_center_is_zero() {
        // Act
        let angle = fan_angle(1, 3);

        // Assert
        assert!(angle.abs() < 1e-6);
    }

    #[test]
    fn when_fan_angle_with_count_two_then_angles_are_symmetric() {
        // Act
        let left = fan_angle(0, 2);
        let right = fan_angle(1, 2);

        // Assert
        assert!(
            (left + right).abs() < 1e-6,
            "expected symmetric, got {left} and {right}"
        );
    }

    #[test]
    fn when_two_cards_then_spacing_equals_card_spacing_degrees() {
        // Act
        let left = fan_angle(0, 2);
        let right = fan_angle(1, 2);

        // Assert
        let expected_step = FAN_CARD_SPACING_DEGREES.to_radians();
        assert!(
            (right - left - expected_step).abs() < 1e-6,
            "expected step≈{expected_step}, got {}",
            right - left
        );
    }

    #[test]
    fn when_two_cards_then_spread_is_less_than_full_arc() {
        // Act
        let left = fan_angle(0, 2);
        let right = fan_angle(1, 2);
        let spread = right - left;

        // Assert
        let full_arc = FAN_ARC_DEGREES.to_radians();
        assert!(
            spread < full_arc - 1e-6,
            "2-card spread {spread} should be less than full arc {full_arc}"
        );
    }

    #[test]
    fn when_many_cards_exceed_arc_then_spread_clamped_to_full_arc() {
        // Arrange — enough cards that desired_step * (n-1) > full arc
        let n = 20;

        // Act
        let first = fan_angle(0, n);
        let last = fan_angle(n - 1, n);
        let spread = last - first;

        // Assert
        let full_arc = FAN_ARC_DEGREES.to_radians();
        assert!(
            (spread - full_arc).abs() < 1e-6,
            "expected spread clamped to full arc {full_arc}, got {spread}"
        );
    }

    #[test]
    fn when_fan_screen_position_at_angle_zero_then_centered_horizontally() {
        // Act
        let pos = fan_screen_position(0.0, 800.0, 600.0);

        // Assert
        assert!(
            (pos.x - 400.0).abs() < 1e-3,
            "expected x≈400.0, got {}",
            pos.x
        );
    }

    #[test]
    fn when_fan_screen_position_at_angle_zero_then_y_is_bottom_offset() {
        // Act
        let pos = fan_screen_position(0.0, 800.0, 600.0);

        // Assert
        let expected_y = 600.0 - FAN_BOTTOM_OFFSET;
        assert!(
            (pos.y - expected_y).abs() < 1e-3,
            "expected y≈{expected_y}, got {}",
            pos.y
        );
    }

    #[test]
    fn when_spring_step_from_zero_toward_target_then_moves_toward_target() {
        // Act
        let (pos, vel) = spring_step(0.0, 100.0, 0.0, 0.016);

        // Assert
        assert!(pos > 0.0, "expected pos > 0, got {pos}");
        assert!(vel > 0.0, "expected vel > 0, got {vel}");
    }

    #[test]
    fn when_spring_step_at_target_with_zero_velocity_then_stays_at_target() {
        // Act
        let (pos, vel) = spring_step(50.0, 50.0, 0.0, 0.016);

        // Assert
        assert!((pos - 50.0).abs() < 1e-6, "expected pos≈50, got {pos}");
        assert!(vel.abs() < 1e-6, "expected vel≈0, got {vel}");
    }

    #[test]
    fn when_spring_card_one_frame_then_moves_toward_target_but_does_not_arrive() {
        // Arrange
        let mut world = make_world(800, 600);
        world.spawn(Camera2D::default());
        let entity = add_spring_card_to_hand(&mut world, Vec2::new(200.0, 200.0));

        // Act
        run_system(&mut world);

        // Assert
        let t = world.get::<Transform2D>(entity).unwrap();
        assert!(
            t.position.x < 200.0,
            "expected card to move left toward target (0), got x={}",
            t.position.x
        );
        assert!(
            t.position.x.abs() > 1.0,
            "expected card not to reach target in one frame, got x={}",
            t.position.x
        );
    }

    #[test]
    fn when_spring_card_many_frames_then_converges_near_target() {
        // Arrange
        let mut world = make_world(800, 600);
        world.spawn(Camera2D::default());
        let entity = add_spring_card_to_hand(&mut world, Vec2::new(200.0, 200.0));

        // Act
        run_n_frames(&mut world, 300);

        // Assert
        let t = world.get::<Transform2D>(entity).unwrap();
        assert!(
            t.position.x.abs() < 1.0,
            "expected x≈0 after convergence, got {}",
            t.position.x
        );
    }

    #[test]
    fn when_spring_card_then_rotation_also_springs() {
        // Arrange
        let mut world = make_world(800, 600);
        world.spawn(Camera2D::default());
        let a = add_spring_card_to_hand(&mut world, Vec2::new(0.0, 0.0));
        let _b = add_spring_card_to_hand(&mut world, Vec2::new(0.0, 0.0));

        // Act
        run_system(&mut world);

        // Assert
        let t = world.get::<Transform2D>(a).unwrap();
        let target_angle = fan_angle(0, 2);
        assert!(
            t.rotation != 0.0,
            "expected rotation to change from initial"
        );
        assert!(
            (t.rotation - target_angle).abs() > 1e-6,
            "expected rotation not to reach target in one frame"
        );
        assert!(
            t.rotation < 0.0,
            "expected left card rotation to be negative, got {}",
            t.rotation
        );
    }

    #[test]
    fn when_spring_card_many_frames_then_rotation_converges() {
        // Arrange
        let mut world = make_world(800, 600);
        world.spawn(Camera2D::default());
        let a = add_spring_card_to_hand(&mut world, Vec2::ZERO);
        let _b = add_spring_card_to_hand(&mut world, Vec2::ZERO);

        // Act
        run_n_frames(&mut world, 300);

        // Assert
        let t = world.get::<Transform2D>(a).unwrap();
        let target_angle = fan_angle(0, 2);
        assert!(
            (t.rotation - target_angle).abs() < 0.01,
            "expected rotation≈{target_angle}, got {}",
            t.rotation
        );
    }

    #[test]
    fn when_spring_overshoots_target_then_velocity_reverses() {
        // Arrange — particle past the target with forward velocity
        let (pos, vel) = spring_step(150.0, 100.0, 50.0, 0.016);

        // Assert — spring pulls back, velocity should decrease
        assert!(
            vel < 50.0,
            "expected spring to slow velocity when past target, got {vel}"
        );
        assert!(
            pos > 100.0,
            "expected still past target after one step, got {pos}"
        );
    }
}
