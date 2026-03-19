use bevy_ecs::prelude::{Entity, World};
use engine_core::prelude::{Color, Pixels, Transform2D};
use engine_physics::prelude::{Collider, PhysicsRes, RigidBody};
use engine_render::prelude::{Material2d, ShaderHandle, Shape, ShapeVariant, Sprite};
use engine_scene::prelude::{ChildOf, RenderLayer, SortOrder, Visible};
use glam::Vec2;

use crate::card::Card;
use crate::card_art_shader::CardArtShader;
use crate::card_damping::{BASE_ANGULAR_DRAG, BASE_LINEAR_DRAG};
use crate::card_face_side::CardFaceSide;
use crate::card_zone::CardZone;
use crate::sort_propagation::LocalSortOrder;
use crate::stash_icon::StashIcon;
use crate::stash_render::{SLOT_HEIGHT, SLOT_WIDTH};

pub const CARD_WIDTH: f32 = 60.0;
pub const CARD_HEIGHT: f32 = 90.0;

fn rect_polygon(half_w: f32, half_h: f32) -> ShapeVariant {
    ShapeVariant::Polygon {
        points: vec![
            Vec2::new(-half_w, -half_h),
            Vec2::new(half_w, -half_h),
            Vec2::new(half_w, half_h),
            Vec2::new(-half_w, half_h),
        ],
    }
}

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

pub fn spawn_visual_card(world: &mut World, card: Card, position: Vec2, card_size: Vec2) -> Entity {
    let half = card_size * 0.5;
    let face_up = card.face_up;

    let root = world
        .spawn((
            card,
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
    spawn_front_face_children(world, root, card_size, face_up, art_shader);
    spawn_back_face_children(world, root, card_size, face_up);

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

#[allow(clippy::too_many_lines)]
fn spawn_front_face_children(
    world: &mut World,
    root: Entity,
    card_size: Vec2,
    face_up: bool,
    art_shader: Option<ShaderHandle>,
) {
    let (w, h) = (card_size.x, card_size.y);
    let children = [
        FaceChildDef {
            side: CardFaceSide::Front,
            visible: face_up,
            offset: Vec2::ZERO,
            half_w: w * 0.5,
            half_h: h * 0.5,
            color: Color::WHITE,
            sort: 1,
            shader: None,
        },
        FaceChildDef {
            side: CardFaceSide::Front,
            visible: face_up,
            offset: Vec2::new(0.0, -(h * 0.5 - 7.5)),
            half_w: w * 0.45,
            half_h: 7.5,
            color: Color::from_u8(220, 220, 220, 255),
            sort: 2,
            shader: None,
        },
        FaceChildDef {
            side: CardFaceSide::Front,
            visible: face_up,
            offset: Vec2::new(0.0, -(h * 0.1)),
            half_w: w * 0.45,
            half_h: 22.5,
            color: Color::from_u8(180, 200, 230, 255),
            sort: 3,
            shader: art_shader,
        },
        FaceChildDef {
            side: CardFaceSide::Front,
            visible: face_up,
            offset: Vec2::new(0.0, h * 0.5 - 15.0),
            half_w: w * 0.45,
            half_h: 15.0,
            color: Color::from_u8(240, 240, 200, 255),
            sort: 4,
            shader: None,
        },
    ];
    for def in &children {
        spawn_face_child(world, root, def);
    }
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

#[allow(clippy::too_many_lines)]
pub fn spawn_table_card(world: &mut World, card: Card, position: Vec2, card_size: Vec2) -> Entity {
    let half = card_size * 0.5;
    let texture = if card.face_up {
        card.face_texture
    } else {
        card.back_texture
    };

    let entity = world
        .spawn((
            card,
            CardZone::Table,
            Sprite {
                texture,
                uv_rect: [0.0, 0.0, 1.0, 1.0],
                color: Color::WHITE,
                width: Pixels(card_size.x),
                height: Pixels(card_size.y),
            },
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
        physics.add_body(entity, &RigidBody::Dynamic, position);
        physics.add_collider(entity, &Collider::Aabb(half));
        physics.set_damping(entity, BASE_LINEAR_DRAG, BASE_ANGULAR_DRAG);
    }

    entity
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::{Arc, Mutex};

    use bevy_ecs::prelude::*;
    use engine_core::prelude::{Seconds, TextureId};
    use engine_physics::prelude::{
        Collider, CollisionEvent, PhysicsBackend, PhysicsRes, RigidBody,
    };
    use engine_render::prelude::ShapeVariant;
    use engine_scene::prelude::{ChildOf, Visible};
    use glam::Vec2;

    use super::*;
    use crate::card_face_side::CardFaceSide;

    type BodyLog = Arc<Mutex<Vec<(Entity, Vec2)>>>;
    type ColliderLog = Arc<Mutex<Vec<Entity>>>;
    type DampingLog = Arc<Mutex<Vec<(Entity, f32, f32)>>>;

    struct SpyPhysicsBackend {
        bodies: BodyLog,
        colliders: ColliderLog,
        dampings: DampingLog,
    }

    impl SpyPhysicsBackend {
        fn new(bodies: BodyLog, colliders: ColliderLog, dampings: DampingLog) -> Self {
            Self {
                bodies,
                colliders,
                dampings,
            }
        }
    }

    impl PhysicsBackend for SpyPhysicsBackend {
        fn step(&mut self, _dt: Seconds) {}
        fn add_body(&mut self, entity: Entity, _body_type: &RigidBody, position: Vec2) -> bool {
            self.bodies.lock().unwrap().push((entity, position));
            true
        }
        fn add_collider(&mut self, entity: Entity, _collider: &Collider) -> bool {
            self.colliders.lock().unwrap().push(entity);
            true
        }
        fn remove_body(&mut self, _: Entity) {}
        fn body_position(&self, _: Entity) -> Option<Vec2> {
            None
        }
        fn body_rotation(&self, _: Entity) -> Option<f32> {
            None
        }
        fn drain_collision_events(&mut self) -> Vec<CollisionEvent> {
            Vec::new()
        }
        fn body_linear_velocity(&self, _: Entity) -> Option<Vec2> {
            None
        }
        fn set_linear_velocity(&mut self, _: Entity, _: Vec2) {}
        fn set_angular_velocity(&mut self, _: Entity, _: f32) {}
        fn add_force_at_point(&mut self, _: Entity, _: Vec2, _: Vec2) {}
        fn body_angular_velocity(&self, _: Entity) -> Option<f32> {
            None
        }
        fn set_damping(&mut self, entity: Entity, linear: f32, angular: f32) {
            self.dampings
                .lock()
                .unwrap()
                .push((entity, linear, angular));
        }
        fn set_collision_group(&mut self, _: Entity, _: u32, _: u32) {}
    }

    fn make_spy_world() -> (World, BodyLog, ColliderLog, DampingLog) {
        let bodies: BodyLog = Arc::new(Mutex::new(Vec::new()));
        let colliders: ColliderLog = Arc::new(Mutex::new(Vec::new()));
        let dampings: DampingLog = Arc::new(Mutex::new(Vec::new()));
        let mut world = World::new();
        world.insert_resource(PhysicsRes::new(Box::new(SpyPhysicsBackend::new(
            bodies.clone(),
            colliders.clone(),
            dampings.clone(),
        ))));
        (world, bodies, colliders, dampings)
    }

    #[test]
    fn when_spawning_table_card_then_physics_body_registered() {
        // Arrange
        let (mut world, bodies, _, _) = make_spy_world();
        let card = Card::face_down(TextureId(1), TextureId(2));
        let pos = Vec2::new(100.0, 50.0);

        // Act
        let entity = spawn_table_card(&mut world, card, pos, Vec2::new(CARD_WIDTH, CARD_HEIGHT));

        // Assert
        let calls = bodies.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, entity);
        assert_eq!(calls[0].1, pos);
    }

    #[test]
    fn when_spawning_table_card_then_physics_collider_registered() {
        // Arrange
        let (mut world, _, colliders, _) = make_spy_world();
        let card = Card::face_down(TextureId(1), TextureId(2));

        // Act
        let entity = spawn_table_card(
            &mut world,
            card,
            Vec2::ZERO,
            Vec2::new(CARD_WIDTH, CARD_HEIGHT),
        );

        // Assert
        let calls = colliders.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0], entity);
    }

    #[test]
    fn when_spawning_table_card_then_initial_damping_set() {
        // Arrange
        let (mut world, _, _, dampings) = make_spy_world();
        let card = Card::face_down(TextureId(1), TextureId(2));

        // Act
        let entity = spawn_table_card(
            &mut world,
            card,
            Vec2::ZERO,
            Vec2::new(CARD_WIDTH, CARD_HEIGHT),
        );

        // Assert
        let calls = dampings.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, entity);
        assert!((calls[0].1 - BASE_LINEAR_DRAG).abs() < 1e-4);
        assert!((calls[0].2 - BASE_ANGULAR_DRAG).abs() < 1e-4);
    }

    #[test]
    fn when_spawning_table_card_then_collider_is_half_card_size() {
        // Arrange
        let (mut world, _, _, _) = make_spy_world();
        let card = Card::face_down(TextureId(1), TextureId(2));
        let card_size = Vec2::new(80.0, 120.0);

        // Act
        let entity = spawn_table_card(&mut world, card, Vec2::ZERO, card_size);

        // Assert
        let collider = world.get::<Collider>(entity).unwrap();
        let Collider::Aabb(half) = collider else {
            panic!("expected Collider::Aabb");
        };
        let expected = card_size * 0.5;
        assert!(
            (half.x - expected.x).abs() < 1e-6 && (half.y - expected.y).abs() < 1e-6,
            "expected half={expected}, got {half}"
        );
    }

    fn children_visible_for_side(world: &mut World, root: Entity, side: CardFaceSide) -> Vec<bool> {
        let mut q = world.query::<(&ChildOf, &CardFaceSide, &Visible)>();
        q.iter(world)
            .filter(|(parent, s, _)| parent.0 == root && **s == side)
            .map(|(_, _, v)| v.0)
            .collect()
    }

    #[test]
    fn when_spawn_visual_card_face_down_then_front_children_not_visible() {
        // Arrange
        let mut world = World::new();
        let card = Card::face_down(TextureId(1), TextureId(2));

        // Act
        let root = spawn_visual_card(
            &mut world,
            card,
            Vec2::ZERO,
            Vec2::new(CARD_WIDTH, CARD_HEIGHT),
        );

        // Assert
        let results = children_visible_for_side(&mut world, root, CardFaceSide::Front);
        assert!(!results.is_empty(), "expected at least one Front child");
        assert!(results.iter().all(|v| !v));
    }

    #[test]
    fn when_spawn_visual_card_face_down_then_back_children_visible() {
        // Arrange
        let mut world = World::new();
        let card = Card::face_down(TextureId(1), TextureId(2));

        // Act
        let root = spawn_visual_card(
            &mut world,
            card,
            Vec2::ZERO,
            Vec2::new(CARD_WIDTH, CARD_HEIGHT),
        );

        // Assert
        let results = children_visible_for_side(&mut world, root, CardFaceSide::Back);
        assert!(!results.is_empty(), "expected at least one Back child");
        assert!(results.iter().all(|v| *v));
    }

    #[test]
    fn when_spawn_visual_card_face_up_then_front_visible_back_hidden() {
        // Arrange
        let mut world = World::new();
        let card = Card {
            face_texture: TextureId(1),
            back_texture: TextureId(2),
            face_up: true,
        };

        // Act
        let root = spawn_visual_card(
            &mut world,
            card,
            Vec2::ZERO,
            Vec2::new(CARD_WIDTH, CARD_HEIGHT),
        );

        // Assert
        let front = children_visible_for_side(&mut world, root, CardFaceSide::Front);
        let back = children_visible_for_side(&mut world, root, CardFaceSide::Back);
        assert!(!front.is_empty(), "expected at least one Front child");
        assert!(front.iter().all(|v| *v));
        assert!(!back.is_empty(), "expected at least one Back child");
        assert!(back.iter().all(|v| !v));
    }

    fn children_with_shape_for_side(world: &mut World, root: Entity, side: CardFaceSide) -> usize {
        let mut q = world.query::<(&ChildOf, &CardFaceSide, &engine_render::prelude::Shape)>();
        q.iter(world)
            .filter(|(parent, s, _)| parent.0 == root && **s == side)
            .count()
    }

    #[test]
    fn when_spawn_visual_card_then_front_face_has_four_shape_children() {
        // Arrange
        let mut world = World::new();
        let card = Card::face_down(TextureId(1), TextureId(2));

        // Act
        let root = spawn_visual_card(
            &mut world,
            card,
            Vec2::ZERO,
            Vec2::new(CARD_WIDTH, CARD_HEIGHT),
        );

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
        let card = Card::face_down(TextureId(1), TextureId(2));

        // Act
        let root = spawn_visual_card(
            &mut world,
            card,
            Vec2::ZERO,
            Vec2::new(CARD_WIDTH, CARD_HEIGHT),
        );

        // Assert
        assert!(children_with_shape_for_side(&mut world, root, CardFaceSide::Back) >= 2);
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

    #[test]
    fn when_spawn_visual_card_then_front_border_matches_card_dimensions() {
        // Arrange
        let mut world = World::new();
        let card = Card::face_down(TextureId(1), TextureId(2));

        // Act
        let root = spawn_visual_card(
            &mut world,
            card,
            Vec2::ZERO,
            Vec2::new(CARD_WIDTH, CARD_HEIGHT),
        );

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
        let card = Card::face_down(TextureId(1), TextureId(2));

        // Act
        let root = spawn_visual_card(
            &mut world,
            card,
            Vec2::ZERO,
            Vec2::new(CARD_WIDTH, CARD_HEIGHT),
        );

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
        let mut registry = engine_render::prelude::ShaderRegistry::new();
        let art_shader = crate::card_art_shader::register_card_art_shader(&mut registry);
        world.insert_resource(registry);
        world.insert_resource(art_shader);
        let card = Card {
            face_texture: TextureId(1),
            back_texture: TextureId(2),
            face_up: true,
        };

        // Act
        let root = spawn_visual_card(
            &mut world,
            card,
            Vec2::ZERO,
            Vec2::new(CARD_WIDTH, CARD_HEIGHT),
        );

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

    #[test]
    fn when_spawn_visual_card_then_root_collider_half_is_card_size_times_half() {
        // Arrange
        let mut world = World::new();
        let card = Card::face_down(TextureId(1), TextureId(2));
        let card_size = Vec2::new(100.0, 200.0);

        // Act
        let root = spawn_visual_card(&mut world, card, Vec2::ZERO, card_size);

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
        // Arrange — card_size=(100, 200), w=100, h=200
        // Name strip (Front sort=2): offset=(0, -(h*0.5 - 7.5))=(0, -92.5), half_w=w*0.45=45
        let mut world = World::new();
        let card = Card::face_down(TextureId(1), TextureId(2));
        let card_size = Vec2::new(100.0, 200.0);

        // Act
        let root = spawn_visual_card(&mut world, card, Vec2::ZERO, card_size);

        // Assert
        let (t, shape) = child_by_side_and_sort(&mut world, root, CardFaceSide::Front, 2)
            .expect("front sort=2 child");
        assert!((t.position.y - (-92.5)).abs() < 1e-4, "y={}", t.position.y);
        let (hw, _) = polygon_half_extents(&shape);
        assert!((hw - 45.0).abs() < 1e-4, "half_w={hw}");
    }

    #[test]
    fn when_spawn_visual_card_then_front_art_area_position_and_size_correct() {
        // Arrange — art area (Front sort=3): offset=(0, -(h*0.1))=(0, -20), half_w=w*0.45=45
        let mut world = World::new();
        let card = Card::face_down(TextureId(1), TextureId(2));
        let card_size = Vec2::new(100.0, 200.0);

        // Act
        let root = spawn_visual_card(&mut world, card, Vec2::ZERO, card_size);

        // Assert
        let (t, shape) = child_by_side_and_sort(&mut world, root, CardFaceSide::Front, 3)
            .expect("front sort=3 child");
        assert!((t.position.y - (-20.0)).abs() < 1e-4, "y={}", t.position.y);
        let (hw, _) = polygon_half_extents(&shape);
        assert!((hw - 45.0).abs() < 1e-4, "half_w={hw}");
    }

    #[test]
    fn when_spawn_visual_card_then_front_description_strip_position_and_size_correct() {
        // Arrange — desc strip (Front sort=4): offset=(0, h*0.5 - 15)=(0, 85), half_w=w*0.45=45
        let mut world = World::new();
        let card = Card::face_down(TextureId(1), TextureId(2));
        let card_size = Vec2::new(100.0, 200.0);

        // Act
        let root = spawn_visual_card(&mut world, card, Vec2::ZERO, card_size);

        // Assert
        let (t, shape) = child_by_side_and_sort(&mut world, root, CardFaceSide::Front, 4)
            .expect("front sort=4 child");
        assert!((t.position.y - 85.0).abs() < 1e-4, "y={}", t.position.y);
        let (hw, _) = polygon_half_extents(&shape);
        assert!((hw - 45.0).abs() < 1e-4, "half_w={hw}");
    }

    #[test]
    fn when_spawn_visual_card_then_back_border_matches_card_half_size() {
        // Arrange — back border (Back sort=1): half_w=w*0.5=50, half_h=h*0.5=100
        let mut world = World::new();
        let card = Card::face_down(TextureId(1), TextureId(2));
        let card_size = Vec2::new(100.0, 200.0);

        // Act
        let root = spawn_visual_card(&mut world, card, Vec2::ZERO, card_size);

        // Assert
        let (_, shape) = child_by_side_and_sort(&mut world, root, CardFaceSide::Back, 1)
            .expect("back sort=1 child");
        let (hw, hh) = polygon_half_extents(&shape);
        assert!((hw - 50.0).abs() < 1e-4, "half_w={hw}");
        assert!((hh - 100.0).abs() < 1e-4, "half_h={hh}");
    }

    #[test]
    fn when_spawn_visual_card_then_back_center_pattern_uses_thirty_percent() {
        // Arrange — back center (Back sort=2): half_w=w*0.3=30, half_h=h*0.3=60
        let mut world = World::new();
        let card = Card::face_down(TextureId(1), TextureId(2));
        let card_size = Vec2::new(100.0, 200.0);

        // Act
        let root = spawn_visual_card(&mut world, card, Vec2::ZERO, card_size);

        // Assert
        let (_, shape) = child_by_side_and_sort(&mut world, root, CardFaceSide::Back, 2)
            .expect("back sort=2 child");
        let (hw, hh) = polygon_half_extents(&shape);
        assert!((hw - 30.0).abs() < 1e-4, "half_w={hw}");
        assert!((hh - 60.0).abs() < 1e-4, "half_h={hh}");
    }

    #[test]
    fn when_spawn_visual_card_then_front_border_half_size_matches_card_half() {
        // Arrange — front border (Front sort=1): half_w=w*0.5=50, half_h=h*0.5=100
        let mut world = World::new();
        let card = Card::face_down(TextureId(1), TextureId(2));
        let card_size = Vec2::new(100.0, 200.0);

        // Act
        let root = spawn_visual_card(&mut world, card, Vec2::ZERO, card_size);

        // Assert
        let (_, shape) = child_by_side_and_sort(&mut world, root, CardFaceSide::Front, 1)
            .expect("front sort=1 child");
        let (hw, hh) = polygon_half_extents(&shape);
        assert!((hw - 50.0).abs() < 1e-4, "half_w={hw}");
        assert!((hh - 100.0).abs() < 1e-4, "half_h={hh}");
    }

    fn find_stash_icon_child(world: &mut World, root: Entity) -> Option<Entity> {
        let mut q = world.query::<(Entity, &ChildOf, &crate::stash_icon::StashIcon)>();
        q.iter(world)
            .find(|(_, parent, _)| parent.0 == root)
            .map(|(e, _, _)| e)
    }

    #[test]
    fn when_spawn_visual_card_then_exactly_one_stash_icon_child_exists() {
        // Arrange
        let mut world = World::new();
        let card = Card::face_down(TextureId(1), TextureId(2));

        // Act
        let root = spawn_visual_card(
            &mut world,
            card,
            Vec2::ZERO,
            Vec2::new(CARD_WIDTH, CARD_HEIGHT),
        );

        // Assert
        let mut q = world.query::<(Entity, &ChildOf, &crate::stash_icon::StashIcon)>();
        let count = q.iter(&world).filter(|(_, p, _)| p.0 == root).count();
        assert_eq!(count, 1, "expected exactly one StashIcon child");
    }

    #[test]
    fn when_spawn_visual_card_then_stash_icon_half_extents_match_slot_dimensions() {
        // Arrange
        let mut world = World::new();
        let card = Card::face_down(TextureId(1), TextureId(2));
        let card_size = Vec2::new(CARD_WIDTH, CARD_HEIGHT);

        // Act
        let root = spawn_visual_card(&mut world, card, Vec2::ZERO, card_size);

        // Assert — icon must match slot size (50×75), not card size (60×90)
        let icon = find_stash_icon_child(&mut world, root).expect("StashIcon child must exist");
        let shape = world
            .get::<engine_render::prelude::Shape>(icon)
            .expect("StashIcon must have Shape");
        let ShapeVariant::Polygon { ref points } = shape.variant else {
            panic!("expected Polygon variant");
        };
        let max_x = points.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
        let max_y = points.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
        let expected_half_w = crate::stash_render::SLOT_WIDTH * 0.5;
        let expected_half_h = crate::stash_render::SLOT_HEIGHT * 0.5;
        assert!(
            (max_x - expected_half_w).abs() < 1e-4,
            "half_w={max_x} expected {expected_half_w} (SLOT_WIDTH/2, not CARD_WIDTH/2)"
        );
        assert!(
            (max_y - expected_half_h).abs() < 1e-4,
            "half_h={max_y} expected {expected_half_h} (SLOT_HEIGHT/2, not CARD_HEIGHT/2)"
        );
    }

    #[test]
    fn when_spawn_visual_card_then_stash_icon_child_is_hidden() {
        // Arrange
        let mut world = World::new();
        let card = Card::face_down(TextureId(1), TextureId(2));

        // Act
        let root = spawn_visual_card(
            &mut world,
            card,
            Vec2::ZERO,
            Vec2::new(CARD_WIDTH, CARD_HEIGHT),
        );

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
        let card = Card::face_down(TextureId(1), TextureId(2));

        // Act
        let root = spawn_visual_card(
            &mut world,
            card,
            Vec2::ZERO,
            Vec2::new(CARD_WIDTH, CARD_HEIGHT),
        );

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
}
