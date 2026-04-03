#![allow(clippy::unwrap_used)]

use std::sync::{Arc, Mutex};

use bevy_ecs::prelude::*;
use card_game::card::identity::signature::CardSignature;
use card_game::card::jack_cable::{
    Cable, Jack, JackDirection, cable_render_system, signature_space_propagation_system,
};
use card_game::card::reader::{SIGNATURE_SPACE_RADIUS, SignatureSpace};
use engine_core::color::Color;
use engine_core::prelude::Transform2D;
use engine_render::prelude::{Camera2D, RendererRes, Shape, ShapeVariant};
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

/// @doc: `cable_render_system` must be a no-op when there are no Cable entities in the
/// world. Any accidental draw here would appear as a stray line floating on the table
/// that corresponds to no connection the player made.
#[test]
fn when_no_cables_exist_then_no_shapes_are_drawn() {
    // Arrange
    let (mut world, shape_calls) = make_cable_render_world();

    // Act
    run_cable_visuals(&mut world);

    // Assert
    assert!(
        shape_calls.lock().unwrap().is_empty(),
        "zero Cable entities must produce zero draw calls"
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
