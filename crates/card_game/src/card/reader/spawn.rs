use bevy_ecs::prelude::*;
use engine_core::prelude::{Color, Transform2D};
use engine_physics::prelude::{Collider, RigidBody};
use engine_render::prelude::{Shape, ShapeVariant, Stroke, rounded_rect_path};
use engine_scene::prelude::{LocalSortOrder, SpawnChildExt};
use engine_scene::render_order::{RenderLayer, SortOrder};
use glam::Vec2;

use super::components::{CardReader, OutputJack};
use crate::card::reader::glow::{ReaderAccent, ReaderRecess, ReaderRune};

// --- Dimensions -----------------------------------------------------------

const READER_HALF_W: f32 = 40.0;
const READER_HALF_H: f32 = 55.0;
const BASE_CORNER_RADIUS: f32 = 4.0;

const RECESS_HALF_W: f32 = 30.0;
const RECESS_HALF_H: f32 = 42.0;
const RECESS_CORNER_RADIUS: f32 = 2.0;

const RUNE_RADIUS: f32 = 4.0;

const ACCENT_HALF_W: f32 = 25.0;
const ACCENT_THICKNESS: f32 = 2.0;
const ACCENT_Y: f32 = -50.0;

// --- Colors ---------------------------------------------------------------

const BASE_FILL: Color = Color {
    r: 0.235,
    g: 0.216,
    b: 0.196,
    a: 1.0,
};

const BASE_STROKE: Color = Color {
    r: 0.471,
    g: 0.392,
    b: 0.275,
    a: 1.0,
};

pub const RECESS_FILL: Color = Color {
    r: 0.118,
    g: 0.110,
    b: 0.098,
    a: 1.0,
};

pub const RECESS_STROKE_DIM: Color = Color {
    r: 0.510,
    g: 0.431,
    b: 0.235,
    a: 0.40,
};

pub const RECESS_STROKE_LIT: Color = Color {
    r: 0.784,
    g: 0.647,
    b: 0.235,
    a: 0.80,
};

pub const RUNE_COLOR_DIM: Color = Color {
    r: 0.706,
    g: 0.588,
    b: 0.196,
    a: 0.25,
};

pub const RUNE_COLOR_LIT: Color = Color {
    r: 0.902,
    g: 0.784,
    b: 0.235,
    a: 0.80,
};

pub const ACCENT_COLOR_DIM: Color = Color {
    r: 0.784,
    g: 0.667,
    b: 0.235,
    a: 0.25,
};

pub const ACCENT_COLOR_LIT: Color = Color {
    r: 0.941,
    g: 0.824,
    b: 0.235,
    a: 0.80,
};

/// Spawns a card reader altar with child entities for the visual elements.
///
/// Returns `(reader_entity, jack_entity)`. The caller is responsible for
/// registering the reader's physics body and collider via `PhysicsRes`.
pub fn spawn_reader(world: &mut World, position: Vec2) -> (Entity, Entity) {
    let jack_entity = world.spawn(OutputJack { data: None }).id();

    let half = Vec2::new(READER_HALF_W, READER_HALF_H);

    let reader_entity = world
        .spawn((
            CardReader {
                loaded: None,
                half_extents: half,
                jack_entity,
            },
            Transform2D {
                position,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
            RigidBody::Kinematic,
            Collider::Aabb(half),
            Shape {
                variant: rounded_rect_path(READER_HALF_W, READER_HALF_H, BASE_CORNER_RADIUS),
                color: BASE_FILL,
            },
            Stroke {
                color: BASE_STROKE,
                width: 2.0,
            },
            RenderLayer::World,
            SortOrder::default(),
            LocalSortOrder(-1),
        ))
        .id();

    // Inner recess
    world.spawn_child(
        reader_entity,
        (
            ReaderRecess,
            Transform2D::default(),
            Shape {
                variant: rounded_rect_path(RECESS_HALF_W, RECESS_HALF_H, RECESS_CORNER_RADIUS),
                color: RECESS_FILL,
            },
            Stroke {
                color: RECESS_STROKE_DIM,
                width: 1.0,
            },
            RenderLayer::World,
            SortOrder::default(),
            LocalSortOrder(1),
        ),
    );

    // Accent line (thin rectangle across the top)
    let accent_points = vec![
        Vec2::new(-ACCENT_HALF_W, ACCENT_Y - ACCENT_THICKNESS * 0.5),
        Vec2::new(ACCENT_HALF_W, ACCENT_Y - ACCENT_THICKNESS * 0.5),
        Vec2::new(ACCENT_HALF_W, ACCENT_Y + ACCENT_THICKNESS * 0.5),
        Vec2::new(-ACCENT_HALF_W, ACCENT_Y + ACCENT_THICKNESS * 0.5),
    ];
    world.spawn_child(
        reader_entity,
        (
            ReaderAccent,
            Transform2D::default(),
            Shape {
                variant: ShapeVariant::Polygon {
                    points: accent_points,
                },
                color: ACCENT_COLOR_DIM,
            },
            RenderLayer::World,
            SortOrder::default(),
            LocalSortOrder(2),
        ),
    );

    // Corner runes
    let rune_positions = [
        Vec2::new(-RECESS_HALF_W, -RECESS_HALF_H), // top-left
        Vec2::new(RECESS_HALF_W, -RECESS_HALF_H),  // top-right
        Vec2::new(-RECESS_HALF_W, RECESS_HALF_H),  // bottom-left
        Vec2::new(RECESS_HALF_W, RECESS_HALF_H),   // bottom-right
    ];

    for &pos in &rune_positions {
        world.spawn_child(
            reader_entity,
            (
                ReaderRune,
                Transform2D {
                    position: pos,
                    ..Default::default()
                },
                Shape {
                    variant: ShapeVariant::Circle {
                        radius: RUNE_RADIUS,
                    },
                    color: RUNE_COLOR_DIM,
                },
                RenderLayer::World,
                SortOrder::default(),
                LocalSortOrder(2),
            ),
        );
    }

    (reader_entity, jack_entity)
}

/// Returns the reader's half-extents used for physics registration.
pub fn reader_half_extents() -> Vec2 {
    Vec2::new(READER_HALF_W, READER_HALF_H)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use engine_scene::prelude::{ChildOf, Children, hierarchy_maintenance_system};

    use super::*;

    fn run_hierarchy(world: &mut World) {
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(hierarchy_maintenance_system);
        schedule.run(world);
    }

    #[test]
    fn when_spawn_reader_then_root_has_card_reader_component() {
        // Arrange
        let mut world = World::new();

        // Act
        let (reader, _jack) = spawn_reader(&mut world, Vec2::new(100.0, 50.0));

        // Assert
        assert!(world.get::<CardReader>(reader).is_some());
    }

    #[test]
    fn when_spawn_reader_then_root_has_rounded_rect_shape() {
        // Arrange
        let mut world = World::new();

        // Act
        let (reader, _) = spawn_reader(&mut world, Vec2::ZERO);

        // Assert
        let shape = world.get::<Shape>(reader).unwrap();
        assert!(
            matches!(shape.variant, ShapeVariant::Path { .. }),
            "expected rounded rect (Path variant)"
        );
    }

    #[test]
    fn when_spawn_reader_then_root_has_stroke() {
        // Arrange
        let mut world = World::new();

        // Act
        let (reader, _) = spawn_reader(&mut world, Vec2::ZERO);

        // Assert
        assert!(world.get::<Stroke>(reader).is_some());
    }

    #[test]
    fn when_spawn_reader_then_six_children_exist() {
        // Arrange
        let mut world = World::new();
        let (reader, _) = spawn_reader(&mut world, Vec2::ZERO);

        // Act
        run_hierarchy(&mut world);

        // Assert — 1 recess + 1 accent + 4 runes = 6 children
        let children = world.get::<Children>(reader).unwrap();
        assert_eq!(children.0.len(), 6);
    }

    #[test]
    fn when_spawn_reader_then_four_rune_children_exist() {
        // Arrange
        let mut world = World::new();
        let (reader, _) = spawn_reader(&mut world, Vec2::ZERO);
        run_hierarchy(&mut world);

        // Act
        let children = &world.get::<Children>(reader).unwrap().0;
        let rune_count = children
            .iter()
            .filter(|e| world.get::<ReaderRune>(**e).is_some())
            .count();

        // Assert
        assert_eq!(rune_count, 4);
    }

    #[test]
    fn when_spawn_reader_then_recess_child_exists() {
        // Arrange
        let mut world = World::new();
        let (reader, _) = spawn_reader(&mut world, Vec2::ZERO);
        run_hierarchy(&mut world);

        // Act
        let children = &world.get::<Children>(reader).unwrap().0;
        let recess_count = children
            .iter()
            .filter(|e| world.get::<ReaderRecess>(**e).is_some())
            .count();

        // Assert
        assert_eq!(recess_count, 1);
    }

    #[test]
    fn when_spawn_reader_then_accent_child_exists() {
        // Arrange
        let mut world = World::new();
        let (reader, _) = spawn_reader(&mut world, Vec2::ZERO);
        run_hierarchy(&mut world);

        // Act
        let children = &world.get::<Children>(reader).unwrap().0;
        let accent_count = children
            .iter()
            .filter(|e| world.get::<ReaderAccent>(**e).is_some())
            .count();

        // Assert
        assert_eq!(accent_count, 1);
    }

    #[test]
    fn when_spawn_reader_then_children_have_child_of_pointing_to_reader() {
        // Arrange
        let mut world = World::new();
        let (reader, _) = spawn_reader(&mut world, Vec2::ZERO);
        run_hierarchy(&mut world);

        // Act / Assert
        let children = &world.get::<Children>(reader).unwrap().0;
        for &child in children {
            let child_of = world.get::<ChildOf>(child).unwrap();
            assert_eq!(child_of.0, reader);
        }
    }

    #[test]
    fn when_spawn_reader_then_position_matches_argument() {
        // Arrange
        let mut world = World::new();
        let pos = Vec2::new(300.0, -100.0);

        // Act
        let (reader, _) = spawn_reader(&mut world, pos);

        // Assert
        let transform = world.get::<Transform2D>(reader).unwrap();
        assert_eq!(transform.position, pos);
    }

    #[test]
    fn when_spawn_reader_then_runes_are_circles() {
        // Arrange
        let mut world = World::new();
        let (reader, _) = spawn_reader(&mut world, Vec2::ZERO);
        run_hierarchy(&mut world);

        // Act / Assert
        let children = &world.get::<Children>(reader).unwrap().0;
        for &child in children {
            if world.get::<ReaderRune>(child).is_some() {
                let shape = world.get::<Shape>(child).unwrap();
                assert!(
                    matches!(shape.variant, ShapeVariant::Circle { .. }),
                    "rune should be a circle"
                );
            }
        }
    }

    #[test]
    fn when_spawn_reader_then_runes_start_dim() {
        // Arrange
        let mut world = World::new();
        let (reader, _) = spawn_reader(&mut world, Vec2::ZERO);
        run_hierarchy(&mut world);

        // Act / Assert
        let children = &world.get::<Children>(reader).unwrap().0;
        for &child in children {
            if world.get::<ReaderRune>(child).is_some() {
                let shape = world.get::<Shape>(child).unwrap();
                assert!(
                    shape.color.a < 0.5,
                    "rune should start dim (alpha < 0.5), got {}",
                    shape.color.a
                );
            }
        }
    }

    #[test]
    fn when_spawn_reader_then_jack_entity_is_returned() {
        // Arrange
        let mut world = World::new();

        // Act
        let (_reader, jack) = spawn_reader(&mut world, Vec2::ZERO);

        // Assert
        assert!(world.get::<OutputJack>(jack).is_some());
    }

    #[test]
    fn when_spawn_reader_then_reader_references_jack() {
        // Arrange
        let mut world = World::new();

        // Act
        let (reader, jack) = spawn_reader(&mut world, Vec2::ZERO);

        // Assert
        let card_reader = world.get::<CardReader>(reader).unwrap();
        assert_eq!(card_reader.jack_entity, jack);
    }
}
