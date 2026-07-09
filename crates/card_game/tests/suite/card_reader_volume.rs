#![allow(clippy::unwrap_used)]

use card_game::card::identity::signature::CardSignature;
use card_game::card::reader::signature_radius;
use card_game::card::reader::volume::{
    polyline_arc_length, solve_tube_radius, sphere_volume_8d, tube_volume_8d,
};

/// @doc: A sphere with radius zero has zero volume in 8D.
#[test]
fn when_sphere_radius_is_zero_then_volume_is_zero() {
    assert_eq!(
        sphere_volume_8d(0.0),
        0.0,
        "zero-radius sphere must have zero volume"
    );
}

/// @doc: `sphere_volume_8d` matches the analytic formula pi^4/24 * r^8 for radius 0.2.
#[test]
fn when_sphere_radius_is_02_then_volume_matches_formula() {
    // Arrange
    let r = 0.2_f32;
    let pi4 = std::f32::consts::PI.powi(4);
    let expected = pi4 / 24.0 * r.powi(8);

    // Act
    let v = sphere_volume_8d(r);

    // Assert
    assert!((v - expected).abs() < 1e-12, "got {v}, expected {expected}");
}

/// @doc: Tube volume with zero arc length equals sphere volume of the same radius.
#[test]
fn when_arc_length_is_zero_then_tube_volume_equals_sphere_volume() {
    // Arrange
    let r = 0.2;

    // Act
    let tube_v = tube_volume_8d(r, 0.0);
    let sphere_v = sphere_volume_8d(r);

    // Assert
    assert!(
        (tube_v - sphere_v).abs() < 1e-12,
        "tube volume with zero arc length ({tube_v}) must equal sphere volume ({sphere_v})"
    );
}

/// @doc: Newton solver recovers the original sphere radius when given a sphere volume and zero arc length.
#[test]
fn when_newton_solver_given_sphere_volume_and_zero_length_then_returns_original_radius() {
    // Arrange
    let r = 0.2;
    let v = sphere_volume_8d(r);

    // Act
    let solved = solve_tube_radius(v, 0.0);

    // Assert
    assert!((solved - r).abs() < 1e-5, "got {solved}, expected {r}");
}

/// @doc: Newton solver produces a tube radius whose volume matches the sum of two capsule sphere volumes.
#[test]
fn when_newton_solver_given_capsule_then_radius_preserves_volume() {
    // Arrange
    let r1 = 0.18;
    let r2 = 0.22;
    let v_total = sphere_volume_8d(r1) + sphere_volume_8d(r2);
    let arc_len = 0.5; // arbitrary segment length

    // Act
    let solved_r = solve_tube_radius(v_total, arc_len);

    // Assert
    let actual_v = tube_volume_8d(solved_r, arc_len);
    assert!(
        (actual_v - v_total).abs() < 1e-6,
        "tube volume {actual_v} must match target {v_total}, solved radius = {solved_r}"
    );
}

/// @doc: Polyline arc length between two points equals Euclidean distance in 8D.
#[test]
fn when_polyline_has_two_points_then_arc_length_is_euclidean_distance() {
    // Arrange
    let a = CardSignature::new([0.0; 8]);
    let b = CardSignature::new([0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);

    // Act
    let len = polyline_arc_length(&[a, b]);

    // Assert
    assert!(
        (len - 0.5).abs() < 1e-6,
        "arc length {len} should equal Euclidean distance 0.5"
    );
}

/// @doc: Polyline arc length with a single point is zero.
#[test]
fn when_polyline_has_one_point_then_arc_length_is_zero() {
    let a = CardSignature::new([0.1; 8]);
    assert_eq!(
        polyline_arc_length(&[a]),
        0.0,
        "single-point polyline must have zero arc length"
    );
}

/// @doc: `signature_radius` returns the minimum (0.15) when all signature axes are zero.
#[test]
fn when_all_axes_zero_then_radius_is_015() {
    // Arrange
    let sig = CardSignature::new([0.0; 8]);

    // Act
    let r = signature_radius(&sig);

    // Assert
    assert!(
        (r - 0.15).abs() < 1e-6,
        "expected 0.15 for zero-intensity signature, got {r}"
    );
}

/// @doc: `signature_radius` returns the maximum (0.25) when all axes are at peak intensity (+-1.0).
#[test]
fn when_all_axes_max_intensity_then_radius_is_025() {
    // Arrange — all axes at ±1.0 (max intensity)
    let sig = CardSignature::new([1.0, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0, -1.0]);

    // Act
    let r = signature_radius(&sig);

    // Assert
    assert!(
        (r - 0.25).abs() < 1e-6,
        "expected 0.25 for max-intensity signature, got {r}"
    );
}

/// @doc: `signature_radius` is deterministic for the same input.
#[test]
fn when_signature_radius_called_twice_then_deterministic() {
    let sig = CardSignature::new([0.3, -0.7, 0.1, 0.4, -0.2, 0.5, -0.8, 0.6]);
    assert_eq!(
        signature_radius(&sig),
        signature_radius(&sig),
        "signature_radius must be deterministic"
    );
}
