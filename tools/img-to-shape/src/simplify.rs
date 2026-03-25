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

/// Simplify an **open** point sequence using the Ramer-Douglas-Peucker algorithm.
/// Endpoints are always preserved.
pub fn rdp_open(points: &[(f32, f32)], epsilon: f32) -> Vec<(f32, f32)> {
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
        let mut left = rdp_open(&points[..=max_idx], epsilon);
        let right = rdp_open(&points[max_idx..], epsilon);
        left.pop(); // Remove duplicate split point.
        left.extend(right);
        left
    } else {
        vec![start, end]
    }
}

/// Simplify a **closed polygon** using Ramer-Douglas-Peucker.
///
/// Unlike `rdp_simplify` (which treats the input as an open polyline with
/// fixed endpoints), this splits the polygon at the two farthest-apart
/// vertices and simplifies each half independently. This ensures no vertex
/// is artificially preserved just because it happens to be the first or
/// last in the input sequence.
pub fn rdp_simplify_closed(points: &[(f32, f32)], epsilon: f32) -> Vec<(f32, f32)> {
    let n = points.len();
    if n < 4 {
        return points.to_vec();
    }

    // Find the two points farthest apart (by Euclidean distance).
    let (mut split_a, mut split_b) = (0, n / 2);
    let mut best_dist = 0.0_f32;
    for i in 0..n {
        for j in (i + 1)..n {
            let dx = points[j].0 - points[i].0;
            let dy = points[j].1 - points[i].1;
            let d = dx * dx + dy * dy;
            if d > best_dist {
                best_dist = d;
                split_a = i;
                split_b = j;
            }
        }
    }

    // Rotate so split_a is at index 0.
    let mut rotated: Vec<(f32, f32)> = Vec::with_capacity(n);
    for i in 0..n {
        rotated.push(points[(split_a + i) % n]);
    }
    let split_b_rotated = (split_b + n - split_a) % n;

    // First half: from split_a to split_b (indices 0..=split_b_rotated).
    let first_half = &rotated[..=split_b_rotated];
    // Second half: from split_b back to split_a (indices split_b_rotated..n, wrapping).
    let mut second_half: Vec<(f32, f32)> = rotated[split_b_rotated..].to_vec();
    second_half.push(rotated[0]); // Close back to split_a.

    let mut simplified_first = rdp_open(first_half, epsilon);
    let simplified_second = rdp_open(&second_half, epsilon);

    // Merge: remove duplicate at split_b junction and at the closing point.
    simplified_first.pop(); // Remove duplicate split_b.
    simplified_first.extend(&simplified_second[..simplified_second.len() - 1]); // Remove duplicate split_a.

    simplified_first
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::{rdp_open as rdp_simplify, rdp_simplify_closed};

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

    #[test]
    fn when_closed_square_simplified_then_four_corners_preserved() {
        // Arrange — square with collinear intermediate points
        let points = vec![
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
        let simplified = rdp_simplify_closed(&points, 0.5);

        // Assert — only 4 corners should remain
        assert_eq!(
            simplified.len(),
            4,
            "square should simplify to 4 corners, got {:?}",
            simplified
        );
    }

    #[test]
    fn when_closed_l_shape_simplified_then_concave_corner_preserved() {
        // Arrange — L-shape with many collinear points
        #[rustfmt::skip]
        let points = vec![
            (0.0, 0.0), (1.0, 0.0), (2.0, 0.0),   // top edge
            (2.0, 1.0), (2.0, 2.0), (2.0, 3.0),     // right edge of narrow part
            (3.0, 3.0), (4.0, 3.0), (5.0, 3.0),     // top of bottom part
            (5.0, 4.0), (5.0, 5.0),                   // right edge
            (4.0, 5.0), (3.0, 5.0), (2.0, 5.0), (1.0, 5.0), (0.0, 5.0), // bottom
            (0.0, 4.0), (0.0, 3.0), (0.0, 2.0), (0.0, 1.0), // left edge
        ];

        // Act
        let simplified = rdp_simplify_closed(&points, 0.5);

        // Assert — the concave corner at (2,3) must survive
        assert!(
            simplified.contains(&(2.0, 3.0)),
            "concave corner (2,3) should be preserved, got {:?}",
            simplified
        );
        // Should have 6 corners: (0,0), (2,0), (2,3), (5,3), (5,5), (0,5)
        assert_eq!(
            simplified.len(),
            6,
            "L-shape should simplify to 6 corners, got {:?}",
            simplified
        );
    }

    #[test]
    fn when_closed_polygon_simplified_then_result_has_no_duplicate_endpoints() {
        // Arrange — triangle
        let points = vec![
            (0.0, 0.0),
            (5.0, 0.0),
            (10.0, 0.0),
            (10.0, 5.0),
            (5.0, 10.0),
            (0.0, 5.0),
        ];

        // Act
        let simplified = rdp_simplify_closed(&points, 0.5);

        // Assert — first and last should NOT be the same point
        assert_ne!(
            simplified.first(),
            simplified.last(),
            "closed polygon should not have duplicate start/end"
        );
    }
}
