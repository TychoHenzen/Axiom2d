use glam::{Affine2, Vec2};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform2D {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
}

impl Transform2D {
    pub fn to_affine2(&self) -> Affine2 {
        Affine2::from_scale_angle_translation(self.scale, self.rotation, self.position)
    }
}

impl Default for Transform2D {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Transform2D;
    use glam::{Affine2, Vec2};

    #[test]
    fn when_default_transform_converted_to_affine2_then_equals_identity() {
        // Act
        let affine = Transform2D::default().to_affine2();

        // Assert
        assert_eq!(affine, Affine2::IDENTITY);
    }

    #[test]
    fn when_transform_has_translation_only_then_affine2_is_pure_translation() {
        // Arrange
        let t = Transform2D {
            position: Vec2::new(3.0, 5.0),
            ..Transform2D::default()
        };

        // Act
        let affine = t.to_affine2();

        // Assert
        assert_eq!(affine, Affine2::from_translation(Vec2::new(3.0, 5.0)));
    }

    #[test]
    fn when_transform_has_rotation_only_then_affine2_is_pure_rotation() {
        // Arrange
        let t = Transform2D {
            rotation: std::f32::consts::FRAC_PI_2,
            ..Transform2D::default()
        };

        // Act
        let affine = t.to_affine2();

        // Assert
        assert_eq!(affine, Affine2::from_angle(std::f32::consts::FRAC_PI_2));
    }

    #[test]
    fn when_transform_has_scale_only_then_affine2_is_pure_scale() {
        // Arrange
        let t = Transform2D {
            scale: Vec2::new(2.0, 3.0),
            ..Transform2D::default()
        };

        // Act
        let affine = t.to_affine2();

        // Assert
        assert_eq!(affine, Affine2::from_scale(Vec2::new(2.0, 3.0)));
    }

    #[test]
    fn when_transform_has_all_components_then_affine2_matches_scale_angle_translation() {
        // Arrange
        let t = Transform2D {
            position: Vec2::new(10.0, -5.0),
            rotation: std::f32::consts::FRAC_PI_4,
            scale: Vec2::splat(2.0),
        };

        // Act
        let affine = t.to_affine2();

        // Assert
        let expected = Affine2::from_scale_angle_translation(
            Vec2::splat(2.0),
            std::f32::consts::FRAC_PI_4,
            Vec2::new(10.0, -5.0),
        );
        assert_eq!(affine, expected);
    }

    #[test]
    fn when_transform_composed_then_order_is_scale_rotate_translate() {
        // Arrange
        let t = Transform2D {
            position: Vec2::new(1.0, 0.0),
            rotation: std::f32::consts::FRAC_PI_2,
            scale: Vec2::new(2.0, 1.0),
        };

        // Act
        let affine = t.to_affine2();

        // Assert
        let translation = affine.translation;
        assert!((translation.x - 1.0).abs() < 1e-6);
        assert!(translation.y.abs() < 1e-6);
    }

    #[test]
    fn when_transform_has_negative_scale_then_affine2_preserves_flip() {
        // Arrange
        let t = Transform2D {
            scale: Vec2::new(-1.0, 1.0),
            ..Transform2D::default()
        };

        // Act
        let affine = t.to_affine2();

        // Assert
        assert_eq!(affine, Affine2::from_scale(Vec2::new(-1.0, 1.0)));
    }

    #[test]
    fn when_transform_has_full_circle_rotation_then_affine2_is_near_identity() {
        // Arrange
        let t = Transform2D {
            rotation: std::f32::consts::TAU,
            ..Transform2D::default()
        };

        // Act
        let affine = t.to_affine2();

        // Assert
        let id = Affine2::IDENTITY;
        assert!((affine.matrix2.x_axis - id.matrix2.x_axis).length() < 1e-6);
        assert!((affine.matrix2.y_axis - id.matrix2.y_axis).length() < 1e-6);
        assert!((affine.translation - id.translation).length() < 1e-6);
    }
}
