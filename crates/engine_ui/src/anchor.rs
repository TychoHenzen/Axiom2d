use glam::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Anchor {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

pub fn anchor_offset(anchor: Anchor, size: Vec2) -> Vec2 {
    let half = size * 0.5;
    match anchor {
        Anchor::TopLeft => Vec2::ZERO,
        Anchor::TopCenter => Vec2::new(-half.x, 0.0),
        Anchor::TopRight => Vec2::new(-size.x, 0.0),
        Anchor::CenterLeft => Vec2::new(0.0, -half.y),
        Anchor::Center => -half,
        Anchor::CenterRight => Vec2::new(-size.x, -half.y),
        Anchor::BottomLeft => Vec2::new(0.0, -size.y),
        Anchor::BottomCenter => Vec2::new(-half.x, -size.y),
        Anchor::BottomRight => -size,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn when_center_anchor_then_negative_half_size() {
        // Arrange
        let size = Vec2::new(100.0, 60.0);

        // Act
        let offset = anchor_offset(Anchor::Center, size);

        // Assert
        assert_eq!(offset, Vec2::new(-50.0, -30.0));
    }

    #[test]
    fn when_top_left_anchor_then_zero_offset() {
        // Arrange
        let size = Vec2::new(100.0, 50.0);

        // Act
        let offset = anchor_offset(Anchor::TopLeft, size);

        // Assert
        assert_eq!(offset, Vec2::ZERO);
    }

    #[test]
    fn when_top_right_anchor_then_negative_width() {
        // Arrange
        let size = Vec2::new(80.0, 40.0);

        // Act
        let offset = anchor_offset(Anchor::TopRight, size);

        // Assert
        assert_eq!(offset, Vec2::new(-80.0, 0.0));
    }

    #[test]
    fn when_bottom_center_anchor_then_half_width_full_height() {
        // Arrange
        let size = Vec2::new(100.0, 60.0);

        // Act
        let offset = anchor_offset(Anchor::BottomCenter, size);

        // Assert
        assert_eq!(offset, Vec2::new(-50.0, -60.0));
    }

    proptest::proptest! {
        #[test]
        fn when_top_left_anchor_and_any_size_then_offset_is_zero(
            w in 0.0_f32..=1000.0,
            h in 0.0_f32..=1000.0,
        ) {
            // Act
            let offset = anchor_offset(Anchor::TopLeft, Vec2::new(w, h));

            // Assert
            assert_eq!(offset, Vec2::ZERO);
        }

        #[test]
        fn when_bottom_right_anchor_and_any_size_then_offset_is_negative_size(
            w in 0.0_f32..=1000.0,
            h in 0.0_f32..=1000.0,
        ) {
            // Arrange
            let size = Vec2::new(w, h);

            // Act
            let offset = anchor_offset(Anchor::BottomRight, size);

            // Assert
            assert_eq!(offset, -size);
        }
    }

    #[test]
    fn when_all_nine_anchors_with_asymmetric_size_then_exact_offsets() {
        // Arrange
        let size = Vec2::new(80.0, 40.0);

        // Act / Assert
        assert_eq!(anchor_offset(Anchor::TopLeft, size), Vec2::new(0.0, 0.0));
        assert_eq!(anchor_offset(Anchor::TopCenter, size), Vec2::new(-40.0, 0.0));
        assert_eq!(anchor_offset(Anchor::TopRight, size), Vec2::new(-80.0, 0.0));
        assert_eq!(anchor_offset(Anchor::CenterLeft, size), Vec2::new(0.0, -20.0));
        assert_eq!(anchor_offset(Anchor::Center, size), Vec2::new(-40.0, -20.0));
        assert_eq!(anchor_offset(Anchor::CenterRight, size), Vec2::new(-80.0, -20.0));
        assert_eq!(anchor_offset(Anchor::BottomLeft, size), Vec2::new(0.0, -40.0));
        assert_eq!(anchor_offset(Anchor::BottomCenter, size), Vec2::new(-40.0, -40.0));
        assert_eq!(anchor_offset(Anchor::BottomRight, size), Vec2::new(-80.0, -40.0));
    }

    #[test]
    fn when_all_nine_anchors_then_all_offsets_distinct() {
        // Arrange
        let size = Vec2::new(100.0, 60.0);
        let anchors = [
            Anchor::TopLeft,
            Anchor::TopCenter,
            Anchor::TopRight,
            Anchor::CenterLeft,
            Anchor::Center,
            Anchor::CenterRight,
            Anchor::BottomLeft,
            Anchor::BottomCenter,
            Anchor::BottomRight,
        ];

        // Act
        let offsets: Vec<Vec2> = anchors.iter().map(|a| anchor_offset(*a, size)).collect();

        // Assert
        for i in 0..offsets.len() {
            for j in (i + 1)..offsets.len() {
                assert_ne!(offsets[i], offsets[j], "{anchors:?}[{i}] and [{j}] collide");
            }
        }
    }
}
