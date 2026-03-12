use bevy_ecs::prelude::Component;
use glam::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Collider {
    Circle(f32),
    Aabb(Vec2),
    ConvexPolygon(Vec<Vec2>),
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn when_collider_variants_serialized_to_ron_then_each_deserializes_to_equal_value() {
        // Arrange
        let colliders = [
            Collider::Circle(15.0),
            Collider::Aabb(Vec2::new(32.0, 64.0)),
            Collider::ConvexPolygon(vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(10.0, 0.0),
                Vec2::new(5.0, 8.0),
            ]),
        ];

        for collider in &colliders {
            // Act
            let ron = ron::to_string(collider).unwrap();
            let back: Collider = ron::from_str(&ron).unwrap();

            // Assert
            assert_eq!(*collider, back);
        }
    }
}
