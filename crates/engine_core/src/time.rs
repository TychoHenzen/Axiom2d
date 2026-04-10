// EVOLVE-BLOCK-START
use std::ops::{Deref, DerefMut};

use bevy_ecs::prelude::{ResMut, Resource};

use crate::types::Seconds;

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct DeltaTime(pub Seconds);

impl Default for DeltaTime {
    fn default() -> Self {
        Self(Seconds(0.0))
    }
}

pub trait Time: Send + Sync {
    fn delta(&mut self) -> Seconds;
}

pub struct FakeClock {
    pending: Seconds,
}

impl Default for FakeClock {
    fn default() -> Self {
        Self {
            pending: Seconds(0.0),
        }
    }
}

impl FakeClock {
    pub fn advance(&mut self, dt: Seconds) {
        self.pending = self.pending + dt;
    }
}

impl Time for FakeClock {
    fn delta(&mut self) -> Seconds {
        let dt = self.pending;
        self.pending = Seconds(0.0);
        dt
    }
}

/// Returns the same fixed delta on every call — use in integration tests that need predictable per-frame time.
pub struct FixedDeltaClock {
    dt: Seconds,
}

impl FixedDeltaClock {
    pub fn new(dt: Seconds) -> Self {
        Self { dt }
    }
}

impl Time for FixedDeltaClock {
    fn delta(&mut self) -> Seconds {
        self.dt
    }
}

pub struct SystemClock {
    last_instant: std::time::Instant,
}

impl Default for SystemClock {
    fn default() -> Self {
        Self {
            last_instant: std::time::Instant::now(),
        }
    }
}

impl Time for SystemClock {
    fn delta(&mut self) -> Seconds {
        let now = std::time::Instant::now();
        let dt = now.duration_since(self.last_instant).as_secs_f32();
        self.last_instant = now;
        Seconds(dt)
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

pub fn time_system(mut clock: ResMut<ClockRes>, mut dt: ResMut<DeltaTime>) {
    dt.0 = clock.delta();
}
// EVOLVE-BLOCK-END
