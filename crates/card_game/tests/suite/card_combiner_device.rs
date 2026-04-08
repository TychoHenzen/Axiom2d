#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::*;
use card_game::card::combiner_device::{CombinerDevice, combiner_system};
use card_game::card::identity::signature::CardSignature;
use card_game::card::jack_cable::{Jack, JackDirection};
use card_game::card::reader::SignatureSpace;
use card_game::card::reader::volume::sphere_volume_8d;

fn run_combiner(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(combiner_system);
    schedule.run(world);
}

fn make_signal(values: [f32; 8], radius: f32) -> SignatureSpace {
    SignatureSpace::from_single(CardSignature::new(values), radius, Entity::from_raw(0))
}

fn spawn_combiner(world: &mut World) -> (Entity, Entity, Entity, Entity) {
    let input_a = world
        .spawn(Jack::<SignatureSpace> {
            direction: JackDirection::Input,
            data: None,
        })
        .id();
    let input_b = world
        .spawn(Jack::<SignatureSpace> {
            direction: JackDirection::Input,
            data: None,
        })
        .id();
    let output = world
        .spawn(Jack::<SignatureSpace> {
            direction: JackDirection::Output,
            data: None,
        })
        .id();
    let device = world
        .spawn(CombinerDevice {
            input_a,
            input_b,
            output,
        })
        .id();
    (device, input_a, input_b, output)
}

#[test]
fn when_both_inputs_none_then_output_is_none() {
    // Arrange
    let mut world = World::new();
    let (_device, _a, _b, output) = spawn_combiner(&mut world);

    // Act
    run_combiner(&mut world);

    // Assert
    let jack = world.get::<Jack<SignatureSpace>>(output).unwrap();
    assert!(jack.data.is_none());
}

#[test]
fn when_only_input_a_has_signal_then_output_passes_through() {
    // Arrange
    let mut world = World::new();
    let (_device, input_a, _input_b, output) = spawn_combiner(&mut world);
    let signal = make_signal([0.3, -0.2, 0.1, 0.4, 0.0, 0.0, 0.0, 0.0], 0.2);
    world.get_mut::<Jack<SignatureSpace>>(input_a).unwrap().data = Some(signal.clone());

    // Act
    run_combiner(&mut world);

    // Assert
    let jack = world.get::<Jack<SignatureSpace>>(output).unwrap();
    assert_eq!(jack.data.as_ref(), Some(&signal));
}

#[test]
fn when_only_input_b_has_signal_then_output_passes_through() {
    // Arrange
    let mut world = World::new();
    let (_device, _input_a, input_b, output) = spawn_combiner(&mut world);
    let signal = make_signal([0.5, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], 0.18);
    world.get_mut::<Jack<SignatureSpace>>(input_b).unwrap().data = Some(signal.clone());

    // Act
    run_combiner(&mut world);

    // Assert
    let jack = world.get::<Jack<SignatureSpace>>(output).unwrap();
    assert_eq!(jack.data.as_ref(), Some(&signal));
}

#[test]
fn when_both_inputs_have_signals_then_output_combines_control_points() {
    // Arrange
    let mut world = World::new();
    let (_device, input_a, input_b, output) = spawn_combiner(&mut world);
    let sig_a = make_signal([0.1, 0.2, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], 0.2);
    let sig_b = make_signal([0.5, 0.6, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], 0.2);
    world.get_mut::<Jack<SignatureSpace>>(input_a).unwrap().data = Some(sig_a.clone());
    world.get_mut::<Jack<SignatureSpace>>(input_b).unwrap().data = Some(sig_b.clone());

    // Act
    run_combiner(&mut world);

    // Assert
    let jack = world.get::<Jack<SignatureSpace>>(output).unwrap();
    let combined = jack.data.as_ref().expect("output must have data");
    assert_eq!(
        combined.control_points.len(),
        2,
        "must have 2 control points"
    );
    assert_eq!(combined.control_points[0], sig_a.control_points[0]);
    assert_eq!(combined.control_points[1], sig_b.control_points[0]);
}

#[test]
fn when_both_inputs_have_signals_then_output_volume_is_sum() {
    // Arrange
    let mut world = World::new();
    let (_device, input_a, input_b, output) = spawn_combiner(&mut world);
    let r_a = 0.18;
    let r_b = 0.22;
    let sig_a = make_signal([0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], r_a);
    let sig_b = make_signal([0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], r_b);
    world.get_mut::<Jack<SignatureSpace>>(input_a).unwrap().data = Some(sig_a);
    world.get_mut::<Jack<SignatureSpace>>(input_b).unwrap().data = Some(sig_b);

    // Act
    run_combiner(&mut world);

    // Assert
    let jack = world.get::<Jack<SignatureSpace>>(output).unwrap();
    let combined = jack.data.as_ref().expect("output must have data");
    let expected_vol = sphere_volume_8d(r_a) + sphere_volume_8d(r_b);
    assert!(
        (combined.volume - expected_vol).abs() < 1e-8,
        "combined volume {} must equal sum of inputs {}",
        combined.volume,
        expected_vol
    );
}

#[test]
fn when_combiner_chained_then_points_merged_and_sorted() {
    // Arrange — two combiners: C1 merges A+B, C2 takes C1's output + C
    let mut world = World::new();
    let (_dev1, in_a, in_b, out1) = spawn_combiner(&mut world);
    let (_dev2, in_c, in_d, out2) = spawn_combiner(&mut world);

    let sig_a = make_signal([0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], 0.2);
    let sig_b = make_signal([0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], 0.2);
    let sig_c = make_signal([0.3, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], 0.2);

    world.get_mut::<Jack<SignatureSpace>>(in_a).unwrap().data = Some(sig_a);
    world.get_mut::<Jack<SignatureSpace>>(in_b).unwrap().data = Some(sig_b);

    // First frame: C1 combines A+B
    run_combiner(&mut world);
    let c1_output = world
        .get::<Jack<SignatureSpace>>(out1)
        .unwrap()
        .data
        .clone();

    // Feed C1's output into C2's input, plus sig_c
    world.get_mut::<Jack<SignatureSpace>>(in_c).unwrap().data = c1_output;
    world.get_mut::<Jack<SignatureSpace>>(in_d).unwrap().data = Some(sig_c);

    // Act — second frame: C2 combines (A+B)+C
    run_combiner(&mut world);

    // Assert
    let jack = world.get::<Jack<SignatureSpace>>(out2).unwrap();
    let combined = jack.data.as_ref().expect("output must have data");
    assert_eq!(
        combined.control_points.len(),
        3,
        "must have 3 control points"
    );
    assert_eq!(
        combined.control_points[0],
        CardSignature::new([0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
    );
    assert_eq!(
        combined.control_points[1],
        CardSignature::new([0.3, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
    );
    assert_eq!(
        combined.control_points[2],
        CardSignature::new([0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
    );
}

#[test]
fn when_canonical_sort_then_same_points_different_order_produce_identical_output() {
    // Arrange
    let mut world = World::new();
    let (_dev1, in_a1, in_b1, out1) = spawn_combiner(&mut world);
    let (_dev2, in_a2, in_b2, out2) = spawn_combiner(&mut world);

    let sig_x = make_signal([0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], 0.2);
    let sig_y = make_signal([0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], 0.2);

    // C1: X on input_a, Y on input_b
    world.get_mut::<Jack<SignatureSpace>>(in_a1).unwrap().data = Some(sig_x.clone());
    world.get_mut::<Jack<SignatureSpace>>(in_b1).unwrap().data = Some(sig_y.clone());

    // C2: Y on input_a, X on input_b (swapped)
    world.get_mut::<Jack<SignatureSpace>>(in_a2).unwrap().data = Some(sig_y);
    world.get_mut::<Jack<SignatureSpace>>(in_b2).unwrap().data = Some(sig_x);

    // Act
    run_combiner(&mut world);

    // Assert
    let data1 = world
        .get::<Jack<SignatureSpace>>(out1)
        .unwrap()
        .data
        .as_ref()
        .unwrap()
        .clone();
    let data2 = world
        .get::<Jack<SignatureSpace>>(out2)
        .unwrap()
        .data
        .as_ref()
        .unwrap()
        .clone();
    assert_eq!(
        data1, data2,
        "identical card sets must produce identical output"
    );
}

#[test]
fn when_capsule_contains_midpoint_then_returns_true_and_far_point_false() {
    // Arrange
    let a = CardSignature::new([0.0; 8]);
    let b = CardSignature::new([0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    let capsule = SignatureSpace::combine(
        &SignatureSpace::from_single(a, 0.2, Entity::from_raw(0)),
        &SignatureSpace::from_single(b, 0.2, Entity::from_raw(1)),
    );

    // Act / Assert — midpoint of segment is inside the capsule
    let midpoint = CardSignature::new([0.25, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    assert!(capsule.contains(&midpoint));

    // Act / Assert — point far from the segment is outside
    let far = CardSignature::new([0.25, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    assert!(!capsule.contains(&far));
}
