use engine_core::color::Color;
use glam::Vec2;

use crate::card::signature::Aspect;

/// Maps each of the 16 Aspect variants to a distinct color.
/// Positive aspects get warm hues (r > b or g > b), negative aspects get cool hues (b > r).
pub fn aspect_color(aspect: Aspect) -> Color {
    match aspect {
        // Positive (warm) — reds, oranges, yellows, greens
        Aspect::Solid => Color::new(0.85, 0.55, 0.20, 1.0), // amber
        Aspect::Heat => Color::new(0.95, 0.25, 0.10, 1.0),  // red-orange
        Aspect::Order => Color::new(0.90, 0.80, 0.10, 1.0), // gold
        Aspect::Light => Color::new(0.98, 0.95, 0.40, 1.0), // bright yellow
        Aspect::Change => Color::new(0.70, 0.85, 0.10, 1.0), // yellow-green
        Aspect::Force => Color::new(0.90, 0.40, 0.05, 1.0), // deep orange
        Aspect::Growth => Color::new(0.20, 0.80, 0.20, 1.0), // green
        Aspect::Expansion => Color::new(0.60, 0.90, 0.30, 1.0), // lime

        // Negative (cool) — blues, purples, teals, indigos
        Aspect::Fragile => Color::new(0.30, 0.50, 0.85, 1.0), // periwinkle
        Aspect::Cold => Color::new(0.10, 0.70, 0.95, 1.0),    // ice blue
        Aspect::Chaos => Color::new(0.55, 0.10, 0.80, 1.0),   // violet
        Aspect::Dark => Color::new(0.15, 0.05, 0.40, 1.0),    // deep indigo
        Aspect::Stasis => Color::new(0.20, 0.60, 0.80, 1.0),  // steel blue
        Aspect::Calm => Color::new(0.10, 0.75, 0.70, 1.0),    // teal
        Aspect::Decay => Color::new(0.35, 0.20, 0.60, 1.0),   // muted purple
        Aspect::Contraction => Color::new(0.05, 0.20, 0.70, 1.0), // navy
    }
}

/// Returns the 8 gem socket positions around the card border perimeter.
/// Positions are relative to card center (child-space offsets).
/// Layout: 2 on each of the 4 edges, evenly spaced at 1/3 and 2/3 along each edge.
pub fn gem_border_positions(card_size: Vec2) -> [Vec2; 8] {
    let hw = card_size.x * 0.5;
    let hh = card_size.y * 0.5;
    let third_w = card_size.x / 3.0;
    let third_h = card_size.y / 3.0;

    [
        // Top edge: left-of-center, right-of-center
        Vec2::new(-third_w * 0.5, hh),
        Vec2::new(third_w * 0.5, hh),
        // Bottom edge
        Vec2::new(-third_w * 0.5, -hh),
        Vec2::new(third_w * 0.5, -hh),
        // Left edge: upper-third, lower-third
        Vec2::new(-hw, third_h * 0.5),
        Vec2::new(-hw, -third_h * 0.5),
        // Right edge
        Vec2::new(hw, third_h * 0.5),
        Vec2::new(hw, -third_h * 0.5),
    ]
}

pub const MIN_GEM_RADIUS: f32 = 1.0;
pub const MAX_GEM_RADIUS: f32 = 3.5;

/// Maps element intensity (0.0..=1.0) to gem circle radius.
pub fn gem_radius(intensity: f32) -> f32 {
    MIN_GEM_RADIUS + intensity * (MAX_GEM_RADIUS - MIN_GEM_RADIUS)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // --- aspect_color tests ---

    #[test]
    fn when_aspect_color_called_with_positive_aspect_then_returns_warm_hue() {
        // Arrange / Act / Assert — positive aspects should have r > b or g > b
        for aspect in [
            Aspect::Solid,
            Aspect::Heat,
            Aspect::Order,
            Aspect::Light,
            Aspect::Change,
            Aspect::Force,
            Aspect::Growth,
            Aspect::Expansion,
        ] {
            let color = aspect_color(aspect);
            assert!(
                color.r > color.b || color.g > color.b,
                "{aspect:?} should be warm (r={} g={} > b={})",
                color.r,
                color.g,
                color.b
            );
        }
    }

    #[test]
    fn when_aspect_color_called_with_negative_aspect_then_returns_cool_hue() {
        // Arrange / Act / Assert — negative aspects should have b > r
        for aspect in [
            Aspect::Fragile,
            Aspect::Cold,
            Aspect::Chaos,
            Aspect::Dark,
            Aspect::Stasis,
            Aspect::Calm,
            Aspect::Decay,
            Aspect::Contraction,
        ] {
            let color = aspect_color(aspect);
            assert!(
                color.b > color.r,
                "{aspect:?} should be cool (b={} > r={})",
                color.b,
                color.r
            );
        }
    }

    // --- gem_border_positions tests ---

    #[test]
    fn when_gem_positions_computed_then_exactly_8_positions_returned() {
        // Arrange
        let card_size = Vec2::new(60.0, 90.0);

        // Act
        let positions = gem_border_positions(card_size);

        // Assert
        assert_eq!(positions.len(), 8);
    }

    #[test]
    fn when_gem_positions_computed_then_all_within_card_border_band() {
        // Arrange
        let card_size = Vec2::new(60.0, 90.0);
        let half_w = card_size.x * 0.5;
        let half_h = card_size.y * 0.5;

        // Act
        let positions = gem_border_positions(card_size);

        // Assert — each gem must be within card bounds + margin
        let margin = MAX_GEM_RADIUS;
        for (i, pos) in positions.iter().enumerate() {
            assert!(
                pos.x.abs() <= half_w + margin && pos.y.abs() <= half_h + margin,
                "gem {i} at ({}, {}) is outside card bounds",
                pos.x,
                pos.y
            );
        }
    }

    // --- gem_radius tests ---

    #[test]
    fn when_gem_radius_at_zero_intensity_then_equals_minimum() {
        // Arrange / Act
        let radius = gem_radius(0.0);

        // Assert
        assert_eq!(radius, MIN_GEM_RADIUS);
    }

    #[test]
    fn when_gem_radius_at_full_intensity_then_equals_maximum() {
        // Arrange / Act
        let radius = gem_radius(1.0);

        // Assert
        assert_eq!(radius, MAX_GEM_RADIUS);
    }

    #[test]
    fn when_gem_radius_at_mid_intensity_then_between_min_and_max() {
        // Arrange / Act
        let radius = gem_radius(0.5);

        // Assert
        assert!(
            radius > MIN_GEM_RADIUS && radius < MAX_GEM_RADIUS,
            "radius {radius} should be between {MIN_GEM_RADIUS} and {MAX_GEM_RADIUS}"
        );
    }
}
