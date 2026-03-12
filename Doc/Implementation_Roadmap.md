# Axiom2d Implementation Roadmap

This document tracks the gap between the architectural blueprint (`Doc/Axiom_Blueprint.md`) and the current implementation. Each step is sized for a single session and ordered by dependency.

## Current State (Baseline)

**Implemented crates:** engine_core (37 tests), engine_ecs (1 test), engine_render (217 tests), engine_app (39 tests), engine_input (53 tests), engine_scene (42 tests), engine_audio (63 tests), engine_physics (42 tests), engine_assets (26 tests + 1 doctest), engine_ui (69 tests), axiom2d facade (15 tests), demo (41 tests). Total: 648 tests + 1 doctest.

**What works:** Archetypal ECS via bevy_ecs, 5-phase scheduling, Renderer trait + WgpuRenderer GPU backend with instanced quad rendering, App with winit integration, Plugin system, ClearColor/clear_system, SpyRenderer for testing, keyboard-controlled rectangle demo, DeltaTime/FixedTimestep/Time trait with FakeClock/SystemClock, time_system in PreUpdate, InputState/InputEventBuffer/input_system for keyboard input, App bridges winit keyboard events to ECS, ActionName/ActionMap for action-level input queries (action_pressed, action_just_pressed), Parent-Child hierarchy via ChildOf/Children with hierarchy_maintenance_system, SpawnChildExt for World, Transform propagation (GlobalTransform2D) through parent-child hierarchy, Visibility system (Visible/EffectiveVisibility) with hierarchy inheritance, RenderLayer enum + SortOrder for deterministic render ordering, TextureAtlas with guillotiere rect packing + AtlasBuilder + load_image_bytes (PNG/JPEG), Sprite component + sprite_render_system with visibility filtering and RenderLayer/SortOrder/BlendMode sorting, Camera2D component with world-to-screen/screen-to-world conversion + frustum culling + GPU view-projection uniform buffer, Shape component (Circle/Polygon variants) with Lyon tessellation + shape_render_system with visibility/sorting/culling/BlendMode, Post-processing framework with bloom (brightness extraction + separable Gaussian blur + composite), fullscreen quad utility, BloomSettings resource + post_process_system, Material2d component with BlendMode + ShaderHandle + TextureBinding + uniforms, ShaderRegistry for shader source management, #ifdef shader preprocessing, effective_blend_mode helper, set_blend_mode on Renderer trait with batching deduplication, DefaultPlugins auto-registration of all standard systems including post-processing, `render` feature flag for headless ECS-only mode, AudioBackend trait + NullAudioBackend + CpalBackend (with real sample mixing) + AudioRes resource wrapper, PlaybackId/SoundData value types, `audio` feature flag (default on) with DefaultPlugins integration, SoundEffect (fundsp factory closure per playback), SoundLibrary resource (named sound effects), PlaySound/PlaySoundBuffer command buffer, play_sound_system in Phase::PreUpdate, MixerTrack enum (Master/Music/Sfx/Ambient/Ui) + MixerState resource with per-track volumes (multiplicative stacking with global volume in mix_into), AudioBackend extended with play_on_track/set_track_volume, Spatial 2D audio: AudioListener marker + AudioEmitter component (volume/max_distance) + spatial_audio_system in Phase::PostUpdate computing stereo gains from GlobalTransform2D positions via distance_attenuation (linear falloff) and compute_pan (constant-power panning), play_sound_system applies SpatialGains with mono→stereo conversion and silent-sound culling, PhysicsBackend trait + NullPhysicsBackend + RapierBackend (rapier2d real physics) + PhysicsRes resource wrapper, RigidBody component (Dynamic/Static/Kinematic) + Collider component (Circle/Aabb/ConvexPolygon), entity-handle mapping for ECS↔rapier correlation, CollisionEvent/CollisionKind/CollisionEventBuffer for collision detection events, physics_step_system in Phase::PreUpdate, RapierBackend collision event wiring via ChannelEventCollector with collider-to-entity handle resolution, physics_sync_system for physics→ECS transform writeback (position + rotation independently gated, scale preserved), MouseState resource (button state + screen_pos + world_pos + scroll_delta) + MouseEventBuffer + mouse_input_system in Phase::Input + mouse_world_pos_system in Phase::PostUpdate (screen→world via Camera2D), App bridges winit CursorMoved/MouseInput/MouseWheel events to ECS, ActionMap extended with mouse button bindings.

Serde + RON serialization on all pure-data types across engine crates (Color, Transform2D, Rect, Pixels, Seconds, TextureId, Sprite, Shape, Camera2D, BloomSettings, BlendMode, Material2d, RenderLayer, SortOrder, Visible, RigidBody, Collider, MixerTrack, AudioEmitter), SceneDef/SceneNodeDef scene format in engine_assets with index-based parent-child references and RON roundtrip support. Generic asset loading and caching via per-type AssetServer<T> with Handle<T> typed references, reference counting (add/clone_handle/remove with eviction at zero), RON file loading (load(path) with AssetError for I/O and parse failures), axiom2d facade re-exports engine_assets prelude.

Snapshot testing via insta across 3 crates: scene serialization snapshots (minimal scene, full node, ConvexPolygon collider, shape variants, audio emitter, material with textures/uniforms), Debug output snapshots (Collider::ConvexPolygon, Material2d, ShapeVariant::Polygon). Snapshot files committed under `crates/*/src/snapshots/`.

Property-based testing via proptest across 7 crates: mathematical invariants (Color::from_u8 range, FixedTimestep accumulator bounds, Gaussian weight normalization/symmetry, camera coordinate inverse roundtrip, distance attenuation range, constant-power panning), serde roundtrips (Color, Transform2D, Pixels, Seconds), ECS invariants (hierarchy Children sort), UI invariants (anchor offset, flex monotonicity), asset invariants (ref-count lifecycle).

**Placeholder crates (empty):** *(none — all crates are implemented)*.

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
- [x] `MouseState` resource: screen_pos (Vec2), world_pos (Vec2), button states (pressed/just_pressed/just_released via HashSet<MouseButton>), scroll_delta (Vec2 per frame, accumulated via add_scroll_delta, zeroed by clear_frame_state)
- [x] `MouseEventBuffer` resource: staging area for button events from winit (Vec<(MouseButton, ElementState)> + push/drain)
- [x] `mouse_input_system` in Phase::Input — drains MouseEventBuffer into MouseState button state
- [x] `mouse_world_pos_system` in Phase::PostUpdate — reads MouseState.screen_pos + Camera2D + WindowSize, writes MouseState.world_pos via screen_to_world()
- [x] App bridges winit CursorMoved → handle_cursor_moved (sets screen_pos directly), MouseInput → handle_mouse_button (pushes to MouseEventBuffer), MouseWheel → handle_mouse_wheel (accumulates scroll_delta, handles both LineDelta and PixelDelta)
- [x] ActionMap extended with mouse_bindings (HashMap<String, Vec<MouseButton>>), bind_mouse(), mouse_bindings_for(). MouseState has action_pressed/action_just_pressed via ActionMap
- [x] `input_system` in Phase::Input — reads buffered winit events via `InputEventBuffer`, updates InputState
- [x] Programmatic population for testing (`press()`/`release()` methods, no hardware needed)
- [x] Add engine_input to axiom2d facade re-exports
- [x] Tests: press/release tracking, just_pressed lasts one frame, mouse button/scroll/position/action tests
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

### Step 3.5 — Vector Graphics (Lyon) `[DONE]`
**Crate:** engine_render
**New dep:** lyon

- [x] Lyon path tessellation → CPU vertex/index buffers via `tessellate()` pure function
- [x] `Shape` component with `ShapeVariant` enum: Circle { radius }, Polygon { points }
- [x] `shape_render_system` in Phase::Render (visibility filtering, RenderLayer/SortOrder sorting, frustum culling)
- [x] Tests: tessellation validity, AABB computation, system behavior (22 tests)

### Step 3.6 — Post-Processing Framework `[DONE]`
**Crate:** engine_render

- [x] `apply_post_process()` method on Renderer trait — triggers post-processing in present()
- [x] Linear pass chain: Clear → Sprite → Shape → PostProcess → Present (inside WgpuRenderer::present())
- [x] Bloom pass: brightness extraction → separable 9-tap Gaussian blur (H+V) → additive composite
- [x] Fullscreen quad utility: `FULLSCREEN_QUAD_VERTICES` covering NDC [-1,1] space, shared `QUAD_INDICES`
- [x] `BloomSettings` resource: enabled, threshold, intensity, blur_radius
- [x] `post_process_system` in Phase::Render (after shape_render_system) — uses `Option<Res<BloomSettings>>`, no-op when absent
- [x] `compute_gaussian_weights(radius)` pure function for 1D separable kernel computation
- [x] Offscreen scene texture + half-res ping/pong textures for bloom pipeline (lazily created on first use)
- [x] Registered in DefaultPlugins (render feature)

### Step 3.7 — Material System `[DONE]`
**Crate:** engine_render

- [x] `Material2d { shader: ShaderHandle, textures: Vec<TextureBinding>, uniforms: Vec<u8>, blend_mode: BlendMode }`
- [x] Shader variant caching by preprocessor defines (`#ifdef` preprocessing via `preprocess()` pure function)
- [x] `BlendMode` enum: Alpha, Additive, Multiply

---

## Phase 4: Audio

### Step 4.1 — Audio Backend Trait `[DONE]`
**Crate:** engine_audio
**New dep:** cpal

- [x] `trait AudioBackend { fn play(&mut self, sound: &SoundData) -> PlaybackId; fn stop(&mut self, id: PlaybackId); fn set_volume(&mut self, volume: f32); }`
- [x] `NullAudioBackend` — no-op for testing (auto-incrementing PlaybackId)
- [x] `CpalBackend` — real audio output via cpal (lazy device init, Arc<Mutex<SharedState>> for testable internals)
- [x] `AudioRes` resource (same pattern as RendererRes)
- [x] `PlaybackId(u32)` and `SoundData { samples, sample_rate, channels }` value types
- [x] `audio` feature flag in axiom2d facade (default on), DefaultPlugins inserts AudioRes
- [x] Prelude re-exports for engine_audio and axiom2d

### Step 4.2 — Sound Synthesis (fundsp) `[DONE]`
**Crate:** engine_audio
**New dep:** fundsp

- [x] `SoundEffect` — code-defined sounds using fundsp DSP graph notation
- [x] `SoundLibrary` resource: named sound effects
- [x] `play_sound_system` reacting to `PlaySound` event/command

### Step 4.3 — Game Audio Features `[DONE]`
**Crate:** engine_audio
**New deps:** engine_scene, engine_core, glam (added to engine_audio)

- [x] Mixer tracks: `MixerTrack` enum (Master, Music, Sfx, Ambient, Ui) + `MixerState` resource with per-track volumes
- [x] Per-track volume in `mix_into`: effective volume = global × track (multiplicative stacking)
- [x] `AudioBackend` trait extended: `play_on_track()`, `set_track_volume()` — implemented on `NullAudioBackend` and `CpalBackend`
- [x] `PlaySound` extended: `track` field (default Sfx), `emitter` (optional Entity), `spatial_gains` (optional)
- [x] Spatial 2D audio: `AudioListener` marker + `AudioEmitter` component (volume, max_distance)
- [x] Pure functions: `distance_attenuation()` (linear falloff), `compute_pan()` (constant-power stereo panning), `compute_spatial_gains()`
- [x] `spatial_audio_system` in Phase::PostUpdate — computes stereo gains from listener/emitter positions via GlobalTransform2D
- [x] `play_sound_system` applies `SpatialGains` (mono→stereo conversion with per-channel gain), culls fully-silent sounds
- [x] Kira investigation: deferred adoption — kira 0.12 is capable but overlaps with implemented features; revisit for streaming audio, per-track effects, tweened volume

---

## Phase 5: Physics

### Step 5.1 — Physics Components & Trait `[DONE]`
**Crate:** engine_physics
**New dep:** rapier2d

- [x] `RigidBody` component: Dynamic, Static, Kinematic variants
- [x] `Collider` component: Circle(f32), Aabb(Vec2), ConvexPolygon(Vec<Vec2>)
- [x] `trait PhysicsBackend { fn step(&mut self, dt: Seconds); fn add_body(...); fn add_collider(...); }`
- [x] `NullPhysicsBackend` for testing
- [x] `RapierBackend` — real physics via rapier2d (PhysicsPipeline, body/collider management, entity-handle mapping)
- [x] `PhysicsRes` resource (newtype wrapping `Box<dyn PhysicsBackend + Send + Sync>`)
- [x] Prelude re-exports for all public types

### Step 5.2 — Collision Events `[DONE]`
**Crate:** engine_physics

- [x] `CollisionEvent { entity_a: Entity, entity_b: Entity, kind: CollisionKind }`
- [x] `CollisionKind` enum: Started, Stopped
- [x] `CollisionEventBuffer` resource (Vec+drain pattern, consistent with InputEventBuffer/PlaySoundBuffer)
- [x] `PhysicsBackend::drain_collision_events()` trait method
- [x] `physics_step_system` in Phase::PreUpdate
- [x] RapierBackend: ChannelEventCollector + collider_to_entity mapping + ActiveEvents::COLLISION_EVENTS
- [x] NullPhysicsBackend: drain returns empty Vec

### Step 5.3 — Physics-Transform Sync `[DONE]`
**Crate:** engine_physics

- [x] Bidirectional sync: Transform2D ↔ rapier rigid bodies (physics_sync_system reads body_position/body_rotation, writes Transform2D.position/.rotation independently)
- [x] Designed for Phase::PreUpdate (after physics_step_system, before transform_propagation_system in PostUpdate)
- [ ] Optional interpolation for smooth rendering between physics steps *(deferred — requires storing previous body state)*

--- 

## Phase 6: Assets & Serialization

### Step 6.1 — Serde + RON Integration `[DONE]`
**Crate:** engine_assets + engine_core + engine_render + engine_scene + engine_physics + engine_audio
**New deps:** serde (with derive), ron, glam serde feature

- [x] Add `Serialize`/`Deserialize` derives to core types: Color, Transform2D, Rect, Pixels, Seconds, TextureId, EntityId
- [x] Add `Serialize`/`Deserialize` to render types: Sprite, Shape, ShapeVariant, Camera2D, BloomSettings, BlendMode, ShaderHandle, TextureBinding, Material2d
- [x] Add `Serialize`/`Deserialize` to scene types: RenderLayer, SortOrder, Visible
- [x] Add `Serialize`/`Deserialize` to physics types: RigidBody, Collider
- [x] Add `Serialize`/`Deserialize` to audio types: MixerTrack, AudioEmitter
- [x] Scene serialization to/from RON format: SceneDef + SceneNodeDef in engine_assets with index-based parent-child references
- [x] Tests: roundtrip serialize → deserialize equals original (25 new tests across all crates)

### Step 6.2 — Asset Loading & Caching `[DONE]`
**Crate:** engine_assets
**New deps:** bevy_ecs, thiserror

- [x] `Handle<T>` — generic Copy newtype wrapping u32 with PhantomData<fn() -> T> for type safety (unconditionally Send+Sync). Manual impls for Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug.
- [x] `AssetServer<T>` — `#[derive(Resource, Default)]` generic struct with `HashMap<u32, AssetEntry<T>>` storage and auto-incrementing id counter. Methods: `add(T) -> Handle<T>`, `get(Handle<T>) -> Option<&T>`, `get_mut(Handle<T>) -> Option<&mut T>`, `ref_count(Handle<T>) -> Option<usize>`, `clone_handle(Handle<T>)` (explicit ref-count increment), `remove(Handle<T>) -> bool` (decrement, evict at zero), `load(path) -> Result<Handle<T>, AssetError>` (RON file deserialization, requires `T: DeserializeOwned`).
- [x] `AssetError` — thiserror enum with `Io(std::io::Error)` and `Parse(ron::error::SpannedError)` variants.
- [x] Per-type AssetServer design (one `AssetServer<T>` per asset type in ECS World, not a single unified server). Consistent with ShaderRegistry/SoundLibrary patterns.
- [x] Compile-fail doctest verifying Handle<T> type safety (Handle<u32> ≠ Handle<String>).
- [x] Prelude re-exports: Handle, AssetServer, AssetError, SceneDef, SceneNodeDef.
- [x] axiom2d facade re-exports engine_assets prelude.
- [x] Tests: 13 new tests (id allocation, retrieval, ref counting lifecycle, error paths, RON file loading) + 1 doctest.

### Step 6.3 — Hot Reloading (Deferred) `[NOT STARTED]`
**Crate:** engine_assets

- [ ] File watching for RON parameter changes
- [ ] Reload textures, sounds, configs without restart
- [ ] `dev` feature flag to enable/disable

---

## Phase 7: UI

### Step 7.1 — Basic UI Layout `[DONE]`
**Crate:** engine_ui

- [x] `UiNode` component: size (Vec2), anchor (Anchor enum with 9 variants), margin (Margin struct), background (Option<Color>)
- [x] `Anchor` enum (TopLeft through BottomRight) + `anchor_offset` pure function for position computation
- [x] `Margin` struct (top/right/bottom/left) with `total_horizontal()`/`total_vertical()` helpers
- [x] `FlexLayout` component with `FlexDirection` (Row/Column) + gap spacing
- [x] `compute_flex_offsets` pure function for testable layout math (handles margin integration)
- [x] `ui_layout_system` — reads FlexLayout + Children + GlobalTransform2D, writes child Transform2D.position
- [x] `ui_render_system` — draws UiNode backgrounds via `draw_rect()`, respects anchor offset and EffectiveVisibility
- [x] `Text` component (data-only: content, font_size, color) — bitmap font glyph rendering deferred to Step 7.2
- [x] All types derive Serialize/Deserialize with RON roundtrip tests
- [x] Prelude re-exports all public types
- [x] SpyRenderer extended with `RectCallLog` + `with_rect_capture()` for draw_rect call assertion

### Step 7.2 — Widgets `[DONE]`
**Crate:** engine_ui
**New dep:** engine_input (for MouseState access in interaction system)

- [x] Button component (disabled field) + button_render_system (UiTheme-driven colors)
- [x] Panel component (border_color, border_width) + panel_render_system (background + 4-edge border)
- [x] ProgressBar component (value, max) + progress_bar_render_system (background + clamped fill rect)
- [x] Interaction component (None/Hovered/Pressed) + ui_interaction_system (AABB hit-testing via MouseState.world_pos + anchor_offset, visibility filtering, disabled button skip)
- [x] UiEvent enum (Clicked, HoverEnter, HoverExit, FocusGained, FocusLost) + UiEventBuffer resource (Vec+drain pattern)
- [x] FocusState resource (Option<Entity>) — click claims focus, focus transfer emits FocusLost/FocusGained events
- [x] UiTheme resource (normal/hovered/pressed/disabled colors, text_color, font_size) — opt-in, not inserted by DefaultPlugins
- [x] All types derive Serialize/Deserialize (except UiEvent — Entity is not serializable)
- [x] Prelude re-exports all new public types and systems
- [x] Shared test_helpers module with make_spy_world() (same pattern as engine_scene)

---

## Phase 8: Cross-Cutting Quality

### Step 8.1 — Property-Based Testing `[DONE]`
**New dep:** proptest

- [x] Mathematical invariants: Color::from_u8 output range, FixedTimestep accumulator bounds, Gaussian weights (sum/symmetry/center-max), camera world↔screen inverse roundtrip, distance_attenuation output range, compute_pan constant-power property
- [x] Serialization roundtrips: Color, Transform2D, Pixels, Seconds RON roundtrips via proptest
- [x] ECS invariants: hierarchy_maintenance_system Children vec always sorted (arbitrary child_count/parent_count)
- [x] UI invariants: anchor_offset TopLeft=ZERO / BottomRight=-size, flex layout output length and monotonic offsets
- [x] Asset invariants: AssetServer ref-count lifecycle (add/clone_handle/remove sequence for arbitrary clone counts)

### Step 8.2 — Snapshot Testing `[DONE]`
**New dep:** insta

- [x] Snapshot tests for complex struct Debug output
- [x] Scene serialization snapshots

### Step 8.3 — Visual Regression Testing `[NOT STARTED]`
**New deps:** image-compare

- [ ] Headless wgpu rendering to texture → readback pixels
- [ ] SSIM comparison against golden images (0.99 threshold)
- [ ] CI integration with llvmpipe software renderer

### Step 8.4 — DefaultPlugins & Feature Flags `[DONE]`

- [x] `DefaultPlugins` struct implementing `Plugin`: registers input, time, scene-graph, and render systems
- [x] `render` feature flag (default on) gates render systems and ClearColor *(audio, physics, dev, hot_reload deferred until those crates are implemented)*
- [x] `default-features = false` gives headless ECS-only core (input, time, scene-graph)

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
