#![allow(clippy::unwrap_used)]

use std::sync::{Arc, Mutex};

use bevy_ecs::prelude::*;
use card_game::card::identity::definition::{
    CardAbilities, CardDefinition, CardType, art_descriptor_default,
};
use card_game::card::identity::signature::CardSignature;
use card_game::card::rendering::gpu_card_mesh::GpuCardMesh;
use card_game::card::rendering::spawn_table_card::spawn_visual_card;
use engine_render::renderer::GpuMeshHandle;
use engine_render::renderer::RendererRes;
use engine_render::testing::SpyRenderer;
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
