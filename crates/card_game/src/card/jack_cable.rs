use bevy_ecs::prelude::{Component, Entity, Query, Res, Without};
use engine_core::color::Color;
use engine_core::prelude::{DeltaTime, Seconds, Transform2D};
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

const ROPE_SAMPLE_SPACING: f32 = 10.0;
const ROPE_REEL_SPEED: f32 = 48.0;
const ROPE_DAMPING: f32 = 0.92;
const ROPE_GRAVITY: f32 = 18.0;
const ROPE_OBSTACLE_PADDING: f32 = 0.35;

#[derive(Component, Debug, Clone)]
pub struct CableRope {
    pub points: Vec<Vec2>,
    pub previous_points: Vec<Vec2>,
    pub sample_spacing: f32,
    pub reel_speed: f32,
    pub damping: f32,
    pub gravity: Vec2,
}

impl CableRope {
    pub fn new(start: Vec2, end: Vec2) -> Self {
        Self {
            points: vec![start, end],
            previous_points: vec![start, end],
            sample_spacing: ROPE_SAMPLE_SPACING,
            reel_speed: ROPE_REEL_SPEED,
            damping: ROPE_DAMPING,
            gravity: Vec2::new(0.0, ROPE_GRAVITY),
        }
    }
}

fn closest_point_on_segment(point: Vec2, a: Vec2, b: Vec2) -> Vec2 {
    let ab = b - a;
    let denom = ab.length_squared();
    if denom <= 1e-12 {
        return a;
    }
    let t = ((point - a).dot(ab) / denom).clamp(0.0, 1.0);
    a + ab * t
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

pub fn project_point_outside_convex_polygon(point: Vec2, polygon: &[Vec2]) -> Vec2 {
    if polygon.len() < 3 || !point_in_convex_polygon(point, polygon) {
        return point;
    }

    let centroid = polygon_centroid(polygon);
    let mut best_point = point;
    let mut best_dist = f32::MAX;
    for i in 0..polygon.len() {
        let a = polygon[i];
        let b = polygon[(i + 1) % polygon.len()];
        let candidate = closest_point_on_segment(point, a, b);
        let dist = (candidate - point).length_squared();
        if dist < best_dist {
            best_dist = dist;
            best_point = candidate;
        }
    }

    let outward = (best_point - centroid).normalize_or_zero();
    if outward.length_squared() <= 1e-8 {
        return best_point;
    }
    best_point + outward * ROPE_OBSTACLE_PADDING
}

fn resample_polyline(waypoints: &[Vec2], spacing: f32) -> Vec<Vec2> {
    if waypoints.len() < 2 {
        return waypoints.to_vec();
    }

    let mut cumulative = Vec::with_capacity(waypoints.len());
    cumulative.push(0.0);
    for pair in waypoints.windows(2) {
        let next = cumulative.last().copied().unwrap_or(0.0) + (pair[1] - pair[0]).length();
        cumulative.push(next);
    }

    let total = *cumulative
        .last()
        .expect("waypoints has at least two points");
    if total <= 1e-6 {
        return vec![
            waypoints[0],
            *waypoints.last().expect("waypoints has at least two points"),
        ];
    }

    let sample_count = ((total / spacing).ceil() as usize).max(1) + 1;
    let mut result = Vec::with_capacity(sample_count);
    let mut seg = 0;
    for sample_idx in 0..sample_count {
        let target = total * sample_idx as f32 / (sample_count - 1) as f32;
        while seg + 1 < cumulative.len() && cumulative[seg + 1] < target {
            seg += 1;
        }
        let a = waypoints[seg];
        let b = waypoints[(seg + 1).min(waypoints.len() - 1)];
        let next_seg = (seg + 1).min(cumulative.len() - 1);
        let seg_len = (cumulative[next_seg] - cumulative[seg]).max(1e-6);
        let t = if seg + 1 < cumulative.len() {
            (target - cumulative[seg]) / seg_len
        } else {
            0.0
        };
        result.push(a.lerp(b, t.clamp(0.0, 1.0)));
    }
    result
}

pub fn rope_solve_system(
    dt: Res<DeltaTime>,
    mut wires: Query<(&WireEndpoints, Option<&WrapWire>, &mut CableRope)>,
    transforms: Query<&Transform2D, Without<WireEndpoints>>,
    colliders: Query<(&Transform2D, &CableCollider), Without<WireEndpoints>>,
) {
    let Seconds(dt_secs) = dt.0;
    let dt_secs = dt_secs.clamp(0.0, 1.0 / 30.0);
    let dt_sq = dt_secs * dt_secs;
    let obstacles: Vec<Vec<Vec2>> = colliders
        .iter()
        .map(|(transform, collider)| {
            collider
                .vertices
                .iter()
                .map(|v| *v + transform.position)
                .collect()
        })
        .collect();

    for (endpoints, wrap_wire, mut rope) in &mut wires {
        let Ok(src_t) = transforms.get(endpoints.source) else {
            continue;
        };
        let Ok(dst_t) = transforms.get(endpoints.dest) else {
            continue;
        };

        let src = src_t.position;
        let dst = dst_t.position;
        let waypoints = if let Some(wrap_wire) = wrap_wire {
            wrap_wire.waypoints(src, dst)
        } else {
            vec![src, dst]
        };
        let target_points = resample_polyline(&waypoints, rope.sample_spacing);

        if rope.points.len() != target_points.len() || rope.points.is_empty() {
            rope.points = target_points.clone();
            rope.previous_points = target_points.clone();
        }

        let last_index = rope.points.len().saturating_sub(1);
        if last_index == 0 {
            continue;
        }

        rope.points[0] = src;
        rope.points[last_index] = dst;
        rope.previous_points[0] = src;
        rope.previous_points[last_index] = dst;

        let tighten = (rope.reel_speed * dt_secs).clamp(0.0, 0.35);
        for i in 1..last_index {
            let current = rope.points[i];
            let previous = rope.previous_points[i];
            let velocity = (current - previous) * rope.damping;
            rope.previous_points[i] = current;
            let target = target_points[i];
            rope.points[i] =
                current + velocity + (target - current) * tighten + rope.gravity * dt_sq;
        }

        let iterations = 4;
        for _ in 0..iterations {
            rope.points[0] = src;
            rope.points[last_index] = dst;

            for i in 0..last_index {
                let p1 = rope.points[i];
                let p2 = rope.points[i + 1];
                let delta = p2 - p1;
                let dist = delta.length();
                if dist <= 1e-6 {
                    continue;
                }
                let desired = if i + 1 < target_points.len() {
                    (target_points[i + 1] - target_points[i])
                        .length()
                        .max(rope.sample_spacing * 0.5)
                } else {
                    rope.sample_spacing
                };
                let error = dist - desired;
                if error.abs() <= 1e-4 {
                    continue;
                }
                let correction = delta * (error / dist) * 0.5;
                if i > 0 {
                    rope.points[i] += correction;
                }
                if i + 1 < last_index {
                    rope.points[i + 1] -= correction;
                }
            }

            for point in rope.points.iter_mut().take(last_index).skip(1) {
                for polygon in &obstacles {
                    *point = project_point_outside_convex_polygon(*point, polygon);
                }
            }
        }
    }
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
        loop {
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

                    if let Some(vidx) = best_idx {
                        let v = verts[vidx];
                        let ccw_prev = verts[(vidx + n - 1) % n];
                        let ccw_next = verts[(vidx + 1) % n];
                        let centroid_sign =
                            span_dir.perp_dot(polygon_centroid(verts) - span_a).signum();
                        let prev_sign = span_dir.perp_dot(ccw_prev - span_a).signum();
                        let next_sign = span_dir.perp_dot(ccw_next - span_a).signum();
                        let prev_dist =
                            (closest_point_on_segment(hit, v, ccw_prev) - hit).length_squared();
                        let next_dist =
                            (closest_point_on_segment(hit, v, ccw_next) - hit).length_squared();
                        let boundary_step = if centroid_sign.abs() > 1e-6 {
                            if prev_sign == -centroid_sign && next_sign != -centroid_sign {
                                -1
                            } else if next_sign == -centroid_sign && prev_sign != -centroid_sign {
                                1
                            } else if prev_dist <= next_dist {
                                -1
                            } else {
                                1
                            }
                        } else if prev_dist <= next_dist {
                            -1
                        } else {
                            1
                        };
                        found = Some(WrapAnchor {
                            position: v,
                            obstacle: entity,
                            vertex_index: vidx,
                            boundary_step,
                            wrap_sign: boundary_step as f32,
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
    pub fn detect_unwraps(&mut self, src: Vec2, dst: Vec2, _obstacles: &[(Entity, &[Vec2])]) {
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
                let shortcut_clear = _obstacles
                    .iter()
                    .find(|(entity, _)| *entity == self.anchors[i].obstacle)
                    .is_none_or(|(_, verts)| !segment_intersects_convex_polygon(prev, next, verts));

                if turn_reversed || shortcut_clear {
                    self.anchors.remove(i);
                    changed = true;
                } else {
                    continue;
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
        Option<&CableRope>,
        &mut Transform2D,
        &mut Shape,
        &mut Visible,
    )>,
    transforms: Query<&Transform2D, Without<WireEndpoints>>,
) {
    for (endpoints, wrap_wire, rope, mut transform, mut shape, mut visible) in &mut wires {
        let Ok(src_t) = transforms.get(endpoints.source) else {
            visible.0 = false;
            continue;
        };
        let Ok(dst_t) = transforms.get(endpoints.dest) else {
            visible.0 = false;
            continue;
        };

        let waypoints = if let Some(rope) = rope {
            if rope.points.len() >= 2 {
                let mut points = rope.points.clone();
                if let Some(first) = points.first_mut() {
                    *first = src_t.position;
                }
                if let Some(last) = points.last_mut() {
                    *last = dst_t.position;
                }
                points
            } else if let Some(wrap) = wrap_wire {
                wrap.waypoints(src_t.position, dst_t.position)
            } else {
                vec![src_t.position, dst_t.position]
            }
        } else if let Some(wrap) = wrap_wire {
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
