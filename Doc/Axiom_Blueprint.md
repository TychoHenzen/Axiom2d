# Building an LLM-optimized 2D game engine in Rust

**A Bevy-inspired archetypal ECS, code-defined assets, WGSL shaders, and trait-abstracted subsystems form the ideal foundation for an LLM-friendly, fully testable 2D engine in Rust.** The key insight across all research is that LLMs generate correct code most reliably when APIs are pattern-based, strongly typed, and use plain functions over complex trait hierarchies. Every architectural decision below prioritizes three properties: predictability for code generation, isolation for independent subsystem work, and testability without hardware dependencies.

This report synthesizes analysis of five major Rust game engines (Bevy, Fyrox, Macroquad, ggez, Piston), current ECS library implementations, shader toolchains, audio DSLs, TDD practices, and emerging LLM-assisted game development workflows. The recommendations are concrete and opinionated — this is a blueprint, not a survey.

## Archetypal ECS wins for LLM code generation

The single most consequential architectural decision is the entity-component-system pattern. After analyzing all major Rust ECS libraries, **archetypal ECS (Bevy-style) produces the most predictable, LLM-friendly code patterns** by a significant margin.

In an archetypal ECS, entities sharing identical component sets are stored together in "archetype tables" for cache-friendly iteration. Queries like `Query<(&Position, &mut Velocity), With<Player>>` map directly to "give me entities with these components" — a natural-language concept LLMs handle well. Bevy's function-parameter system declaration is the critical pattern: systems are plain Rust functions where typed parameters encode data access. No trait implementations, no builder patterns, no closure captures:

```rust
#[derive(Component, Debug, Clone)]
struct Position { x: f32, y: f32 }

#[derive(Component, Debug, Clone)]
struct Velocity { x: f32, y: f32 }

fn movement(mut query: Query<(&mut Position, &Velocity)>) {
    for (mut pos, vel) in &mut query {
        pos.x += vel.x;
        pos.y += vel.y;
    }
}
```

The competing sparse-set approach (Specs, Shipyard) requires explicit `ReadStorage`/`WriteStorage` declarations and `.join()` semantics with lifetime annotations — patterns that LLMs generate incorrectly at much higher rates. **Specs and Legion are both effectively unmaintained** and should be avoided entirely.

For the custom engine, two viable paths exist. The first is using `bevy_ecs` as a standalone crate — it works independently of the full Bevy engine, even in `no_std` environments, and brings a battle-tested implementation with ~44k GitHub stars of community validation. The second is wrapping `hecs` (~1,500 LOC minimal archetypal ECS) with a custom scheduling layer. The `bevy_ecs` path is recommended unless compile-time constraints are severe, since it provides change detection (`Changed<T>`, `Added<T>`), automatic parallel scheduling, and the complete `World` + `Schedule` testing pattern that makes TDD straightforward.

Component registration should use derive macros with automatic registration — `#[derive(Component)]` is a clear signal that LLMs pattern-match reliably. Manual `world.register::<T>()` calls (as Specs required) are a forgotten step waiting to cause a runtime panic. Resources follow the same pattern: `#[derive(Resource)]` with `Res<T>` / `ResMut<T>` access in systems, maintaining a consistent mental model across the entire data layer.

For scheduling, use **named phases with explicit ordering**: `Input → PreUpdate → Update → PostUpdate → Render`. Systems auto-parallelize within phases. Simple `.chain()` or `.after(other_system)` handles explicit dependencies. Avoid complex conditional scheduling (`run_if()` with compound conditions) in the core API — it's a frequent source of LLM-generated bugs.

## Code-defined assets replace binary files entirely

Every asset in the engine — sprites, shapes, tilemaps, audio, shaders, materials — should be expressible as Rust code or RON data. This eliminates binary files from version control, makes assets LLM-generatable, and ensures everything is diffable and reviewable.

**Vector graphics and sprites** use a two-layer approach. `lyon` provides GPU-ready tessellation: its builder-pattern API takes paths (lines, Bézier curves) and produces vertex+index buffers directly uploadable to wgpu. For offline sprite rasterization, `tiny-skia` renders vector shapes to pixel buffers that become GPU textures. The workflow is: define shapes programmatically → rasterize with tiny-skia → pack into texture atlases using `guillotiere` or `rectangle-pack` → upload to GPU. Procedural textures use the `noise` crate (Perlin, Simplex, Worley, Fbm) composed via combinators.

**Audio** is where code-defined assets truly shine. `fundsp` provides an inline graph notation for DSP networks using Rust operators:

```rust
// Laser: descending frequency sweep with exponential decay
let laser = lfo(|t| 1000.0 * exp(-t * 5.0)) >> sine() * envelope(|t| exp(-t * 3.0));
// Explosion: filtered noise burst
let explosion = white() >> lowpass_hz(200.0, 1.0) * envelope(|t| exp(-t * 2.0));
```

This reads almost like mathematical notation, has zero-cost abstractions, and is remarkably LLM-friendly. Pair `fundsp` with `cpal` for cross-platform audio output. For higher-level game audio features (tweening, mixer tracks, clock-based timing), `kira` provides a game-oriented API.

**Shaders** use WGSL as the primary language — it's the modern standard for wgpu, well-documented, and LLMs handle its C-like syntax reliably. Define shaders as `const &str` or `include_str!()` in Rust. Use `naga-oil` for modular shader composition with `#define_import_path` and preprocessor directives, allowing complex shaders built from reusable modules. While `rust-gpu` (now community-maintained after Embark Studios archived it in October 2025) can compile Rust to SPIR-V, WGSL remains more practical for LLM-assisted development: more training data, no nightly requirement, and direct wgpu integration.

**Tilemaps and animations** are defined as Rust structs with `serde` derives, serializable to RON. Animation state machines use enums with transition tables (`HashMap<(AnimState, Trigger), AnimState>`). Spritesheets are defined as uniform grids or explicit rect regions — no proprietary formats.

**RON (Rusty Object Notation)** is the preferred serialization format for asset parameters. It supports struct names, enum variants with data, comments, and trailing commas — looking almost like Rust syntax. A hybrid approach works best: Rust code defines asset types and generation logic, while RON files define hot-reloadable parameters. The `assets_manager` crate provides built-in hot-reloading with file watching, concurrent caching, and format-agnostic loading.

## Every subsystem testable through trait abstraction

The TDD architecture rests on one principle: **hardware-dependent subsystems hide behind traits, enabling complete testing without GPUs, audio devices, or displays.**

```rust
#[cfg_attr(test, automock)]
pub trait Renderer {
    fn draw_sprite(&mut self, sprite: &SpriteData, transform: &Transform2D);
    fn clear(&mut self, color: Color);
    fn present(&mut self);
}

pub struct NullRenderer;  // No-op for unit tests
pub struct WgpuRenderer;  // Production GPU rendering
pub struct RecordingRenderer(Vec<DrawCall>);  // Asserts on draw calls
```

This same pattern applies to audio (`NullAudioBackend`), input (`InputState` populated programmatically), and physics. The canonical test pattern uses `World` + `Schedule` directly:

```rust
#[test]
fn movement_updates_position() {
    let mut world = World::new();
    let entity = world.spawn((
        Position { x: 0.0, y: 0.0 },
        Velocity { x: 1.0, y: 0.0 },
    )).id();
    let mut schedule = Schedule::default();
    schedule.add_systems(movement);
    schedule.run(&mut world);
    let pos = world.get::<Position>(entity).unwrap();
    assert_eq!(pos.x, 1.0);
}
```

**Visual regression testing** uses wgpu's headless mode — create a device without a surface, render to a texture with `RENDER_ATTACHMENT | COPY_SRC`, read pixels back via a staging buffer. Compare against golden images using `image-compare`'s SSIM-based comparison with a **0.99 threshold** rather than exact pixel matching (different GPU drivers produce 2-3 value differences on RGB channels). The `insta` crate with `cargo-insta review` manages snapshot updates.

**Deterministic game loops** are non-negotiable for reproducible tests. Use the "Fix Your Timestep" pattern with a fixed `DT` (e.g., 1/60 second). Replace real clocks with injectable mock time. For seeded randomness, use `ChaCha8Rng::seed_from_u64(seed)` — never `StdRng`, whose algorithm may change between Rust versions. Watch for hidden non-determinism in `HashMap` iteration order; use `BTreeMap` or `ahash` with a fixed seed.

**Property-based testing** with `proptest` catches invariant violations that unit tests miss. The highest-value targets are physics constraints ("no entity passes through walls regardless of velocity"), serialization roundtrips ("serialize then deserialize equals original"), and ECS invariants ("no entity simultaneously has `Alive` and `Dead` components").

For CI with GPU tests, use `llvmpipe` (Mesa software renderer) on Linux with `WGPU_BACKEND=gl` and `LIBGL_ALWAYS_SOFTWARE=1`. Categorize tests into tiers: fast unit tests (< 5 seconds, run on every save), integration tests (< 30 seconds, run before commit), and visual regression tests (run in CI only).

## API design that guides LLMs toward correct code

Research from Microsoft's RustAssistant project shows that feeding Rust compiler errors back to an LLM achieves **~74% fix rate** on real-world compilation errors. This means every constraint encoded in the type system is an active collaborator in LLM code correctness. The engine's API should maximize type-level constraints.

**Plain structs with `Default`** are the most reliable pattern for LLM generation. LLMs understand struct literal syntax and `..Default::default()` perfectly:

```rust
let config = SpriteConfig {
    texture: textures.player,
    position: Vec2::new(100.0, 200.0),
    ..Default::default()
};
```

**NewType wrappers** prevent parameter confusion — a common LLM failure mode:

```rust
pub struct TextureId(u32);
pub struct EntityId(u64);
pub struct Pixels(f32);
pub struct Seconds(f32);

fn move_entity(entity: EntityId, dx: Pixels, dy: Pixels, dt: Seconds) { /* ... */ }
```

**Enums constrain valid states** and enable exhaustive matching. `BlendMode::Alpha` is unambiguous; `blend_mode: u32` is not. The **typestate pattern** encodes valid state transitions at the type level, making it impossible for LLMs to generate code that calls methods in the wrong order — the compiler rejects it before runtime.

**Documentation doubles as LLM context.** Every public item needs a one-line summary, semantic description, and an `# Examples` section with full `use` statements. Doc tests compile and run via `cargo test`, so examples stay verified. Consider a top-level `docs/llms.txt` file following the emerging convention for providing LLM-friendly project descriptions.

Module boundaries should enable an LLM to work on one subsystem with minimal context. Each crate exports a `prelude` module with common types. Trait boundaries between crates mean physics code never needs to know about wgpu. Limit indirection to ≤2 levels — Cursor engineers specifically noted this makes LLMs substantially better at reasoning about codebases.

## Scene hierarchy designed for 2D simplicity

A flat ECS with opt-in parent-child relationships (Bevy's model) is the right approach. Entities are flat by default; hierarchy is added via `ChildOf(parent)` components. For 2D games, hierarchies are typically shallow (2-3 levels), so ECS-based traversal outperforms pointer-based trees.

Use a **simplified 2D transform** rather than Bevy's 3D-based system:

```rust
#[derive(Component)]
struct Transform2D {
    position: Vec2,
    rotation: f32,  // radians
    scale: Vec2,
}

#[derive(Component)]
struct GlobalTransform2D(Affine2);  // Engine-managed, computed from hierarchy
```

Transform propagation runs in `PostUpdate`: root entities copy `Transform2D` to `GlobalTransform2D`; children compute `parent.GlobalTransform2D * child.Transform2D`. Use change detection to skip unchanged subtrees.

For **render ordering**, use a two-level system: `RenderLayer` (named enum: `Background`, `World`, `Characters`, `Foreground`, `UI`) plus `SortOrder` (i32 within layer). This is far more LLM-friendly than Z-coordinate abuse with magic numbers. Y-sorting for top-down games computes `SortOrder` from the entity's Y position.

**Visibility** uses a simple `Visible(bool)` component with inheritance through the hierarchy. Frustum culling simplifies to AABB-vs-AABB (camera viewport rect vs entity bounding box). For tilemaps, use grid-based culling as a fast path — calculate camera bounds in tile coordinates and only process visible cells.

**Tilemaps should NOT be individual entities per tile** (performance disaster). Instead, one entity per tilemap chunk handles its own internal rendering and participates in the scene hierarchy as a single node.

## The 2D shader and rendering pipeline

**wgpu** is the only serious GPU abstraction for Rust — safe, cross-platform (Vulkan, Metal, DX12, OpenGL, WebGPU), and well-maintained. For 2D rendering, use **instanced quad rendering**: a single quad vertex buffer plus an instance buffer with per-sprite transforms, UV rects, and colors. This minimizes draw calls.

The render pipeline follows a linear pass chain — simpler than Bevy's full render graph but sufficient for 2D:

```
[Clear] → [Sprite Pass] → [Particle Pass] → [Post-Process: Bloom] → 
[Post-Process: Color Grade] → [UI Pass] → [Present]
```

Each pass implements a `RenderPass` trait with a `render(&self, encoder, view)` method. Post-processing uses fullscreen quad passes reading from the previous pass's output texture.

**Post-processing in WGSL** follows standard patterns: brightness extraction → separable Gaussian blur (ping-pong between two textures) → composite for bloom; UV offset via `sin`/`cos` with a time uniform for water distortion; noise-texture threshold comparison for dissolve effects. **Particle systems** use compute shaders updating position/velocity/life buffers on the GPU, rendered as instanced point sprites.

A material system maps shader + parameters + textures into a data-driven definition:

```rust
struct Material2d {
    shader: ShaderHandle,
    textures: Vec<TextureBinding>,
    uniforms: Vec<u8>,  // Shader-compatible layout via `encase` crate
    blend_mode: BlendMode,
}
```

Shader variants use `naga-oil`'s `#ifdef` preprocessor directives, with specialized pipelines cached by variant key hash.

## Workspace structure optimized for incremental compilation

The engine uses a **flat workspace layout under `crates/`** with a virtual manifest at root — the pattern recommended by the rust-analyzer creator and validated by Bevy's architecture:

```
my_engine/
├── Cargo.toml                    # Virtual manifest (workspace only)
├── .cargo/config.toml            # Fast linker, sccache config
├── docs/
│   ├── llms.txt                  # LLM-friendly project description
│   └── ARCHITECTURE.md
├── crates/
│   ├── engine_core/              # Types, math, Color, IDs, Time
│   ├── engine_ecs/               # ECS: World, Entity, Component, System, Query
│   ├── engine_app/               # App builder, Plugin trait, schedules, state
│   ├── engine_render/            # wgpu renderer, sprites, camera, shaders, batching
│   ├── engine_audio/             # cpal + fundsp, playback, synthesis
│   ├── engine_input/             # Keyboard, mouse, gamepad, action mapping
│   ├── engine_physics/           # rapier2d wrapper, collision events
│   ├── engine_scene/             # Hierarchy, transforms, serialization
│   ├── engine_assets/            # Loading, caching, hot-reload
│   ├── engine_ui/                # Basic widgets, layout
│   └── my_engine/                # Thin facade: re-exports with feature flags
├── examples/
│   ├── hello_world/
│   ├── platformer/
│   └── particles/
└── tools/
    └── xtask/                    # Build automation (cargo xtask pattern)
```

**Workspace-level dependency management** ensures all crates use identical versions:

```toml
[workspace.dependencies]
wgpu = "24"
winit = "0.30"
glam = "0.29"
serde = { version = "1.0", features = ["derive"] }
thiserror = "2.0"
image = { version = "0.25", default-features = false, features = ["png"] }
rapier2d = "0.22"
lyon = "1.0"
tiny-skia = "0.11"
fundsp = "0.23"
cpal = "0.15"
noise = "0.9"
ron = "0.8"
proptest = "1.10"
```

**Feature flags** follow Bevy's three-tier model: subsystem features (`render`, `audio`, `physics`), development features (`dev`, `hot_reload`, `debug_draw`), and asset format features (`png`, `ogg`). Compiling with `default-features = false` gives a headless core for testing.

The plugin architecture uses two forms: **function plugins** (`fn my_plugin(app: &mut App)`) for simple cases and **struct plugins** with `Default` for configurable ones. A `DefaultPlugins` group provides the standard setup. Each plugin is self-contained — all systems, resources, and events a feature needs are registered in one `build()` method.

## Compile times stay fast with the right tooling

Rust compile times are the primary threat to a TDD workflow. The target is **< 3 seconds for incremental rebuilds** and **< 5 seconds for unit test runs**.

The `.cargo/config.toml` configuration combines three critical optimizations:

```toml
[build]
rustc-wrapper = "sccache"

[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]

[profile.dev]
debug = "line-tables-only"
opt-level = 1

[profile.dev.package."*"]
opt-level = 3    # Optimize dependencies fully
```

**`mold`** (Linux) or **`lld`** (cross-platform) replaces the default linker, reducing link times dramatically. The **cranelift backend** (`codegen-backend = "cranelift"` on nightly) provides ~25% faster clean builds and ~75% faster incremental builds. **`sccache`** caches compilation results across builds.

**Dynamic linking** during development follows Bevy's pattern — compile the engine as a dynamic library for development, reducing incremental recompiles from ~30 seconds to ~1-3 seconds. Never ship with dynamic linking.

**`cargo-nextest`** replaces the default test runner, executing each test in its own process for true parallelism — up to **3x faster** than `cargo test` for large suites. Combine integration tests into a single binary (`tests/main.rs` with `mod` includes) to eliminate redundant linking.

Workspace structure matters enormously: place frequently-changed game logic in leaf crates so changes don't cascade. Use `cargo build --timings` to visualize the compilation critical path and `cargo llvm-lines` to identify generics generating excessive LLVM IR. The "inner non-generic function" pattern reduces monomorphization overhead for public generic APIs.

## MCP servers and LLM workflow integration

The emerging Model Context Protocol ecosystem already includes MCP servers for Unity (most mature, with WebSocket bridge), Unreal Engine (36+ tools), and Godot. Building an MCP server for the custom engine would expose scene CRUD operations, entity inspection, game state queries, screenshot capture, and build/run commands to any MCP-compatible LLM tool.

Key tools to expose: `create_entity(name, components)`, `set_component(entity, type, values)`, `get_scene_hierarchy()` (returns current scene tree as LLM context), `run_game()` / `stop_game()`, and `take_screenshot()` for visual feedback. The GDC 2025 talk "Build Faster, Iterate More" demonstrated MCP servers as universal middleware between LLMs and game engines.

For development-time LLM interaction without an MCP server, the most effective pattern is **template-based scaffolding**. Define canonical examples for every common task (new component, new system, new plugin, new scene, new shader effect) in the `examples/` directory. LLMs use these as generation templates with high reliability. The `docs/llms.txt` file provides project-level context: architecture overview, crate responsibilities, common patterns, and dependency relationships.

## Recommended dependency map

| Category | Crate | Purpose |
|----------|-------|---------|
| **ECS** | `bevy_ecs` or `hecs` | Archetypal entity-component-system |
| **Math** | `glam` | Vec2, Mat3, Affine2 (game-optimized, SIMD) |
| **GPU** | `wgpu` | Cross-platform graphics API |
| **Windowing** | `winit` | Cross-platform window management |
| **Shaders** | `naga-oil` | Modular WGSL shader composition |
| **Vector graphics** | `lyon` | Path tessellation to GPU triangles |
| **CPU rasterization** | `tiny-skia` | Procedural sprite generation |
| **Audio I/O** | `cpal` | Cross-platform audio output |
| **Audio DSP** | `fundsp` | Code-defined sound synthesis |
| **Physics** | `rapier2d` | 2D rigid body physics |
| **Noise** | `noise` | Procedural texture generation |
| **Serialization** | `serde` + `ron` | Asset data, scene files, config |
| **Errors** | `thiserror` | Derive-based error types |
| **Images** | `image` | PNG loading, golden-file tests |
| **Testing** | `proptest` | Property-based testing |
| **Testing** | `insta` | Snapshot testing |
| **Testing** | `image-compare` | Visual regression (SSIM) |
| **Testing** | `mockall` | Trait mocking |
| **Test runner** | `cargo-nextest` | Parallel test execution |
| **Rect packing** | `guillotiere` | Texture atlas generation |
| **Assets** | `assets_manager` | Hot-reloading asset cache |

## Conclusion

The engine architecture converges on a clear set of principles: **archetypal ECS with function-based systems** for predictable LLM code generation, **trait-abstracted hardware backends** for complete testability, **WGSL shaders + fundsp audio + lyon/tiny-skia graphics** for code-defined assets, and a **flat workspace of focused crates** for both fast compilation and isolated LLM work sessions.

The most counterintuitive finding is that simplicity beats power for LLM-assisted development. Macroquad's immediate-mode API generates the most correct simple code, but lacks architecture. Bevy's full engine has massive API surface area that confuses LLMs. The sweet spot is a Bevy-ECS-powered core with a deliberately small, consistent, type-driven API surface — every component uses `#[derive(Component)]`, every config uses plain structs with `Default`, every subsystem hides behind a trait, and every test runs against a `World` + `Schedule` without touching hardware.

The novel insight for the LLM-friendly dimension is that **Rust's compiler is the LLM's best pair programmer**. The ~74% error self-correction rate means that encoding constraints in types, using newtypes, enums, and typestate patterns, creates a feedback loop where the compiler catches LLM mistakes and provides enough context for the LLM to fix them. Design the API to maximize the number of bugs that are compile-time errors rather than runtime failures, and the LLM-assisted development workflow becomes dramatically more reliable.