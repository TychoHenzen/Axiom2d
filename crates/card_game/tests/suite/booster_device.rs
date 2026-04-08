#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::*;
use card_game::booster::device::{BoosterMachine, BoosterSealButton, spawn_booster_machine};
use card_game::card::jack_cable::{Jack, JackDirection};
use card_game::card::reader::SignatureSpace;
use engine_core::prelude::{EventBus, Transform2D};
use engine_physics::prelude::PhysicsCommand;
use glam::Vec2;

#[test]
fn when_spawn_booster_machine_then_has_components() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(EventBus::<PhysicsCommand>::default());

    // Act
    let (device, input_jack) = spawn_booster_machine(&mut world, Vec2::new(100.0, 200.0));

    // Assert
    let machine = world.get::<BoosterMachine>(device).unwrap();
    assert_eq!(machine.signal_input, input_jack);
    assert!(machine.output_pack.is_none());

    let jack = world.get::<Jack<SignatureSpace>>(input_jack).unwrap();
    assert_eq!(jack.direction, JackDirection::Input);

    let transform = world.get::<Transform2D>(device).unwrap();
    assert_eq!(transform.position, Vec2::new(100.0, 200.0));
}

#[test]
fn when_spawn_booster_machine_then_button_references_device() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(EventBus::<PhysicsCommand>::default());

    // Act
    let (device, _input_jack) = spawn_booster_machine(&mut world, Vec2::ZERO);

    // Assert
    let machine = world.get::<BoosterMachine>(device).unwrap();
    let button = world
        .get::<BoosterSealButton>(machine.button_entity)
        .unwrap();
    assert_eq!(button.machine_entity, device);
}
