use glam::Vec2;

use crate::collider::Collider;

pub fn collider_half_extents(collider: &Collider) -> Option<Vec2> {
    match collider {
        Collider::Aabb(half) => Some(*half),
        _ => None,
    }
}

pub fn local_space_hit(cursor_local: Vec2, half: Vec2) -> bool {
    cursor_local.x.abs() <= half.x && cursor_local.y.abs() <= half.y
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn when_collider_is_aabb_then_returns_half_extents() {
        // Arrange
        let collider = Collider::Aabb(Vec2::new(30.0, 45.0));

        // Act
        let result = collider_half_extents(&collider);

        // Assert
        assert_eq!(result, Some(Vec2::new(30.0, 45.0)));
    }

    #[test]
    fn when_collider_is_circle_then_returns_none() {
        // Arrange
        let collider = Collider::Circle(15.0);

        // Act
        let result = collider_half_extents(&collider);

        // Assert
        assert_eq!(result, None);
    }

    #[test]
    fn when_collider_is_convex_polygon_then_returns_none() {
        // Arrange
        let collider = Collider::ConvexPolygon(vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(0.0, 1.0),
        ]);

        // Act
        let result = collider_half_extents(&collider);

        // Assert
        assert_eq!(result, None);
    }

    #[test]
    fn when_cursor_inside_aabb_then_hit_returns_true() {
        assert!(local_space_hit(
            Vec2::new(10.0, 20.0),
            Vec2::new(30.0, 45.0)
        ));
    }

    #[test]
    fn when_cursor_outside_on_x_axis_then_hit_returns_false() {
        assert!(!local_space_hit(
            Vec2::new(31.0, 0.0),
            Vec2::new(30.0, 45.0)
        ));
    }

    #[test]
    fn when_cursor_outside_on_y_axis_then_hit_returns_false() {
        assert!(!local_space_hit(
            Vec2::new(0.0, 46.0),
            Vec2::new(30.0, 45.0)
        ));
    }

    #[test]
    fn when_cursor_on_positive_edge_then_hit_returns_true() {
        assert!(local_space_hit(
            Vec2::new(30.0, 45.0),
            Vec2::new(30.0, 45.0)
        ));
    }

    #[test]
    fn when_cursor_on_negative_edge_then_hit_returns_true() {
        assert!(local_space_hit(
            Vec2::new(-30.0, -45.0),
            Vec2::new(30.0, 45.0)
        ));
    }
}
