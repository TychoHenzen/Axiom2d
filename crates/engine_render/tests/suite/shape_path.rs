#![allow(clippy::unwrap_used)]

use engine_render::shape::{
    PathCommand, resolve_commands, reverse_path, sample_cubic, sample_quadratic, split_contours,
};
use glam::Vec2;

#[test]
fn when_reverse_path_on_triangle_then_winding_is_reversed() {
    // Arrange
    let path = vec![
        PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
        PathCommand::LineTo(Vec2::new(10.0, 0.0)),
        PathCommand::LineTo(Vec2::new(5.0, 10.0)),
        PathCommand::Close,
    ];

    // Act
    let reversed = reverse_path(&path);

    // Assert — starts at last endpoint, traces back to first
    assert_eq!(reversed.len(), 4);
    assert_eq!(reversed[0], PathCommand::MoveTo(Vec2::new(5.0, 10.0)));
    assert_eq!(reversed[1], PathCommand::LineTo(Vec2::new(10.0, 0.0)));
    assert_eq!(reversed[2], PathCommand::LineTo(Vec2::new(0.0, 0.0)));
    assert_eq!(reversed[3], PathCommand::Close);
}

#[test]
fn when_reverse_path_with_cubic_then_control_points_are_swapped() {
    // Arrange
    let path = vec![
        PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
        PathCommand::CubicTo {
            control1: Vec2::new(1.0, 2.0),
            control2: Vec2::new(3.0, 4.0),
            to: Vec2::new(5.0, 0.0),
        },
        PathCommand::Close,
    ];

    // Act
    let reversed = reverse_path(&path);

    // Assert — reversed cubic swaps control1 and control2
    assert_eq!(reversed[0], PathCommand::MoveTo(Vec2::new(5.0, 0.0)));
    match &reversed[1] {
        PathCommand::CubicTo {
            control1,
            control2,
            to,
        } => {
            assert_eq!(*control1, Vec2::new(3.0, 4.0));
            assert_eq!(*control2, Vec2::new(1.0, 2.0));
            assert_eq!(*to, Vec2::new(0.0, 0.0));
        }
        _ => panic!("expected CubicTo"),
    }
    assert_eq!(reversed[2], PathCommand::Close);
}

#[test]
fn when_reverse_path_twice_then_original_is_restored() {
    // Arrange
    let path = vec![
        PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
        PathCommand::QuadraticTo {
            control: Vec2::new(5.0, 10.0),
            to: Vec2::new(10.0, 0.0),
        },
        PathCommand::LineTo(Vec2::new(5.0, -5.0)),
        PathCommand::Close,
    ];

    // Act
    let roundtrip = reverse_path(&reverse_path(&path));

    // Assert
    assert_eq!(roundtrip, path);
}

#[test]
fn when_resolve_commands_with_reverse_then_contour_winding_is_flipped() {
    // Arrange — triangle with Reverse before Close
    let commands = vec![
        PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
        PathCommand::LineTo(Vec2::new(10.0, 0.0)),
        PathCommand::LineTo(Vec2::new(5.0, 10.0)),
        PathCommand::Reverse,
        PathCommand::Close,
    ];

    // Act
    let resolved = resolve_commands(&commands);

    // Assert — equivalent to manually reversed triangle
    let expected = vec![
        PathCommand::MoveTo(Vec2::new(5.0, 10.0)),
        PathCommand::LineTo(Vec2::new(10.0, 0.0)),
        PathCommand::LineTo(Vec2::new(0.0, 0.0)),
        PathCommand::Close,
    ];
    assert_eq!(resolved, expected);
}

#[test]
fn when_resolve_commands_with_moveto_then_records_contour_start() {
    // Arrange — two contours, second MoveTo should start fresh
    let commands = vec![
        PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
        PathCommand::LineTo(Vec2::new(10.0, 0.0)),
        PathCommand::Close,
        PathCommand::MoveTo(Vec2::new(20.0, 0.0)),
        PathCommand::LineTo(Vec2::new(30.0, 0.0)),
        PathCommand::Close,
    ];

    // Act
    let resolved = resolve_commands(&commands);

    // Assert — both contours preserved, second MoveTo position intact
    assert_eq!(resolved.len(), 6);
    assert_eq!(resolved[0], PathCommand::MoveTo(Vec2::new(0.0, 0.0)));
    assert_eq!(resolved[3], PathCommand::MoveTo(Vec2::new(20.0, 0.0)));
}

// Mutant 1: delete match arm PathCommand::MoveTo(_) in resolve_commands (line 58).
// Without the explicit arm, contour_start stays 0 and Reverse in the second contour
// incorrectly reverses from the very beginning of the path.
#[test]
fn when_resolve_commands_with_two_contours_then_second_reverse_only_affects_second() {
    // Arrange — first contour is a line, second is a triangle with Reverse before Close
    let cmds = vec![
        PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
        PathCommand::LineTo(Vec2::new(10.0, 0.0)),
        PathCommand::Close,
        PathCommand::MoveTo(Vec2::new(20.0, 0.0)),
        PathCommand::LineTo(Vec2::new(30.0, 0.0)),
        PathCommand::LineTo(Vec2::new(25.0, 10.0)),
        PathCommand::Reverse,
        PathCommand::Close,
    ];

    // Act
    let resolved = resolve_commands(&cmds);

    // Assert — first contour must be exactly unchanged
    assert_eq!(resolved[0], PathCommand::MoveTo(Vec2::new(0.0, 0.0)));
    assert_eq!(resolved[1], PathCommand::LineTo(Vec2::new(10.0, 0.0)));
    assert_eq!(resolved[2], PathCommand::Close);
    // Second contour starts at index 3 with the correct MoveTo position
    assert_eq!(resolved[3], PathCommand::MoveTo(Vec2::new(25.0, 10.0)));
}

// --- split_contours tests ---

#[test]
fn when_split_contours_single_contour_then_returns_one_vec() {
    // Arrange
    let commands = vec![
        PathCommand::MoveTo(Vec2::ZERO),
        PathCommand::LineTo(Vec2::X),
        PathCommand::Close,
    ];

    // Act
    let contours = split_contours(&commands);

    // Assert
    assert_eq!(contours.len(), 1);
    assert_eq!(contours[0].len(), 3);
}

#[test]
fn when_split_contours_empty_then_returns_empty() {
    // Act
    let contours = split_contours(&[]);

    // Assert
    assert!(contours.is_empty());
}

#[test]
fn when_split_contours_three_contours_then_returns_three() {
    // Arrange
    let commands = vec![
        PathCommand::MoveTo(Vec2::ZERO),
        PathCommand::LineTo(Vec2::X),
        PathCommand::Close,
        PathCommand::MoveTo(Vec2::Y),
        PathCommand::LineTo(Vec2::ONE),
        PathCommand::Close,
        PathCommand::MoveTo(Vec2::new(5.0, 5.0)),
        PathCommand::LineTo(Vec2::new(6.0, 6.0)),
        PathCommand::Close,
    ];

    // Act
    let contours = split_contours(&commands);

    // Assert
    assert_eq!(contours.len(), 3);
}

#[test]
fn when_split_contours_then_each_starts_with_moveto() {
    // Arrange
    let commands = vec![
        PathCommand::MoveTo(Vec2::ZERO),
        PathCommand::LineTo(Vec2::X),
        PathCommand::Close,
        PathCommand::MoveTo(Vec2::Y),
        PathCommand::LineTo(Vec2::ONE),
        PathCommand::Close,
    ];

    // Act
    let contours = split_contours(&commands);

    // Assert
    for (i, contour) in contours.iter().enumerate() {
        assert!(
            matches!(contour[0], PathCommand::MoveTo(_)),
            "contour {i} should start with MoveTo"
        );
    }
}

// --- sample_quadratic tests ---

#[test]
fn when_sample_quadratic_then_first_point_equals_from() {
    // Arrange
    let from = Vec2::new(1.0, 2.0);
    let control = Vec2::new(5.0, 10.0);
    let to = Vec2::new(9.0, 2.0);

    // Act
    let points = sample_quadratic(from, control, to, 4);

    // Assert
    assert!(
        (points[0] - from).length() < 1e-6,
        "first point should be from: {:?}",
        points[0]
    );
}

#[test]
fn when_sample_quadratic_then_last_point_equals_to() {
    // Arrange
    let from = Vec2::new(1.0, 2.0);
    let control = Vec2::new(5.0, 10.0);
    let to = Vec2::new(9.0, 2.0);

    // Act
    let points = sample_quadratic(from, control, to, 4);

    // Assert
    assert!(
        (points[4] - to).length() < 1e-6,
        "last point should be to: {:?}",
        points[4]
    );
}

#[test]
fn when_sample_quadratic_then_returns_n_plus_one_points() {
    // Act
    let points = sample_quadratic(Vec2::ZERO, Vec2::Y, Vec2::X, 8);

    // Assert
    assert_eq!(points.len(), 9);
}

#[test]
fn when_sample_quadratic_midpoint_then_follows_bezier_formula() {
    // Arrange — at t=0.5: (1-t)^2 * from + 2*(1-t)*t * control + t^2 * to
    let from = Vec2::new(0.0, 0.0);
    let control = Vec2::new(0.0, 10.0);
    let to = Vec2::new(10.0, 0.0);

    // Act
    let points = sample_quadratic(from, control, to, 2);

    // Assert — midpoint (t=0.5): 0.25*(0,0) + 0.5*(0,10) + 0.25*(10,0) = (2.5, 5.0)
    let mid = points[1];
    assert!(
        (mid.x - 2.5).abs() < 1e-4,
        "expected mid.x=2.5, got {}",
        mid.x
    );
    assert!(
        (mid.y - 5.0).abs() < 1e-4,
        "expected mid.y=5.0, got {}",
        mid.y
    );
}

#[test]
fn when_sample_quadratic_with_collinear_points_then_midpoint_on_line() {
    // Arrange — all three points on a line → quadratic degenerates to line
    let from = Vec2::new(0.0, 0.0);
    let control = Vec2::new(5.0, 5.0);
    let to = Vec2::new(10.0, 10.0);

    // Act
    let points = sample_quadratic(from, control, to, 4);

    // Assert — all points should lie on y = x
    for (i, p) in points.iter().enumerate() {
        assert!(
            (p.x - p.y).abs() < 1e-4,
            "point {i} should be on y=x line: ({}, {})",
            p.x,
            p.y
        );
    }
}

// --- sample_cubic tests ---

#[test]
fn when_sample_cubic_then_first_point_equals_from() {
    // Arrange
    let from = Vec2::new(1.0, 2.0);
    let c1 = Vec2::new(3.0, 10.0);
    let c2 = Vec2::new(7.0, 10.0);
    let to = Vec2::new(9.0, 2.0);

    // Act
    let points = sample_cubic(from, c1, c2, to, 4);

    // Assert
    assert!(
        (points[0] - from).length() < 1e-6,
        "first point should be from: {:?}",
        points[0]
    );
}

#[test]
fn when_sample_cubic_then_last_point_equals_to() {
    // Arrange
    let from = Vec2::new(1.0, 2.0);
    let c1 = Vec2::new(3.0, 10.0);
    let c2 = Vec2::new(7.0, 10.0);
    let to = Vec2::new(9.0, 2.0);

    // Act
    let points = sample_cubic(from, c1, c2, to, 4);

    // Assert
    assert!(
        (points[4] - to).length() < 1e-6,
        "last point should be to: {:?}",
        points[4]
    );
}

#[test]
fn when_sample_cubic_then_returns_n_plus_one_points() {
    // Act
    let points = sample_cubic(Vec2::ZERO, Vec2::Y, Vec2::X, Vec2::ONE, 6);

    // Assert
    assert_eq!(points.len(), 7);
}

#[test]
fn when_sample_cubic_midpoint_then_follows_bezier_formula() {
    // Arrange — at t=0.5: (1-t)^3*p0 + 3*(1-t)^2*t*p1 + 3*(1-t)*t^2*p2 + t^3*p3
    let from = Vec2::new(0.0, 0.0);
    let c1 = Vec2::new(0.0, 10.0);
    let c2 = Vec2::new(10.0, 10.0);
    let to = Vec2::new(10.0, 0.0);

    // Act
    let points = sample_cubic(from, c1, c2, to, 2);

    // Assert — midpoint (t=0.5):
    // 0.125*(0,0) + 3*0.25*0.5*(0,10) + 3*0.5*0.25*(10,10) + 0.125*(10,0)
    // = (0,0) + 0.375*(0,10) + 0.375*(10,10) + (1.25,0)
    // = (0, 3.75) + (3.75, 3.75) + (1.25, 0)
    // = (5.0, 7.5)
    let mid = points[1];
    assert!(
        (mid.x - 5.0).abs() < 1e-3,
        "expected mid.x=5.0, got {}",
        mid.x
    );
    assert!(
        (mid.y - 7.5).abs() < 1e-3,
        "expected mid.y=7.5, got {}",
        mid.y
    );
}
