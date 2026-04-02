#![allow(clippy::unwrap_used)]

use engine_core::time::{ClockRes, DeltaTime, FakeClock, FixedTimestep, Time, time_system};
use engine_core::types::Seconds;

/// @doc: `FakeClock` enables deterministic testing — `advance()` accumulates, `delta()` drains
#[test]
fn when_fake_clock_advanced_then_delta_returns_advancement() {
    // Arrange
    let mut clock = FakeClock::default();

    // Act
    clock.advance(Seconds(0.016));

    // Assert
    assert_eq!(clock.delta(), Seconds(0.016));
}

/// @doc: Delta drains on read — calling `delta()` twice without advancing returns zero, preventing double-counting
#[test]
fn when_fake_clock_delta_called_twice_then_second_call_returns_zero() {
    // Arrange
    let mut clock = FakeClock::default();
    clock.advance(Seconds(0.016));
    clock.delta();

    // Act
    let second = clock.delta();

    // Assert
    assert_eq!(second, Seconds(0.0));
}

#[test]
fn when_fake_clock_advanced_multiple_times_then_delta_accumulates() {
    // Arrange
    let mut clock = FakeClock::default();

    // Act
    clock.advance(Seconds(0.1));
    clock.advance(Seconds(0.1));
    clock.advance(Seconds(0.1));

    // Assert
    let dt = clock.delta().0;
    assert!((dt - 0.3).abs() < f32::EPSILON * 4.0);
}

#[test]
fn when_fake_clock_behind_dyn_time_then_delta_is_correct() {
    // Arrange
    let mut clock = FakeClock::default();
    clock.advance(Seconds(0.5));
    let dyn_clock: &mut dyn Time = &mut clock;

    // Act
    let dt = dyn_clock.delta();

    // Assert
    assert_eq!(dt, Seconds(0.5));
}

#[test]
fn when_clock_res_derefmut_then_reaches_inner_delta() {
    // Arrange
    let mut fake = FakeClock::default();
    fake.advance(Seconds(0.25));
    let mut clock_res = ClockRes::new(Box::new(fake));

    // Act
    let dt = clock_res.delta();

    // Assert
    assert_eq!(dt, Seconds(0.25));
}

/// @doc: Sub-step deltas accumulate silently — no simulation steps fire until a full `step_size` is reached
#[test]
fn when_tick_below_step_size_then_returns_zero_steps() {
    // Arrange
    let mut ts = FixedTimestep::with_step_size(Seconds(0.016));

    // Act
    let steps = ts.tick(Seconds(0.010));

    // Assert
    assert_eq!(steps, 0);
}

#[test]
fn when_tick_exactly_one_step_then_returns_one_step() {
    // Arrange
    let mut ts = FixedTimestep::with_step_size(Seconds(0.016));

    // Act
    let steps = ts.tick(Seconds(0.016));

    // Assert
    assert_eq!(steps, 1);
    assert!(ts.accumulator.0.abs() < f32::EPSILON);
}

/// @doc: Fix Your Timestep pattern — large frame deltas produce multiple fixed steps with leftover accumulated for the next frame
#[test]
fn when_tick_large_delta_then_returns_multiple_steps_and_retains_remainder() {
    // Arrange
    let mut ts = FixedTimestep::with_step_size(Seconds(0.016));

    // Act
    let steps = ts.tick(Seconds(0.050));

    // Assert
    assert_eq!(steps, 3);
    let remainder = ts.accumulator.0;
    assert!((remainder - 0.002).abs() < f32::EPSILON * 10.0);
}

/// @doc: Accumulator carries sub-step remainder across frames, ensuring no simulation time is lost
#[test]
fn when_tick_across_frames_then_accumulator_carries_forward() {
    // Arrange — use binary-exact fractions to avoid f32 rounding
    let mut ts = FixedTimestep::with_step_size(Seconds(0.25));
    ts.tick(Seconds(0.375)); // 1 step, remainder 0.125

    // Act
    let steps = ts.tick(Seconds(0.125)); // 0.125 + 0.125 = 0.25 → 1 step

    // Assert
    assert_eq!(steps, 1);
    assert!(ts.accumulator.0.abs() < f32::EPSILON);
}

#[test]
fn when_time_system_runs_then_delta_time_updated_from_clock() {
    // Arrange
    let mut world = bevy_ecs::world::World::new();
    let mut fake = FakeClock::default();
    fake.advance(Seconds(0.016));
    world.insert_resource(ClockRes::new(Box::new(fake)));
    world.insert_resource(DeltaTime::default());
    let mut schedule = bevy_ecs::schedule::Schedule::default();
    schedule.add_systems(time_system);

    // Act
    schedule.run(&mut world);

    // Assert
    let dt = world.resource::<DeltaTime>();
    assert_eq!(dt.0, Seconds(0.016));
}

proptest::proptest! {
    #[test]
    fn when_any_positive_delta_and_step_size_then_accumulator_stays_below_step_size(
        step_size in 0.001_f32..=1.0,
        delta in 0.0_f32..=2.0,
    ) {
        // Arrange
        let mut ts = FixedTimestep::with_step_size(Seconds(step_size));

        // Act
        let _steps = ts.tick(Seconds(delta));

        // Assert
        assert!(
            ts.accumulator.0 >= 0.0,
            "accumulator should be non-negative, got {}",
            ts.accumulator.0
        );
        assert!(
            ts.accumulator.0 < step_size + f32::EPSILON * 16.0,
            "accumulator {} should be < step_size {}",
            ts.accumulator.0,
            step_size
        );
    }
}

/// @doc: Without clock advance between frames, delta is zero — prevents phantom time from stale clock state
#[test]
fn when_time_system_runs_twice_without_advance_then_second_delta_is_zero() {
    // Arrange
    let mut world = bevy_ecs::world::World::new();
    let mut fake = FakeClock::default();
    fake.advance(Seconds(0.016));
    world.insert_resource(ClockRes::new(Box::new(fake)));
    world.insert_resource(DeltaTime::default());
    let mut schedule = bevy_ecs::schedule::Schedule::default();
    schedule.add_systems(time_system);

    // Act — frame 1 consumes the advance, frame 2 has nothing
    schedule.run(&mut world);
    schedule.run(&mut world);

    // Assert
    assert_eq!(world.resource::<DeltaTime>().0, Seconds(0.0));
}
