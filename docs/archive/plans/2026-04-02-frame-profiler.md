# Frame Profiler Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a `FrameProfiler` ECS resource that automatically times each schedule phase and exposes manual span recording, flushing structured CSV output to disk for external analysis.

**Architecture:** A `FrameProfiler` resource in `engine_core::profiler` accumulates `(frame, scope, duration_us)` records. `App::handle_redraw()` wraps each `schedule.run()` call with `Instant` measurements and calls `record_phase()` after each. Hot-path systems add `Option<ResMut<FrameProfiler>>` and record spans manually. Every N frames, records flush to a CSV file.

**Tech Stack:** `std::time::Instant`, `std::fs::OpenOptions`, `bevy_ecs::prelude::Resource`, no new external dependencies.

---

## File Map

| File | Action | Responsibility |
|------|--------|----------------|
| `crates/engine_ecs/src/schedule.rs` | Modify | Add `Phase::name() -> &'static str` |
| `crates/engine_core/src/profiler.rs` | Create | `FrameProfiler`, `ProfileScope`, `Record`, CSV flush |
| `crates/engine_core/src/lib.rs` | Modify | Add `pub mod profiler` |
| `crates/engine_core/src/prelude.rs` | Modify | Re-export `FrameProfiler`, `ProfileScope` |
| `crates/engine_core/tests/suite/profiler.rs` | Create | Tests for profiler behavior |
| `crates/engine_core/tests/suite/mod.rs` | Modify | Register `mod profiler` |
| `crates/engine_app/src/profiler_plugin.rs` | Create | `FrameProfilerPlugin` struct + `Plugin` impl |
| `crates/engine_app/src/lib.rs` | Modify | Add `pub mod profiler_plugin` |
| `crates/engine_app/src/prelude.rs` | Modify | Re-export `FrameProfilerPlugin` |
| `crates/engine_app/src/app.rs` | Modify | Wrap phases with timing, call `end_frame()` |
| `crates/engine_app/tests/suite/app.rs` | Modify | Add test for phase recording |
| `crates/engine_physics/src/physics_step_system.rs` | Modify | Add `Option<ResMut<FrameProfiler>>` + manual span |
| `crates/engine_ui/src/unified_render.rs` | Modify | Add `Option<ResMut<FrameProfiler>>` + two spans |
| `crates/card_game_bin/src/main.rs` | Modify | Register `FrameProfilerPlugin` in `setup()` |

---

## Task 1: Add `Phase::name()` to `engine_ecs`

**Files:**
- Modify: `crates/engine_ecs/src/schedule.rs`

- [ ] **Step 1: Add `name()` to `Phase`**

Open `crates/engine_ecs/src/schedule.rs`. The file currently ends after the `index()` method. Add `name()` inside the `impl Phase` block:

```rust
use bevy_ecs::schedule::ScheduleLabel;

pub const PHASE_COUNT: usize = 5;

#[derive(ScheduleLabel, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Phase {
    Input,
    PreUpdate,
    Update,
    PostUpdate,
    Render,
}

impl Phase {
    pub const ALL: [Self; PHASE_COUNT] = [
        Self::Input,
        Self::PreUpdate,
        Self::Update,
        Self::PostUpdate,
        Self::Render,
    ];

    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Input => "Input",
            Self::PreUpdate => "PreUpdate",
            Self::Update => "Update",
            Self::PostUpdate => "PostUpdate",
            Self::Render => "Render",
        }
    }
}
```

- [ ] **Step 2: Verify build**

```
cargo.exe check -p engine_ecs
```

Expected: no errors.

- [ ] **Step 3: Commit**

```
git add crates/engine_ecs/src/schedule.rs
git commit -m "feat(engine-ecs): add Phase::name() for profiler phase labels"
```

---

## Task 2: Create `engine_core::profiler` module

**Files:**
- Create: `crates/engine_core/src/profiler.rs`
- Create: `crates/engine_core/tests/suite/profiler.rs`
- Modify: `crates/engine_core/tests/suite/mod.rs`

- [ ] **Step 1: Write the failing tests**

Create `crates/engine_core/tests/suite/profiler.rs`:

```rust
#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

use engine_core::profiler::FrameProfiler;

// ── record_phase ─────────────────────────────────────────────────────────────

#[test]
fn when_record_phase_called_then_record_count_increases() {
    // Arrange
    let mut profiler = FrameProfiler::new(9999, PathBuf::from("unused.csv"));

    // Act
    profiler.record_phase("Input", 100);
    profiler.record_phase("Update", 200);

    // Assert
    assert_eq!(profiler.record_count(), 2);
}

// ── span ─────────────────────────────────────────────────────────────────────

#[test]
fn when_span_dropped_then_record_count_increases() {
    // Arrange
    let mut profiler = FrameProfiler::new(9999, PathBuf::from("unused.csv"));

    // Act
    {
        let _s = profiler.span("my_span");
    }

    // Assert
    assert_eq!(profiler.record_count(), 1);
}

// ── end_frame / flush ─────────────────────────────────────────────────────────

#[test]
fn when_end_frame_reaches_flush_interval_then_csv_written() {
    // Arrange
    let path = std::env::temp_dir().join("axiom2d_profiler_test_flush.csv");
    let _ = std::fs::remove_file(&path);
    let mut profiler = FrameProfiler::new(1, path.clone());
    profiler.record_phase("Input", 100);
    profiler.record_phase("Update", 200);

    // Act — frame increments to 1, 1 % 1 == 0, triggers flush
    profiler.end_frame();

    // Assert
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("frame,scope,duration_us"), "missing header");
    assert!(content.contains("0,Input,100"), "missing Input record");
    assert!(content.contains("0,Update,200"), "missing Update record");

    let _ = std::fs::remove_file(&path);
}

#[test]
fn when_flush_occurs_then_in_memory_records_cleared() {
    // Arrange
    let path = std::env::temp_dir().join("axiom2d_profiler_test_clear.csv");
    let _ = std::fs::remove_file(&path);
    let mut profiler = FrameProfiler::new(1, path.clone());
    profiler.record_phase("Input", 100);

    // Act
    profiler.end_frame();

    // Assert
    assert_eq!(profiler.record_count(), 0);

    let _ = std::fs::remove_file(&path);
}

#[test]
fn when_flush_interval_not_reached_then_records_not_cleared() {
    // Arrange
    let mut profiler = FrameProfiler::new(10, PathBuf::from("unused.csv"));
    profiler.record_phase("Input", 100);

    // Act — frame becomes 1, 1 % 10 != 0
    profiler.end_frame();

    // Assert
    assert_eq!(profiler.record_count(), 1);
}

#[test]
fn when_flush_path_invalid_then_no_panic() {
    // Arrange — path inside a non-existent directory
    let path = std::env::temp_dir()
        .join("axiom2d_no_such_dir_xyz")
        .join("profiler.csv");
    let mut profiler = FrameProfiler::new(1, path);
    profiler.record_phase("Input", 100);

    // Act + Assert — IO error must not panic
    profiler.end_frame();
}

#[test]
fn when_second_flush_appends_to_existing_file_without_duplicate_header() {
    // Arrange
    let path = std::env::temp_dir().join("axiom2d_profiler_test_append.csv");
    let _ = std::fs::remove_file(&path);
    let mut profiler = FrameProfiler::new(1, path.clone());
    profiler.record_phase("Input", 100);
    profiler.end_frame(); // frame 1 → flush

    profiler.record_phase("Update", 200);
    profiler.end_frame(); // frame 2 → flush

    // Assert — header appears exactly once
    let content = std::fs::read_to_string(&path).unwrap();
    let header_count = content.matches("frame,scope,duration_us").count();
    assert_eq!(header_count, 1, "header must appear exactly once");
    assert!(content.contains("1,Update,200"), "second flush records frame 1");

    let _ = std::fs::remove_file(&path);
}
```

- [ ] **Step 2: Register the test module**

Open `crates/engine_core/tests/suite/mod.rs`. Add `mod profiler;` to the existing list:

```rust
mod color;
mod event_bus;
mod profiler;
mod scale_spring;
mod spring;
mod time;
mod transform;
mod types;
```

- [ ] **Step 3: Run tests to confirm they fail**

```
cargo.exe test -p engine_core profiler
```

Expected: compile error — `module 'profiler' not found` or `FrameProfiler` not found.

- [ ] **Step 4: Implement `FrameProfiler`**

Create `crates/engine_core/src/profiler.rs`:

```rust
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

use bevy_ecs::prelude::Resource;

struct Record {
    frame: u64,
    scope: &'static str,
    duration_us: u64,
}

/// Per-frame performance profiler. Insert as an ECS resource via [`FrameProfilerPlugin`].
///
/// Records phase timings (set by `App::handle_redraw`) and manual spans from systems.
/// Flushes to CSV every `flush_interval` frames.
///
/// ## Analysing the output
/// ```python
/// import pandas as pd
/// df = pd.read_csv("perf_log.csv")
/// print(df.groupby("scope")["duration_us"].agg(["median", lambda x: x.quantile(0.99)]))
/// ```
#[derive(Resource)]
pub struct FrameProfiler {
    frame: u64,
    flush_interval: u64,
    records: Vec<Record>,
    output_path: PathBuf,
}

impl FrameProfiler {
    /// `flush_interval` must be >= 1. Default: 120 (≈2 s at 60 fps).
    pub fn new(flush_interval: u64, output_path: PathBuf) -> Self {
        assert!(flush_interval >= 1, "flush_interval must be >= 1");
        Self {
            frame: 0,
            flush_interval,
            records: Vec::new(),
            output_path,
        }
    }

    /// Record a named duration in microseconds. Called by `App::handle_redraw` for
    /// phase timings; also usable inside systems for manual instrumentation.
    pub fn record_phase(&mut self, name: &'static str, duration_us: u64) {
        self.records.push(Record {
            frame: self.frame,
            scope: name,
            duration_us,
        });
    }

    /// Start a named span. The elapsed time is recorded when the returned guard is dropped.
    ///
    /// Spans cannot be nested — the `&mut` borrow prevents holding two guards simultaneously.
    /// Drop the first guard (or use `drop(span)`) before starting another.
    pub fn span(&mut self, name: &'static str) -> ProfileScope<'_> {
        ProfileScope {
            profiler: self,
            name,
            start: Instant::now(),
        }
    }

    /// Advance the frame counter. Flushes to disk when `frame % flush_interval == 0`.
    /// Call once per frame, after all schedules have run.
    pub fn end_frame(&mut self) {
        self.frame += 1;
        if self.frame % self.flush_interval == 0 {
            self.flush();
        }
    }

    /// Number of buffered records not yet flushed. Used in tests.
    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    fn flush(&mut self) {
        if let Err(e) = self.write_records() {
            eprintln!("[FrameProfiler] flush error: {e}");
        }
        self.records.clear();
    }

    fn write_records(&self) -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.output_path)?;

        if file.metadata()?.len() == 0 {
            writeln!(file, "frame,scope,duration_us")?;
        }

        for r in &self.records {
            writeln!(file, "{},{},{}", r.frame, r.scope, r.duration_us)?;
        }

        Ok(())
    }
}

/// Drop guard that records elapsed time into a [`FrameProfiler`] on drop.
/// Obtained from [`FrameProfiler::span`].
pub struct ProfileScope<'a> {
    profiler: &'a mut FrameProfiler,
    name: &'static str,
    start: Instant,
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

- [ ] **Step 5: Run tests to confirm they pass**

```
cargo.exe test -p engine_core profiler
```

Expected: all 6 tests pass.

- [ ] **Step 6: Commit**

```
git add crates/engine_core/src/profiler.rs crates/engine_core/tests/suite/profiler.rs crates/engine_core/tests/suite/mod.rs
git commit -m "feat(engine-core): add FrameProfiler with CSV flush"
```

---

## Task 3: Export `FrameProfiler` from `engine_core`

**Files:**
- Modify: `crates/engine_core/src/lib.rs`
- Modify: `crates/engine_core/src/prelude.rs`

- [ ] **Step 1: Register the module**

Open `crates/engine_core/src/lib.rs`. Add `pub mod profiler;` to the list:

```rust
pub mod color;
pub mod error;
pub mod event_bus;
pub mod prelude;
pub mod profiler;
pub mod scale_spring;
pub mod spring;
pub mod time;
pub mod transform;
pub mod types;
```

- [ ] **Step 2: Re-export from prelude**

Open `crates/engine_core/src/prelude.rs`. Add the profiler exports:

```rust
pub use crate::color::Color;
pub use crate::error::EngineError;
pub use crate::event_bus::{Event, EventBus};
pub use crate::profiler::{FrameProfiler, ProfileScope};
pub use crate::scale_spring::{ScaleSpring, scale_spring_system};
pub use crate::spring::spring_step;
pub use crate::time::{ClockRes, DeltaTime, FixedTimestep, SystemClock, time_system};
pub use crate::transform::Transform2D;
pub use crate::types::{Pixels, Seconds, TextureId};
pub use glam::{Affine2, Vec2};
```

- [ ] **Step 3: Verify full engine_core build and tests**

```
cargo.exe test -p engine_core
```

Expected: all tests pass (no regressions).

- [ ] **Step 4: Commit**

```
git add crates/engine_core/src/lib.rs crates/engine_core/src/prelude.rs
git commit -m "feat(engine-core): export FrameProfiler and ProfileScope from prelude"
```

---

## Task 4: Create `FrameProfilerPlugin` in `engine_app`

**Files:**
- Create: `crates/engine_app/src/profiler_plugin.rs`
- Modify: `crates/engine_app/src/lib.rs`
- Modify: `crates/engine_app/src/prelude.rs`

- [ ] **Step 1: Create the plugin**

Create `crates/engine_app/src/profiler_plugin.rs`:

```rust
use std::path::PathBuf;

use engine_core::profiler::FrameProfiler;

use crate::app::{App, Plugin};

/// Inserts a [`FrameProfiler`] resource into the app.
///
/// Register in `setup()` before calling `app.run()`:
/// ```ignore
/// app.add_plugin(FrameProfilerPlugin::default());
/// ```
pub struct FrameProfilerPlugin {
    /// Frames between CSV flushes. Must be >= 1. Default: 120.
    pub flush_interval: u64,
    /// Path to the output CSV file. Default: `"perf_log.csv"`.
    pub output_path: PathBuf,
}

impl Default for FrameProfilerPlugin {
    fn default() -> Self {
        Self {
            flush_interval: 120,
            output_path: PathBuf::from("perf_log.csv"),
        }
    }
}

impl Plugin for FrameProfilerPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().insert_resource(FrameProfiler::new(
            self.flush_interval,
            self.output_path.clone(),
        ));
    }
}
```

- [ ] **Step 2: Register the module in `lib.rs`**

Open `crates/engine_app/src/lib.rs`. Add `pub mod profiler_plugin;`:

```rust
pub mod app;
pub mod mouse_world_pos_system;
pub mod prelude;
pub mod profiler_plugin;
pub mod window_size;
```

- [ ] **Step 3: Re-export from prelude**

Open `crates/engine_app/src/prelude.rs`. Add the plugin export:

```rust
pub use crate::app::{App, Plugin};
pub use crate::mouse_world_pos_system::mouse_world_pos_system;
pub use crate::profiler_plugin::FrameProfilerPlugin;
pub use crate::window_size::WindowSize;
pub use engine_ecs::prelude::{Phase, World};
```

- [ ] **Step 4: Verify build**

```
cargo.exe check -p engine_app
```

Expected: no errors.

- [ ] **Step 5: Commit**

```
git add crates/engine_app/src/profiler_plugin.rs crates/engine_app/src/lib.rs crates/engine_app/src/prelude.rs
git commit -m "feat(engine-app): add FrameProfilerPlugin"
```

---

## Task 5: Time phases in `App::handle_redraw()`

**Files:**
- Modify: `crates/engine_app/src/app.rs`
- Modify: `crates/engine_app/tests/suite/app.rs`

- [ ] **Step 1: Write the failing test**

Open `crates/engine_app/tests/suite/app.rs`. Add this test at the bottom:

```rust
/// @doc: Phase timing records five entries per frame — one per schedule phase
#[test]
fn when_handle_redraw_called_with_profiler_then_five_phase_records_buffered() {
    use std::path::PathBuf;

    // Arrange
    let mut app = App::new();
    app.world_mut().insert_resource(
        engine_core::profiler::FrameProfiler::new(9999, PathBuf::from("unused.csv")),
    );

    // Act
    app.handle_redraw();

    // Assert — one record per phase (flush_interval 9999 keeps them in memory)
    let profiler = app
        .world()
        .resource::<engine_core::profiler::FrameProfiler>();
    assert_eq!(profiler.record_count(), 5);
}
```

- [ ] **Step 2: Run to confirm it fails**

```
cargo.exe test -p engine_app when_handle_redraw_called_with_profiler
```

Expected: FAIL — `record_count()` returns 0 (timing not wired yet).

- [ ] **Step 3: Add timing to `handle_redraw()`**

Open `crates/engine_app/src/app.rs`. Add this import near the top with the other `use` statements:

```rust
use engine_core::profiler::FrameProfiler;
```

Replace the `handle_redraw` method (currently lines 159–166):

```rust
pub fn handle_redraw(&mut self) {
    for (i, schedule) in self.schedules.iter_mut().enumerate() {
        let start = std::time::Instant::now();
        schedule.run(&mut self.world);
        let elapsed_us = start.elapsed().as_micros() as u64;
        if let Some(mut profiler) = self.world.get_resource_mut::<FrameProfiler>() {
            profiler.record_phase(Phase::ALL[i].name(), elapsed_us);
        }
    }
    if let Some(mut renderer) = self.world.get_resource_mut::<RendererRes>() {
        renderer.present();
    }
    if let Some(mut profiler) = self.world.get_resource_mut::<FrameProfiler>() {
        profiler.end_frame();
    }
}
```

- [ ] **Step 4: Run to confirm it passes**

```
cargo.exe test -p engine_app when_handle_redraw_called_with_profiler
```

Expected: PASS.

- [ ] **Step 5: Run the full engine_app test suite to check for regressions**

```
cargo.exe test -p engine_app
```

Expected: all tests pass.

- [ ] **Step 6: Commit**

```
git add crates/engine_app/src/app.rs crates/engine_app/tests/suite/app.rs
git commit -m "feat(engine-app): time schedule phases in handle_redraw"
```

---

## Task 6: Manual span in `physics_step_system`

**Files:**
- Modify: `crates/engine_physics/src/physics_step_system.rs`

Existing tests for `physics_step_system` create worlds without a `FrameProfiler`. Using `Option<ResMut<FrameProfiler>>` makes the profiler optional — bevy_ecs passes `None` when the resource is absent. All existing tests will continue to pass unchanged.

- [ ] **Step 1: Add the span**

Replace the full contents of `crates/engine_physics/src/physics_step_system.rs`:

```rust
use bevy_ecs::prelude::{Res, ResMut};
use engine_core::prelude::{DeltaTime, EventBus};
use engine_core::profiler::FrameProfiler;

use crate::collision_event::CollisionEvent;
use crate::physics_res::PhysicsRes;

pub fn physics_step_system(
    dt: Res<DeltaTime>,
    mut physics: ResMut<PhysicsRes>,
    mut events: ResMut<EventBus<CollisionEvent>>,
    mut profiler: Option<ResMut<FrameProfiler>>,
) {
    let t = std::time::Instant::now();
    physics.step(dt.0);
    for event in physics.drain_collision_events() {
        events.push(event);
    }
    if let Some(p) = profiler.as_deref_mut() {
        p.record_phase("physics_step", t.elapsed().as_micros() as u64);
    }
}
```

- [ ] **Step 2: Run existing physics tests to confirm no regressions**

```
cargo.exe test -p engine_physics physics_step_system
```

Expected: all tests pass (the `None` profiler path is exercised silently).

- [ ] **Step 3: Commit**

```
git add crates/engine_physics/src/physics_step_system.rs
git commit -m "feat(engine-physics): record physics_step span in FrameProfiler"
```

---

## Task 7: Manual spans in `unified_render_system`

**Files:**
- Modify: `crates/engine_ui/src/unified_render.rs`

Existing `unified_render` tests create worlds without `FrameProfiler`. Same `Option<ResMut<FrameProfiler>>` pattern — tests pass unchanged.

- [ ] **Step 1: Add the spans**

In `crates/engine_ui/src/unified_render.rs`, add this import at the top with the existing `use` statements:

```rust
use engine_core::profiler::FrameProfiler;
```

Replace the `unified_render_system` function signature and body (lines 113–193):

```rust
/// Unified render system that draws both shapes and text in a single sorted
/// pass, preventing text from rendering on top of shapes that should occlude it.
pub fn unified_render_system(
    shape_query: Query<ShapeItem>,
    text_query: Query<TextItem>,
    color_mesh_query: Query<ColorMeshItem>,
    camera_query: Query<&Camera2D>,
    mut renderer: ResMut<RendererRes>,
    mut cache: Local<GlyphCache>,
    mut profiler: Option<ResMut<FrameProfiler>>,
) {
    let view_rect = compute_view_rect(&camera_query, &renderer);

    let t_sort = std::time::Instant::now();
    let items = collect_draw_items(&shape_query, &text_query, &color_mesh_query);
    let sort_us = t_sort.elapsed().as_micros() as u64;

    let t_draw = std::time::Instant::now();

    let mut last_shader = None;
    let mut last_blend_mode = None;

    for item in &items {
        match item.kind {
            DrawKind::Shape => {
                let Ok((_, shape, transform, _, _, _, mat, stroke, cached)) =
                    shape_query.get(item.entity)
                else {
                    continue;
                };
                if is_shape_culled(transform.0.translation, &shape.variant, view_rect) {
                    continue;
                }
                apply_material(&mut **renderer, mat, &mut last_shader, &mut last_blend_mode);
                let model = affine2_to_mat4(&transform.0);
                if let Some(cached) = cached {
                    renderer.draw_shape(&cached.0.vertices, &cached.0.indices, shape.color, model);
                } else {
                    let Ok(mesh) = tessellate(&shape.variant) else {
                        continue;
                    };
                    renderer.draw_shape(&mesh.vertices, &mesh.indices, shape.color, model);
                }
                if let Some(stroke) = stroke
                    && let Ok(sm) = tessellate_stroke(&shape.variant, stroke.width)
                {
                    renderer.draw_shape(&sm.vertices, &sm.indices, stroke.color, model);
                }
            }
            DrawKind::Text => {
                let Ok((_, text, global_transform, _, _, _)) = text_query.get(item.entity) else {
                    continue;
                };
                draw_text(&mut **renderer, &mut cache, text, global_transform);
            }
            DrawKind::ColorMesh => {
                let Ok((_, mesh, transform, _, _, _, overlays)) =
                    color_mesh_query.get(item.entity)
                else {
                    continue;
                };
                if mesh.is_empty() {
                    continue;
                }
                apply_material(
                    &mut **renderer,
                    None,
                    &mut last_shader,
                    &mut last_blend_mode,
                );
                let model = affine2_to_mat4(&transform.0);
                renderer.draw_colored_mesh(&mesh.vertices, &mesh.indices, model);
                if let Some(overlays) = overlays {
                    for entry in overlays.0.iter().filter(|e| e.visible) {
                        apply_material(
                            &mut **renderer,
                            Some(&entry.material),
                            &mut last_shader,
                            &mut last_blend_mode,
                        );
                        renderer.draw_colored_mesh(
                            &entry.mesh.vertices,
                            &entry.mesh.indices,
                            model,
                        );
                    }
                }
            }
        }
    }

    let draw_us = t_draw.elapsed().as_micros() as u64;

    if let Some(p) = profiler.as_deref_mut() {
        p.record_phase("render_sort", sort_us);
        p.record_phase("render_draw", draw_us);
    }
}
```

- [ ] **Step 2: Run existing unified_render tests to confirm no regressions**

```
cargo.exe test -p engine_ui unified_render
```

Expected: all tests pass.

- [ ] **Step 3: Commit**

```
git add crates/engine_ui/src/unified_render.rs
git commit -m "feat(engine-ui): record render_sort and render_draw spans in FrameProfiler"
```

---

## Task 8: Register `FrameProfilerPlugin` in `card_game_bin`

**Files:**
- Modify: `crates/card_game_bin/src/main.rs`

- [ ] **Step 1: Add the plugin to `setup()`**

Open `crates/card_game_bin/src/main.rs`. Add `FrameProfilerPlugin` to the `setup` function. It goes after `CardGamePlugin` so the profiler starts recording from the first real frame:

```rust
fn setup(app: &mut App) {
    // PhysicsRes must be inserted before DefaultPlugins (which checks for it)
    app.world_mut()
        .insert_resource(PhysicsRes::new(Box::new(RapierBackend::new(Vec2::ZERO))));

    app.add_plugin(DefaultPlugins);
    app.add_plugin(CardGamePlugin);
    app.add_plugin(FrameProfilerPlugin::default());

    app.set_window_config(WindowConfig {
        title: "Card Game",
        width: 1024,
        height: 768,
        ..Default::default()
    });

    // ... rest unchanged ...
```

The `FrameProfilerPlugin::default()` uses `flush_interval: 120` (flushes every ~2 s) and writes to `perf_log.csv` in the working directory.

You also need to add `FrameProfilerPlugin` to the imports. The `setup` function uses `use axiom2d::prelude::*;` which brings in the engine types. `FrameProfilerPlugin` lives in `engine_app::prelude` which is re-exported via `axiom2d::prelude`. Verify it's re-exported by checking `crates/axiom2d/src/prelude.rs` — if not, add `pub use engine_app::profiler_plugin::FrameProfilerPlugin;` there.

- [ ] **Step 2: Verify no facade changes needed**

`crates/axiom2d/src/prelude.rs` already contains `pub use engine_app::prelude::*;` and `pub use engine_core::prelude::*;`. After Tasks 3 and 4 added `FrameProfilerPlugin` to `engine_app::prelude` and `FrameProfiler` to `engine_core::prelude`, both are automatically re-exported by the `axiom2d` facade. No changes to `crates/axiom2d/src/prelude.rs` are needed.

- [ ] **Step 3: Build the binary**

```
cargo.exe build -p card_game_bin
```

Expected: compiles without errors.

- [ ] **Step 4: Run the game briefly to confirm CSV is written**

Start the game, wait ~3 seconds, close it. Verify `perf_log.csv` exists in the directory where `card_game_bin.exe` was launched from (`/mnt/c/Users/siriu/RustroverProjects/Axiom2d/` if run from WSL, or the `target/debug/` directory if run from RustRover). The file should contain rows like:

```
frame,scope,duration_us
1,Input,12
1,PreUpdate,45
1,Update,8203
...
1,physics_step,7210
...
1,render_sort,841
1,render_draw,6201
```

- [ ] **Step 5: Run the full test suite to confirm no regressions**

```
cargo.exe test
```

Expected: all tests pass.

- [ ] **Step 6: Format**

```
cargo.exe fmt --all
```

- [ ] **Step 7: Commit**

```
git add crates/card_game_bin/src/main.rs
git commit -m "feat(card-game-bin): register FrameProfilerPlugin for CSV perf logging"
```

---

## Spec Coverage Check

| Spec requirement | Task |
|-----------------|------|
| `FrameProfiler` ECS resource in `engine_core` | Task 2 |
| Automatic phase timing in `App::handle_redraw()` | Task 5 |
| `span()` drop guard for manual hot-path timing | Task 2 |
| `FrameProfilerPlugin` registered in `card_game_bin` | Tasks 4 + 8 |
| CSV output: `frame,scope,duration_us` | Task 2 |
| Append mode, header only if file is empty | Task 2 |
| IO errors logged to stderr, never crash | Task 2 |
| Manual span in `physics_step_system` | Task 6 |
| Manual spans in `unified_render_system` (`render_sort`, `render_draw`) | Task 7 |
| `Phase::name()` for phase label strings | Task 1 |
