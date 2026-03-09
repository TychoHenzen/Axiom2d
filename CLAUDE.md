# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Axiom2d is an LLM-optimized 2D game engine written in Rust. The project is in early development (scaffolding stage). The full architectural vision is documented in `Doc/Axiom_Blueprint.md`. The implementation roadmap with progress tracking lives in `Doc/Implementation_Roadmap.md`.

## Implementation Roadmap

**Before starting feature work**, read `Doc/Implementation_Roadmap.md` to understand what's been done, what's next, and the dependency order between phases.

### Working on a step

1. Pick the next `[NOT STARTED]` step that has all dependencies satisfied (check the dependency graph at the bottom of the roadmap).
2. Mark the step `[IN PROGRESS]` in `Doc/Implementation_Roadmap.md` before writing code.
3. Implement with tests (follow the Testing Strategy below). Each checkbox in the step is a deliverable — check it off as you complete it.
4. Run `cargo.exe test` and `cargo.exe build` to verify everything passes.
5. Mark the step `[DONE]` when all checkboxes are checked and tests pass.

### After completing a step

1. Update `Doc/Implementation_Roadmap.md`: mark `[DONE]`, check all boxes.
2. Update the **Implemented Abstractions** section in this file if new public types/traits/systems were added.
3. Update the **Current State** baseline at the top of the roadmap (test counts, feature summary).
4. If new crates gained real implementations, move them from "placeholder" to "implemented" in the Architecture section below.
5. If new workspace dependencies were added, mention them in the Development Environment section.

## Development Environment

This project is developed with **RustRover on Windows** while **Claude Code runs in WSL2 (Ubuntu 24.04)**. The repo lives on the Windows filesystem at `/mnt/c/Users/t.henzen/RustroverProjects/Axiom2d`.

**Rust is installed on the Windows side only** (stable-x86_64-pc-windows-msvc). There is no WSL-native Rust toolchain. To run cargo commands from WSL:

```bash
cargo.exe build              # Build the library
cargo.exe test               # Run all tests
cargo.exe test <test_name>   # Run a single test by name
cargo.exe clippy             # Lint (when configured)
```

Always use `cargo.exe` (not `cargo`) since the Rust toolchain is Windows-only. The same applies to `rustc.exe`, `rustfmt.exe`, etc.

The project uses Rust edition 2024. Dependencies are declared at the workspace level: `glam` (math), `thiserror` (errors), `winit` (windowing), `wgpu` (GPU rendering), `pollster` (async blocking), `bytemuck` (safe type casting for GPU buffers), `bevy_ecs` (standalone ECS).

### WSL/Windows Gotchas

- **Use `cargo.exe` not `cargo`**: The Rust toolchain lives at `/mnt/c/Users/t.henzen/.cargo/bin/` and is a Windows installation. WSL has no native Rust.
- **Filesystem performance**: The project is on a `/mnt/c` drvfs mount. File I/O from WSL is slower than native. Build artifacts in `target/` are written through this mount.
- **Line endings**: Git and editors should be configured for consistent line endings. RustRover on Windows may default to CRLF; ensure `.gitattributes` or git config handles this.
- **Path formats**: WSL sees `/mnt/c/...` paths; Windows/RustRover sees `C:\...` paths. Cargo and rustc invoked via `.exe` will interpret paths as Windows paths. When passing paths to `cargo.exe`, use Windows-style paths or rely on the working directory.
- **File locking**: Both RustRover (Windows) and Claude Code (WSL) access the same files. Avoid concurrent builds — RustRover's build and `cargo.exe` from WSL share the same `target/` directory and lock file.
- **Target triple**: The Windows toolchain compiles for `x86_64-pc-windows-msvc` by default. Any platform-specific code or dependencies should target Windows, not Linux.

## Architecture (Planned)

The engine follows a **Bevy-inspired archetypal ECS** pattern optimized for LLM code generation. Key design principles:

- **Archetypal ECS**: Entities with identical component sets stored together. Systems are plain functions with typed parameters (e.g., `Query<(&mut Position, &Velocity)>`). Uses `bevy_ecs` as a standalone crate, wrapped by `engine_ecs`.
- **Code-defined assets**: All assets (sprites, audio, shaders, tilemaps) are expressed as Rust code or RON data — no binary asset files. Uses `lyon` for vector graphics, `fundsp` for audio synthesis, WGSL for shaders.
- **Trait-abstracted hardware**: Every hardware-dependent subsystem (renderer, audio, input) hides behind a trait with null/mock implementations for testing. Canonical test pattern: `World` + `Schedule` without touching hardware.
- **Flat workspace of crates**: Layout under `crates/` — `engine_core`, `engine_render`, `engine_app`, `engine_ecs`, `engine_input`, and `axiom2d` (facade) are implemented; `engine_audio`, `engine_physics`, `engine_scene`, `engine_assets`, `engine_ui` are placeholders. Virtual manifest at root. `demo` binary crate for smoke testing.

### Implemented Abstractions

- **Renderer trait** (`engine_render::renderer`): `clear(&mut self, color: Color)` + `draw_rect(&mut self, rect: Rect)` + `present(&mut self)` + `resize(&mut self, width, height)`. Object-safe (supports `Box<dyn Renderer>`). `NullRenderer` unit struct provides no-op impl for testing.
- **RendererRes** (`engine_render::renderer`): `#[derive(Resource)]` newtype wrapping `Box<dyn Renderer + Send + Sync>` with `Deref`/`DerefMut`. Lives in the ECS `World` as a resource — systems access it via `ResMut<RendererRes>`. Inserted by App during winit `resumed` event.
- **ClearColor** (`engine_render::clear`): `#[derive(Resource)]` wrapping `Color`. Default is cornflower blue. Paired with `clear_system(Res<ClearColor>, ResMut<RendererRes>)` which calls `renderer.clear()` with the resource's color. Apps insert `ClearColor` and register `clear_system` in `Phase::Render`.
- **SpyRenderer** (`engine_render::testing`): Test helper recording method calls via `Arc<Mutex<Vec<String>>>`. Optional `with_color_capture()` constructor records color argument from `clear()`. Available behind the `testing` feature flag. Shared across crates (engine_app, demo) via dev-dependency with `features = ["testing"]`.
- **Rect** (`engine_render::rect`): `x: Pixels`, `y: Pixels`, `width: Pixels`, `height: Pixels`, `color: Color`. Derives Debug, Clone, Copy, PartialEq. Manual Default (zero-sized, WHITE color).
- **Plugin trait** (`engine_app::app`): `build(&self, app: &mut App)` — called eagerly inside `add_plugin()`. Plugins can register systems via `app.add_systems(Phase, system)` and insert resources via `app.world_mut()`.
- **App ECS integration** (`engine_app::app`): App owns a `World` and a `HashMap<Phase, Schedule>` (one per Phase). `handle_redraw()` iterates `PHASE_ORDER` const array, runs each schedule, then calls `present()` on `RendererRes` from the World. Public API: `world()`, `world_mut()`, `add_systems(Phase, systems)`, `schedule_count()`. `App::new()` pre-inserts `WindowSize::default()` and `DeltaTime::default()`. `set_window_config()` and `handle_resize()` keep `WindowSize` synced.
- **WindowSize** (`engine_app::window_size`): `#[derive(Resource)]` with `width: Pixels` and `height: Pixels`. Default is 0x0. Automatically managed by App — updated on `set_window_config()` and window resize events. Systems read it via `Res<WindowSize>`.
- **Position** (`engine_core::spatial`): `#[derive(Component)]` with `x: Pixels` and `y: Pixels`. Screen-space position for entities.
- **Velocity** (`engine_core::spatial`): `#[derive(Component)]` with `dx: Pixels` and `dy: Pixels`. Per-second velocity for entities (scaled by DeltaTime in systems).
- **DeltaTime** (`engine_core::time`): `#[derive(Resource)]` newtype wrapping `Seconds`. Default is 0.0. Updated each frame by `time_system`. Systems read via `Res<DeltaTime>`. Pre-inserted by `App::new()`.
- **FixedTimestep** (`engine_core::time`): `#[derive(Resource)]` with `accumulator: Seconds` and `step_size: Seconds`. Default step_size is 1/60. `tick(delta) -> u32` returns number of fixed steps to run, retaining remainder in accumulator ("Fix Your Timestep" pattern).
- **Time trait** (`engine_core::time`): `fn delta(&mut self) -> Seconds`. Object-safe (`Send + Sync`). `SystemClock` tracks `last_instant` and returns elapsed since last call. `FakeClock` allows manual `advance(Seconds)` — `delta()` returns accumulated pending time and resets to zero.
- **ClockRes** (`engine_core::time`): `#[derive(Resource)]` newtype wrapping `Box<dyn Time>` with `Deref`/`DerefMut`. Same pattern as `RendererRes`.
- **time_system** (`engine_core::time`): `fn time_system(ResMut<ClockRes>, ResMut<DeltaTime>)` — reads delta from clock, writes to DeltaTime. Registered in `Phase::PreUpdate`.
- **ECS wrapper** (`engine_ecs`): Thin wrapper around `bevy_ecs`. Re-exports `Component`, `Resource`, `World`, `Entity`, `Query`, `Res`, `ResMut`, `Schedule`, `Commands`, `Added`, `Changed`, `With`, `Without`, `SystemSet`, `IntoScheduleConfigs`, `ScheduleSystem`. Defines `Phase` enum as schedule labels.
- **InputState** (`engine_input::input_state`): `#[derive(Resource)]` with `HashSet<KeyCode>` for pressed, just_pressed, just_released. Methods: `pressed(key)`, `just_pressed(key)`, `just_released(key)`, `press(key)`, `release(key)`, `clear_frame_state()`, `action_pressed(&ActionMap, &str)`, `action_just_pressed(&ActionMap, &str)`. `press()`/`release()` allow programmatic population for testing without hardware. Action methods check if ANY bound key satisfies the condition (OR semantics).
- **ActionName** (`engine_input::action_map`): `#[derive(Debug, Clone, PartialEq, Eq, Hash)]` newtype wrapping `String`. Used as HashMap key in ActionMap.
- **ActionMap** (`engine_input::action_map`): `#[derive(Resource, Debug, Clone, Default)]` mapping `ActionName → Vec<KeyCode>`. Methods: `bind(&str, Vec<KeyCode>)` to register bindings, `bindings_for(&str) -> &[KeyCode]` to query. Unbound actions return empty slice. Systems take both `Res<InputState>` and `Res<ActionMap>` as parameters.
- **InputEventBuffer** (`engine_input::input_event_buffer`): `#[derive(Resource)]` wrapping `Vec<(KeyCode, ElementState)>`. Staging area between winit callbacks and ECS systems. `push()` adds events, `drain()` returns and clears all. App's `handle_key_event()` pushes to this buffer if present in the World.
- **input_system** (`engine_input::input_system`): `fn input_system(ResMut<InputEventBuffer>, ResMut<InputState>)` — clears per-frame state, drains buffer, updates InputState. Registered in `Phase::Input`.
- **KeyCode** (`engine_input::prelude`): Re-export of `winit::keyboard::KeyCode`. No translation layer.

### Scheduling Phases

`Input → PreUpdate → Update → PostUpdate → Render` — implemented as `engine_ecs::schedule::Phase` enum with `ScheduleLabel` derive.

### Render Pipeline

`Clear → Sprite Pass → Particle Pass → Post-Process → UI Pass → Present`

## API Design Conventions

- Components use `#[derive(Component)]` — no manual registration
- Configs use plain structs with `Default` — LLMs handle `..Default::default()` reliably
- NewType wrappers for type safety (e.g., `Pixels(f32)`, `Seconds(f32)`, `TextureId(u32)`)
- Enums over magic numbers for constrained states (e.g., `BlendMode`, `RenderLayer`)
- Limit trait indirection to 2 levels max
- Each crate exports a `prelude` module

## Testing Strategy

- Test naming: `when_action_then_outcome` hybrid style (skip `given_` when precondition is obvious)
- Test body structure: `// Arrange` / `// Act` / `// Assert` section markers
- Inline `#[cfg(test)] mod tests` in each source file — no separate test files
- Deterministic game loop: fixed timestep, injectable mock time, seeded RNG (`ChaCha8Rng`)
- Use `BTreeMap` or fixed-seed `ahash` instead of `HashMap` where iteration order matters
- Property-based testing with `proptest` for physics invariants, serialization roundtrips, ECS invariants
- Visual regression via wgpu headless mode with SSIM comparison (0.99 threshold)
- Snapshot testing with `insta`
- Target: `cargo-nextest` as test runner for parallel execution

### What NOT to test

Only test **behavior you wrote**, not language or framework guarantees. The following test categories are banned:

- **Prelude/re-export tests**: Don't test that `use crate::prelude::*` makes types available. Real usage catches missing re-exports at compile time.
- **Derive tests**: Don't test that `Clone`, `Copy`, `PartialEq`, `Debug`, `Hash` derives work. Rust's derive macros are not broken.
- **Struct construction tests**: Don't test that `Foo { x: 1 }.x == 1`. Rust structs store their fields.
- **Resource insertion tests**: Don't test that `world.insert_resource(X); world.get_resource::<X>().is_some()`. The `#[derive(Resource)]` macro from bevy_ecs works.
- **Component spawn tests**: Don't test that `world.spawn(C)` makes `C` queryable. Same reasoning as Resource insertion.
- **Trivial default tests**: Don't test that `Default` returns the value written in `impl Default`. If the default matters for correctness, test it through the system that depends on it.
- **Boxing/trait-object tests**: Don't test that `Box::new(X) as Box<dyn Trait>` compiles. Compilation is the test.

**Do test**: Custom logic (arithmetic operators, conversion functions like `from_u8`), system behavior (clear_system, time_system, input_system), non-trivial algorithms (FixedTimestep.tick accumulator math), and design constraints (no-clamping on Rect values).
