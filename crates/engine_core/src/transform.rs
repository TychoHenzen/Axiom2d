use bevy_ecs::prelude::Component;
use glam::{Affine2, Vec2};
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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
#[allow(clippy::unwrap_used)]
mod tests {
    use super::Transform2D;
    use glam::{Affine2, Vec2};

    #[test]
    fn when_transform2d_serialized_to_ron_then_deserializes_to_equal_value() {
        // Arrange
        let transform = Transform2D {
            position: Vec2::new(100.0, -50.0),
            rotation: 1.2,
            scale: Vec2::new(2.0, 0.5),
        };

        // Act
        let ron = ron::to_string(&transform).unwrap();
        let back: Transform2D = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(transform, back);
    }

    #[test]
    fn when_transform2d_with_negative_rotation_serialized_to_ron_then_roundtrip_preserves_sign() {
        // Arrange
        let transform = Transform2D {
            rotation: -std::f32::consts::FRAC_PI_2,
            ..Transform2D::default()
        };

        // Act
        let ron = ron::to_string(&transform).unwrap();
        let back: Transform2D = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(transform, back);
    }

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

    /// @doc: Transform composition order is scale-then-rotate-then-translate (SRT).
    /// Getting this wrong would cause objects to orbit the origin instead of
    /// rotating in place, since translation applied before rotation shifts the
    /// pivot point.
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

    /// @doc: Negative scale enables horizontal flip (used during card flip
    /// animation). If `to_affine2` rejected or clamped negative scale values,
    /// the flip animation's scale-x-through-zero trick would break.
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

    proptest::proptest! {
        #[test]
        fn when_any_finite_transform2d_then_ron_roundtrip_preserves_value(
            px in -1000.0_f32..=1000.0,
            py in -1000.0_f32..=1000.0,
            rot in -std::f32::consts::TAU..=std::f32::consts::TAU,
            sx in -1000.0_f32..=1000.0,
            sy in -1000.0_f32..=1000.0,
        ) {
            // Arrange
            let transform = Transform2D {
                position: Vec2::new(px, py),
                rotation: rot,
                scale: Vec2::new(sx, sy),
            };

            // Act
            let ron = ron::to_string(&transform).unwrap();
            let back: Transform2D = ron::from_str(&ron).unwrap();

            // Assert
            assert_eq!(transform, back);
        }
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
