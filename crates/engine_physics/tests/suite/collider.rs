use engine_physics::collider::Collider;
use glam::Vec2;

/// @doc: Verifies that convex polygon collider debug formatting matches the expected snapshot.
#[test]
fn when_convex_polygon_collider_debug_formatted_then_snapshot_matches() {
    // Arrange
    let collider = Collider::ConvexPolygon(vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(10.0, 0.0),
        Vec2::new(15.0, 8.0),
        Vec2::new(5.0, 14.0),
        Vec2::new(-5.0, 8.0),
    ]);

    // Act
    let debug = format!("{collider:#?}");

    // Assert
    insta::assert_snapshot!(debug);
}

/// @doc: Verifies that all collider variants (Circle, Aabb, ConvexPolygon) survive RON serialization round-trip with equal values.
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
        assert_eq!(
            *collider, back,
            "collider variant {:?} failed RON round-trip",
            collider
        );
    }
}

/// @doc: Verifies that a Circle collider with zero radius survives RON serialization round-trip (boundary case).
#[test]
fn when_circle_radius_is_zero_then_serializes_and_deserializes() {
    // Arrange
    let collider = Collider::Circle(0.0);

    // Act
    let ron = ron::to_string(&collider).unwrap();
    let back: Collider = ron::from_str(&ron).unwrap();

    // Assert
    assert_eq!(
        collider, back,
        "Circle(0.0) should round-trip: {} -> {}",
        ron, ron::to_string(&back).unwrap()
    );
}

/// @doc: Verifies that a Circle collider with negative radius survives RON serialization round-trip (edge case for physics).
#[test]
fn when_circle_radius_is_negative_then_serializes_and_deserializes() {
    // Arrange
    let collider = Collider::Circle(-5.0);

    // Act
    let ron = ron::to_string(&collider).unwrap();
    let back: Collider = ron::from_str(&ron).unwrap();

    // Assert
    assert_eq!(
        collider, back,
        "Circle(-5.0) should round-trip, got {:?}",
        back
    );
}

/// @doc: Verifies that an Aabb collider with zero extent survives RON serialization round-trip (degenerate AABB).
#[test]
fn when_aabb_extent_is_zero_then_serializes_and_deserializes() {
    // Arrange
    let collider = Collider::Aabb(Vec2::ZERO);

    // Act
    let ron = ron::to_string(&collider).unwrap();
    let back: Collider = ron::from_str(&ron).unwrap();

    // Assert
    assert_eq!(
        collider, back,
        "Aabb(ZERO) should round-trip, got {:?}",
        back
    );
}
