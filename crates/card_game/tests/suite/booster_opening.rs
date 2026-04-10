#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::Entity;
use card_game::booster::opening::{BoosterOpenPhase, BoosterOpening};
use card_game::card::identity::signature::CardSignature;
use glam::Vec2;

/// @doc: The opening state machine progresses through all five animation phases
/// in order: `MovingToCenter` -> Ripping -> `LoweringPack` -> `RevealingCards` -> Completing -> Done.
#[test]
fn when_opening_advances_then_phases_progress_in_order() {
    // Arrange
    let cards = vec![CardSignature::new([0.3; 8]); 3];
    let mut opening = BoosterOpening::new(
        Entity::PLACEHOLDER,
        cards,
        Vec2::new(100.0, 200.0),
        Vec2::ZERO,
        0.5,                   // start_rotation
        Vec2::new(50.0, 50.0), // camera_start_pos
    );

    // Act & Assert — walk through each phase
    assert!(matches!(
        opening.phase,
        BoosterOpenPhase::MovingToCenter { .. }
    ));
    opening.advance(0.4); // past 0.3s
    assert!(matches!(opening.phase, BoosterOpenPhase::Ripping { .. }));
    opening.advance(0.5); // past 0.4s
    assert!(matches!(
        opening.phase,
        BoosterOpenPhase::LoweringPack { .. }
    ));
    opening.advance(0.4); // past 0.3s
    assert!(matches!(
        opening.phase,
        BoosterOpenPhase::RevealingCards { .. }
    ));
    // 3 cards x 0.5s each
    for _ in 0..3 {
        opening.advance(0.6);
    }
    assert!(matches!(opening.phase, BoosterOpenPhase::Completing { .. }));
    // Completing needs enough time for the last card's fan animation too
    opening.advance(1.0);
    assert!(matches!(opening.phase, BoosterOpenPhase::Done));
}

/// @doc: During the `RevealingCards` phase, `card_index` advances each time
/// a single card's reveal duration elapses, allowing each card to animate
/// individually before the next begins.
#[test]
fn when_opening_reveals_cards_then_card_index_advances() {
    // Arrange
    let cards = vec![CardSignature::new([0.3; 8]); 2];
    let mut opening = BoosterOpening::new(
        Entity::PLACEHOLDER,
        cards,
        Vec2::new(100.0, 200.0),
        Vec2::ZERO,
        0.0,
        Vec2::ZERO,
    );

    // Act — advance past the first three phases
    opening.advance(0.4); // past MovingToCenter
    opening.advance(0.5); // past Ripping
    opening.advance(0.4); // past LoweringPack

    // Assert
    assert!(matches!(
        opening.phase,
        BoosterOpenPhase::RevealingCards { card_index: 0, .. }
    ));
    opening.advance(0.6); // past first card
    assert!(matches!(
        opening.phase,
        BoosterOpenPhase::RevealingCards { card_index: 1, .. }
    ));
}

/// @doc: Fan positions spread cards evenly around the original position
/// at `FAN_RADIUS` distance (80 pixels), ensuring all cards are visible
/// and distinct after the animation completes.
#[test]
fn when_fan_position_then_spread_around_original() {
    // Arrange
    let opening = BoosterOpening::new(
        Entity::PLACEHOLDER,
        vec![],
        Vec2::new(100.0, 100.0),
        Vec2::ZERO,
        0.0,
        Vec2::ZERO,
    );

    // Act
    let p0 = opening.fan_position(0, 3);
    let p1 = opening.fan_position(1, 3);
    let p2 = opening.fan_position(2, 3);

    // Assert — all should be roughly FAN_RADIUS from original_position
    let d0 = (p0 - Vec2::new(100.0, 100.0)).length();
    let d1 = (p1 - Vec2::new(100.0, 100.0)).length();
    let d2 = (p2 - Vec2::new(100.0, 100.0)).length();
    assert!((d0 - 80.0).abs() < 1.0);
    assert!((d1 - 80.0).abs() < 1.0);
    assert!((d2 - 80.0).abs() < 1.0);

    // They should be at different positions
    assert!(p0.distance(p1) > 10.0);
    assert!(p1.distance(p2) > 10.0);
}
