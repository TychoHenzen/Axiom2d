# Frame Profiler Design

**Date:** 2026-04-02  
**Status:** Approved  
**Scope:** `engine_core`, `engine_app`, `card_game_bin`

## Problem

Two observed symptoms:
1. Idle scene (no cards) consumes ~10% CPU — caused by `about_to_wait` calling `request_redraw()` unconditionally, running all 5 schedules at maximum rate even when nothing changes.
2. ~50 cards on screen drops to <10 fps — hot path unknown without instrumentation.

No timing or profiling infrastructure exists today. We need structured per-frame data to diagnose both issues.

## Solution Overview

Add a `FrameProfiler` ECS resource to `engine_core` that:
- Automatically times each of the 5 schedule phases in `App::handle_redraw()`
- Exposes a `span()` method for manual hot-path timing in individual systems
- Flushes a CSV log to disk every N frames for external analysis

No new dependencies — uses `std::time::Instant` and `std::fs`.

## Architecture

### Components

**`engine_core::profiler`** — new module  
Contains `FrameProfiler`, `ProfileScope`, and the CSV flush logic. Single responsibility: accumulate timing records and periodically write them to disk.

**`App::handle_redraw()` in `engine_app`** — modified  
Wraps each `schedule.run(&mut self.world)` with `Instant` measurements. After each schedule's borrow ends, borrows the profiler resource and calls `record_phase()`. Calls `end_frame()` after all phases complete.

**`FrameProfilerPlugin` in `engine_app`** — new  
A `Plugin` impl that inserts `FrameProfiler` into `World`. Registered in `card_game_bin/src/main.rs` like any other plugin.

### Data Flow

```
App::handle_redraw()
  for each Phase:
    start = Instant::now()
    schedule.run(&mut world)          // borrow ends
    elapsed_us = start.elapsed()
    profiler.record_phase(name, elapsed_us)

  renderer.present()
  profiler.end_frame()                // increments frame, flushes if due

System with hot path:
  let _s = profiler.span("render_sort")
  // ... work ...
  // _s drops → records elapsed_us into profiler
```

## API

```rust
// engine_core::profiler

pub struct FrameProfiler { /* private */ }

impl FrameProfiler {
    /// Create a new profiler.
    ///
    /// `flush_interval`: number of frames between CSV flushes (default: 120).
    /// `output_path`: path to the CSV file (default: "perf_log.csv").
    ///
    /// ## Analyzing the output
    /// ```python
    /// import pandas as pd
    /// df = pd.read_csv("perf_log.csv")
    /// print(df.groupby("scope")["duration_us"].agg(["median", lambda x: x.quantile(0.99)]))
    /// ```
    pub fn new(flush_interval: u64, output_path: PathBuf) -> Self;

    /// Record a phase duration measured externally (called by App::handle_redraw).
    pub fn record_phase(&mut self, name: &'static str, duration_us: u64);

    /// Start a named span. Records elapsed time when the returned guard is dropped.
    ///
    /// Spans cannot be nested — the `&mut` borrow prevents holding two guards
    /// simultaneously. Drop the first guard before starting the next.
    pub fn span(&mut self, name: &'static str) -> ProfileScope<'_>;

    /// Advance the frame counter and flush to disk if the interval has elapsed.
    pub fn end_frame(&mut self);
}

pub struct ProfileScope<'a> {
    profiler: &'a mut FrameProfiler,
    name: &'static str,
    start: std::time::Instant,
}

impl Drop for ProfileScope<'_> {
    fn drop(&mut self) {
        let duration_us = self.start.elapsed().as_micros() as u64;
        self.profiler.records.push(Record {
            frame: self.profiler.frame,
            scope: self.name,
            duration_us,
        });
    }
}
```

### System Usage Pattern

```rust
fn unified_render_system(
    mut profiler: ResMut<FrameProfiler>,
    // ... other params
) {
    let _s = profiler.span("render_sort");
    // sort work
    drop(_s);

    let _s = profiler.span("render_draw");
    // draw work
}
```

### Plugin Registration

```rust
// card_game_bin/src/main.rs
app.add_plugin(FrameProfilerPlugin {
    flush_interval: 120,
    output_path: "perf_log.csv".into(),
});
```

## CSV Output

**Format:** `frame,scope,duration_us` — one row per measurement per frame. Phase timings and manual spans share the same table, distinguishable by the `scope` column.

**Example:**
```
frame,scope,duration_us
0,Input,12
0,PreUpdate,45
0,Update,8203
0,PostUpdate,31
0,Render,14502
0,render_sort,7841
0,render_draw,6201
1,Input,9
...
```

**File behavior:**
- Opens in append mode, creates if absent
- Writes header row only if the file is empty on open
- Never truncates — accumulates a full session log
- Flush interval default: 120 frames (~2s at 60fps)

**Error handling:** File I/O errors print to stderr via `eprintln!` and are silently skipped. A profiling failure must never crash the game.

## Implementation Plan (files to touch)

| File | Change |
|------|--------|
| `crates/engine_core/src/profiler.rs` | New — `FrameProfiler`, `ProfileScope`, `Record`, CSV flush |
| `crates/engine_core/src/lib.rs` | Add `pub mod profiler` |
| `crates/engine_core/src/prelude.rs` | Re-export `FrameProfiler`, `ProfileScope` |
| `crates/engine_app/src/app.rs` | Wrap phase runs with Instant, call `record_phase` + `end_frame` |
| `crates/engine_app/src/lib.rs` | Add `FrameProfilerPlugin` |
| `crates/engine_app/src/prelude.rs` | Re-export `FrameProfilerPlugin` |
| `crates/card_game_bin/src/main.rs` | Register `FrameProfilerPlugin` |
| `crates/engine_ui/src/unified_render.rs` | Add manual spans: `render_sort`, `render_draw` |
| `crates/engine_physics/src/physics_step_system.rs` | Add manual span: `physics_step` |

## Constraints and Non-Goals

- **No nested spans** — the `&'_ mut FrameProfiler` borrow prevents it. Sequential spans are sufficient for the current hot paths. Nesting can be added later.
- **No in-game overlay** — out of scope for this iteration. CSV output is sufficient for diagnosis.
- **No feature flag** — the profiler runs unconditionally for now. A `profiling` feature flag can be added once the hot paths are understood and fixed.
- **No system-level bevy_ecs instrumentation** — individual system timing requires middleware hooks not exposed by the current `bevy_ecs` schedule API. Phase + manual spans cover the known hot paths.
