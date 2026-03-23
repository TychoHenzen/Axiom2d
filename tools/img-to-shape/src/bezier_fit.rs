use engine_render::shape::PathCommand;
use glam::Vec2;

/// Convert a sequence of contour points into a closed path of `PathCommand`s.
///
/// Uses cubic bezier curve fitting (Schneider's algorithm) for curved segments.
/// Collinear segments fall back to `LineTo`. The `max_error` parameter controls
/// how closely the fitted curves must approximate the input points.
pub fn fit_bezier_path(points: &[(f32, f32)], max_error: f32) -> Vec<PathCommand> {
    if points.is_empty() {
        return Vec::new();
    }
    if points.len() == 1 {
        return vec![
            PathCommand::MoveTo(Vec2::new(points[0].0, points[0].1)),
            PathCommand::Close,
        ];
    }

    let mut commands = vec![PathCommand::MoveTo(Vec2::new(points[0].0, points[0].1))];

    if points.len() == 2 {
        commands.push(PathCommand::LineTo(Vec2::new(points[1].0, points[1].1)));
        commands.push(PathCommand::Close);
        return commands;
    }

    // Fit bezier curves to the point sequence.
    let vecs: Vec<Vec2> = points.iter().map(|&(x, y)| Vec2::new(x, y)).collect();
    fit_cubic_segments(&vecs, max_error, &mut commands);
    commands.push(PathCommand::Close);
    commands
}

/// Chord-length parameterization of points.
fn chord_length_parameterize(points: &[Vec2]) -> Vec<f32> {
    let mut params = vec![0.0_f32; points.len()];
    for i in 1..points.len() {
        params[i] = params[i - 1] + points[i].distance(points[i - 1]);
    }
    let total = params[points.len() - 1];
    if total > f32::EPSILON {
        for p in &mut params {
            *p /= total;
        }
    }
    params
}

/// Evaluate a cubic bezier at parameter t.
fn cubic_bezier(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let u = 1.0 - t;
    let tt = t * t;
    let uu = u * u;
    p0 * (uu * u) + p1 * (3.0 * uu * t) + p2 * (3.0 * u * tt) + p3 * (tt * t)
}

/// Compute the tangent direction from a sequence of points at the start/end.
fn estimate_left_tangent(points: &[Vec2]) -> Vec2 {
    (points[1] - points[0]).normalize_or_zero()
}

fn estimate_right_tangent(points: &[Vec2]) -> Vec2 {
    let n = points.len();
    (points[n - 2] - points[n - 1]).normalize_or_zero()
}

/// Fit a single cubic bezier to the points using least-squares.
/// Returns (control1, control2) for the bezier [start, c1, c2, end].
fn fit_single_cubic(
    points: &[Vec2],
    params: &[f32],
    tan_left: Vec2,
    tan_right: Vec2,
) -> (Vec2, Vec2) {
    let start = points[0];
    let end = points[points.len() - 1];

    if points.len() == 2 {
        let dist = start.distance(end) / 3.0;
        return (start + tan_left * dist, end + tan_right * dist);
    }

    // Compute A matrix components for least-squares fit.
    let mut c = [[0.0_f32; 2]; 2];
    let mut x = [0.0_f32; 2];

    for (i, &t) in params.iter().enumerate() {
        let u = 1.0 - t;
        let a1 = tan_left * (3.0 * u * u * t);
        let a2 = tan_right * (3.0 * u * t * t);

        c[0][0] += a1.dot(a1);
        c[0][1] += a1.dot(a2);
        c[1][0] = c[0][1];
        c[1][1] += a2.dot(a2);

        let tmp = points[i]
            - start * (u * u * u)
            - start * (3.0 * u * u * t)
            - end * (3.0 * u * t * t)
            - end * (t * t * t);

        x[0] += a1.dot(tmp);
        x[1] += a2.dot(tmp);
    }

    let det = c[0][0] * c[1][1] - c[0][1] * c[1][0];
    let (alpha_l, alpha_r) = if det.abs() < f32::EPSILON {
        let dist = start.distance(end) / 3.0;
        (dist, dist)
    } else {
        let al = (x[0] * c[1][1] - x[1] * c[0][1]) / det;
        let ar = (c[0][0] * x[1] - c[1][0] * x[0]) / det;
        // If alphas are negative, use heuristic.
        if al < 0.0 || ar < 0.0 {
            let dist = start.distance(end) / 3.0;
            (dist, dist)
        } else {
            (al, ar)
        }
    };

    (start + tan_left * alpha_l, end + tan_right * alpha_r)
}

/// Check if points are approximately collinear within tolerance.
fn is_collinear(points: &[Vec2], tolerance: f32) -> bool {
    if points.len() <= 2 {
        return true;
    }
    let start = points[0];
    let end = points[points.len() - 1];
    let line_dir = end - start;
    let line_len = line_dir.length();
    if line_len < f32::EPSILON {
        return true;
    }
    let normal = Vec2::new(-line_dir.y, line_dir.x) / line_len;
    points[1..points.len() - 1]
        .iter()
        .all(|&p| (p - start).dot(normal).abs() <= tolerance)
}

/// Recursively fit cubic bezier segments to the point sequence.
fn fit_cubic_segments(points: &[Vec2], max_error: f32, commands: &mut Vec<PathCommand>) {
    if points.len() <= 1 {
        return;
    }

    if points.len() == 2 {
        commands.push(PathCommand::LineTo(points[1]));
        return;
    }

    // Collinear points → use LineTo.
    if is_collinear(points, max_error) {
        commands.push(PathCommand::LineTo(points[points.len() - 1]));
        return;
    }

    let params = chord_length_parameterize(points);
    let tan_left = estimate_left_tangent(points);
    let tan_right = estimate_right_tangent(points);
    let (c1, c2) = fit_single_cubic(points, &params, tan_left, tan_right);

    // Find max error of the fit.
    let start = points[0];
    let end = points[points.len() - 1];
    let mut max_dist = 0.0_f32;
    let mut split_idx = points.len() / 2;
    for (i, (&pt, &t)) in points.iter().zip(params.iter()).enumerate().skip(1) {
        let fitted = cubic_bezier(start, c1, c2, end, t);
        let dist = pt.distance(fitted);
        if dist > max_dist {
            max_dist = dist;
            split_idx = i;
        }
    }

    if max_dist <= max_error {
        commands.push(PathCommand::CubicTo {
            control1: c1,
            control2: c2,
            to: end,
        });
    } else {
        // Split at the point of maximum error and recurse.
        let split = split_idx.max(1).min(points.len() - 2);
        fit_cubic_segments(&points[..=split], max_error, commands);
        fit_cubic_segments(&points[split..], max_error, commands);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::fit_bezier_path;
    use engine_render::shape::PathCommand;

    #[test]
    fn when_two_points_then_produces_moveto_lineto_close() {
        // Arrange
        let points = [(0.0, 0.0), (10.0, 5.0)];

        // Act
        let commands = fit_bezier_path(&points, 0.5);

        // Assert
        assert!(matches!(commands[0], PathCommand::MoveTo(_)));
        assert!(matches!(
            *commands.last().expect("non-empty"),
            PathCommand::Close
        ));
    }

    #[test]
    fn when_square_points_then_first_command_is_moveto() {
        // Arrange
        let points = [
            (0.0, 0.0),
            (10.0, 0.0),
            (10.0, 10.0),
            (0.0, 10.0),
            (0.0, 0.0),
        ];

        // Act
        let commands = fit_bezier_path(&points, 0.5);

        // Assert
        assert!(matches!(commands[0], PathCommand::MoveTo(_)));
    }

    #[test]
    fn when_square_points_then_last_command_is_close() {
        // Arrange
        let points = [(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)];

        // Act
        let commands = fit_bezier_path(&points, 0.5);

        // Assert
        assert!(matches!(
            *commands.last().expect("non-empty"),
            PathCommand::Close
        ));
    }

    #[test]
    fn when_fit_path_then_exactly_one_moveto() {
        // Arrange
        let points = [
            (0.0, 0.0),
            (5.0, 0.0),
            (10.0, 0.0),
            (10.0, 5.0),
            (10.0, 10.0),
            (5.0, 10.0),
            (0.0, 10.0),
            (0.0, 5.0),
        ];

        // Act
        let commands = fit_bezier_path(&points, 0.5);

        // Assert
        let moveto_count = commands
            .iter()
            .filter(|c| matches!(c, PathCommand::MoveTo(_)))
            .count();
        assert_eq!(moveto_count, 1, "expected exactly one MoveTo");
    }

    #[test]
    fn when_fit_path_then_only_legal_command_types() {
        // Arrange
        let points = [
            (0.0, 0.0),
            (5.0, 0.0),
            (10.0, 5.0),
            (5.0, 10.0),
            (0.0, 5.0),
            (0.0, 0.0),
        ];

        // Act
        let commands = fit_bezier_path(&points, 0.5);

        // Assert
        for cmd in &commands {
            assert!(
                matches!(
                    cmd,
                    PathCommand::MoveTo(_)
                        | PathCommand::LineTo(_)
                        | PathCommand::CubicTo { .. }
                        | PathCommand::Close
                ),
                "unexpected command type: {cmd:?}"
            );
        }
    }

    #[test]
    fn when_curved_points_then_produces_cubic_commands() {
        // Arrange — semicircle of points
        let n = 20;
        let points: Vec<(f32, f32)> = (0..n)
            .map(|i| {
                let t = std::f32::consts::PI * i as f32 / (n - 1) as f32;
                (t.cos() * 10.0, t.sin() * 10.0)
            })
            .collect();

        // Act
        let commands = fit_bezier_path(&points, 0.5);

        // Assert
        let cubic_count = commands
            .iter()
            .filter(|c| matches!(c, PathCommand::CubicTo { .. }))
            .count();
        assert!(
            cubic_count > 0,
            "curved input should produce CubicTo commands"
        );
    }

    #[test]
    fn when_collinear_points_then_no_cubic_commands() {
        // Arrange — straight line
        let points = [(0.0, 0.0), (5.0, 0.0), (10.0, 0.0)];

        // Act
        let commands = fit_bezier_path(&points, 0.5);

        // Assert
        let cubic_count = commands
            .iter()
            .filter(|c| matches!(c, PathCommand::CubicTo { .. }))
            .count();
        assert_eq!(
            cubic_count, 0,
            "collinear points should not produce CubicTo"
        );
    }
}
