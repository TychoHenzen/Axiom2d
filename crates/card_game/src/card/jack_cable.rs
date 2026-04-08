use bevy_ecs::prelude::{Component, Entity, Query, Without};
use engine_core::color::Color;
use engine_core::prelude::Transform2D;
use engine_render::prelude::{Shape, ShapeVariant};
use engine_render::shape::PathCommand;
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

fn segment_intersects_convex_polygon(a: Vec2, b: Vec2, polygon: &[Vec2]) -> bool {
    if polygon.len() < 3 {
        return false;
    }
    if point_in_convex_polygon(a, polygon) || point_in_convex_polygon(b, polygon) {
        return true;
    }
    polygon.iter().enumerate().any(|(i, &p1)| {
        let p2 = polygon[(i + 1) % polygon.len()];
        segment_intersects_segment(a, b, p1, p2).is_some_and(|t| t > 1e-4 && t < 1.0 - 1e-4)
    })
}

fn segment_crosses_convex_polygon(a: Vec2, b: Vec2, polygon: &[Vec2]) -> Option<f32> {
    if polygon.len() < 3 {
        return None;
    }

    let endpoint_on_vertex = |p: Vec2| polygon.iter().any(|v| (p - *v).length_squared() <= 1e-6);
    let mut first_t = f32::MAX;
    let mut hit_count = 0;
    for i in 0..polygon.len() {
        let p1 = polygon[i];
        let p2 = polygon[(i + 1) % polygon.len()];
        if let Some(t) = segment_intersects_segment(a, b, p1, p2)
            && t > 1e-4
            && t < 1.0 - 1e-4
        {
            hit_count += 1;
            first_t = first_t.min(t);
        }
    }

    if hit_count >= 2 || (hit_count == 1 && (endpoint_on_vertex(a) || endpoint_on_vertex(b))) {
        Some(first_t)
    } else {
        None
    }
}

pub fn point_in_convex_polygon(point: Vec2, polygon: &[Vec2]) -> bool {
    if polygon.len() < 3 {
        return false;
    }

    let mut sign = 0.0;
    for i in 0..polygon.len() {
        let a = polygon[i];
        let b = polygon[(i + 1) % polygon.len()];
        let cross = (b - a).perp_dot(point - a);
        if cross.abs() <= 1e-5 {
            continue;
        }
        if sign == 0.0 {
            sign = cross.signum();
        } else if sign * cross < 0.0 {
            return false;
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

/// Convert particle positions to a smooth cubic bezier path using Catmull-Rom interpolation.
/// Produces one `MoveTo` + (N-1) `CubicTo` commands for N input points.
pub fn particles_to_bezier_path(points: &[Vec2]) -> Vec<PathCommand> {
    if points.len() < 2 {
        return vec![];
    }
    let mut commands = vec![PathCommand::MoveTo(points[0])];
    let n = points.len();
    for i in 0..n - 1 {
        let p0 = if i == 0 { points[0] } else { points[i - 1] };
        let p1 = points[i];
        let p2 = points[i + 1];
        let p3 = if i + 2 < n {
            points[i + 2]
        } else {
            points[n - 1]
        };

        let control1 = p1 + (p2 - p0) / 6.0;
        let control2 = p2 - (p3 - p1) / 6.0;

        commands.push(PathCommand::CubicTo {
            control1,
            control2,
            to: p2,
        });
    }
    commands
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
    /// Construct from AABB half-extents (backward compat for readers/screens).
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
        let vertex_count = vertex_count.max(1);
        match boundary_step.cmp(&0) {
            std::cmp::Ordering::Greater => (vertex_index + 1) % vertex_count,
            std::cmp::Ordering::Less => (vertex_index + vertex_count - 1) % vertex_count,
            std::cmp::Ordering::Equal => (vertex_index + 1) % vertex_count,
        }
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
                    let hit = span_a + (span_b - span_a) * first_t;
                    let last_anchor = self.anchors.iter().rev().find(|a| a.obstacle == entity);

                    let mut best_idx: Option<usize> = None;
                    let mut best_dist = f32::MAX;
                    if let Some(last_anchor) = last_anchor
                        && last_anchor.obstacle == entity
                        && last_anchor.vertex_index < n
                    {
                        let candidate_idx = Self::boundary_neighbor_index(
                            last_anchor.vertex_index,
                            last_anchor.boundary_step,
                            n,
                        );
                        best_idx = Some(candidate_idx);
                    } else {
                        for (j, v) in verts.iter().enumerate() {
                            let dist = (*v - hit).length_squared();
                            if dist < best_dist {
                                best_dist = dist;
                                best_idx = Some(j);
                            }
                        }
                    }

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
                        } else if let Some(last) = last_anchor {
                            last.wrap_sign
                        } else {
                            // Vertex on cable line, no prior anchor — wrap away
                            // from polygon interior.
                            let c = span_dir.perp_dot(polygon_centroid(verts) - span_a);
                            if c.abs() > 1e-6 { -c.signum() } else { 1.0 }
                        };
                        // boundary_step: consistent walk direction for this obstacle.
                        // Inherit from existing anchor; first anchor derives from wrap_sign.
                        let boundary_step = if let Some(last) = last_anchor {
                            last.boundary_step
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
        if self.anchors.is_empty() {
            return (dst - src).length();
        }
        let mut total = (self.anchors[0].position - src).length();
        for i in 0..self.anchors.len() - 1 {
            total += (self.anchors[i + 1].position - self.anchors[i].position).length();
        }
        total += (dst - self.anchors.last().expect("checked non-empty").position).length();
        total
    }

    /// Return the full waypoint list: `[src, anchor1, ..., anchorN, dst]`.
    pub fn waypoints(&self, src: Vec2, dst: Vec2) -> Vec<Vec2> {
        let mut pts = Vec::with_capacity(self.anchors.len() + 2);
        pts.push(src);
        for anchor in &self.anchors {
            pts.push(anchor.position);
        }
        pts.push(dst);
        pts
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

/// Given a cable span (a→b) that crosses a convex polygon, find the vertex to wrap around.
/// Returns `(vertex_index, wrap_sign)` where `wrap_sign` is +1.0 (CCW) or -1.0 (CW).
/// Returns `None` if the span doesn't cross the polygon.
pub fn find_wrap_vertex(a: Vec2, b: Vec2, polygon: &[Vec2]) -> Option<(usize, f32)> {
    let n = polygon.len();
    if n < 3 {
        return None;
    }

    let has_intersection = (0..n).any(|i| {
        let e1 = polygon[i];
        let e2 = polygon[(i + 1) % n];
        segment_intersects_segment(a, b, e1, e2).is_some()
    });
    if !has_intersection {
        return None;
    }

    let span_dir = b - a;
    let mut best_idx = 0;
    let mut best_detour = f32::MAX;
    for (i, v) in polygon.iter().enumerate() {
        let detour = (*v - a).length() + (b - *v).length();
        if detour < best_detour {
            best_detour = detour;
            best_idx = i;
        }
    }

    let wrap_sign = span_dir.perp_dot(polygon[best_idx] - a).signum();
    Some((best_idx, wrap_sign))
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

    for (mut wrap, endpoints) in &mut wires {
        let Ok(src_t) = transforms.get(endpoints.source) else {
            continue;
        };
        let Ok(dst_t) = transforms.get(endpoints.dest) else {
            continue;
        };
        let src = src_t.position;
        let dst = dst_t.position;

        let obstacle_refs: Vec<(Entity, &[Vec2])> =
            obstacles.iter().map(|(e, v)| (*e, v.as_slice())).collect();

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
