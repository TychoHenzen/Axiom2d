#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::*;
use engine_render::material::Material2d;
use engine_render::shape::TessellatedColorMesh;
use glam::Vec2;

use card_game::card::rendering::baked_mesh::{BakedCardMesh, CardOverlay, CardOverlays};

/// @doc: BakedCardMesh::default() produces empty front and back meshes with no vertices or indices.
#[test]
fn when_default_baked_mesh_then_front_and_back_empty() {
    // Arrange / Act
    let mesh = BakedCardMesh::default();

    // Assert
    assert!(
        mesh.front.vertices.is_empty(),
        "default front mesh should have no vertices"
    );
    assert!(
        mesh.front.indices.is_empty(),
        "default front mesh should have no indices"
    );
    assert!(
        mesh.back.vertices.is_empty(),
        "default back mesh should have no vertices"
    );
    assert!(
        mesh.back.indices.is_empty(),
        "default back mesh should have no indices"
    );
}

/// @doc: BakedCardMesh can be constructed with explicit front/back meshes and fields are accessible.
#[test]
fn when_baked_mesh_constructed_with_meshes_then_fields_accessible() {
    // Arrange
    let front = TessellatedColorMesh {
        vertices: vec![],
        indices: vec![],
    };
    let back = TessellatedColorMesh {
        vertices: vec![],
        indices: vec![],
    };

    // Act
    let mesh = BakedCardMesh { front, back };

    // Assert
    assert_eq!(
        mesh.front.vertices.len(),
        0,
        "front vertices should match constructed value"
    );
    assert_eq!(
        mesh.back.vertices.len(),
        0,
        "back vertices should match constructed value"
    );
}

/// @doc: BakedCardMesh with non-empty meshes retains vertex and index data.
#[test]
fn when_baked_mesh_has_non_empty_meshes_then_data_preserved() {
    // Arrange
    let front = TessellatedColorMesh {
        vertices: vec![
            engine_render::shape::ColorVertex {
                position: [0.0, 0.0],
                color: [1.0; 4],
                uv: [0.0; 2],
            },
            engine_render::shape::ColorVertex {
                position: [1.0, 0.0],
                color: [1.0; 4],
                uv: [0.0; 2],
            },
            engine_render::shape::ColorVertex {
                position: [0.0; 2],
                color: [0.5; 4],
                uv: [0.0; 2],
            },
        ],
        indices: vec![0, 1, 2],
    };
    let back = TessellatedColorMesh {
        vertices: vec![],
        indices: vec![],
    };

    // Act
    let mesh = BakedCardMesh { front, back };

    // Assert
    assert_eq!(
        mesh.front.vertices.len(),
        3,
        "front should have 3 vertices"
    );
    assert_eq!(
        mesh.front.indices.len(),
        3,
        "front should have 3 indices"
    );
    assert_eq!(
        mesh.front.indices,
        vec![0, 1, 2],
        "front indices should match constructed sequence"
    );
    assert!(
        mesh.back.vertices.is_empty(),
        "back should remain empty"
    );
}

/// @doc: BakedCardMesh can be spawned as an ECS component and queried back.
#[test]
fn when_baked_mesh_spawned_in_world_then_queryable() {
    // Arrange
    let mut world = World::new();
    let mesh = BakedCardMesh::default();

    // Act
    let entity = world.spawn(mesh).id();

    // Assert
    let stored = world
        .get::<BakedCardMesh>(entity)
        .expect("spawned entity should have BakedCardMesh");
    assert!(
        stored.front.vertices.is_empty(),
        "queried front mesh should be empty after default spawn"
    );
}

/// @doc: CardOverlay constructed with quad corners and material retains field values.
#[test]
fn when_card_overlay_constructed_then_fields_accessible() {
    // Arrange
    let quad = [
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(0.0, 1.0),
    ];
    let material = Material2d::default();

    // Act
    let overlay = CardOverlay { quad, material };

    // Assert
    assert_eq!(
        overlay.quad[0],
        Vec2::new(0.0, 0.0),
        "quad corner 0 should match"
    );
    assert_eq!(
        overlay.quad[2],
        Vec2::new(1.0, 1.0),
        "quad corner 2 should match"
    );
    assert_eq!(
        overlay.material.shader,
        Material2d::default().shader,
        "material shader should match default"
    );
}

/// @doc: CardOverlay clone produces an independent copy with identical field values.
#[test]
fn when_card_overlay_cloned_then_fields_equal() {
    // Arrange
    let overlay = CardOverlay {
        quad: [Vec2::new(10.0, 20.0); 4],
        material: Material2d::default(),
    };

    // Act
    let cloned = overlay.clone();

    // Assert
    for i in 0..4 {
        assert_eq!(
            overlay.quad[i], cloned.quad[i],
            "cloned overlay quad corner {} should equal original",
            i
        );
    }
    assert_eq!(
        overlay.material, cloned.material,
        "cloned overlay material should equal original"
    );
}

/// @doc: CardOverlays::default() returns all overlay layers as None.
#[test]
fn when_default_card_overlays_then_all_layers_none() {
    // Arrange / Act
    let overlays = CardOverlays::default();

    // Assert
    assert!(
        overlays.art.is_none(),
        "default overlays should have no art layer"
    );
    assert!(
        overlays.foil.is_none(),
        "default overlays should have no foil layer"
    );
    assert!(
        overlays.back.is_none(),
        "default overlays should have no back layer"
    );
}

/// @doc: CardOverlays with art overlay set returns Some for art and None for other layers.
#[test]
fn when_art_overlay_set_then_art_some_others_none() {
    // Arrange
    let art_overlay = CardOverlay {
        quad: [Vec2::new(0.0, 0.0); 4],
        material: Material2d::default(),
    };

    // Act
    let overlays = CardOverlays {
        art: Some(art_overlay),
        ..CardOverlays::default()
    };

    // Assert
    assert!(
        overlays.art.is_some(),
        "art overlay should be Some after setting"
    );
    assert!(
        overlays.foil.is_none(),
        "foil should remain None when only art is set"
    );
    assert!(
        overlays.back.is_none(),
        "back should remain None when only art is set"
    );
}

/// @doc: CardOverlays with all three layers set returns Some for each.
#[test]
fn when_all_overlays_set_then_all_some() {
    // Arrange
    let make_overlay = || CardOverlay {
        quad: [Vec2::new(0.0, 0.0); 4],
        material: Material2d::default(),
    };

    // Act
    let overlays = CardOverlays {
        art: Some(make_overlay()),
        foil: Some(make_overlay()),
        back: Some(make_overlay()),
    };

    // Assert
    assert!(
        overlays.art.is_some(),
        "art overlay should be Some"
    );
    assert!(
        overlays.foil.is_some(),
        "foil overlay should be Some"
    );
    assert!(
        overlays.back.is_some(),
        "back overlay should be Some"
    );
}

/// @doc: CardOverlays clone produces an independent copy with matching fields.
#[test]
fn when_card_overlays_cloned_then_fields_equal() {
    // Arrange
    let overlay = CardOverlay {
        quad: [Vec2::new(5.0, 10.0); 4],
        material: Material2d::default(),
    };
    let overlays = CardOverlays {
        art: Some(overlay.clone()),
        foil: Some(overlay),
        back: None,
    };

    // Act
    let cloned = overlays.clone();

    // Assert
    assert!(
        cloned.art.is_some(),
        "cloned overlays should retain art"
    );
    assert!(
        cloned.foil.is_some(),
        "cloned overlays should retain foil"
    );
    assert!(
        cloned.back.is_none(),
        "cloned overlays should retain None back"
    );
    assert_eq!(
        overlays.art.as_ref().unwrap().quad,
        cloned.art.as_ref().unwrap().quad,
        "cloned art overlay quad should equal original"
    );
}

/// @doc: CardOverlays can be spawned as an ECS component and queried back.
#[test]
fn when_card_overlays_spawned_in_world_then_queryable() {
    // Arrange
    let mut world = World::new();
    let overlays = CardOverlays::default();

    // Act
    let entity = world.spawn(overlays).id();

    // Assert
    let stored = world
        .get::<CardOverlays>(entity)
        .expect("spawned entity should have CardOverlays");
    assert!(
        stored.art.is_none(),
        "default overlays should be None after spawn"
    );
}

/// @doc: BakedCardMesh and CardOverlays can coexist on the same entity.
#[test]
fn when_both_components_spawned_together_then_both_queryable() {
    // Arrange
    let mut world = World::new();
    let baked = BakedCardMesh::default();
    let overlays = CardOverlays::default();

    // Act
    let entity = world.spawn((baked, overlays)).id();

    // Assert
    assert!(
        world.get::<BakedCardMesh>(entity).is_some(),
        "entity should have BakedCardMesh"
    );
    assert!(
        world.get::<CardOverlays>(entity).is_some(),
        "entity should have CardOverlays"
    );
}
