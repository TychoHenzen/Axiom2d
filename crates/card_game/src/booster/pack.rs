// EVOLVE-BLOCK-START
// BoosterPack entity — spawn, pickup, physics

use bevy_ecs::prelude::{Component, Entity, World};
use engine_core::prelude::{Color, EventBus, Transform2D};
use engine_physics::prelude::{Collider, PhysicsCommand, RigidBody};
use engine_render::prelude::{Shape, ShapeVariant, Stroke};
use engine_scene::render_order::{RenderLayer, SortOrder};
use glam::Vec2;

use crate::card::component::CardZone;
use crate::card::identity::signature::CardSignature;
use crate::card::interaction::click_resolve::{ClickHitShape, Clickable, on_card_clicked};
use crate::card::interaction::damping::{BASE_ANGULAR_DRAG, BASE_LINEAR_DRAG};
use crate::card::rendering::geometry::{TABLE_CARD_HEIGHT, TABLE_CARD_WIDTH};

/// A sealed booster pack containing cards that can be opened.
#[derive(Component, Debug, Clone, PartialEq)]
pub struct BoosterPack {
    pub cards: Vec<CardSignature>,
}

pub const PACK_WIDTH: f32 = TABLE_CARD_WIDTH * 1.1; // 66.0
pub const PACK_HEIGHT: f32 = TABLE_CARD_HEIGHT * 1.3; // 117.0
const BODY_HEIGHT: f32 = TABLE_CARD_HEIGHT * 1.1; // 99.0
const TEETH_COUNT: usize = 6;
const PACK_DRAG_MULTIPLIER: f32 = 2.5;

/// Build the polygon points for a booster pack shape with jagged top and bottom edges.
#[must_use]
pub fn pack_shape_points() -> Vec<Vec2> {
    let half_w = PACK_WIDTH / 2.0;
    let half_h = PACK_HEIGHT / 2.0;
    let body_half_h = BODY_HEIGHT / 2.0;
    let tooth_width = PACK_WIDTH / TEETH_COUNT as f32;

    let mut points = Vec::new();

    // Bottom-left corner
    points.push(Vec2::new(-half_w, -body_half_h));

    // Bottom jagged edge (left to right), teeth point downward
    for i in 0..TEETH_COUNT {
        let x_left = -half_w + tooth_width * i as f32;
        let x_mid = x_left + tooth_width / 2.0;
        let x_right = x_left + tooth_width;
        // Tooth tip (downward)
        points.push(Vec2::new(x_mid, -half_h));
        // Back up to body edge
        points.push(Vec2::new(x_right, -body_half_h));
    }

    // Right side — go up to top body edge
    // (last bottom-right point was already added by the loop)

    // Top jagged edge (right to left), teeth point upward
    for i in (0..TEETH_COUNT).rev() {
        let x_left = -half_w + tooth_width * i as f32;
        let x_mid = x_left + tooth_width / 2.0;
        let x_right = x_left + tooth_width;
        // Start at top body edge on the right side of this tooth
        points.push(Vec2::new(x_right, body_half_h));
        // Tooth tip (upward)
        points.push(Vec2::new(x_mid, half_h));
    }

    // Close back to bottom-left via top-left corner
    points.push(Vec2::new(-half_w, body_half_h));

    points
}

/// Spawn a booster pack entity at the given position containing the given cards.
pub fn spawn_booster_pack(world: &mut World, position: Vec2, cards: Vec<CardSignature>) -> Entity {
    let half = Vec2::new(PACK_WIDTH / 2.0, PACK_HEIGHT / 2.0);

    let fill_color = Color {
        r: 0.15,
        g: 0.12,
        b: 0.20,
        a: 1.0,
    };
    let stroke_color = Color {
        r: 0.80,
        g: 0.65,
        b: 0.20,
        a: 1.0,
    };

    let root = world
        .spawn((
            BoosterPack { cards },
            CardZone::Table,
            Transform2D {
                position,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
            RigidBody::Dynamic,
            Collider::Aabb(half),
            Shape {
                variant: ShapeVariant::Polygon {
                    points: pack_shape_points(),
                },
                color: fill_color,
            },
            Stroke {
                color: stroke_color,
                width: 2.0,
            },
            RenderLayer::World,
            SortOrder::default(),
            Clickable(ClickHitShape::Aabb(half)),
        ))
        .id();

    world.entity_mut(root).observe(on_card_clicked);

    if let Some(mut bus) = world.get_resource_mut::<EventBus<PhysicsCommand>>() {
        bus.push(PhysicsCommand::AddBody {
            entity: root,
            body_type: RigidBody::Dynamic,
            position,
        });
        bus.push(PhysicsCommand::AddCollider {
            entity: root,
            collider: Collider::Aabb(half),
        });
        bus.push(PhysicsCommand::SetDamping {
            entity: root,
            linear: BASE_LINEAR_DRAG * PACK_DRAG_MULTIPLIER,
            angular: BASE_ANGULAR_DRAG * PACK_DRAG_MULTIPLIER,
        });
    }

    root
}
// EVOLVE-BLOCK-END
