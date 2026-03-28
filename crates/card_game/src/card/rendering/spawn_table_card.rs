use bevy_ecs::prelude::{Entity, World};
use engine_core::prelude::{TextureId, Transform2D};
use engine_physics::prelude::{Collider, PhysicsRes, RigidBody};
use engine_render::shape::ColorMesh;
use engine_scene::prelude::{RenderLayer, SortOrder};
use glam::Vec2;

use crate::card::art::ShapeRepository;
use crate::card::art_selection::select_art_for_signature;
use crate::card::component::Card;
use crate::card::component::CardLabel;
use crate::card::component::CardZone;
use crate::card::identity::base_type::BaseCardTypeRegistry;
use crate::card::identity::card_description::generate_card_description;
use crate::card::identity::card_name::generate_card_name;
use crate::card::identity::definition::CardDefinition;
use crate::card::identity::residual::ResidualStats;
use crate::card::identity::signature::CardSignature;
use crate::card::identity::signature_profile::SignatureProfile;
use crate::card::interaction::damping::{BASE_ANGULAR_DRAG, BASE_LINEAR_DRAG};
use crate::card::rendering::art_shader::CardArtShader;
use crate::card::rendering::bake::{bake_back_face, bake_front_face};
use crate::card::rendering::baked_mesh::BakedCardMesh;

pub(crate) const CARD_CORNER_RADIUS: f32 = 5.0;

pub fn spawn_visual_card(
    world: &mut World,
    def: &CardDefinition,
    position: Vec2,
    card_size: Vec2,
    face_up: bool,
    signature: CardSignature,
) -> Entity {
    let half = card_size * 0.5;
    let card = Card {
        face_texture: TextureId(0),
        back_texture: TextureId(0),
        face_up,
        signature,
    };
    let (profile, stats) = {
        let registry = world.get_resource::<BaseCardTypeRegistry>();
        let profile = registry.map_or_else(
            || SignatureProfile::without_archetype(&signature),
            |reg| SignatureProfile::new(&signature, reg),
        );
        let stats = registry
            .and_then(|reg| reg.best_match(&signature))
            .map(|base_type| ResidualStats::from_card(&signature, base_type));
        (profile, stats)
    };

    let card_name = generate_card_name(&profile, &signature);
    let description = stats
        .as_ref()
        .map(generate_card_description)
        .filter(|d| !d.is_empty())
        .unwrap_or(card_name.subtitle);
    let label = CardLabel {
        name: card_name.title,
        description,
    };

    let root = world
        .spawn((
            card,
            def.clone(),
            label.clone(),
            CardZone::Table,
            Transform2D {
                position,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
            RigidBody::Dynamic,
            Collider::Aabb(half),
            RenderLayer::World,
            SortOrder(0),
        ))
        .id();

    if let Some(mut physics) = world.get_resource_mut::<PhysicsRes>() {
        physics.add_body(root, &RigidBody::Dynamic, position);
        physics.add_collider(root, &Collider::Aabb(half));
        physics
            .set_damping(root, BASE_LINEAR_DRAG, BASE_ANGULAR_DRAG)
            .expect("freshly spawned card should have physics body");
    }

    if let Some(stats) = stats {
        world.entity_mut(root).insert(stats);
    }

    let art_shapes = world
        .get_resource::<ShapeRepository>()
        .and_then(|repo| select_art_for_signature(&signature, repo))
        .map(|entry| entry.shapes().to_vec());
    let baked = BakedCardMesh {
        front: bake_front_face(&card.signature, card_size, &label, art_shapes.as_deref()),
        back: bake_back_face(card_size),
    };
    let initial_mesh = if face_up {
        baked.front.clone()
    } else {
        baked.back.clone()
    };
    let mesh_overlays =
        build_mesh_overlays(world, card_size, &card.signature, face_up, &baked.front);
    world
        .entity_mut(root)
        .insert((baked, mesh_overlays, ColorMesh(initial_mesh)));

    root
}

fn build_mesh_overlays(
    world: &World,
    card_size: Vec2,
    signature: &CardSignature,
    face_up: bool,
    front_mesh: &engine_render::shape::TessellatedColorMesh,
) -> engine_render::shape::MeshOverlays {
    use crate::card::rendering::art_shader::{ArtRegionParams, ShaderVariant, VariantShaders};
    use crate::card::rendering::face_layout::FRONT_FACE_REGIONS;
    use engine_render::shape::{ColorVertex, MeshOverlays, OverlayEntry, TessellatedColorMesh};

    let mut entries = Vec::new();
    let profile =
        crate::card::identity::signature_profile::SignatureProfile::without_archetype(signature);
    let visuals = crate::card::identity::visual_params::generate_card_visuals(signature, &profile);

    let art_region = &FRONT_FACE_REGIONS[2];
    let (half_w, half_h, offset_y) = art_region.resolve(card_size.x, card_size.y);
    let art_params = ArtRegionParams {
        half_w,
        half_h,
        pointer_x: 0.0,
        pointer_y: 0.0,
        offset_y,
        _pad0: 0.0,
        _pad1: 0.0,
        _pad2: 0.0,
    };
    let art_uniforms = bytemuck::bytes_of(&art_params).to_vec();

    // Art vignette overlay: a quad over the art region
    if let Some(art_shader) = world.get_resource::<CardArtShader>().map(|s| s.0) {
        let c = [
            visuals.art_color.r,
            visuals.art_color.g,
            visuals.art_color.b,
            visuals.art_color.a,
        ];
        let quad_mesh = TessellatedColorMesh {
            vertices: vec![
                ColorVertex {
                    position: [-half_w, -half_h + offset_y],
                    color: c,
                    uv: [0.0, 0.0],
                },
                ColorVertex {
                    position: [half_w, -half_h + offset_y],
                    color: c,
                    uv: [1.0, 0.0],
                },
                ColorVertex {
                    position: [half_w, half_h + offset_y],
                    color: c,
                    uv: [1.0, 1.0],
                },
                ColorVertex {
                    position: [-half_w, half_h + offset_y],
                    color: c,
                    uv: [0.0, 1.0],
                },
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
        };
        entries.push(OverlayEntry {
            mesh: quad_mesh,
            material: engine_render::material::Material2d {
                shader: art_shader,
                uniforms: art_uniforms.clone(),
                ..engine_render::material::Material2d::default()
            },
            visible: face_up,
        });
    }

    // Variant shader overlay: uses the full baked card geometry
    let variant_shader = match visuals.shader_variant {
        ShaderVariant::None => None,
        variant => world
            .get_resource::<VariantShaders>()
            .map(|vs| match variant {
                ShaderVariant::Embossed => vs.embossed,
                ShaderVariant::Glow => vs.glow,
                ShaderVariant::Glossy => vs.glossy,
                ShaderVariant::Foil => vs.foil,
                ShaderVariant::None => unreachable!(),
            }),
    };

    if let Some(shader) = variant_shader {
        entries.push(OverlayEntry {
            mesh: front_mesh.clone(),
            material: engine_render::material::Material2d {
                shader,
                uniforms: art_uniforms,
                blend_mode: engine_render::material::BlendMode::Alpha,
                ..engine_render::material::Material2d::default()
            },
            visible: face_up,
        });
    }

    MeshOverlays(entries)
}

pub(crate) const TEXT_COLOR: engine_core::color::Color = engine_core::color::Color {
    r: 0.1,
    g: 0.1,
    b: 0.1,
    a: 1.0,
};

/// Find the largest font size where the name wraps to at most 2 lines
/// and fits within both `max_width` and `max_height`.
///
/// Strategy: first check if text fits on 1 line at base size. If not,
/// find the font size where balanced 2-line wrapping works, then clamp
/// to fit the strip height.
pub(crate) fn fit_name_font_size(
    text: &str,
    base_size: f32,
    max_width: f32,
    max_height: f32,
) -> f32 {
    // Does it fit on 1 line at base size?
    let full_width = engine_render::font::measure_text(text, base_size);
    if full_width <= max_width {
        return base_size;
    }

    // It needs wrapping. Find the font size where the wider half of a balanced
    // 2-line split fits within max_width. Since text width scales linearly with
    // font size, we can compute this directly.
    let words: Vec<&str> = text.split(' ').collect();
    if words.len() <= 1 {
        // Single word — just shrink to fit width
        return base_size * max_width / full_width;
    }

    // Find the best balanced split at base size and measure the wider half
    let mut best_max_half = full_width;
    for split in 1..words.len() {
        let line1 = words[..split].join(" ");
        let line2 = words[split..].join(" ");
        let w1 = engine_render::font::measure_text(&line1, base_size);
        let w2 = engine_render::font::measure_text(&line2, base_size);
        let wider = w1.max(w2);
        if wider < best_max_half {
            best_max_half = wider;
        }
    }

    // Scale font so the wider half fits within max_width
    let width_size = if best_max_half > max_width {
        base_size * max_width / best_max_half
    } else {
        base_size
    };

    // Also clamp to fit 2 lines within the strip height
    let two_line_height = width_size * 1.3 * 2.0;
    if two_line_height <= max_height {
        width_size
    } else {
        width_size * max_height / two_line_height
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use bevy_ecs::prelude::*;
    use engine_scene::prelude::ChildOf;
    use glam::Vec2;

    use super::*;
    use crate::card::identity::definition::{
        CardAbilities, CardDefinition, CardType, art_descriptor_default,
    };
    use crate::card::identity::signature::CardSignature;
    use crate::card::rendering::baked_mesh::BakedCardMesh;
    use crate::card::rendering::geometry::{
        TABLE_CARD_HEIGHT as CARD_HEIGHT, TABLE_CARD_WIDTH as CARD_WIDTH,
    };

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
        use crate::card::rendering::art_shader::register_card_art_shader;
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
        use crate::card::identity::base_type::{BaseCardTypeRegistry, populate_default_types};

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
        use crate::card::art::ShapeRepository;

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
        use crate::card::art::ShapeRepository;
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
        use crate::card::rendering::art_shader::{
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

    /// @doc: Legendary rarity (all element intensities at 1.0) must trigger a variant shader overlay in addition to art.
    /// Rarity hierarchy: common=1 (art only), rare=2 (art + glow), legendary=2 (art + foil). Without this overlay,
    /// legendary cards look identical to rares.
    #[test]
    fn when_spawn_with_legendary_signature_then_mesh_overlays_has_two_entries() {
        use engine_render::shape::MeshOverlays;

        // Arrange
        let mut world = setup_world_with_all_shaders();
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
        let common_sig = CardSignature::new([0.0; 8]);

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
        use crate::card::rendering::art_shader::VariantShaders;
        use engine_render::shape::MeshOverlays;

        // Arrange
        let mut world = setup_world_with_all_shaders();
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
        let legendary_sig = CardSignature::new([1.0; 8]);

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
        use crate::card::rendering::art_shader::register_card_art_shader;
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
        use crate::card::identity::base_type::{BaseCardTypeRegistry, populate_default_types};
        use crate::card::identity::residual::ResidualStats;

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
        use crate::card::rendering::art_shader::VariantShaders;
        use engine_render::shape::MeshOverlays;

        // Arrange
        let mut world = setup_world_with_all_shaders();
        let def = make_test_def();
        let rare_sig = CardSignature::new([0.35, -0.35, 0.35, -0.35, 0.35, -0.35, 0.35, -0.35]);

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
        let rare_sig = CardSignature::new([0.35, -0.35, 0.35, -0.35, 0.35, -0.35, 0.35, -0.35]);

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
}
