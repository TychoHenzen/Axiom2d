#![allow(clippy::unwrap_used)]

use card_game::card::identity::base_type::{
    BaseCardType, BaseCardTypeRegistry, CardCategory, populate_default_types,
};
use card_game::card::identity::signature::CardSignature;

#[test]
fn when_signature_is_identical_to_base_then_can_match_returns_true() {
    // Arrange
    let base_sig = CardSignature::new([0.5, -0.3, 0.1, 0.9, -0.7, 0.2, -0.4, 0.6]);
    let base_type = BaseCardType {
        name: "Test".to_string(),
        base_signature: base_sig,
        match_radius: 1.0,
        category: CardCategory::Skill,
        modifiers: vec![],
    };
    let card_sig = base_sig; // identical — distance == 0.0

    // Act
    let result = base_type.can_match(&card_sig);

    // Assert
    assert!(result);
}

#[test]
fn when_signature_is_well_inside_radius_then_can_match_returns_true() {
    // Arrange
    let base_type = BaseCardType {
        name: "Test".to_string(),
        base_signature: CardSignature::new([0.0; 8]),
        match_radius: 2.0,
        category: CardCategory::Equipment,
        modifiers: vec![],
    };
    let card_sig = CardSignature::new([0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]); // distance = 0.5

    // Act
    let result = base_type.can_match(&card_sig);

    // Assert
    assert!(result);
}

/// @doc: Boundary-inclusive matching — cards exactly at the radius edge still belong to the base type
#[test]
fn when_signature_is_exactly_at_radius_boundary_then_can_match_returns_true() {
    // Arrange
    let base_type = BaseCardType {
        name: "Test".to_string(),
        base_signature: CardSignature::new([0.0; 8]),
        match_radius: 1.0,
        category: CardCategory::Skill,
        modifiers: vec![],
    };
    let card_sig = CardSignature::new([1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]); // distance = 1.0

    // Act
    let result = base_type.can_match(&card_sig);

    // Assert
    assert!(result);
}

#[test]
fn when_signature_is_outside_radius_then_can_match_returns_false() {
    // Arrange
    let base_type = BaseCardType {
        name: "Test".to_string(),
        base_signature: CardSignature::new([0.0; 8]),
        match_radius: 1.0,
        category: CardCategory::Playstyle,
        modifiers: vec![],
    };
    // distance = sqrt(0.72^2 + 0.72^2) ~ 1.018 > 1.0
    let card_sig = CardSignature::new([0.72, 0.72, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);

    // Act
    let result = base_type.can_match(&card_sig);

    // Assert
    assert!(!result);
}

/// @doc: Match weight is 1.0 at center, 0.0 at edge — linear falloff drives "how strongly" a card belongs to a type
#[test]
fn when_signature_is_identical_to_base_then_match_weight_is_one() {
    // Arrange
    let base_sig = CardSignature::new([0.5, -0.3, 0.1, 0.9, -0.7, 0.2, -0.4, 0.6]);
    let base_type = BaseCardType {
        name: "Test".to_string(),
        base_signature: base_sig,
        match_radius: 1.0,
        category: CardCategory::Skill,
        modifiers: vec![],
    };

    // Act
    let weight = base_type.match_weight(&base_sig);

    // Assert
    assert!((weight - 1.0).abs() < 1e-5);
}

#[test]
fn when_signature_is_halfway_to_radius_then_match_weight_is_half() {
    // Arrange
    let base_type = BaseCardType {
        name: "Test".to_string(),
        base_signature: CardSignature::new([0.0; 8]),
        match_radius: 2.0,
        category: CardCategory::Equipment,
        modifiers: vec![],
    };
    let card_sig = CardSignature::new([1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]); // distance = 1.0

    // Act
    let weight = base_type.match_weight(&card_sig);

    // Assert
    assert!((weight - 0.5).abs() < 1e-5);
}

#[test]
fn when_signature_is_at_radius_boundary_then_match_weight_is_zero() {
    // Arrange
    let base_type = BaseCardType {
        name: "Test".to_string(),
        base_signature: CardSignature::new([0.0; 8]),
        match_radius: 1.0,
        category: CardCategory::Skill,
        modifiers: vec![],
    };
    let card_sig = CardSignature::new([1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]); // distance = 1.0

    // Act
    let weight = base_type.match_weight(&card_sig);

    // Assert
    assert!((weight - 0.0).abs() < 1e-5);
}

#[test]
fn when_signature_is_outside_radius_then_match_weight_is_zero() {
    // Arrange
    let base_type = BaseCardType {
        name: "Test".to_string(),
        base_signature: CardSignature::new([0.0; 8]),
        match_radius: 1.0,
        category: CardCategory::Playstyle,
        modifiers: vec![],
    };
    let card_sig = CardSignature::new([0.72, 0.72, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]); // distance ~ 1.018

    // Act
    let weight = base_type.match_weight(&card_sig);

    // Assert
    assert!((weight - 0.0).abs() < 1e-5);
}

fn make_base_type(name: &str, axes: [f32; 8], radius: f32, category: CardCategory) -> BaseCardType {
    BaseCardType {
        name: name.to_string(),
        base_signature: CardSignature::new(axes),
        match_radius: radius,
        category,
        modifiers: vec![],
    }
}

#[test]
fn when_registry_is_empty_then_best_match_returns_none() {
    // Arrange
    let registry = BaseCardTypeRegistry::new();
    let sig = CardSignature::new([0.0; 8]);

    // Act
    let result = registry.best_match(&sig);

    // Assert
    assert!(result.is_none());
}

#[test]
fn when_registry_has_one_matching_type_then_best_match_returns_it() {
    // Arrange
    let mut registry = BaseCardTypeRegistry::new();
    registry.register(make_base_type(
        "Weapon",
        [0.0; 8],
        2.0,
        CardCategory::Equipment,
    ));
    let sig = CardSignature::new([0.1; 8]); // distance = sqrt(8)*0.1 ~ 0.283, within radius 2.0

    // Act
    let result = registry.best_match(&sig);

    // Assert
    assert_eq!(result.expect("should match").name, "Weapon");
}

#[test]
fn when_signature_is_outside_only_registered_type_then_best_match_returns_none() {
    // Arrange
    let mut registry = BaseCardTypeRegistry::new();
    registry.register(make_base_type(
        "Weapon",
        [0.0; 8],
        0.5,
        CardCategory::Equipment,
    ));
    let sig = CardSignature::new([1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]); // distance = 1.0 > 0.5

    // Act
    let result = registry.best_match(&sig);

    // Assert
    assert!(result.is_none());
}

/// @doc: Closest match wins — when a card falls in multiple type radii, the highest-weight type determines identity
#[test]
fn when_two_types_both_match_then_best_match_returns_closer_one() {
    // Arrange
    let mut registry = BaseCardTypeRegistry::new();
    registry.register(make_base_type(
        "TypeA",
        [0.0; 8],
        2.0,
        CardCategory::Equipment,
    ));
    registry.register(make_base_type("TypeB", [0.5; 8], 2.0, CardCategory::Skill));
    let sig = CardSignature::new([0.1; 8]); // closer to TypeA

    // Act
    let result = registry.best_match(&sig);

    // Assert
    assert_eq!(result.expect("should match").name, "TypeA");
}

/// @doc: Registration order doesn't affect matching — `best_match` compares all types, not first-registered
#[test]
fn when_best_match_is_registered_second_then_it_still_wins() {
    // Arrange
    let mut registry = BaseCardTypeRegistry::new();
    registry.register(make_base_type(
        "TypeA",
        [0.0; 8],
        2.0,
        CardCategory::Equipment,
    ));
    registry.register(make_base_type("TypeB", [0.1; 8], 2.0, CardCategory::Skill));
    let sig = CardSignature::new([0.1; 8]); // exact match for TypeB

    // Act
    let result = registry.best_match(&sig);

    // Assert
    assert_eq!(result.expect("should match").name, "TypeB");
}

#[test]
fn when_only_one_of_multiple_types_is_in_radius_then_best_match_returns_it() {
    // Arrange
    let mut registry = BaseCardTypeRegistry::new();
    registry.register(make_base_type(
        "Near",
        [0.0; 8],
        0.5,
        CardCategory::Equipment,
    ));
    registry.register(make_base_type("Far", [1.0; 8], 0.1, CardCategory::Skill));
    let sig = CardSignature::new([0.1; 8]); // within Near's radius, outside Far's

    // Act
    let result = registry.best_match(&sig);

    // Assert
    assert_eq!(result.expect("should match").name, "Near");
}

#[test]
fn when_registry_populated_with_defaults_then_five_archetypes_findable() {
    // Arrange
    let mut registry = BaseCardTypeRegistry::new();

    // Act
    populate_default_types(&mut registry);

    // Assert — each archetype's exact base signature should match itself
    let weapon_sig = CardSignature::new([0.8, 0.3, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    let spell_sig = CardSignature::new([0.0, 0.8, 0.0, 0.3, 0.0, 0.0, 0.0, 0.0]);
    let shield_sig = CardSignature::new([0.8, 0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0]);
    let healer_sig = CardSignature::new([0.0, 0.0, 0.0, 0.3, 0.0, 0.0, 0.8, 0.0]);
    let scout_sig = CardSignature::new([0.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.8]);

    assert_eq!(registry.best_match(&weapon_sig).unwrap().name, "Weapon");
    assert_eq!(registry.best_match(&spell_sig).unwrap().name, "Spell");
    assert_eq!(registry.best_match(&shield_sig).unwrap().name, "Shield");
    assert_eq!(registry.best_match(&healer_sig).unwrap().name, "Healer");
    assert_eq!(registry.best_match(&scout_sig).unwrap().name, "Scout");
}

#[test]
fn when_type_is_registered_then_it_participates_in_best_match() {
    // Arrange
    let mut registry = BaseCardTypeRegistry::new();
    assert!(registry.best_match(&CardSignature::new([0.0; 8])).is_none());

    // Act
    registry.register(make_base_type(
        "Added",
        [0.0; 8],
        1.0,
        CardCategory::Playstyle,
    ));

    // Assert
    let result = registry.best_match(&CardSignature::new([0.0; 8]));
    assert_eq!(result.expect("should match").name, "Added");
}
