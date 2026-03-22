use bevy_ecs::prelude::{Entity, World};
use engine_core::prelude::{TextureId, Transform2D};
use engine_physics::prelude::{Collider, PhysicsRes, RigidBody};
use engine_render::shape::ColorMesh;
use engine_scene::prelude::{RenderLayer, SortOrder};
use glam::Vec2;

use crate::card::bake::{bake_back_face, bake_front_face};
use crate::card::baked_mesh::{BakedCardMesh, CardOverlays};
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
    world
        .entity_mut(root)
        .insert((baked, CardOverlays::default(), ColorMesh(initial_mesh)));

    root
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
    use engine_scene::prelude::{ChildOf, Visible};
    use glam::Vec2;

    use super::*;
    use crate::card::definition::{
        CardAbilities, CardDefinition, CardType, Rarity, art_descriptor_default,
        rarity_border_color,
    };
    use crate::card::face_side::CardFaceSide;
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

    fn children_visible_for_side(world: &mut World, root: Entity, side: CardFaceSide) -> Vec<bool> {
        let mut q = world.query::<(&ChildOf, &CardFaceSide, &Visible)>();
        q.iter(world)
            .filter(|(parent, s, _)| parent.0 == root && **s == side)
            .map(|(_, _, v)| v.0)
            .collect()
    }

    fn border_shape_for_side(
        world: &mut World,
        root: Entity,
        side: CardFaceSide,
    ) -> Option<engine_render::prelude::Shape> {
        let mut q = world.query::<(
            &ChildOf,
            &CardFaceSide,
            &engine_render::prelude::Shape,
            &SortOrder,
        )>();
        q.iter(world)
            .filter(|(parent, s, _, _)| parent.0 == root && **s == side)
            .min_by_key(|(_, _, _, sort)| sort.0)
            .map(|(_, _, shape, _)| shape.clone())
    }

    fn find_stash_icon_child(world: &mut World, root: Entity) -> Option<Entity> {
        let mut q = world.query::<(Entity, &ChildOf, &crate::stash::icon::StashIcon)>();
        q.iter(world)
            .find(|(_, parent, _)| parent.0 == root)
            .map(|(e, _, _)| e)
    }

    fn text_children_for_side(
        world: &mut World,
        root: Entity,
        side: CardFaceSide,
    ) -> Vec<(Text, LocalSortOrder, Visible)> {
        let mut q = world.query::<(&ChildOf, &CardFaceSide, &Text, &LocalSortOrder, &Visible)>();
        q.iter(world)
            .filter(|(parent, s, _, _, _)| parent.0 == root && **s == side)
            .map(|(_, _, t, sort, vis)| (t.clone(), *sort, *vis))
            .collect()
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
    fn when_spawn_visual_card_then_root_has_card_label_with_procedural_name() {
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
    fn when_spawn_visual_card_then_card_label_has_procedural_subtitle() {
        // Arrange
        let mut world = World::new();
        let def = make_test_def();

        // Act
        let root = spawn_def(&mut world, &def);

        // Assert
        let label = world
            .get::<CardLabel>(root)
            .expect("root should have CardLabel");
        assert!(
            !label.description.is_empty(),
            "procedural subtitle must not be empty"
        );
    }

    #[test]
    fn when_spawn_with_default_signature_then_border_color_matches_common_rarity() {
        // Arrange — default signature (all zeros) → determine_rarity → Common
        let mut world = World::new();
        let def = make_test_def();

        // Act
        let root = spawn_def(&mut world, &def);

        // Assert
        let border =
            border_shape_for_side(&mut world, root, CardFaceSide::Front).expect("front border");
        assert_eq!(border.color, rarity_border_color(Rarity::Common));
    }

    #[test]
    fn when_spawn_with_legendary_signature_then_border_color_is_golden_not_white() {
        // Arrange — all-ones signature → Legendary rarity
        let mut world = World::new();
        let def = make_test_def();
        let signature = CardSignature::new([1.0; 8]);

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
        let border =
            border_shape_for_side(&mut world, root, CardFaceSide::Front).expect("front border");
        assert_eq!(border.color, rarity_border_color(Rarity::Legendary));
        assert_ne!(border.color, Color::WHITE);
    }

    #[test]
    fn when_spawn_visual_card_face_down_then_front_children_not_visible() {
        // Arrange
        let mut world = World::new();
        let def = make_test_def();

        // Act
        let root = spawn_def(&mut world, &def);

        // Assert
        let results = children_visible_for_side(&mut world, root, CardFaceSide::Front);
        assert!(!results.is_empty(), "expected at least one Front child");
        assert!(results.iter().all(|v| !v));
    }

    #[test]
    fn when_spawn_visual_card_face_down_then_back_children_visible() {
        // Arrange
        let mut world = World::new();
        let def = make_test_def();

        // Act
        let root = spawn_def(&mut world, &def);

        // Assert
        let results = children_visible_for_side(&mut world, root, CardFaceSide::Back);
        assert!(!results.is_empty(), "expected at least one Back child");
        assert!(results.iter().all(|v| *v));
    }

    #[test]
    fn when_spawn_visual_card_face_up_then_front_visible_back_hidden() {
        // Arrange
        let mut world = World::new();
        let def = make_test_def();

        // Act
        let root = spawn_def_face_up(&mut world, &def);

        // Assert
        let front = children_visible_for_side(&mut world, root, CardFaceSide::Front);
        let back = children_visible_for_side(&mut world, root, CardFaceSide::Back);
        assert!(!front.is_empty(), "expected at least one Front child");
        assert!(front.iter().all(|v| *v));
        assert!(!back.is_empty(), "expected at least one Back child");
        assert!(back.iter().all(|v| !v));
    }

    #[test]
    fn when_spawn_visual_card_then_front_and_back_borders_have_different_colors() {
        // Arrange
        let mut world = World::new();
        let def = make_test_def();

        // Act
        let root = spawn_def(&mut world, &def);

        // Assert
        let front =
            border_shape_for_side(&mut world, root, CardFaceSide::Front).expect("front border");
        let back =
            border_shape_for_side(&mut world, root, CardFaceSide::Back).expect("back border");
        assert_ne!(front.color, back.color);
    }

    #[test]
    fn when_card_art_shader_registered_then_art_area_has_material2d() {
        // Arrange
        let mut world = World::new();
        let mut registry = engine_render::prelude::ShaderRegistry::default();
        let art_shader = crate::card::art_shader::register_card_art_shader(&mut registry);
        world.insert_resource(registry);
        world.insert_resource(art_shader);
        let def = make_test_def();

        // Act
        let root = spawn_def_face_up(&mut world, &def);

        // Assert — art area is front child at sort 3
        let mut q = world.query::<(
            &ChildOf,
            &CardFaceSide,
            &LocalSortOrder,
            Option<&Material2d>,
        )>();
        let art_child = q.iter(&world).find(|(parent, side, sort, _)| {
            parent.0 == root && **side == CardFaceSide::Front && sort.0 == 3
        });
        let (_, _, _, mat) = art_child.expect("art area child should exist");
        let mat = mat.expect("art area should have Material2d");
        assert_eq!(mat.shader, art_shader.0);
    }

    #[test]
    fn when_spawn_visual_card_then_root_collider_half_is_card_size_times_half() {
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
        let collider = world.get::<Collider>(root).unwrap();
        let Collider::Aabb(half) = collider else {
            panic!("expected Collider::Aabb");
        };
        assert!((half.x - 50.0).abs() < 1e-4, "half.x={}", half.x);
        assert!((half.y - 100.0).abs() < 1e-4, "half.y={}", half.y);
    }

    #[test]
    fn when_spawn_visual_card_then_stash_icon_child_is_hidden() {
        // Arrange
        let mut world = World::new();
        let def = make_test_def();

        // Act
        let root = spawn_def(&mut world, &def);

        // Assert
        let icon = find_stash_icon_child(&mut world, root).expect("StashIcon child must exist");
        let visible = world
            .get::<Visible>(icon)
            .expect("StashIcon must have Visible");
        assert!(!visible.0, "StashIcon must be Visible(false) by default");
    }

    #[test]
    fn when_no_card_art_shader_then_art_area_has_no_material2d() {
        // Arrange
        let mut world = World::new();
        let def = make_test_def();

        // Act
        let root = spawn_def(&mut world, &def);

        // Assert
        let mut q = world.query::<(
            &ChildOf,
            &CardFaceSide,
            &LocalSortOrder,
            Option<&Material2d>,
        )>();
        let art_child = q.iter(&world).find(|(parent, side, sort, _)| {
            parent.0 == root && **side == CardFaceSide::Front && sort.0 == 3
        });
        let (_, _, _, mat) = art_child.expect("art area child should exist");
        assert!(
            mat.is_none(),
            "art area should not have Material2d without shader resource"
        );
    }

    #[test]
    fn when_spawn_visual_card_then_two_text_children_with_front_side() {
        // Arrange
        let mut world = World::new();
        let def = make_test_def();

        // Act
        let root = spawn_def_face_up(&mut world, &def);

        // Assert
        let texts = text_children_for_side(&mut world, root, CardFaceSide::Front);
        assert_eq!(texts.len(), 2);
    }

    #[test]
    fn when_spawn_visual_card_then_name_text_matches_label() {
        // Arrange
        let mut world = World::new();
        let def = make_test_def();

        // Act
        let root = spawn_def_face_up(&mut world, &def);

        // Assert
        let label = world.get::<CardLabel>(root).expect("label").clone();
        let texts = text_children_for_side(&mut world, root, CardFaceSide::Front);
        let name_text = texts.iter().find(|(t, _, _)| t.content == label.name);
        assert!(
            name_text.is_some(),
            "expected text child matching label name '{}'",
            label.name
        );
    }

    #[test]
    fn when_spawn_visual_card_then_description_text_matches_label() {
        // Arrange
        let mut world = World::new();
        let def = make_test_def();

        // Act
        let root = spawn_def_face_up(&mut world, &def);

        // Assert
        let label = world.get::<CardLabel>(root).expect("label").clone();
        let texts = text_children_for_side(&mut world, root, CardFaceSide::Front);
        let desc_text = texts
            .iter()
            .find(|(t, _, _)| t.content == label.description);
        assert!(
            desc_text.is_some(),
            "expected text child matching label description '{}'",
            label.description
        );
    }

    #[test]
    fn when_spawn_visual_card_then_name_text_sort_higher_than_name_strip() {
        // Arrange
        let mut world = World::new();
        let def = make_test_def();

        // Act
        let root = spawn_def_face_up(&mut world, &def);

        // Assert — name strip is sort 2, name text must be > 2
        let label = world.get::<CardLabel>(root).expect("label").clone();
        let texts = text_children_for_side(&mut world, root, CardFaceSide::Front);
        let name_text = texts
            .iter()
            .find(|(t, _, _)| t.content == label.name)
            .expect("name text");
        assert!(
            name_text.1.0 > 2,
            "name text sort {} must be > 2",
            name_text.1.0
        );
    }

    #[test]
    fn when_spawn_visual_card_then_desc_text_sort_higher_than_desc_strip() {
        // Arrange
        let mut world = World::new();
        let def = make_test_def();

        // Act
        let root = spawn_def_face_up(&mut world, &def);

        // Assert — desc strip is sort 4, desc text must be > 4
        let label = world.get::<CardLabel>(root).expect("label").clone();
        let texts = text_children_for_side(&mut world, root, CardFaceSide::Front);
        let desc_text = texts
            .iter()
            .find(|(t, _, _)| t.content == label.description)
            .expect("desc text");
        assert!(
            desc_text.1.0 > 4,
            "desc text sort {} must be > 4",
            desc_text.1.0
        );
    }

    #[test]
    fn when_spawn_visual_card_face_down_then_text_children_hidden() {
        // Arrange
        let mut world = World::new();
        let def = make_test_def();

        // Act
        let root = spawn_def(&mut world, &def);

        // Assert
        let texts = text_children_for_side(&mut world, root, CardFaceSide::Front);
        assert_eq!(texts.len(), 2);
        assert!(texts.iter().all(|(_, _, vis)| !vis.0));
    }

    #[test]
    fn when_spawn_visual_card_with_signature_then_card_stores_it() {
        // Arrange
        let mut world = World::new();
        let def = make_test_def();
        let signature = CardSignature::new([0.5, -0.5, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0]);

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

    #[test]
    fn when_spawn_with_no_matching_base_type_then_no_residual_stats() {
        use crate::card::base_type::BaseCardTypeRegistry;
        use crate::card::residual::ResidualStats;

        // Arrange — empty registry guarantees no match
        let mut world = World::new();
        world.insert_resource(BaseCardTypeRegistry::new());
        let def = make_test_def();
        let signature = CardSignature::new([0.5, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);

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
            world.get::<ResidualStats>(root).is_none(),
            "card not matching any base type should not have ResidualStats"
        );
    }

    #[test]
    fn when_spawn_with_matching_base_type_then_residual_stats_values_match_computation() {
        use crate::card::base_type::{BaseCardTypeRegistry, populate_default_types};
        use crate::card::residual::ResidualStats;

        // Arrange — signature near Weapon archetype
        let mut world = World::new();
        let mut registry = BaseCardTypeRegistry::new();
        populate_default_types(&mut registry);
        let signature = CardSignature::new([0.7, 0.2, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let expected = {
            let base = registry
                .best_match(&signature)
                .expect("should match Weapon");
            ResidualStats::from_card(&signature, base)
        };
        world.insert_resource(registry);
        let def = make_test_def();

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
        let stats = world
            .get::<ResidualStats>(root)
            .expect("should have ResidualStats");
        assert_eq!(*stats, expected);
    }

    #[test]
    fn when_signature_provided_then_border_color_matches_signature_rarity() {
        // Arrange — all-ones signature → Legendary rarity
        let mut world = World::new();
        let def = make_test_def();
        let signature = CardSignature::new([1.0; 8]);

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
        let border =
            border_shape_for_side(&mut world, root, CardFaceSide::Front).expect("front border");
        assert_eq!(
            border.color,
            rarity_border_color(signature.rarity()),
            "border color should match signature-derived rarity"
        );
    }

    fn art_area_shape(world: &mut World, root: Entity) -> Option<engine_render::prelude::Shape> {
        let mut q = world.query::<(
            &ChildOf,
            &CardFaceSide,
            &engine_render::prelude::Shape,
            &LocalSortOrder,
        )>();
        q.iter(world)
            .find(|(parent, side, _, sort)| {
                parent.0 == root && **side == CardFaceSide::Front && sort.0 == 3
            })
            .map(|(_, _, shape, _)| shape.clone())
    }

    #[test]
    fn when_spawn_visual_card_then_art_area_color_matches_generated_visuals() {
        // Arrange
        let mut world = World::new();
        let def = make_test_def();
        let signature = CardSignature::new([0.3, -0.7, 0.1, 0.9, -0.5, 0.2, -0.8, 0.6]);
        let expected = crate::card::visual_params::generate_card_visuals(&signature);

        // Act
        let root = spawn_visual_card(
            &mut world,
            &def,
            Vec2::ZERO,
            Vec2::new(CARD_WIDTH, CARD_HEIGHT),
            true,
            signature,
        );

        // Assert
        let art = art_area_shape(&mut world, root).expect("front art area child");
        assert_eq!(art.color, expected.art_color);
    }

    #[test]
    fn when_spawn_two_cards_with_different_signatures_then_art_area_colors_differ() {
        // Arrange
        let def = make_test_def();
        let sig_a = CardSignature::new([0.0; 8]);
        let sig_b = CardSignature::new([1.0; 8]);

        let mut world_a = World::new();
        let mut world_b = World::new();

        // Act
        let root_a = spawn_visual_card(
            &mut world_a,
            &def,
            Vec2::ZERO,
            Vec2::new(CARD_WIDTH, CARD_HEIGHT),
            true,
            sig_a,
        );
        let root_b = spawn_visual_card(
            &mut world_b,
            &def,
            Vec2::ZERO,
            Vec2::new(CARD_WIDTH, CARD_HEIGHT),
            true,
            sig_b,
        );

        // Assert
        let art_a = art_area_shape(&mut world_a, root_a).expect("art area A");
        let art_b = art_area_shape(&mut world_b, root_b).expect("art area B");
        assert_ne!(art_a.color, art_b.color);
    }

    #[test]
    fn when_spawn_two_cards_with_identical_signatures_then_art_area_colors_are_identical() {
        // Arrange
        let def = make_test_def();
        let sig = CardSignature::new([0.4, -0.6, 0.2, 0.8, -0.3, 0.1, -0.7, 0.5]);

        let mut world_a = World::new();
        let mut world_b = World::new();

        // Act
        let root_a = spawn_visual_card(
            &mut world_a,
            &def,
            Vec2::ZERO,
            Vec2::new(CARD_WIDTH, CARD_HEIGHT),
            true,
            sig,
        );
        let root_b = spawn_visual_card(
            &mut world_b,
            &def,
            Vec2::ZERO,
            Vec2::new(CARD_WIDTH, CARD_HEIGHT),
            true,
            sig,
        );

        // Assert
        let art_a = art_area_shape(&mut world_a, root_a).expect("art area A");
        let art_b = art_area_shape(&mut world_b, root_b).expect("art area B");
        assert_eq!(art_a.color, art_b.color);
    }

    // --- Gem socket integration tests ---

    fn gem_circle_children(
        world: &mut World,
        root: Entity,
    ) -> Vec<(
        engine_render::prelude::Shape,
        LocalSortOrder,
        Visible,
        CardFaceSide,
        RenderLayer,
    )> {
        let mut q = world.query::<(
            &ChildOf,
            &engine_render::prelude::Shape,
            &LocalSortOrder,
            &Visible,
            &CardFaceSide,
            &RenderLayer,
        )>();
        q.iter(world)
            .filter(|(parent, shape, _, _, _, _)| {
                parent.0 == root
                    && matches!(
                        shape.variant,
                        engine_render::prelude::ShapeVariant::Circle { .. }
                    )
            })
            .map(|(_, shape, sort, vis, side, layer)| (shape.clone(), *sort, *vis, *side, *layer))
            .collect()
    }

    #[test]
    fn when_spawn_visual_card_then_exactly_8_gem_circle_children_exist() {
        // Arrange
        let mut world = World::new();
        let def = make_test_def();

        // Act
        let root = spawn_def(&mut world, &def);

        // Assert
        let gems = gem_circle_children(&mut world, root);
        assert_eq!(gems.len(), 8, "expected 8 gem circles, got {}", gems.len());
    }

    #[test]
    fn when_spawn_visual_card_then_gems_have_correct_components() {
        // Arrange
        let mut world = World::new();
        let def = make_test_def();

        // Act
        let root = spawn_def(&mut world, &def);

        // Assert — all gems: Front face, not visible (face_down), World layer, sort 7–14
        let gems = gem_circle_children(&mut world, root);
        let mut sorts: Vec<i32> = gems.iter().map(|(_, sort, _, _, _)| sort.0).collect();
        sorts.sort_unstable();
        assert_eq!(sorts, vec![7, 8, 9, 10, 11, 12, 13, 14]);
        for (_, _, vis, side, layer) in &gems {
            assert_eq!(*side, CardFaceSide::Front);
            assert!(!vis.0, "face-down card gems should not be visible");
            assert_eq!(*layer, RenderLayer::World);
        }
    }

    #[test]
    fn when_spawn_visual_card_face_up_then_gems_are_visible() {
        // Arrange
        let mut world = World::new();
        let def = make_test_def();

        // Act
        let root = spawn_def_face_up(&mut world, &def);

        // Assert
        let gems = gem_circle_children(&mut world, root);
        assert_eq!(gems.len(), 8);
        for (_, _, vis, _, _) in &gems {
            assert!(vis.0, "face-up card gems should be visible");
        }
    }

    #[test]
    fn when_spawn_with_high_intensity_element_then_gem_radius_is_larger() {
        // Arrange — Solidum=1.0 vs Solidum=0.0, gem for Solidum is sort 7
        let def = make_test_def();
        let mut high_axes = [0.0_f32; 8];
        high_axes[0] = 1.0;
        let sig_high = CardSignature::new(high_axes);
        let sig_low = CardSignature::default();

        let mut world_high = World::new();
        let mut world_low = World::new();

        // Act
        let root_high = spawn_visual_card(
            &mut world_high,
            &def,
            Vec2::ZERO,
            Vec2::new(CARD_WIDTH, CARD_HEIGHT),
            true,
            sig_high,
        );
        let root_low = spawn_visual_card(
            &mut world_low,
            &def,
            Vec2::ZERO,
            Vec2::new(CARD_WIDTH, CARD_HEIGHT),
            true,
            sig_low,
        );

        // Assert — find gem at sort 7 (Solidum, index 0)
        let gems_high = gem_circle_children(&mut world_high, root_high);
        let gems_low = gem_circle_children(&mut world_low, root_low);
        let gem_high = gems_high
            .iter()
            .find(|(_, s, _, _, _)| s.0 == 7)
            .expect("sort 7 gem");
        let gem_low = gems_low
            .iter()
            .find(|(_, s, _, _, _)| s.0 == 7)
            .expect("sort 7 gem");

        let radius_high = match gem_high.0.variant {
            engine_render::prelude::ShapeVariant::Circle { radius } => radius,
            _ => panic!("expected circle"),
        };
        let radius_low = match gem_low.0.variant {
            engine_render::prelude::ShapeVariant::Circle { radius } => radius,
            _ => panic!("expected circle"),
        };
        assert!(
            radius_high > radius_low,
            "high-intensity radius {radius_high} should exceed low-intensity {radius_low}"
        );
    }

    #[test]
    fn when_spawn_with_positive_febris_then_gem_has_warm_color() {
        // Arrange — Febris=+0.8, gem for Febris is sort 8
        let def = make_test_def();
        let mut axes = [0.0_f32; 8];
        axes[1] = 0.8;
        let sig = CardSignature::new(axes);
        let mut world = World::new();

        // Act
        let root = spawn_visual_card(
            &mut world,
            &def,
            Vec2::ZERO,
            Vec2::new(CARD_WIDTH, CARD_HEIGHT),
            true,
            sig,
        );

        // Assert — Febris positive = Heat aspect → warm (r > b)
        let gems = gem_circle_children(&mut world, root);
        let gem = gems
            .iter()
            .find(|(_, s, _, _, _)| s.0 == 8)
            .expect("sort 8 gem (Febris)");
        assert!(
            gem.0.color.r > gem.0.color.b,
            "positive Febris gem should be warm: r={} > b={}",
            gem.0.color.r,
            gem.0.color.b
        );
    }

    #[test]
    fn when_spawn_with_negative_febris_then_gem_has_cool_color() {
        // Arrange — Febris=-0.8
        let def = make_test_def();
        let mut axes = [0.0_f32; 8];
        axes[1] = -0.8;
        let sig = CardSignature::new(axes);
        let mut world = World::new();

        // Act
        let root = spawn_visual_card(
            &mut world,
            &def,
            Vec2::ZERO,
            Vec2::new(CARD_WIDTH, CARD_HEIGHT),
            true,
            sig,
        );

        // Assert — Febris negative = Cold aspect → cool (b > r)
        let gems = gem_circle_children(&mut world, root);
        let gem = gems
            .iter()
            .find(|(_, s, _, _, _)| s.0 == 8)
            .expect("sort 8 gem (Febris)");
        assert!(
            gem.0.color.b > gem.0.color.r,
            "negative Febris gem should be cool: b={} > r={}",
            gem.0.color.b,
            gem.0.color.r
        );
    }

    fn shape_child_at_sort(
        world: &mut World,
        root: Entity,
        side: CardFaceSide,
        sort: i32,
    ) -> Option<engine_render::prelude::Shape> {
        let mut q = world.query::<(
            &ChildOf,
            &CardFaceSide,
            &engine_render::prelude::Shape,
            &LocalSortOrder,
        )>();
        q.iter(world)
            .find(|(parent, s, _, ls)| parent.0 == root && **s == side && ls.0 == sort)
            .map(|(_, _, shape, _)| shape.clone())
    }

    // --- Rounded corners integration tests ---

    #[test]
    fn when_spawn_visual_card_then_border_uses_path_variant() {
        // Arrange
        let mut world = World::new();
        let def = make_test_def();

        // Act
        let root = spawn_def(&mut world, &def);

        // Assert — front border is sort 1
        let border = shape_child_at_sort(&mut world, root, CardFaceSide::Front, 1)
            .expect("front border child");
        assert!(
            matches!(
                border.variant,
                engine_render::prelude::ShapeVariant::Path { .. }
            ),
            "front border should use Path (rounded), got {:?}",
            std::mem::discriminant(&border.variant)
        );
    }

    #[test]
    fn when_spawn_visual_card_then_back_border_uses_path_variant() {
        // Arrange
        let mut world = World::new();
        let def = make_test_def();

        // Act
        let root = spawn_def(&mut world, &def);

        // Assert — back border is sort 1
        let border = shape_child_at_sort(&mut world, root, CardFaceSide::Back, 1)
            .expect("back border child");
        assert!(
            matches!(
                border.variant,
                engine_render::prelude::ShapeVariant::Path { .. }
            ),
            "back border should use Path (rounded)"
        );
    }

    #[test]
    fn when_spawn_visual_card_then_inner_regions_stay_polygon() {
        // Arrange
        let mut world = World::new();
        let def = make_test_def();

        // Act
        let root = spawn_def(&mut world, &def);

        // Assert — front sorts 2 (name strip), 3 (art area), 4 (desc strip) should be Polygon
        for sort in [2, 3, 4] {
            let shape = shape_child_at_sort(&mut world, root, CardFaceSide::Front, sort)
                .unwrap_or_else(|| panic!("front child at sort {sort}"));
            assert!(
                matches!(
                    shape.variant,
                    engine_render::prelude::ShapeVariant::Polygon { .. }
                ),
                "front sort {sort} should stay Polygon"
            );
        }
        // Back sort 2 (inner panel) should be Polygon
        let inner =
            shape_child_at_sort(&mut world, root, CardFaceSide::Back, 2).expect("back inner panel");
        assert!(
            matches!(
                inner.variant,
                engine_render::prelude::ShapeVariant::Polygon { .. }
            ),
            "back inner panel should stay Polygon"
        );
    }

    // --- Gem repositioning integration tests ---

    #[test]
    fn when_spawn_visual_card_then_gems_within_card_bounds() {
        // Arrange
        let mut world = World::new();
        let def = make_test_def();
        let card_size = Vec2::new(CARD_WIDTH, CARD_HEIGHT);

        // Act
        let root = spawn_def(&mut world, &def);

        // Assert
        let mut q = world.query::<(&ChildOf, &engine_render::prelude::Shape, &Transform2D)>();
        let gem_positions: Vec<Vec2> = q
            .iter(&world)
            .filter(|(parent, shape, _)| {
                parent.0 == root
                    && matches!(
                        shape.variant,
                        engine_render::prelude::ShapeVariant::Circle { .. }
                    )
            })
            .map(|(_, _, t)| t.position)
            .collect();
        assert_eq!(gem_positions.len(), 8);
        let half_w = card_size.x * 0.5;
        let half_h = card_size.y * 0.5;
        for (i, pos) in gem_positions.iter().enumerate() {
            assert!(
                pos.x.abs() <= half_w && pos.y.abs() <= half_h,
                "gem {i} at ({}, {}) is outside card bounds",
                pos.x,
                pos.y
            );
        }
    }

    #[test]
    fn when_spawn_visual_card_then_desc_text_fits_within_desc_strip() {
        use crate::card::face_layout::FRONT_FACE_REGIONS;

        // Arrange
        let mut world = World::new();
        let def = make_test_def();
        let card_size = Vec2::new(CARD_WIDTH, CARD_HEIGHT);
        let (desc_half_w, _, _) = FRONT_FACE_REGIONS[3].resolve(card_size.x, card_size.y);
        let desc_full_width = desc_half_w * 2.0;

        // Act
        let root = spawn_def_face_up(&mut world, &def);

        // Assert
        let texts = text_children_for_side(&mut world, root, CardFaceSide::Front);
        let label = world.get::<CardLabel>(root).expect("label").clone();
        let desc_text = texts
            .iter()
            .find(|(t, _, _)| t.content == label.description)
            .expect("desc text");
        let max_width = desc_text.0.max_width.expect("desc should have max_width");
        assert!(
            max_width <= desc_full_width,
            "desc max_width {max_width} should fit within desc strip width {desc_full_width}"
        );
    }

    #[test]
    fn when_spawn_same_signature_twice_then_gem_colors_are_identical() {
        // Arrange
        let def = make_test_def();
        let sig = CardSignature::new([0.4, -0.6, 0.2, 0.8, -0.3, 0.1, -0.7, 0.5]);
        let mut world_a = World::new();
        let mut world_b = World::new();

        // Act
        let root_a = spawn_visual_card(
            &mut world_a,
            &def,
            Vec2::ZERO,
            Vec2::new(CARD_WIDTH, CARD_HEIGHT),
            true,
            sig,
        );
        let root_b = spawn_visual_card(
            &mut world_b,
            &def,
            Vec2::ZERO,
            Vec2::new(CARD_WIDTH, CARD_HEIGHT),
            true,
            sig,
        );

        // Assert — sort gems by LocalSortOrder and compare colors pairwise
        let mut gems_a = gem_circle_children(&mut world_a, root_a);
        let mut gems_b = gem_circle_children(&mut world_b, root_b);
        gems_a.sort_by_key(|(_, s, _, _, _)| s.0);
        gems_b.sort_by_key(|(_, s, _, _, _)| s.0);
        for (a, b) in gems_a.iter().zip(gems_b.iter()) {
            assert_eq!(
                a.0.color, b.0.color,
                "gem at sort {} should have identical color",
                a.1.0
            );
        }
    }
}
