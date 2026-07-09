// Cable physics simulation — types, propagation, rendering, and geometry.
//
// Split across mod.rs (types + propagation), geom.rs (geometry utilities),
// and render.rs (ECS render/wrap systems + WrapWire impl).

use bevy_ecs::prelude::{Component, Entity, Query};
use engine_core::color::Color;
use glam::Vec2;

use crate::card::reader::SignatureSpace;

pub mod geom;
pub mod render;

// Re-export public items from submodules
pub use geom::{point_in_convex_polygon, polyline_to_ribbon, segment_intersects_segment};
pub use render::{wire_render_system, wrap_detect_system, wrap_update_system};

pub(crate) const CABLE_COLOR: Color = Color {
    r: 0.7,
    g: 0.6,
    b: 0.3,
    a: 0.9,
};
pub(crate) const CABLE_HALF_THICKNESS: f32 = 1.5;
pub(crate) const CABLE_LOCAL_SORT: i32 = -2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JackDirection {
    Input,
    Output,
}

#[derive(Component, Debug, Clone)]
pub struct Jack<T: Clone + Send + Sync + 'static> {
    pub direction: JackDirection,
    pub data: Option<T>,
}

#[derive(Component, Debug, Clone)]
pub struct Cable {
    pub source: Entity,
    pub dest: Entity,
}

// ---------------------------------------------------------------------------
// Cable collider (convex polygon for wrap detection)
// ---------------------------------------------------------------------------

#[derive(Component, Debug, Clone)]
pub struct CableCollider {
    /// Convex hull vertices in local space, wound counter-clockwise.
    pub vertices: Vec<Vec2>,
}

impl CableCollider {
    /// Construct from AABB half-extents.
    pub fn from_aabb(half: Vec2) -> Self {
        Self {
            vertices: vec![
                Vec2::new(-half.x, -half.y),
                Vec2::new(half.x, -half.y),
                Vec2::new(half.x, half.y),
                Vec2::new(-half.x, half.y),
            ],
        }
    }
}

// ---------------------------------------------------------------------------
// Wrap detection (geometric — no particle simulation)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct WrapAnchor {
    /// World-space position of this anchor (polygon vertex).
    pub position: Vec2,
    /// Which obstacle entity this anchor belongs to.
    pub obstacle: Entity,
    /// Index into that obstacle's CableCollider.vertices.
    pub vertex_index: usize,
    /// Boundary step along the polygon: +1 moves to the next CCW vertex, -1 moves to the previous.
    pub boundary_step: i8,
    /// Wrap direction: +1.0 for CCW wrap, -1.0 for CW wrap.
    pub wrap_sign: f32,
}

#[derive(Component, Debug, Clone, Default)]
pub struct WrapWire {
    /// Ordered anchor points from source toward dest.
    pub anchors: Vec<WrapAnchor>,
}

impl WrapWire {
    pub fn new() -> Self {
        Self::default()
    }
}

// ---------------------------------------------------------------------------
// Wire endpoint component
// ---------------------------------------------------------------------------

#[derive(Component, Debug, Clone)]
pub struct WireEndpoints {
    pub source: Entity,
    pub dest: Entity,
}

// ---------------------------------------------------------------------------
// ECS systems
// ---------------------------------------------------------------------------

/// Propagate `SignatureSpace` data along cables from source jacks to dest jacks.
pub fn signature_space_propagation_system(
    cables: Query<&Cable>,
    mut jacks: Query<&mut Jack<SignatureSpace>>,
) {
    let updates: Vec<(Entity, Option<SignatureSpace>)> = cables
        .iter()
        .filter_map(|cable| {
            let data = jacks.get(cable.source).ok()?.data.clone();
            Some((cable.dest, data))
        })
        .collect();

    for (dest, data) in updates {
        if let Ok(mut jack) = jacks.get_mut(dest) {
            jack.data = data;
        }
    }
}
