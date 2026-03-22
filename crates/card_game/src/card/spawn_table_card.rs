use bevy_ecs::prelude::{Entity, World};
use engine_core::prelude::{TextureId, Transform2D};
use engine_physics::prelude::{Collider, PhysicsRes, RigidBody};
use engine_render::shape::ColorMesh;
use engine_scene::prelude::{RenderLayer, SortOrder};
use glam::Vec2;

use crate::card::art_shader::CardArtShader;
use crate::card::bake::{bake_back_face, bake_front_face};
use crate::card::baked_mesh::BakedCardMesh;
use crate::card::base_type::BaseCardTypeRegistry;
use crate::card::card_name::generate_card_name;
use crate::card::component::Card;
use crate::card::damping::{BASE_ANGULAR_DRAG, BASE_LINEAR_DRAG};
use crate::card::definition::CardDefinition;
use crate::card::label::CardLabel;
use crate::card::residual::ResidualStats;
use crate::card::signature::CardSignature;
use crate::card::signature_profile::SignatureProfile;
use crate::card::zone::CardZone;

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
    let label = {
        let profile = world.get_resource::<BaseCardTypeRegistry>().map_or_else(
            || SignatureProfile::without_archetype(&signature),
            |reg| SignatureProfile::new(&signature, reg),
        );
        let card_name = generate_card_name(&profile, &signature);
        CardLabel {
            name: card_name.title,
            description: card_name.subtitle,
        }
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

    if let Some(registry) = world.get_resource::<BaseCardTypeRegistry>()
        && let Some(base_type) = registry.best_match(&card.signature)
    {
        let stats = ResidualStats::from_card(&card.signature, base_type);
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
    use crate::card::face_layout::FRONT_FACE_REGIONS;
    use engine_render::shape::{MeshOverlays, OverlayEntry};

    let mut entries = Vec::new();

    if let Some(art_shader) = world.get_resource::<CardArtShader>().map(|s| s.0) {
        let art_region = &FRONT_FACE_REGIONS[2];
        let (half_w, half_h, offset_y) = art_region.resolve(card_size.x, card_size.y);
        let visuals = crate::card::visual_params::generate_card_visuals(signature);
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

pub(crate) fn fit_font_size(text: &str, base_size: f32, max_width: f32) -> f32 {
    let width = engine_render::font::measure_text(text, base_size);
    if width <= max_width {
        base_size
    } else {
        base_size * max_width / width
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use bevy_ecs::prelude::*;
    use engine_scene::prelude::ChildOf;
    use glam::Vec2;

    use super::*;
    use crate::card::baked_mesh::BakedCardMesh;
    use crate::card::definition::{
        CardAbilities, CardDefinition, CardType, art_descriptor_default,
    };
    use crate::card::geometry::{TABLE_CARD_HEIGHT as CARD_HEIGHT, TABLE_CARD_WIDTH as CARD_WIDTH};
    use crate::card::signature::CardSignature;

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
        use crate::card::art_shader::register_card_art_shader;
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
    fn when_spawn_with_matching_base_type_then_entity_has_residual_stats() {
        use crate::card::base_type::{BaseCardTypeRegistry, populate_default_types};
        use crate::card::residual::ResidualStats;

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
