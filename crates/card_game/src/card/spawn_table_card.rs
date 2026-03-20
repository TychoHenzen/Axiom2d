use bevy_ecs::prelude::{Entity, World};
use engine_core::prelude::{Color, TextureId, Transform2D};
use engine_physics::prelude::{Collider, PhysicsRes, RigidBody};
use engine_render::prelude::{Material2d, ShaderHandle, Shape};
use engine_scene::prelude::{ChildOf, RenderLayer, SortOrder, Visible};
use glam::Vec2;

use crate::card::art_shader::CardArtShader;
use crate::card::component::Card;
use crate::card::damping::{BASE_ANGULAR_DRAG, BASE_LINEAR_DRAG};
use crate::card::definition::{CardDefinition, description_from_abilities, rarity_border_color};
use crate::card::face_layout::FRONT_FACE_REGIONS;
use crate::card::face_side::CardFaceSide;
use crate::card::geometry::rect_polygon;
use crate::card::label::CardLabel;
use crate::card::zone::CardZone;
use crate::stash::constants::{SLOT_HEIGHT, SLOT_WIDTH};
use crate::stash::icon::StashIcon;
use engine_scene::sort_propagation::LocalSortOrder;
use engine_ui::prelude::Text;

struct FaceChildDef {
    side: CardFaceSide,
    visible: bool,
    offset: Vec2,
    half_w: f32,
    half_h: f32,
    color: Color,
    sort: i32,
    shader: Option<ShaderHandle>,
}

fn spawn_face_child(world: &mut World, root: Entity, def: &FaceChildDef) {
    let entity = world
        .spawn((
            ChildOf(root),
            def.side,
            Visible(def.visible),
            Shape {
                variant: rect_polygon(def.half_w, def.half_h),
                color: def.color,
            },
            Transform2D {
                position: def.offset,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
            RenderLayer::World,
            LocalSortOrder(def.sort),
            SortOrder(0),
        ))
        .id();
    if let Some(shader) = def.shader {
        world.entity_mut(entity).insert(Material2d {
            shader,
            ..Material2d::default()
        });
    }
}

pub fn spawn_visual_card(
    world: &mut World,
    def: &CardDefinition,
    position: Vec2,
    card_size: Vec2,
    face_up: bool,
) -> Entity {
    let half = card_size * 0.5;

    let card = Card {
        face_texture: TextureId(0),
        back_texture: TextureId(0),
        face_up,
    };
    let label = CardLabel {
        name: def.name.clone(),
        description: description_from_abilities(&def.abilities),
    };
    let border_color = rarity_border_color(def.rarity);

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
        physics.set_damping(root, BASE_LINEAR_DRAG, BASE_ANGULAR_DRAG);
    }

    let art_shader = world.get_resource::<CardArtShader>().map(|s| s.0);
    spawn_front_face_children(world, root, card_size, face_up, art_shader, border_color);
    spawn_back_face_children(world, root, card_size, face_up);
    spawn_text_children(world, root, card_size, face_up, &label);

    let icon = world
        .spawn((
            ChildOf(root),
            StashIcon,
            Shape {
                variant: rect_polygon(SLOT_WIDTH * 0.5, SLOT_HEIGHT * 0.5),
                color: Color::from_u8(180, 200, 230, 255),
            },
            Visible(false),
            Transform2D {
                position: Vec2::ZERO,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
            RenderLayer::UI,
            LocalSortOrder(1),
            SortOrder(0),
        ))
        .id();
    if let Some(shader) = art_shader {
        world.entity_mut(icon).insert(Material2d {
            shader,
            ..Material2d::default()
        });
    }

    root
}

fn spawn_front_face_children(
    world: &mut World,
    root: Entity,
    card_size: Vec2,
    face_up: bool,
    art_shader: Option<ShaderHandle>,
    border_color: Color,
) {
    let (w, h) = (card_size.x, card_size.y);
    for (i, region) in FRONT_FACE_REGIONS.iter().enumerate() {
        let (half_w, half_h, offset_y) = region.resolve(w, h);
        let shader = if region.use_art_shader {
            art_shader
        } else {
            None
        };
        let color = if i == 0 { border_color } else { region.color };
        spawn_face_child(
            world,
            root,
            &FaceChildDef {
                side: CardFaceSide::Front,
                visible: face_up,
                offset: Vec2::new(0.0, offset_y),
                half_w,
                half_h,
                color,
                sort: (i + 1) as i32,
                shader,
            },
        );
    }
}

const NAME_TEXT_SORT: i32 = 5;
const DESC_TEXT_SORT: i32 = 6;
const TEXT_COLOR: Color = Color {
    r: 0.1,
    g: 0.1,
    b: 0.1,
    a: 1.0,
};

fn spawn_text_children(
    world: &mut World,
    root: Entity,
    card_size: Vec2,
    face_up: bool,
    label: &CardLabel,
) {
    let h = card_size.y;

    let (_, _, name_offset_y) = FRONT_FACE_REGIONS[1].resolve(card_size.x, h);
    let name_font_size = h / 12.0;
    world.spawn((
        ChildOf(root),
        CardFaceSide::Front,
        Visible(face_up),
        Text {
            content: label.name.clone(),
            font_size: name_font_size,
            color: TEXT_COLOR,
            max_width: None,
        },
        Transform2D {
            position: Vec2::new(0.0, name_offset_y),
            rotation: 0.0,
            scale: Vec2::ONE,
        },
        RenderLayer::World,
        LocalSortOrder(NAME_TEXT_SORT),
        SortOrder(0),
    ));

    let (desc_half_w, _, desc_offset_y) = FRONT_FACE_REGIONS[3].resolve(card_size.x, h);
    let desc_font_size = h / 16.0;
    let desc_max_width = desc_half_w * 2.0 * 0.9;
    world.spawn((
        ChildOf(root),
        CardFaceSide::Front,
        Visible(face_up),
        Text {
            content: label.description.clone(),
            font_size: desc_font_size,
            color: TEXT_COLOR,
            max_width: Some(desc_max_width),
        },
        Transform2D {
            position: Vec2::new(0.0, desc_offset_y),
            rotation: 0.0,
            scale: Vec2::ONE,
        },
        RenderLayer::World,
        LocalSortOrder(DESC_TEXT_SORT),
        SortOrder(0),
    ));
}

fn spawn_back_face_children(world: &mut World, root: Entity, card_size: Vec2, face_up: bool) {
    let (w, h) = (card_size.x, card_size.y);
    let children = [
        FaceChildDef {
            side: CardFaceSide::Back,
            visible: !face_up,
            offset: Vec2::ZERO,
            half_w: w * 0.5,
            half_h: h * 0.5,
            color: Color::from_u8(30, 60, 120, 255),
            sort: 1,
            shader: None,
        },
        FaceChildDef {
            side: CardFaceSide::Back,
            visible: !face_up,
            offset: Vec2::ZERO,
            half_w: w * 0.3,
            half_h: h * 0.3,
            color: Color::from_u8(60, 100, 180, 255),
            sort: 2,
            shader: None,
        },
    ];
    for def in &children {
        spawn_face_child(world, root, def);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use bevy_ecs::prelude::*;
    use engine_render::prelude::ShapeVariant;
    use engine_scene::prelude::{ChildOf, Visible};
    use glam::Vec2;

    use super::*;
    use crate::card::definition::{
        CardAbilities, CardDefinition, CardType, Keyword, Rarity, art_descriptor_default,
        rarity_border_color,
    };
    use crate::card::face_side::CardFaceSide;
    use crate::card::geometry::{TABLE_CARD_HEIGHT as CARD_HEIGHT, TABLE_CARD_WIDTH as CARD_WIDTH};

    fn make_test_def() -> CardDefinition {
        CardDefinition {
            card_type: CardType::Spell,
            rarity: Rarity::Common,
            name: "Fireball".to_owned(),
            stats: None,
            abilities: CardAbilities {
                keywords: vec![],
                text: "Deal 3 damage".to_owned(),
            },
            art: art_descriptor_default(CardType::Spell),
        }
    }

    fn make_test_def_with_rarity(rarity: Rarity) -> CardDefinition {
        CardDefinition {
            rarity,
            ..make_test_def()
        }
    }

    fn spawn_def(world: &mut World, def: &CardDefinition) -> Entity {
        spawn_visual_card(
            world,
            def,
            Vec2::ZERO,
            Vec2::new(CARD_WIDTH, CARD_HEIGHT),
            false,
        )
    }

    fn spawn_def_face_up(world: &mut World, def: &CardDefinition) -> Entity {
        spawn_visual_card(
            world,
            def,
            Vec2::ZERO,
            Vec2::new(CARD_WIDTH, CARD_HEIGHT),
            true,
        )
    }

    fn children_visible_for_side(world: &mut World, root: Entity, side: CardFaceSide) -> Vec<bool> {
        let mut q = world.query::<(&ChildOf, &CardFaceSide, &Visible)>();
        q.iter(world)
            .filter(|(parent, s, _)| parent.0 == root && **s == side)
            .map(|(_, _, v)| v.0)
            .collect()
    }

    fn children_with_shape_for_side(world: &mut World, root: Entity, side: CardFaceSide) -> usize {
        let mut q = world.query::<(&ChildOf, &CardFaceSide, &engine_render::prelude::Shape)>();
        q.iter(world)
            .filter(|(parent, s, _)| parent.0 == root && **s == side)
            .count()
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

    fn child_by_side_and_sort(
        world: &mut World,
        root: Entity,
        side: CardFaceSide,
        sort: i32,
    ) -> Option<(Transform2D, engine_render::prelude::Shape)> {
        let mut q = world.query::<(
            &ChildOf,
            &CardFaceSide,
            &LocalSortOrder,
            &Transform2D,
            &engine_render::prelude::Shape,
        )>();
        q.iter(world)
            .find(|(parent, s, local_sort, _, _)| {
                parent.0 == root && **s == side && local_sort.0 == sort
            })
            .map(|(_, _, _, t, s)| (*t, s.clone()))
    }

    fn polygon_half_extents(shape: &engine_render::prelude::Shape) -> (f32, f32) {
        let ShapeVariant::Polygon { ref points } = shape.variant else {
            panic!("expected Polygon");
        };
        let max_x = points.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
        let max_y = points.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
        (max_x, max_y)
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

    // --- New behavior tests ---

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
    fn when_spawn_visual_card_then_root_has_card_label_with_name_from_definition() {
        // Arrange
        let mut world = World::new();
        let def = make_test_def();

        // Act
        let root = spawn_def(&mut world, &def);

        // Assert
        let label = world
            .get::<CardLabel>(root)
            .expect("root should have CardLabel");
        assert_eq!(label.name, "Fireball");
    }

    #[test]
    fn when_spawn_visual_card_then_card_label_description_from_abilities() {
        // Arrange
        let mut world = World::new();
        let def = make_test_def();

        // Act
        let root = spawn_def(&mut world, &def);

        // Assert
        let label = world
            .get::<CardLabel>(root)
            .expect("root should have CardLabel");
        assert_eq!(label.description, "Deal 3 damage");
    }

    #[test]
    fn when_spawn_visual_card_with_keywords_then_description_includes_keyword_names() {
        // Arrange
        let mut world = World::new();
        let def = CardDefinition {
            abilities: CardAbilities {
                keywords: vec![Keyword::Taunt],
                text: String::new(),
            },
            ..make_test_def()
        };

        // Act
        let root = spawn_def(&mut world, &def);

        // Assert
        let label = world
            .get::<CardLabel>(root)
            .expect("root should have CardLabel");
        assert!(
            label.description.contains("Taunt"),
            "desc={}",
            label.description
        );
    }

    #[test]
    fn when_spawn_common_rarity_then_border_color_matches_rarity() {
        // Arrange
        let mut world = World::new();
        let def = make_test_def_with_rarity(Rarity::Common);

        // Act
        let root = spawn_def(&mut world, &def);

        // Assert
        let border =
            border_shape_for_side(&mut world, root, CardFaceSide::Front).expect("front border");
        assert_eq!(border.color, rarity_border_color(Rarity::Common));
    }

    #[test]
    fn when_spawn_legendary_rarity_then_border_color_is_golden_not_white() {
        // Arrange
        let mut world = World::new();
        let def = make_test_def_with_rarity(Rarity::Legendary);

        // Act
        let root = spawn_def(&mut world, &def);

        // Assert
        let border =
            border_shape_for_side(&mut world, root, CardFaceSide::Front).expect("front border");
        assert_eq!(border.color, rarity_border_color(Rarity::Legendary));
        assert_ne!(border.color, Color::WHITE);
    }

    // --- Ported existing tests ---

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
    fn when_spawn_visual_card_then_front_face_has_four_shape_children() {
        // Arrange
        let mut world = World::new();
        let def = make_test_def();

        // Act
        let root = spawn_def(&mut world, &def);

        // Assert
        assert_eq!(
            children_with_shape_for_side(&mut world, root, CardFaceSide::Front),
            4
        );
    }

    #[test]
    fn when_spawn_visual_card_then_back_face_has_at_least_two_shape_children() {
        // Arrange
        let mut world = World::new();
        let def = make_test_def();

        // Act
        let root = spawn_def(&mut world, &def);

        // Assert
        assert!(children_with_shape_for_side(&mut world, root, CardFaceSide::Back) >= 2);
    }

    #[test]
    fn when_spawn_visual_card_then_front_border_matches_card_dimensions() {
        // Arrange
        let mut world = World::new();
        let def = make_test_def();

        // Act
        let root = spawn_def(&mut world, &def);

        // Assert
        let border = border_shape_for_side(&mut world, root, CardFaceSide::Front)
            .expect("front border should exist");
        let ShapeVariant::Polygon { ref points } = border.variant else {
            panic!("expected Polygon variant for border");
        };
        let min_x = points.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
        let max_x = points.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
        let min_y = points.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
        let max_y = points.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
        let width = max_x - min_x;
        let height = max_y - min_y;
        assert!(
            (width - CARD_WIDTH).abs() < 1e-4,
            "width={width} expected {CARD_WIDTH}"
        );
        assert!(
            (height - CARD_HEIGHT).abs() < 1e-4,
            "height={height} expected {CARD_HEIGHT}"
        );
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
        let root = spawn_visual_card(&mut world, &def, Vec2::ZERO, card_size, false);

        // Assert
        let collider = world.get::<Collider>(root).unwrap();
        let Collider::Aabb(half) = collider else {
            panic!("expected Collider::Aabb");
        };
        assert!((half.x - 50.0).abs() < 1e-4, "half.x={}", half.x);
        assert!((half.y - 100.0).abs() < 1e-4, "half.y={}", half.y);
    }

    #[test]
    fn when_spawn_visual_card_then_front_name_strip_position_and_size_correct() {
        // Arrange
        let mut world = World::new();
        let def = make_test_def();
        let card_size = Vec2::new(100.0, 200.0);

        // Act
        let root = spawn_visual_card(&mut world, &def, Vec2::ZERO, card_size, false);

        // Assert
        let (t, shape) = child_by_side_and_sort(&mut world, root, CardFaceSide::Front, 2)
            .expect("front sort=2 child");
        let expected_y = -(200.0 * 0.5 - 200.0 / 12.0);
        assert!(
            (t.position.y - expected_y).abs() < 1e-2,
            "y={}, expected {expected_y}",
            t.position.y
        );
        let (hw, _) = polygon_half_extents(&shape);
        assert!((hw - 45.0).abs() < 1e-4, "half_w={hw}");
    }

    #[test]
    fn when_spawn_visual_card_then_front_art_area_position_and_size_correct() {
        // Arrange
        let mut world = World::new();
        let def = make_test_def();
        let card_size = Vec2::new(100.0, 200.0);

        // Act
        let root = spawn_visual_card(&mut world, &def, Vec2::ZERO, card_size, false);

        // Assert
        let (t, shape) = child_by_side_and_sort(&mut world, root, CardFaceSide::Front, 3)
            .expect("front sort=3 child");
        assert!((t.position.y - (-20.0)).abs() < 1e-4, "y={}", t.position.y);
        let (hw, _) = polygon_half_extents(&shape);
        assert!((hw - 45.0).abs() < 1e-4, "half_w={hw}");
    }

    #[test]
    fn when_spawn_visual_card_then_front_description_strip_position_and_size_correct() {
        // Arrange
        let mut world = World::new();
        let def = make_test_def();
        let card_size = Vec2::new(100.0, 200.0);

        // Act
        let root = spawn_visual_card(&mut world, &def, Vec2::ZERO, card_size, false);

        // Assert
        let (t, shape) = child_by_side_and_sort(&mut world, root, CardFaceSide::Front, 4)
            .expect("front sort=4 child");
        let expected_y = 200.0 * 0.5 - 200.0 / 6.0;
        assert!(
            (t.position.y - expected_y).abs() < 1e-2,
            "y={}, expected {expected_y}",
            t.position.y
        );
        let (hw, _) = polygon_half_extents(&shape);
        assert!((hw - 45.0).abs() < 1e-4, "half_w={hw}");
    }

    #[test]
    fn when_spawn_visual_card_then_back_border_matches_card_half_size() {
        // Arrange
        let mut world = World::new();
        let def = make_test_def();
        let card_size = Vec2::new(100.0, 200.0);

        // Act
        let root = spawn_visual_card(&mut world, &def, Vec2::ZERO, card_size, false);

        // Assert
        let (_, shape) = child_by_side_and_sort(&mut world, root, CardFaceSide::Back, 1)
            .expect("back sort=1 child");
        let (hw, hh) = polygon_half_extents(&shape);
        assert!((hw - 50.0).abs() < 1e-4, "half_w={hw}");
        assert!((hh - 100.0).abs() < 1e-4, "half_h={hh}");
    }

    #[test]
    fn when_spawn_visual_card_then_back_center_pattern_uses_thirty_percent() {
        // Arrange
        let mut world = World::new();
        let def = make_test_def();
        let card_size = Vec2::new(100.0, 200.0);

        // Act
        let root = spawn_visual_card(&mut world, &def, Vec2::ZERO, card_size, false);

        // Assert
        let (_, shape) = child_by_side_and_sort(&mut world, root, CardFaceSide::Back, 2)
            .expect("back sort=2 child");
        let (hw, hh) = polygon_half_extents(&shape);
        assert!((hw - 30.0).abs() < 1e-4, "half_w={hw}");
        assert!((hh - 60.0).abs() < 1e-4, "half_h={hh}");
    }

    #[test]
    fn when_spawn_visual_card_then_front_border_half_size_matches_card_half() {
        // Arrange
        let mut world = World::new();
        let def = make_test_def();
        let card_size = Vec2::new(100.0, 200.0);

        // Act
        let root = spawn_visual_card(&mut world, &def, Vec2::ZERO, card_size, false);

        // Assert
        let (_, shape) = child_by_side_and_sort(&mut world, root, CardFaceSide::Front, 1)
            .expect("front sort=1 child");
        let (hw, hh) = polygon_half_extents(&shape);
        assert!((hw - 50.0).abs() < 1e-4, "half_w={hw}");
        assert!((hh - 100.0).abs() < 1e-4, "half_h={hh}");
    }

    #[test]
    fn when_spawn_visual_card_then_exactly_one_stash_icon_child_exists() {
        // Arrange
        let mut world = World::new();
        let def = make_test_def();

        // Act
        let root = spawn_def(&mut world, &def);

        // Assert
        let mut q = world.query::<(Entity, &ChildOf, &crate::stash::icon::StashIcon)>();
        let count = q.iter(&world).filter(|(_, p, _)| p.0 == root).count();
        assert_eq!(count, 1, "expected exactly one StashIcon child");
    }

    #[test]
    fn when_spawn_visual_card_then_stash_icon_half_extents_match_slot_dimensions() {
        // Arrange
        let mut world = World::new();
        let def = make_test_def();
        let card_size = Vec2::new(CARD_WIDTH, CARD_HEIGHT);

        // Act
        let root = spawn_visual_card(&mut world, &def, Vec2::ZERO, card_size, false);

        // Assert
        let icon = find_stash_icon_child(&mut world, root).expect("StashIcon child must exist");
        let shape = world
            .get::<engine_render::prelude::Shape>(icon)
            .expect("StashIcon must have Shape");
        let ShapeVariant::Polygon { ref points } = shape.variant else {
            panic!("expected Polygon variant");
        };
        let max_x = points.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
        let max_y = points.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
        let expected_half_w = crate::stash::constants::SLOT_WIDTH * 0.5;
        let expected_half_h = crate::stash::constants::SLOT_HEIGHT * 0.5;
        assert!(
            (max_x - expected_half_w).abs() < 1e-4,
            "half_w={max_x} expected {expected_half_w}"
        );
        assert!(
            (max_y - expected_half_h).abs() < 1e-4,
            "half_h={max_y} expected {expected_half_h}"
        );
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
    fn when_spawn_visual_card_then_name_text_matches_definition_name() {
        // Arrange
        let mut world = World::new();
        let def = make_test_def();

        // Act
        let root = spawn_def_face_up(&mut world, &def);

        // Assert
        let texts = text_children_for_side(&mut world, root, CardFaceSide::Front);
        let name_text = texts.iter().find(|(t, _, _)| t.content == "Fireball");
        assert!(name_text.is_some(), "expected text child with name content");
    }

    #[test]
    fn when_spawn_visual_card_then_description_text_matches_abilities() {
        // Arrange
        let mut world = World::new();
        let def = make_test_def();

        // Act
        let root = spawn_def_face_up(&mut world, &def);

        // Assert
        let texts = text_children_for_side(&mut world, root, CardFaceSide::Front);
        let desc_text = texts.iter().find(|(t, _, _)| t.content == "Deal 3 damage");
        assert!(
            desc_text.is_some(),
            "expected text child with description content"
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
        let texts = text_children_for_side(&mut world, root, CardFaceSide::Front);
        let name_text = texts
            .iter()
            .find(|(t, _, _)| t.content == "Fireball")
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
        let texts = text_children_for_side(&mut world, root, CardFaceSide::Front);
        let desc_text = texts
            .iter()
            .find(|(t, _, _)| t.content == "Deal 3 damage")
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
}
