// Cable geometry utilities — polygon intersection, wrapping, Catmull-Rom.

use glam::Vec2;

/// Returns the parameter `t` along segment (a1→a2) where it intersects segment (b1→b2).
/// Returns `None` if the segments don't intersect.
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

pub fn polygon_centroid(polygon: &[Vec2]) -> Vec2 {
    if polygon.is_empty() {
        return Vec2::ZERO;
    }
    let sum = polygon.iter().copied().fold(Vec2::ZERO, |acc, v| acc + v);
    sum / polygon.len() as f32
}

pub fn polygon_edges(polygon: &[Vec2]) -> impl Iterator<Item = (Vec2, Vec2)> + '_ {
    polygon
        .iter()
        .copied()
        .zip(polygon.iter().copied().cycle().skip(1))
        .take(polygon.len())
}

pub fn segment_intersects_convex_polygon(a: Vec2, b: Vec2, polygon: &[Vec2]) -> bool {
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

pub fn segment_crosses_convex_polygon(a: Vec2, b: Vec2, polygon: &[Vec2]) -> Option<f32> {
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

// ---------------------------------------------------------------------------
// Wire rendering (geometric — no simulation)
// ---------------------------------------------------------------------------

/// Maximum spacing between sample points along the wire.
const WIRE_SAMPLE_SPACING: f32 = 12.0;

/// Number of Catmull-Rom subdivisions per densified segment.
const SUBDIVISIONS_PER_SEGMENT: usize = 4;

/// Densify a polyline by inserting intermediate points so no segment exceeds
/// `max_spacing`.
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
