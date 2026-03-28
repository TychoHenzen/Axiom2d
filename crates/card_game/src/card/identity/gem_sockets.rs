use engine_core::color::Color;
use glam::Vec2;

use crate::card::identity::signature::Aspect;

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

/// Returns the 8 gem socket positions flanking the description strip.
/// Layout: 4 gems in a vertical column on each side, in the border margin
/// between the description strip outer edge and the card edge.
/// Positions are relative to card center (child-space offsets).
pub fn gem_desc_positions(card_size: Vec2) -> [Vec2; 8] {
    use crate::card::rendering::face_layout::FRONT_FACE_REGIONS;

    let card_half_w = card_size.x * 0.5;
    let (desc_half_w, desc_half_h, desc_offset_y) =
        FRONT_FACE_REGIONS[3].resolve(card_size.x, card_size.y);

    // Gem columns sit in the margin between desc strip edge and card edge,
    // centered so they don't exceed card bounds.
    let outer_limit = card_half_w - MAX_GEM_RADIUS;
    let col_x = (desc_half_w + outer_limit) * 0.5;

    let gem_count_per_col = 4;
    let step = (desc_half_h * 2.0) / (gem_count_per_col as f32 + 1.0);

    let mut positions = [Vec2::ZERO; 8];
    for i in 0..gem_count_per_col {
        let y = desc_offset_y - desc_half_h + step * (i as f32 + 1.0);
        positions[i] = Vec2::new(-col_x, y);
        positions[i + gem_count_per_col] = Vec2::new(col_x, y);
    }
    positions
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

    /// @doc: All 8 gem border positions must lie within card bounds plus `MAX_GEM_RADIUS` margin.
    /// Out-of-bounds gems would render off-card, corrupting the visual frame and player experience.
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

    // --- gem_desc_positions tests ---

    #[test]
    fn when_gem_desc_positions_called_then_returns_eight_positions() {
        // Arrange
        let card_size = Vec2::new(60.0, 90.0);

        // Act
        let positions = gem_desc_positions(card_size);

        // Assert
        assert_eq!(positions.len(), 8);
    }

    /// @doc: Description-flanking gems split evenly: 4 on left, 4 on right (symmetric layout).
    /// Asymmetric distribution would visually unbalance the card or collide with art.
    #[test]
    fn when_gem_desc_positions_called_then_four_left_four_right_of_center() {
        // Arrange
        let card_size = Vec2::new(60.0, 90.0);

        // Act
        let positions = gem_desc_positions(card_size);

        // Assert
        let left_count = positions.iter().filter(|p| p.x < 0.0).count();
        let right_count = positions.iter().filter(|p| p.x > 0.0).count();
        assert_eq!(left_count, 4, "should have 4 gems on the left");
        assert_eq!(right_count, 4, "should have 4 gems on the right");
    }

    #[test]
    fn when_gem_desc_positions_called_then_all_within_card_bounds() {
        // Arrange
        let card_size = Vec2::new(60.0, 90.0);
        let half_w = card_size.x * 0.5;
        let half_h = card_size.y * 0.5;

        // Act
        let positions = gem_desc_positions(card_size);

        // Assert
        for (i, pos) in positions.iter().enumerate() {
            assert!(
                pos.x.abs() <= half_w && pos.y.abs() <= half_h,
                "gem {i} at ({}, {}) is outside card bounds",
                pos.x,
                pos.y
            );
        }
    }

    /// @doc: Left and right gem columns must be vertically aligned and horizontally mirrored.
    /// Asymmetry would look unbalanced and suggest intentional layout bugs to players.
    #[test]
    fn when_gem_desc_positions_called_then_columns_are_symmetric() {
        // Arrange
        let card_size = Vec2::new(60.0, 90.0);

        // Act
        let positions = gem_desc_positions(card_size);
        let mut left: Vec<Vec2> = positions.iter().filter(|p| p.x < 0.0).copied().collect();
        let mut right: Vec<Vec2> = positions.iter().filter(|p| p.x > 0.0).copied().collect();
        left.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap());
        right.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap());

        // Assert — each left/right pair should have same y and mirrored x
        for (l, r) in left.iter().zip(right.iter()) {
            assert!(
                (l.y - r.y).abs() < 1e-4,
                "y mismatch: left={}, right={}",
                l.y,
                r.y
            );
            assert!(
                (l.x + r.x).abs() < 1e-4,
                "x not mirrored: left={}, right={}",
                l.x,
                r.x
            );
        }
    }

    /// @doc: All gems must fit vertically within the description strip bounds.
    /// Gems outside this range would either overlap the card art above/below or look misplaced.
    #[test]
    fn when_gem_desc_positions_called_then_y_positions_span_desc_strip() {
        use crate::card::rendering::face_layout::FRONT_FACE_REGIONS;

        // Arrange
        let card_size = Vec2::new(60.0, 90.0);
        let (_, desc_half_h, desc_offset_y) =
            FRONT_FACE_REGIONS[3].resolve(card_size.x, card_size.y);
        let desc_top = desc_offset_y - desc_half_h;
        let desc_bottom = desc_offset_y + desc_half_h;

        // Act
        let positions = gem_desc_positions(card_size);

        // Assert
        for (i, pos) in positions.iter().enumerate() {
            assert!(
                pos.y >= desc_top - 1e-4 && pos.y <= desc_bottom + 1e-4,
                "gem {i} y={} outside desc strip [{}, {}]",
                pos.y,
                desc_top,
                desc_bottom
            );
        }
    }

    /// @doc: The 4 gems per column must have distinct y-positions (spacing > 1.0 unit apart).
    /// Overlapping gem positions would render on top of each other, hiding some visually.
    #[test]
    fn when_gem_desc_positions_called_then_four_distinct_y_values() {
        // Arrange
        let card_size = Vec2::new(60.0, 90.0);

        // Act
        let positions = gem_desc_positions(card_size);
        let mut left_ys: Vec<f32> = positions
            .iter()
            .filter(|p| p.x < 0.0)
            .map(|p| p.y)
            .collect();
        left_ys.sort_by(|a, b| a.partial_cmp(b).unwrap());

        // Assert — all 4 y-values should be distinct
        for window in left_ys.windows(2) {
            assert!(
                (window[1] - window[0]).abs() > 1.0,
                "y-values too close: {} and {}",
                window[0],
                window[1]
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

    /// @doc: Gem radius scales linearly from `MIN_GEM_RADIUS` to `MAX_GEM_RADIUS` with intensity [0.0, 1.0].
    /// Non-linearity would distort the visual feedback of element intensity on the card.
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
