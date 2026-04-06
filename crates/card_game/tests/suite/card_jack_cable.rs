#![allow(clippy::unwrap_used)]

use std::sync::{Arc, Mutex};

use bevy_ecs::prelude::*;
use card_game::card::identity::signature::CardSignature;
use card_game::card::jack_cable::{
    Cable, CableCollider, Jack, JackDirection, RopeParticle, RopeWire, RopeWireEndpoints,
    WrapAnchor, WrapWire, cable_render_system, particles_to_bezier_path, rope_physics_system,
    rope_render_system, signature_space_propagation_system, wrap_detect_system, wrap_update_system,
};
use card_game::card::reader::{SIGNATURE_SPACE_RADIUS, SignatureSpace};
use engine_core::color::Color;
use engine_core::prelude::Transform2D;
use engine_render::prelude::{Camera2D, RendererRes, Shape, ShapeVariant};
use engine_render::shape::PathCommand;
use engine_render::testing::{ShapeCallLog, SpyRenderer};
use engine_scene::prelude::{SortOrder, Visible, transform_propagation_system, visibility_system};
use engine_scene::render_order::RenderLayer;
use engine_ui::unified_render::unified_render_system;
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
// TC015 / TC016 — cable_render_system line drawing
// ---------------------------------------------------------------------------

fn make_cable_render_world() -> (World, ShapeCallLog) {
    let mut world = World::new();
    let shape_calls: ShapeCallLog = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(Arc::new(Mutex::new(Vec::new())))
        .with_shape_capture(shape_calls.clone())
        .with_viewport(1024, 768);
    world.insert_resource(RendererRes::new(Box::new(spy)));
    world.spawn(Camera2D {
        position: Vec2::ZERO,
        zoom: 1.0,
    });
    (world, shape_calls)
}

fn spawn_renderable_cable(world: &mut World, source: Entity, dest: Entity) {
    world.spawn((
        Cable { source, dest },
        Transform2D::default(),
        Shape {
            variant: ShapeVariant::Polygon {
                points: vec![
                    Vec2::new(-1.0, -1.0),
                    Vec2::new(1.0, -1.0),
                    Vec2::new(1.0, 1.0),
                    Vec2::new(-1.0, 1.0),
                ],
            },
            color: Color::WHITE,
        },
        Visible(true),
        RenderLayer::World,
        SortOrder::default(),
    ));
}

fn run_cable_visuals(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(
        (
            cable_render_system,
            transform_propagation_system,
            visibility_system,
            unified_render_system,
        )
            .chain(),
    );
    schedule.run(world);
}

/// @doc: `cable_render_system` must update the cable entity so the unified shape renderer
/// draws one quad between the world positions of its two endpoint jacks. Without this visual
/// the wiring layer is invisible and a player has no feedback that a reader is connected
/// to a screen.
#[test]
fn when_cable_connects_two_positioned_entities_then_one_line_shape_is_drawn() {
    // Arrange
    let (mut world, shape_calls) = make_cable_render_world();

    let source = world
        .spawn(Transform2D {
            position: Vec2::new(0.0, 0.0),
            rotation: 0.0,
            scale: Vec2::ONE,
        })
        .id();
    let dest = world
        .spawn(Transform2D {
            position: Vec2::new(200.0, 100.0),
            rotation: 0.0,
            scale: Vec2::ONE,
        })
        .id();
    spawn_renderable_cable(&mut world, source, dest);

    // Act
    run_cable_visuals(&mut world);

    // Assert
    assert_eq!(
        shape_calls.lock().unwrap().len(),
        1,
        "one Cable entity must produce exactly one line draw call"
    );
}

/// @doc: `cable_render_system` must reposition the cable's transform to the midpoint of its
/// two endpoint entities. If the system does nothing, the cable transform stays at its initial
/// default position and renders in the wrong place regardless of where the endpoints are.
#[test]
fn when_cable_render_system_runs_then_cable_transform_is_at_midpoint_of_endpoints() {
    // Arrange
    let (mut world, _) = make_cable_render_world();

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
    let cable_entity = world
        .spawn((
            Cable { source, dest },
            Transform2D::default(), // starts at origin — system must move it to midpoint
            Shape {
                variant: ShapeVariant::Polygon {
                    points: vec![
                        Vec2::new(-1.0, -1.0),
                        Vec2::new(1.0, -1.0),
                        Vec2::new(1.0, 1.0),
                        Vec2::new(-1.0, 1.0),
                    ],
                },
                color: Color::WHITE,
            },
            Visible(true),
            RenderLayer::World,
            SortOrder::default(),
        ))
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems(cable_render_system);

    // Act
    schedule.run(&mut world);

    // Assert — transform must be at midpoint (100, 0), not at the pre-spawn default (0, 0)
    let transform = world.get::<Transform2D>(cable_entity).unwrap();
    assert!(
        (transform.position.x - 100.0).abs() < 0.01,
        "cable transform x should be at midpoint 100.0, got {}",
        transform.position.x
    );
    assert!(
        (transform.position.y - 0.0).abs() < 0.01,
        "cable transform y should be at midpoint 0.0, got {}",
        transform.position.y
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
// Rope wire — TC001: construction with evenly-spaced particles
// ---------------------------------------------------------------------------

/// @doc: A `RopeWire` models the slack cable that hangs between two jack sockets. Particles
/// must be seeded at evenly-spaced positions along the straight line from endpoint A to B so
/// that the Verlet integrator starts from a physically plausible rest pose. If particles were
/// clumped at one end, the first simulation tick would produce a violent jerk that tears the
/// rope apart or causes tunnelling through card geometry. The `prev == pos` invariant encodes
/// zero initial velocity — any divergence would inject phantom momentum that the solver cannot
/// distinguish from real user input.
#[test]
fn given_two_endpoints_and_particle_count_when_rope_wire_constructed_then_particles_linearly_interpolated_with_zero_velocity()
 {
    // Arrange
    let a = Vec2::new(0.0, 0.0);
    let b = Vec2::new(9.0, 0.0);
    let n = 4_usize; // particles at x = 0, 3, 6, 9

    // Act
    let rope = RopeWire::new(a, b, n);

    // Assert
    assert_eq!(rope.particles.len(), n);
    for (i, particle) in rope.particles.iter().enumerate() {
        let t = i as f32 / (n - 1) as f32;
        let expected = a.lerp(b, t);
        assert!(
            (particle.pos - expected).length() < 1e-5,
            "particle[{i}].pos expected {expected}, got {}",
            particle.pos
        );
        assert!(
            (particle.prev - particle.pos).length() < 1e-5,
            "particle[{i}].prev should equal pos (zero velocity), got prev={} pos={}",
            particle.prev,
            particle.pos
        );
    }
}

// ---------------------------------------------------------------------------
// Rope wire — TC002–TC004: verlet_step integration
// ---------------------------------------------------------------------------

/// @doc: A `RopeWire` in which every particle's `prev` equals its `pos` encodes zero velocity.
/// Applying one Verlet step must leave all positions unchanged because the displacement vector
/// `pos - prev` is zero for every particle. If `verlet_step` moves any particle when there is
/// no momentum, the rope would spontaneously accelerate from rest, making it impossible to
/// author a hanging rope that stays put before user interaction.
#[test]
fn when_all_particles_have_zero_velocity_then_verlet_step_leaves_positions_unchanged() {
    // Arrange
    let mut wire = RopeWire::new(Vec2::new(0.0, 0.0), Vec2::new(30.0, 0.0), 4);
    // Capture positions before the step (prev == pos already by construction)
    let positions_before: Vec<Vec2> = wire.particles.iter().map(|p| p.pos).collect();

    // Act
    wire.verlet_step(1.0);

    // Assert
    for (i, (particle, before)) in wire
        .particles
        .iter()
        .zip(positions_before.iter())
        .enumerate()
    {
        assert!(
            (particle.pos - *before).length() < 1e-5,
            "particle[{i}].pos should be unchanged when velocity is zero, \
             expected {before}, got {}",
            particle.pos
        );
    }
}

/// @doc: A single particle with `pos=(10,0)` and `prev=(0,0)` carries a displacement of
/// `(10,0)`. With damping=1.0, Verlet integration must produce `new_pos = pos + (pos - prev)
/// * 1.0 = (20,0)`. This is the canonical unit test for the integration formula itself: if
/// the arithmetic is wrong (e.g. velocity is subtracted instead of added, or prev is not
/// updated) the particle ends up at the wrong position and every rope physics interaction
/// produces systematically incorrect trajectories.
#[test]
fn when_single_particle_has_nonzero_velocity_and_damping_one_then_verlet_step_adds_displacement() {
    // Arrange
    let mut wire = RopeWire::with_particles(vec![RopeParticle {
        pos: Vec2::new(10.0, 0.0),
        prev: Vec2::new(0.0, 0.0),
    }]);

    // Act
    wire.verlet_step(1.0);

    // Assert
    let new_pos = wire.particles[0].pos;
    assert!(
        (new_pos - Vec2::new(20.0, 0.0)).length() < 1e-5,
        "verlet_step with damping=1.0 must yield pos=(20,0), got {new_pos}"
    );
}

/// @doc: Damping values below 1.0 must attenuate the displacement each tick, causing the rope
/// to decelerate realistically. With `pos=(10,0)`, `prev=(0,0)` and damping=0.9 the expected
/// result is `pos + (pos - prev) * 0.9 = (10,0) + (10,0)*0.9 = (19,0)`. If `verlet_step`
/// ignores the damping coefficient the rope will never lose energy and a freely-hanging rope
/// would oscillate indefinitely instead of settling. Testing damping=0.9 specifically catches
/// the common bug of multiplying the wrong term or applying damping after the position update.
#[test]
fn when_single_particle_has_nonzero_velocity_and_damping_point_nine_then_verlet_step_attenuates_displacement()
 {
    // Arrange
    let mut wire = RopeWire::with_particles(vec![RopeParticle {
        pos: Vec2::new(10.0, 0.0),
        prev: Vec2::new(0.0, 0.0),
    }]);

    // Act
    wire.verlet_step(0.9);

    // Assert
    let new_pos = wire.particles[0].pos;
    assert!(
        (new_pos - Vec2::new(19.0, 0.0)).length() < 1e-5,
        "verlet_step with damping=0.9 must yield pos=(19,0), got {new_pos}"
    );
}

// ---------------------------------------------------------------------------
// Rope wire — TC005–TC007: Jakobsen distance constraint relaxation
// ---------------------------------------------------------------------------

/// @doc: `relax_constraints` must be a no-op when particles are already at rest-length
/// separation. The Jakobsen correction is proportional to the signed distance error
/// `(actual - rest)`; when that error is zero, both correction halves are zero and no
/// position should change. If the method moves particles even when no error exists it will
/// fight the Verlet integrator every tick, preventing the rope from ever reaching a stable
/// resting pose and producing perpetual jitter visible to the player.
#[test]
fn when_particles_already_at_rest_length_then_relax_constraints_leaves_positions_unchanged() {
    // Arrange
    let rest_length = 10.0_f32;
    let mut wire = RopeWire::with_particles(vec![
        RopeParticle {
            pos: Vec2::new(0.0, 0.0),
            prev: Vec2::new(0.0, 0.0),
        },
        RopeParticle {
            pos: Vec2::new(10.0, 0.0),
            prev: Vec2::new(10.0, 0.0),
        },
    ]);
    let pos0_before = wire.particles[0].pos;
    let pos1_before = wire.particles[1].pos;

    // Act
    wire.relax_constraints(rest_length);

    // Assert
    assert!(
        (wire.particles[0].pos - pos0_before).length() < 1e-5,
        "particle[0] must not move when distance already equals rest_length, \
         expected {pos0_before}, got {}",
        wire.particles[0].pos
    );
    assert!(
        (wire.particles[1].pos - pos1_before).length() < 1e-5,
        "particle[1] must not move when distance already equals rest_length, \
         expected {pos1_before}, got {}",
        wire.particles[1].pos
    );
}

/// @doc: When two particles are closer than the rest length, `relax_constraints` must push
/// them apart until their separation equals `rest_length`. For a 2-particle chain there is
/// exactly one constraint, so a single pass gives the full correction: each particle moves
/// half of `(actual_dist - rest_length)` along the separation axis. If the method only
/// partially corrects (e.g. applies the full delta to one particle instead of splitting it)
/// the rope will never reach rest length in finite passes and will appear rubbery or collapsed
/// on screen rather than taut at the authored length.
#[test]
fn when_particles_too_close_then_relax_constraints_restores_rest_length() {
    // Arrange — particles at x=0 and x=5, rest_length=10 (half the rest distance)
    let rest_length = 10.0_f32;
    let mut wire = RopeWire::with_particles(vec![
        RopeParticle {
            pos: Vec2::new(0.0, 0.0),
            prev: Vec2::new(0.0, 0.0),
        },
        RopeParticle {
            pos: Vec2::new(5.0, 0.0),
            prev: Vec2::new(5.0, 0.0),
        },
    ]);

    // Act
    wire.relax_constraints(rest_length);

    // Assert
    let dist = (wire.particles[0].pos - wire.particles[1].pos).length();
    assert!(
        (dist - rest_length).abs() < 1e-4,
        "distance after one relax pass must equal rest_length={rest_length}, got {dist}"
    );
}

/// @doc: When two particles are further apart than the rest length, `relax_constraints` must
/// pull them together until their separation equals `rest_length`. This is the symmetrical
/// case to TC006: the signed error `(actual - rest)` is positive, so the correction vector
/// points inward and each particle moves half the error toward the other. If the sign of the
/// correction is inverted the particles would be pushed further apart every tick, causing the
/// cable to stretch without bound and producing a visual artefact where the rope grows longer
/// the longer the simulation runs.
#[test]
fn when_particles_too_far_then_relax_constraints_restores_rest_length() {
    // Arrange — particles at x=0 and x=20, rest_length=10 (double the rest distance)
    let rest_length = 10.0_f32;
    let mut wire = RopeWire::with_particles(vec![
        RopeParticle {
            pos: Vec2::new(0.0, 0.0),
            prev: Vec2::new(0.0, 0.0),
        },
        RopeParticle {
            pos: Vec2::new(20.0, 0.0),
            prev: Vec2::new(20.0, 0.0),
        },
    ]);

    // Act
    wire.relax_constraints(rest_length);

    // Assert
    let dist = (wire.particles[0].pos - wire.particles[1].pos).length();
    assert!(
        (dist - rest_length).abs() < 1e-4,
        "distance after one relax pass must equal rest_length={rest_length}, got {dist}"
    );
}

// ---------------------------------------------------------------------------
// Rope wire — TC008–TC009: apply_shrinkage self-straightening restoring force
// ---------------------------------------------------------------------------

/// @doc: `apply_shrinkage` models the visual tendency of a slack cable to resist wild
/// lateral bowing — a gentle restoring force that pulls each interior particle toward
/// the straight chord between the two endpoints. With a positive `strength`, a single
/// call must reduce the perpendicular displacement of every interior particle; if the
/// force is absent or reversed, a rope displaced sideways will never self-straighten
/// and the cable will appear to float rigidly in the wrong shape regardless of how
/// long the simulation runs.
#[test]
fn when_interior_particles_displaced_perpendicularly_then_apply_shrinkage_reduces_displacement() {
    // Arrange — 4-particle rope; chord is y=0 (endpoints at x=0 and x=30);
    // interior particles are 20 units above the chord line
    let mut wire = RopeWire::with_particles(vec![
        RopeParticle {
            pos: Vec2::new(0.0, 0.0),
            prev: Vec2::new(0.0, 0.0),
        },
        RopeParticle {
            pos: Vec2::new(10.0, 20.0),
            prev: Vec2::new(10.0, 20.0),
        },
        RopeParticle {
            pos: Vec2::new(20.0, 20.0),
            prev: Vec2::new(20.0, 20.0),
        },
        RopeParticle {
            pos: Vec2::new(30.0, 0.0),
            prev: Vec2::new(30.0, 0.0),
        },
    ]);
    let y_before_1 = wire.particles[1].pos.y;
    let y_before_2 = wire.particles[2].pos.y;

    // Act
    wire.apply_shrinkage(0.1);

    // Assert — both interior particles must be strictly closer to the chord (y=0)
    assert!(
        wire.particles[1].pos.y.abs() < y_before_1.abs(),
        "particle[1].y must move toward 0 after shrinkage; before={y_before_1}, after={}",
        wire.particles[1].pos.y
    );
    assert!(
        wire.particles[2].pos.y.abs() < y_before_2.abs(),
        "particle[2].y must move toward 0 after shrinkage; before={y_before_2}, after={}",
        wire.particles[2].pos.y
    );
}

/// @doc: When interior particles are already on the straight chord between the two
/// endpoints their interpolated target equals their current position, so the
/// correction vector is zero and `apply_shrinkage` must leave every position
/// untouched. If the method moves particles that are already collinear it introduces
/// drift that accumulates each tick and shifts a perfectly taut rope away from its
/// authored path, causing visible position errors for cables strung between nearby
/// sockets.
#[test]
fn when_all_particles_collinear_then_apply_shrinkage_leaves_positions_unchanged() {
    // Arrange — 4 particles perfectly on the x-axis (chord line)
    let mut wire = RopeWire::with_particles(vec![
        RopeParticle {
            pos: Vec2::new(0.0, 0.0),
            prev: Vec2::new(0.0, 0.0),
        },
        RopeParticle {
            pos: Vec2::new(10.0, 0.0),
            prev: Vec2::new(10.0, 0.0),
        },
        RopeParticle {
            pos: Vec2::new(20.0, 0.0),
            prev: Vec2::new(20.0, 0.0),
        },
        RopeParticle {
            pos: Vec2::new(30.0, 0.0),
            prev: Vec2::new(30.0, 0.0),
        },
    ]);
    let pos1_before = wire.particles[1].pos;
    let pos2_before = wire.particles[2].pos;

    // Act
    wire.apply_shrinkage(0.1);

    // Assert — interior positions must be identical to within floating-point noise
    assert!(
        (wire.particles[1].pos - pos1_before).length() < 1e-6,
        "particle[1] must not move when already collinear; expected {pos1_before}, got {}",
        wire.particles[1].pos
    );
    assert!(
        (wire.particles[2].pos - pos2_before).length() < 1e-6,
        "particle[2] must not move when already collinear; expected {pos2_before}, got {}",
        wire.particles[2].pos
    );
}

// ---------------------------------------------------------------------------
// Rope wire — TC010: pin_endpoints pure function
// ---------------------------------------------------------------------------

/// @doc: `pin_endpoints` is the boundary-condition enforcer that prevents the Verlet solver
/// from drifting the rope's two ends away from their socket anchors. Given a rope whose
/// particles are at arbitrary positions, a single call must teleport particle[0] to `a` and
/// particle[n-1] to `b`, leaving all interior particles exactly where they were. Without this
/// hard snap, every integration tick would allow the endpoints to wander, and the visual rope
/// would appear to detach from its jack sockets during gameplay.
#[test]
fn given_rope_with_arbitrary_positions_when_pin_endpoints_then_first_and_last_snap_and_interior_unchanged()
 {
    // Arrange
    let mut wire = RopeWire::with_particles(vec![
        RopeParticle {
            pos: Vec2::new(100.0, 100.0),
            prev: Vec2::new(100.0, 100.0),
        },
        RopeParticle {
            pos: Vec2::new(100.0, 100.0),
            prev: Vec2::new(100.0, 100.0),
        },
        RopeParticle {
            pos: Vec2::new(100.0, 100.0),
            prev: Vec2::new(100.0, 100.0),
        },
        RopeParticle {
            pos: Vec2::new(100.0, 100.0),
            prev: Vec2::new(100.0, 100.0),
        },
    ]);
    let pin_a = Vec2::new(0.0, 0.0);
    let pin_b = Vec2::new(50.0, 0.0);

    // Act
    wire.pin_endpoints(pin_a, pin_b);

    // Assert
    assert_eq!(
        wire.particles[0].pos, pin_a,
        "particle[0].pos must snap to pin_a after pin_endpoints"
    );
    assert_eq!(
        wire.particles[3].pos, pin_b,
        "particle[3].pos must snap to pin_b after pin_endpoints"
    );
    assert_eq!(
        wire.particles[1].pos,
        Vec2::new(100.0, 100.0),
        "particle[1] (interior) must be unchanged after pin_endpoints"
    );
    assert_eq!(
        wire.particles[2].pos,
        Vec2::new(100.0, 100.0),
        "particle[2] (interior) must be unchanged after pin_endpoints"
    );
}

// ---------------------------------------------------------------------------
// Rope wire — TC011: rope_physics_system ECS integration (positions track sockets)
// ---------------------------------------------------------------------------

/// @doc: `rope_physics_system` is responsible for keeping a `RopeWire`'s endpoints welded
/// to their jack socket entities every physics tick. A freshly-constructed rope whose
/// `RopeWireEndpoints` components point at sockets at (0,0) and (200,0) must, after one
/// schedule tick, have its first and last particles within floating-point tolerance of those
/// socket positions. Without this system the rope would hang in space at its spawn-time
/// positions and never respond to where the sockets actually are, making visual plug-in
/// connections impossible to author.
#[test]
fn given_rope_wired_between_two_sockets_when_rope_physics_system_runs_then_endpoints_match_socket_positions()
 {
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

    let rope_entity = world
        .spawn((
            RopeWire::new(Vec2::new(0.0, 0.0), Vec2::new(200.0, 0.0), 6),
            RopeWireEndpoints { source, dest },
        ))
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems(rope_physics_system);

    // Act
    schedule.run(&mut world);

    // Assert
    let rope = world.get::<RopeWire>(rope_entity).unwrap();
    let first = rope.particles.first().unwrap().pos;
    let last = rope.particles.last().unwrap().pos;
    assert!(
        (first - Vec2::new(0.0, 0.0)).length() < 0.1,
        "first particle must be within 0.1 of source socket (0,0), got {first}"
    );
    assert!(
        (last - Vec2::new(200.0, 0.0)).length() < 0.1,
        "last particle must be within 0.1 of dest socket (200,0), got {last}"
    );
}

// ---------------------------------------------------------------------------
// Rope wire — TC012: rope_physics_system tracks a moved source socket
// ---------------------------------------------------------------------------

/// @doc: `rope_physics_system` must re-read socket `Transform2D` positions each tick so that
/// a rope responds immediately when a socket is relocated. After moving the source socket
/// from (0,0) to (50,50) before the system runs, the rope's first particle must end up at
/// the new socket position rather than the old one. Without live position reading, the rope
/// would remain anchored to stale coordinates and appear disconnected from a socket that the
/// player has just dragged to a new location.
#[test]
fn given_source_socket_moved_before_tick_when_rope_physics_system_runs_then_first_particle_at_new_socket_position()
 {
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

    let rope_entity = world
        .spawn((
            RopeWire::new(Vec2::new(0.0, 0.0), Vec2::new(200.0, 0.0), 6),
            RopeWireEndpoints { source, dest },
        ))
        .id();

    // Move source socket to a new position before the system runs
    world.get_mut::<Transform2D>(source).unwrap().position = Vec2::new(50.0, 50.0);

    let mut schedule = Schedule::default();
    schedule.add_systems(rope_physics_system);

    // Act
    schedule.run(&mut world);

    // Assert
    let rope = world.get::<RopeWire>(rope_entity).unwrap();
    let first = rope.particles[0].pos;
    assert!(
        (first - Vec2::new(50.0, 50.0)).length() < 0.1,
        "particles[0].pos must track the moved source socket at (50,50), got {first}"
    );
}

// ---------------------------------------------------------------------------
// Rope wire — TC013: rope_render_system emits exactly N-1 draw calls
// ---------------------------------------------------------------------------

fn spawn_renderable_rope(world: &mut World, particles: Vec<RopeParticle>) -> Entity {
    let n = particles.len();
    let segment_count = n.saturating_sub(1);

    let segment_ids: Vec<Entity> = (0..segment_count)
        .map(|_| {
            world
                .spawn((
                    Transform2D::default(),
                    Shape {
                        variant: ShapeVariant::Polygon {
                            points: vec![
                                Vec2::new(-1.0, -1.0),
                                Vec2::new(1.0, -1.0),
                                Vec2::new(1.0, 1.0),
                                Vec2::new(-1.0, 1.0),
                            ],
                        },
                        color: engine_core::color::Color::WHITE,
                    },
                    Visible(true),
                    RenderLayer::World,
                    SortOrder::default(),
                ))
                .id()
        })
        .collect();

    let mut rope = RopeWire::with_particles(particles);
    rope.segments = segment_ids;

    world
        .spawn((
            rope,
            Transform2D::default(),
            RenderLayer::World,
            SortOrder::default(),
            Visible(true),
        ))
        .id()
}

fn run_rope_visuals(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(
        (
            rope_render_system,
            transform_propagation_system,
            visibility_system,
            unified_render_system,
        )
            .chain(),
    );
    schedule.run(world);
}

/// @doc: `rope_render_system` must produce exactly one draw call for a rope entity by writing
/// a single bezier `Path` shape, rather than the old approach of N-1 separate polygon segment
/// entities. One draw call means a single unified curve rendered by the GPU, which eliminates
/// visible seams between segments and reduces the entity count per cable from O(N) to O(1).
#[test]
fn given_rope_wire_with_n_particles_when_rope_render_system_runs_then_one_draw_call_emitted() {
    // Arrange
    let (mut world, shape_calls) = make_cable_render_world();
    let particles = vec![
        RopeParticle {
            pos: Vec2::new(0.0, 0.0),
            prev: Vec2::new(0.0, 0.0),
        },
        RopeParticle {
            pos: Vec2::new(50.0, 0.0),
            prev: Vec2::new(50.0, 0.0),
        },
        RopeParticle {
            pos: Vec2::new(100.0, 0.0),
            prev: Vec2::new(100.0, 0.0),
        },
        RopeParticle {
            pos: Vec2::new(150.0, 0.0),
            prev: Vec2::new(150.0, 0.0),
        },
    ];
    let rope = RopeWire::with_particles(particles);
    world.spawn((
        rope,
        Transform2D::default(),
        Shape {
            variant: ShapeVariant::Polygon {
                points: vec![Vec2::ZERO],
            },
            color: Color::WHITE,
        },
        Visible(true),
        RenderLayer::World,
        SortOrder::default(),
    ));

    // Act
    run_rope_visuals(&mut world);

    // Assert — the rope entity produces a single ribbon polygon draw call
    assert_eq!(
        shape_calls.lock().unwrap().len(),
        1,
        "a RopeWire must produce exactly 1 draw call (ribbon polygon)"
    );
}

// ---------------------------------------------------------------------------
// TC-F01 — rope_physics_system must not apply gravity to particles
// ---------------------------------------------------------------------------

/// @doc: `rope_physics_system` must not apply any vertical force to particles. A rope
/// whose particles all start at rest (`prev == pos`) between two horizontal sockets must
/// keep every particle at its initial Y position after one tick. Without this invariant,
/// every rope in the UI would visibly sag further on each frame until it collapses out of
/// view, making a static wiring layout impossible to maintain visually.
#[test]
fn when_rope_at_rest_pinned_between_horizontal_sockets_then_interior_particle_y_positions_unchanged()
 {
    // Arrange
    let mut world = World::new();
    let a = Vec2::new(0.0, 0.0);
    let b = Vec2::new(200.0, 0.0);

    let source = world
        .spawn(Transform2D {
            position: a,
            rotation: 0.0,
            scale: Vec2::ONE,
        })
        .id();
    let dest = world
        .spawn(Transform2D {
            position: b,
            rotation: 0.0,
            scale: Vec2::ONE,
        })
        .id();

    let rope_entity = world
        .spawn((RopeWire::new(a, b, 6), RopeWireEndpoints { source, dest }))
        .id();

    let y_before: Vec<f32> = world
        .get::<RopeWire>(rope_entity)
        .unwrap()
        .particles
        .iter()
        .map(|p| p.pos.y)
        .collect();

    let mut schedule = Schedule::default();
    schedule.add_systems(rope_physics_system);

    // Act
    schedule.run(&mut world);

    // Assert
    let rope = world.get::<RopeWire>(rope_entity).unwrap();
    for (i, (particle, &y_init)) in rope.particles.iter().zip(y_before.iter()).enumerate() {
        assert!(
            (particle.pos.y - y_init).abs() < 1e-4,
            "particle[{i}].pos.y must remain {y_init} after one tick at rest, \
             got {} (gravity offset detected)",
            particle.pos.y
        );
    }
}

// ---------------------------------------------------------------------------
// TC-F08 / TC-F09 — dynamic particle count scales with distance
// ---------------------------------------------------------------------------

/// @doc: `RopeWire::for_distance` must produce fewer particles for short cables than for
/// long ones, scaling proportionally with a fixed segment length. A 30-pixel cable should
/// have only a handful of particles, while a 300-pixel cable should have substantially more.
/// Without distance-based scaling, short cables would have too many segments (wasting memory
/// and producing near-zero-length constraints that jitter), while long cables would have too
/// few (producing visible polygonal kinks instead of a smooth curve).
#[test]
fn when_rope_wire_created_for_short_vs_long_distance_then_long_has_more_particles() {
    // Arrange
    let a = Vec2::ZERO;
    let short_b = Vec2::new(30.0, 0.0);
    let long_b = Vec2::new(300.0, 0.0);

    // Act
    let short_rope = RopeWire::for_distance(a, short_b);
    let long_rope = RopeWire::for_distance(a, long_b);

    // Assert
    assert!(
        short_rope.particles.len() >= 2,
        "even the shortest cable needs at least 2 particles (endpoints), got {}",
        short_rope.particles.len()
    );
    assert!(
        long_rope.particles.len() > short_rope.particles.len(),
        "a 300px cable must have more particles than a 30px cable: long={}, short={}",
        long_rope.particles.len(),
        short_rope.particles.len()
    );
    assert!(
        long_rope.particles.len() >= 10,
        "a 300px cable should have at least 10 particles for smooth bezier rendering, got {}",
        long_rope.particles.len()
    );
}

// ---------------------------------------------------------------------------
// TC-F12 — B-spline to bezier conversion produces correct segment count
// ---------------------------------------------------------------------------

/// @doc: `particles_to_bezier_path` converts Verlet particle positions into a smooth B-spline
/// curve represented as cubic bezier `PathCommand`s. For N particles, the result must contain
/// a single `MoveTo` followed by exactly N-1 `CubicTo` commands — one bezier segment per
/// adjacent particle pair. Fewer segments leave gaps in the cable; more segments produce
/// phantom arcs between non-adjacent particles.
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
// TC-F13 — collinear particles produce a straight bezier
// ---------------------------------------------------------------------------

/// @doc: When all particles lie on the same line, the B-spline approximation must also lie
/// on that line — all `CubicTo` control points must have the same Y coordinate as the
/// particles. If control points deviate from the chord, a taut cable would render as a
/// wavy curve, confusing the player about whether the cable is slack or tight.
#[test]
fn given_collinear_particles_when_bezier_path_computed_then_all_control_points_on_same_line() {
    // Arrange — 4 particles on Y=0
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
                "control1.y must be ~0.0 for collinear particles, got {}",
                control1.y
            );
            assert!(
                control2.y.abs() < 1e-4,
                "control2.y must be ~0.0 for collinear particles, got {}",
                control2.y
            );
            assert!(
                to.y.abs() < 1e-4,
                "to.y must be ~0.0 for collinear particles, got {}",
                to.y
            );
        }
    }
}

// ---------------------------------------------------------------------------
// TC-F14 — rope_render_system writes a ribbon Polygon from particle offsets
// ---------------------------------------------------------------------------

/// @doc: `rope_render_system` converts rope particles into a ribbon polygon by offsetting
/// each particle position perpendicular to the local tangent. The forward edge runs along
/// one side of the cable, and the backward edge along the other, forming a closed filled
/// strip. This avoids the Path fill+close rendering artifact where the GPU would draw a
/// filled region between the bezier curve and a straight-line closure.
#[test]
fn given_rope_wire_when_rope_render_system_runs_then_shape_is_ribbon_polygon() {
    // Arrange
    let mut world = World::new();
    let particles = vec![
        RopeParticle {
            pos: Vec2::new(0.0, 0.0),
            prev: Vec2::new(0.0, 0.0),
        },
        RopeParticle {
            pos: Vec2::new(50.0, 10.0),
            prev: Vec2::new(50.0, 10.0),
        },
        RopeParticle {
            pos: Vec2::new(100.0, 0.0),
            prev: Vec2::new(100.0, 0.0),
        },
    ];
    let rope = RopeWire::with_particles(particles);

    let rope_entity = world
        .spawn((
            rope,
            Transform2D::default(),
            Shape {
                variant: ShapeVariant::Polygon {
                    points: vec![Vec2::ZERO],
                },
                color: Color::WHITE,
            },
            Visible(true),
            RenderLayer::World,
            SortOrder::default(),
        ))
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems(rope_render_system);

    // Act
    schedule.run(&mut world);

    // Assert — ribbon is a Polygon with subdivided smooth edges (more vertices than raw particles)
    let shape = world.get::<Shape>(rope_entity).unwrap();
    match &shape.variant {
        ShapeVariant::Polygon { points } => {
            // 3 particles → 2 segments × 4 subdivisions + 1 = 9 per edge × 2 edges = 18
            assert!(
                points.len() > 6,
                "ribbon must have more vertices than 2*N due to Catmull-Rom subdivision, got {}",
                points.len()
            );
        }
        other => panic!(
            "rope_render_system must write ShapeVariant::Polygon, got {:?}",
            std::mem::discriminant(other)
        ),
    }
}

// ---------------------------------------------------------------------------
// Polygon collision — particle pushed out of convex polygon
// ---------------------------------------------------------------------------

#[test]
fn when_particle_inside_polygon_then_resolve_polygon_collisions_pushes_it_out() {
    // Arrange — a square obstacle centered at (100, 100) with half-extent 20
    let vertices = vec![
        Vec2::new(80.0, 80.0),
        Vec2::new(120.0, 80.0),
        Vec2::new(120.0, 120.0),
        Vec2::new(80.0, 120.0),
    ];
    let mut wire = RopeWire::with_particles(vec![
        RopeParticle {
            pos: Vec2::new(0.0, 0.0),
            prev: Vec2::new(0.0, 0.0),
        },
        RopeParticle {
            pos: Vec2::new(105.0, 100.0),
            prev: Vec2::new(105.0, 100.0),
        },
        RopeParticle {
            pos: Vec2::new(200.0, 0.0),
            prev: Vec2::new(200.0, 0.0),
        },
    ]);

    // Act
    wire.resolve_polygon_collisions(&[(Vec2::new(100.0, 100.0), &vertices)]);

    // Assert — particle must be outside the polygon
    let p = wire.particles[1].pos;
    let dx = (p.x - 100.0).abs();
    let dy = (p.y - 100.0).abs();
    assert!(
        dx >= 19.9 || dy >= 19.9,
        "particle must be pushed to polygon boundary, got {p}"
    );
}

// ---------------------------------------------------------------------------
// WrapWire — shortest_path computation
// ---------------------------------------------------------------------------

/// @doc: A `WrapWire` with no anchors represents a cable that wraps around zero obstacles,
/// so its shortest geometric path is simply the straight-line distance between the two
/// endpoints. This is the base case for the path-length computation used by the retraction
/// system — if the zero-anchor case returns the wrong value, every retraction target length
/// will be miscalculated even for cables that have never contacted an obstacle.
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
/// also be wrong, breaking retraction distance calculations for any wrapped cable.
#[test]
fn when_wrap_wire_has_one_anchor_then_shortest_path_goes_through_it() {
    // Arrange
    let mut wire = WrapWire::new();
    wire.anchors.push(WrapAnchor {
        position: Vec2::new(50.0, 50.0),
        obstacle: Entity::from_raw(999),
        vertex_index: 0,
        wrap_sign: 1.0,
        pinned_particle: 0,
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

use card_game::card::jack_cable::{find_wrap_vertex, segment_intersects_segment};

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

    // Assert
    assert_eq!(wire.anchors.len(), 1, "must insert exactly one anchor");
    assert_eq!(wire.anchors[0].obstacle, obstacle);
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
    let mut wire = WrapWire::new();
    wire.anchors.push(WrapAnchor {
        position: Vec2::new(50.0, 10.0),
        obstacle: Entity::from_raw(42),
        vertex_index: 2,
        wrap_sign: 1.0,
        pinned_particle: 0,
    });

    let src = Vec2::new(0.0, 0.0);
    // dst on the opposite side — cable has swung past
    let dst = Vec2::new(100.0, -20.0);

    // Act
    wire.detect_unwraps(src, dst);

    // Assert
    assert!(
        wire.anchors.is_empty(),
        "anchor must be removed when cable swings past"
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
        .spawn((
            RopeWire::new(Vec2::new(0.0, 5.0), Vec2::new(200.0, 5.0), 10),
            RopeWireEndpoints { source, dest },
            WrapWire::new(),
        ))
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
// rope_physics_system pins particles at WrapWire anchor positions
// ---------------------------------------------------------------------------

#[test]
fn when_rope_has_wrap_anchor_then_physics_pins_particle_at_anchor_position() {
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

    let anchor_pos = Vec2::new(100.0, 50.0);
    let mut wrap = WrapWire::new();
    wrap.anchors.push(WrapAnchor {
        position: anchor_pos,
        obstacle: Entity::from_raw(999),
        vertex_index: 0,
        wrap_sign: 1.0,
        pinned_particle: 5,
    });
    wrap.target_length = 300.0;

    let rope_entity = world
        .spawn((
            RopeWire::new(Vec2::new(0.0, 0.0), Vec2::new(200.0, 0.0), 10),
            RopeWireEndpoints { source, dest },
            wrap,
        ))
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems(rope_physics_system);

    // Act
    schedule.run(&mut world);

    // Assert
    let rope = world.get::<RopeWire>(rope_entity).unwrap();
    let pinned = rope.particles[5].pos;
    assert!(
        (pinned - anchor_pos).length() < 0.1,
        "particle[5] must be pinned to anchor at {anchor_pos}, got {pinned}"
    );
}

// ---------------------------------------------------------------------------
// Retraction — WrapWire::retract reduces target_length toward shortest path
// ---------------------------------------------------------------------------

#[test]
fn when_target_length_exceeds_shortest_path_then_retraction_reduces_it() {
    // Arrange
    let mut wire = WrapWire::new();
    wire.target_length = 200.0;
    let src = Vec2::new(0.0, 0.0);
    let dst = Vec2::new(100.0, 0.0);
    let shortest = wire.shortest_path(src, dst); // 100.0
    let dt = 0.016;
    let retraction_rate = 3.0;

    // Act
    wire.retract(src, dst, retraction_rate, dt);

    // Assert
    assert!(wire.target_length < 200.0, "target_length must decrease");
    assert!(
        wire.target_length > shortest,
        "target_length must not go below shortest path"
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
        wrap_sign: 1.0,
        pinned_particle: 3,
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

// ---------------------------------------------------------------------------
// Bug regression — wrap_detect_system must assign a non-zero pinned_particle (Bug 2)
// ---------------------------------------------------------------------------

/// @doc: `wrap_detect_system` must assign each newly-inserted `WrapAnchor` a `pinned_particle`
/// index that points to an interior particle in the associated `RopeWire`, not to index 0.
/// `rope_physics_system` guards `if idx > 0 && idx < n - 1`, so an anchor with
/// `pinned_particle == 0` silently pins nothing: the cable appears to wrap geometrically but
/// the Verlet physics ignores the anchor and lets the cable pull straight through the obstacle.
#[test]
fn when_wrap_detect_system_inserts_anchor_then_pinned_particle_is_nonzero_interior_index() {
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

    // Obstacle box centered at (100, 0) with half-extents (20, 20)
    world.spawn((
        Transform2D {
            position: Vec2::new(100.0, 0.0),
            rotation: 0.0,
            scale: Vec2::ONE,
        },
        CableCollider::from_aabb(Vec2::new(20.0, 20.0)),
    ));

    let cable_entity = world
        .spawn((
            RopeWire::new(Vec2::new(0.0, 5.0), Vec2::new(200.0, 5.0), 10),
            RopeWireEndpoints { source, dest },
            WrapWire::new(),
        ))
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems((wrap_update_system, wrap_detect_system).chain());

    // Act
    schedule.run(&mut world);

    // Assert
    let wrap = world.get::<WrapWire>(cable_entity).unwrap();
    let rope = world.get::<RopeWire>(cable_entity).unwrap();
    assert!(
        !wrap.anchors.is_empty(),
        "wrap_detect_system must insert at least one anchor"
    );
    for anchor in &wrap.anchors {
        let n = rope.particles.len();
        assert!(
            anchor.pinned_particle > 0 && anchor.pinned_particle < n - 1,
            "pinned_particle must be an interior index (1..{}) so rope_physics_system \
             actually pins it; got {}",
            n - 1,
            anchor.pinned_particle
        );
    }
}

// ---------------------------------------------------------------------------
// Integration — wrap + physics together: particle is actually pinned to obstacle vertex
// ---------------------------------------------------------------------------

/// @doc: End-to-end test verifying that after `wrap_detect_system` and `rope_physics_system`
/// both run, the particle corresponding to the wrap anchor is held at the obstacle vertex
/// position. This is the observable player-facing invariant: the cable must not pass through
/// the reader box when dragged across it. The previous two bugs (wrong `already_anchored`
/// scope and `pinned_particle: 0`) together allowed anchors to be created but ignored by the
/// physics solver, so the cable pulled straight through.
#[test]
fn when_cable_crosses_obstacle_and_both_systems_run_then_anchor_particle_is_pinned_to_vertex() {
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

    // Obstacle box centered at (100, 0) with half-extents (20, 20)
    let obstacle_pos = Vec2::new(100.0, 0.0);
    world.spawn((
        Transform2D {
            position: obstacle_pos,
            rotation: 0.0,
            scale: Vec2::ONE,
        },
        CableCollider::from_aabb(Vec2::new(20.0, 20.0)),
    ));

    // Enough particles so index 0 and n-1 are endpoints, and interior indices exist
    let mut wrap = WrapWire::new();
    wrap.target_length = 300.0;
    let cable_entity = world
        .spawn((
            RopeWire::new(Vec2::new(0.0, 5.0), Vec2::new(200.0, 5.0), 12),
            RopeWireEndpoints { source, dest },
            wrap,
        ))
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems((wrap_update_system, wrap_detect_system, rope_physics_system).chain());

    // Act — run two ticks so the anchor from tick 1 is pinned in tick 2
    schedule.run(&mut world);
    schedule.run(&mut world);

    // Assert — at least one anchor exists and its particle is at the anchor position
    let wrap = world.get::<WrapWire>(cable_entity).unwrap();
    let rope = world.get::<RopeWire>(cable_entity).unwrap();
    assert!(
        !wrap.anchors.is_empty(),
        "cable crossing obstacle must have at least one anchor"
    );
    for anchor in &wrap.anchors {
        let idx = anchor.pinned_particle;
        let n = rope.particles.len();
        assert!(
            idx > 0 && idx < n - 1,
            "pinned_particle must be an interior index, got {idx} (n={n})"
        );
        let pinned_pos = rope.particles[idx].pos;
        assert!(
            (pinned_pos - anchor.position).length() < 1.0,
            "particle[{idx}] must be pinned to anchor at {:?}; got {:?} (distance {})",
            anchor.position,
            pinned_pos,
            (pinned_pos - anchor.position).length()
        );
    }
}
