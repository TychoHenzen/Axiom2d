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
        let p3 = if i + 2 < n { points[i + 2] } else { points[n - 1] };

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
    pub half_extents: Vec2,
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

    pub fn resolve_aabb_collisions(&mut self, boxes: &[(Vec2, Vec2)]) {
        let len = self.particles.len();
        // Skip first and last particles — they're pinned to endpoints
        for i in 1..len.saturating_sub(1) {
            let particle = &mut self.particles[i];
            for &(center, half) in boxes {
                let d = particle.pos - center;
                if d.x.abs() < half.x && d.y.abs() < half.y {
                    let overlap_x = half.x - d.x.abs();
                    let overlap_y = half.y - d.y.abs();
                    if overlap_x < overlap_y {
                        particle.pos.x += overlap_x * d.x.signum();
                    } else {
                        particle.pos.y += overlap_y * d.y.signum();
                    }
                }
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

        if target_n > current_n {
            self.rebuild_particles(a, b, target_n);
        } else if target_n < current_n && wrap_ratio < 1.4 {
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
                self.particles[lo].pos.lerp(self.particles[lo + 1].pos, frac)
            } else {
                a.lerp(b, t)
            };
            new_particles.push(RopeParticle { pos, prev: pos });
        }
        self.particles = new_particles;
    }
}

const ROPE_DAMPING: f32 = 0.95;
const ROPE_CONSTRAINT_ITERATIONS: usize = 8;
const ROPE_SLACK: f32 = 1.0;
const SEGMENT_LENGTH: f32 = 12.0;
pub fn rope_physics_system(
    mut ropes: Query<(&mut RopeWire, &RopeWireEndpoints)>,
    transforms: Query<&Transform2D, Without<RopeWire>>,
    colliders: Query<(&Transform2D, &CableCollider), Without<RopeWire>>,
) {
    let boxes: Vec<(Vec2, Vec2)> = colliders
        .iter()
        .map(|(t, c)| (t.position, c.half_extents))
        .collect();

    for (mut rope, endpoints) in &mut ropes {
        let Ok(src_t) = transforms.get(endpoints.source) else {
            continue;
        };
        let Ok(dst_t) = transforms.get(endpoints.dest) else {
            continue;
        };
        let src_pos = src_t.position;
        let dst_pos = dst_t.position;

        rope.resize_for_endpoints(src_pos, dst_pos);
        let rest_length = rope.rest_length;
        rope.verlet_step(ROPE_DAMPING);
        for _ in 0..ROPE_CONSTRAINT_ITERATIONS {
            rope.relax_constraints(rest_length);
            rope.resolve_aabb_collisions(&boxes);
        }
        rope.pin_endpoints(src_pos, dst_pos);
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
        let p3 = if i + 2 < n { points[i + 2] } else { points[n - 1] };

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

pub fn rope_render_system(
    mut ropes: Query<(&RopeWire, &mut Transform2D, &mut Shape)>,
) {
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
