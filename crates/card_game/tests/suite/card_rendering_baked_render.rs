#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::*;
use card_game::card::rendering::baked_render::baked_card_sync_system;
use engine_render::shape::{ColorMesh, MeshOverlays, OverlayEntry};
use glam::Vec2;

use card_game::card::component::Card;
use card_game::card::component::CardItemForm;
use card_game::card::component::CardLabel;
use card_game::card::identity::signature::CardSignature;
use card_game::card::rendering::bake::{bake_back_face, bake_front_face};
use card_game::card::rendering::baked_mesh::BakedCardMesh;
use engine_core::prelude::TextureId;

fn make_baked() -> BakedCardMesh {
    let label = CardLabel {
        name: "Test".to_owned(),
        description: "Desc".to_owned(),
    };
    let size = Vec2::new(60.0, 90.0);
    let sig = CardSignature::default();
    BakedCardMesh {
        front: bake_front_face(&sig, size, &label, None),
        back: bake_back_face(size),
    }
}

fn make_card(face_up: bool) -> Card {
    Card {
        face_texture: TextureId(0),
        back_texture: TextureId(0),
        face_up,
        signature: CardSignature::default(),
    }
}

fn run(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(baked_card_sync_system);
    schedule.run(world);
}

#[test]
fn when_face_up_then_color_mesh_matches_front() {
    // Arrange
    let mut world = World::new();
    let baked = make_baked();
    let expected_len = baked.front.vertices.len();
    world.spawn((baked, make_card(true), ColorMesh::default()));

    // Act
    run(&mut world);

    // Assert
    let mut q = world.query::<&ColorMesh>();
    let mesh = q.single(&world).unwrap();
    assert_eq!(mesh.vertices.len(), expected_len);
}

/// @doc: Cards in `ItemForm` (stash icon grid) must have no ColorMesh—they are rendered as small icons,
/// not full-size cards. Without this check, item-form cards would render twice (icon + full mesh).
#[test]
fn when_card_has_item_form_then_color_mesh_is_empty() {
    // Arrange
    let mut world = World::new();
    world.spawn((
        make_baked(),
        make_card(true),
        ColorMesh::default(),
        CardItemForm,
    ));

    // Act
    run(&mut world);

    // Assert
    let mut q = world.query::<&ColorMesh>();
    let mesh = q.single(&world).unwrap();
    assert!(
        mesh.is_empty(),
        "item-form card ColorMesh must have no vertices"
    );
}

/// @doc: Art shader overlays must be hidden for stash item-form cards, matching the `ColorMesh` behavior.
/// Overlays include art shader and variant shader effects; hiding them prevents visual pollution in the dense grid.
#[test]
fn when_card_has_item_form_then_overlays_hidden() {
    // Arrange
    let mut world = World::new();
    let overlay = MeshOverlays(vec![OverlayEntry {
        mesh: engine_render::shape::TessellatedColorMesh::new(),
        material: engine_render::material::Material2d::default(),
        visible: true,
        front_only: true,
    }]);
    world.spawn((
        make_baked(),
        make_card(true),
        ColorMesh::default(),
        overlay,
        CardItemForm,
    ));

    // Act
    run(&mut world);

    // Assert
    let mut q = world.query::<&MeshOverlays>();
    let overlays = q.single(&world).unwrap();
    assert!(
        overlays.0.iter().all(|e| !e.visible),
        "item-form card overlays must all be hidden"
    );
}

#[test]
fn when_card_has_item_form_face_down_then_color_mesh_is_empty() {
    // Arrange
    let mut world = World::new();
    world.spawn((
        make_baked(),
        make_card(false),
        ColorMesh::default(),
        CardItemForm,
    ));

    // Act
    run(&mut world);

    // Assert
    let mut q = world.query::<&ColorMesh>();
    let mesh = q.single(&world).unwrap();
    assert!(
        mesh.is_empty(),
        "item-form face-down card must have no vertices"
    );
}

/// @doc: Removing `ItemForm` component must restore the card's full mesh on the next sync.
/// This test exercises the full transition path: Table → Stash (`ItemForm` added, mesh cleared) → Hand (`ItemForm` removed, mesh restored).
#[test]
fn when_item_form_removed_then_mesh_restored_on_next_sync() {
    // Arrange — spawn card with ItemForm, run system to clear mesh
    let mut world = World::new();
    let baked = make_baked();
    let expected_len = baked.front.vertices.len();
    let entity = world
        .spawn((baked, make_card(true), ColorMesh::default(), CardItemForm))
        .id();
    run(&mut world);
    // Confirm mesh is empty after first sync
    let mut q = world.query::<&ColorMesh>();
    assert!(q.get(&world, entity).unwrap().is_empty());

    // Act — remove ItemForm (no need to touch Card — system runs every frame)
    world.entity_mut(entity).remove::<CardItemForm>();
    run(&mut world);

    // Assert
    let mut q = world.query::<&ColorMesh>();
    let mesh = q.get(&world, entity).unwrap();
    assert_eq!(
        mesh.vertices.len(),
        expected_len,
        "mesh must be restored after leaving stash"
    );
}

/// @doc: The sync system must re-zero `ColorMesh` every frame for stash cards,
/// not just on the frame `CardItemForm` is inserted. Without per-frame enforcement,
/// a change-detection gap between Update (where `CardItemForm` is inserted via commands)
/// and `PostUpdate` (where the sync runs) can leave the mesh populated, causing the
/// full card to visibly render behind the stash grid — even when the stash is closed.
#[test]
fn when_item_form_present_and_mesh_repopulated_then_sync_re_zeros_every_frame() {
    // Arrange — persistent schedule simulating real game loop (reused across frames)
    let mut world = World::new();
    let mut schedule = Schedule::default();
    schedule.add_systems(baked_card_sync_system);

    let baked = make_baked();
    let front_clone = baked.front.clone();
    let entity = world
        .spawn((baked, make_card(true), ColorMesh::default(), CardItemForm))
        .id();
    schedule.run(&mut world); // frame 1: zeros mesh, advances change ticks

    // Simulate another system repopulating the mesh (or a change detection gap)
    world.get_mut::<ColorMesh>(entity).unwrap().0 = front_clone;
    assert!(
        !world.get::<ColorMesh>(entity).unwrap().is_empty(),
        "precondition: mesh must be non-empty before re-sync"
    );

    // Act — re-run the SAME schedule (same system state, advanced ticks)
    schedule.run(&mut world);

    // Assert
    let mesh = world.get::<ColorMesh>(entity).unwrap();
    assert!(
        mesh.is_empty(),
        "ColorMesh must be re-zeroed every frame while CardItemForm is present"
    );
}

#[test]
fn when_face_down_then_color_mesh_matches_back() {
    // Arrange
    let mut world = World::new();
    let baked = make_baked();
    let expected_len = baked.back.vertices.len();
    world.spawn((baked, make_card(false), ColorMesh::default()));

    // Act
    run(&mut world);

    // Assert
    let mut q = world.query::<&ColorMesh>();
    let mesh = q.single(&world).unwrap();
    assert_eq!(mesh.vertices.len(), expected_len);
}
