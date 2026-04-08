use bevy_ecs::prelude::World;
use engine_core::prelude::EventBus;
use engine_physics::prelude::{PhysicsCommand, RigidBody};
use glam::Vec2;

use card_game::booster::pack::{
    BoosterPack, PACK_HEIGHT, PACK_WIDTH, pack_shape_points, spawn_booster_pack,
};
use card_game::card::component::CardZone;
use card_game::card::identity::signature::CardSignature;

#[test]
fn when_spawn_booster_pack_then_has_pack_component() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(EventBus::<PhysicsCommand>::default());
    let sigs = vec![CardSignature::new([0.3; 8]); 5];

    // Act
    let entity = spawn_booster_pack(&mut world, Vec2::new(100.0, 200.0), sigs.clone());

    // Assert
    let pack = world.get::<BoosterPack>(entity).unwrap();
    assert_eq!(pack.cards.len(), 5);
    let zone = world.get::<CardZone>(entity).unwrap();
    assert_eq!(*zone, CardZone::Table);
    let rb = world.get::<RigidBody>(entity).unwrap();
    assert_eq!(*rb, RigidBody::Dynamic);
}

#[test]
fn when_spawn_booster_pack_then_physics_commands_queued() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(EventBus::<PhysicsCommand>::default());
    let sigs = vec![CardSignature::new([0.5; 8]); 3];

    // Act
    spawn_booster_pack(&mut world, Vec2::new(0.0, 0.0), sigs);

    // Assert
    let bus = world.get_resource::<EventBus<PhysicsCommand>>().unwrap();
    assert_eq!(bus.len(), 3, "expected AddBody, AddCollider, SetDamping");
}

#[test]
fn pack_shape_points_has_expected_vertex_count() {
    // Act
    let points = pack_shape_points();

    // Assert — bottom: 1 corner + TEETH_COUNT * 2, top: TEETH_COUNT * 2, close: 1
    // = 1 + 12 + 12 + 1 = 26
    assert_eq!(points.len(), 26);
}

#[test]
fn pack_shape_points_stays_within_bounds() {
    // Act
    let points = pack_shape_points();

    // Assert
    let half_w = PACK_WIDTH / 2.0;
    let half_h = PACK_HEIGHT / 2.0;
    for (i, p) in points.iter().enumerate() {
        assert!(
            p.x >= -half_w - f32::EPSILON && p.x <= half_w + f32::EPSILON,
            "point {i} x={} out of bounds ±{half_w}",
            p.x
        );
        assert!(
            p.y >= -half_h - f32::EPSILON && p.y <= half_h + f32::EPSILON,
            "point {i} y={} out of bounds ±{half_h}",
            p.y
        );
    }
}
