use engine_physics::collider::Collider;
use glam::Vec2;

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
