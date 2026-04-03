#![allow(clippy::unwrap_used)]

use std::sync::{Arc, Mutex};

use bevy_ecs::prelude::*;
use card_game::card::component::Card;
use card_game::card::component::CardItemForm;
use card_game::card::identity::definition::{
    CardAbilities, CardDefinition, CardType, art_descriptor_default,
};
use card_game::card::identity::signature::CardSignature;
use card_game::card::rendering::baked_render::baked_card_render_system;
use card_game::card::rendering::gpu_card_mesh::GpuCardMesh;
use card_game::card::rendering::spawn_table_card::spawn_visual_card;
use engine_core::prelude::{TextureId, Transform2D};
use engine_render::renderer::GpuMeshHandle;
use engine_render::renderer::RendererRes;
use engine_render::shape::ColorMesh;
use engine_render::testing::{PersistentMeshCallLog, SpyRenderer};
use engine_scene::prelude::{GlobalTransform2D, RenderLayer, SortOrder};
use glam::Vec2;

fn placeholder_def() -> CardDefinition {
    CardDefinition {
        card_type: CardType::Creature,
        name: String::new(),
        stats: None,
        abilities: CardAbilities {
            keywords: vec![],
            text: String::new(),
        },
        art: art_descriptor_default(CardType::Creature),
    }
}

#[test]
fn when_card_spawned_then_gpu_card_mesh_uploaded() {
    // Arrange
    let log = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(log);
    let mut world = World::new();
    world.insert_resource(RendererRes::new(Box::new(spy)));

    // Act
    let entity = spawn_visual_card(
        &mut world,
        &placeholder_def(),
        Vec2::ZERO,
        Vec2::new(60.0, 90.0),
        true,
        CardSignature::default(),
    );

    // Assert
    let gpu_mesh = world.get::<GpuCardMesh>(entity).unwrap();
    assert_ne!(
        gpu_mesh.front,
        GpuMeshHandle(0),
        "front handle must be non-zero"
    );
    assert_ne!(
        gpu_mesh.back,
        GpuMeshHandle(0),
        "back handle must be non-zero"
    );
    assert_ne!(
        gpu_mesh.front, gpu_mesh.back,
        "front and back must be distinct"
    );
}

#[test]
fn when_card_spawned_with_gpu_mesh_then_color_mesh_is_not_inserted() {
    // Arrange
    let log = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(log);
    let mut world = World::new();
    world.insert_resource(RendererRes::new(Box::new(spy)));

    // Act
    let entity = spawn_visual_card(
        &mut world,
        &placeholder_def(),
        Vec2::ZERO,
        Vec2::new(60.0, 90.0),
        true,
        CardSignature::default(),
    );

    // Assert
    assert!(
        world.get::<ColorMesh>(entity).is_none(),
        "GPU-backed cards should not insert a fallback ColorMesh"
    );
}

fn run_baked_card_render(world: &mut World) {
    let mut schedule = bevy_ecs::schedule::Schedule::default();
    schedule.add_systems(baked_card_render_system);
    schedule.run(world);
}

fn spawn_card_for_render(
    world: &mut World,
    face_up: bool,
    front_handle: GpuMeshHandle,
    back_handle: GpuMeshHandle,
) -> Entity {
    let sig = CardSignature::default();
    world
        .spawn((
            Card {
                face_texture: TextureId(0),
                back_texture: TextureId(0),
                face_up,
                signature: sig,
            },
            GpuCardMesh {
                front: front_handle,
                back: back_handle,
            },
            Transform2D::default(),
            GlobalTransform2D(glam::Affine2::IDENTITY),
            RenderLayer::World,
            SortOrder::default(),
        ))
        .id()
}

#[test]
fn when_card_face_up_then_draws_front_handle() {
    // Arrange
    let calls: PersistentMeshCallLog = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(Arc::new(Mutex::new(Vec::new())))
        .with_persistent_mesh_capture(Arc::clone(&calls));
    let mut world = World::new();
    world.insert_resource(RendererRes::new(Box::new(spy)));
    let front_handle = GpuMeshHandle(1);
    let back_handle = GpuMeshHandle(2);
    spawn_card_for_render(&mut world, true, front_handle, back_handle);

    // Act
    run_baked_card_render(&mut world);

    // Assert
    let recorded = calls.lock().unwrap();
    assert!(
        recorded.iter().any(|(h, _)| *h == front_handle),
        "face-up card must draw front handle"
    );
    assert!(
        !recorded.iter().any(|(h, _)| *h == back_handle),
        "face-up card must not draw back handle"
    );
}

#[test]
fn when_card_face_down_then_draws_back_handle() {
    // Arrange
    let calls: PersistentMeshCallLog = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(Arc::new(Mutex::new(Vec::new())))
        .with_persistent_mesh_capture(Arc::clone(&calls));
    let mut world = World::new();
    world.insert_resource(RendererRes::new(Box::new(spy)));
    let front_handle = GpuMeshHandle(1);
    let back_handle = GpuMeshHandle(2);
    spawn_card_for_render(&mut world, false, front_handle, back_handle);

    // Act
    run_baked_card_render(&mut world);

    // Assert
    let recorded = calls.lock().unwrap();
    assert!(
        recorded.iter().any(|(h, _)| *h == back_handle),
        "face-down card must draw back handle"
    );
    assert!(
        !recorded.iter().any(|(h, _)| *h == front_handle),
        "face-down card must not draw front handle"
    );
}

#[test]
fn when_card_has_item_form_then_not_drawn() {
    // Arrange
    let calls: PersistentMeshCallLog = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(Arc::new(Mutex::new(Vec::new())))
        .with_persistent_mesh_capture(Arc::clone(&calls));
    let mut world = World::new();
    world.insert_resource(RendererRes::new(Box::new(spy)));
    let sig = CardSignature::default();
    world.spawn((
        Card {
            face_texture: TextureId(0),
            back_texture: TextureId(0),
            face_up: true,
            signature: sig,
        },
        GpuCardMesh {
            front: GpuMeshHandle(1),
            back: GpuMeshHandle(2),
        },
        CardItemForm,
        Transform2D::default(),
        GlobalTransform2D(glam::Affine2::IDENTITY),
        RenderLayer::World,
        SortOrder::default(),
    ));

    // Act
    run_baked_card_render(&mut world);

    // Assert
    let recorded = calls.lock().unwrap();
    assert!(
        recorded.is_empty(),
        "item-form card must not emit a persistent draw call"
    );
}
