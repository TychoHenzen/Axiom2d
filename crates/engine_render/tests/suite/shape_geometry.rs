#![allow(clippy::unwrap_used, clippy::float_cmp)]

use engine_render::shape::{
    PathCommand, QUAD_INDICES, ShapeVariant, UNIT_QUAD, rect_polygon, rect_vertices,
    rounded_rect_path, sample_quadratic, unit_quad_model,
};
use glam::Vec2;

#[test]
fn when_unit_quad_then_vertices_span_one() {
    let w = UNIT_QUAD[1][0] - UNIT_QUAD[0][0];
    let h = UNIT_QUAD[3][1] - UNIT_QUAD[0][1];
    assert!((w - 1.0).abs() < 1e-6, "width={w}");
    assert!((h - 1.0).abs() < 1e-6, "height={h}");
}

#[test]
fn when_unit_quad_model_then_matrix_scales_and_translates() {
    // Act
    let m = unit_quad_model(100.0, 200.0, 10.0, 20.0);

    // Assert
    assert_eq!(m[0][0], 100.0, "scale x");
    assert_eq!(m[1][1], 200.0, "scale y");
    assert_eq!(m[3][0], 10.0, "translate x");
    assert_eq!(m[3][1], 20.0, "translate y");
    assert_eq!(m[2][2], 1.0, "z identity");
    assert_eq!(m[3][3], 1.0, "w");
}

#[test]
fn when_rect_polygon_then_half_extents_match() {
    let ShapeVariant::Polygon { ref points } = rect_polygon(30.0, 45.0) else {
        panic!("expected Polygon");
    };
    let max_x = points.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
    let max_y = points.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
    assert!((max_x - 30.0).abs() < 1e-6);
    assert!((max_y - 45.0).abs() < 1e-6);
}

#[test]
fn when_rect_vertices_then_corners_at_expected_positions() {
    let verts = rect_vertices(10.0, 20.0, 30.0, 40.0);
    assert_eq!(verts[0], [10.0, 20.0]);
    assert_eq!(verts[1], [40.0, 20.0]);
    assert_eq!(verts[2], [40.0, 60.0]);
    assert_eq!(verts[3], [10.0, 60.0]);
}

#[test]
fn when_rounded_rect_path_called_then_returns_path_variant() {
    // Arrange
    let half_w = 30.0_f32;
    let half_h = 45.0_f32;
    let radius = 5.0_f32;

    // Act
    let variant = rounded_rect_path(half_w, half_h, radius);

    // Assert
    assert!(
        matches!(variant, ShapeVariant::Path { .. }),
        "expected ShapeVariant::Path, got {variant:?}"
    );
}

#[test]
fn when_rounded_rect_path_called_then_starts_with_moveto_ends_with_close() {
    // Arrange / Act
    let ShapeVariant::Path { ref commands } = rounded_rect_path(30.0, 45.0, 5.0) else {
        panic!("expected Path");
    };

    // Assert
    assert!(
        matches!(commands.first(), Some(PathCommand::MoveTo(_))),
        "path must start with MoveTo"
    );
    assert!(
        matches!(commands.last(), Some(PathCommand::Close)),
        "path must end with Close"
    );
}

#[test]
fn when_rounded_rect_path_called_then_contains_four_quadratic_curves() {
    // Arrange / Act
    let ShapeVariant::Path { ref commands } = rounded_rect_path(30.0, 45.0, 5.0) else {
        panic!("expected Path");
    };

    // Assert
    let quad_count = commands
        .iter()
        .filter(|c| matches!(c, PathCommand::QuadraticTo { .. }))
        .count();
    assert_eq!(quad_count, 4, "one QuadraticTo per corner");
}

#[test]
fn when_rounded_rect_path_called_then_no_sampled_point_exceeds_half_extents() {
    // Arrange
    let half_w = 30.0_f32;
    let half_h = 45.0_f32;
    let ShapeVariant::Path { ref commands } = rounded_rect_path(half_w, half_h, 5.0) else {
        panic!("expected Path");
    };

    // Act — sample all quadratic curves
    let mut prev = Vec2::ZERO;
    for cmd in commands {
        match *cmd {
            PathCommand::MoveTo(p) | PathCommand::LineTo(p) => {
                assert!(
                    p.x.abs() <= half_w + 1e-4 && p.y.abs() <= half_h + 1e-4,
                    "point ({}, {}) exceeds bounds",
                    p.x,
                    p.y
                );
                prev = p;
            }
            PathCommand::QuadraticTo { control, to } => {
                for pt in sample_quadratic(prev, control, to, 20) {
                    assert!(
                        pt.x.abs() <= half_w + 1e-4 && pt.y.abs() <= half_h + 1e-4,
                        "sampled point ({}, {}) exceeds bounds",
                        pt.x,
                        pt.y
                    );
                }
                prev = to;
            }
            _ => {}
        }
    }
}

#[test]
fn when_rounded_rect_path_called_then_control_points_at_sharp_corners() {
    // Arrange
    let half_w = 30.0_f32;
    let half_h = 45.0_f32;
    let ShapeVariant::Path { ref commands } = rounded_rect_path(half_w, half_h, 5.0) else {
        panic!("expected Path");
    };

    // Act
    let controls: Vec<Vec2> = commands
        .iter()
        .filter_map(|c| match c {
            PathCommand::QuadraticTo { control, .. } => Some(*control),
            _ => None,
        })
        .collect();

    // Assert — each control point should be at one of the 4 sharp corners
    let corners = [
        Vec2::new(half_w, -half_h),
        Vec2::new(half_w, half_h),
        Vec2::new(-half_w, half_h),
        Vec2::new(-half_w, -half_h),
    ];
    for ctrl in &controls {
        assert!(
            corners
                .iter()
                .any(|c| (ctrl.x - c.x).abs() < 1e-4 && (ctrl.y - c.y).abs() < 1e-4),
            "control point ({}, {}) is not at any corner",
            ctrl.x,
            ctrl.y
        );
    }
}

#[test]
fn when_rounded_rect_path_called_then_arc_endpoints_offset_by_radius() {
    // Arrange
    let half_w = 30.0_f32;
    let half_h = 45.0_f32;
    let radius = 5.0_f32;
    let ShapeVariant::Path { ref commands } = rounded_rect_path(half_w, half_h, radius) else {
        panic!("expected Path");
    };

    // Act — collect (control, to) pairs from QuadraticTo commands
    // and the preceding LineTo endpoint as the arc start
    let mut prev = Vec2::ZERO;
    for cmd in commands {
        match *cmd {
            PathCommand::MoveTo(p) | PathCommand::LineTo(p) => prev = p,
            PathCommand::QuadraticTo { control, to } => {
                // Arc start (prev) should be radius away from the control point along one axis
                let d_start = (prev - control).abs();
                let d_end = (to - control).abs();
                assert!(
                    (d_start.x.min(d_start.y) < 1e-4
                        && (d_start.x.max(d_start.y) - radius).abs() < 1e-4),
                    "arc start ({}, {}) not radius={radius} from control ({}, {})",
                    prev.x,
                    prev.y,
                    control.x,
                    control.y
                );
                assert!(
                    (d_end.x.min(d_end.y) < 1e-4 && (d_end.x.max(d_end.y) - radius).abs() < 1e-4),
                    "arc end ({}, {}) not radius={radius} from control ({}, {})",
                    to.x,
                    to.y,
                    control.x,
                    control.y
                );
                prev = to;
            }
            _ => {}
        }
    }
}

#[test]
fn when_rounded_rect_path_with_zero_radius_then_points_at_corners() {
    // Arrange / Act
    let ShapeVariant::Path { ref commands } = rounded_rect_path(30.0, 45.0, 0.0) else {
        panic!("expected Path");
    };

    // Assert — all MoveTo/LineTo/QuadraticTo endpoints should be at corners
    let corners = [
        Vec2::new(30.0, -45.0),
        Vec2::new(30.0, 45.0),
        Vec2::new(-30.0, 45.0),
        Vec2::new(-30.0, -45.0),
    ];
    for cmd in commands {
        let points: Vec<Vec2> = match *cmd {
            PathCommand::MoveTo(p) | PathCommand::LineTo(p) => vec![p],
            PathCommand::QuadraticTo { control, to } => vec![control, to],
            _ => vec![],
        };
        for pt in points {
            assert!(
                corners.iter().any(|c| (pt - *c).length() < 1e-4),
                "point ({}, {}) is not at any corner with radius=0",
                pt.x,
                pt.y
            );
        }
    }
}
