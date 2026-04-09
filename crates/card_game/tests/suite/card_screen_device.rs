#![allow(clippy::unwrap_used)]

use std::sync::{Arc, Mutex};

use bevy_ecs::prelude::*;
use card_game::card::identity::signature::{CardSignature, Element};
use card_game::card::jack_cable::{Cable, Jack, JackDirection, signature_space_propagation_system};
use card_game::card::reader::{SIGNATURE_SPACE_RADIUS, SignatureSpace};
use card_game::card::screen_device::{display_axes, screen_render_system, spawn_screen_device};
use engine_core::prelude::{Color, Transform2D};
use engine_render::prelude::{Camera2D, RendererRes};
use engine_render::testing::{ShapeCallLog, SpyRenderer};
use engine_scene::prelude::{
    hierarchy_maintenance_system, transform_propagation_system, visibility_system,
};
use engine_ui::draw_command::DrawQueue;
use engine_ui::unified_render::unified_render_system;
use glam::Vec2;

// ---------------------------------------------------------------------------
// Shared helper
// ---------------------------------------------------------------------------

fn make_screen_world(jack_data: Option<SignatureSpace>) -> (World, ShapeCallLog) {
    let mut world = World::new();
    let (_device_entity, jack_entity) = spawn_screen_device(&mut world, Vec2::ZERO);
    world
        .get_mut::<Jack<SignatureSpace>>(jack_entity)
        .unwrap()
        .data = jack_data;

    let shape_calls: ShapeCallLog = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(Arc::new(Mutex::new(Vec::new())))
        .with_shape_capture(shape_calls.clone())
        .with_viewport(1024, 768);
    world.insert_resource(RendererRes::new(Box::new(spy)));
    world.insert_resource(DrawQueue::default());
    world.spawn(Camera2D {
        position: Vec2::ZERO,
        zoom: 1.0,
    });

    (world, shape_calls)
}

fn run_screen_visuals(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(
        (
            hierarchy_maintenance_system,
            screen_render_system,
            transform_propagation_system,
            visibility_system,
            unified_render_system,
        )
            .chain(),
    );
    schedule.run(world);
}

// ---------------------------------------------------------------------------
// TC010 / TC011 — axis-pair extraction
// ---------------------------------------------------------------------------

/// @doc: Display 0 of the `ScreenDevice` maps to the Solidum/Febris element pair — the first two
/// dimensions of the 8D signature space. If `display_axes` returned the wrong pair (e.g. indices 2
/// and 3), a card hovering near the origin in Solidum/Febris space would appear on the wrong
/// panel, breaking the visual correspondence between the card's elemental identity and the display
/// that lights up. Verifying exact f32 values against the named Element variants catches any
/// index-offset bug at the axis-extraction layer before it reaches the rendering pipeline.
#[test]
fn when_display_index_is_zero_then_axes_map_to_solidum_and_febris() {
    // Arrange
    let center = CardSignature::new([0.3, 0.7, 0.1, 0.2, 0.4, 0.5, 0.6, 0.8]);
    let space = SignatureSpace::from_single(center, SIGNATURE_SPACE_RADIUS, Entity::from_raw(0));

    // Act
    let (x, y) = display_axes(&space, 0);

    // Assert
    assert_eq!(x, space.control_points[0][Element::Solidum]);
    assert_eq!(y, space.control_points[0][Element::Febris]);
}

/// @doc: Display 3 maps to the Subsidium/Spatium element pair — the last two dimensions of the
/// 8D signature space. Testing the final panel alongside the first closes the bounds: if the
/// index formula `display_index * 2` is off by one in either direction, at least one of these
/// boundary tests will produce a wrong axis, catching fencepost errors that a single mid-range
/// test would miss. Together TC010 and TC011 bracket all four display panels with minimal
/// redundancy.
#[test]
fn when_display_index_is_three_then_axes_map_to_subsidium_and_spatium() {
    // Arrange
    let center = CardSignature::new([0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.9, -0.8]);
    let space = SignatureSpace::from_single(center, SIGNATURE_SPACE_RADIUS, Entity::from_raw(0));

    // Act
    let (x, y) = display_axes(&space, 3);

    // Assert
    assert_eq!(x, space.control_points[0][Element::Subsidium]);
    assert_eq!(y, space.control_points[0][Element::Spatium]);
}

// ---------------------------------------------------------------------------
// TC012 — no draws when input jack has None
// ---------------------------------------------------------------------------

/// @doc: A `ScreenDevice` with no signal must still draw its chassis, stroke, socket, and
/// four panel backgrounds while hiding the four signal dots. Without this panel-only state
/// the display would vanish until a card is inserted, giving no indication that a connectable
/// device exists on the table.
#[test]
fn given_input_jack_data_is_none_when_screen_render_system_runs_then_only_panel_backgrounds_drawn()
{
    // Arrange
    let (mut world, shape_calls) = make_screen_world(None);

    // Act
    run_screen_visuals(&mut world);

    // Assert — body fill + body stroke + socket + 4 panels = 7 draw calls
    assert_eq!(
        shape_calls.lock().unwrap().len(),
        7,
        "disconnected screen must draw body, stroke, socket, and 4 panels with no signal dots"
    );
}

// ---------------------------------------------------------------------------
// TC013 — 4 draws when input jack has Some(SignatureSpace)
// ---------------------------------------------------------------------------

/// @doc: When a `SignatureSpace` arrives on the input jack, the screen must render the same
/// chassis/panel base plus 4 signal dots — 11 draw calls total including the body stroke and
/// socket. Fewer would leave some displays dark; more would duplicate visuals or invent
/// phantom signal markers.
#[test]
fn given_input_jack_has_signature_space_when_screen_render_system_runs_then_draws_eight_shapes() {
    // Arrange
    let signal = SignatureSpace::from_single(
        CardSignature::new([0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8]),
        SIGNATURE_SPACE_RADIUS,
        Entity::from_raw(0),
    );
    let (mut world, shape_calls) = make_screen_world(Some(signal));

    // Act
    run_screen_visuals(&mut world);

    // Assert — body fill + body stroke + socket + 4 panels + 4 dots
    assert_eq!(
        shape_calls.lock().unwrap().len(),
        11,
        "screen with active signal must draw body, stroke, socket, 4 panels, and 4 signal dots"
    );
}

// ---------------------------------------------------------------------------
// TC014 — full signal chain integration
// ---------------------------------------------------------------------------

/// @doc: This integration test validates the complete wiring chain from card insertion
/// to screen display: a card dropped into a reader populates the reader's output jack with
/// a `SignatureSpace`, the cable propagation system transfers that data to the screen's input
/// jack, and the screen render system draws the correct number of signal dots. If any link
/// in this chain is missing — the insert system doesn't write the jack, the propagation
/// system doesn't copy it, or the render system doesn't read it — the player sees a blank
/// screen despite a card being loaded, which would make the wiring feature appear broken.
#[test]
fn when_card_inserted_and_cable_propagated_then_screen_input_jack_holds_signature_space() {
    // Arrange — reader output jack
    let mut world = World::new();
    let sig = CardSignature::new([0.2, 0.4, -0.3, 0.7, 0.1, -0.5, 0.6, -0.2]);

    let reader_output_jack = world
        .spawn((
            Jack::<SignatureSpace> {
                direction: JackDirection::Output,
                data: None,
            },
            Transform2D::default(),
        ))
        .id();

    // Arrange — screen input jack
    let screen_input_jack = world
        .spawn((
            Jack::<SignatureSpace> {
                direction: JackDirection::Input,
                data: None,
            },
            Transform2D::default(),
        ))
        .id();

    // Arrange — cable connecting reader to screen
    world.spawn(Cable {
        source: reader_output_jack,
        dest: screen_input_jack,
    });

    // Manually populate the reader's output jack (simulates card_reader_insert_system)
    world
        .entity_mut(reader_output_jack)
        .get_mut::<Jack<SignatureSpace>>()
        .unwrap()
        .data = Some(SignatureSpace::from_single(
        sig,
        SIGNATURE_SPACE_RADIUS,
        Entity::from_raw(0),
    ));

    // Act — run propagation
    let mut schedule = Schedule::default();
    schedule.add_systems(signature_space_propagation_system);
    schedule.run(&mut world);

    // Assert — screen input jack now holds the SignatureSpace
    let jack = world
        .get::<Jack<SignatureSpace>>(screen_input_jack)
        .unwrap();
    assert_eq!(
        jack.data,
        Some(SignatureSpace::from_single(
            sig,
            SIGNATURE_SPACE_RADIUS,
            Entity::from_raw(0)
        )),
        "screen input jack must receive the SignatureSpace after cable propagation"
    );
}

// ---------------------------------------------------------------------------
// TC015 — multi-point signal rendering
// ---------------------------------------------------------------------------

#[test]
fn when_signal_has_two_control_points_then_screen_draws_signal_shapes() {
    // Arrange
    let sig_a = CardSignature::new([0.3, 0.7, 0.1, 0.2, 0.4, 0.5, 0.6, 0.8]);
    let sig_b = CardSignature::new([-0.3, 0.2, 0.5, -0.1, 0.3, -0.4, 0.7, 0.1]);
    let combined = SignatureSpace::combine(
        &SignatureSpace::from_single(sig_a, 0.2, Entity::from_raw(0)),
        &SignatureSpace::from_single(sig_b, 0.2, Entity::from_raw(1)),
    );
    let (mut world, shape_calls) = make_screen_world(Some(combined));

    // Act
    run_screen_visuals(&mut world);

    // Assert — body fill + body stroke + socket + 4 panels + 4 signal shapes = 11
    let calls = shape_calls.lock().unwrap();
    assert_eq!(
        calls.len(),
        11,
        "screen with 2-point signal must draw body, stroke, socket, 4 panels, and 4 signal shapes"
    );
}

// ---------------------------------------------------------------------------
// TC016 — 2-point signal capsule geometry
// ---------------------------------------------------------------------------

const EXPECTED_SIGNAL_COLOR: Color = Color {
    r: 0.4,
    g: 0.9,
    b: 0.4,
    a: 1.0,
};

/// @doc: A two-control-point signal must render as a capsule with semicircular endcap fans, not
/// as a bare 4-corner rectangle. Without curved endcaps the signal shape is visually
/// indistinguishable from a unanimated panel background, losing all directional cue about where
/// the signal is pointing in 2D element-space. This test guards the endcap implementation by
/// demanding that every signal draw call contains substantially more tessellated vertices than
/// a degenerate quad — if any panel regresses to a 4-vertex rectangle, the minimum across all
/// panels will fall to 4 and the assertion fails.
#[test]
fn when_two_point_signal_rendered_then_capsule_has_more_vertices_than_plain_rectangle() {
    // Arrange
    let sig_a = CardSignature::new([0.3, 0.7, 0.1, 0.2, 0.4, 0.5, 0.6, 0.8]);
    let sig_b = CardSignature::new([-0.3, 0.2, 0.5, -0.1, 0.3, -0.4, 0.7, 0.1]);
    let combined = SignatureSpace::combine(
        &SignatureSpace::from_single(sig_a, 0.2, Entity::from_raw(0)),
        &SignatureSpace::from_single(sig_b, 0.2, Entity::from_raw(1)),
    );
    let (mut world, shape_calls) = make_screen_world(Some(combined));

    // Act
    run_screen_visuals(&mut world);

    // Assert — all four signal panels must have capsule geometry, not a minimal rectangle
    let calls = shape_calls.lock().unwrap();
    let signal_vertex_counts: Vec<usize> = calls
        .iter()
        .filter(|(_, _, color, _)| *color == EXPECTED_SIGNAL_COLOR)
        .map(|(vertices, _, _, _)| vertices.len())
        .collect();
    assert_eq!(
        signal_vertex_counts.len(),
        4,
        "expected 4 signal draw calls (one per display panel)"
    );
    let min_vertices = signal_vertex_counts.into_iter().min().unwrap();
    assert!(
        min_vertices > 10,
        "each signal panel must tessellate to a capsule (> 10 vertices), got {min_vertices} — \
         a plain rectangle produces only 4"
    );
}

// ---------------------------------------------------------------------------
// TC017 — capsule clipping near panel edge
// ---------------------------------------------------------------------------

/// @doc: A capsule whose endpoint projects to Solidum axis 0.9 (pixel x = 45.0) has a
/// semicircular endcap with radius ~10 px, placing raw vertices as far as x = 55.0 — 5 px
/// outside the ±50 px panel boundary. The Sutherland-Hodgman clipper in
/// `build_signal_polyline` must trim every such vertex back to the panel rectangle before
/// the shape reaches the GPU. Verifying all four panels and both axes ensures a regression
/// in any one clip-plane pass is caught immediately.
#[test]
fn when_two_point_signal_projects_near_panel_edge_then_capsule_vertices_stay_within_panel_bounds() {
    const EPSILON: f32 = 0.01;

    // Arrange — one endpoint near the Solidum boundary (axis value 0.9 → pixel x = 45.0),
    // with a signal radius of 0.2 so the endcap fan extends ~10 px beyond that.
    let sig_a = CardSignature::new([0.9, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    let sig_b = CardSignature::new([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    let combined = SignatureSpace::combine(
        &SignatureSpace::from_single(sig_a, 0.2, Entity::from_raw(0)),
        &SignatureSpace::from_single(sig_b, 0.2, Entity::from_raw(1)),
    );
    let (mut world, shape_calls) = make_screen_world(Some(combined));

    // Act
    run_screen_visuals(&mut world);

    // Assert — every vertex of every signal draw call must lie within the panel rectangle
    let calls = shape_calls.lock().unwrap();
    let signal_calls: Vec<_> = calls
        .iter()
        .filter(|(_, _, color, _)| *color == EXPECTED_SIGNAL_COLOR)
        .collect();
    assert_eq!(
        signal_calls.len(),
        4,
        "expected 4 signal draw calls (one per display panel)"
    );
    for (panel_idx, (vertices, _, _, _)) in signal_calls.iter().enumerate() {
        for v in vertices {
            assert!(
                v[0].abs() <= 50.0 + EPSILON,
                "panel {panel_idx}: vertex x={} exceeds panel half-width 50.0",
                v[0]
            );
            assert!(
                v[1].abs() <= 50.0 + EPSILON,
                "panel {panel_idx}: vertex y={} exceeds panel half-height 50.0",
                v[1]
            );
        }
    }
}

// ---------------------------------------------------------------------------
// TC018 — 3-point spline vertex density
// ---------------------------------------------------------------------------

/// @doc: A three-control-point closed-loop signal must be rendered through Catmull-Rom
/// subdivision, producing a densely sampled annular polygon rather than the raw 8-vertex
/// annular ring the unsubdivided path gives. Without subdivision, the signal shape on a
/// screen panel looks like a sharp-cornered triangle ring instead of a smooth organic
/// curve, which defeats the purpose of the display — visualising how a combined signal
/// flows through element space.
#[test]
fn when_three_point_signal_rendered_then_spline_polygon_has_many_more_vertices_than_unsubdivided_annular_ring()
 {
    // Arrange — three distinct signatures with varied values across all 8 axes so every
    // panel gets a non-degenerate triangle projection.
    let sig_a = CardSignature::new([0.2, 0.2, 0.3, 0.1, -0.2, 0.4, 0.1, -0.3]);
    let sig_b = CardSignature::new([-0.3, 0.4, -0.1, 0.3, 0.3, -0.2, 0.4, 0.2]);
    let sig_c = CardSignature::new([0.1, -0.3, 0.2, -0.4, 0.1, 0.3, -0.3, 0.1]);
    let three_point = SignatureSpace::combine(
        &SignatureSpace::combine(
            &SignatureSpace::from_single(sig_a, 0.2, Entity::from_raw(0)),
            &SignatureSpace::from_single(sig_b, 0.2, Entity::from_raw(1)),
        ),
        &SignatureSpace::from_single(sig_c, 0.2, Entity::from_raw(2)),
    );
    assert_eq!(
        three_point.control_points.len(),
        3,
        "test setup: must have exactly 3 control points"
    );
    let (mut world, shape_calls) = make_screen_world(Some(three_point));

    // Act
    run_screen_visuals(&mut world);

    // Assert — subdivided spline must produce many more vertices than the 8-vertex annular ring
    let calls = shape_calls.lock().unwrap();
    let signal_vertex_counts: Vec<usize> = calls
        .iter()
        .filter(|(_, _, color, _)| *color == EXPECTED_SIGNAL_COLOR)
        .map(|(vertices, _, _, _)| vertices.len())
        .collect();
    assert_eq!(
        signal_vertex_counts.len(),
        4,
        "expected 4 signal draw calls (one per display panel)"
    );
    let min_vertices = signal_vertex_counts.into_iter().min().unwrap();
    assert!(
        min_vertices > 12,
        "spline subdivision must densify the annular ring (> 12 vertices), got {min_vertices} — \
         the unsubdivided 3-point ring produces ~8 vertices"
    );
}

// ---------------------------------------------------------------------------
// TC019 — 3-point spline clipping near panel corner
// ---------------------------------------------------------------------------

/// @doc: A closed-loop spline whose outer annular ring overshoots the ±50 px panel boundary
/// must be clipped back before reaching the GPU. Placing one control point at (0.85, 0.85) in
/// normalised element space puts it at pixel (42.5, 42.5); the annular band's outer ring at
/// that vertex extends diagonally to ~(52, 52). Without clipping after subdivision, the spline
/// would draw outside the panel background, visually bleeding into adjacent panels or the
/// device chassis.
#[test]
fn when_three_point_signal_projects_near_panel_corner_then_spline_loop_vertices_stay_within_panel_bounds()
 {
    const EPSILON: f32 = 0.01;

    // Arrange — one point near the corner of the Solidum/Febris panel
    let sig_a = CardSignature::new([0.85, 0.85, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1]);
    let sig_b = CardSignature::new([-0.2, 0.1, -0.3, 0.2, -0.1, 0.3, -0.2, 0.2]);
    let sig_c = CardSignature::new([0.1, -0.4, 0.2, -0.1, 0.3, -0.2, 0.1, -0.3]);
    let three_point = SignatureSpace::combine(
        &SignatureSpace::combine(
            &SignatureSpace::from_single(sig_a, 0.2, Entity::from_raw(0)),
            &SignatureSpace::from_single(sig_b, 0.2, Entity::from_raw(1)),
        ),
        &SignatureSpace::from_single(sig_c, 0.2, Entity::from_raw(2)),
    );
    assert_eq!(
        three_point.control_points.len(),
        3,
        "test setup: must have exactly 3 control points"
    );
    let (mut world, shape_calls) = make_screen_world(Some(three_point));

    // Act
    run_screen_visuals(&mut world);

    // Assert — every vertex must stay inside the panel rectangle
    let calls = shape_calls.lock().unwrap();
    let signal_calls: Vec<_> = calls
        .iter()
        .filter(|(_, _, color, _)| *color == EXPECTED_SIGNAL_COLOR)
        .collect();
    assert_eq!(
        signal_calls.len(),
        4,
        "expected 4 signal draw calls (one per display panel)"
    );
    for (panel_idx, (vertices, _, _, _)) in signal_calls.iter().enumerate() {
        for v in vertices {
            assert!(
                v[0].abs() <= 50.0 + EPSILON,
                "panel {panel_idx}: vertex x={} exceeds panel bounds",
                v[0]
            );
            assert!(
                v[1].abs() <= 50.0 + EPSILON,
                "panel {panel_idx}: vertex y={} exceeds panel bounds",
                v[1]
            );
        }
    }
}

// ---------------------------------------------------------------------------
// TC020 — spline density scales with control point count
// ---------------------------------------------------------------------------

/// @doc: The Catmull-Rom subdivision produces `n_segments × subdivisions_per_segment` samples per
/// ring, so a 4-point closed loop must yield a denser polygon than a 3-point loop. If the
/// subdivision count were hardcoded (e.g., always 24 samples total regardless of segment count),
/// adding more control points to a signal would not increase curve resolution, making complex
/// combined signals look just as angular as simple ones. This comparative test catches that
/// regression by running both worlds through the same pipeline and asserting the vertex count
/// scales upward.
#[test]
fn when_four_point_signal_rendered_then_spline_is_denser_than_three_point_spline() {
    // Arrange — 3-point signal
    let sig_a = CardSignature::new([0.2, 0.2, 0.3, 0.1, -0.2, 0.4, 0.1, -0.3]);
    let sig_b = CardSignature::new([-0.3, 0.4, -0.1, 0.3, 0.3, -0.2, 0.4, 0.2]);
    let sig_c = CardSignature::new([0.1, -0.3, 0.2, -0.4, 0.1, 0.3, -0.3, 0.1]);
    let three_point = SignatureSpace::combine(
        &SignatureSpace::combine(
            &SignatureSpace::from_single(sig_a, 0.2, Entity::from_raw(0)),
            &SignatureSpace::from_single(sig_b, 0.2, Entity::from_raw(1)),
        ),
        &SignatureSpace::from_single(sig_c, 0.2, Entity::from_raw(2)),
    );
    assert_eq!(three_point.control_points.len(), 3);
    let (mut world_3, shape_calls_3) = make_screen_world(Some(three_point));

    // Arrange — 4-point signal (extend the 3-point set with a 4th distinct point)
    let sig_d = CardSignature::new([-0.1, -0.2, -0.3, 0.5, -0.4, 0.1, 0.2, 0.4]);
    let four_point = SignatureSpace::combine(
        &SignatureSpace::combine(
            &SignatureSpace::combine(
                &SignatureSpace::from_single(sig_a, 0.2, Entity::from_raw(0)),
                &SignatureSpace::from_single(sig_b, 0.2, Entity::from_raw(1)),
            ),
            &SignatureSpace::from_single(sig_c, 0.2, Entity::from_raw(2)),
        ),
        &SignatureSpace::from_single(sig_d, 0.2, Entity::from_raw(3)),
    );
    assert_eq!(four_point.control_points.len(), 4);
    let (mut world_4, shape_calls_4) = make_screen_world(Some(four_point));

    // Act
    run_screen_visuals(&mut world_3);
    run_screen_visuals(&mut world_4);

    // Assert — 4-point spline must be denser than 3-point on the same panel
    let calls_3 = shape_calls_3.lock().unwrap();
    let calls_4 = shape_calls_4.lock().unwrap();
    let verts_3: usize = calls_3
        .iter()
        .filter(|(_, _, color, _)| *color == EXPECTED_SIGNAL_COLOR)
        .map(|(v, _, _, _)| v.len())
        .max()
        .expect("3-point signal must produce draw calls");
    let verts_4: usize = calls_4
        .iter()
        .filter(|(_, _, color, _)| *color == EXPECTED_SIGNAL_COLOR)
        .map(|(v, _, _, _)| v.len())
        .max()
        .expect("4-point signal must produce draw calls");
    assert!(
        verts_4 > verts_3,
        "4-point spline ({verts_4} vertices) must be denser than 3-point spline ({verts_3} vertices)"
    );
}
