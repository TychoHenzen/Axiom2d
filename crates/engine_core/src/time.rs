use std::ops::{Deref, DerefMut};

use bevy_ecs::prelude::{Res, ResMut, Resource};

use crate::types::Seconds;

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct DeltaTime(pub Seconds);

impl Default for DeltaTime {
    fn default() -> Self {
        Self(Seconds(0.0))
    }
}

pub trait Time: Send + Sync {
    fn elapsed(&self) -> Seconds;
}

pub struct FakeClock {
    elapsed: Seconds,
}

impl FakeClock {
    pub fn new() -> Self {
        Self {
            elapsed: Seconds(0.0),
        }
    }

    pub fn advance(&mut self, dt: Seconds) {
        self.elapsed = self.elapsed + dt;
    }
}

impl Time for FakeClock {
    fn elapsed(&self) -> Seconds {
        self.elapsed
    }
}

pub struct SystemClock {
    last_instant: std::time::Instant,
}

impl SystemClock {
    pub fn new() -> Self {
        Self {
            last_instant: std::time::Instant::now(),
        }
    }
}

impl Time for SystemClock {
    fn elapsed(&self) -> Seconds {
        Seconds(self.last_instant.elapsed().as_secs_f32())
    }
}

#[derive(Resource)]
pub struct ClockRes(pub Box<dyn Time>);

impl ClockRes {
    pub fn new(clock: Box<dyn Time>) -> Self {
        Self(clock)
    }
}

impl Deref for ClockRes {
    type Target = dyn Time;
    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl DerefMut for ClockRes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.0
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct FixedTimestep {
    pub accumulator: Seconds,
    pub step_size: Seconds,
}

impl Default for FixedTimestep {
    fn default() -> Self {
        Self {
            accumulator: Seconds(0.0),
            step_size: Seconds(1.0 / 60.0),
        }
    }
}

impl FixedTimestep {
    pub fn with_step_size(step_size: Seconds) -> Self {
        Self {
            accumulator: Seconds(0.0),
            step_size,
        }
    }

    pub fn tick(&mut self, delta: Seconds) -> u32 {
        self.accumulator = self.accumulator + delta;
        let steps = (self.accumulator.0 / self.step_size.0) as u32;
        self.accumulator = Seconds(self.accumulator.0 - steps as f32 * self.step_size.0);
        steps
    }
}

pub fn time_system(clock: Res<ClockRes>, mut dt: ResMut<DeltaTime>) {
    dt.0 = clock.elapsed();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_constructed_then_stores_seconds_value() {
        // Act
        let dt = DeltaTime(Seconds(0.016));

        // Assert
        assert_eq!(dt.0, Seconds(0.016));
    }

    #[test]
    fn when_copied_then_supports_copy_eq_and_debug() {
        // Arrange
        let a = DeltaTime(Seconds(0.016));

        // Act
        let b = a;

        // Assert
        assert_eq!(a, b);
        let s = format!("{:?}", a);
        assert!(s.contains("0.016"));
    }

    #[test]
    fn when_default_then_value_is_zero() {
        // Act
        let dt = DeltaTime::default();

        // Assert
        assert_eq!(dt.0, Seconds(0.0));
    }

    #[test]
    fn when_inserted_into_world_then_retrievable_as_resource() {
        // Arrange
        let mut world = bevy_ecs::world::World::new();

        // Act
        world.insert_resource(DeltaTime(Seconds(0.016)));

        // Assert
        let dt = world.resource::<DeltaTime>();
        assert_eq!(dt.0, Seconds(0.016));
    }

    // --- Time trait + FakeClock + ClockRes ---

    #[test]
    fn when_fake_clock_constructed_then_elapsed_is_zero() {
        // Act
        let clock = FakeClock::new();

        // Assert
        assert_eq!(clock.elapsed(), Seconds(0.0));
    }

    #[test]
    fn when_fake_clock_advanced_then_elapsed_reflects_advancement() {
        // Arrange
        let mut clock = FakeClock::new();

        // Act
        clock.advance(Seconds(0.016));

        // Assert
        assert_eq!(clock.elapsed(), Seconds(0.016));
    }

    #[test]
    fn when_fake_clock_advanced_multiple_times_then_elapsed_accumulates() {
        // Arrange
        let mut clock = FakeClock::new();

        // Act
        clock.advance(Seconds(0.1));
        clock.advance(Seconds(0.1));
        clock.advance(Seconds(0.1));

        // Assert
        let elapsed = clock.elapsed().0;
        assert!((elapsed - 0.3).abs() < f32::EPSILON * 4.0);
    }

    #[test]
    fn when_fake_clock_behind_dyn_time_then_elapsed_is_correct() {
        // Arrange
        let mut clock = FakeClock::new();
        clock.advance(Seconds(0.5));
        let dyn_clock: &dyn Time = &clock;

        // Act
        let elapsed = dyn_clock.elapsed();

        // Assert
        assert_eq!(elapsed, Seconds(0.5));
    }

    #[test]
    fn when_clock_res_inserted_into_world_then_retrievable_as_resource() {
        // Arrange
        let mut world = bevy_ecs::world::World::new();
        let clock = ClockRes::new(Box::new(FakeClock::new()));

        // Act
        world.insert_resource(clock);

        // Assert
        assert!(world.get_resource::<ClockRes>().is_some());
    }

    #[test]
    fn when_clock_res_derefed_then_reaches_inner_elapsed() {
        // Arrange
        let mut fake = FakeClock::new();
        fake.advance(Seconds(0.25));
        let clock_res = ClockRes::new(Box::new(fake));

        // Act
        let elapsed = clock_res.elapsed();

        // Assert
        assert_eq!(elapsed, Seconds(0.25));
    }

    // --- FixedTimestep ---

    #[test]
    fn when_fixed_timestep_default_then_accumulator_zero_and_step_size_60fps() {
        // Act
        let ts = FixedTimestep::default();

        // Assert
        assert_eq!(ts.accumulator, Seconds(0.0));
        assert!((ts.step_size.0 - 1.0 / 60.0).abs() < f32::EPSILON * 10.0);
    }

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
    fn when_fixed_timestep_inserted_into_world_then_retrievable_as_resource() {
        // Arrange
        let mut world = bevy_ecs::world::World::new();

        // Act
        world.insert_resource(FixedTimestep::default());

        // Assert
        assert!(world.get_resource::<FixedTimestep>().is_some());
    }

    // --- time_system ---

    #[test]
    fn when_time_system_runs_then_delta_time_updated_from_clock() {
        // Arrange
        let mut world = bevy_ecs::world::World::new();
        let mut fake = FakeClock::new();
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
}
