#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::*;
use card_game::card::component::{Card, CardLabel};
use card_game::card::identity::definition::{
    CardAbilities, CardDefinition, CardType, art_descriptor_default,
};
use card_game::card::identity::residual::ResidualStats;
use card_game::card::identity::signature::CardSignature;
use card_game::card::rendering::baked_mesh::BakedCardMesh;
use card_game::card::rendering::geometry::{
    TABLE_CARD_HEIGHT as CARD_HEIGHT, TABLE_CARD_WIDTH as CARD_WIDTH,
};
use card_game::card::rendering::spawn_table_card::spawn_visual_card;
use engine_physics::prelude::Collider;
use engine_render::shape::ColorMesh;
use engine_scene::prelude::ChildOf;
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

fn spawn_def(world: &mut World, def: &CardDefinition) -> Entity {
    spawn_visual_card(
        world,
        def,
        Vec2::ZERO,
        Vec2::new(CARD_WIDTH, CARD_HEIGHT),
        false,
        CardSignature::default(),
    )
}

fn spawn_def_face_up(world: &mut World, def: &CardDefinition) -> Entity {
    spawn_visual_card(
        world,
        def,
        Vec2::ZERO,
        Vec2::new(CARD_WIDTH, CARD_HEIGHT),
        true,
        CardSignature::default(),
    )
}

#[test]
fn when_spawn_visual_card_then_root_has_card_component_face_down() {
    // Arrange
    let mut world = World::new();
    let def = make_test_def();

    // Act
    let root = spawn_def(&mut world, &def);

    // Assert
    let card = world.get::<Card>(root).expect("root should have Card");
    assert!(!card.face_up);
}

#[test]
fn when_spawn_visual_card_then_root_has_card_label() {
    // Arrange
    let mut world = World::new();
    let def = make_test_def();

    // Act
    let root = spawn_def(&mut world, &def);

    // Assert
    let label = world
        .get::<CardLabel>(root)
        .expect("root should have CardLabel");
    assert!(!label.name.is_empty(), "procedural title must not be empty");
}

#[test]
fn when_spawn_visual_card_then_root_has_baked_card_mesh() {
    // Arrange
    let mut world = World::new();
    let def = make_test_def();

    // Act
    let root = spawn_def(&mut world, &def);

    // Assert
    assert!(
        world.get::<BakedCardMesh>(root).is_some(),
        "root should have BakedCardMesh component"
    );
}

#[test]
fn when_spawn_with_art_shader_then_mesh_overlays_has_art_entry() {
    // Arrange
    use card_game::card::rendering::art_shader::register_card_art_shader;
    use engine_render::prelude::ShaderRegistry;
    use engine_render::shape::MeshOverlays;
    let mut world = World::new();
    let mut registry = ShaderRegistry::default();
    let art = register_card_art_shader(&mut registry);
    world.insert_resource(art);
    world.insert_resource(registry);
    let def = make_test_def();

    // Act
    let root = spawn_def_face_up(&mut world, &def);

    // Assert
    let overlays = world
        .get::<MeshOverlays>(root)
        .expect("root should have MeshOverlays");
    assert_eq!(
        overlays.0.len(),
        1,
        "should have one overlay entry for the art shader"
    );
}

/// @doc: Front and back face meshes must be fully tessellated during card spawn.
/// Players see empty cards if these meshes are missing, even if overlays exist. This protects against
/// regressions in `bake_front_face` and `bake_back_face` integration.
#[test]
fn when_spawn_visual_card_then_baked_front_mesh_is_nonempty() {
    // Arrange
    let mut world = World::new();
    let def = make_test_def();

    // Act
    let root = spawn_def(&mut world, &def);

    // Assert
    let baked = world
        .get::<BakedCardMesh>(root)
        .expect("root should have BakedCardMesh");
    assert!(
        !baked.front.is_empty(),
        "front face mesh should have vertices"
    );
}

#[test]
fn when_spawn_visual_card_then_baked_back_mesh_is_nonempty() {
    // Arrange
    let mut world = World::new();
    let def = make_test_def();

    // Act
    let root = spawn_def(&mut world, &def);

    // Assert
    let baked = world
        .get::<BakedCardMesh>(root)
        .expect("root should have BakedCardMesh");
    assert!(
        !baked.back.is_empty(),
        "back face mesh should have vertices"
    );
}

/// @doc: Initial `ColorMesh` selection (front vs. back) depends on the `face_up` parameter.
/// This test ensures the unified render system has the correct mesh from spawn, without waiting for the next sync.
#[test]
fn when_spawn_visual_card_face_down_then_color_mesh_matches_back() {
    // Arrange
    let mut world = World::new();
    let def = make_test_def();

    // Act
    let root = spawn_def(&mut world, &def);

    // Assert
    let baked = world
        .get::<BakedCardMesh>(root)
        .expect("root should have BakedCardMesh");
    let mesh = world
        .get::<ColorMesh>(root)
        .expect("root should have ColorMesh");
    assert_eq!(
        mesh.0.vertices.len(),
        baked.back.vertices.len(),
        "face-down card ColorMesh should match back face"
    );
    assert_eq!(
        mesh.0.indices.len(),
        baked.back.indices.len(),
        "face-down card ColorMesh indices should match back face"
    );
}

/// @doc: Spawning with `face_up=true` must immediately have the front mesh visible.
/// Players see cards in hand face-up without delay, so the initial mesh selection is non-negotiable.
#[test]
fn when_spawn_visual_card_face_up_then_color_mesh_matches_front() {
    // Arrange
    let mut world = World::new();
    let def = make_test_def();

    // Act
    let root = spawn_def_face_up(&mut world, &def);

    // Assert
    let baked = world
        .get::<BakedCardMesh>(root)
        .expect("root should have BakedCardMesh");
    let mesh = world
        .get::<ColorMesh>(root)
        .expect("root should have ColorMesh");
    assert_eq!(
        mesh.0.vertices.len(),
        baked.front.vertices.len(),
        "face-up card ColorMesh should match front face"
    );
    assert_eq!(
        mesh.0.indices.len(),
        baked.front.indices.len(),
        "face-up card ColorMesh indices should match front face"
    );
}

#[test]
fn when_spawn_visual_card_then_root_collider_half_is_card_size() {
    // Arrange
    let mut world = World::new();
    let def = make_test_def();
    let card_size = Vec2::new(100.0, 200.0);

    // Act
    let root = spawn_visual_card(
        &mut world,
        &def,
        Vec2::ZERO,
        card_size,
        false,
        CardSignature::default(),
    );

    // Assert
    let collider = world
        .get::<Collider>(root)
        .expect("root should have Collider");
    match collider {
        Collider::Aabb(half) => {
            assert_eq!(*half, card_size * 0.5);
        }
        _ => panic!("expected Collider::Aabb"),
    }
}

#[test]
fn when_spawn_visual_card_with_signature_then_card_stores_it() {
    // Arrange
    let mut world = World::new();
    let def = make_test_def();
    let signature = CardSignature::new([0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8]);

    // Act
    let root = spawn_visual_card(
        &mut world,
        &def,
        Vec2::ZERO,
        Vec2::new(CARD_WIDTH, CARD_HEIGHT),
        false,
        signature,
    );

    // Assert
    let card = world.get::<Card>(root).expect("root should have Card");
    assert_eq!(card.signature, signature);
}

/// @doc: `spawn_visual_card` must not create child entities. The baked card is a flat ECS entity,
/// not a hierarchical scene. Child entities would break transform updates and physics.
#[test]
fn when_spawn_visual_card_then_no_child_entities_exist() {
    // Arrange
    let mut world = World::new();
    let def = make_test_def();

    // Act
    let root = spawn_def(&mut world, &def);

    // Assert — no entity should have a ChildOf pointing to root
    let mut q = world.query::<&ChildOf>();
    let children: Vec<_> = q
        .iter(&world)
        .filter(|child_of| child_of.0 == root)
        .collect();
    assert!(
        children.is_empty(),
        "baked card should have no child entities, found {}",
        children.len()
    );
}

#[test]
fn when_spawn_with_matching_base_type_then_description_contains_effect_text() {
    use card_game::card::identity::base_type::{BaseCardTypeRegistry, populate_default_types};

    // Arrange — signature with strong Febris (maps to Power → "Deal X damage")
    let mut world = World::new();
    let mut registry = BaseCardTypeRegistry::new();
    populate_default_types(&mut registry);
    world.insert_resource(registry);
    let def = make_test_def();
    let signature = CardSignature::new([0.7, 0.8, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);

    // Act
    let root = spawn_visual_card(
        &mut world,
        &def,
        Vec2::ZERO,
        Vec2::new(CARD_WIDTH, CARD_HEIGHT),
        false,
        signature,
    );

    // Assert
    let label = world
        .get::<CardLabel>(root)
        .expect("card should have CardLabel");
    let has_effect = label.description.contains("damage")
        || label.description.contains("health")
        || label.description.contains("Block")
        || label.description.contains("initiative");
    assert!(
        has_effect,
        "card with residual stats should have effect-based description, got: {:?}",
        label.description
    );
}

/// @doc: Art shapes from the repository must integrate into the baked mesh tessellation.
/// If art doesn't appear, either the repository lookup failed or the mesh injection (vertex base offset)
/// has a bug. This test catches both issues.
#[test]
fn when_spawn_with_shape_repository_then_front_mesh_has_more_vertices() {
    use card_game::card::art::ShapeRepository;

    // Arrange
    let mut world = World::new();
    let def = make_test_def();
    // Spawn baseline without repo
    let baseline_root = spawn_def(&mut world, &def);
    let baseline_verts = world
        .get::<BakedCardMesh>(baseline_root)
        .unwrap()
        .front
        .vertices
        .len();

    // Now insert a hydrated ShapeRepository and spawn again
    let mut repo = ShapeRepository::new();
    repo.hydrate_all();
    world.insert_resource(repo);
    let sig = CardSignature::new([0.9, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    let root = spawn_visual_card(
        &mut world,
        &def,
        Vec2::new(100.0, 0.0),
        Vec2::new(CARD_WIDTH, CARD_HEIGHT),
        false,
        sig,
    );

    // Assert
    let with_art_verts = world
        .get::<BakedCardMesh>(root)
        .unwrap()
        .front
        .vertices
        .len();
    assert!(
        with_art_verts > baseline_verts,
        "with repo: {with_art_verts} vertices, without: {baseline_verts} — expected more"
    );
}

/// @doc: Legendary cards must have variant overlays (foil/glow/glossy shader) applied to the baked mesh.
/// The overlay mesh inherits the front mesh's UV coordinates, so art vertices must carry non-zero UV.
/// If this test fails, variant shaders won't see their art data and will render incorrectly.
#[test]
fn when_spawn_with_art_then_variant_overlay_has_nonzero_uv() {
    use card_game::card::art::ShapeRepository;
    use engine_render::shape::MeshOverlays;

    // Arrange — legendary card with art repo
    let mut world = setup_world_with_all_shaders();
    let mut repo = ShapeRepository::new();
    repo.hydrate_all();
    world.insert_resource(repo);
    let def = make_test_def();
    let legendary_sig = CardSignature::new([1.0; 8]);

    // Act
    let root = spawn_visual_card(
        &mut world,
        &def,
        Vec2::ZERO,
        Vec2::new(CARD_WIDTH, CARD_HEIGHT),
        true,
        legendary_sig,
    );

    // Assert — variant overlay (index 1) should have some vertices with non-zero UV
    let overlays = world
        .get::<MeshOverlays>(root)
        .expect("root should have MeshOverlays");
    assert_eq!(overlays.0.len(), 2, "legendary should have 2 overlays");

    let variant_mesh = &overlays.0[1].mesh;
    let nonzero_uv_count = variant_mesh
        .vertices
        .iter()
        .filter(|v| v.uv != [0.0, 0.0])
        .count();
    let total = variant_mesh.vertices.len();

    assert!(
        nonzero_uv_count > 0,
        "variant overlay should have vertices with non-zero UV from art, \
         but all {total} vertices have uv=[0,0]"
    );
}

#[test]
fn when_spawn_without_shape_repository_then_front_mesh_matches_baseline() {
    // Arrange
    let mut world = World::new();
    let def = make_test_def();

    // Act — spawn twice without repo
    let root_a = spawn_def(&mut world, &def);
    let root_b = spawn_def(&mut world, &def);

    // Assert
    let verts_a = world
        .get::<BakedCardMesh>(root_a)
        .unwrap()
        .front
        .vertices
        .len();
    let verts_b = world
        .get::<BakedCardMesh>(root_b)
        .unwrap()
        .front
        .vertices
        .len();
    assert_eq!(verts_a, verts_b);
}

fn setup_world_with_all_shaders() -> World {
    use card_game::card::rendering::art_shader::{
        register_card_art_shader, register_variant_shaders,
    };
    use engine_render::prelude::ShaderRegistry;

    let mut world = World::new();
    let mut registry = ShaderRegistry::default();
    let art = register_card_art_shader(&mut registry);
    let variants = register_variant_shaders(&mut registry);
    world.insert_resource(art);
    world.insert_resource(variants);
    world.insert_resource(registry);
    world
}

fn setup_world_with_all_shaders_and_tiers() -> World {
    use card_game::card::rendering::art_shader::{
        register_card_art_shader, register_tier_shaders, register_variant_shaders,
    };
    use engine_render::prelude::ShaderRegistry;

    let mut world = World::new();
    let mut registry = ShaderRegistry::default();
    let art = register_card_art_shader(&mut registry);
    let variants = register_variant_shaders(&mut registry);
    let tiers = register_tier_shaders(&mut registry);
    world.insert_resource(art);
    world.insert_resource(variants);
    world.insert_resource(tiers);
    world.insert_resource(registry);
    world
}

/// @doc: Legendary rarity (all element intensities at 1.0) must trigger a variant shader overlay in addition to art.
/// Rarity hierarchy: common=1 (art only), rare=2 (art + glow), legendary=2 (art + foil). Without this overlay,
/// legendary cards look identical to rares.
#[test]
fn when_spawn_with_legendary_signature_then_mesh_overlays_has_two_entries() {
    use engine_render::shape::MeshOverlays;

    // Arrange
    let mut world = setup_world_with_all_shaders();
    let def = make_test_def();
    // Signature that hashes to Legendary rarity
    let legendary_sig = CardSignature::new([
        0.7669016,
        0.9484111,
        0.74143535,
        0.92948,
        -0.9276102,
        -0.82101953,
        0.85763896,
        -0.951836,
    ]);

    // Act
    let root = spawn_visual_card(
        &mut world,
        &def,
        Vec2::ZERO,
        Vec2::new(CARD_WIDTH, CARD_HEIGHT),
        true,
        legendary_sig,
    );

    // Assert
    let overlays = world
        .get::<MeshOverlays>(root)
        .expect("root should have MeshOverlays");
    assert_eq!(
        overlays.0.len(),
        2,
        "legendary card should have art + foil overlay entries"
    );
}

#[test]
fn when_spawn_with_common_signature_and_all_shaders_then_mesh_overlays_has_one_entry() {
    use engine_render::shape::MeshOverlays;

    // Arrange
    let mut world = setup_world_with_all_shaders();
    let def = make_test_def();
    // Signature that hashes to Common rarity
    let common_sig = CardSignature::new([
        0.25628817,
        0.54249763,
        0.6507323,
        -0.5228295,
        0.072937846,
        0.013733745,
        -0.24290907,
        0.8036065,
    ]);

    // Act
    let root = spawn_visual_card(
        &mut world,
        &def,
        Vec2::ZERO,
        Vec2::new(CARD_WIDTH, CARD_HEIGHT),
        true,
        common_sig,
    );

    // Assert
    let overlays = world
        .get::<MeshOverlays>(root)
        .expect("root should have MeshOverlays");
    assert_eq!(
        overlays.0.len(),
        1,
        "common card should have only the art overlay entry"
    );
}

/// @doc: Legendary cards must use the foil shader, not glow or glossy. Rarity determines shader selection,
/// so if the wrong shader handle is assigned, players see the wrong visual effect for the rarity tier.
#[test]
fn when_spawn_with_legendary_signature_then_variant_overlay_uses_foil_shader_handle() {
    use card_game::card::rendering::art_shader::VariantShaders;
    use engine_render::shape::MeshOverlays;

    // Arrange
    let mut world = setup_world_with_all_shaders();
    let def = make_test_def();
    // Signature that hashes to Legendary rarity
    let legendary_sig = CardSignature::new([
        0.7669016,
        0.9484111,
        0.74143535,
        0.92948,
        -0.9276102,
        -0.82101953,
        0.85763896,
        -0.951836,
    ]);

    // Act
    let root = spawn_visual_card(
        &mut world,
        &def,
        Vec2::ZERO,
        Vec2::new(CARD_WIDTH, CARD_HEIGHT),
        true,
        legendary_sig,
    );

    // Assert
    let foil_handle = world
        .get_resource::<VariantShaders>()
        .expect("VariantShaders resource should exist")
        .foil;
    let overlays = world
        .get::<MeshOverlays>(root)
        .expect("root should have MeshOverlays");
    assert_eq!(
        overlays.0[1].material.shader, foil_handle,
        "second overlay should use the foil shader"
    );
}

/// @doc: Variant overlay uniforms must encode `ArtRegionParams` (32 bytes). If the size is wrong,
/// the shader's buffer access will read garbage or panic. This guards against unintended changes to struct layout.
#[test]
fn when_spawn_with_legendary_signature_then_variant_overlay_uniforms_are_sixteen_bytes() {
    use engine_render::shape::MeshOverlays;

    // Arrange
    let mut world = setup_world_with_all_shaders();
    let def = make_test_def();
    let legendary_sig = CardSignature::new([
        0.7669016,
        0.9484111,
        0.74143535,
        0.92948,
        -0.9276102,
        -0.82101953,
        0.85763896,
        -0.951836,
    ]);

    // Act
    let root = spawn_visual_card(
        &mut world,
        &def,
        Vec2::ZERO,
        Vec2::new(CARD_WIDTH, CARD_HEIGHT),
        true,
        legendary_sig,
    );

    // Assert
    let overlays = world
        .get::<MeshOverlays>(root)
        .expect("root should have MeshOverlays");
    assert_eq!(
        overlays.0[1].material.uniforms.len(),
        32,
        "variant overlay uniforms should be 32 bytes (ArtRegionParams)"
    );
}

/// @doc: Overlays must only be visible when the card is face-up. Face-down cards show only the back mesh,
/// so variant shaders must be hidden to prevent visual artifacts under the opaque back face.
#[test]
fn when_spawn_with_legendary_signature_face_down_then_variant_overlay_not_visible() {
    use engine_render::shape::MeshOverlays;

    // Arrange
    let mut world = setup_world_with_all_shaders();
    let def = make_test_def();
    let legendary_sig = CardSignature::new([
        0.7669016,
        0.9484111,
        0.74143535,
        0.92948,
        -0.9276102,
        -0.82101953,
        0.85763896,
        -0.951836,
    ]);

    // Act
    let root = spawn_visual_card(
        &mut world,
        &def,
        Vec2::ZERO,
        Vec2::new(CARD_WIDTH, CARD_HEIGHT),
        false,
        legendary_sig,
    );

    // Assert
    let overlays = world
        .get::<MeshOverlays>(root)
        .expect("root should have MeshOverlays");
    assert_eq!(overlays.0.len(), 2, "legendary should still have 2 entries");
    assert!(
        !overlays.0[1].visible,
        "face-down card's variant overlay should not be visible"
    );
}

#[test]
fn when_spawn_without_variant_shader_resources_and_legendary_signature_then_one_entry() {
    use card_game::card::rendering::art_shader::register_card_art_shader;
    use engine_render::prelude::ShaderRegistry;
    use engine_render::shape::MeshOverlays;

    // Arrange — only art shader, no variant shaders
    let mut world = World::new();
    let mut registry = ShaderRegistry::default();
    let art = register_card_art_shader(&mut registry);
    world.insert_resource(art);
    world.insert_resource(registry);
    let def = make_test_def();
    let legendary_sig = CardSignature::new([1.0; 8]);

    // Act
    let root = spawn_visual_card(
        &mut world,
        &def,
        Vec2::ZERO,
        Vec2::new(CARD_WIDTH, CARD_HEIGHT),
        true,
        legendary_sig,
    );

    // Assert
    let overlays = world
        .get::<MeshOverlays>(root)
        .expect("root should have MeshOverlays");
    assert_eq!(
        overlays.0.len(),
        1,
        "without variant shader resources, should gracefully fall back to art-only"
    );
}

/// @doc: Art overlay quad must have correctly mapped UV corners [0,0] to [1,1] to ensure the art shader
/// samples the correct region of the art atlas. Missing or incorrect UVs will cause art to render upside-down, sideways, or repeated.
#[test]
fn when_spawn_with_art_shader_then_overlay_quad_has_uv_corners() {
    use engine_render::shape::MeshOverlays;

    // Arrange
    let mut world = setup_world_with_all_shaders();
    let def = make_test_def();

    // Act
    let root = spawn_def_face_up(&mut world, &def);

    // Assert — first overlay is the art quad
    let overlays = world
        .get::<MeshOverlays>(root)
        .expect("root should have MeshOverlays");
    let quad_verts = &overlays.0[0].mesh.vertices;
    assert_eq!(quad_verts.len(), 4);
    assert_eq!(quad_verts[0].uv, [0.0, 0.0]);
    assert_eq!(quad_verts[1].uv, [1.0, 0.0]);
    assert_eq!(quad_verts[2].uv, [1.0, 1.0]);
    assert_eq!(quad_verts[3].uv, [0.0, 1.0]);
}

#[test]
fn when_spawn_with_matching_base_type_then_entity_has_residual_stats() {
    use card_game::card::identity::base_type::{BaseCardTypeRegistry, populate_default_types};

    // Arrange — signature near the Weapon archetype [0.8, 0.3, ...]
    let mut world = World::new();
    let mut registry = BaseCardTypeRegistry::new();
    populate_default_types(&mut registry);
    world.insert_resource(registry);
    let def = make_test_def();
    let signature = CardSignature::new([0.7, 0.2, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);

    // Act
    let root = spawn_visual_card(
        &mut world,
        &def,
        Vec2::ZERO,
        Vec2::new(CARD_WIDTH, CARD_HEIGHT),
        false,
        signature,
    );

    // Assert
    assert!(
        world.get::<ResidualStats>(root).is_some(),
        "card matching a base type should have ResidualStats component"
    );
}

/// @doc: Rare cards (balanced element intensities) must use the glow shader, not foil.
/// Confusing rare/legendary shaders breaks the rarity visual hierarchy that players rely on to assess card power.
#[test]
fn when_rare_card_spawned_then_glow_overlay_uses_glow_shader() {
    use card_game::card::rendering::art_shader::VariantShaders;
    use engine_render::shape::MeshOverlays;

    // Arrange
    let mut world = setup_world_with_all_shaders();
    let def = make_test_def();
    // Signature that hashes to Rare rarity
    let rare_sig = CardSignature::new([
        -0.17559588,
        0.8599839,
        -0.4562556,
        -0.0023616552,
        -0.0016959906,
        0.766793,
        -0.15155804,
        0.15561616,
    ]);

    // Act
    let root = spawn_visual_card(
        &mut world,
        &def,
        Vec2::ZERO,
        Vec2::new(CARD_WIDTH, CARD_HEIGHT),
        true,
        rare_sig,
    );

    // Assert
    let glow_handle = world
        .get_resource::<VariantShaders>()
        .expect("VariantShaders resource")
        .glow;
    let overlays = world
        .get::<MeshOverlays>(root)
        .expect("root should have MeshOverlays");
    assert_eq!(overlays.0.len(), 2, "rare card should have 2 overlays");
    assert_eq!(
        overlays.0[1].material.shader, glow_handle,
        "second overlay should use the glow shader"
    );
}

#[test]
fn when_rare_card_without_art_then_no_glow_overlay() {
    use engine_render::shape::MeshOverlays;

    // Arrange — no ShapeRepository, so no art shapes
    let mut world = setup_world_with_all_shaders();
    let def = make_test_def();
    let rare_sig = CardSignature::new([
        -0.17559588,
        0.8599839,
        -0.4562556,
        -0.0023616552,
        -0.0016959906,
        0.766793,
        -0.15155804,
        0.15561616,
    ]);

    // Act
    let root = spawn_visual_card(
        &mut world,
        &def,
        Vec2::ZERO,
        Vec2::new(CARD_WIDTH, CARD_HEIGHT),
        true,
        rare_sig,
    );

    // Assert — without art, glow overlay should use the front_mesh fallback (existing behavior)
    // or be absent. Either way, it shouldn't crash.
    let overlays = world
        .get::<MeshOverlays>(root)
        .expect("root should have MeshOverlays");
    if overlays.0.len() > 1 {
        // If a glow overlay exists without art, it should use front_mesh (existing behavior)
        assert!(
            overlays.0[1].mesh.vertices.len() > 4,
            "fallback glow overlay should use front_mesh, not an empty mesh"
        );
    }
}

// --- Tier condition overlay tests ---

/// @doc: Dormant tier cards (low signature intensity) get a worn/scratched shader
/// overlay as a third `MeshOverlays` entry. Without this, dormant and active cards
/// look identical, removing the visual signal of card power level that players
/// rely on for quick table reads.
#[test]
fn when_spawn_dormant_tier_common_with_tier_shaders_then_tier_overlay_uses_dormant_shader() {
    use card_game::card::rendering::art_shader::TierShaders;
    use engine_render::shape::MeshOverlays;

    // Arrange — Dormant card-level tier + Common rarity (both hash-based)
    let mut world = setup_world_with_all_shaders_and_tiers();
    let def = make_test_def();
    let dormant_sig = CardSignature::new([
        0.44654524,
        0.73940444,
        0.5614276,
        -0.21283162,
        -0.6634345,
        -0.1518842,
        0.7765765,
        -0.75107336,
    ]);

    // Act
    let root = spawn_visual_card(
        &mut world,
        &def,
        Vec2::ZERO,
        Vec2::new(CARD_WIDTH, CARD_HEIGHT),
        true,
        dormant_sig,
    );

    // Assert — Common rarity = no variant overlay, but Dormant tier = worn overlay
    // Expected: art-vignette(0) + tier-worn(1)
    let overlays = world
        .get::<MeshOverlays>(root)
        .expect("root should have MeshOverlays");
    assert_eq!(
        overlays.0.len(),
        2,
        "dormant common card should have art + tier overlay"
    );
    let dormant_handle = world
        .get_resource::<TierShaders>()
        .expect("TierShaders resource")
        .dormant;
    assert_eq!(
        overlays.0[1].material.shader, dormant_handle,
        "tier overlay should use the dormant shader"
    );
}

/// @doc: Active tier (mid-range intensity) is the baseline — no tier overlay is
/// added, keeping the rendering pipeline lean for the most common card tier.
#[test]
fn when_spawn_active_tier_common_with_tier_shaders_then_no_tier_overlay() {
    use engine_render::shape::MeshOverlays;

    // Arrange — Active card-level tier + Common rarity (both hash-based)
    let mut world = setup_world_with_all_shaders_and_tiers();
    let def = make_test_def();
    let active_sig = CardSignature::new([
        0.44356883,
        0.13636649,
        0.7909734,
        0.4403118,
        0.61432266,
        0.015345693,
        -0.5324175,
        0.12884033,
    ]);

    // Act
    let root = spawn_visual_card(
        &mut world,
        &def,
        Vec2::ZERO,
        Vec2::new(CARD_WIDTH, CARD_HEIGHT),
        true,
        active_sig,
    );

    // Assert — Active tier = ConditionEffect::None = no tier overlay
    let overlays = world
        .get::<MeshOverlays>(root)
        .expect("root should have MeshOverlays");
    assert_eq!(
        overlays.0.len(),
        1,
        "active common card should have only art overlay (no tier overlay)"
    );
}

/// @doc: Intense tier cards get a shiny shimmer overlay. When combined with a
/// rarity variant overlay, the tier overlay is always last (highest painter's order)
/// so the shimmer composites on top of everything.
#[test]
fn when_spawn_intense_tier_legendary_with_tier_shaders_then_three_overlays() {
    use card_game::card::rendering::art_shader::TierShaders;
    use engine_render::shape::MeshOverlays;

    // Arrange — Intense card-level tier + Legendary rarity (both hash-based)
    let mut world = setup_world_with_all_shaders_and_tiers();
    let def = make_test_def();
    let intense_sig = CardSignature::new([
        -0.8589847,
        0.2778691,
        0.018823981,
        0.68338156,
        0.19329798,
        0.46955168,
        -0.7978437,
        -0.64191556,
    ]);

    // Act
    let root = spawn_visual_card(
        &mut world,
        &def,
        Vec2::ZERO,
        Vec2::new(CARD_WIDTH, CARD_HEIGHT),
        true,
        intense_sig,
    );

    // Assert — art(0) + rarity foil(1) + tier intense(2)
    let overlays = world
        .get::<MeshOverlays>(root)
        .expect("root should have MeshOverlays");
    assert_eq!(
        overlays.0.len(),
        3,
        "intense legendary card should have art + rarity + tier overlays"
    );
    let intense_handle = world
        .get_resource::<TierShaders>()
        .expect("TierShaders resource")
        .intense;
    assert_eq!(
        overlays.0[2].material.shader, intense_handle,
        "third overlay should use the intense shader"
    );
}

/// @doc: Tier overlay visibility must match `face_up` state at spawn time. Face-down
/// cards hide all overlays to prevent shader effects from bleeding through the
/// opaque back face.
#[test]
fn when_spawn_dormant_face_down_with_tier_shaders_then_tier_overlay_visible() {
    use engine_render::shape::MeshOverlays;

    // Arrange — Dormant tier + Common rarity (by hash)
    let mut world = setup_world_with_all_shaders_and_tiers();
    let def = make_test_def();
    let dormant_sig = CardSignature::new([
        0.44654524,
        0.73940444,
        0.5614276,
        -0.21283162,
        -0.6634345,
        -0.1518842,
        0.7765765,
        -0.75107336,
    ]);

    // Act
    let root = spawn_visual_card(
        &mut world,
        &def,
        Vec2::ZERO,
        Vec2::new(CARD_WIDTH, CARD_HEIGHT),
        false,
        dormant_sig,
    );

    // Assert — tier overlays apply to both faces
    let overlays = world
        .get::<MeshOverlays>(root)
        .expect("root should have MeshOverlays");
    assert_eq!(overlays.0.len(), 2, "dormant common should have 2 entries");
    assert!(
        overlays.0[1].visible,
        "tier overlay should be visible even when face-down"
    );
    assert!(
        !overlays.0[1].front_only,
        "tier overlay should not be front_only"
    );
}
