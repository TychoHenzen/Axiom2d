use glam::Vec2;

/// Convert pixel coordinates (column, row) to engine coordinates (Y-up, centered).
///
/// Pixel space: (0, 0) = top-left, Y increases downward.
/// Engine space: (0, 0) = center, Y increases upward.
pub fn pixel_to_engine(col: f32, row: f32, width: f32, height: f32) -> Vec2 {
    let x = col + 0.5 - width / 2.0;
    let y = height / 2.0 - (row + 0.5);
    Vec2::new(x, y)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::pixel_to_engine;

    #[test]
    fn when_top_left_pixel_then_negative_x_positive_y() {
        // Arrange / Act
        let v = pixel_to_engine(0.0, 0.0, 10.0, 10.0);

        // Assert
        assert!(v.x < 0.0, "expected x < 0, got {}", v.x);
        assert!(v.y > 0.0, "expected y > 0, got {}", v.y);
    }

    #[test]
    fn when_center_pixel_then_near_origin() {
        // Arrange / Act
        let v = pixel_to_engine(5.0, 5.0, 10.0, 10.0);

        // Assert
        assert!(v.x.abs() < 1.0, "expected x near 0, got {}", v.x);
        assert!(v.y.abs() < 1.0, "expected y near 0, got {}", v.y);
    }

    #[test]
    fn when_top_row_then_positive_y_and_bottom_row_then_negative_y() {
        // Arrange / Act
        let top = pixel_to_engine(5.0, 0.0, 10.0, 10.0);
        let bottom = pixel_to_engine(5.0, 9.0, 10.0, 10.0);

        // Assert
        assert!(top.y > 0.0, "top row should have positive y");
        assert!(bottom.y < 0.0, "bottom row should have negative y");
    }

    #[test]
    fn when_bottom_right_pixel_then_positive_x_negative_y() {
        // Arrange / Act
        let v = pixel_to_engine(9.0, 9.0, 10.0, 10.0);

        // Assert
        assert!(v.x > 0.0, "expected x > 0, got {}", v.x);
        assert!(v.y < 0.0, "expected y < 0, got {}", v.y);
    }

    #[test]
    fn when_symmetric_columns_then_x_values_are_mirrors() {
        // Arrange / Act
        let left = pixel_to_engine(0.0, 5.0, 10.0, 10.0);
        let right = pixel_to_engine(9.0, 5.0, 10.0, 10.0);

        // Assert
        assert_eq!(left.y, right.y);
        assert!(
            (left.x + right.x).abs() < 1.0,
            "expected mirrored x values, got {} and {}",
            left.x,
            right.x
        );
    }
}
