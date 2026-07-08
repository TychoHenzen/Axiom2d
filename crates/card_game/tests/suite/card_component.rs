#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::Entity;
use engine_core::prelude::TextureId;

use card_game::card::component::{Card, CardFaceSide, CardLabel, CardZone};
use card_game::card::identity::signature::CardSignature;

/// @doc: Card constructed via face_down() has face_up set to false
#[test]
fn when_face_down_then_face_up_is_false() {
    // Arrange / Act
    let card = Card::face_down(TextureId(0), TextureId(1));

    // Assert
    assert!(!card.face_up, "Card::face_down should set face_up to false");
}

/// @doc: Card constructed via face_down() stores both textures correctly
#[test]
fn when_face_down_then_textures_stored() {
    // Arrange
    let face = TextureId(42);
    let back = TextureId(99);

    // Act
    let card = Card::face_down(face, back);

    // Assert
    assert_eq!(
        card.face_texture, face,
        "face_texture should match the value passed to face_down"
    );
    assert_eq!(
        card.back_texture, back,
        "back_texture should match the value passed to face_down"
    );
}

/// @doc: Card::face_down() sets signature to CardSignature::default()
#[test]
fn when_card_has_default_signature_then_signature_is_not_overwritten() {
    // Arrange / Act
    let card = Card::face_down(TextureId(0), TextureId(1));

    // Assert
    assert_eq!(
        card.signature,
        CardSignature::default(),
        "Card::face_down should produce a card with the default signature"
    );
}

/// @doc: CardZone::Table is distinct from other variants and has no associated data
#[test]
fn when_zone_table_then_variant_is_table() {
    // Arrange / Act
    let zone = CardZone::Table;

    // Assert
    assert!(
        matches!(zone, CardZone::Table),
        "CardZone::Table should match itself"
    );
    assert_ne!(
        zone,
        CardZone::Hand(0),
        "Table should not equal Hand variant"
    );
}

/// @doc: CardZone::Hand stores a usize index identifying the hand position
#[test]
fn when_zone_hand_then_variant_has_index() {
    // Arrange
    let index = 3;

    // Act
    let zone = CardZone::Hand(index);

    // Assert
    assert!(
        matches!(zone, CardZone::Hand(idx) if idx == 3),
        "CardZone::Hand should store the provided index"
    );
}

/// @doc: CardZone::Stash stores page, column, and row coordinates for grid placement
#[test]
fn when_zone_stash_then_variant_has_page_col_row() {
    // Arrange
    let page: u8 = 1;
    let col: u8 = 4;
    let row: u8 = 2;

    // Act
    let zone = CardZone::Stash { page, col, row };

    // Assert
    assert!(
        matches!(zone, CardZone::Stash { page: 1, col: 4, row: 2 }),
        "CardZone::Stash should store page, col, and row"
    );
}

/// @doc: CardZone::Reader stores an Entity referencing the reader device
#[test]
fn when_zone_reader_then_variant_has_entity() {
    // Arrange
    let entity = Entity::from_raw(7);

    // Act
    let zone = CardZone::Reader(entity);

    // Assert
    assert!(
        matches!(zone, CardZone::Reader(e) if e == entity),
        "CardZone::Reader should store the provided Entity"
    );
}

/// @doc: CardLabel stores a name and description string
#[test]
fn when_label_created_then_name_and_description_stored() {
    // Arrange
    let name = "Test Card".to_string();
    let description = "A card for testing".to_string();

    // Act
    let label = CardLabel {
        name: name.clone(),
        description: description.clone(),
    };

    // Assert
    assert_eq!(
        label.name, name,
        "CardLabel.name should match the value provided at construction"
    );
    assert_eq!(
        label.description, description,
        "CardLabel.description should match the value provided at construction"
    );
}

/// @doc: CardFaceSide variants Front and Back are distinct and not equal
#[test]
fn when_card_face_side_variants_then_front_and_back_distinct() {
    // Arrange / Act
    let front = CardFaceSide::Front;
    let back = CardFaceSide::Back;

    // Assert
    assert_ne!(
        front, back,
        "CardFaceSide::Front and CardFaceSide::Back should be distinct"
    );
    assert_eq!(front, CardFaceSide::Front, "Front should equal itself");
    assert_eq!(back, CardFaceSide::Back, "Back should equal itself");
}
