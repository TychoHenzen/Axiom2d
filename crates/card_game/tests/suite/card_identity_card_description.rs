#![allow(clippy::unwrap_used)]

use card_game::card::identity::card_description::generate_card_description;
use card_game::card::identity::residual::ResidualStats;

fn zero_stats() -> ResidualStats {
    ResidualStats {
        power: 0.0,
        cost: 0.0,
        duration: 0.0,
        range: 0.0,
        healing: 0.0,
        speed: 0.0,
        defense: 0.0,
        special: 0.0,
    }
}

#[test]
fn when_all_stats_zero_then_description_is_empty() {
    // Arrange
    let stats = zero_stats();

    // Act
    let desc = generate_card_description(&stats);

    // Assert
    assert!(
        desc.is_empty(),
        "zero stats should produce empty description, got: {desc:?}"
    );
}

#[test]
fn when_only_power_nonzero_then_description_is_deal_damage() {
    // Arrange
    let stats = ResidualStats {
        power: 0.6,
        ..zero_stats()
    };

    // Act
    let desc = generate_card_description(&stats);

    // Assert — 0.6 * 20 = 12
    assert_eq!(desc, "Deal 12 damage");
}

#[test]
fn when_only_healing_nonzero_then_description_is_restore_health() {
    // Arrange
    let stats = ResidualStats {
        healing: 0.4,
        ..zero_stats()
    };

    // Act
    let desc = generate_card_description(&stats);

    // Assert — 0.4 * 20 = 8
    assert_eq!(desc, "Restore 8 health");
}

/// @doc: Multiple stat effects are sorted by magnitude (absolute value), strongest first.
/// This prioritization ensures key effects appear first in the description text.
#[test]
fn when_multiple_stats_then_ordered_by_magnitude_descending() {
    // Arrange
    let stats = ResidualStats {
        power: 0.3,
        healing: 0.5,
        defense: 0.1,
        ..zero_stats()
    };

    // Act
    let desc = generate_card_description(&stats);

    // Assert — healing (0.5) > power (0.3) > defense (0.1)
    let lines: Vec<&str> = desc.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "Restore 10 health");
    assert_eq!(lines[1], "Deal 6 damage");
    assert_eq!(lines[2], "Block 2 damage");
}

/// @doc: Card descriptions are capped at 3 effects (take(3)) to avoid UI clutter.
/// Without this limit, complex cards would overflow the description panel.
#[test]
fn when_more_than_three_stats_then_only_top_three_shown() {
    // Arrange
    let stats = ResidualStats {
        power: 0.5,
        healing: 0.4,
        defense: 0.3,
        speed: 0.2,
        ..zero_stats()
    };

    // Act
    let desc = generate_card_description(&stats);

    // Assert
    let lines: Vec<&str> = desc.lines().collect();
    assert_eq!(lines.len(), 3, "should show at most 3 effects");
    assert!(
        !desc.contains("initiative"),
        "speed (lowest) should be excluded"
    );
}

/// @doc: Residual stats below `MIN_DISPLAY_VALUE` (1) after scaling are filtered out (`filter_map`).
/// This prevents tiny fractional residuals from cluttering the description with "Deal 0 damage" noise.
#[test]
fn when_stat_too_small_to_display_then_omitted() {
    // Arrange — 0.02 * 20 = 0.4, rounds to 0, below MIN_DISPLAY_VALUE
    let stats = ResidualStats {
        power: 0.02,
        ..zero_stats()
    };

    // Act
    let desc = generate_card_description(&stats);

    // Assert
    assert!(
        desc.is_empty(),
        "tiny stats should be omitted, got: {desc:?}"
    );
}

#[test]
fn when_defense_nonzero_then_description_says_block() {
    // Arrange
    let stats = ResidualStats {
        defense: 0.25,
        ..zero_stats()
    };

    // Act
    let desc = generate_card_description(&stats);

    // Assert — 0.25 * 20 = 5
    assert_eq!(desc, "Block 5 damage");
}

#[test]
fn when_speed_nonzero_then_description_says_initiative() {
    // Arrange
    let stats = ResidualStats {
        speed: 0.15,
        ..zero_stats()
    };

    // Act
    let desc = generate_card_description(&stats);

    // Assert — 0.15 * 20 = 3
    assert_eq!(desc, "+3 initiative");
}
