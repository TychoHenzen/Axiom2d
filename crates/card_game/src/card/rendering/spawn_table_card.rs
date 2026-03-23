use bevy_ecs::prelude::{Entity, World};
use engine_core::prelude::{TextureId, Transform2D};
use engine_physics::prelude::{Collider, PhysicsRes, RigidBody};
use engine_render::shape::ColorMesh;
use engine_scene::prelude::{RenderLayer, SortOrder};
use glam::Vec2;

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

    let baked = BakedCardMesh {
        front: bake_front_face(&card.signature, card_size, &label),
        back: bake_back_face(card_size),
    };
    let initial_mesh = if face_up {
        baked.front.clone()
    } else {
        baked.back.clone()
    };
    let mesh_overlays = build_mesh_overlays(world, card_size, &card.signature, face_up);
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
) -> engine_render::shape::MeshOverlays {
    use crate::card::rendering::face_layout::FRONT_FACE_REGIONS;
    use engine_render::shape::{MeshOverlays, OverlayEntry};

    let mut entries = Vec::new();

    if let Some(art_shader) = world.get_resource::<CardArtShader>().map(|s| s.0) {
        let art_region = &FRONT_FACE_REGIONS[2];
        let (half_w, half_h, offset_y) = art_region.resolve(card_size.x, card_size.y);
        let visuals = crate::card::identity::visual_params::generate_card_visuals(signature);
        entries.push(OverlayEntry {
            vertices: [
                [-half_w, -half_h + offset_y],
                [half_w, -half_h + offset_y],
                [half_w, half_h + offset_y],
                [-half_w, half_h + offset_y],
            ],
            indices: [0, 1, 2, 0, 2, 3],
            color: visuals.art_color,
            material: engine_render::material::Material2d {
                shader: art_shader,
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

    // --- Behavior tests ---

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
}
