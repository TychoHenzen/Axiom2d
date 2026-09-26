use std::time::Instant;

use engine_core::profiler::FrameProfiler;
use engine_ecs::prelude::{PHASE_COUNT, Phase, Schedule, World};

pub(crate) fn run_phase(
    schedules: &mut [Schedule; PHASE_COUNT],
    world: &mut World,
    startup_executed: &mut bool,
    phase: Phase,
    fixed_steps: u32,
) {
    match phase {
        Phase::Startup if *startup_executed => {}
        Phase::Startup => {
            run_schedule(schedules, world, phase);
            *startup_executed = true;
        }
        Phase::FixedUpdate => {
            for _ in 0..fixed_steps {
                run_schedule(schedules, world, phase);
            }
        }
        _ => run_schedule(schedules, world, phase),
    }
}

fn run_schedule(schedules: &mut [Schedule; PHASE_COUNT], world: &mut World, phase: Phase) {
    let start = Instant::now();
    schedules[phase.index()].run(world);
    let elapsed_us = start.elapsed().as_micros() as u64;
    if let Some(mut profiler) = world.get_resource_mut::<FrameProfiler>() {
        profiler.record_phase(phase.name(), elapsed_us);
    }
}
