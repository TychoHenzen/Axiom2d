#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::*;
use card_game::card::identity::definition::{
    CardAbilities, CardDefinition, CardType, art_descriptor_default,
};
use card_game::card::identity::signature::CardSignature;
use card_game::card::rendering::art_shader::{CardArtShader, GemShader};
use card_game::card::rendering::geometry::{
    TABLE_CARD_HEIGHT as CARD_HEIGHT, TABLE_CARD_WIDTH as CARD_WIDTH,
};
use card_game::card::rendering::spawn_table_card::spawn_visual_card;
use engine_render::prelude::ShaderHandle;
use engine_render::shape::MeshOverlays;
use glam::Vec2;

fn make_test_def() -> CardDefinition {
    CardDefinition {
        card_type: CardType::Spell,
        name: "Fireball".to_owned(),
        stats: None,
        abilities: CardAbilities {
            keywords: vec![],
            text: "Deal 3 damage".to_owned(),
        },
        art: art_descriptor_default(CardType::Spell),
    }
}

fn spawn_def(world: &mut World, def: &CardDefinition, face_up: bool) -> Entity {
    spawn_visual_card(
        world,
        def,
        Vec2::ZERO,
        Vec2::new(CARD_WIDTH, CARD_HEIGHT),
        face_up,
        CardSignature::default(),
    )
}

/// @doc: When no shader resources are present, MeshOverlays is present but empty.
#[test]
fn when_no_shader_resources_then_overlays_empty() {
    // Arrange
    let mut world = World::new();
    let def = make_test_def();

    // Act
    let entity = spawn_def(&mut world, &def, true);

    // Assert
    let overlays = world
        .get::<MeshOverlays>(entity)
        .expect("spawned card should always have MeshOverlays component");
    assert!(
        overlays.0.is_empty(),
        "expected empty overlays without shader resources, got {} entries",
        overlays.0.len()
    );
}

/// @doc: When CardArtShader resource exists, the art overlay entry is included.
#[test]
fn when_card_art_shader_exists_then_art_overlay_present() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(CardArtShader(ShaderHandle(0)));
    let def = make_test_def();

    // Act
    let entity = spawn_def(&mut world, &def, true);

    // Assert
    let overlays = world
        .get::<MeshOverlays>(entity)
        .expect("spawned card should have MeshOverlays");
    assert!(
        !overlays.0.is_empty(),
        "expected at least one overlay entry when CardArtShader exists"
    );
}

/// @doc: The art overlay entry respects face_up=false by setting visible=false.
#[test]
fn when_face_down_then_art_overlay_not_visible() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(CardArtShader(ShaderHandle(0)));
    let def = make_test_def();

    // Act
    let entity = spawn_def(&mut world, &def, false);

    // Assert
    let overlays = world
        .get::<MeshOverlays>(entity)
        .expect("spawned card should have MeshOverlays");
    let art = &overlays.0[0];
    assert!(art.front_only, "art overlay should be front_only");
    assert!(
        !art.visible,
        "art overlay should not be visible when card is face_down"
    );
}

/// @doc: The art overlay entry is visible when face_up is true.
#[test]
fn when_face_up_then_art_overlay_visible() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(CardArtShader(ShaderHandle(0)));
    let def = make_test_def();

    // Act
    let entity = spawn_def(&mut world, &def, true);

    // Assert
    let overlays = world
        .get::<MeshOverlays>(entity)
        .expect("spawned card should have MeshOverlays");
    let art = &overlays.0[0];
    assert!(art.visible, "art overlay should be visible when card is face_up");
}

/// @doc: When GemShader resource exists, eight gem overlay entries are created (one per element).
#[test]
fn when_gem_shader_exists_then_eight_gem_overlays_present() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(GemShader(ShaderHandle(0)));
    let def = make_test_def();

    // Act
    let entity = spawn_def(&mut world, &def, true);

    // Assert
    let overlays = world
        .get::<MeshOverlays>(entity)
        .expect("spawned card should have MeshOverlays");
    assert_eq!(
        overlays.0.len(),
        8,
        "expected 8 gem overlays for 8 elements"
    );
}

/// @doc: Gem overlays are front_only and not visible when card is face_down.
#[test]
fn when_face_down_then_gem_overlays_not_visible() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(GemShader(ShaderHandle(0)));
    let def = make_test_def();

    // Act
    let entity = spawn_def(&mut world, &def, false);

    // Assert
    let overlays = world
        .get::<MeshOverlays>(entity)
        .expect("spawned card should have MeshOverlays");
    for (i, gem) in overlays.0.iter().enumerate() {
        assert!(gem.front_only, "gem overlay {} should be front_only", i);
        assert!(
            !gem.visible,
            "gem overlay {} should not be visible when card is face_down",
            i
        );
    }
}

/// @doc: Gem overlays are visible when card is face_up.
#[test]
fn when_face_up_then_gem_overlays_visible() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(GemShader(ShaderHandle(0)));
    let def = make_test_def();

    // Act
    let entity = spawn_def(&mut world, &def, true);

    // Assert
    let overlays = world
        .get::<MeshOverlays>(entity)
        .expect("spawned card should have MeshOverlays");
    for (i, gem) in overlays.0.iter().enumerate() {
        assert!(
            gem.visible,
            "gem overlay {} should be visible when card is face_up",
            i
        );
    }
}

/// @doc: With both CardArtShader and GemShader, overlays contain art and gem entries combined.
#[test]
fn when_art_and_gem_shaders_exist_then_combined_overlays_present() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(CardArtShader(ShaderHandle(0)));
    world.insert_resource(GemShader(ShaderHandle(0)));
    let def = make_test_def();

    // Act
    let entity = spawn_def(&mut world, &def, true);

    // Assert
    let overlays = world
        .get::<MeshOverlays>(entity)
        .expect("spawned card should have MeshOverlays");
    // 1 art + 8 gems = 9; more if variant/condition overlays also apply
    assert!(
        overlays.0.len() >= 9,
        "expected at least 9 overlay entries (1 art + 8 gems), got {}",
        overlays.0.len()
    );
}
