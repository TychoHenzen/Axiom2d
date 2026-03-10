# Axiom2d Implementation Roadmap

This document tracks the gap between the architectural blueprint (`Doc/Axiom_Blueprint.md`) and the current implementation. Each step is sized for a single session and ordered by dependency.

## Current State (Baseline)

**Implemented crates:** engine_core (27 tests), engine_ecs (7 tests), engine_render (95 tests), engine_app (30 tests), engine_input (28 tests), engine_scene (39 tests), axiom2d facade (0 tests), demo (9 tests). Total: 235 tests.

**What works:** Archetypal ECS via bevy_ecs, 5-phase scheduling, Renderer trait + WgpuRenderer GPU backend with instanced quad rendering, App with winit integration, Plugin system, ClearColor/clear_system, SpyRenderer for testing, keyboard-controlled rectangle demo, DeltaTime/FixedTimestep/Time trait with FakeClock/SystemClock, time_system in PreUpdate, InputState/InputEventBuffer/input_system for keyboard input, App bridges winit keyboard events to ECS, ActionName/ActionMap for action-level input queries (action_pressed, action_just_pressed), Parent-Child hierarchy via ChildOf/Children with hierarchy_maintenance_system, SpawnChildExt for World, Transform propagation (GlobalTransform2D) through parent-child hierarchy, Visibility system (Visible/EffectiveVisibility) with hierarchy inheritance, RenderLayer enum + SortOrder for deterministic render ordering, TextureAtlas with guillotiere rect packing + AtlasBuilder + load_image_bytes (PNG/JPEG), Sprite component + sprite_render_system with visibility filtering and RenderLayer/SortOrder sorting, Camera2D component with world-to-screen/screen-to-world conversion + frustum culling + GPU view-projection uniform buffer.

**Placeholder crates (empty):** engine_audio, engine_physics, engine_assets, engine_ui.

---

## Phase 1: Time & Input

### Step 1.1 — DeltaTime & Fixed Timestep `[DONE]`
**Crate:** engine_core
**Why first:** Every future system (physics, animation, audio) depends on frame-independent time.

- [x] `DeltaTime(Seconds)` resource — systems read via `Res<DeltaTime>`
- [x] `FixedTimestep` resource with accumulator for "Fix Your Timestep" pattern
- [x] `Time` trait + `SystemClock` (real) / `FakeClock` (test) implementations
- [x] `time_system` in Phase::PreUpdate that updates DeltaTime each frame
- [x] Update demo to multiply velocity by DeltaTime instead of per-frame constants
- [x] Tests: deterministic time with FakeClock, accumulator behavior

### Step 1.2 — Input State (Keyboard + Mouse) `[DONE]`
**Crate:** engine_input
**Deps:** winit (already in workspace)

- [x] `KeyCode` enum (re-exported from winit::keyboard::KeyCode)
- [x] `InputState` resource: `pressed(KeyCode) -> bool`, `just_pressed()`, `just_released()`
- [ ] `MouseState` resource: position (Pixels, Pixels), button states, scroll delta *(deferred — keyboard-only for now)*
- [x] `input_system` in Phase::Input — reads buffered winit events via `InputEventBuffer`, updates InputState
- [x] Programmatic population for testing (`press()`/`release()` methods, no hardware needed)
- [x] Add engine_input to axiom2d facade re-exports
- [x] Tests: press/release tracking, just_pressed lasts one frame *(mouse tests deferred with MouseState)*
- [x] App bridges winit `KeyboardInput` events to `InputEventBuffer` via `handle_key_event()`
- [x] Demo updated: keyboard-controlled rectangle (arrow keys) replaces auto-bounce

### Step 1.3 — Action Mapping `[DONE]`
**Crate:** engine_input

- [x] `ActionName(String)` newtype
- [x] `ActionMap` resource: maps ActionName → Vec<KeyCode>
- [x] `action_pressed()`, `action_just_pressed()` methods on InputState using ActionMap
- [x] Tests: multiple keys bound to same action, unbound actions return false

### Step 1.4 — Gamepad Support (Deferred) `[NOT STARTED]`
**Crate:** engine_input
**New dep:** gilrs

- [ ] `GamepadState` resource via gilrs
- [ ] Integrate gamepad axes/buttons into ActionMap
- [ ] *Can skip until needed — keyboard+mouse covers most 2D games*

---

## Phase 2: Scene Graph & Transform Hierarchy

### Step 2.1 — Parent-Child Hierarchy `[DONE]`
**Crate:** engine_scene

- [x] `ChildOf(Entity)` component — opt-in hierarchy on flat ECS
- [x] `Children(Vec<Entity>)` component — engine-managed, derived from ChildOf
- [x] `hierarchy_maintenance_system` in PostUpdate — syncs Children from ChildOf
- [x] Spawn helpers via World extension (`SpawnChildExt` trait)
- [x] Tests: add/remove ChildOf updates Children, orphan cleanup

### Step 2.2 — Transform Propagation `[DONE]`
**Crate:** engine_scene
**Deps:** engine_core (Transform2D, Affine2)

- [x] `GlobalTransform2D(Affine2)` component — engine-computed, read-only for users
- [x] `transform_propagation_system` in PostUpdate (after hierarchy_maintenance):
  - Root entities: copy Transform2D → GlobalTransform2D
  - Children: parent.GlobalTransform2D * child.Transform2D
  - *(Change detection deferred — always-update approach for now)*
- [x] Tests: single entity propagation, 2-level hierarchy, 3-level hierarchy, siblings, scale interaction, re-run update
- [x] `Transform2D` now derives `Component` (was plain struct)

### Step 2.3 — Visibility & Render Ordering `[DONE]`
**Crate:** engine_scene

- [x] `Visible(bool)` component — default true, inherited through hierarchy
- [x] `RenderLayer` enum: Background, World, Characters, Foreground, UI
- [x] `SortOrder(i32)` component — ordering within a layer
- [x] `visibility_system` in PostUpdate — computes effective visibility from hierarchy
- [x] Tests: hidden parent hides children, layer+sort ordering is deterministic

---

## Phase 3: Rendering Pipeline Upgrades

### Step 3.1 — Instanced Quad Rendering `[DONE]`
**Crate:** engine_render

- [x] Single quad vertex buffer (shared across all sprites)
- [x] Instance buffer: per-sprite transform, UV rect, color
- [x] Batch draw calls — single draw call per texture/material
- [x] Benchmark: measure draw call reduction vs current per-rect vertex rebuild

### Step 3.2 — Texture Support `[DONE]`
**Crate:** engine_render
**New deps:** image, guillotiere

- [x] `TextureHandle` wrapping TextureId + UV rect
- [x] `TextureAtlas` resource — rect packing via guillotiere
- [x] `AtlasBuilder::add_image()` → TextureHandle with UV rect
- [x] Texture bind group in render pipeline

### Step 3.3 — Sprite Component & Rendering `[DONE]`
**Crate:** engine_render
**Deps:** engine_scene (GlobalTransform2D, RenderLayer, SortOrder)

- [x] `Sprite { texture: TextureId, uv_rect: [f32; 4], color: Color, width: Pixels, height: Pixels }` component
- [x] `sprite_render_system` in Phase::Render — draws sprites sorted by RenderLayer + SortOrder
- [x] Uses GlobalTransform2D for positioning
- [x] Batches by texture atlas page (single-atlas model — all sprites share one bind group)

### Step 3.4 — Camera `[DONE]`
**Crate:** engine_render

- [x] `Camera2D` component: position (Vec2), zoom (f32), viewport
- [x] View/projection matrix as uniform buffer, injected into shaders
- [x] Frustum culling: AABB camera rect vs entity bounding box
- [x] Tests: world-to-screen / screen-to-world conversion, culling logic

### Step 3.5 — Vector Graphics (Lyon) `[NOT STARTED]`
**Crate:** engine_render
**New dep:** lyon

- [ ] Lyon path tessellation → GPU vertex/index buffers
- [ ] `Shape` component: Circle, Polygon, Path variants
- [ ] `shape_render_system` in Phase::Render
- [ ] Tests: tessellation produces valid vertex data

### Step 3.6 — Post-Processing Framework `[NOT STARTED]`
**Crate:** engine_render
**New dep:** naga-oil

- [ ] `RenderPass` trait: `render(&self, encoder, view)`
- [ ] Linear pass chain: Clear → Sprite → Particle → PostProcess → UI → Present
- [ ] Bloom pass: brightness extraction → separable Gaussian blur → composite
- [ ] Fullscreen quad utility for post-process passes

### Step 3.7 — Material System `[NOT STARTED]`
**Crate:** engine_render

- [ ] `Material2d { shader: ShaderHandle, textures: Vec<TextureBinding>, uniforms: Vec<u8>, blend_mode: BlendMode }`
- [ ] Shader variant caching by preprocessor defines (naga-oil `#ifdef`)
- [ ] `BlendMode` enum: Alpha, Additive, Multiply

---

## Phase 4: Audio

### Step 4.1 — Audio Backend Trait `[NOT STARTED]`
**Crate:** engine_audio
**New dep:** cpal

- [ ] `trait AudioBackend { fn play(&mut self, sound: &SoundData); fn stop(&mut self, id: PlaybackId); fn set_volume(&mut self, volume: f32); }`
- [ ] `NullAudioBackend` — no-op for testing
- [ ] `CpalBackend` — real audio output via cpal
- [ ] `AudioRes` resource (same pattern as RendererRes)

### Step 4.2 — Sound Synthesis (fundsp) `[NOT STARTED]`
**Crate:** engine_audio
**New dep:** fundsp

- [ ] `SoundEffect` — code-defined sounds using fundsp DSP graph notation
- [ ] `SoundLibrary` resource: named sound effects
- [ ] `play_sound_system` reacting to `PlaySound` event/command

### Step 4.3 — Game Audio Features (Deferred) `[NOT STARTED]`
**Crate:** engine_audio

- [ ] Mixer tracks, volume groups
- [ ] Spatial 2D audio (panning based on entity position relative to listener)
- [ ] Consider kira crate for higher-level features

---

## Phase 5: Physics

### Step 5.1 — Physics Components & Trait `[NOT STARTED]`
**Crate:** engine_physics
**New dep:** rapier2d

- [ ] `RigidBody` component: Dynamic, Static, Kinematic variants
- [ ] `Collider` component: Circle(f32), Aabb(Vec2), ConvexPolygon(Vec<Vec2>)
- [ ] `trait PhysicsBackend { fn step(&mut self, dt: Seconds); fn add_body(...); fn add_collider(...); }`
- [ ] `NullPhysicsBackend` for testing

### Step 5.2 — Collision Events `[NOT STARTED]`
**Crate:** engine_physics

- [ ] `CollisionEvent { entity_a: Entity, entity_b: Entity, kind: CollisionKind }`
- [ ] `CollisionKind` enum: Started, Stopped
- [ ] `Events<CollisionEvent>` resource (bevy_ecs event pattern)
- [ ] `physics_step_system` in Phase::PreUpdate

### Step 5.3 — Physics-Transform Sync `[NOT STARTED]`
**Crate:** engine_physics

- [ ] Bidirectional sync: Position/Transform2D ↔ rapier rigid bodies
- [ ] Runs after physics step, before transform propagation
- [ ] Optional interpolation for smooth rendering between physics steps

---

## Phase 6: Assets & Serialization

### Step 6.1 — Serde + RON Integration `[NOT STARTED]`
**Crate:** engine_assets + engine_core
**New deps:** serde (with derive), ron

- [ ] Add `Serialize`/`Deserialize` derives to core types: Color, Transform2D, Rect, Pixels, Seconds
- [ ] Scene serialization to/from RON format
- [ ] Tests: roundtrip serialize → deserialize equals original

### Step 6.2 — Asset Loading & Caching `[NOT STARTED]`
**Crate:** engine_assets
**New dep:** assets_manager

- [ ] `AssetServer` resource: load, cache, reference-count assets
- [ ] `Handle<T>` for type-safe asset references
- [ ] Directory-based asset organization

### Step 6.3 — Hot Reloading (Deferred) `[NOT STARTED]`
**Crate:** engine_assets

- [ ] File watching for RON parameter changes
- [ ] Reload textures, sounds, configs without restart
- [ ] `dev` feature flag to enable/disable

---

## Phase 7: UI

### Step 7.1 — Basic UI Layout `[NOT STARTED]`
**Crate:** engine_ui

- [ ] `UiNode` component: rect, anchor point, margin
- [ ] `FlexLayout` — simple row/column child arrangement
- [ ] `Text` component with bitmap font rendering
- [ ] UI renders in the UI RenderLayer during Phase::Render

### Step 7.2 — Widgets (Deferred) `[NOT STARTED]`
**Crate:** engine_ui

- [ ] Button, Label, Panel, ProgressBar components
- [ ] Interaction events: hover, click, focus
- [ ] Style resources for theming

---

## Phase 8: Cross-Cutting Quality

### Step 8.1 — Property-Based Testing `[NOT STARTED]`
**New dep:** proptest

- [ ] Physics invariants (no tunneling regardless of velocity)
- [ ] Serialization roundtrips (serialize → deserialize == original)
- [ ] ECS invariants (no entity has contradictory components)

### Step 8.2 — Snapshot Testing `[NOT STARTED]`
**New dep:** insta

- [ ] Snapshot tests for complex struct Debug output
- [ ] Scene serialization snapshots

### Step 8.3 — Visual Regression Testing `[NOT STARTED]`
**New deps:** image-compare

- [ ] Headless wgpu rendering to texture → readback pixels
- [ ] SSIM comparison against golden images (0.99 threshold)
- [ ] CI integration with llvmpipe software renderer

### Step 8.4 — DefaultPlugins & Feature Flags `[NOT STARTED]`

- [ ] `DefaultPlugins` group: registers render, input, audio, physics plugins
- [ ] Feature flags: `render`, `audio`, `physics`, `dev`, `hot_reload`
- [ ] `default-features = false` gives headless ECS-only core

### Step 8.5 — Documentation & Examples `[NOT STARTED]`

- [ ] `docs/llms.txt` — LLM-friendly project context file
- [ ] `examples/hello_world/` — minimal window + colored background
- [ ] `examples/platformer/` — input + physics + sprites
- [ ] `examples/particles/` — particle system showcase

---

## Progress Legend

- `[NOT STARTED]` — No work done
- `[IN PROGRESS]` — Actively being implemented
- `[DONE]` — Implemented, tested, merged

## Dependency Graph (Simplified)

```
Phase 1 (Time+Input)
  └─→ Phase 2 (Scene Graph)
       └─→ Phase 3 (Render Upgrades) ←── Phase 3.4 (Camera) is independent
            └─→ Phase 7 (UI)
Phase 4 (Audio) ←── independent, can start after Phase 1
Phase 5 (Physics) ←── needs Phase 1 (DeltaTime) and Phase 2 (Transforms)
Phase 6 (Assets) ←── independent, but more useful after Phase 3 (textures exist)
Phase 8 (Quality) ←── ongoing, weave in as features land
```
