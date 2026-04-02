use engine_physics::rigid_body::RigidBody;

#[test]
fn when_rigid_body_variants_serialized_to_ron_then_each_deserializes_to_matching_variant() {
    for body in [RigidBody::Dynamic, RigidBody::Static, RigidBody::Kinematic] {
        let ron = ron::to_string(&body).unwrap();
        let back: RigidBody = ron::from_str(&ron).unwrap();
        assert_eq!(body, back);
    }
}
