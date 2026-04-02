#![allow(clippy::unwrap_used, clippy::float_cmp)]

use glam::Vec2;

use engine_render::camera::Camera2D;
use engine_render::culling::{aabb_intersects_view_rect, camera_view_rect};

#[test]
fn when_view_rect_at_zoom_one_then_half_extents_equal_half_viewport() {
    // Arrange
    let camera = Camera2D {
        position: Vec2::new(400.0, 300.0),
        zoom: 1.0,
    };

    // Act
    let (min, max) = camera_view_rect(&camera, 800.0, 600.0);

    // Assert
    assert!((min.x - 0.0).abs() < 1e-4);
    assert!((min.y - 0.0).abs() < 1e-4);
    assert!((max.x - 800.0).abs() < 1e-4);
    assert!((max.y - 600.0).abs() < 1e-4);
}

#[test]
fn when_view_rect_at_zoom_two_then_half_extents_are_halved() {
    // Arrange
    let camera = Camera2D {
        position: Vec2::new(400.0, 300.0),
        zoom: 2.0,
    };

    // Act
    let (min, max) = camera_view_rect(&camera, 800.0, 600.0);

    // Assert
    assert!((min.x - 200.0).abs() < 1e-4);
    assert!((min.y - 150.0).abs() < 1e-4);
    assert!((max.x - 600.0).abs() < 1e-4);
    assert!((max.y - 450.0).abs() < 1e-4);
}

#[test]
fn when_entity_fully_inside_view_then_aabb_intersects_returns_true() {
    // Act / Assert
    assert!(aabb_intersects_view_rect(
        Vec2::new(100.0, 100.0),
        Vec2::new(200.0, 200.0),
        Vec2::new(0.0, 0.0),
        Vec2::new(800.0, 600.0),
    ));
}

/// @doc: Frustum culling AABB test — entity fully outside on any axis means no intersection
#[test]
fn when_entity_completely_left_of_view_then_aabb_intersects_returns_false() {
    // Act / Assert
    assert!(!aabb_intersects_view_rect(
        Vec2::new(-200.0, 0.0),
        Vec2::new(-10.0, 100.0),
        Vec2::new(0.0, 0.0),
        Vec2::new(800.0, 600.0),
    ));
}

#[test]
fn when_entity_completely_right_of_view_then_aabb_intersects_returns_false() {
    // Act / Assert
    assert!(!aabb_intersects_view_rect(
        Vec2::new(850.0, 0.0),
        Vec2::new(1000.0, 100.0),
        Vec2::new(0.0, 0.0),
        Vec2::new(800.0, 600.0),
    ));
}

#[test]
fn when_entity_completely_above_view_then_aabb_intersects_returns_false() {
    // Act / Assert
    assert!(!aabb_intersects_view_rect(
        Vec2::new(0.0, -200.0),
        Vec2::new(100.0, -10.0),
        Vec2::new(0.0, 0.0),
        Vec2::new(800.0, 600.0),
    ));
}

#[test]
fn when_entity_completely_below_view_then_aabb_intersects_returns_false() {
    // Act / Assert
    assert!(!aabb_intersects_view_rect(
        Vec2::new(0.0, 650.0),
        Vec2::new(100.0, 800.0),
        Vec2::new(0.0, 0.0),
        Vec2::new(800.0, 600.0),
    ));
}

#[test]
fn when_entity_partially_overlaps_left_edge_then_aabb_intersects_returns_true() {
    // Act / Assert
    assert!(aabb_intersects_view_rect(
        Vec2::new(-50.0, 100.0),
        Vec2::new(50.0, 200.0),
        Vec2::new(0.0, 0.0),
        Vec2::new(800.0, 600.0),
    ));
}

#[test]
fn when_entity_exactly_touches_view_edge_then_aabb_intersects_returns_true() {
    // Act / Assert
    assert!(aabb_intersects_view_rect(
        Vec2::new(-10.0, 0.0),
        Vec2::new(0.0, 100.0),
        Vec2::new(0.0, 0.0),
        Vec2::new(800.0, 600.0),
    ));
}

#[test]
fn when_entity_contains_entire_view_then_aabb_intersects_returns_true() {
    // Act / Assert
    assert!(aabb_intersects_view_rect(
        Vec2::new(0.0, 0.0),
        Vec2::new(800.0, 600.0),
        Vec2::new(100.0, 100.0),
        Vec2::new(700.0, 500.0),
    ));
}
