use bevy_ecs::prelude::{Component, Entity};

use crate::card::identity::signature::CardSignature;
use crate::card::identity::signature::Element;
use crate::card::reader::volume::{polyline_arc_length, solve_tube_radius, sphere_volume_8d};

/// Default radius used as a convenience constant in tests.
pub const SIGNATURE_SPACE_RADIUS: f32 = 0.2;

/// Compute the per-card signal sphere radius from signature intensity.
///
/// Maps mean absolute intensity [0.0, 1.0] → radius [0.15, 0.25].
pub fn signature_radius(sig: &CardSignature) -> f32 {
    let mean_intensity: f32 = Element::ALL.iter().map(|&e| sig.intensity(e)).sum::<f32>() / 8.0;
    0.15 + mean_intensity * 0.10
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct SignatureSpace {
    pub control_points: Vec<CardSignature>,
    pub radius: f32,
    pub volume: f32,
    /// Entities of the cards that contributed to this signal.
    pub source_cards: Vec<Entity>,
}

impl SignatureSpace {
    /// Create a single-point sphere signal (the common case for one card).
    pub fn from_single(center: CardSignature, radius: f32, source: Entity) -> Self {
        let volume = sphere_volume_8d(radius);
        Self {
            control_points: vec![center],
            radius,
            volume,
            source_cards: vec![source],
        }
    }

    /// Combine two signals by unioning control points and recomputing the radius.
    pub fn combine(a: &Self, b: &Self) -> Self {
        let mut points = Vec::with_capacity(a.control_points.len() + b.control_points.len());
        points.extend_from_slice(&a.control_points);
        points.extend_from_slice(&b.control_points);
        points.sort();
        points.dedup_by(|x, y| x.distance_to(y) < 1e-6);

        let mut sources = Vec::with_capacity(a.source_cards.len() + b.source_cards.len());
        sources.extend_from_slice(&a.source_cards);
        for &e in &b.source_cards {
            if !sources.contains(&e) {
                sources.push(e);
            }
        }

        let volume = a.volume + b.volume;
        let arc_length = polyline_arc_length(&points);
        let radius = solve_tube_radius(volume, arc_length);

        Self {
            control_points: points,
            radius,
            volume,
            source_cards: sources,
        }
    }

    /// Check whether a point in signature space lies within this signal's volume.
    pub fn contains(&self, point: &CardSignature) -> bool {
        self.min_distance_to(point) <= self.radius
    }

    fn min_distance_to(&self, point: &CardSignature) -> f32 {
        match self.control_points.len() {
            0 => f32::INFINITY,
            1 => self.control_points[0].distance_to(point),
            _ => {
                let n = self.control_points.len();
                let mut best = f32::INFINITY;
                let segment_count = if n == 2 { 1 } else { n };
                for i in 0..segment_count {
                    let j = (i + 1) % n;
                    let d = point_to_segment_distance(
                        point,
                        &self.control_points[i],
                        &self.control_points[j],
                    );
                    best = best.min(d);
                }
                best
            }
        }
    }
}

fn point_to_segment_distance(p: &CardSignature, a: &CardSignature, b: &CardSignature) -> f32 {
    let pa = p.axes();
    let aa = a.axes();
    let ba = b.axes();

    let mut dot_ab_ab = 0.0_f32;
    let mut dot_ap_ab = 0.0_f32;
    for i in 0..8 {
        let ab_i = ba[i] - aa[i];
        let ap_i = pa[i] - aa[i];
        dot_ab_ab += ab_i * ab_i;
        dot_ap_ab += ap_i * ab_i;
    }

    if dot_ab_ab < f32::EPSILON {
        return p.distance_to(a);
    }

    let t = (dot_ap_ab / dot_ab_ab).clamp(0.0, 1.0);
    let mut dist_sq = 0.0_f32;
    for i in 0..8 {
        let closest_i = aa[i] + t * (ba[i] - aa[i]);
        let diff = pa[i] - closest_i;
        dist_sq += diff * diff;
    }
    dist_sq.sqrt()
}
