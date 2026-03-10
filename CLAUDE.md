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

The project uses Rust edition 2024. Dependencies are declared at the workspace level: `glam` (math), `thiserror` (errors), `winit` (windowing), `wgpu` (GPU rendering), `pollster` (async blocking), `bytemuck` (safe type casting for GPU buffers), `bevy_ecs` (standalone ECS), `guillotiere` (2D texture atlas rect packing), `image` (PNG/JPEG decoding for embedded assets).

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
- **Flat workspace of crates**: Layout under `crates/` — `engine_core`, `engine_render`, `engine_app`, `engine_ecs`, `engine_input`, `engine_scene`, and `axiom2d` (facade) are implemented; `engine_audio`, `engine_physics`, `engine_assets`, `engine_ui` are placeholders. Virtual manifest at root. `demo` binary crate for smoke testing.

### Implemented Abstractions

- **Renderer trait** (`engine_render::renderer`): `clear(&mut self, color: Color)` + `draw_rect(&mut self, rect: Rect)` + `present(&mut self)` + `resize(&mut self, width, height)`. Object-safe (supports `Box<dyn Renderer>`). `NullRenderer` unit struct provides no-op impl for testing.
- **RendererRes** (`engine_render::renderer`): `#[derive(Resource)]` newtype wrapping `Box<dyn Renderer + Send + Sync>` with `Deref`/`DerefMut`. Lives in the ECS `World` as a resource — systems access it via `ResMut<RendererRes>`. Inserted by App during winit `resumed` event.
- **ClearColor** (`engine_render::clear`): `#[derive(Resource)]` wrapping `Color`. Default is cornflower blue. Paired with `clear_system(Res<ClearColor>, ResMut<RendererRes>)` which calls `renderer.clear()` with the resource's color. Apps insert `ClearColor` and register `clear_system` in `Phase::Render`.
- **SpyRenderer** (`engine_render::testing`): Test helper recording method calls via `Arc<Mutex<Vec<String>>>`. Optional `with_color_capture()` constructor records color argument from `clear()`. Available behind the `testing` feature flag. Shared across crates (engine_app, demo) via dev-dependency with `features = ["testing"]`.
- **Rect** (`engine_render::rect`): `x: Pixels`, `y: Pixels`, `width: Pixels`, `height: Pixels`, `color: Color`. Derives Debug, Clone, Copy, PartialEq. Manual Default (zero-sized, WHITE color).
- **Instanced Quad Rendering** (`engine_render::wgpu_renderer`, private module): WgpuRenderer uses GPU instancing — persistent `QuadVertex` vertex buffer (4 corners of unit square) + `u16` index buffer (6 indices, 2 CCW triangles) + per-frame `Instance` buffer. `Instance { ndc_rect: [f32; 4], uv_rect: [f32; 4], color: [f32; 4] }` — `#[repr(C)]`, bytemuck Pod. `ndc_rect` encodes world coordinates `[x, y, width, height]` (not NDC despite the field name). `draw_rect()` and `draw_sprite()` push Instances via `rect_to_instance(rect)` (pub(crate) free fn, no viewport params). `present()` issues single `draw_indexed` call with `instance_count`. WGSL shader scales unit quad by per-instance `ndc_rect` (origin+size encoding), then multiplies by `camera.view_proj` to convert world→clip space. `QUAD_VERTICES` and `QUAD_INDICES` are pub(crate) constants. Default 1x1 white texture bind group; `upload_atlas(&mut self, &TextureAtlas)` replaces the bind group with atlas texture data.
- **TextureHandle** (`engine_render::atlas`): `#[derive(Debug, Clone, Copy, PartialEq)]` with `pub texture_id: TextureId` and `pub uv_rect: [f32; 4]`. Lightweight value type returned by `AtlasBuilder::add_image()`. UV rect is in normalized [0,1] coordinates.
- **AtlasError** (`engine_render::atlas`): Error enum with variants `NoSpace`, `DataLengthMismatch { expected, actual }`, `InvalidDimensions`, `DecodeError(String)`. Uses `thiserror::Error` derive.
- **AtlasBuilder** (`engine_render::atlas`): CPU-side builder using `guillotiere::AtlasAllocator` for rect packing. `new(width, height)` creates the builder. `add_image(width, height, &[u8]) -> Result<TextureHandle, AtlasError>` allocates a region and stores RGBA pixel data. `build(self) -> TextureAtlas` produces the final atlas with packed pixel buffer. Auto-increments TextureId for each added image.
- **TextureAtlas** (`engine_render::atlas`): `#[derive(Resource)]` with `data: Vec<u8>` (packed RGBA pixel buffer), `width: u32`, `height: u32`, `lookups: HashMap<TextureId, [f32; 4]>`. `lookup(TextureId) -> Option<[f32; 4]>` retrieves UV rect for a texture. Produced by `AtlasBuilder::build()`.
- **ImageData** (`engine_render::atlas`): Plain struct with `width: u32`, `height: u32`, `data: Vec<u8>`. Returned by `load_image_bytes()`.
- **load_image_bytes** (`engine_render::atlas`): `fn(&[u8]) -> Result<ImageData, AtlasError>` — wraps `image::load_from_memory`, converts to RGBA8. Supports PNG and JPEG formats.
- **normalize_uv_rect** (`engine_render::atlas`): `pub(crate) fn(x, y, w, h, atlas_w, atlas_h) -> [f32; 4]` — converts pixel-space allocation to [u0, v0, u1, v1] in [0,1] range.
- **Sprite** (`engine_render::sprite`): `#[derive(Component)]` with `texture: TextureId`, `uv_rect: [f32; 4]`, `color: Color`, `width: Pixels`, `height: Pixels`. Derives Debug, Clone, Copy, PartialEq.
- **sprite_render_system** (`engine_render::sprite`): `fn(Query<(&Sprite, &GlobalTransform2D, Option<&RenderLayer>, Option<&SortOrder>, Option<&EffectiveVisibility>)>, ResMut<RendererRes>)` — filters out entities with `EffectiveVisibility(false)`, sorts by `(RenderLayer, SortOrder)` with defaults `(World, 0)`, constructs `Rect` from `GlobalTransform2D.translation` + sprite dimensions, calls `renderer.draw_sprite(rect, uv_rect)`. Registered in `Phase::Render`.
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
- **ChildOf** (`engine_scene::hierarchy`): `#[derive(Component)]` newtype wrapping `Entity`. User-facing: attach to a child entity to declare its parent. Opt-in hierarchy on flat ECS.
- **Children** (`engine_scene::hierarchy`): `#[derive(Component)]` wrapping `Vec<Entity>`. Engine-managed — never set by users directly. Derived from ChildOf by `hierarchy_maintenance_system`. Vec is sorted by Entity for deterministic traversal.
- **hierarchy_maintenance_system** (`engine_scene::hierarchy`): `fn(Query<(Entity, &ChildOf)>, Query<Entity, With<Children>>, Commands)` — rebuilds Children from ChildOf each frame. Removes stale Children when all ChildOf references to a parent are gone. Registered in `Phase::PostUpdate`.
- **SpawnChildExt** (`engine_scene::spawn_child`): Extension trait on `World`. `spawn_child(parent: Entity, bundle: impl Bundle) -> Entity` — spawns a new entity with ChildOf(parent) plus the provided bundle.
- **GlobalTransform2D** (`engine_scene::transform_propagation`): `#[derive(Component)]` newtype wrapping `Affine2`. Engine-computed world-space transform. Written by `transform_propagation_system`, never by users directly.
- **transform_propagation_system** (`engine_scene::transform_propagation`): Walks hierarchy from roots (no ChildOf) down through Children. Root: copies `Transform2D.to_affine2()` → GlobalTransform2D. Children: `parent.GlobalTransform2D * child.Transform2D.to_affine2()`. Registered in `Phase::PostUpdate` (after hierarchy_maintenance_system).
- **Visible** (`engine_scene::visibility`): `#[derive(Component)]` newtype wrapping `bool`. Default is `true`. User-facing: attach to entities to control visibility. Entities without Visible are treated as visible.
- **EffectiveVisibility** (`engine_scene::visibility`): `#[derive(Component)]` newtype wrapping `bool`. Engine-computed inherited visibility. Written by `visibility_system`, never by users directly. `parent_effective && child_visible` (AND logic).
- **visibility_system** (`engine_scene::visibility`): Walks hierarchy from roots (no ChildOf) down through Children. Root: `EffectiveVisibility = Visible` (or true if absent). Children: `parent_effective AND child_visible`. Registered in `Phase::PostUpdate` (after hierarchy_maintenance_system).
- **RenderLayer** (`engine_scene::render_order`): Enum with variants `Background`, `World`, `Characters`, `Foreground`, `UI` — declaration order IS render order via `#[derive(Ord)]`. Derives Component, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord.
- **SortOrder** (`engine_scene::render_order`): `#[derive(Component, Default)]` newtype wrapping `i32`. Ordering within a RenderLayer. Default is 0. No clamping — negative values allowed. Used with RenderLayer as `(RenderLayer, SortOrder)` tuple sort for deterministic draw order.
- **Camera2D** (`engine_render::camera`): `#[derive(Component)]` with `position: Vec2` and `zoom: f32`. Default: position=ZERO, zoom=1.0. Camera position is the center of the view. Zoom > 1 magnifies, zoom < 1 zooms out.
- **CameraUniform** (`engine_render::camera`): `#[derive(Resource)]` with `view_proj: [[f32; 4]; 4]`. GPU-ready orthographic view-projection matrix. `CameraUniform::from_camera(camera, vw, vh)` combines view transform with Y-flipped orthographic projection for wgpu NDC.
- **camera_prepare_system** (`engine_render::camera`): `fn(Query<&Camera2D>, ResMut<RendererRes>)` — queries the first Camera2D entity (or defaults to a viewport-centered camera with zoom 1.0 if none exists), gets viewport size from renderer, computes CameraUniform, calls `renderer.set_view_projection()`. Always sets a projection — required because `rect_to_instance` passes world coordinates that need a camera matrix for correct rendering. Registered in `Phase::Render` (before sprite_render_system).
- **world_to_screen** (`engine_render::camera`): `fn(Vec2, &Camera2D, viewport_width, viewport_height) -> Vec2` — converts world-space point to screen-space pixels. Camera position maps to viewport center.
- **screen_to_world** (`engine_render::camera`): `fn(Vec2, &Camera2D, viewport_width, viewport_height) -> Vec2` — inverse of world_to_screen. Viewport center maps to camera position.
- **camera_view_rect** (`engine_render::camera`): `pub(crate) fn(&Camera2D, viewport_width, viewport_height) -> (Vec2, Vec2)` — returns (min, max) world-space AABB the camera can see. Half extents = viewport / (2 * zoom).
- **aabb_intersects_view_rect** (`engine_render::camera`): `pub(crate) fn(entity_min, entity_max, view_min, view_max) -> bool` — AABB overlap test using >= comparisons (edge-touching counts as intersecting).
- **Frustum culling in sprite_render_system**: When a Camera2D entity exists, sprites whose world AABB doesn't intersect the camera view rect are skipped. No camera → no culling (backward compatible).
- **Renderer trait** now includes `set_view_projection(&mut self, matrix: [[f32; 4]; 4])` and `viewport_size(&self) -> (u32, u32)`. WgpuRenderer uploads matrix to GPU uniform buffer; NullRenderer/SpyRenderer are no-ops.
- **GPU camera uniform**: WgpuRenderer has a persistent camera uniform buffer (bind group 1, binding 0). WGSL shader multiplies vertex position by `camera.view_proj`. `camera_prepare_system` always sets the projection each frame (viewport-centered ortho when no Camera2D exists).

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
