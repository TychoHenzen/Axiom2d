//! Signal geometry helpers for screen device rendering.

use engine_render::prelude::ShapeVariant;
use glam::Vec2;
use std::f32::consts::TAU;

const SIGNAL_SEGMENTS: usize = 32;
const PANEL_SPLINE_SUBDIVISIONS: usize = 8;
const PANEL_CLIP_MIN: Vec2 = Vec2::new(-50.0, -50.0);
const PANEL_CLIP_MAX: Vec2 = Vec2::new(50.0, 50.0);

pub(super) fn build_signal_polyline(points: &[Vec2], thickness: f32) -> ShapeVariant {
    let thickness = thickness.max(1.0);
    match points {
        [] => ShapeVariant::Polygon { points: Vec::new() },
        [point] => clipped_signal_circle(*point, thickness),
        [a, b] => {
            let dir = (*b - *a).normalize_or_zero();
            let normal = Vec2::new(-dir.y, dir.x);
            let half_steps = SIGNAL_SEGMENTS / 2;
            let polygon = [*a + normal * thickness, *b + normal * thickness]
                .into_iter()
                .chain(semicircle_fan(*b, thickness, dir, half_steps))
                .chain([*b - normal * thickness, *a - normal * thickness])
                .chain(semicircle_fan(*a, thickness, -dir, half_steps))
                .collect::<Vec<_>>();
            ShapeVariant::Polygon {
                points: clip_to_panel(polygon),
            }
        }
        _ => {
            let dense = catmull_rom_subdivide_closed(points, PANEL_SPLINE_SUBDIVISIONS);
            let m = dense.len();
            let mut inner = Vec::with_capacity(m);
            let mut polygon = Vec::with_capacity(2 * m + 2);
            for i in 0..m {
                let prev = dense[(i + m - 1) % m];
                let next = dense[(i + 1) % m];
                let dir = (next - prev).normalize_or_zero();
                let normal = Vec2::new(-dir.y, dir.x);
                polygon.push(dense[i] + normal * thickness);
                inner.push(dense[i] - normal * thickness);
            }
            polygon.push(polygon[0]);
            polygon.push(inner[0]);
            for i in (0..m).rev() {
                let prev = dense[(i + m - 1) % m];
                let next = dense[(i + 1) % m];
                let dir = (next - prev).normalize_or_zero();
                let normal = Vec2::new(-dir.y, dir.x);
                polygon.push(dense[i] - normal * thickness);
            }
            ShapeVariant::Polygon {
                points: clip_to_panel(polygon),
            }
        }
    }
}

/// Sweeps the interior semicircle samples for a capsule cap around `center`.
/// The caller adds the two endpoints, so this only yields the intermediate arc points.
pub(super) fn semicircle_fan(
    center: Vec2,
    radius: f32,
    axis: Vec2,
    half_steps: usize,
) -> impl Iterator<Item = Vec2> {
    let base_angle = axis.y.atan2(axis.x);
    (1..half_steps).map(move |step| {
        // Sweep from -π/2 to +π/2 relative to axis (i.e., the outward hemisphere).
        let t = step as f32 / half_steps as f32;
        let angle = base_angle + std::f32::consts::PI * (0.5 - t);
        center + Vec2::new(radius * angle.cos(), radius * angle.sin())
    })
}

/// Catmull-Rom subdivision for a closed loop of control points.
///
/// Wraps indices so the spline closes smoothly from the last point back to the first.
/// Produces `n * subdivisions` output points (no duplicate at the seam).
pub(super) fn catmull_rom_subdivide_closed(points: &[Vec2], subdivisions: usize) -> Vec<Vec2> {
    let n = points.len();
    if n < 3 {
        return points.to_vec();
    }
    let mut result = Vec::with_capacity(n * subdivisions);
    for i in 0..n {
        let p0 = points[(i + n - 1) % n];
        let p1 = points[i];
        let p2 = points[(i + 1) % n];
        let p3 = points[(i + 2) % n];

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
    result
}

pub(super) fn clipped_signal_circle(center: Vec2, radius: f32) -> ShapeVariant {
    let points = circle_polygon(center, radius, SIGNAL_SEGMENTS);
    ShapeVariant::Polygon {
        points: if circle_fits_rect(center, radius, PANEL_CLIP_MIN, PANEL_CLIP_MAX) {
            points
        } else {
            clip_to_panel(points)
        },
    }
}

pub(super) fn clip_to_panel(points: Vec<Vec2>) -> Vec<Vec2> {
    clip_polygon_to_rect(points, PANEL_CLIP_MIN, PANEL_CLIP_MAX)
}

pub(super) fn circle_fits_rect(center: Vec2, radius: f32, min: Vec2, max: Vec2) -> bool {
    center.x - radius >= min.x
        && center.x + radius <= max.x
        && center.y - radius >= min.y
        && center.y + radius <= max.y
}

pub(super) fn circle_polygon(center: Vec2, radius: f32, segments: usize) -> Vec<Vec2> {
    let mut points = Vec::with_capacity(segments);
    for index in 0..segments {
        let angle = TAU * index as f32 / segments as f32;
        points.push(center + Vec2::new(radius * angle.cos(), radius * angle.sin()));
    }
    points
}

pub(super) fn clip_polygon_to_rect(points: Vec<Vec2>, min: Vec2, max: Vec2) -> Vec<Vec2> {
    let left = clip_polygon(points, |p| p.x >= min.x, |a, b| intersect_x(a, b, min.x));
    let right = clip_polygon(left, |p| p.x <= max.x, |a, b| intersect_x(a, b, max.x));
    let bottom = clip_polygon(right, |p| p.y >= min.y, |a, b| intersect_y(a, b, min.y));
    clip_polygon(bottom, |p| p.y <= max.y, |a, b| intersect_y(a, b, max.y))
}

fn clip_polygon<F, G>(points: Vec<Vec2>, is_inside: F, intersect: G) -> Vec<Vec2>
where
    F: Fn(Vec2) -> bool,
    G: Fn(Vec2, Vec2) -> Vec2,
{
    let Some(mut previous) = points.last().copied() else {
        return points;
    };
    let mut result = Vec::with_capacity(points.len().saturating_mul(2));
    let mut previous_inside = is_inside(previous);

    for current in points {
        let current_inside = is_inside(current);
        match (previous_inside, current_inside) {
            (true, true) => result.push(current),
            (true, false) => result.push(intersect(previous, current)),
            (false, true) => {
                result.push(intersect(previous, current));
                result.push(current);
            }
            (false, false) => {}
        }
        previous = current;
        previous_inside = current_inside;
    }

    result
}

pub(super) fn intersect_x(a: Vec2, b: Vec2, x: f32) -> Vec2 {
    let delta = b.x - a.x;
    if delta.abs() <= f32::EPSILON {
        return Vec2::new(x, a.y);
    }
    let t = (x - a.x) / delta;
    a + (b - a) * t
}

pub(super) fn intersect_y(a: Vec2, b: Vec2, y: f32) -> Vec2 {
    let delta = b.y - a.y;
    if delta.abs() <= f32::EPSILON {
        return Vec2::new(a.x, y);
    }
    let t = (y - a.y) / delta;
    a + (b - a) * t
}
