#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::*;
use card_game::card::identity::signature::CardSignature;
use card_game::card::jack_cable::{
    Cable, CableCollider, CableRope, Jack, JackDirection, WireEndpoints, WrapAnchor, WrapWire,
    find_wrap_vertex, particles_to_bezier_path, point_in_convex_polygon, polyline_to_ribbon,
    rope_solve_system, segment_intersects_segment, signature_space_propagation_system,
    wire_render_system, wrap_detect_system, wrap_update_system,
};
use card_game::card::reader::{SIGNATURE_SPACE_RADIUS, SignatureSpace};
use engine_core::prelude::{DeltaTime, Seconds, Transform2D};
use engine_render::prelude::{Shape, ShapeVariant};
use engine_render::shape::PathCommand;
use engine_scene::prelude::{SortOrder, Visible};
use engine_scene::render_order::RenderLayer;
use glam::Vec2;

// ---------------------------------------------------------------------------
// TC004 — signature_space_propagation_system copies output jack data to input jack
// ---------------------------------------------------------------------------

fn run_cable_propagation(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(signature_space_propagation_system);
    schedule.run(world);
}

fn make_space(center_values: [f32; 8]) -> SignatureSpace {
    SignatureSpace {
        center: CardSignature::new(center_values),
        radius: SIGNATURE_SPACE_RADIUS,
    }
}

/// @doc: The cable propagation system is the signal backbone of the wiring layer: without
/// it, a `SignatureSpace` emitted by an output jack never reaches the input jack it is
/// connected to, so downstream devices remain blind to the card's identity. This test
/// verifies the core contract — a `Cable` carries a `SignatureSpace` from the output jack's
/// `data` field to the input jack's `data` field in a single schedule tick. If this transfer
/// is absent, no wiring chain of any length can deliver a signal to its destination.
#[test]
fn when_output_jack_has_data_and_cable_connects_it_then_input_jack_receives_data() {
    // Arrange
    let mut world = World::new();
    let signal = make_space([0.1, 0.2, 0.3, 0.4, -0.1, -0.2, -0.3, -0.4]);

    let output_jack = world
        .spawn(Jack::<SignatureSpace> {
            direction: JackDirection::Output,
            data: Some(signal.clone()),
        })
        .id();

    let input_jack = world
        .spawn(Jack::<SignatureSpace> {
            direction: JackDirection::Input,
            data: None,
        })
        .id();

    world.spawn(Cable {
        source: output_jack,
        dest: input_jack,
    });

    // Act
    run_cable_propagation(&mut world);

    // Assert
    let received = world
        .entity_mut(input_jack)
        .get_mut::<Jack<SignatureSpace>>()
        .unwrap()
        .data
        .clone();
    assert_eq!(
        received,
        Some(signal),
        "input jack must hold the exact SignatureSpace that was on the output jack"
    );
}

// ---------------------------------------------------------------------------
// TC005 — None output must clear input (no stale signal)
// ---------------------------------------------------------------------------

/// @doc: Cable propagation must faithfully represent the absence of a signal: when the
/// output jack carries no data (`None`), the cable must write `None` to the input jack
/// rather than leaving stale data from a previous tick. Without this reset behaviour the
/// wiring system would continue delivering phantom signatures after a card is removed from
/// its reader, causing downstream devices to react to cards that are no longer present.
/// This boundary test validates that a silent output jack silences the connected input jack.
#[test]
fn when_output_jack_has_no_data_then_input_jack_data_remains_none_after_propagation() {
    // Arrange – input jack starts with stale data from a previous tick
    let mut world = World::new();
    let stale = make_space([0.9, 0.8, 0.7, 0.6, -0.9, -0.8, -0.7, -0.6]);

    let output_jack = world
        .spawn(Jack::<SignatureSpace> {
            direction: JackDirection::Output,
            data: None,
        })
        .id();

    let input_jack = world
        .spawn(Jack::<SignatureSpace> {
            direction: JackDirection::Input,
            data: Some(stale),
        })
        .id();

    world.spawn(Cable {
        source: output_jack,
        dest: input_jack,
    });

    // Act
    run_cable_propagation(&mut world);

    // Assert – stale signal must be overwritten with None
    let data_after = world
        .entity_mut(input_jack)
        .get_mut::<Jack<SignatureSpace>>()
        .unwrap()
        .data
        .clone();
    assert!(
        data_after.is_none(),
        "input jack must be cleared to None when the connected output jack carries no signal"
    );
}

// ---------------------------------------------------------------------------
// Original TC (SignatureSpace identity axiom — pre-existing, do not remove)
// ---------------------------------------------------------------------------

/// @doc: A `SignatureSpace` centered on a given `CardSignature` must report that its own
/// center point is contained within itself. This is the irreducible identity axiom for
/// any radius-based inclusion test: if the center is not "in range" of itself (distance
/// zero, which is always <= radius), the `contains()` boundary check is fundamentally
/// broken and the wiring system would reject valid same-point signature matches,
/// making it impossible for a card reader to recognize a card whose signature exactly
/// matches its emission zone center.
#[test]
fn when_point_is_center_of_space_then_contains_returns_true() {
    // Arrange
    let center = CardSignature::new([0.3, -0.1, 0.5, 0.0, -0.4, 0.2, -0.7, 0.6]);
    let space = SignatureSpace {
        center,
        radius: SIGNATURE_SPACE_RADIUS,
    };

    // Act
    let result = space.contains(&center);

    // Assert
    assert!(
        result,
        "center point must always be contained within its own SignatureSpace"
    );
}

// ---------------------------------------------------------------------------
// TC-F12 — B-spline to bezier conversion produces correct segment count
// ---------------------------------------------------------------------------

/// @doc: `particles_to_bezier_path` converts waypoint positions into a smooth B-spline
/// curve represented as cubic bezier `PathCommand`s. For N points, the result must contain
/// a single `MoveTo` followed by exactly N-1 `CubicTo` commands — one bezier segment per
/// adjacent point pair. Fewer segments leave gaps in the cable; more segments produce
/// phantom arcs between non-adjacent points.
#[test]
fn given_five_particles_when_bezier_path_computed_then_result_has_move_to_plus_four_cubic_to() {
    // Arrange
    let positions = vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(25.0, 10.0),
        Vec2::new(50.0, 0.0),
        Vec2::new(75.0, -10.0),
        Vec2::new(100.0, 0.0),
    ];

    // Act
    let commands = particles_to_bezier_path(&positions);

    // Assert
    let move_count = commands
        .iter()
        .filter(|c| matches!(c, PathCommand::MoveTo(_)))
        .count();
    let cubic_count = commands
        .iter()
        .filter(|c| matches!(c, PathCommand::CubicTo { .. }))
        .count();
    assert_eq!(move_count, 1, "path must start with exactly one MoveTo");
    assert_eq!(
        cubic_count, 4,
        "5 particles must produce 4 CubicTo segments, got {cubic_count}"
    );
}

// ---------------------------------------------------------------------------
// TC-F13 — collinear points produce a straight bezier
// ---------------------------------------------------------------------------

/// @doc: When all points lie on the same line, the B-spline approximation must also lie
/// on that line — all `CubicTo` control points must have the same Y coordinate as the
/// points. If control points deviate from the chord, a taut cable would render as a
/// wavy curve, confusing the player about whether the cable is slack or tight.
#[test]
fn given_collinear_particles_when_bezier_path_computed_then_all_control_points_on_same_line() {
    // Arrange — 4 points on Y=0
    let positions = vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(30.0, 0.0),
        Vec2::new(60.0, 0.0),
        Vec2::new(90.0, 0.0),
    ];

    // Act
    let commands = particles_to_bezier_path(&positions);

    // Assert
    for cmd in &commands {
        if let PathCommand::CubicTo {
            control1,
            control2,
            to,
        } = cmd
        {
            assert!(
                control1.y.abs() < 1e-4,
                "control1.y must be ~0.0 for collinear points, got {}",
                control1.y
            );
            assert!(
                control2.y.abs() < 1e-4,
                "control2.y must be ~0.0 for collinear points, got {}",
                control2.y
            );
            assert!(
                to.y.abs() < 1e-4,
                "to.y must be ~0.0 for collinear points, got {}",
                to.y
            );
        }
    }
}

// ---------------------------------------------------------------------------
// WrapWire — shortest_path computation
// ---------------------------------------------------------------------------

/// @doc: A `WrapWire` with no anchors represents a cable that wraps around zero obstacles,
/// so its shortest geometric path is simply the straight-line distance between the two
/// endpoints. This is the base case for the path-length computation — if the zero-anchor
/// case returns the wrong value, every path calculation will be wrong even for cables that
/// have never contacted an obstacle.
#[test]
fn when_wrap_wire_has_no_anchors_then_shortest_path_is_endpoint_distance() {
    // Arrange
    let wire = WrapWire::new();
    let src = Vec2::new(0.0, 0.0);
    let dst = Vec2::new(100.0, 0.0);

    // Act
    let path = wire.shortest_path(src, dst);

    // Assert
    assert!((path - 100.0).abs() < 0.01, "expected 100.0, got {path}");
}

/// @doc: When a `WrapWire` has one anchor, the shortest path must go from `src` to the
/// anchor position and then from the anchor to `dst`, forming a two-segment polyline. This
/// tests the fundamental anchor-routing logic: the cable bends around a single obstacle
/// vertex, and the total path length equals the sum of the two segment lengths. If this
/// case is wrong, multi-anchor paths (which are just chained single-anchor segments) will
/// also be wrong, breaking path distance calculations for any wrapped cable.
#[test]
fn when_wrap_wire_has_one_anchor_then_shortest_path_goes_through_it() {
    // Arrange
    let mut wire = WrapWire::new();
    wire.anchors.push(WrapAnchor {
        position: Vec2::new(50.0, 50.0),
        obstacle: Entity::from_raw(999),
        vertex_index: 0,
        boundary_step: 0,
        wrap_sign: 1.0,
    });
    let src = Vec2::new(0.0, 0.0);
    let dst = Vec2::new(100.0, 0.0);

    // Act
    let path = wire.shortest_path(src, dst);

    // Assert — src→(50,50)→dst = ~70.71 + ~70.71 = ~141.42
    let expected = (Vec2::new(50.0, 50.0) - src).length() + (dst - Vec2::new(50.0, 50.0)).length();
    assert!(
        (path - expected).abs() < 0.01,
        "expected {expected}, got {path}"
    );
}

// ---------------------------------------------------------------------------
// Segment intersection — crossing and parallel cases
// ---------------------------------------------------------------------------

/// @doc: `segment_intersects_segment` detects when two finite line segments cross each other
/// and returns the parametric `t` along the first segment at the intersection point. For two
/// perpendicular segments that cross at their midpoints, `t` must be 0.5. This is the
/// foundation of the wrap-detection system — without accurate segment–segment intersection,
/// the cable cannot detect when it crosses a polygon edge.
#[test]
fn when_two_crossing_segments_then_intersection_returns_some() {
    // Arrange — horizontal and vertical crossing segments
    let a1 = Vec2::new(0.0, 0.0);
    let a2 = Vec2::new(10.0, 0.0);
    let b1 = Vec2::new(5.0, -5.0);
    let b2 = Vec2::new(5.0, 5.0);

    // Act
    let result = segment_intersects_segment(a1, a2, b1, b2);

    // Assert
    assert!(result.is_some(), "crossing segments must intersect");
    let t = result.unwrap();
    assert!((t - 0.5).abs() < 0.01, "intersection at midpoint, t={t}");
}

/// @doc: Parallel segments can never intersect regardless of how close they are, so
/// `segment_intersects_segment` must return `None`. The denominator (perp-dot of the two
/// direction vectors) is zero for parallel lines, and the function must detect this
/// degenerate case rather than dividing by zero or returning a spurious intersection.
#[test]
fn when_parallel_segments_then_intersection_returns_none() {
    // Arrange
    let a1 = Vec2::new(0.0, 0.0);
    let a2 = Vec2::new(10.0, 0.0);
    let b1 = Vec2::new(0.0, 5.0);
    let b2 = Vec2::new(10.0, 5.0);

    // Act
    let result = segment_intersects_segment(a1, a2, b1, b2);

    // Assert
    assert!(result.is_none(), "parallel segments must not intersect");
}

// ---------------------------------------------------------------------------
// find_wrap_vertex — span crosses polygon
// ---------------------------------------------------------------------------

/// @doc: `find_wrap_vertex` identifies which vertex of a convex polygon a cable should wrap
/// around when the cable span crosses the polygon. For a horizontal cable at y=5 crossing a
/// box centered at (50, 0), the function must pick one of the top vertices (y=10) because
/// those are on the same side as the cable. The wrap sign indicates whether the cable bends
/// clockwise or counter-clockwise around that vertex.
#[test]
fn when_span_crosses_polygon_then_find_wrap_vertex_returns_correct_corner() {
    // Arrange — cable spans from left to right across a box centered at (50, 0)
    let span_a = Vec2::new(0.0, 5.0);
    let span_b = Vec2::new(100.0, 5.0);
    // Box vertices CCW: bottom-left, bottom-right, top-right, top-left
    let verts = vec![
        Vec2::new(40.0, -10.0),
        Vec2::new(60.0, -10.0),
        Vec2::new(60.0, 10.0),
        Vec2::new(40.0, 10.0),
    ];

    // Act
    let result = find_wrap_vertex(span_a, span_b, &verts);

    // Assert — should pick one of the top vertices (y=10 side, same side as the cable at y=5)
    assert!(result.is_some(), "must find a wrap vertex");
    let (idx, sign) = result.unwrap();
    let chosen = verts[idx];
    assert!(
        chosen.y > 0.0,
        "cable at y=5 should wrap around a top vertex, got {chosen}"
    );
    assert!(sign.abs() > 0.0, "wrap_sign must be nonzero");
}

// ---------------------------------------------------------------------------
// detect_wraps — inserts anchor when cable crosses polygon
// ---------------------------------------------------------------------------

/// @doc: `detect_wraps` walks each span of the cable path and checks for polygon intersections,
/// inserting `WrapAnchor`s where the cable crosses an obstacle. This test verifies the basic
/// contract: a straight cable that passes through a box gets exactly one anchor inserted.
#[test]
fn when_line_crosses_polygon_edge_then_detect_wraps_inserts_anchor() {
    // Arrange — cable from (0,5) to (100,5), box centered at (50,0)
    let mut wire = WrapWire::new();
    let src = Vec2::new(0.0, 5.0);
    let dst = Vec2::new(100.0, 5.0);
    let obstacle = Entity::from_raw(42);
    let verts = vec![
        Vec2::new(40.0, -10.0),
        Vec2::new(60.0, -10.0),
        Vec2::new(60.0, 10.0),
        Vec2::new(40.0, 10.0),
    ];
    let obstacles = vec![(obstacle, verts.as_slice())];

    // Act
    wire.detect_wraps(src, dst, &obstacles);

    // Assert — cable passes fully through the box, so it wraps at entry and exit corners
    assert_eq!(
        wire.anchors.len(),
        2,
        "cable through box needs entry + exit anchors"
    );
    assert_eq!(wire.anchors[0].obstacle, obstacle);
    assert_eq!(wire.anchors[1].obstacle, obstacle);
}

// ---------------------------------------------------------------------------
// detect_unwraps — removes anchor when cable swings past
// ---------------------------------------------------------------------------

/// @doc: `detect_unwraps` checks each anchor's cross product to determine if the cable has swung
/// past the wrap point. When the cross product sign no longer matches the wrap direction, the
/// anchor is removed. This test verifies that an anchor is correctly removed when the cable
/// endpoint moves to the opposite side.
#[test]
fn when_cable_swings_past_anchor_then_detect_unwraps_removes_it() {
    // Arrange — anchor at (50, 10) with CCW wrap
    let obstacle = Entity::from_raw(42);
    let mut wire = WrapWire::new();
    wire.anchors.push(WrapAnchor {
        position: Vec2::new(50.0, 10.0),
        obstacle,
        vertex_index: 2,
        boundary_step: 0,
        wrap_sign: 1.0,
    });

    let src = Vec2::new(0.0, 0.0);
    // dst on the opposite side — cable has swung past
    let dst = Vec2::new(100.0, -20.0);
    // Provide obstacle polygon around the anchor so the bypass check doesn't
    // short-circuit — we want to test the turn-reversal path.
    let verts = vec![
        Vec2::new(40.0, -5.0),
        Vec2::new(60.0, -5.0),
        Vec2::new(60.0, 15.0),
        Vec2::new(40.0, 15.0),
    ];
    let obstacles: Vec<(Entity, &[Vec2])> = vec![(obstacle, &verts)];

    // Act
    wire.detect_unwraps(src, dst, &obstacles);

    // Assert
    assert!(
        wire.anchors.is_empty(),
        "anchor must be removed when cable swings past"
    );
}

/// @doc: A cable that is still turning around the same side of a corner must not lose the
/// anchor just because the direct previous-to-next segment would bypass the obstacle. That
/// premature unwrap is what caused half-wrapped cables to phase through an obstacle and snap
/// to the opposite side.
#[test]
fn when_cable_can_bypass_obstacle_but_has_not_reversed_turn_then_detect_unwraps_keeps_anchor() {
    // Arrange
    let obstacle = Entity::from_raw(77);
    let mut wire = WrapWire::new();
    wire.anchors.push(WrapAnchor {
        position: Vec2::new(40.0, 10.0),
        obstacle,
        vertex_index: 3,
        boundary_step: 0,
        wrap_sign: 1.0,
    });

    let src = Vec2::new(0.0, 5.0);
    let dst = Vec2::new(100.0, 20.0);
    let verts = vec![
        Vec2::new(40.0, 0.0),
        Vec2::new(60.0, 0.0),
        Vec2::new(60.0, 10.0),
        Vec2::new(40.0, 10.0),
    ];
    let obstacles: Vec<(Entity, &[Vec2])> = vec![(obstacle, &verts)];

    // Act
    wire.detect_unwraps(src, dst, &obstacles);

    // Assert
    assert!(
        wire.anchors.len() == 1,
        "anchor must remain while the cable is still turning the same way"
    );
}

// ---------------------------------------------------------------------------
// detect_wraps — cable through a box creates anchors
// ---------------------------------------------------------------------------

/// @doc: A cable that passes straight through a box (entering one edge, exiting another) must
/// generate at least one wrap anchor. This complements the offset-cable test by checking the
/// head-on case where the cable's y-coordinate matches the box center.
#[test]
fn when_cable_passes_through_box_then_detect_wraps_creates_anchors() {
    let mut wire = WrapWire::new();
    let obstacle = Entity::from_raw(1);
    let verts = vec![
        Vec2::new(40.0, -10.0),
        Vec2::new(60.0, -10.0),
        Vec2::new(60.0, 10.0),
        Vec2::new(40.0, 10.0),
    ];
    let obstacles = vec![(obstacle, verts.as_slice())];

    wire.detect_wraps(Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), &obstacles);
    assert!(
        !wire.anchors.is_empty(),
        "cable through box must create anchors"
    );
}

// ---------------------------------------------------------------------------
// wrap_update_system + wrap_detect_system — ECS integration
// ---------------------------------------------------------------------------

#[test]
fn when_cable_dragged_across_obstacle_then_wrap_detect_system_adds_anchor() {
    // Arrange
    let mut world = World::new();

    let source = world
        .spawn(Transform2D {
            position: Vec2::new(0.0, 5.0),
            rotation: 0.0,
            scale: Vec2::ONE,
        })
        .id();
    let dest = world
        .spawn(Transform2D {
            position: Vec2::new(200.0, 5.0),
            rotation: 0.0,
            scale: Vec2::ONE,
        })
        .id();

    // Obstacle box centered at (100, 0)
    world.spawn((
        Transform2D {
            position: Vec2::new(100.0, 0.0),
            rotation: 0.0,
            scale: Vec2::ONE,
        },
        CableCollider::from_aabb(Vec2::new(20.0, 20.0)),
    ));

    let cable_entity = world
        .spawn((WireEndpoints { source, dest }, WrapWire::new()))
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems((wrap_update_system, wrap_detect_system).chain());

    // Act
    schedule.run(&mut world);

    // Assert
    let wrap = world.get::<WrapWire>(cable_entity).unwrap();
    assert!(
        !wrap.anchors.is_empty(),
        "wrap_detect_system must find the obstacle crossing"
    );
}

// ---------------------------------------------------------------------------
// Bug regression — detect_wraps allowed only one anchor per obstacle (Bug 1)
// ---------------------------------------------------------------------------

/// @doc: A cable that loops around two separate corners of the same obstacle box must receive
/// one `WrapAnchor` per corner vertex, not just one for the whole obstacle. The previous
/// implementation skipped an obstacle entirely once any of its vertices was already anchored,
/// making it impossible to wrap the cable around a second corner of the same box. This test
/// pre-inserts an anchor at the top-left corner, then places the destination far enough to
/// the right that the span from that corner to the destination crosses the box again at the
/// top-right corner, verifying that a second anchor is added at the different vertex.
#[test]
fn when_cable_requires_two_corners_of_same_obstacle_then_detect_wraps_inserts_two_anchors() {
    // Arrange — box with vertices at x=[30..70], y=[-20..20].
    // Pre-insert anchor at TL corner (30, 20), vertex index 3.
    // src is to the left of the box; dst is far to the right at y=20 (same height as TL).
    // The span from anchor TL (30,20) to dst (150,20) is horizontal and crosses the top
    // edge of the box — specifically the TR corner (70,20) — so a second anchor must be
    // added at vertex index 2 (TR).
    let mut wire = WrapWire::new();
    let obstacle = Entity::from_raw(7);
    // CCW vertices: BL, BR, TR, TL  (indices 0,1,2,3)
    let verts = vec![
        Vec2::new(30.0, -20.0), // 0 BL
        Vec2::new(70.0, -20.0), // 1 BR
        Vec2::new(70.0, 20.0),  // 2 TR
        Vec2::new(30.0, 20.0),  // 3 TL
    ];
    let obstacles = vec![(obstacle, verts.as_slice())];

    // Pre-insert the first anchor at the TL corner to simulate a partially-wrapped state.
    wire.anchors.push(WrapAnchor {
        position: Vec2::new(30.0, 20.0), // vertex index 3 (TL)
        obstacle,
        vertex_index: 3,
        boundary_step: -1,
        wrap_sign: 1.0,
    });

    // src is to the left; dst is far to the right at y=20.
    // Span layout: src(-10,20) → anchor-TL(30,20) → dst(150,20).
    // The span anchor-TL → dst(150,20) crosses the top edge of the box (TR corner at 70,20),
    // so detect_wraps must add a second anchor at vertex 2 (TR).
    let src = Vec2::new(-10.0, 20.0);
    let dst = Vec2::new(150.0, 20.0);

    // Act
    wire.detect_wraps(src, dst, &obstacles);

    // Assert — a second anchor must have been inserted alongside the pre-existing TL anchor
    assert!(
        wire.anchors.len() >= 2,
        "detect_wraps must add a second anchor at a different corner of the same obstacle; \
         anchors by vertex_index: {:?}",
        wire.anchors
            .iter()
            .map(|a| a.vertex_index)
            .collect::<Vec<_>>()
    );
    // The two anchors must reference different vertices on the same obstacle
    let indices: Vec<usize> = wire.anchors.iter().map(|a| a.vertex_index).collect();
    assert!(
        indices.windows(2).all(|w| w[0] != w[1]),
        "anchors on the same obstacle must reference different vertices, got {:?}",
        indices
    );
}

/// @doc: When a cable is already committed to one side of an obstacle, a new crossing on the
/// same obstacle must keep the same wrap direction instead of snapping to the opposite side.
/// Without this constraint, the anchor selection can jump from the top edge to the bottom edge
/// as soon as the free endpoint moves past the midpoint of the obstacle, which is the "51% wrap"
/// failure reported by the user.
#[test]
fn when_partial_wrap_crosses_same_obstacle_then_detect_wraps_keeps_same_side() {
    // Arrange — box with vertices at x=[30..70], y=[-20..20].
    // Pre-insert anchor at the top-left corner (TL), vertex index 3.
    let mut wire = WrapWire::new();
    let obstacle = Entity::from_raw(8);
    let verts = vec![
        Vec2::new(30.0, -20.0), // 0 BL
        Vec2::new(70.0, -20.0), // 1 BR
        Vec2::new(70.0, 20.0),  // 2 TR
        Vec2::new(30.0, 20.0),  // 3 TL
    ];
    let obstacles = vec![(obstacle, verts.as_slice())];

    wire.anchors.push(WrapAnchor {
        position: Vec2::new(30.0, 20.0),
        obstacle,
        vertex_index: 3,
        boundary_step: -1,
        wrap_sign: 1.0,
    });

    // This destination makes the span TL -> dst cross the right side close to the bottom,
    // which previously caused the algorithm to choose BR instead of staying on the top side.
    let src = Vec2::new(-10.0, 20.0);
    let dst = Vec2::new(150.0, -95.0);

    // Act
    wire.detect_wraps(src, dst, &obstacles);

    // Assert
    let indices: Vec<usize> = wire.anchors.iter().map(|a| a.vertex_index).collect();
    assert!(
        indices.contains(&2),
        "partial wrap must stay on the same side and choose TR, got {:?}",
        indices
    );
    assert!(
        !indices.contains(&1),
        "partial wrap must not flip to BR while the cable is still on the top side, got {:?}",
        indices
    );
}

// ---------------------------------------------------------------------------
// ST002 — detect_wraps inserts anchors on multiple obstacles in one call
// ---------------------------------------------------------------------------

/// @doc: When a cable span crosses two different obstacles, `detect_wraps` must insert
/// anchors for both in a single call rather than requiring one frame per obstacle. Without
/// this, a cable dragged across multiple objects takes N frames to fully wrap and visually
/// passes through intermediate obstacles during the delay.
#[test]
fn when_cable_crosses_two_different_obstacles_then_detect_wraps_anchors_both_in_one_call() {
    // Arrange — two separate boxes along the cable path
    let mut wire = WrapWire::new();
    let obstacle_a = Entity::from_raw(1);
    let obstacle_b = Entity::from_raw(2);
    let verts_a = vec![
        Vec2::new(30.0, -10.0),
        Vec2::new(50.0, -10.0),
        Vec2::new(50.0, 10.0),
        Vec2::new(30.0, 10.0),
    ];
    let verts_b = vec![
        Vec2::new(80.0, -10.0),
        Vec2::new(100.0, -10.0),
        Vec2::new(100.0, 10.0),
        Vec2::new(80.0, 10.0),
    ];
    let obstacles: Vec<(Entity, &[Vec2])> = vec![(obstacle_a, &verts_a), (obstacle_b, &verts_b)];

    let src = Vec2::new(0.0, 5.0);
    let dst = Vec2::new(130.0, 5.0);

    // Act
    wire.detect_wraps(src, dst, &obstacles);

    // Assert — must have anchors for both obstacles
    let anchored_obstacles: Vec<Entity> = wire.anchors.iter().map(|a| a.obstacle).collect();
    assert!(
        anchored_obstacles.contains(&obstacle_a),
        "must anchor obstacle_a; got {:?}",
        anchored_obstacles
    );
    assert!(
        anchored_obstacles.contains(&obstacle_b),
        "must anchor obstacle_b; got {:?}",
        anchored_obstacles
    );
}

// ---------------------------------------------------------------------------
// ST003 — unwrap hysteresis prevents flicker near zero cross product
// ---------------------------------------------------------------------------

/// @doc: An anchor whose cross product is very close to zero (cable nearly straight
/// through the anchor) must NOT be removed. Without hysteresis, near-zero cross products
/// cause the anchor to flicker between wrapped and unwrapped every frame, making the
/// cable appear to stick to corners.
#[test]
fn when_cross_product_near_zero_then_detect_unwraps_keeps_anchor() {
    // Arrange — anchor at (50, 10) with CCW wrap, cable nearly straight through it
    let obstacle = Entity::from_raw(42);
    let mut wire = WrapWire::new();
    wire.anchors.push(WrapAnchor {
        position: Vec2::new(50.0, 10.0),
        obstacle,
        vertex_index: 2,
        boundary_step: 0,
        wrap_sign: 1.0,
    });

    let src = Vec2::new(0.0, 10.0);
    // dst very slightly below the anchor line — cross product magnitude < threshold.
    // to_anchor = (50,0), from_anchor = (50, dst.y-10).
    // cross = 50 * (dst.y - 10), so dst.y = 9.98 → cross = -1.0 (inside ±2 threshold).
    let dst = Vec2::new(100.0, 9.98);
    let verts = vec![
        Vec2::new(40.0, 0.0),
        Vec2::new(60.0, 0.0),
        Vec2::new(60.0, 20.0),
        Vec2::new(40.0, 20.0),
    ];
    let obstacles: Vec<(Entity, &[Vec2])> = vec![(obstacle, &verts)];

    // Act
    wire.detect_unwraps(src, dst, &obstacles);

    // Assert — anchor must survive because cross product is near zero
    assert_eq!(
        wire.anchors.len(),
        1,
        "anchor must not be removed when cross product is near zero (hysteresis)"
    );
}

// ---------------------------------------------------------------------------
// wire_render_system — renders wire as a ribbon polygon
// ---------------------------------------------------------------------------

/// @doc: `wire_render_system` must read the `WireEndpoints` positions and optional `WrapWire`
/// anchors, then write a `ShapeVariant::Polygon` ribbon to the wire entity's `Shape` component.
/// The ribbon must be a non-empty polygon built from the polyline waypoints (src, anchors, dst).
/// Without this system, wires are invisible to the player.
#[test]
fn when_wire_has_endpoints_and_wrap_anchor_then_wire_render_system_produces_polygon() {
    // Arrange
    let mut world = World::new();

    let source = world
        .spawn(Transform2D {
            position: Vec2::new(0.0, 0.0),
            rotation: 0.0,
            scale: Vec2::ONE,
        })
        .id();
    let dest = world
        .spawn(Transform2D {
            position: Vec2::new(200.0, 0.0),
            rotation: 0.0,
            scale: Vec2::ONE,
        })
        .id();

    let mut wrap = WrapWire::new();
    wrap.anchors.push(WrapAnchor {
        position: Vec2::new(100.0, 50.0),
        obstacle: Entity::from_raw(999),
        vertex_index: 0,
        boundary_step: 0,
        wrap_sign: 1.0,
    });

    let wire_entity = world
        .spawn((
            WireEndpoints { source, dest },
            wrap,
            Transform2D::default(),
            Shape {
                variant: ShapeVariant::Polygon {
                    points: vec![Vec2::ZERO],
                },
                color: engine_core::color::Color::WHITE,
            },
            Visible(false),
            RenderLayer::World,
            SortOrder::default(),
        ))
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems(wire_render_system);

    // Act
    schedule.run(&mut world);

    // Assert — shape must be a non-empty Polygon
    let shape = world.get::<Shape>(wire_entity).unwrap();
    match &shape.variant {
        ShapeVariant::Polygon { points } => {
            assert!(
                points.len() > 6,
                "ribbon must have more vertices than 2*waypoints due to Catmull-Rom subdivision, got {}",
                points.len()
            );
        }
        other => panic!(
            "wire_render_system must write ShapeVariant::Polygon, got {:?}",
            std::mem::discriminant(other)
        ),
    }
    // wire_render_system must set visibility to true
    let visible = world.get::<Visible>(wire_entity).unwrap();
    assert!(visible.0, "wire_render_system must set Visible(true)");
}

// ---------------------------------------------------------------------------
// polyline_to_ribbon — builds ribbon from waypoints with subdivision
// ---------------------------------------------------------------------------

/// @doc: `polyline_to_ribbon` converts a polyline of waypoints into a ribbon polygon by
/// offsetting perpendicular to the local tangent. Catmull-Rom subdivision smooths the
/// corners. For N waypoints, the ribbon must have substantially more vertices than 2*N
/// due to the subdivision, and the result must be non-empty.
#[test]
fn when_three_waypoints_given_then_polyline_to_ribbon_produces_subdivided_polygon() {
    // Arrange — an L-shaped polyline with a bend
    let waypoints = vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(50.0, 50.0),
        Vec2::new(100.0, 0.0),
    ];
    let half_thickness = 1.5;

    // Act
    let ribbon = polyline_to_ribbon(&waypoints, half_thickness);

    // Assert — ribbon must be non-empty and have more vertices than 2*waypoints
    assert!(
        !ribbon.is_empty(),
        "ribbon must not be empty for 3 waypoints"
    );
    assert!(
        ribbon.len() > 2 * waypoints.len(),
        "ribbon must have more vertices than 2*N due to Catmull-Rom subdivision; \
         got {} for {} waypoints",
        ribbon.len(),
        waypoints.len()
    );
}

fn polyline_length(points: &[Vec2]) -> f32 {
    points
        .windows(2)
        .map(|pair| (pair[1] - pair[0]).length())
        .sum()
}

fn run_rope_solver(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(rope_solve_system);
    schedule.run(world);
}

#[test]
fn when_rope_point_starts_inside_obstacle_then_solver_projects_it_outside() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(DeltaTime(Seconds(1.0 / 60.0)));

    let source = world
        .spawn(Transform2D {
            position: Vec2::new(0.0, 0.0),
            rotation: 0.0,
            scale: Vec2::ONE,
        })
        .id();
    let dest = world
        .spawn(Transform2D {
            position: Vec2::new(160.0, 0.0),
            rotation: 0.0,
            scale: Vec2::ONE,
        })
        .id();

    let obstacle = vec![
        Vec2::new(60.0, -20.0),
        Vec2::new(100.0, -20.0),
        Vec2::new(100.0, 20.0),
        Vec2::new(60.0, 20.0),
    ];
    world.spawn((
        Transform2D {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
        },
        CableCollider {
            vertices: obstacle.clone(),
        },
    ));

    let mut rope = CableRope::new(Vec2::new(0.0, 0.0), Vec2::new(160.0, 0.0));
    rope.points = vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(80.0, 0.0),
        Vec2::new(160.0, 0.0),
    ];
    rope.previous_points = rope.points.clone();

    let cable = world
        .spawn((
            WireEndpoints { source, dest },
            rope,
            Transform2D::default(),
            Shape {
                variant: ShapeVariant::Polygon {
                    points: vec![Vec2::ZERO],
                },
                color: engine_core::color::Color::WHITE,
            },
            Visible(false),
            RenderLayer::World,
            SortOrder::default(),
        ))
        .id();

    // Act
    run_rope_solver(&mut world);

    // Assert
    let rope = world.get::<CableRope>(cable).unwrap();
    assert!(
        rope.points
            .iter()
            .skip(1)
            .take(rope.points.len().saturating_sub(2))
            .all(|point| !point_in_convex_polygon(*point, &obstacle)),
        "rope solver must keep interior points outside the obstacle"
    );
}

#[test]
fn when_rope_is_recomputed_many_times_then_point_count_stays_bounded() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(DeltaTime(Seconds(1.0 / 60.0)));

    let source = world
        .spawn(Transform2D {
            position: Vec2::new(0.0, 0.0),
            rotation: 0.0,
            scale: Vec2::ONE,
        })
        .id();
    let dest = world
        .spawn(Transform2D {
            position: Vec2::new(180.0, 0.0),
            rotation: 0.0,
            scale: Vec2::ONE,
        })
        .id();

    let obstacle = vec![
        Vec2::new(70.0, -20.0),
        Vec2::new(110.0, -20.0),
        Vec2::new(110.0, 20.0),
        Vec2::new(70.0, 20.0),
    ];
    world.spawn((
        Transform2D {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
        },
        CableCollider {
            vertices: obstacle.clone(),
        },
    ));

    let mut wrap = WrapWire::new();
    wrap.anchors.push(WrapAnchor {
        position: Vec2::new(70.0, 20.0),
        obstacle: Entity::from_raw(77),
        vertex_index: 3,
        boundary_step: -1,
        wrap_sign: 1.0,
    });

    let mut rope = CableRope::new(Vec2::new(0.0, 0.0), Vec2::new(180.0, 0.0));
    rope.points = (0..96)
        .map(|i| Vec2::new(i as f32 * 2.0, if i % 2 == 0 { 8.0 } else { -8.0 }))
        .collect();
    rope.previous_points = rope.points.clone();

    let cable = world
        .spawn((
            WireEndpoints { source, dest },
            wrap,
            rope,
            Transform2D::default(),
            Shape {
                variant: ShapeVariant::Polygon {
                    points: vec![Vec2::ZERO],
                },
                color: engine_core::color::Color::WHITE,
            },
            Visible(false),
            RenderLayer::World,
            SortOrder::default(),
        ))
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems((rope_solve_system, wire_render_system).chain());

    // Act
    for _ in 0..8 {
        schedule.run(&mut world);
    }

    // Assert
    let rope = world.get::<CableRope>(cable).unwrap();
    let length = polyline_length(&rope.points);
    assert!(
        rope.points.len() <= 24,
        "solver must collapse runaway rope history into a bounded sample set, got {} points",
        rope.points.len()
    );
    assert!(
        length > 180.0,
        "wrapped path should still be longer than the straight-line cable, got {length}"
    );
    assert!(
        rope.points
            .iter()
            .skip(1)
            .take(rope.points.len().saturating_sub(2))
            .all(|point| !point_in_convex_polygon(*point, &obstacle)),
        "rope solver must keep the bounded path outside the obstacle"
    );

    let shape = world.get::<Shape>(cable).unwrap();
    match &shape.variant {
        ShapeVariant::Polygon { points } => {
            assert!(
                points.len() > rope.points.len(),
                "rendered ribbon should still expand the sampled rope polyline"
            );
        }
        other => panic!(
            "wire_render_system must write ShapeVariant::Polygon, got {:?}",
            std::mem::discriminant(other)
        ),
    }
}
