# Terrain Viewer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a standalone terrain viewer binary on the real engine pipeline for iterating on procedural terrain materials, auto-tile transitions, and WFC grid generation.

**Architecture:** Two new crates — `crates/terrain/` (library: materials, dual-grid, WFC) and `crates/terrain_viewer/` (binary: engine app with UI controls). The terrain shader (`terrain.wgsl`) uses a master dispatch + per-type sub-shader pattern. Everything renders through the existing `Material2d` + `ShaderRegistry` pipeline with zero engine modifications.

**Tech Stack:** Rust, WGSL, `bevy_ecs`, `engine_app`, `engine_render`, `engine_ui`, `engine_input`, `rand_chacha`

**Spec:** `docs/superpowers/specs/2026-04-15-terrain-viewer-design.md`

---

## File Structure

```
crates/terrain/
  Cargo.toml
  src/
    lib.rs                    # re-exports, TerrainMaterial registry resource
    material.rs               # TerrainId, TerrainKind, TerrainMaterial, MaterialParams
    dual_grid.rs              # DualGrid, VisualTile, corner bitmask
    wfc.rs                    # WFC solver, ConstraintTable, Grid
    shader.rs                 # shader source embedding + registration
    shader/
      terrain.wgsl            # master shader + sub-shaders + noise helpers
  tests/
    main.rs
    suite/
      mod.rs
      material.rs
      dual_grid.rs
      wfc.rs

crates/terrain_viewer/
  Cargo.toml
  src/
    main.rs                   # App setup, systems, UI spawning
```

---

## Task 1: Create `terrain` crate scaffold with data types

**Files:**
- Create: `crates/terrain/Cargo.toml`
- Create: `crates/terrain/src/lib.rs`
- Create: `crates/terrain/src/material.rs`
- Create: `crates/terrain/tests/main.rs`
- Create: `crates/terrain/tests/suite/mod.rs`
- Create: `crates/terrain/tests/suite/material.rs`

- [ ] **Step 1: Create `Cargo.toml`**

```toml
[package]
name = "terrain"
edition.workspace = true
version.workspace = true

[dependencies]
engine_core = { path = "../engine_core" }
engine_render = { path = "../engine_render" }
engine_ecs = { path = "../engine_ecs" }
bevy_ecs = { workspace = true }
bytemuck = { workspace = true }
glam = { workspace = true }
rand = { workspace = true }
rand_chacha = { workspace = true }

[dev-dependencies]
proptest = { workspace = true }

[lints]
workspace = true
```

- [ ] **Step 2: Create `src/material.rs` with core data types**

```rust
use bytemuck::{Pod, Zeroable};

/// Unique terrain type identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TerrainId(pub u8);

/// Determines which sub-shader branch evaluates this terrain.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum TerrainKind {
    Grass = 0,
    Stone = 1,
    Water = 2,
    Sand = 3,
    Lava = 4,
    Snow = 5,
}

impl TerrainKind {
    pub const ALL: [Self; 6] = [
        Self::Grass,
        Self::Stone,
        Self::Water,
        Self::Sand,
        Self::Lava,
        Self::Snow,
    ];

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Grass => "Grass",
            Self::Stone => "Stone",
            Self::Water => "Water",
            Self::Sand => "Sand",
            Self::Lava => "Lava",
            Self::Snow => "Snow",
        }
    }
}

/// Describes a terrain type's visual parameters. Colors and floats are
/// runtime-adjustable via the uniform buffer without recompiling the shader.
#[derive(Clone, Debug, PartialEq)]
pub struct TerrainMaterial {
    pub id: TerrainId,
    pub kind: TerrainKind,
    pub color_a: [f32; 3],
    pub color_b: [f32; 3],
    /// frequency, amplitude, warp strength, scale.
    pub params: [f32; 4],
    /// Type-specific tunables (wind direction, strata angle, etc.).
    pub extra: [f32; 4],
}

/// GPU-compatible material parameters. Matches the WGSL `MaterialParams` struct.
/// 64 bytes, 16-byte aligned.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct MaterialParams {
    pub color_a: [f32; 4], // rgb + padding
    pub color_b: [f32; 4], // rgb + padding
    pub params: [f32; 4],  // frequency, amplitude, warp, scale
    pub extra: [f32; 4],   // type-specific
}

impl TerrainMaterial {
    /// Pack into GPU-compatible `MaterialParams`.
    #[must_use]
    pub fn to_gpu_params(&self) -> MaterialParams {
        MaterialParams {
            color_a: [self.color_a[0], self.color_a[1], self.color_a[2], 0.0],
            color_b: [self.color_b[0], self.color_b[1], self.color_b[2], 0.0],
            params: self.params,
            extra: self.extra,
        }
    }
}

/// Default material definitions for the initial terrain set.
#[must_use]
pub fn default_materials() -> Vec<TerrainMaterial> {
    vec![
        TerrainMaterial {
            id: TerrainId(0),
            kind: TerrainKind::Grass,
            color_a: [0.18, 0.42, 0.12],
            color_b: [0.30, 0.58, 0.20],
            params: [6.0, 0.4, 0.3, 1.0],
            extra: [0.3, 0.0, 0.0, 0.0], // wind direction
        },
        TerrainMaterial {
            id: TerrainId(1),
            kind: TerrainKind::Stone,
            color_a: [0.45, 0.42, 0.38],
            color_b: [0.58, 0.55, 0.50],
            params: [4.0, 0.3, 0.1, 8.0],
            extra: [0.0, 0.0, 0.0, 0.0],
        },
        TerrainMaterial {
            id: TerrainId(2),
            kind: TerrainKind::Water,
            color_a: [0.10, 0.25, 0.55],
            color_b: [0.20, 0.45, 0.70],
            params: [3.0, 0.5, 0.6, 6.0],
            extra: [1.0, 0.0, 0.0, 0.0], // animation speed
        },
        TerrainMaterial {
            id: TerrainId(3),
            kind: TerrainKind::Sand,
            color_a: [0.76, 0.65, 0.42],
            color_b: [0.85, 0.75, 0.52],
            params: [8.0, 0.2, 0.1, 1.0],
            extra: [0.5, 0.0, 0.0, 0.0], // ripple direction
        },
        TerrainMaterial {
            id: TerrainId(4),
            kind: TerrainKind::Lava,
            color_a: [0.25, 0.05, 0.02],
            color_b: [1.0, 0.35, 0.05],
            params: [5.0, 0.6, 0.4, 6.0],
            extra: [0.3, 0.0, 0.0, 0.0], // drift speed
        },
        TerrainMaterial {
            id: TerrainId(5),
            kind: TerrainKind::Snow,
            color_a: [0.90, 0.92, 0.95],
            color_b: [0.80, 0.85, 0.92],
            params: [4.0, 0.08, 0.05, 1.0],
            extra: [0.0, 0.0, 0.0, 0.0],
        },
    ]
}
```

- [ ] **Step 3: Create `src/lib.rs`**

```rust
pub mod material;

use bevy_ecs::prelude::Resource;
use material::TerrainMaterial;

/// ECS resource holding all terrain material definitions.
#[derive(Resource, Debug, Clone)]
pub struct TerrainMaterials(pub Vec<TerrainMaterial>);
```

- [ ] **Step 4: Create test scaffold**

`tests/main.rs`:
```rust
#![allow(clippy::unwrap_used)]

mod suite;
```

`tests/suite/mod.rs`:
```rust
mod material;
```

`tests/suite/material.rs`:
```rust
use terrain::material::{
    MaterialParams, TerrainId, TerrainKind, TerrainMaterial, default_materials,
};

#[test]
fn when_packing_material_to_gpu_then_colors_are_padded_to_vec4() {
    // Arrange
    let mat = TerrainMaterial {
        id: TerrainId(0),
        kind: TerrainKind::Grass,
        color_a: [0.1, 0.2, 0.3],
        color_b: [0.4, 0.5, 0.6],
        params: [1.0, 2.0, 3.0, 4.0],
        extra: [5.0, 0.0, 0.0, 0.0],
    };

    // Act
    let gpu = mat.to_gpu_params();

    // Assert
    assert_eq!(gpu.color_a, [0.1, 0.2, 0.3, 0.0]);
    assert_eq!(gpu.color_b, [0.4, 0.5, 0.6, 0.0]);
    assert_eq!(gpu.params, [1.0, 2.0, 3.0, 4.0]);
}

#[test]
fn when_calling_default_materials_then_returns_six_distinct_types() {
    // Act
    let materials = default_materials();

    // Assert
    assert_eq!(materials.len(), 6);
    for (i, mat) in materials.iter().enumerate() {
        assert_eq!(mat.id.0, i as u8);
    }
}

#[test]
fn when_gpu_params_size_then_is_64_bytes() {
    // Assert — validates WGSL alignment assumption
    assert_eq!(std::mem::size_of::<MaterialParams>(), 64);
}
```

- [ ] **Step 5: Verify tests pass**

Run: `cargo.exe test -p terrain`

Expected: 3 tests pass.

- [ ] **Step 6: Commit**

```bash
git add crates/terrain/
git commit -m "feat(terrain): add terrain crate with TerrainMaterial data types"
```

---

## Task 2: Write `terrain.wgsl` with noise primitives and grass sub-shader

**Files:**
- Create: `crates/terrain/src/shader/terrain.wgsl`
- Create: `crates/terrain/src/shader.rs`
- Modify: `crates/terrain/src/lib.rs`

- [ ] **Step 1: Create `src/shader.rs` with shader source embedding**

```rust
use engine_render::shader::{ShaderHandle, ShaderRegistry};

pub const TERRAIN_WGSL: &str = include_str!("shader/terrain.wgsl");

/// Register the terrain shader and return its handle.
pub fn register_terrain_shader(registry: &mut ShaderRegistry) -> ShaderHandle {
    registry.register(TERRAIN_WGSL)
}
```

- [ ] **Step 2: Add `pub mod shader;` to `src/lib.rs`**

```rust
pub mod material;
pub mod shader;

use bevy_ecs::prelude::Resource;
use engine_render::shader::ShaderHandle;
use material::TerrainMaterial;

/// ECS resource holding all terrain material definitions.
#[derive(Resource, Debug, Clone)]
pub struct TerrainMaterials(pub Vec<TerrainMaterial>);

/// ECS resource holding the terrain shader handle.
#[derive(Resource, Debug, Clone, Copy)]
pub struct TerrainShader(pub ShaderHandle);
```

- [ ] **Step 3: Create `src/shader/terrain.wgsl`**

This is the master shader. Start with noise primitives + grass sub-shader only. Other sub-shaders are added in Task 6.

```wgsl
// === terrain.wgsl — master terrain shader ===

struct MaterialParams {
    color_a: vec4<f32>,
    color_b: vec4<f32>,
    params: vec4<f32>,   // frequency, amplitude, warp, scale
    extra: vec4<f32>,    // type-specific
};

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

@group(0) @binding(0) var<uniform> view_proj: mat4x4<f32>;
@group(1) @binding(0) var<uniform> model: mat4x4<f32>;
@group(2) @binding(0) var<uniform> material: array<vec4<f32>, 16>;

// Material uniform unpacking: 16 vec4s = 256 bytes.
// Layout: [0] = corner_types (as u32 bits), world_pos.xy, seed
//         [1..4] = MaterialParams for corner 0
//         [5..8] = MaterialParams for corner 1
//         [9..12] = MaterialParams for corner 2
//         [13..15] = MaterialParams for corner 3 (partial, expand if needed)
// For Phase 1 (single material preview), we only use material[1..4].

fn unpack_type_id() -> u32 {
    return bitcast<u32>(material[0].x);
}

fn unpack_world_pos() -> vec2<f32> {
    return material[0].yz;
}

fn unpack_seed() -> f32 {
    return material[0].w;
}

fn unpack_params(offset: u32) -> MaterialParams {
    let base = offset;
    return MaterialParams(
        material[base],
        material[base + 1],
        material[base + 2],
        material[base + 3],
    );
}

// ============================================================
// Noise primitives
// ============================================================

fn hash21(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.x, p.y, p.x) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

fn hash22(p: vec2<f32>) -> vec2<f32> {
    let n = vec3<f32>(dot(p, vec2<f32>(127.1, 311.7)),
                       dot(p, vec2<f32>(269.5, 183.3)),
                       dot(p, vec2<f32>(419.2, 371.9)));
    return fract(sin(n.xy) * 43758.5453);
}

fn gradient_noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f); // smoothstep

    let a = hash21(i);
    let b = hash21(i + vec2<f32>(1.0, 0.0));
    let c = hash21(i + vec2<f32>(0.0, 1.0));
    let d = hash21(i + vec2<f32>(1.0, 1.0));

    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn fbm(p: vec2<f32>, octaves: i32) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var freq_p = p;
    for (var i = 0; i < octaves; i++) {
        value += amplitude * gradient_noise(freq_p);
        freq_p *= 2.0;
        amplitude *= 0.5;
    }
    return value;
}

fn voronoi(p: vec2<f32>) -> vec2<f32> {
    // Returns (distance to nearest cell edge, distance to nearest cell center)
    let n = floor(p);
    let f = fract(p);
    var min_dist = 8.0;
    var min_edge = 8.0;
    var nearest_center = vec2<f32>(0.0);
    for (var j = -1; j <= 1; j++) {
        for (var i = -1; i <= 1; i++) {
            let g = vec2<f32>(f32(i), f32(j));
            let o = hash22(n + g);
            let r = g + o - f;
            let d = dot(r, r);
            if d < min_dist {
                min_dist = d;
                nearest_center = n + g + o;
            }
        }
    }
    min_dist = sqrt(min_dist);
    // Second pass for edge distance
    for (var j = -1; j <= 1; j++) {
        for (var i = -1; i <= 1; i++) {
            let g = vec2<f32>(f32(i), f32(j));
            let o = hash22(n + g);
            let r = g + o - f;
            let cell = n + g + o;
            if distance(cell, nearest_center) > 0.001 {
                let edge_d = dot(0.5 * (nearest_center - (n + f) + r), normalize(r - (nearest_center - (n + f))));
                min_edge = min(min_edge, abs(edge_d));
            }
        }
    }
    return vec2<f32>(min_dist, min_edge);
}

fn domain_warp(p: vec2<f32>, strength: f32) -> vec2<f32> {
    let ox = fbm(p + vec2<f32>(0.0, 0.0), 2);
    let oy = fbm(p + vec2<f32>(5.2, 1.3), 2);
    return p + vec2<f32>(ox, oy) * strength;
}

// ============================================================
// Sub-shaders — one per terrain type
// ============================================================

fn grass(uv: vec2<f32>, p: MaterialParams) -> vec3<f32> {
    let freq = p.params.x;
    let amp = p.params.y;
    let warp = p.params.z;
    let wind_dir = p.extra.x;

    // Anisotropic stretch along wind direction
    let angle = wind_dir * 6.2832;
    let rot = mat2x2<f32>(cos(angle), -sin(angle), sin(angle), cos(angle));
    let stretched_uv = rot * uv * vec2<f32>(1.0, 2.5);

    // Layered directional noise
    let n1 = fbm(stretched_uv * freq, 3);
    let n2 = gradient_noise(uv * freq * 2.0 + vec2<f32>(17.0, 31.0));

    // Domain warp for organic flow
    let warped = domain_warp(uv * freq * 0.5, warp);
    let n3 = gradient_noise(warped * 3.0);

    let t = clamp(n1 * amp + n2 * 0.2 + n3 * 0.15, 0.0, 1.0);
    return mix(p.color_a.rgb, p.color_b.rgb, t);
}

fn stone(uv: vec2<f32>, p: MaterialParams) -> vec3<f32> {
    let freq = p.params.x;
    let amp = p.params.y;
    let cell_scale = p.params.w;

    let v = voronoi(uv * cell_scale);
    let crack = smoothstep(0.02, 0.0, v.y);
    let surface = fbm(uv * freq, 3) * amp;
    let base = mix(p.color_a.rgb, p.color_b.rgb, surface);
    return mix(base, base * 0.55, crack);
}

fn water(uv: vec2<f32>, p: MaterialParams) -> vec3<f32> {
    let freq = p.params.x;
    let amp = p.params.y;
    let warp_str = p.params.z;
    let cell_scale = p.params.w;

    // Caustic pattern from warped Voronoi
    let warped = domain_warp(uv * freq, warp_str);
    let v = voronoi(warped * cell_scale);
    let caustic = smoothstep(0.3, 0.0, v.x) * amp;

    let surface = fbm(uv * freq * 0.5, 2) * 0.3;
    let base = mix(p.color_a.rgb, p.color_b.rgb, surface);
    return base + vec3<f32>(caustic * 0.15);
}

fn sand(uv: vec2<f32>, p: MaterialParams) -> vec3<f32> {
    let freq = p.params.x;
    let amp = p.params.y;
    let ripple_dir = p.extra.x;

    // Directional ripple pattern
    let angle = ripple_dir * 6.2832;
    let dir = vec2<f32>(cos(angle), sin(angle));
    let ripple = sin(dot(uv * freq, dir) * 12.0 + fbm(uv * freq * 0.3, 2) * 4.0) * 0.5 + 0.5;

    // Grain sparkle
    let grain = step(0.97, hash21(floor(uv * freq * 20.0)));

    let t = ripple * amp;
    let base = mix(p.color_a.rgb, p.color_b.rgb, t);
    return base + vec3<f32>(grain * 0.08);
}

fn lava(uv: vec2<f32>, p: MaterialParams) -> vec3<f32> {
    let freq = p.params.x;
    let amp = p.params.y;
    let warp_str = p.params.z;
    let cell_scale = p.params.w;

    let v = voronoi(uv * cell_scale);
    let crack_glow = smoothstep(0.08, 0.0, v.y) * amp;

    // Surface variation on cooled plates
    let surface = fbm(uv * freq, 2) * 0.2;
    let crust = mix(p.color_a.rgb, p.color_a.rgb * 1.2, surface);
    let glow = p.color_b.rgb;

    return mix(crust, glow, crack_glow);
}

fn snow(uv: vec2<f32>, p: MaterialParams) -> vec3<f32> {
    let freq = p.params.x;
    let amp = p.params.y;

    // Very subtle surface undulation
    let n = fbm(uv * freq, 2) * amp;

    // Sparse sparkle
    let sparkle = step(0.985, hash21(floor(uv * freq * 30.0))) * 0.12;

    let base = mix(p.color_a.rgb, p.color_b.rgb, n);
    return base + vec3<f32>(sparkle);
}

// ============================================================
// Master dispatch
// ============================================================

fn eval_terrain(uv: vec2<f32>, type_id: u32, p: MaterialParams) -> vec3<f32> {
    switch type_id {
        case 0u { return grass(uv, p); }
        case 1u { return stone(uv, p); }
        case 2u { return water(uv, p); }
        case 3u { return sand(uv, p); }
        case 4u { return lava(uv, p); }
        case 5u { return snow(uv, p); }
        default { return p.color_a.rgb; }
    }
}

// ============================================================
// Vertex / Fragment entry
// ============================================================

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = view_proj * model * vec4<f32>(in.position, 0.0, 1.0);
    out.color = in.color;
    out.uv = in.uv;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let type_id = unpack_type_id();
    let world_pos = unpack_world_pos();
    let seed = unpack_seed();
    let params = unpack_params(1u);

    // Phase 1: single material on full quad
    let world_uv = world_pos + in.uv * 10.0 + vec2<f32>(seed * 17.0, seed * 31.0);
    let color = eval_terrain(world_uv, type_id, params);
    return vec4<f32>(color, 1.0);
}
```

- [ ] **Step 4: Verify the crate builds**

Run: `cargo.exe build -p terrain`

Expected: compiles without errors.

- [ ] **Step 5: Commit**

```bash
git add crates/terrain/src/shader.rs crates/terrain/src/shader/ crates/terrain/src/lib.rs
git commit -m "feat(terrain): add terrain.wgsl with noise primitives and 6 sub-shaders"
```

---

## Task 3: Create `terrain_viewer` binary with quad on screen

**Files:**
- Create: `crates/terrain_viewer/Cargo.toml`
- Create: `crates/terrain_viewer/src/main.rs`

- [ ] **Step 1: Create `Cargo.toml`**

```toml
[package]
name = "terrain_viewer"
edition.workspace = true
version.workspace = true

[[bin]]
name = "terrain_viewer"
path = "src/main.rs"

[dependencies]
terrain = { path = "../terrain" }
axiom2d = { path = "../axiom2d" }
engine_app = { path = "../engine_app" }
engine_core = { path = "../engine_core" }
engine_ecs = { path = "../engine_ecs" }
engine_input = { path = "../engine_input" }
engine_render = { path = "../engine_render" }
engine_scene = { path = "../engine_scene" }
engine_ui = { path = "../engine_ui" }
bevy_ecs = { workspace = true }
bytemuck = { workspace = true }
glam = { workspace = true }

[lints]
workspace = true
```

- [ ] **Step 2: Create `src/main.rs` with minimal viewer**

```rust
#![allow(clippy::unwrap_used)] // viewer binary, not library

use bevy_ecs::prelude::*;
use bytemuck::bytes_of;
use engine_app::prelude::*;
use engine_core::prelude::*;
use engine_ecs::schedule::Phase;
use engine_input::prelude::*;
use engine_render::camera::Camera2D;
use engine_render::material::Material2d;
use engine_render::shader::ShaderRegistry;
use engine_render::shape::components::{ColorMesh, ColorVertex, TessellatedColorMesh};
use engine_scene::prelude::*;
use glam::{Affine2, Vec2};
use terrain::material::{TerrainKind, default_materials};
use terrain::shader::register_terrain_shader;
use terrain::{TerrainMaterials, TerrainShader};

/// Currently selected terrain type index.
#[derive(Resource, Debug)]
struct SelectedTerrain(usize);

/// Entity that displays the terrain quad.
#[derive(Resource, Debug)]
struct TerrainQuadEntity(Entity);

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing_subscriber::filter::LevelFilter::WARN)
        .init();

    let mut app = App::new();
    setup(&mut app);
    app.run();
}

fn setup(app: &mut App) {
    app.add_plugin(DefaultPlugins);

    app.set_window_config(WindowConfig {
        title: "Terrain Viewer",
        width: 1024,
        height: 768,
        ..Default::default()
    });

    // Register shader and insert resources before post-splash setup.
    {
        let world = app.world_mut();
        let shader_handle = {
            let mut reg = world.resource_mut::<ShaderRegistry>();
            register_terrain_shader(&mut reg)
        };
        world.insert_resource(TerrainShader(shader_handle));

        let materials = default_materials();
        world.insert_resource(TerrainMaterials(materials));
        world.insert_resource(SelectedTerrain(0));
    }

    app.world_mut()
        .resource_mut::<PostSplashSetup>()
        .add_systems(spawn_viewer_scene);

    app.add_systems(Phase::Update, terrain_input_system);
}

fn spawn_viewer_scene(world: &mut World) {
    // Camera
    world.spawn(Camera2D::default());

    let shader = world.resource::<TerrainShader>().0;
    let materials = world.resource::<TerrainMaterials>().clone();
    let selected = world.resource::<SelectedTerrain>().0;

    // Build terrain quad — large quad centered at origin
    let half = 350.0_f32;
    let mesh = TessellatedColorMesh {
        vertices: vec![
            ColorVertex { position: [-half, -half], color: [1.0; 4], uv: [0.0, 0.0] },
            ColorVertex { position: [half, -half],  color: [1.0; 4], uv: [1.0, 0.0] },
            ColorVertex { position: [half, half],   color: [1.0; 4], uv: [1.0, 1.0] },
            ColorVertex { position: [-half, half],  color: [1.0; 4], uv: [0.0, 1.0] },
        ],
        indices: vec![0, 1, 2, 0, 2, 3],
    };

    let uniforms = build_single_material_uniform(&materials.0[selected]);

    let entity = world
        .spawn((
            Transform2D {
                position: Vec2::ZERO,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
            GlobalTransform2D(Affine2::IDENTITY),
            ColorMesh(mesh),
            Material2d {
                shader,
                uniforms,
                ..Material2d::default()
            },
            RenderLayer::Background,
            SortOrder::default(),
        ))
        .id();

    world.insert_resource(TerrainQuadEntity(entity));
}

/// Pack a single `TerrainMaterial` into the uniform buffer for Phase 1.
/// Layout: material[0] = (type_id_bits, world_x, world_y, seed)
///         material[1..4] = MaterialParams
fn build_single_material_uniform(mat: &terrain::material::TerrainMaterial) -> Vec<u8> {
    let gpu = mat.to_gpu_params();
    let type_id_f32 = f32::from_bits(mat.kind as u32);
    let header: [f32; 4] = [type_id_f32, 0.0, 0.0, 42.0]; // world_pos=(0,0), seed=42

    let mut buf = Vec::with_capacity(256);
    buf.extend_from_slice(bytes_of(&header));
    buf.extend_from_slice(bytes_of(&gpu));
    // Pad to 256 bytes (16 vec4s) for the uniform array
    buf.resize(256, 0);
    buf
}

fn terrain_input_system(
    input: Res<InputState>,
    mut selected: ResMut<SelectedTerrain>,
    materials: Res<TerrainMaterials>,
    quad: Res<TerrainQuadEntity>,
    mut query: Query<&mut Material2d>,
) {
    let count = materials.0.len();
    let mut changed = false;

    if input.just_pressed(KeyCode::ArrowRight) || input.just_pressed(KeyCode::KeyD) {
        selected.0 = (selected.0 + 1) % count;
        changed = true;
    }
    if input.just_pressed(KeyCode::ArrowLeft) || input.just_pressed(KeyCode::KeyA) {
        selected.0 = (selected.0 + count - 1) % count;
        changed = true;
    }

    // Number keys 1-6 for direct selection
    let key_map = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
    ];
    for (i, key) in key_map.iter().enumerate() {
        if i < count && input.just_pressed(*key) {
            selected.0 = i;
            changed = true;
        }
    }

    if changed {
        if let Ok(mut mat) = query.get_mut(quad.0) {
            mat.uniforms = build_single_material_uniform(&materials.0[selected.0]);
        }
    }
}
```

- [ ] **Step 3: Build and run the viewer**

Run: `cargo.exe build -p terrain_viewer`

Expected: compiles. Then run: `cargo.exe run -p terrain_viewer`

Expected: a window opens with a terrain quad showing the grass material. Press arrow keys or 1-6 to switch terrain types.

- [ ] **Step 4: Commit**

```bash
git add crates/terrain_viewer/
git commit -m "feat(terrain_viewer): add viewer binary with terrain quad and keyboard switching"
```

---

## Task 4: Add parameter adjustment and text overlay

**Files:**
- Modify: `crates/terrain_viewer/src/main.rs`

- [ ] **Step 1: Add parameter adjustment keys to `terrain_input_system`**

Add to the system, after the terrain type switching code:

```rust
    // Parameter adjustment — applies to the selected material
    let step = if input.pressed(KeyCode::ShiftLeft) { 0.5 } else { 0.1 };
    let mat_ref = &mut materials.0[selected.0]; // Need ResMut<TerrainMaterials>

    // ... but TerrainMaterials is Res, not ResMut.
```

Change the system signature to take `ResMut<TerrainMaterials>` instead of `Res<TerrainMaterials>`. Then add parameter adjustment:

```rust
fn terrain_input_system(
    input: Res<InputState>,
    mut selected: ResMut<SelectedTerrain>,
    mut materials: ResMut<TerrainMaterials>,
    quad: Res<TerrainQuadEntity>,
    mut query: Query<&mut Material2d>,
) {
    let count = materials.0.len();
    let mut changed = false;

    // --- Terrain type switching (same as before) ---
    if input.just_pressed(KeyCode::ArrowRight) || input.just_pressed(KeyCode::KeyD) {
        selected.0 = (selected.0 + 1) % count;
        changed = true;
    }
    if input.just_pressed(KeyCode::ArrowLeft) || input.just_pressed(KeyCode::KeyA) {
        selected.0 = (selected.0 + count - 1) % count;
        changed = true;
    }
    let key_map = [
        KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3,
        KeyCode::Digit4, KeyCode::Digit5, KeyCode::Digit6,
    ];
    for (i, key) in key_map.iter().enumerate() {
        if i < count && input.just_pressed(*key) {
            selected.0 = i;
            changed = true;
        }
    }

    // --- Parameter adjustment ---
    let step = if input.pressed(KeyCode::ShiftLeft) { 0.5 } else { 0.1 };
    let mat = &mut materials.0[selected.0];

    // Q/W: frequency
    if input.just_pressed(KeyCode::KeyQ) { mat.params[0] = (mat.params[0] - step).max(0.1); changed = true; }
    if input.just_pressed(KeyCode::KeyW) { mat.params[0] += step; changed = true; }
    // E/R: amplitude
    if input.just_pressed(KeyCode::KeyE) { mat.params[1] = (mat.params[1] - step * 0.5).max(0.0); changed = true; }
    if input.just_pressed(KeyCode::KeyR) { mat.params[1] += step * 0.5; changed = true; }
    // T/Y: warp strength
    if input.just_pressed(KeyCode::KeyT) { mat.params[2] = (mat.params[2] - step * 0.5).max(0.0); changed = true; }
    if input.just_pressed(KeyCode::KeyY) { mat.params[2] += step * 0.5; changed = true; }

    if changed {
        if let Ok(mut mat2d) = query.get_mut(quad.0) {
            mat2d.uniforms = build_single_material_uniform(&materials.0[selected.0]);
        }
    }
}
```

- [ ] **Step 2: Add HUD text entity showing current parameters**

Add a `HudTextEntity` resource and spawn a `Text` entity in `spawn_viewer_scene`:

```rust
#[derive(Resource, Debug)]
struct HudTextEntity(Entity);

// In spawn_viewer_scene, after spawning the quad:
let hud = world
    .spawn((
        engine_ui::widget::Text {
            content: format_hud(&materials.0[selected]),
            font_size: 16.0,
            color: Color::WHITE,
            max_width: None,
        },
        GlobalTransform2D(Affine2::from_translation(Vec2::new(-480.0, 350.0))),
        RenderLayer::UI,
        SortOrder(100),
    ))
    .id();
world.insert_resource(HudTextEntity(hud));
```

Add the format function:

```rust
fn format_hud(mat: &terrain::material::TerrainMaterial) -> String {
    format!(
        "{} | freq:{:.1} amp:{:.2} warp:{:.2} scale:{:.1}\n\
         color_a: ({:.2},{:.2},{:.2})  color_b: ({:.2},{:.2},{:.2})\n\
         [1-6] type  [Q/W] freq  [E/R] amp  [T/Y] warp  [Shift] fast",
        mat.kind.name(),
        mat.params[0], mat.params[1], mat.params[2], mat.params[3],
        mat.color_a[0], mat.color_a[1], mat.color_a[2],
        mat.color_b[0], mat.color_b[1], mat.color_b[2],
    )
}
```

- [ ] **Step 3: Add a HUD update system**

```rust
fn hud_update_system(
    selected: Res<SelectedTerrain>,
    materials: Res<TerrainMaterials>,
    hud: Res<HudTextEntity>,
    mut query: Query<&mut engine_ui::widget::Text>,
) {
    if let Ok(mut text) = query.get_mut(hud.0) {
        let new_content = format_hud(&materials.0[selected.0]);
        if text.content != new_content {
            text.content = new_content;
        }
    }
}
```

Register it in `setup`:

```rust
app.add_systems(Phase::Update, (terrain_input_system, hud_update_system).chain());
```

- [ ] **Step 4: Add camera zoom via mouse scroll**

Add to `setup`:

```rust
app.add_systems(Phase::Update, camera_zoom_system.after(terrain_input_system));
```

Add the system:

```rust
fn camera_zoom_system(mouse: Res<MouseState>, mut query: Query<&mut Camera2D>) {
    let scroll = mouse.scroll_delta().y;
    if scroll == 0.0 {
        return;
    }
    if let Ok(mut camera) = query.single_mut() {
        camera.zoom = (camera.zoom + 0.1 * scroll).max(0.1);
    }
}
```

- [ ] **Step 5: Build and test**

Run: `cargo.exe run -p terrain_viewer`

Expected: terrain quad visible. Arrow keys / 1-6 switch materials. Q/W/E/R/T/Y adjust parameters (hold Shift for larger steps). HUD text shows current values. Mouse wheel zooms.

- [ ] **Step 6: Commit**

```bash
git add crates/terrain_viewer/src/main.rs
git commit -m "feat(terrain_viewer): add parameter adjustment keys, HUD overlay, and zoom"
```

---

## Task 5: Implement dual-grid module

**Files:**
- Create: `crates/terrain/src/dual_grid.rs`
- Modify: `crates/terrain/src/lib.rs`
- Create: `crates/terrain/tests/suite/dual_grid.rs`
- Modify: `crates/terrain/tests/suite/mod.rs`

- [ ] **Step 1: Write tests for dual-grid bitmask computation**

`tests/suite/dual_grid.rs`:

```rust
use terrain::dual_grid::{DualGrid, VisualTile, corner_bitmask};
use terrain::material::TerrainId;

#[test]
fn when_all_corners_same_then_bitmask_is_zero() {
    // Arrange
    let id = TerrainId(0);

    // Act
    let mask = corner_bitmask([id, id, id, id], id);

    // Assert — all corners match the "primary" type, no transitions
    assert_eq!(mask, 0);
}

#[test]
fn when_all_corners_differ_from_primary_then_bitmask_is_15() {
    // Arrange
    let primary = TerrainId(0);
    let other = TerrainId(1);

    // Act
    let mask = corner_bitmask([other, other, other, other], primary);

    // Assert
    assert_eq!(mask, 15);
}

#[test]
fn when_ne_corner_differs_then_bitmask_bit_0_set() {
    let a = TerrainId(0);
    let b = TerrainId(1);

    let mask = corner_bitmask([b, a, a, a], a);

    assert_eq!(mask, 1); // NE=1
}

#[test]
fn when_se_and_sw_corners_differ_then_bitmask_is_6() {
    let a = TerrainId(0);
    let b = TerrainId(1);

    let mask = corner_bitmask([a, b, b, a], a);

    assert_eq!(mask, 6); // SE=2 + SW=4
}

#[test]
fn when_grid_2x2_then_produces_3x3_visual_tiles() {
    // A 2x2 data grid produces a 3x3 visual grid (straddle pattern)
    let mut grid = DualGrid::new(2, 2, TerrainId(0));
    grid.set(1, 0, TerrainId(1));
    grid.set(1, 1, TerrainId(1));

    let tiles = grid.visual_tiles();

    assert_eq!(tiles.len(), 9); // (2+1) * (2+1)
}

#[test]
fn when_visual_tile_straddles_edge_then_corners_include_border_default() {
    let grid = DualGrid::new(2, 2, TerrainId(0));
    let tiles = grid.visual_tiles();

    // Top-left visual tile at (-0.5, -0.5) straddles outside the grid.
    // Out-of-bounds corners should use the border default (same as the nearest cell).
    let tl = &tiles[0];
    // All corners should be TerrainId(0) since the whole grid is uniform
    assert_eq!(tl.corners, [TerrainId(0); 4]);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo.exe test -p terrain`

Expected: compilation errors — `dual_grid` module doesn't exist yet.

- [ ] **Step 3: Implement `src/dual_grid.rs`**

```rust
use crate::material::TerrainId;

/// A data grid where each cell holds a `TerrainId`.
#[derive(Clone, Debug)]
pub struct DualGrid {
    width: usize,
    height: usize,
    cells: Vec<TerrainId>,
}

/// A visual tile in the dual-grid, straddling 4 data cells.
#[derive(Clone, Debug, PartialEq)]
pub struct VisualTile {
    /// Grid-space position of the visual tile center.
    pub x: f32,
    pub y: f32,
    /// The 4 data-cell terrain IDs: [NE, SE, SW, NW].
    pub corners: [TerrainId; 4],
    /// Per-tile seed derived from position.
    pub seed: u32,
}

impl DualGrid {
    /// Create a grid filled with a uniform terrain type.
    #[must_use]
    pub fn new(width: usize, height: usize, fill: TerrainId) -> Self {
        Self {
            width,
            height,
            cells: vec![fill; width * height],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    /// Get the terrain type at `(x, y)`. Returns `None` for out-of-bounds.
    #[must_use]
    pub fn get(&self, x: usize, y: usize) -> Option<TerrainId> {
        if x < self.width && y < self.height {
            Some(self.cells[y * self.width + x])
        } else {
            None
        }
    }

    /// Set the terrain type at `(x, y)`. Panics if out of bounds.
    pub fn set(&mut self, x: usize, y: usize, id: TerrainId) {
        self.cells[y * self.width + x] = id;
    }

    /// Get terrain type, clamping out-of-bounds coordinates to the nearest edge cell.
    #[must_use]
    fn get_clamped(&self, x: i32, y: i32) -> TerrainId {
        let cx = x.clamp(0, self.width as i32 - 1) as usize;
        let cy = y.clamp(0, self.height as i32 - 1) as usize;
        self.cells[cy * self.width + cx]
    }

    /// Generate all visual tiles for the dual-grid.
    /// The visual grid is offset by (-0.5, -0.5) and has dimensions (width+1, height+1).
    #[must_use]
    pub fn visual_tiles(&self) -> Vec<VisualTile> {
        let vw = self.width + 1;
        let vh = self.height + 1;
        let mut tiles = Vec::with_capacity(vw * vh);

        for vy in 0..vh {
            for vx in 0..vw {
                // Data cells that this visual tile straddles:
                // NE = (vx, vy-1), SE = (vx, vy), SW = (vx-1, vy), NW = (vx-1, vy-1)
                let dx = vx as i32;
                let dy = vy as i32;
                let ne = self.get_clamped(dx, dy - 1);
                let se = self.get_clamped(dx, dy);
                let sw = self.get_clamped(dx - 1, dy);
                let nw = self.get_clamped(dx - 1, dy - 1);

                let seed = simple_hash(vx as u32, vy as u32);

                tiles.push(VisualTile {
                    x: vx as f32 - 0.5,
                    y: vy as f32 - 0.5,
                    corners: [ne, se, sw, nw],
                    seed,
                });
            }
        }

        tiles
    }
}

/// Compute the corner16 bitmask: which corners differ from `primary`.
/// NE=bit0, SE=bit1, SW=bit2, NW=bit3.
#[must_use]
pub fn corner_bitmask(corners: [TerrainId; 4], primary: TerrainId) -> u8 {
    let mut mask = 0u8;
    if corners[0] != primary { mask |= 1; }
    if corners[1] != primary { mask |= 2; }
    if corners[2] != primary { mask |= 4; }
    if corners[3] != primary { mask |= 8; }
    mask
}

fn simple_hash(x: u32, y: u32) -> u32 {
    let mut h = x.wrapping_mul(374_761_393).wrapping_add(y.wrapping_mul(668_265_263));
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^ (h >> 16)
}
```

- [ ] **Step 4: Add `pub mod dual_grid;` to `src/lib.rs` and register test module**

In `src/lib.rs`, add:
```rust
pub mod dual_grid;
```

In `tests/suite/mod.rs`, add:
```rust
mod dual_grid;
```

- [ ] **Step 5: Run tests**

Run: `cargo.exe test -p terrain`

Expected: all tests pass (material tests + dual_grid tests).

- [ ] **Step 6: Commit**

```bash
git add crates/terrain/src/dual_grid.rs crates/terrain/src/lib.rs crates/terrain/tests/
git commit -m "feat(terrain): add dual-grid coordinate mapping with corner bitmask"
```

---

## Task 6: Add auto-tile boundary SDF to shader and multi-tile rendering

**Files:**
- Modify: `crates/terrain/src/shader/terrain.wgsl`
- Modify: `crates/terrain_viewer/src/main.rs`

- [ ] **Step 1: Add boundary SDF functions to `terrain.wgsl`**

Add before the `fs_main` function:

```wgsl
// ============================================================
// Auto-tile boundary SDF (corner16 patterns)
// ============================================================

// Returns signed distance to the boundary between "primary" and "other"
// terrain types. Negative = primary side, positive = other side.
// bitmask bits: NE=1, SE=2, SW=4, NW=8.
fn autotile_sdf(uv: vec2<f32>, bitmask: u32) -> f32 {
    let x = uv.x;
    let y = uv.y;

    switch bitmask {
        // Uniform: all primary (no boundary)
        case 0u { return -1.0; }
        // Uniform: all other
        case 15u { return 1.0; }

        // Single corner
        case 1u { return corner_sdf(x, 1.0 - y); }         // NE
        case 2u { return corner_sdf(x, y); }                // SE
        case 4u { return corner_sdf(1.0 - x, y); }          // SW
        case 8u { return corner_sdf(1.0 - x, 1.0 - y); }   // NW

        // Edge (two adjacent corners)
        case 3u { return 0.5 - x; }             // NE+SE = right edge
        case 6u { return y - 0.5; }             // SE+SW = bottom edge
        case 12u { return x - 0.5; }            // SW+NW = left edge
        case 9u { return 0.5 - y; }             // NW+NE = top edge

        // Diagonal (two opposite corners)
        case 5u { return diagonal_sdf(x, y); }  // NE+SW
        case 10u { return diagonal_sdf(y, x); } // SE+NW

        // Three corners (L-shaped, inverted single corner)
        case 7u { return -corner_sdf(1.0 - x, 1.0 - y); }  // all except NW
        case 11u { return -corner_sdf(1.0 - x, y); }        // all except SW
        case 13u { return -corner_sdf(x, y); }              // all except SE
        case 14u { return -corner_sdf(x, 1.0 - y); }       // all except NE

        default { return 0.0; }
    }
}

fn corner_sdf(x: f32, y: f32) -> f32 {
    // Quarter-circle arc in one corner
    return length(vec2<f32>(x, y) - vec2<f32>(1.0, 1.0)) - 0.7;
}

fn diagonal_sdf(x: f32, y: f32) -> f32 {
    // Diagonal split
    return (x + y - 1.0) * 0.7071;
}

fn displace_boundary(sdf: f32, world_uv: vec2<f32>, strength: f32) -> f32 {
    let noise = gradient_noise(world_uv * 8.0) * 2.0 - 1.0;
    return sdf + noise * strength;
}
```

- [ ] **Step 2: Update `fs_main` to support multi-tile mode (Phase 2)**

Replace the existing `fs_main` with a version that handles both Phase 1 (single material) and Phase 2 (4-corner dual-grid):

```wgsl
fn unpack_corner_types() -> vec4<u32> {
    let packed = bitcast<u32>(material[0].x);
    return vec4<u32>(
        packed & 0xFFu,
        (packed >> 8u) & 0xFFu,
        (packed >> 16u) & 0xFFu,
        (packed >> 24u) & 0xFFu,
    );
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let corners = unpack_corner_types();
    let world_pos = unpack_world_pos();
    let seed = unpack_seed();

    let world_uv = world_pos + in.uv * 10.0 + vec2<f32>(seed * 0.17, seed * 0.31);

    // Check if all corners are the same type (uniform tile or Phase 1 single-material)
    if corners.x == corners.y && corners.y == corners.z && corners.z == corners.w {
        let params = unpack_params(1u);
        let color = eval_terrain(world_uv, corners.x, params);
        return vec4<f32>(color, 1.0);
    }

    // Dual-grid: determine primary type (most common corner) and compute bitmask
    let primary = corners.x; // simplified: use NE as primary
    var bitmask = 0u;
    if corners.x != primary { bitmask |= 1u; }
    if corners.y != primary { bitmask |= 2u; }
    if corners.z != primary { bitmask |= 4u; }
    if corners.w != primary { bitmask |= 8u; }

    let raw_sdf = autotile_sdf(in.uv, bitmask);
    let sdf = displace_boundary(raw_sdf, world_uv, 0.06);

    let params_primary = unpack_params(1u);

    if sdf < 0.0 {
        // Primary side
        let color = eval_terrain(world_uv, primary, params_primary);
        return vec4<f32>(color, 1.0);
    } else {
        // Other side — find the other type
        var other_type = corners.y;
        if corners.y == primary { other_type = corners.z; }
        if corners.z == primary { other_type = corners.w; }
        // Use params slot 5 for the secondary material
        let params_other = unpack_params(5u);
        let color = eval_terrain(world_uv, other_type, params_other);
        return vec4<f32>(color, 1.0);
    }
}
```

Note: this is a simplified two-type transition. Full 4-type support and per-pair transition effects come in Task 7. The old `unpack_type_id()` and single-material `fs_main` are replaced.

- [ ] **Step 3: Update `build_single_material_uniform` for the new packing**

In `terrain_viewer/src/main.rs`, update the uniform builder:

```rust
fn build_single_material_uniform(mat: &terrain::material::TerrainMaterial) -> Vec<u8> {
    let gpu = mat.to_gpu_params();
    let kind = mat.kind as u32;
    // Pack all 4 corners as the same type (uniform tile)
    let packed_corners = kind | (kind << 8) | (kind << 16) | (kind << 24);
    let header: [f32; 4] = [f32::from_bits(packed_corners), 0.0, 0.0, 42.0];

    let mut buf = Vec::with_capacity(256);
    buf.extend_from_slice(bytes_of(&header));
    buf.extend_from_slice(bytes_of(&gpu)); // slot 1..4 = primary material
    // Pad remaining slots
    buf.resize(256, 0);
    buf
}
```

- [ ] **Step 4: Add a grid rendering mode to the viewer**

Add a `ViewerMode` resource and a system to spawn a 4x4 dual-grid:

```rust
#[derive(Resource, Debug, PartialEq, Eq)]
enum ViewerMode {
    SingleMaterial,
    DualGrid,
}

// In setup, insert default mode:
world.insert_resource(ViewerMode::SingleMaterial);
```

Add a key (Tab) to toggle modes, and a function `spawn_dual_grid_tiles` that:
1. Despawns the single quad entity
2. Creates a `DualGrid` with 2-3 terrain types
3. Iterates `visual_tiles()` and spawns a quad entity per tile with proper uniform data

This is substantial code — the core logic packs each tile's 4-corner data into the uniform buffer using the same format the shader now reads.

```rust
fn build_tile_uniform(
    tile: &terrain::dual_grid::VisualTile,
    materials: &[terrain::material::TerrainMaterial],
) -> Vec<u8> {
    let kind_of = |id: terrain::material::TerrainId| -> u32 {
        materials.iter()
            .find(|m| m.id == id)
            .map_or(0, |m| m.kind as u32)
    };

    let c = tile.corners;
    let packed = kind_of(c[0])
        | (kind_of(c[1]) << 8)
        | (kind_of(c[2]) << 16)
        | (kind_of(c[3]) << 24);

    let header: [f32; 4] = [
        f32::from_bits(packed),
        tile.x,
        tile.y,
        (tile.seed % 1000) as f32,
    ];

    let gpu_primary = materials.iter()
        .find(|m| m.id == c[0])
        .map(terrain::material::TerrainMaterial::to_gpu_params)
        .unwrap_or_default();

    // Find the first corner that differs from c[0] for the secondary material
    let other_id = c.iter().find(|&&id| id != c[0]).copied().unwrap_or(c[0]);
    let gpu_secondary = materials.iter()
        .find(|m| m.id == other_id)
        .map(terrain::material::TerrainMaterial::to_gpu_params)
        .unwrap_or_default();

    let mut buf = Vec::with_capacity(256);
    buf.extend_from_slice(bytes_of(&header));
    buf.extend_from_slice(bytes_of(&gpu_primary));   // slots 1-4
    buf.extend_from_slice(bytes_of(&gpu_secondary)); // slots 5-8
    buf.resize(256, 0);
    buf
}
```

- [ ] **Step 5: Build and test visually**

Run: `cargo.exe run -p terrain_viewer`

Expected: starts in single-material mode. Press Tab to switch to dual-grid mode showing a 4x4 grid with transitions visible. Press Tab again to return to single material.

- [ ] **Step 6: Commit**

```bash
git add crates/terrain/src/shader/terrain.wgsl crates/terrain_viewer/src/main.rs
git commit -m "feat(terrain): add auto-tile boundary SDF and dual-grid rendering mode"
```

---

## Task 7: Add per-pair transition effects

**Files:**
- Modify: `crates/terrain/src/shader/terrain.wgsl`

- [ ] **Step 1: Add transition effect functions to `terrain.wgsl`**

Add after the sub-shader functions, before the master dispatch:

```wgsl
// ============================================================
// Per-pair transition effects
// ============================================================

fn transition_effect(
    base_color: vec3<f32>,
    world_uv: vec2<f32>,
    my_type: u32,
    neighbor_type: u32,
    dist_to_edge: f32,
) -> vec3<f32> {
    // Only apply effects within a narrow band near the boundary
    let band = 0.15;
    if dist_to_edge > band { return base_color; }
    let t = 1.0 - dist_to_edge / band; // 1.0 at edge, 0.0 at band limit

    // Stone -> Sand: scattered pebbles
    if my_type == 3u && neighbor_type == 1u {
        let pebble = step(0.85, hash21(floor(world_uv * 25.0)));
        return mix(base_color, base_color * 0.65, pebble * t);
    }

    // Lava -> Grass: singed/darkened
    if my_type == 0u && neighbor_type == 4u {
        return mix(base_color, base_color * vec3<f32>(0.4, 0.3, 0.2), t * 0.8);
    }

    // Water -> Sand: wet sand darkening
    if my_type == 3u && neighbor_type == 2u {
        return mix(base_color, base_color * 0.7, t * 0.6);
    }

    // Grass -> Stone: moss/soil strip
    if my_type == 1u && neighbor_type == 0u {
        let moss = gradient_noise(world_uv * 15.0) * t;
        return mix(base_color, vec3<f32>(0.25, 0.35, 0.18), moss * 0.4);
    }

    // Generic fallback: subtle darkening at boundary
    return mix(base_color, base_color * 0.85, t * 0.3);
}
```

- [ ] **Step 2: Integrate transition effects into `fs_main`**

Update the dual-grid branch of `fs_main` to call `transition_effect` when near the boundary:

```wgsl
    let abs_sdf = abs(sdf);

    if sdf < 0.0 {
        let color = eval_terrain(world_uv, primary, params_primary);
        let result = transition_effect(color, world_uv, primary, other_type, abs_sdf);
        return vec4<f32>(result, 1.0);
    } else {
        let params_other = unpack_params(5u);
        let color = eval_terrain(world_uv, other_type, params_other);
        let result = transition_effect(color, world_uv, other_type, primary, abs_sdf);
        return vec4<f32>(result, 1.0);
    }
```

- [ ] **Step 3: Build and test visually**

Run: `cargo.exe run -p terrain_viewer`

Switch to dual-grid mode (Tab). Verify that transitions between terrain types show per-pair effects (pebbles on sand near stone, darkened grass near lava, etc.).

- [ ] **Step 4: Commit**

```bash
git add crates/terrain/src/shader/terrain.wgsl
git commit -m "feat(terrain): add per-pair transition effects near auto-tile boundaries"
```

---

## Task 8: Implement WFC solver

**Files:**
- Create: `crates/terrain/src/wfc.rs`
- Modify: `crates/terrain/src/lib.rs`
- Create: `crates/terrain/tests/suite/wfc.rs`
- Modify: `crates/terrain/tests/suite/mod.rs`

- [ ] **Step 1: Write WFC tests**

`tests/suite/wfc.rs`:

```rust
use rand_chacha::ChaCha8Rng;
use rand::SeedableRng;
use terrain::material::TerrainId;
use terrain::wfc::{ConstraintTable, Grid, WfcError, collapse};

fn two_type_constraints() -> ConstraintTable {
    let a = TerrainId(0);
    let b = TerrainId(1);
    let mut table = ConstraintTable::new(vec![a, b]);
    // Both types can be adjacent to themselves and each other
    table.allow(a, b);
    table.allow(b, a);
    table.allow(a, a);
    table.allow(b, b);
    table
}

#[test]
fn when_collapsing_small_grid_then_all_cells_filled() {
    // Arrange
    let mut grid = Grid::new(4, 4);
    let constraints = two_type_constraints();
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    // Act
    let result = collapse(&mut grid, &constraints, &mut rng);

    // Assert
    assert!(result.is_ok());
    for y in 0..4 {
        for x in 0..4 {
            assert!(grid.get(x, y).is_some(), "cell ({x},{y}) was not collapsed");
        }
    }
}

#[test]
fn when_cell_pinned_then_pinned_value_preserved() {
    let mut grid = Grid::new(4, 4);
    let pinned = TerrainId(1);
    grid.set(2, 2, Some(pinned));

    let constraints = two_type_constraints();
    let mut rng = ChaCha8Rng::seed_from_u64(99);

    let result = collapse(&mut grid, &constraints, &mut rng);

    assert!(result.is_ok());
    assert_eq!(grid.get(2, 2), Some(pinned));
}

#[test]
fn when_same_seed_then_deterministic_output() {
    let constraints = two_type_constraints();

    let mut grid1 = Grid::new(6, 6);
    let mut rng1 = ChaCha8Rng::seed_from_u64(123);
    collapse(&mut grid1, &constraints, &mut rng1).unwrap();

    let mut grid2 = Grid::new(6, 6);
    let mut rng2 = ChaCha8Rng::seed_from_u64(123);
    collapse(&mut grid2, &constraints, &mut rng2).unwrap();

    for y in 0..6 {
        for x in 0..6 {
            assert_eq!(grid1.get(x, y), grid2.get(x, y),
                "mismatch at ({x},{y})");
        }
    }
}

#[test]
fn when_impossible_constraints_then_returns_error() {
    let a = TerrainId(0);
    let b = TerrainId(1);
    let mut table = ConstraintTable::new(vec![a, b]);
    // a cannot be adjacent to b, but grid is small enough that both must appear
    table.allow(a, a);
    table.allow(b, b);
    // no cross-type adjacency allowed

    let mut grid = Grid::new(2, 2);
    grid.set(0, 0, Some(a));
    grid.set(1, 1, Some(b));
    let mut rng = ChaCha8Rng::seed_from_u64(1);

    let result = collapse(&mut grid, &table, &mut rng);

    assert!(result.is_err());
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo.exe test -p terrain`

Expected: compilation errors — `wfc` module doesn't exist yet.

- [ ] **Step 3: Implement `src/wfc.rs`**

```rust
use std::collections::BTreeSet;

use rand::Rng;
use rand_chacha::ChaCha8Rng;

use crate::material::TerrainId;

#[derive(Debug)]
pub enum WfcError {
    Contradiction,
}

/// Which terrain types may be placed next to each other.
#[derive(Clone, Debug)]
pub struct ConstraintTable {
    types: Vec<TerrainId>,
    /// Set of allowed `(from, to)` adjacency pairs.
    allowed: BTreeSet<(TerrainId, TerrainId)>,
}

impl ConstraintTable {
    #[must_use]
    pub fn new(types: Vec<TerrainId>) -> Self {
        Self {
            types,
            allowed: BTreeSet::new(),
        }
    }

    pub fn allow(&mut self, from: TerrainId, to: TerrainId) {
        self.allowed.insert((from, to));
    }

    fn is_allowed(&self, from: TerrainId, to: TerrainId) -> bool {
        self.allowed.contains(&(from, to))
    }

    fn types(&self) -> &[TerrainId] {
        &self.types
    }
}

/// A 2D grid of optionally-collapsed cells.
#[derive(Clone, Debug)]
pub struct Grid {
    width: usize,
    height: usize,
    cells: Vec<Option<TerrainId>>,
}

impl Grid {
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![None; width * height],
        }
    }

    pub fn width(&self) -> usize { self.width }
    pub fn height(&self) -> usize { self.height }

    #[must_use]
    pub fn get(&self, x: usize, y: usize) -> Option<TerrainId> {
        self.cells[y * self.width + x]
    }

    pub fn set(&mut self, x: usize, y: usize, value: Option<TerrainId>) {
        self.cells[y * self.width + x] = value;
    }

    fn neighbors(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let mut result = Vec::with_capacity(4);
        if x > 0 { result.push((x - 1, y)); }
        if x + 1 < self.width { result.push((x + 1, y)); }
        if y > 0 { result.push((x, y - 1)); }
        if y + 1 < self.height { result.push((x, y + 1)); }
        result
    }
}

/// Collapse the grid using WFC with constraint propagation and backtracking.
pub fn collapse(
    grid: &mut Grid,
    constraints: &ConstraintTable,
    rng: &mut ChaCha8Rng,
) -> Result<(), WfcError> {
    let w = grid.width;
    let h = grid.height;

    // Build possibility sets for each cell
    let all_types: BTreeSet<TerrainId> = constraints.types().iter().copied().collect();
    let mut possible: Vec<BTreeSet<TerrainId>> = vec![all_types.clone(); w * h];

    // Constrain from pinned cells
    for y in 0..h {
        for x in 0..w {
            if let Some(id) = grid.get(x, y) {
                possible[y * w + x] = BTreeSet::from([id]);
                propagate(grid, &mut possible, constraints, x, y);
            }
        }
    }

    // Main loop: pick lowest-entropy uncollapsed cell, collapse it
    let max_backtracks = w * h * 4;
    let mut backtrack_count = 0;
    let mut history: Vec<(usize, usize, BTreeSet<TerrainId>, Vec<BTreeSet<TerrainId>>)> = Vec::new();

    loop {
        // Find uncollapsed cell with fewest possibilities
        let mut best: Option<(usize, usize, usize)> = None;
        for y in 0..h {
            for x in 0..w {
                if grid.get(x, y).is_some() { continue; }
                let count = possible[y * w + x].len();
                if count == 0 {
                    // Contradiction — need backtracking
                    if let Some((bx, by, remaining, snapshot)) = history.pop() {
                        backtrack_count += 1;
                        if backtrack_count > max_backtracks {
                            return Err(WfcError::Contradiction);
                        }
                        // Restore state
                        grid.set(bx, by, None);
                        possible = snapshot;
                        possible[by * w + bx] = remaining;
                        continue;
                    }
                    return Err(WfcError::Contradiction);
                }
                if best.is_none() || count < best.expect("checked").2 {
                    best = Some((x, y, count));
                }
            }
        }

        let Some((cx, cy, _)) = best else {
            break; // All cells collapsed
        };

        let options: Vec<TerrainId> = possible[cy * w + cx].iter().copied().collect();
        let chosen_idx = rng.random_range(0..options.len());
        let chosen = options[chosen_idx];

        // Save state for backtracking
        let mut remaining = possible[cy * w + cx].clone();
        remaining.remove(&chosen);
        history.push((cx, cy, remaining, possible.clone()));

        // Collapse this cell
        grid.set(cx, cy, Some(chosen));
        possible[cy * w + cx] = BTreeSet::from([chosen]);
        propagate(grid, &mut possible, constraints, cx, cy);
    }

    Ok(())
}

fn propagate(
    grid: &Grid,
    possible: &mut [BTreeSet<TerrainId>],
    constraints: &ConstraintTable,
    start_x: usize,
    start_y: usize,
) {
    let w = grid.width;
    let mut queue = vec![(start_x, start_y)];

    while let Some((x, y)) = queue.pop() {
        let current_set = possible[y * w + x].clone();
        for (nx, ny) in grid.neighbors(x, y) {
            if grid.get(nx, ny).is_some() { continue; }
            let idx = ny * w + nx;
            let before = possible[idx].len();
            possible[idx].retain(|&candidate| {
                current_set.iter().any(|&src| constraints.is_allowed(src, candidate))
            });
            if possible[idx].len() < before {
                queue.push((nx, ny));
            }
        }
    }
}
```

- [ ] **Step 4: Add `pub mod wfc;` to `src/lib.rs` and register test module**

In `src/lib.rs`:
```rust
pub mod wfc;
```

In `tests/suite/mod.rs`:
```rust
mod wfc;
```

- [ ] **Step 5: Run tests**

Run: `cargo.exe test -p terrain`

Expected: all tests pass (material + dual_grid + wfc).

- [ ] **Step 6: Commit**

```bash
git add crates/terrain/src/wfc.rs crates/terrain/src/lib.rs crates/terrain/tests/
git commit -m "feat(terrain): add WFC solver with backtracking and cell pinning"
```

---

## Task 9: Connect WFC to viewer with generate/re-roll and camera pan

**Files:**
- Modify: `crates/terrain_viewer/src/main.rs`

- [ ] **Step 1: Add a `WfcGrid` viewer mode**

Add a third variant to `ViewerMode`:

```rust
#[derive(Resource, Debug, PartialEq, Eq)]
enum ViewerMode {
    SingleMaterial,
    DualGrid,
    WfcGrid,
}
```

- [ ] **Step 2: Add WFC generation resource and system**

```rust
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use terrain::wfc::{ConstraintTable, Grid as WfcGrid, collapse};
use terrain::dual_grid::DualGrid;

#[derive(Resource, Debug)]
struct WfcState {
    seed: u64,
    data_grid: DualGrid,
}

fn default_constraints(materials: &[terrain::material::TerrainMaterial]) -> ConstraintTable {
    let types: Vec<terrain::material::TerrainId> = materials.iter().map(|m| m.id).collect();
    let mut table = ConstraintTable::new(types.clone());
    // Allow all pairs for now — can be refined later
    for &a in &types {
        for &b in &types {
            table.allow(a, b);
        }
    }
    table
}

fn generate_wfc_grid(
    materials: &[terrain::material::TerrainMaterial],
    width: usize,
    height: usize,
    seed: u64,
) -> DualGrid {
    let constraints = default_constraints(materials);
    let mut wfc_grid = WfcGrid::new(width, height);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    match collapse(&mut wfc_grid, &constraints, &mut rng) {
        Ok(()) => {
            let mut dual = DualGrid::new(width, height, terrain::material::TerrainId(0));
            for y in 0..height {
                for x in 0..width {
                    if let Some(id) = wfc_grid.get(x, y) {
                        dual.set(x, y, id);
                    }
                }
            }
            dual
        }
        Err(_) => {
            // Fallback: uniform grid
            DualGrid::new(width, height, terrain::material::TerrainId(0))
        }
    }
}
```

- [ ] **Step 3: Add key bindings for WFC mode**

In the input system, add:
- **G**: generate new grid (current seed)
- **N**: re-roll (increment seed + generate)
- **Tab**: cycle SingleMaterial → DualGrid → WfcGrid

- [ ] **Step 4: Add camera pan system**

```rust
#[derive(Resource, Debug, Default)]
struct CameraDragState {
    anchor: Option<Vec2>,
}

fn camera_drag_system(
    mouse: Res<MouseState>,
    mut drag_state: ResMut<CameraDragState>,
    mut query: Query<&mut Camera2D>,
) {
    if mouse.just_released(MouseButton::Right) {
        drag_state.anchor = None;
        return;
    }
    if mouse.just_pressed(MouseButton::Right) {
        drag_state.anchor = Some(mouse.screen_pos());
        return;
    }
    if mouse.pressed(MouseButton::Right) {
        if let Some(anchor) = drag_state.anchor {
            let delta = mouse.screen_pos() - anchor;
            if let Ok(mut camera) = query.single_mut() {
                camera.position -= delta / camera.zoom;
            }
            drag_state.anchor = Some(mouse.screen_pos());
        }
    }
}
```

Register in `setup`:

```rust
world.insert_resource(CameraDragState::default());
app.add_systems(Phase::Update, (terrain_input_system, hud_update_system, camera_drag_system, camera_zoom_system).chain());
```

- [ ] **Step 5: Add grid tile spawning**

When entering WFC mode, the system:
1. Generates a grid via `generate_wfc_grid`
2. For each `VisualTile`, spawns a quad entity with the proper uniform data
3. Stores the entity list in a resource for cleanup when switching modes

This reuses `build_tile_uniform` from Task 6 for each visual tile, with quad size matching the cell spacing.

- [ ] **Step 6: Build and test visually**

Run: `cargo.exe run -p terrain_viewer`

Expected: Tab cycles through all three modes. In WFC mode, G generates a map, N re-rolls with a new seed. Right-click drag pans the camera. Mouse wheel zooms.

- [ ] **Step 7: Commit**

```bash
git add crates/terrain_viewer/src/main.rs
git commit -m "feat(terrain_viewer): add WFC grid mode with generate, re-roll, and camera pan"
```

---

## Task 10: Update HUD for all modes and add terrain toggles

**Files:**
- Modify: `crates/terrain_viewer/src/main.rs`

- [ ] **Step 1: Update HUD to show mode-specific information**

```rust
fn format_hud(
    mode: &ViewerMode,
    mat: &terrain::material::TerrainMaterial,
    seed: Option<u64>,
) -> String {
    let mode_str = match mode {
        ViewerMode::SingleMaterial => "SINGLE MATERIAL",
        ViewerMode::DualGrid => "DUAL GRID (4x4)",
        ViewerMode::WfcGrid => "WFC GRID",
    };

    let mut s = format!(
        "{mode_str} | {}\n\
         freq:{:.1} amp:{:.2} warp:{:.2} scale:{:.1}\n",
        mat.kind.name(),
        mat.params[0], mat.params[1], mat.params[2], mat.params[3],
    );

    if let Some(seed) = seed {
        s.push_str(&format!("seed: {seed}\n"));
    }

    s.push_str("[Tab] mode  [1-6] type  [Q/W] freq  [E/R] amp  [T/Y] warp  [Shift] fast");
    if matches!(mode, ViewerMode::WfcGrid) {
        s.push_str("\n[G] generate  [N] re-roll  [RMB] pan  [Scroll] zoom");
    }

    s
}
```

- [ ] **Step 2: Build and test**

Run: `cargo.exe run -p terrain_viewer`

Expected: HUD text updates based on current mode and shows appropriate controls.

- [ ] **Step 3: Commit**

```bash
git add crates/terrain_viewer/src/main.rs
git commit -m "feat(terrain_viewer): update HUD for all viewer modes"
```

---

## Task 11: Final cleanup and format

**Files:**
- All modified files

- [ ] **Step 1: Run clippy**

Run: `cargo.exe clippy -p terrain -p terrain_viewer`

Fix any warnings.

- [ ] **Step 2: Run formatter**

Run: `cargo.exe fmt --all`

- [ ] **Step 3: Run all tests**

Run: `cargo.exe test -p terrain`

Expected: all tests pass.

- [ ] **Step 4: Run the viewer one final time**

Run: `cargo.exe run -p terrain_viewer`

Verify all three modes work: single material, dual-grid transitions, WFC grid generation.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "chore(terrain): clippy fixes and formatting"
```
