// EVOLVE-BLOCK-START
use bevy_ecs::prelude::*;
use engine_core::prelude::{Color, Transform2D};
use engine_physics::prelude::{Collider, RigidBody};
use engine_render::prelude::{Shape, ShapeVariant, Stroke, rounded_rect_path};
use engine_scene::prelude::{LocalSortOrder, SpawnChildExt};
use engine_scene::render_order::{RenderLayer, SortOrder};
use glam::Vec2;

use crate::card::interaction::click_resolve::{ClickHitShape, Clickable};
use crate::card::jack_cable::{CableCollider, Jack, JackDirection};
use crate::card::jack_socket::{JackSocket, on_socket_clicked};
use crate::card::reader::components::CardReader;
use crate::card::reader::glow::{ReaderAccent, ReaderRecess, ReaderRune};
use crate::card::reader::pick::on_reader_clicked;
use crate::card::reader::signature_space::SignatureSpace;

// --- Dimensions -----------------------------------------------------------

const READER_HALF_W: f32 = 40.0;
const READER_HALF_H: f32 = 55.0;
const BASE_CORNER_RADIUS: f32 = 4.0;

const RECESS_HALF_W: f32 = 30.0;
const RECESS_HALF_H: f32 = 42.0;
const RECESS_CORNER_RADIUS: f32 = 2.0;

const RUNE_RADIUS: f32 = 4.0;

const READER_SOCKET_RADIUS: f32 = 8.0;
const READER_SOCKET_COLOR: Color = Color {
    r: 0.7,
    g: 0.6,
    b: 0.3,
    a: 1.0,
};
const READER_SOCKET_LOCAL_SORT: i32 = 1;
pub(crate) const READER_JACK_OFFSET: Vec2 =
    Vec2::new(READER_HALF_W + READER_SOCKET_RADIUS + 4.0, 0.0);

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

const RECESS_FILL: Color = Color {
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
    let jack_entity = world
        .spawn((
            Jack::<SignatureSpace> {
                direction: JackDirection::Output,
                data: None,
            },
            JackSocket {
                radius: READER_SOCKET_RADIUS,
                color: READER_SOCKET_COLOR,
                connected_cable: None,
            },
            Transform2D {
                position: position + READER_JACK_OFFSET,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
            Shape {
                variant: ShapeVariant::Circle {
                    radius: READER_SOCKET_RADIUS,
                },
                color: READER_SOCKET_COLOR,
            },
            RenderLayer::World,
            SortOrder::default(),
            LocalSortOrder(READER_SOCKET_LOCAL_SORT),
            Clickable(ClickHitShape::Circle(READER_SOCKET_RADIUS)),
        ))
        .id();

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
            CableCollider::from_aabb(half),
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
            Clickable(ClickHitShape::Aabb(half)),
        ))
        .id();
    world.entity_mut(reader_entity).observe(on_reader_clicked);
    world.entity_mut(jack_entity).observe(on_socket_clicked);

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

/// The reader's half-extents used for physics registration.
pub const READER_HALF_EXTENTS: Vec2 = Vec2::new(READER_HALF_W, READER_HALF_H);
// EVOLVE-BLOCK-END
