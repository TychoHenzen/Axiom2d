// EVOLVE-BLOCK-START
use bevy_ecs::prelude::{Component, Entity, Query, Without};
use engine_core::color::Color;
use engine_core::prelude::Transform2D;
use engine_render::prelude::{Shape, ShapeVariant};
use engine_scene::prelude::Visible;
use glam::Vec2;

use crate::card::reader::SignatureSpace;

const CABLE_COLOR: Color = Color {
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

fn polygon_centroid(polygon: &[Vec2]) -> Vec2 {
    if polygon.is_empty() {
        return Vec2::ZERO;
    }
    let sum = polygon.iter().copied().fold(Vec2::ZERO, |acc, v| acc + v);
    sum / polygon.len() as f32
}

fn polygon_edges(polygon: &[Vec2]) -> impl Iterator<Item = (Vec2, Vec2)> + '_ {
    polygon
        .iter()
        .copied()
        .zip(polygon.iter().copied().cycle().skip(1))
        .take(polygon.len())
}

fn segment_intersects_convex_polygon(a: Vec2, b: Vec2, polygon: &[Vec2]) -> bool {
    if polygon.len() < 3 {
        return false;
    }
    if point_in_convex_polygon(a, polygon) || point_in_convex_polygon(b, polygon) {
        return true;
    }
    polygon_edges(polygon).any(|(p1, p2)| {
        segment_intersects_segment(a, b, p1, p2).is_some_and(|t| t > 1e-4 && t < 1.0 - 1e-4)
    })
}

fn segment_crosses_convex_polygon(a: Vec2, b: Vec2, polygon: &[Vec2]) -> Option<f32> {
    if polygon.len() < 3 {
        return None;
    }

    let endpoint_on_vertex = |p: Vec2| polygon.iter().any(|v| (p - *v).length_squared() <= 1e-6);
    let mut first_t: Option<f32> = None;
    for (p1, p2) in polygon_edges(polygon) {
        if let Some(t) = segment_intersects_segment(a, b, p1, p2)
            && t > 1e-4
            && t < 1.0 - 1e-4
        {
            if let Some(prev_t) = first_t {
                return Some(prev_t.min(t));
            }
            first_t = Some(t);
        }
    }

    if first_t.is_some() && (endpoint_on_vertex(a) || endpoint_on_vertex(b)) {
        first_t
    } else {
        None
    }
}

pub fn point_in_convex_polygon(point: Vec2, polygon: &[Vec2]) -> bool {
    if polygon.len() < 3 {
        return false;
    }

    let mut sign: Option<f32> = None;
    for (a, b) in polygon_edges(polygon) {
        let cross = (b - a).perp_dot(point - a);
        if cross.abs() <= 1e-5 {
            continue;
        }

        let edge_sign = cross.signum();
        if let Some(expected) = sign {
            if expected != edge_sign {
                return false;
            }
        } else {
            sign = Some(edge_sign);
        }
    }
    true
}

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

/// Sine of the minimum angular overshoot past the straight-through direction
/// before an anchor is unwrapped via turn reversal.  ~9 degrees — provides
/// hysteresis that prevents rapid wrap/unwrap oscillation when the cable sits
/// near the decision boundary.
const UNWRAP_SIN_THRESHOLD: f32 = 0.15;

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

    fn boundary_neighbor_index(
        vertex_index: usize,
        boundary_step: i8,
        vertex_count: usize,
    ) -> usize {
        let vertex_count = vertex_count.max(1) as isize;
        let next = vertex_index as isize + boundary_step as isize;
        next.rem_euclid(vertex_count) as usize
    }

    fn select_wrap_vertex(
        verts: &[Vec2],
        hit: Vec2,
        prev_anchor: Option<&WrapAnchor>,
    ) -> Option<usize> {
        let n = verts.len();
        if let Some(prev) = prev_anchor {
            if prev.vertex_index < n {
                return Some(Self::boundary_neighbor_index(
                    prev.vertex_index,
                    prev.boundary_step,
                    n,
                ));
            }
            return None;
        }

        verts
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                ((*a - hit).length_squared()).total_cmp(&((*b - hit).length_squared()))
            })
            .map(|(idx, _)| idx)
    }

    /// Check each span for polygon intersections and insert wrap anchors.
    ///
    /// Each iteration of the outer loop adds at most one anchor, then restarts
    /// the scan from scratch with updated pins.  The `already` check on vertex
    /// index prevents re-adding the same vertex, so multiple corners of the
    /// same obstacle are added across successive iterations (not frames).
    pub fn detect_wraps(&mut self, src: Vec2, dst: Vec2, obstacles: &[(Entity, &[Vec2])]) {
        // Each iteration adds one anchor.  The total vertex count across all
        // obstacles is the hard upper bound on useful iterations.
        let max_iters = obstacles.iter().map(|(_, v)| v.len()).sum::<usize>().max(1);
        for _ in 0..max_iters {
            // Rebuild pin list from current anchor state each iteration,
            // since insertions invalidate the previous pin list.
            let mut pins: Vec<Vec2> = Vec::with_capacity(self.anchors.len() + 2);
            pins.push(src);
            for anchor in &self.anchors {
                pins.push(anchor.position);
            }
            pins.push(dst);

            let mut found_any = false;
            let mut insert_idx = 0;
            let mut i = 0;
            while i < pins.len() - 1 {
                let span_a = pins[i];
                let span_b = pins[i + 1];

                let mut found = None;
                'obstacle: for &(entity, verts) in obstacles {
                    let n = verts.len();
                    if n < 3 {
                        continue;
                    }
                    // We only create a wrap when the span actually crosses the
                    // polygon boundary in more than one place. A one-point graze
                    // against the middle of an edge should not snap the cable to
                    // a corner.
                    let Some(first_t) = segment_crosses_convex_polygon(span_a, span_b, verts)
                    else {
                        continue 'obstacle;
                    };
                    let span_dir = span_b - span_a;
                    let hit = span_a + span_dir * first_t;

                    // Only use boundary-neighbor logic when the anchor
                    // immediately preceding this span belongs to the same
                    // obstacle — i.e. we are extending an existing wrap.
                    // Using the *last* anchor on the entity anywhere in the
                    // list caused wrong vertex selection when the span
                    // bridges two different obstacles (e.g. reader→display).
                    let prev_anchor = if insert_idx > 0 {
                        let a = &self.anchors[insert_idx - 1];
                        if a.obstacle == entity { Some(a) } else { None }
                    } else {
                        None
                    };

                    let best_idx = Self::select_wrap_vertex(verts, hit, prev_anchor);

                    if let Some(vidx) = best_idx
                        && !self
                            .anchors
                            .iter()
                            .any(|a| a.obstacle == entity && a.vertex_index == vidx)
                    {
                        let v = verts[vidx];
                        // wrap_sign: expected bend direction at this anchor.
                        // Computed from the actual bend: (v - prev) × (next - v).
                        let bend = (v - span_a).perp_dot(span_b - v);
                        let wrap_sign = if bend.abs() > 1e-6 {
                            bend.signum()
                        } else if let Some(prev) = prev_anchor {
                            prev.wrap_sign
                        } else {
                            // Vertex on cable line, no prior anchor — wrap away
                            // from polygon interior.
                            let c = span_dir.perp_dot(polygon_centroid(verts) - span_a);
                            if c.abs() > 1e-6 { -c.signum() } else { 1.0 }
                        };
                        // boundary_step: consistent walk direction for this obstacle.
                        // Inherit from preceding anchor on same obstacle; otherwise
                        // derive from wrap_sign.
                        let boundary_step = if let Some(prev) = prev_anchor {
                            prev.boundary_step
                        } else if wrap_sign > 0.0 {
                            1i8
                        } else {
                            -1i8
                        };
                        found = Some(WrapAnchor {
                            position: v,
                            obstacle: entity,
                            vertex_index: vidx,
                            boundary_step,
                            wrap_sign,
                        });
                        break 'obstacle;
                    }
                }

                if let Some(anchor) = found {
                    self.anchors.insert(insert_idx, anchor);
                    found_any = true;
                    break; // Restart scan with updated pins
                }
                insert_idx += 1;
                i += 1;
            }

            if !found_any {
                break;
            }
        }
    }

    /// Remove anchors only after the cable has clearly swung past the wrap point.
    /// We intentionally do not unwrap just because the straight line would be
    /// able to bypass the obstacle, since that causes mid-wrap side flipping.
    pub fn detect_unwraps(&mut self, src: Vec2, dst: Vec2, obstacles: &[(Entity, &[Vec2])]) {
        loop {
            let mut changed = false;
            let mut i = self.anchors.len();
            while i > 0 {
                i -= 1;
                let prev = if i == 0 {
                    src
                } else {
                    self.anchors[i - 1].position
                };
                let next = if i + 1 < self.anchors.len() {
                    self.anchors[i + 1].position
                } else {
                    dst
                };

                let to_anchor = self.anchors[i].position - prev;
                let from_anchor = next - self.anchors[i].position;
                let cross = to_anchor.perp_dot(from_anchor);

                let to_len = to_anchor.length();
                let from_len = from_anchor.length();
                let sin_angle = if to_len > 1e-6 && from_len > 1e-6 {
                    cross / (to_len * from_len)
                } else {
                    0.0
                };
                let turn_reversed = sin_angle * self.anchors[i].wrap_sign < -UNWRAP_SIN_THRESHOLD;
                // When two consecutive anchors share an obstacle, `prev` is a
                // polygon vertex and point_in_convex_polygon treats it as
                // "inside", permanently blocking the shortcut.  Nudge the
                // start slightly along the segment to move off the boundary.
                // Only nudge for same-obstacle chains — nudging when prev is
                // src or on a different obstacle can false-clear the shortcut
                // (e.g. src sitting on the obstacle boundary).
                let same_obstacle_prev =
                    i > 0 && self.anchors[i - 1].obstacle == self.anchors[i].obstacle;
                let shortcut_start = if same_obstacle_prev {
                    prev + (next - prev) * 0.005
                } else {
                    prev
                };
                let shortcut_clear = obstacles
                    .iter()
                    .find(|(entity, _)| *entity == self.anchors[i].obstacle)
                    .is_none_or(|(_, verts)| {
                        !segment_intersects_convex_polygon(shortcut_start, next, verts)
                    });

                if turn_reversed || shortcut_clear {
                    self.anchors.remove(i);
                    changed = true;
                }
            }

            if !changed {
                break;
            }
        }
    }

    /// Compute the shortest geometric path from `src` through all anchors to `dst`.
    pub fn shortest_path(&self, src: Vec2, dst: Vec2) -> f32 {
        self.waypoints(src, dst)
            .windows(2)
            .map(|pair| (pair[1] - pair[0]).length())
            .sum()
    }

    /// Return the full waypoint list: `[src, anchor1, ..., anchorN, dst]`.
    pub fn waypoints(&self, src: Vec2, dst: Vec2) -> Vec<Vec2> {
        let mut waypoints = Vec::with_capacity(self.anchors.len() + 2);
        waypoints.push(src);
        waypoints.extend(self.anchors.iter().map(|anchor| anchor.position));
        waypoints.push(dst);
        waypoints
    }
}

// ---------------------------------------------------------------------------
// Geometry utilities
// ---------------------------------------------------------------------------

/// Returns the parameter `t` along segment (a1→a2) where it intersects segment (b1→b2).
/// Returns `None` if the segments don't intersect. Both `t` and `u` must be in [0, 1].
pub fn segment_intersects_segment(a1: Vec2, a2: Vec2, b1: Vec2, b2: Vec2) -> Option<f32> {
    let d1 = a2 - a1;
    let d2 = b2 - b1;
    let denom = d1.perp_dot(d2);
    if denom.abs() < 1e-10 {
        return None; // parallel
    }
    let diff = b1 - a1;
    let t = diff.perp_dot(d2) / denom;
    let u = diff.perp_dot(d1) / denom;
    if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
        Some(t)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Wire rendering (geometric — no simulation)
// ---------------------------------------------------------------------------

/// Maximum spacing between sample points along the wire.
const WIRE_SAMPLE_SPACING: f32 = 12.0;

/// Number of Catmull-Rom subdivisions per densified segment.
const SUBDIVISIONS_PER_SEGMENT: usize = 4;

/// Densify a polyline by inserting intermediate points so no segment exceeds
/// `max_spacing`.  This constrains the subsequent Catmull-Rom subdivision to
/// follow straight segments closely and only curve in a small radius around
/// the original waypoint corners.
fn densify_polyline(waypoints: &[Vec2], max_spacing: f32) -> Vec<Vec2> {
    if waypoints.len() < 2 {
        return waypoints.to_vec();
    }
    let mut result = Vec::with_capacity(waypoints.len() * 4);
    result.push(waypoints[0]);
    for pair in waypoints.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let dist = (b - a).length();
        let n = ((dist / max_spacing).ceil() as usize).max(1);
        for i in 1..n {
            let t = i as f32 / n as f32;
            result.push(a.lerp(b, t));
        }
        result.push(b);
    }
    result
}

/// Build a ribbon polygon from a polyline of waypoints.
///
/// First densifies the polyline so no segment exceeds `WIRE_SAMPLE_SPACING`,
/// then offsets left/right perpendicular to the local tangent at each sample.
/// Subdivides each edge with Catmull-Rom interpolation to produce slight
/// rounding at corners (where the cable wraps around an obstacle vertex).
///
/// Returns vertices going forward along one edge, then backward along the other.
pub fn polyline_to_ribbon(waypoints: &[Vec2], half_thickness: f32) -> Vec<Vec2> {
    let dense = densify_polyline(waypoints, WIRE_SAMPLE_SPACING);
    let n = dense.len();
    if n < 2 {
        return vec![];
    }
    let mut left_ctrl = Vec::with_capacity(n);
    let mut right_ctrl = Vec::with_capacity(n);
    for i in 0..n {
        let tangent = if i == 0 {
            dense[1] - dense[0]
        } else if i == n - 1 {
            dense[n - 1] - dense[n - 2]
        } else {
            dense[i + 1] - dense[i - 1]
        };
        let perp = Vec2::new(-tangent.y, tangent.x).normalize_or_zero() * half_thickness;
        left_ctrl.push(dense[i] + perp);
        right_ctrl.push(dense[i] - perp);
    }
    // Catmull-Rom subdivide for slight rounding at corners
    let mut left = catmull_rom_subdivide(&left_ctrl, SUBDIVISIONS_PER_SEGMENT);
    let mut right = catmull_rom_subdivide(&right_ctrl, SUBDIVISIONS_PER_SEGMENT);
    right.reverse();
    left.extend(right);
    left
}

/// Evaluate a Catmull-Rom spline through `points` at evenly spaced parameter values,
/// producing `(points.len() - 1) * subdivisions + 1` output samples.
fn catmull_rom_subdivide(points: &[Vec2], subdivisions: usize) -> Vec<Vec2> {
    let n = points.len();
    if n < 2 {
        return points.to_vec();
    }
    let mut result = Vec::with_capacity((n - 1) * subdivisions + 1);
    for i in 0..n - 1 {
        let p0 = if i == 0 { points[0] } else { points[i - 1] };
        let p1 = points[i];
        let p2 = points[i + 1];
        let p3 = if i + 2 < n {
            points[i + 2]
        } else {
            points[n - 1]
        };

        for s in 0..subdivisions {
            let t = s as f32 / subdivisions as f32;
            let t2 = t * t;
            let t3 = t2 * t;
            let pos = 0.5
                * ((2.0 * p1)
                    + (-p0 + p2) * t
                    + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2
                    + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t3);
            result.push(pos);
        }
    }
    result.push(points[n - 1]);
    result
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

/// Update anchor world positions from obstacle transforms each frame.
pub fn wrap_update_system(
    mut wires: Query<&mut WrapWire>,
    colliders: Query<(&Transform2D, &CableCollider), Without<WrapWire>>,
) {
    for mut wrap in &mut wires {
        for anchor in &mut wrap.anchors {
            if let Ok((transform, collider)) = colliders.get(anchor.obstacle)
                && let Some(local_vert) = collider.vertices.get(anchor.vertex_index)
            {
                anchor.position = *local_vert + transform.position;
            }
        }
    }
}

/// Detect wrap and unwrap events based on cable span vs obstacle polygon intersections.
pub fn wrap_detect_system(
    mut wires: Query<(&mut WrapWire, &WireEndpoints)>,
    transforms: Query<&Transform2D, Without<WrapWire>>,
    colliders: Query<(Entity, &Transform2D, &CableCollider), Without<WrapWire>>,
) {
    let obstacles: Vec<(Entity, Vec<Vec2>)> = colliders
        .iter()
        .map(|(entity, t, c)| {
            let world_verts: Vec<Vec2> = c.vertices.iter().map(|v| *v + t.position).collect();
            (entity, world_verts)
        })
        .collect();
    let obstacle_refs: Vec<(Entity, &[Vec2])> =
        obstacles.iter().map(|(e, v)| (*e, v.as_slice())).collect();

    for (mut wrap, endpoints) in &mut wires {
        let Ok(src_t) = transforms.get(endpoints.source) else {
            continue;
        };
        let Ok(dst_t) = transforms.get(endpoints.dest) else {
            continue;
        };
        let src = src_t.position;
        let dst = dst_t.position;

        wrap.detect_unwraps(src, dst, &obstacle_refs);
        wrap.detect_wraps(src, dst, &obstacle_refs);
    }
}

/// Render each wire as a ribbon polygon along its geometric path.
pub fn wire_render_system(
    mut wires: Query<(
        &WireEndpoints,
        Option<&WrapWire>,
        &mut Transform2D,
        &mut Shape,
        &mut Visible,
    )>,
    transforms: Query<&Transform2D, Without<WireEndpoints>>,
) {
    for (endpoints, wrap_wire, mut transform, mut shape, mut visible) in &mut wires {
        let Ok(src_t) = transforms.get(endpoints.source) else {
            visible.0 = false;
            continue;
        };
        let Ok(dst_t) = transforms.get(endpoints.dest) else {
            visible.0 = false;
            continue;
        };

        let waypoints = if let Some(wrap) = wrap_wire {
            wrap.waypoints(src_t.position, dst_t.position)
        } else {
            vec![src_t.position, dst_t.position]
        };

        transform.position = Vec2::ZERO;
        transform.rotation = 0.0;
        transform.scale = Vec2::ONE;
        shape.variant = ShapeVariant::Polygon {
            points: polyline_to_ribbon(&waypoints, CABLE_HALF_THICKNESS),
        };
        shape.color = CABLE_COLOR;
        visible.0 = true;
    }
}
// EVOLVE-BLOCK-END
