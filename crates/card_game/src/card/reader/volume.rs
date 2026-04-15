use crate::card::identity::signature::CardSignature;

use std::f32::consts::PI;

pub fn sphere_volume_8d(r: f32) -> f32 {
    PI.powi(4) / 24.0 * r.powi(8)
}

/// Volume of the 7D unit ball: V₇(1) = 16π³/105
fn ball_volume_7d_unit() -> f32 {
    16.0 * PI.powi(3) / 105.0
}

/// Volume of a tube of radius `r` around a curve of arc length `L` in 8D.
///
/// `V_tube = V₈(r) + L × V₇(r)` where `V₇(r) = V₇(1) × r⁷`.
pub fn tube_volume_8d(r: f32, arc_length: f32) -> f32 {
    sphere_volume_8d(r) + arc_length * ball_volume_7d_unit() * r.powi(7)
}

/// Derivative of tube volume w.r.t. radius:
/// dV/dr = 8 × π⁴/24 × r⁷ + 7 × L × V₇(1) × r⁶
fn tube_volume_derivative(r: f32, arc_length: f32) -> f32 {
    8.0 * PI.powi(4) / 24.0 * r.powi(7) + 7.0 * arc_length * ball_volume_7d_unit() * r.powi(6)
}

/// Solve for the tube radius that produces the target volume.
///
/// Uses Newton's method starting from the sphere-only radius.
/// Converges in 3-5 iterations for typical inputs.
pub fn solve_tube_radius(target_volume: f32, arc_length: f32) -> f32 {
    if target_volume <= 0.0 {
        return 0.0;
    }
    // Initial guess: radius of a sphere with this volume.
    // V₈(r) = π⁴/24 × r⁸  →  r = (V × 24/π⁴)^(1/8)
    let mut r = (target_volume * 24.0 / PI.powi(4)).powf(1.0 / 8.0);

    for _ in 0..10 {
        let f = tube_volume_8d(r, arc_length) - target_volume;
        let df = tube_volume_derivative(r, arc_length);
        if df.abs() < f32::EPSILON {
            break;
        }
        let delta = f / df;
        r -= delta;
        r = r.max(f32::EPSILON);
        if delta.abs() < 1e-8 {
            break;
        }
    }
    r
}

/// Total arc length of a polyline through the given control points.
///
/// For 0 or 1 points, returns 0.0.
/// For 2 points, returns the Euclidean distance between them.
/// For 3+ points, returns the total perimeter of the closed loop.
pub fn polyline_arc_length(points: &[CardSignature]) -> f32 {
    if points.len() <= 1 {
        return 0.0;
    }
    let mut total = 0.0_f32;
    for i in 0..points.len() {
        let next = if i + 1 < points.len() {
            i + 1
        } else if points.len() > 2 {
            0 // close the loop for 3+ points
        } else {
            break;
        };
        total += points[i].distance_to(&points[next]);
    }
    total
}
