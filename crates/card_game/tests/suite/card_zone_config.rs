#![allow(clippy::unwrap_used)]

use card_game::card::component::CardZone;
use card_game::card::zone_config::ZoneConfig;
use engine_scene::prelude::RenderLayer;

/// @doc: Hand cards have no physics — they float in screen-space UI and can't be knocked by table collisions
#[test]
fn when_zone_is_hand_then_config_has_no_physics_and_ui_layer() {
    // Arrange / Act
    let config = ZoneConfig::for_zone(&CardZone::Hand(0));

    // Assert
    assert!(!config.has_physics);
    assert_eq!(config.render_layer, RenderLayer::UI);
    assert!(!config.has_item_form);
}

/// @doc: Table cards have physics bodies and render in World layer — they
/// participate in collisions and are drawn behind UI elements like the hand
/// and stash. Removing physics would make table cards immovable; wrong layer
/// would draw them on top of the hand fan.
#[test]
fn when_zone_is_table_then_config_has_physics_and_world_layer() {
    // Arrange / Act
    let config = ZoneConfig::for_zone(&CardZone::Table);

    // Assert
    assert!(config.has_physics);
    assert_eq!(config.render_layer, RenderLayer::World);
    assert!(!config.has_item_form);
}

/// @doc: Stash cards use item-form rendering — compact slot appearance instead of full card geometry
#[test]
fn when_zone_is_stash_then_config_has_item_form_and_ui_layer() {
    // Arrange / Act
    let config = ZoneConfig::for_zone(&CardZone::Stash {
        page: 0,
        col: 0,
        row: 0,
    });

    // Assert
    assert!(!config.has_physics);
    assert_eq!(config.render_layer, RenderLayer::UI);
    assert!(config.has_item_form);
}
