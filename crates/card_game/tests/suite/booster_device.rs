#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::*;
use card_game::booster::device::{
    BoosterMachine, BoosterSealButton, SealButtonPressed, booster_seal_system,
    spawn_booster_machine,
};
use card_game::booster::pack::BoosterPack;
use card_game::card::component::Card;
use card_game::card::identity::signature::CardSignature;
use card_game::card::jack_cable::{Jack, JackDirection};
use card_game::card::reader::{CardReader, SignatureSpace, signature_radius};
use engine_core::prelude::{EventBus, TextureId, Transform2D};
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

#[test]
fn when_seal_pressed_with_signal_then_pack_spawned_and_cards_destroyed() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(EventBus::<PhysicsCommand>::default());

    let sig = CardSignature::new([0.5; 8]);
    let card_entity = world
        .spawn(Card::face_down(TextureId(1), TextureId(2)))
        .id();
    world.get_mut::<Card>(card_entity).unwrap().signature = sig;

    // Create a reader's output jack with the signal
    let reader_jack_entity = world
        .spawn(Jack::<SignatureSpace> {
            direction: JackDirection::Output,
            data: Some(SignatureSpace::from_single(
                sig,
                signature_radius(&sig),
                card_entity,
            )),
        })
        .id();

    // Create a CardReader that has the card loaded
    let reader_entity = world
        .spawn(CardReader {
            loaded: Some(card_entity),
            half_extents: Vec2::new(30.0, 40.0),
            jack_entity: reader_jack_entity,
        })
        .id();

    // Create the booster machine
    let (device, input_jack) = spawn_booster_machine(&mut world, Vec2::ZERO);

    // Simulate cable connection: copy signal data to input jack
    let signal = world
        .get::<Jack<SignatureSpace>>(reader_jack_entity)
        .unwrap()
        .data
        .clone();
    world
        .get_mut::<Jack<SignatureSpace>>(input_jack)
        .unwrap()
        .data = signal;

    // Trigger seal
    world.insert_resource(SealButtonPressed(Some(device)));

    // Act
    let mut schedule = Schedule::default();
    schedule.add_systems(booster_seal_system);
    schedule.run(&mut world);

    // Assert
    let machine = world.get::<BoosterMachine>(device).unwrap();
    assert!(machine.output_pack.is_some());

    let pack_entity = machine.output_pack.unwrap();
    let pack = world.get::<BoosterPack>(pack_entity).unwrap();
    assert!(!pack.cards.is_empty());
    assert!(
        pack.cards.len() >= 5 && pack.cards.len() <= 15,
        "expected 5..=15 cards, got {}",
        pack.cards.len()
    );

    // Source card should be despawned
    assert!(
        world.get_entity(card_entity).is_err() || world.get::<Card>(card_entity).is_none(),
        "source card should be despawned"
    );

    // Reader should be cleared
    let reader = world.get::<CardReader>(reader_entity).unwrap();
    assert!(reader.loaded.is_none(), "reader.loaded should be None");

    // Reader jack data should be cleared
    let reader_jack = world
        .get::<Jack<SignatureSpace>>(reader_jack_entity)
        .unwrap();
    assert!(
        reader_jack.data.is_none(),
        "reader jack data should be None"
    );
}
