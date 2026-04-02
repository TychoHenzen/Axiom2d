#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::World;
use card_game::hand::*;

#[test]
fn when_adding_first_card_then_returns_index_zero() {
    // Arrange
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut hand = Hand::new(5);

    // Act
    let result = hand.add(entity);

    // Assert
    assert_eq!(result.unwrap(), 0);
}

#[test]
fn when_adding_second_card_then_returns_index_one() {
    // Arrange
    let mut world = World::new();
    let first = world.spawn_empty().id();
    let second = world.spawn_empty().id();
    let mut hand = Hand::new(5);
    hand.add(first).unwrap();

    // Act
    let result = hand.add(second);

    // Assert
    assert_eq!(result.unwrap(), 1);
}

/// @doc: Hand capacity is a hard boundary — prevents silent card drops and enforces discard mechanics when adding to a full hand.
#[test]
fn when_adding_card_to_full_hand_then_returns_hand_full_error() {
    // Arrange
    let mut world = World::new();
    let first = world.spawn_empty().id();
    let second = world.spawn_empty().id();
    let overflow = world.spawn_empty().id();
    let mut hand = Hand::new(2);
    hand.add(first).unwrap();
    hand.add(second).unwrap();

    // Act
    let result = hand.add(overflow);

    // Assert
    assert!(result.is_err());
}

#[test]
fn when_hand_is_full_then_is_full_returns_true() {
    // Arrange
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut hand = Hand::new(1);
    hand.add(entity).unwrap();

    // Act / Assert
    assert!(hand.is_full());
}

#[test]
fn when_hand_is_not_full_then_is_full_returns_false() {
    // Arrange
    let hand = Hand::new(3);

    // Act / Assert
    assert!(!hand.is_full());
}

#[test]
fn when_removing_a_present_card_then_returns_its_former_index() {
    // Arrange
    let mut world = World::new();
    let a = world.spawn_empty().id();
    let b = world.spawn_empty().id();
    let c = world.spawn_empty().id();
    let mut hand = Hand::new(5);
    hand.add(a).unwrap();
    hand.add(b).unwrap();
    hand.add(c).unwrap();

    // Act
    let result = hand.remove(b);

    // Assert
    assert_eq!(result, Some(1));
}

#[test]
fn when_removing_an_unknown_entity_then_returns_none() {
    // Arrange
    let mut world = World::new();
    let known = world.spawn_empty().id();
    let unknown = world.spawn_empty().id();
    let mut hand = Hand::new(5);
    hand.add(known).unwrap();

    // Act
    let result = hand.remove(unknown);

    // Assert
    assert_eq!(result, None);
}

/// @doc: Removing a card must not shuffle the hand — preserving order ensures the fan layout does not glitch when cards are discarded mid-game.
#[test]
fn when_removing_a_card_then_remaining_cards_preserve_relative_order() {
    // Arrange
    let mut world = World::new();
    let a = world.spawn_empty().id();
    let b = world.spawn_empty().id();
    let c = world.spawn_empty().id();
    let mut hand = Hand::new(5);
    hand.add(a).unwrap();
    hand.add(b).unwrap();
    hand.add(c).unwrap();

    // Act
    hand.remove(a);

    // Assert
    assert_eq!(hand.cards(), &[b, c]);
}

#[test]
fn when_removing_a_card_then_len_decrements_by_one() {
    // Arrange
    let mut world = World::new();
    let a = world.spawn_empty().id();
    let b = world.spawn_empty().id();
    let mut hand = Hand::new(5);
    hand.add(a).unwrap();
    hand.add(b).unwrap();

    // Act
    hand.remove(a);

    // Assert
    assert_eq!(hand.len(), 1);
}

#[test]
fn when_cards_added_in_order_then_cards_slice_reflects_insertion_order() {
    // Arrange
    let mut world = World::new();
    let a = world.spawn_empty().id();
    let b = world.spawn_empty().id();
    let c = world.spawn_empty().id();
    let mut hand = Hand::new(3);
    hand.add(a).unwrap();
    hand.add(b).unwrap();
    hand.add(c).unwrap();

    // Act / Assert
    assert_eq!(hand.cards(), &[a, b, c]);
}
