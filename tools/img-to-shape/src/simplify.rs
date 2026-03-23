/// Perpendicular distance from point `p` to the line segment `start`–`end`.
fn perpendicular_distance(p: (f32, f32), start: (f32, f32), end: (f32, f32)) -> f32 {
    let dx = end.0 - start.0;
    let dy = end.1 - start.1;
    let len_sq = dx * dx + dy * dy;
    if len_sq < f32::EPSILON {
        // Degenerate segment: distance to the point itself.
        let px = p.0 - start.0;
        let py = p.1 - start.1;
        return (px * px + py * py).sqrt();
    }
    ((dy * p.0 - dx * p.1 + end.0 * start.1 - end.1 * start.0).abs()) / len_sq.sqrt()
}

/// Simplify a point sequence using the Ramer-Douglas-Peucker algorithm.
/// Endpoints are always preserved.
pub fn rdp_simplify(points: &[(f32, f32)], epsilon: f32) -> Vec<(f32, f32)> {
    if points.len() <= 2 {
        return points.to_vec();
    }

    let start = points[0];
    let end = points[points.len() - 1];

    let mut max_dist = 0.0_f32;
    let mut max_idx = 0;
    for (i, &p) in points.iter().enumerate().skip(1).take(points.len() - 2) {
        let dist = perpendicular_distance(p, start, end);
        if dist > max_dist {
            max_dist = dist;
            max_idx = i;
        }
    }

    if max_dist > epsilon {
        let mut left = rdp_simplify(&points[..=max_idx], epsilon);
        let right = rdp_simplify(&points[max_idx..], epsilon);
        left.pop(); // Remove duplicate split point.
        left.extend(right);
        left
    } else {
        vec![start, end]
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::rdp_simplify;

    #[test]
    fn when_collinear_points_simplified_with_nonzero_epsilon_then_only_endpoints_remain() {
        // Arrange
        let points = [(0.0, 0.0), (1.0, 0.0), (2.0, 0.0), (3.0, 0.0), (4.0, 0.0)];

        // Act
        let simplified = rdp_simplify(&points, 0.1);

        // Assert
        assert_eq!(simplified, vec![(0.0, 0.0), (4.0, 0.0)]);
    }

    #[test]
    fn when_epsilon_zero_then_all_points_preserved() {
        // Arrange
        let points = [(0.0, 0.0), (1.0, 1.0), (2.0, 0.5), (3.0, 2.0), (4.0, 0.0)];

        // Act
        let simplified = rdp_simplify(&points, 0.0);

        // Assert
        assert_eq!(simplified.len(), 5);
    }

    #[test]
    fn when_large_epsilon_then_only_endpoints_preserved() {
        // Arrange
        let points = [(0.0, 0.0), (1.0, 5.0), (2.0, -3.0), (3.0, 7.0), (4.0, 0.0)];

        // Act
        let simplified = rdp_simplify(&points, 100.0);

        // Assert
        assert_eq!(simplified.len(), 2);
        assert_eq!(simplified[0], (0.0, 0.0));
        assert_eq!(simplified[1], (4.0, 0.0));
    }

    #[test]
    fn when_right_angle_with_small_epsilon_then_corner_preserved() {
        // Arrange — right angle: horizontal then vertical
        let points = [(0.0, 0.0), (10.0, 0.0), (10.0, 10.0)];

        // Act
        let simplified = rdp_simplify(&points, 0.5);

        // Assert — corner point has max deviation, must survive
        assert_eq!(simplified.len(), 3);
    }

    #[test]
    fn when_noisy_circle_then_output_has_fewer_points() {
        // Arrange — 100 points on a rough circle
        let n = 100;
        let points: Vec<(f32, f32)> = (0..n)
            .map(|i| {
                let angle = 2.0 * std::f32::consts::PI * i as f32 / n as f32;
                (angle.cos() * 10.0, angle.sin() * 10.0)
            })
            .collect();

        // Act
        let simplified = rdp_simplify(&points, 0.05);

        // Assert — should reduce but not collapse
        assert!(
            simplified.len() < n,
            "expected fewer than {n} points, got {}",
            simplified.len()
        );
        assert!(
            simplified.len() > 2,
            "circle should not collapse to 2 points"
        );
    }
}
