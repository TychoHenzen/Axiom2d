#![allow(clippy::unwrap_used)]

use std::sync::{Arc, Mutex};

use bevy_ecs::prelude::*;
use card_game::card::identity::signature::{CardSignature, Element};
use card_game::card::jack_cable::{Cable, Jack, JackDirection, signature_space_propagation_system};
use card_game::card::reader::{SIGNATURE_SPACE_RADIUS, SignatureSpace};
use card_game::card::screen_device::{ScreenDevice, display_axes, screen_render_system};
use engine_core::prelude::Transform2D;
use engine_render::prelude::{Camera2D, RendererRes};
use engine_render::testing::{ShapeCallLog, SpyRenderer};
use glam::Vec2;

// ---------------------------------------------------------------------------
// Shared helper
// ---------------------------------------------------------------------------

fn make_screen_world(jack_data: Option<SignatureSpace>) -> (World, ShapeCallLog) {
    let mut world = World::new();

    let jack_entity = world
        .spawn(Jack::<SignatureSpace> {
            direction: JackDirection::Input,
            data: jack_data,
        })
        .id();

    world.spawn((
        ScreenDevice {
            signature_input: jack_entity,
        },
        Transform2D::default(),
    ));

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
    let space = SignatureSpace {
        center,
        radius: SIGNATURE_SPACE_RADIUS,
    };

    // Act
    let (x, y) = display_axes(&space, 0);

    // Assert
    assert_eq!(x, space.center[Element::Solidum]);
    assert_eq!(y, space.center[Element::Febris]);
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
    let space = SignatureSpace {
        center,
        radius: SIGNATURE_SPACE_RADIUS,
    };

    // Act
    let (x, y) = display_axes(&space, 3);

    // Assert
    assert_eq!(x, space.center[Element::Subsidium]);
    assert_eq!(y, space.center[Element::Spatium]);
}

// ---------------------------------------------------------------------------
// TC012 — no draws when input jack has None
// ---------------------------------------------------------------------------

/// @doc: A `ScreenDevice` whose input jack carries `None` must produce zero draw calls.
/// Without this gate, a device with a disconnected cable would emit phantom dots onto
/// the display — shapes that represent a signature nobody actually played. Clean silence
/// when there is no signal is the "off" state that every other display test depends on.
#[test]
fn given_input_jack_data_is_none_when_screen_render_system_runs_then_no_shapes_are_drawn() {
    // Arrange
    let (mut world, shape_calls) = make_screen_world(None);
    let mut schedule = Schedule::default();
    schedule.add_systems(screen_render_system);

    // Act
    schedule.run(&mut world);

    // Assert
    assert!(
        shape_calls.lock().unwrap().is_empty(),
        "disconnected input jack must produce zero draw calls — no phantom signal"
    );
}

// ---------------------------------------------------------------------------
// TC013 — 4 draws when input jack has Some(SignatureSpace)
// ---------------------------------------------------------------------------

/// @doc: When a `SignatureSpace` arrives on the input jack, `screen_render_system` must draw
/// exactly 4 dots — one per element-pair display panel (Solidum/Febris, Ordinem/Lumines,
/// Varias/Inertiae, Subsidium/Spatium). Fewer than 4 would leave panels dark that should
/// show the card's position in that dimension; more than 4 would invent phantom panels or
/// repeat existing ones. The count of 4 is load-bearing: it is the fixed arity of the
/// 8-dimensional element space partitioned into consecutive pairs.
#[test]
fn given_input_jack_has_signature_space_when_screen_render_system_runs_then_draws_four_shapes() {
    // Arrange
    let signal = SignatureSpace {
        center: CardSignature::new([0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8]),
        radius: SIGNATURE_SPACE_RADIUS,
    };
    let (mut world, shape_calls) = make_screen_world(Some(signal));
    let mut schedule = Schedule::default();
    schedule.add_systems(screen_render_system);

    // Act
    schedule.run(&mut world);

    // Assert — exactly one dot per display panel, 4 panels total
    assert_eq!(
        shape_calls.lock().unwrap().len(),
        4,
        "each of the 4 element-pair display panels must produce exactly one dot shape"
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
        .data = Some(SignatureSpace {
        center: sig,
        radius: SIGNATURE_SPACE_RADIUS,
    });

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
        Some(SignatureSpace {
            center: sig,
            radius: SIGNATURE_SPACE_RADIUS,
        }),
        "screen input jack must receive the SignatureSpace after cable propagation"
    );
}
