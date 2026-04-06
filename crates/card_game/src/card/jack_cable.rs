use bevy_ecs::prelude::{Component, Entity, Query, Res, Without};
use engine_core::color::Color;
use engine_core::prelude::Transform2D;
use engine_core::time::DeltaTime;
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

pub(crate) fn cable_visuals(a: Vec2, b: Vec2) -> (Transform2D, Shape) {
    let midpoint = (a + b) * 0.5;
    let half_delta = (b - a) * 0.5;
    let dir = (b - a).normalize_or_zero();
    let perp = Vec2::new(-dir.y, dir.x) * CABLE_HALF_THICKNESS;

    (
        Transform2D {
            position: midpoint,
            rotation: 0.0,
            scale: Vec2::ONE,
        },
        Shape {
            variant: ShapeVariant::Polygon {
                points: vec![
                    -half_delta - perp,
                    -half_delta + perp,
                    half_delta + perp,
                    half_delta - perp,
                ],
            },
            color: CABLE_COLOR,
        },
    )
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

pub fn cable_render_system(
    mut cables: Query<(&Cable, &mut Transform2D, &mut Shape, &mut Visible)>,
    transforms: Query<&Transform2D, Without<Cable>>,
) {
    for (cable, mut transform, mut shape, mut visible) in &mut cables {
        let Ok(src_t) = transforms.get(cable.source) else {
            visible.0 = false;
            continue;
        };
        let Ok(dst_t) = transforms.get(cable.dest) else {
            visible.0 = false;
            continue;
        };

        let (next_transform, next_shape) = cable_visuals(src_t.position, dst_t.position);
        *transform = next_transform;
        *shape = next_shape;
        visible.0 = true;
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

#[derive(Component, Debug, Clone)]
pub struct CableCollider {
    /// Convex hull vertices in local space, wound counter-clockwise.
    pub vertices: Vec<Vec2>,
}

#[derive(Debug, Clone)]
pub struct WrapAnchor {
    /// World-space position of this anchor (polygon vertex).
    pub position: Vec2,
    /// Which obstacle entity this anchor belongs to.
    pub obstacle: Entity,
    /// Index into that obstacle's CableCollider.vertices.
    pub vertex_index: usize,
    /// Wrap direction: +1.0 for CCW wrap, -1.0 for CW wrap.
    pub wrap_sign: f32,
    /// Index of the pinned particle in the `RopeWire` chain.
    pub pinned_particle: usize,
}

#[derive(Component, Debug, Clone)]
pub struct WrapWire {
    /// Ordered anchor points from source toward dest.
    pub anchors: Vec<WrapAnchor>,
    /// Target length the cable is retracting toward.
    pub target_length: f32,
}

impl Default for WrapWire {
    fn default() -> Self {
        Self {
            anchors: vec![],
            target_length: 0.0,
        }
    }
}

impl WrapWire {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check each span for polygon intersections and insert wrap anchors.
    pub fn detect_wraps(&mut self, src: Vec2, dst: Vec2, obstacles: &[(Entity, &[Vec2])]) {
        // Build list of pin points: src, anchor positions, dst
        let mut pins: Vec<Vec2> = Vec::with_capacity(self.anchors.len() + 2);
        pins.push(src);
        for anchor in &self.anchors {
            pins.push(anchor.position);
        }
        pins.push(dst);

        // Walk spans and check for intersections
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
                // Check whether the span *enters* a polygon edge at a point strictly
                // interior to the span (t > ε).  Intersections at t≈0 mean the span
                // starts exactly at the polygon boundary (i.e. the start point is an
                // already-pinned anchor on this obstacle) and must not trigger another wrap.
                let has_interior_intersection = (0..n).any(|j| {
                    segment_intersects_segment(span_a, span_b, verts[j], verts[(j + 1) % n])
                        .map_or(false, |t| t > 1e-6)
                });
                if !has_interior_intersection {
                    continue 'obstacle;
                }

                // Pick the best vertex that (a) creates the shortest detour and
                // (b) is not already an anchor on this obstacle.  Excluding already-anchored
                // vertices lets the cable add a second anchor at a different corner of the
                // same box — necessary for cables that wrap around more than one corner.
                let span_dir = span_b - span_a;
                let mut best_idx: Option<usize> = None;
                let mut best_detour = f32::MAX;
                for (j, v) in verts.iter().enumerate() {
                    let already = self
                        .anchors
                        .iter()
                        .any(|a| a.obstacle == entity && a.vertex_index == j);
                    if already {
                        continue;
                    }
                    let detour = (*v - span_a).length() + (span_b - *v).length();
                    if detour < best_detour {
                        best_detour = detour;
                        best_idx = Some(j);
                    }
                }

                if let Some(vidx) = best_idx {
                    let v = verts[vidx];
                    let wrap_sign = span_dir.perp_dot(v - span_a).signum();
                    found = Some(WrapAnchor {
                        position: v,
                        obstacle: entity,
                        vertex_index: vidx,
                        wrap_sign,
                        // pinned_particle is 0 as a placeholder; wrap_detect_system
                        // updates it to the closest interior particle index after insertion.
                        pinned_particle: 0,
                    });
                    break 'obstacle;
                }
            }

            if let Some(anchor) = found {
                self.anchors.insert(insert_idx, anchor);
                // Stop after inserting one anchor per call. The next frame will
                // discover additional corners if the cable has actually moved
                // around them. Without this limit, a single span crossing a box
                // cascades: V1 is inserted, the new span V1→B still crosses the
                // same box, so V2 is added immediately, then V3, etc.
                return;
            } else {
                insert_idx += 1;
                i += 1;
            }
        }
    }

    /// Remove anchors where the cable has swung past the wrap point.
    pub fn detect_unwraps(&mut self, src: Vec2, dst: Vec2) {
        let mut i = 0;
        while i < self.anchors.len() {
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

            if cross * self.anchors[i].wrap_sign <= 0.0 {
                self.anchors.remove(i);
            } else {
                i += 1;
            }
        }
    }

    /// Proportionally retract `target_length` toward the shortest geometric path.
    pub fn retract(&mut self, src: Vec2, dst: Vec2, rate: f32, dt: f32) {
        let shortest = self.shortest_path(src, dst);
        let slack_factor = 1.05;
        let floor = shortest * slack_factor;

        if self.target_length > floor {
            self.target_length -= (self.target_length - floor) * rate * dt;
            if self.target_length < floor {
                self.target_length = floor;
            }
        }

        if self.target_length < shortest {
            self.target_length = shortest;
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

#[derive(Debug, Clone, PartialEq)]
pub struct RopeParticle {
    pub pos: Vec2,
    pub prev: Vec2,
}

#[derive(Component, Debug, Clone)]
pub struct RopeWire {
    pub particles: Vec<RopeParticle>,
    pub segments: Vec<Entity>,
    rest_length: f32,
}

#[derive(Component, Debug, Clone)]
pub struct RopeWireEndpoints {
    pub source: Entity,
    pub dest: Entity,
}

impl RopeWire {
    pub fn rest_length(&self) -> f32 {
        self.rest_length
    }

    pub fn with_particles(particles: Vec<RopeParticle>) -> Self {
        Self {
            particles,
            segments: vec![],
            rest_length: 0.0,
        }
    }

    pub fn for_distance(a: Vec2, b: Vec2) -> Self {
        let dist = (b - a).length();
        let n = ((dist / SEGMENT_LENGTH).ceil() as usize + 1).max(2);
        Self::new(a, b, n)
    }

    pub fn new(a: Vec2, b: Vec2, n: usize) -> Self {
        let rest_length = if n > 1 {
            (b - a).length() * ROPE_SLACK / (n - 1) as f32
        } else {
            0.0
        };
        let mut particles = Vec::with_capacity(n);
        for i in 0..n {
            let t = i as f32 / (n - 1) as f32;
            let pos = a.lerp(b, t);
            particles.push(RopeParticle { pos, prev: pos });
        }
        Self {
            particles,
            segments: vec![],
            rest_length,
        }
    }

    pub fn verlet_step(&mut self, damping: f32) {
        for particle in &mut self.particles {
            let new_pos = particle.pos + (particle.pos - particle.prev) * damping;
            particle.prev = particle.pos;
            particle.pos = new_pos;
        }
    }

    pub fn apply_shrinkage(&mut self, strength: f32) {
        let n = self.particles.len();
        if n < 3 {
            return;
        }
        let start = self.particles[0].pos;
        let end = self.particles[n - 1].pos;
        for i in 1..n - 1 {
            let t = i as f32 / (n - 1) as f32;
            let target = start.lerp(end, t);
            let current = self.particles[i].pos;
            self.particles[i].pos += (target - current) * strength;
        }
    }

    pub fn relax_constraints(&mut self, rest_length: f32) {
        for i in 0..self.particles.len().saturating_sub(1) {
            let delta = self.particles[i + 1].pos - self.particles[i].pos;
            let dist = delta.length();
            if dist < 1e-8 {
                continue;
            }
            let correction = delta * ((dist - rest_length) / dist) * 0.5;
            self.particles[i].pos += correction;
            self.particles[i + 1].pos -= correction;
        }
    }

    pub fn pin_endpoints(&mut self, a: Vec2, b: Vec2) {
        if let Some(first) = self.particles.first_mut() {
            first.pos = a;
            first.prev = a;
        }
        if let Some(last) = self.particles.last_mut() {
            last.pos = b;
            last.prev = b;
        }
    }

    /// Push interior particles out of convex polygons.
    /// Each entry in `polygons` is `(center, &world_verts)`.
    pub fn resolve_polygon_collisions(&mut self, polygons: &[(Vec2, &[Vec2])]) {
        let len = self.particles.len();
        for i in 1..len.saturating_sub(1) {
            let p = self.particles[i].pos;
            for &(_, verts) in polygons {
                let n = verts.len();
                if n < 3 {
                    continue;
                }
                // Check if point is inside convex polygon using cross products
                let mut inside = true;
                for j in 0..n {
                    let a = verts[j];
                    let b = verts[(j + 1) % n];
                    let cross = (b - a).perp_dot(p - a);
                    if cross < 0.0 {
                        inside = false;
                        break;
                    }
                }
                if !inside {
                    continue;
                }
                // Find closest edge and push out along its normal
                let mut min_dist = f32::MAX;
                let mut push = Vec2::ZERO;
                for j in 0..n {
                    let a = verts[j];
                    let b = verts[(j + 1) % n];
                    let edge = b - a;
                    let edge_len_sq = edge.length_squared();
                    if edge_len_sq < 1e-10 {
                        continue;
                    }
                    let t = ((p - a).dot(edge) / edge_len_sq).clamp(0.0, 1.0);
                    let closest = a + edge * t;
                    let dist = (p - closest).length();
                    if dist < min_dist {
                        min_dist = dist;
                        let normal = Vec2::new(edge.y, -edge.x).normalize_or_zero();
                        push = normal * (min_dist + 0.1);
                    }
                }
                self.particles[i].pos = p + push;
            }
        }
    }

    fn path_length(&self) -> f32 {
        self.particles
            .windows(2)
            .map(|w| (w[1].pos - w[0].pos).length())
            .sum()
    }

    /// Rebuild particles when the distance between endpoints changes enough
    /// to warrant a different segment count. Only shrinks when the actual
    /// cable path is close to the straight-line distance — if the cable is
    /// wrapped around an obstacle, the path is much longer and shrinking
    /// would phase the cable through the obstacle.
    pub fn resize_for_endpoints(&mut self, a: Vec2, b: Vec2) {
        let straight_dist = (b - a).length();
        let target_n = ((straight_dist / SEGMENT_LENGTH).ceil() as usize + 1).max(2);
        let current_n = self.particles.len();
        let path_len = self.path_length();
        let wrap_ratio = if straight_dist > 1e-4 {
            path_len / straight_dist
        } else {
            1.0
        };

        if target_n > current_n || (target_n < current_n && wrap_ratio < 1.4) {
            self.rebuild_particles(a, b, target_n);
        }
        // else: same count or wrapped — keep current particles

        // Rest length: use path length when wrapped so constraints don't
        // compress the cable through obstacles
        let n = self.particles.len();
        if n > 1 {
            if wrap_ratio > 1.2 {
                self.rest_length = path_len / (n - 1) as f32;
            } else {
                self.rest_length = straight_dist / (n - 1) as f32;
            }
        } else {
            self.rest_length = 0.0;
        }
    }

    fn rebuild_particles(&mut self, a: Vec2, b: Vec2, target_n: usize) {
        let straight_dist = (b - a).length();
        self.rest_length = if target_n > 1 {
            straight_dist / (target_n - 1) as f32
        } else {
            0.0
        };

        let current_n = self.particles.len();
        let mut new_particles = Vec::with_capacity(target_n);
        for i in 0..target_n {
            let t = i as f32 / (target_n - 1) as f32;
            let pos = if current_n >= 2 {
                let float_idx = t * (current_n - 1) as f32;
                let lo = (float_idx as usize).min(current_n - 2);
                let frac = float_idx - lo as f32;
                self.particles[lo]
                    .pos
                    .lerp(self.particles[lo + 1].pos, frac)
            } else {
                a.lerp(b, t)
            };
            new_particles.push(RopeParticle { pos, prev: pos });
        }
        self.particles = new_particles;
    }
}

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

    // Check if the span intersects any edge of the polygon
    let has_intersection = (0..n).any(|i| {
        let e1 = polygon[i];
        let e2 = polygon[(i + 1) % n];
        segment_intersects_segment(a, b, e1, e2).is_some()
    });
    if !has_intersection {
        return None;
    }

    // Find the vertex that creates the shortest detour
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

const RETRACTION_RATE: f32 = 3.0;

pub fn retraction_system(
    mut wires: Query<(&mut WrapWire, &RopeWireEndpoints)>,
    transforms: Query<&Transform2D, Without<WrapWire>>,
    dt: Res<DeltaTime>,
) {
    for (mut wrap, endpoints) in &mut wires {
        let Ok(src_t) = transforms.get(endpoints.source) else {
            continue;
        };
        let Ok(dst_t) = transforms.get(endpoints.dest) else {
            continue;
        };
        wrap.retract(src_t.position, dst_t.position, RETRACTION_RATE, dt.0.0);
    }
}

const ROPE_DAMPING: f32 = 0.95;
const ROPE_CONSTRAINT_ITERATIONS: usize = 8;
const ROPE_SLACK: f32 = 1.0;
const SEGMENT_LENGTH: f32 = 12.0;
pub fn rope_physics_system(
    mut ropes: Query<(&mut RopeWire, &RopeWireEndpoints, Option<&WrapWire>)>,
    transforms: Query<&Transform2D, Without<RopeWire>>,
    colliders: Query<(&Transform2D, &CableCollider), Without<RopeWire>>,
) {
    let polygons: Vec<(Vec2, Vec<Vec2>)> = colliders
        .iter()
        .map(|(t, c)| {
            let world_verts: Vec<Vec2> = c.vertices.iter().map(|v| *v + t.position).collect();
            (t.position, world_verts)
        })
        .collect();

    for (mut rope, endpoints, wrap_wire) in &mut ropes {
        let Ok(src_t) = transforms.get(endpoints.source) else {
            continue;
        };
        let Ok(dst_t) = transforms.get(endpoints.dest) else {
            continue;
        };
        let src_pos = src_t.position;
        let dst_pos = dst_t.position;

        rope.resize_for_endpoints(src_pos, dst_pos);

        let rest_length = if let Some(wrap) = wrap_wire {
            if wrap.target_length > 0.0 && rope.particles.len() > 1 {
                wrap.target_length / (rope.particles.len() - 1) as f32
            } else {
                rope.rest_length()
            }
        } else {
            rope.rest_length()
        };

        rope.verlet_step(ROPE_DAMPING);
        for _ in 0..ROPE_CONSTRAINT_ITERATIONS {
            rope.relax_constraints(rest_length);

            // Re-pin anchored particles after each constraint iteration
            if let Some(wrap) = wrap_wire {
                for anchor in &wrap.anchors {
                    let idx = anchor.pinned_particle;
                    if idx > 0 && idx < rope.particles.len() - 1 {
                        rope.particles[idx].pos = anchor.position;
                        rope.particles[idx].prev = anchor.position;
                    }
                }
            }

            let poly_refs: Vec<(Vec2, &[Vec2])> =
                polygons.iter().map(|(c, v)| (*c, v.as_slice())).collect();
            rope.resolve_polygon_collisions(&poly_refs);
        }
        rope.pin_endpoints(src_pos, dst_pos);

        if let Some(wrap) = wrap_wire {
            for anchor in &wrap.anchors {
                let idx = anchor.pinned_particle;
                if idx > 0 && idx < rope.particles.len() - 1 {
                    rope.particles[idx].pos = anchor.position;
                    rope.particles[idx].prev = anchor.position;
                }
            }
        }
    }
}

const SUBDIVISIONS_PER_SEGMENT: usize = 4;

/// Evaluate a Catmull-Rom spline through `points` at evenly spaced parameter values,
/// producing `(points.len() - 1) * subdivisions + 1` output samples.
pub fn catmull_rom_subdivide(points: &[Vec2], subdivisions: usize) -> Vec<Vec2> {
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
            // Catmull-Rom matrix evaluation
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

/// Build a ribbon polygon from particle positions. First offsets each particle
/// perpendicular to the local tangent, then subdivides each edge with Catmull-Rom
/// interpolation for smooth curves. Returns vertices going forward along one edge,
/// then backward along the other.
pub fn particles_to_ribbon(positions: &[Vec2], half_thickness: f32) -> Vec<Vec2> {
    let n = positions.len();
    if n < 2 {
        return vec![];
    }
    let mut left_control = Vec::with_capacity(n);
    let mut right_control = Vec::with_capacity(n);
    for i in 0..n {
        let tangent = if i == 0 {
            positions[1] - positions[0]
        } else if i == n - 1 {
            positions[n - 1] - positions[n - 2]
        } else {
            positions[i + 1] - positions[i - 1]
        };
        let perp = Vec2::new(-tangent.y, tangent.x).normalize_or_zero() * half_thickness;
        left_control.push(positions[i] + perp);
        right_control.push(positions[i] - perp);
    }
    // Subdivide each edge into smooth curves
    let mut left = catmull_rom_subdivide(&left_control, SUBDIVISIONS_PER_SEGMENT);
    let mut right = catmull_rom_subdivide(&right_control, SUBDIVISIONS_PER_SEGMENT);
    right.reverse();
    left.extend(right);
    left
}

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
    mut wires: Query<(&mut WrapWire, &RopeWireEndpoints, Option<&RopeWire>)>,
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

    for (mut wrap, endpoints, rope) in &mut wires {
        let Ok(src_t) = transforms.get(endpoints.source) else {
            continue;
        };
        let Ok(dst_t) = transforms.get(endpoints.dest) else {
            continue;
        };
        let src = src_t.position;
        let dst = dst_t.position;

        // Unwrap first, then wrap — order matters
        wrap.detect_unwraps(src, dst);

        let obstacle_refs: Vec<(Entity, &[Vec2])> =
            obstacles.iter().map(|(e, v)| (*e, v.as_slice())).collect();
        wrap.detect_wraps(src, dst, &obstacle_refs);

        // Assign pinned_particle for any anchor whose index is still 0 (the placeholder set
        // by detect_wraps).  We pick the interior particle closest to the anchor's world
        // position so that rope_physics_system can actually lock a particle onto the vertex.
        // Without this, every new anchor would reference particle[0] (the source endpoint),
        // which rope_physics_system deliberately skips, leaving no particle pinned.
        if let Some(rope) = rope {
            let n = rope.particles.len();
            if n > 2 {
                for anchor in wrap.anchors.iter_mut() {
                    if anchor.pinned_particle == 0 {
                        // Search interior particles only (skip endpoints 0 and n-1)
                        let best = rope
                            .particles
                            .iter()
                            .enumerate()
                            .skip(1)
                            .take(n - 2)
                            .min_by(|(_, a), (_, b)| {
                                a.pos
                                    .distance_squared(anchor.position)
                                    .partial_cmp(&b.pos.distance_squared(anchor.position))
                                    .unwrap_or(std::cmp::Ordering::Equal)
                            });
                        if let Some((idx, _)) = best {
                            anchor.pinned_particle = idx;
                        }
                    }
                }
            }
        }
    }
}

pub fn rope_render_system(mut ropes: Query<(&RopeWire, &mut Transform2D, &mut Shape)>) {
    for (rope, mut transform, mut shape) in &mut ropes {
        transform.position = Vec2::ZERO;
        transform.rotation = 0.0;
        transform.scale = Vec2::ONE;

        let positions: Vec<Vec2> = rope.particles.iter().map(|p| p.pos).collect();
        let ribbon = particles_to_ribbon(&positions, CABLE_HALF_THICKNESS);
        shape.variant = ShapeVariant::Polygon { points: ribbon };
        shape.color = CABLE_COLOR;
    }
}
