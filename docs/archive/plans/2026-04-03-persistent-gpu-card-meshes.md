# Persistent GPU Card Meshes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Eliminate per-frame card mesh copying by uploading card geometry to the GPU once at spawn, and remove the redundant `baked_card_sync_system` that clones mesh data every frame.

**Architecture:** Each card uploads its front and back `TessellatedColorMesh` to persistent GPU vertex/index buffers at spawn time, storing handles in a new `GpuCardMesh` component. `unified_render_system` issues draw calls referencing these persistent buffers instead of copying vertices into the shared batch. `baked_card_sync_system` and `ColorMesh` are removed from cards entirely; the render system reads `BakedCardMesh + Card.face_up` directly and applies `front_only` overlay visibility inline.

**Tech Stack:** Rust, bevy_ecs, wgpu (via `WgpuRenderer`), `engine_render` crate, `card_game` crate.

---

## File Map

| Action | File | Change |
|--------|------|--------|
| Modify | `crates/engine_render/src/renderer.rs` | Add `GpuMeshHandle`, three new `Renderer` trait methods, `NullRenderer` impls |
| Modify | `crates/engine_render/src/wgpu_renderer/renderer.rs` | Add `MeshSource` enum, `PersistentMesh` struct, fields on `WgpuRenderer`, change `ShapeDrawRecord.index_offset` → `source` |
| Modify | `crates/engine_render/src/wgpu_renderer/renderer_trait.rs` | Implement new trait methods; refactor `draw_shape`/`draw_colored_mesh` to use `index_start+count`; rewrite `draw_shape_batches` + `issue_shape_draw_calls` to handle mixed batched/persistent |
| Modify | `crates/engine_render/src/testing/mod.rs` | Add `PersistentMeshCallLog`, `persistent_mesh_calls` field + capture builder + `SpyRenderer` impl of new trait methods |
| Create | `crates/card_game/src/card/rendering/gpu_card_mesh.rs` | `GpuCardMesh` component (`front: GpuMeshHandle`, `back: GpuMeshHandle`) |
| Modify | `crates/card_game/src/card/rendering/mod.rs` | `pub mod gpu_card_mesh;` |
| Modify | `crates/card_game/src/card/rendering/spawn_table_card.rs` | Upload front+back at spawn, insert `GpuCardMesh` |
| Modify | `crates/engine_ui/src/unified_render.rs` | Add `BakedCardItem` query, `DrawKind::BakedCard`, handle in collect+draw |
| Delete | `crates/card_game/src/card/rendering/baked_render.rs` | Remove `baked_card_sync_system` entirely |
| Modify | `crates/card_game/src/card/rendering/mod.rs` | Remove `pub mod baked_render;` |
| Modify | `crates/card_game/src/plugin.rs` | Remove `baked_card_sync_system` import + registration |
| Modify | `crates/card_game/src/prelude.rs` | Remove re-export of `baked_render` if any |
| Delete | `crates/card_game/tests/suite/card_rendering_baked_render.rs` | Tests for deleted system |
| Modify | `crates/card_game/tests/suite/mod.rs` | Remove `card_rendering_baked_render` module |
| Create | `crates/card_game/tests/suite/card_rendering_gpu_card_mesh.rs` | Behavioral tests for GPU card draw path |
| Modify | `crates/card_game/tests/suite/mod.rs` | Add `card_rendering_gpu_card_mesh` module |
| Modify | `crates/engine_core/src/scale_spring.rs` | Add `Option<ResMut<FrameProfiler>>` param + `span("scale_spring")` |
| Modify | `crates/engine_scene/src/sort_propagation.rs` | Add `Option<ResMut<FrameProfiler>>` param + `span("hierarchy_sort")` |
| Modify | `crates/card_game/src/stash/layout.rs` | Add `Option<ResMut<FrameProfiler>>` param + `span("stash_layout")` |
| Modify | `crates/card_game/src/hand/layout.rs` | Add `Option<ResMut<FrameProfiler>>` param + `span("hand_layout")` |

---

## Task 1: Add `GpuMeshHandle` and persistent mesh API to `engine_render`

**Files:**
- Modify: `crates/engine_render/src/renderer.rs`

- [ ] **Step 1: Add `GpuMeshHandle` and new trait methods**

In `crates/engine_render/src/renderer.rs`, add after the `use` block and before `RenderError`:

```rust
/// Opaque handle to a persistent GPU mesh buffer.
/// Created by [`Renderer::upload_persistent_colored_mesh`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GpuMeshHandle(pub u32);
```

Add three new methods to the `Renderer` trait (after `draw_colored_mesh`):

```rust
/// Upload a colored mesh to a persistent GPU buffer. Returns a handle valid for the
/// lifetime of the renderer. The mesh data is NOT copied per-frame.
fn upload_persistent_colored_mesh(
    &mut self,
    vertices: &[ColorVertex],
    indices: &[u32],
) -> GpuMeshHandle;

/// Record a draw call that reads from a previously uploaded persistent mesh.
/// No vertex copy occurs; the GPU reads from the persistent buffer directly.
fn draw_persistent_colored_mesh(
    &mut self,
    handle: GpuMeshHandle,
    model: [[f32; 4]; 4],
);

/// Release a persistent GPU mesh buffer. Safe to call with an already-freed or
/// invalid handle (no-op). Does not affect in-flight draw calls.
fn free_persistent_colored_mesh(&mut self, handle: GpuMeshHandle);
```

Add no-op impls to `NullRenderer`:

```rust
fn upload_persistent_colored_mesh(
    &mut self,
    _vertices: &[ColorVertex],
    _indices: &[u32],
) -> GpuMeshHandle {
    GpuMeshHandle(0)
}
fn draw_persistent_colored_mesh(&mut self, _handle: GpuMeshHandle, _model: [[f32; 4]; 4]) {}
fn free_persistent_colored_mesh(&mut self, _handle: GpuMeshHandle) {}
```

- [ ] **Step 2: Add `GpuMeshHandle` to `engine_render` prelude**

In `crates/engine_render/src/prelude.rs`, add:
```rust
pub use crate::renderer::GpuMeshHandle;
```

- [ ] **Step 3: Build to confirm trait impls compile**

```bash
cargo.exe build -p engine_render 2>&1 | tail -5
```
Expected: error about missing impls in `WgpuRenderer`, `SpyRenderer`, `VisualRegressionRenderer` — we'll fix those in later tasks. For now just check `NullRenderer` doesn't error.

---

## Task 2: Add `SpyRenderer` support for persistent mesh calls

**Files:**
- Modify: `crates/engine_render/src/testing/mod.rs`

- [ ] **Step 1: Add capture type and field**

After the existing type aliases at the top of `testing/mod.rs`, add:
```rust
pub type PersistentMeshCallLog = Arc<Mutex<Vec<(crate::renderer::GpuMeshHandle, [[f32; 4]; 4])>>>;
```

Add to the `SpyRenderer` struct after `colored_mesh_calls`:
```rust
persistent_mesh_calls: Option<PersistentMeshCallLog>,
next_persistent_id: u32,
```

Set both to defaults in `SpyRenderer::new`:
```rust
persistent_mesh_calls: None,
next_persistent_id: 1,  // start at 1 so 0 is distinguishable as "invalid"
```

- [ ] **Step 2: Add builder method**

After `with_colored_mesh_capture`, add:
```rust
pub fn with_persistent_mesh_capture(
    mut self,
    persistent_mesh_calls: PersistentMeshCallLog,
) -> Self {
    self.persistent_mesh_calls = Some(persistent_mesh_calls);
    self
}
```

- [ ] **Step 3: Implement trait methods**

In the `impl Renderer for SpyRenderer` block, add after `draw_colored_mesh`:

```rust
fn upload_persistent_colored_mesh(
    &mut self,
    _vertices: &[crate::shape::ColorVertex],
    _indices: &[u32],
) -> crate::renderer::GpuMeshHandle {
    let handle = crate::renderer::GpuMeshHandle(self.next_persistent_id);
    self.next_persistent_id += 1;
    handle
}

fn draw_persistent_colored_mesh(
    &mut self,
    handle: crate::renderer::GpuMeshHandle,
    model: [[f32; 4]; 4],
) {
    self.log_call("draw_persistent_colored_mesh");
    if let Some(capture) = &self.persistent_mesh_calls {
        capture
            .lock()
            .expect("persistent mesh capture poisoned")
            .push((handle, model));
    }
}

fn free_persistent_colored_mesh(&mut self, _handle: crate::renderer::GpuMeshHandle) {}
```

- [ ] **Step 4: Build `engine_render` (testing still has visual_regression to fix)**

```bash
cargo.exe build -p engine_render 2>&1 | grep "^error" | head -10
```

Expected: errors about `visual_regression.rs` missing the new methods. Fix in next step.

- [ ] **Step 5: Add stubs to `VisualRegressionRenderer`**

In `crates/engine_render/src/testing/visual_regression.rs`, add after `draw_colored_mesh`:

```rust
fn upload_persistent_colored_mesh(
    &mut self,
    _vertices: &[crate::shape::ColorVertex],
    _indices: &[u32],
) -> crate::renderer::GpuMeshHandle {
    crate::renderer::GpuMeshHandle(0)
}

fn draw_persistent_colored_mesh(
    &mut self,
    _handle: crate::renderer::GpuMeshHandle,
    _model: [[f32; 4]; 4],
) {
}

fn free_persistent_colored_mesh(&mut self, _handle: crate::renderer::GpuMeshHandle) {}
```

- [ ] **Step 6: Build `engine_render` cleanly**

```bash
cargo.exe build -p engine_render 2>&1 | tail -5
```
Expected: `Finished` or only warnings. No errors.

- [ ] **Step 7: Commit**

```bash
git add crates/engine_render/src/renderer.rs crates/engine_render/src/prelude.rs crates/engine_render/src/testing/mod.rs crates/engine_render/src/testing/visual_regression.rs
git commit -m "feat(engine-render): add GpuMeshHandle and persistent colored mesh API"
```

---

## Task 3: Implement persistent mesh storage in `WgpuRenderer`

**Files:**
- Modify: `crates/engine_render/src/wgpu_renderer/renderer.rs`
- Modify: `crates/engine_render/src/wgpu_renderer/renderer_trait.rs`

- [ ] **Step 1: Add `MeshSource` enum and `PersistentMesh` struct to `renderer.rs`**

In `crates/engine_render/src/wgpu_renderer/renderer.rs`, add after the `use` imports:

```rust
use crate::renderer::GpuMeshHandle;

pub(super) struct PersistentMesh {
    pub(super) vertex_buffer: wgpu::Buffer,
    pub(super) index_buffer: wgpu::Buffer,
    pub(super) index_count: u32,
}

pub(super) enum MeshSource {
    Batched { index_start: u32, index_count: u32 },
    Persistent { handle: GpuMeshHandle },
}
```

- [ ] **Step 2: Replace `index_offset` with `source` in `ShapeDrawRecord`**

Change:
```rust
pub(super) struct ShapeDrawRecord {
    pub(super) blend_mode: crate::material::BlendMode,
    pub(super) shader_handle: ShaderHandle,
    pub(super) index_offset: u32,
    pub(super) model: [[f32; 4]; 4],
    pub(super) material_uniforms: Vec<u8>,
    #[allow(dead_code)]
    pub(super) material_textures: Vec<(TextureId, u32)>,
}
```

To:
```rust
pub(super) struct ShapeDrawRecord {
    pub(super) blend_mode: crate::material::BlendMode,
    pub(super) shader_handle: ShaderHandle,
    pub(super) source: MeshSource,
    pub(super) model: [[f32; 4]; 4],
    pub(super) material_uniforms: Vec<u8>,
    #[allow(dead_code)]
    pub(super) material_textures: Vec<(TextureId, u32)>,
}
```

- [ ] **Step 3: Add persistent mesh fields to `WgpuRenderer`**

Add two fields to `WgpuRenderer` struct after `shape_draws`:
```rust
pub(super) persistent_meshes: std::collections::HashMap<GpuMeshHandle, PersistentMesh>,
pub(super) next_persistent_id: u32,
```

Add to `from_parts` initializer:
```rust
persistent_meshes: std::collections::HashMap::new(),
next_persistent_id: 1,
```

- [ ] **Step 4: Fix `draw_shape` and `draw_colored_mesh` in `renderer_trait.rs`**

These methods currently push to `shape_draws` BEFORE pushing vertices. Change them to push vertices first so we can compute `index_count`.

Change `draw_shape` (around line 340 in `renderer_trait.rs`) from:
```rust
self.shape_draws.push(ShapeDrawRecord {
    blend_mode: self.current_blend_mode,
    shader_handle: self.active_shader,
    index_offset: self.shape_batch.index_count() as u32,
    model,
    material_uniforms,
    material_textures,
});
self.shape_batch.push(vertices, indices, color);
```

To:
```rust
#[allow(clippy::cast_possible_truncation)]
let index_start = self.shape_batch.index_count() as u32;
self.shape_batch.push(vertices, indices, color);
#[allow(clippy::cast_possible_truncation)]
let index_count = (self.shape_batch.index_count() as u32) - index_start;
self.shape_draws.push(ShapeDrawRecord {
    blend_mode: self.current_blend_mode,
    shader_handle: self.active_shader,
    source: MeshSource::Batched { index_start, index_count },
    model,
    material_uniforms,
    material_textures,
});
```

Apply the same change to `draw_colored_mesh` (around line 367):
```rust
#[allow(clippy::cast_possible_truncation)]
let index_start = self.shape_batch.index_count() as u32;
let shape_verts: &[ShapeVertex] = bytemuck::cast_slice(vertices);
self.shape_batch.push_colored(shape_verts, indices);
#[allow(clippy::cast_possible_truncation)]
let index_count = (self.shape_batch.index_count() as u32) - index_start;
self.shape_draws.push(ShapeDrawRecord {
    blend_mode: self.current_blend_mode,
    shader_handle: self.active_shader,
    source: MeshSource::Batched { index_start, index_count },
    model,
    material_uniforms,
    material_textures,
});
```

- [ ] **Step 5: Implement `upload_persistent_colored_mesh` and `draw_persistent_colored_mesh`**

In `renderer_trait.rs`, add the `Renderer` trait impl methods for `WgpuRenderer`:

```rust
fn upload_persistent_colored_mesh(
    &mut self,
    vertices: &[crate::shape::ColorVertex],
    indices: &[u32],
) -> GpuMeshHandle {
    use wgpu::util::DeviceExt;
    let handle = GpuMeshHandle(self.next_persistent_id);
    self.next_persistent_id += 1;
    let shape_verts: &[ShapeVertex] = bytemuck::cast_slice(vertices);
    let vertex_buffer =
        self.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(shape_verts),
                usage: wgpu::BufferUsages::VERTEX,
            });
    let index_buffer =
        self.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX,
            });
    #[allow(clippy::cast_possible_truncation)]
    self.persistent_meshes.insert(
        handle,
        PersistentMesh {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
        },
    );
    handle
}

fn draw_persistent_colored_mesh(
    &mut self,
    handle: GpuMeshHandle,
    model: [[f32; 4]; 4],
) {
    let material_uniforms = self.pending_material.take_uniforms();
    let material_textures = self.pending_material.take_textures();
    self.shape_draws.push(ShapeDrawRecord {
        blend_mode: self.current_blend_mode,
        shader_handle: self.active_shader,
        source: MeshSource::Persistent { handle },
        model,
        material_uniforms,
        material_textures,
    });
}

fn free_persistent_colored_mesh(&mut self, handle: GpuMeshHandle) {
    self.persistent_meshes.remove(&handle);
}
```

- [ ] **Step 6: Rewrite `issue_shape_draw_calls` to handle mixed sources**

Replace `issue_shape_draw_calls` in `renderer_trait.rs`:

```rust
#[allow(clippy::cast_possible_truncation)]
fn issue_shape_draw_calls(
    &self,
    pass: &mut wgpu::RenderPass,
    model_bg: &wgpu::BindGroup,
    material_bgs: &[wgpu::BindGroup],
    aligned_entry: usize,
    batched_buffers: Option<&(wgpu::Buffer, wgpu::Buffer)>,
) {
    let mut last_key: Option<(ShaderHandle, crate::material::BlendMode)> = None;
    let mut batched_bound = false;

    for (i, draw) in self.shape_draws.iter().enumerate() {
        let key = (draw.shader_handle, draw.blend_mode);
        if last_key != Some(key) {
            pass.set_pipeline(self.select_shape_pipeline(key));
            last_key = Some(key);
        }
        let dyn_offset = (i * aligned_entry) as u32;
        pass.set_bind_group(1, model_bg, &[dyn_offset]);
        pass.set_bind_group(2, &material_bgs[i], &[]);

        match &draw.source {
            MeshSource::Batched {
                index_start,
                index_count,
            } => {
                if !batched_bound {
                    if let Some((vb, ib)) = batched_buffers {
                        pass.set_vertex_buffer(0, vb.slice(..));
                        pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
                    }
                    batched_bound = true;
                }
                pass.draw_indexed(*index_start..*index_start + *index_count, 0, 0..1);
            }
            MeshSource::Persistent { handle } => {
                if let Some(mesh) = self.persistent_meshes.get(handle) {
                    pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                    pass.set_index_buffer(
                        mesh.index_buffer.slice(..),
                        wgpu::IndexFormat::Uint32,
                    );
                    batched_bound = false;
                    pass.draw_indexed(0..mesh.index_count, 0, 0..1);
                }
            }
        }
    }
}
```

- [ ] **Step 7: Update `draw_shape_batches` to pass buffers and fix empty-batch guard**

Replace `draw_shape_batches`:

```rust
fn draw_shape_batches(&self, pass: &mut wgpu::RenderPass) {
    let aligned_entry = (self.model_uniform_align as usize).max(64);
    let model_bg = self.create_model_bind_group(aligned_entry);
    let material_bgs = self.create_material_bind_groups();
    pass.set_bind_group(0, &self.camera_bind_group, &[]);
    pass.set_bind_group(3, &self.texture_bind_group, &[]);
    let batched_buffers = if !self.shape_batch.is_empty() {
        Some(self.create_shape_buffers())
    } else {
        None
    };
    if let Some((vb, ib)) = &batched_buffers {
        pass.set_vertex_buffer(0, vb.slice(..));
        pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
    }
    self.issue_shape_draw_calls(
        pass,
        &model_bg,
        &material_bgs,
        aligned_entry,
        batched_buffers.as_ref(),
    );
}
```

Also update the guard in `draw_scene_to` (change the condition from checking `shape_batch` to `shape_draws`):

```rust
// was:  if !self.shape_batch.is_empty() {
if !self.shape_draws.is_empty() {
    self.draw_shape_batches(&mut pass);
}
```

- [ ] **Step 8: Build `engine_render` cleanly**

```bash
cargo.exe build -p engine_render 2>&1 | tail -5
```
Expected: `Finished`. No errors.

- [ ] **Step 9: Run engine_render tests**

```bash
cargo.exe test -p engine_render 2>&1 | tail -10
```
Expected: all tests pass.

- [ ] **Step 10: Commit**

```bash
git add crates/engine_render/src/wgpu_renderer/renderer.rs crates/engine_render/src/wgpu_renderer/renderer_trait.rs
git commit -m "feat(engine-render): implement persistent GPU mesh buffers in WgpuRenderer"
```

---

## Task 4: Add `GpuCardMesh` component and upload at card spawn

**Files:**
- Create: `crates/card_game/src/card/rendering/gpu_card_mesh.rs`
- Modify: `crates/card_game/src/card/rendering/mod.rs`
- Modify: `crates/card_game/src/card/rendering/spawn_table_card.rs`

- [ ] **Step 1: Write the failing test**

Create `crates/card_game/tests/suite/card_rendering_gpu_card_mesh.rs`:

```rust
#![allow(clippy::unwrap_used)]

use std::sync::{Arc, Mutex};

use bevy_ecs::prelude::*;
use card_game::card::rendering::gpu_card_mesh::GpuCardMesh;
use card_game::card::rendering::spawn_table_card::spawn_visual_card;
use card_game::card::identity::definition::{CardAbilities, CardDefinition, CardType, art_descriptor_default};
use card_game::card::identity::signature::CardSignature;
use engine_render::renderer::GpuMeshHandle;
use engine_render::testing::{PersistentMeshCallLog, SpyRenderer};
use engine_render::renderer::RendererRes;
use glam::Vec2;

fn placeholder_def() -> CardDefinition {
    CardDefinition {
        card_type: CardType::Creature,
        name: String::new(),
        stats: None,
        abilities: CardAbilities { keywords: vec![], text: String::new() },
        art: art_descriptor_default(CardType::Creature),
    }
}

#[test]
fn when_card_spawned_then_gpu_card_mesh_uploaded() {
    // Arrange
    let log = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(log);
    let mut world = World::new();
    world.insert_resource(RendererRes::new(Box::new(spy)));

    // Act
    let entity = spawn_visual_card(
        &mut world,
        &placeholder_def(),
        Vec2::ZERO,
        Vec2::new(60.0, 90.0),
        true,
        CardSignature::default(),
    );

    // Assert
    let gpu_mesh = world.get::<GpuCardMesh>(entity).unwrap();
    assert_ne!(gpu_mesh.front, GpuMeshHandle(0), "front handle must be non-zero");
    assert_ne!(gpu_mesh.back, GpuMeshHandle(0), "back handle must be non-zero");
    assert_ne!(gpu_mesh.front, gpu_mesh.back, "front and back must be distinct");
}
```

Add to `crates/card_game/tests/suite/mod.rs`:
```rust
pub mod card_rendering_gpu_card_mesh;
```

- [ ] **Step 2: Run the test to confirm it fails**

```bash
cargo.exe test -p card_game when_card_spawned_then_gpu_card_mesh_uploaded 2>&1 | tail -10
```
Expected: compile error — `GpuCardMesh` does not exist.

- [ ] **Step 3: Create `gpu_card_mesh.rs`**

Create `crates/card_game/src/card/rendering/gpu_card_mesh.rs`:

```rust
use bevy_ecs::prelude::Component;
use engine_render::renderer::GpuMeshHandle;

/// Persistent GPU vertex/index buffers for a card's front and back faces.
/// Uploaded once at spawn via [`Renderer::upload_persistent_colored_mesh`].
/// The render system reads the correct handle based on [`Card::face_up`].
#[derive(Component, Debug, Clone, Copy)]
pub struct GpuCardMesh {
    pub front: GpuMeshHandle,
    pub back: GpuMeshHandle,
}
```

Add to `crates/card_game/src/card/rendering/mod.rs`:
```rust
pub mod gpu_card_mesh;
```

- [ ] **Step 4: Upload meshes in `spawn_visual_card`**

In `crates/card_game/src/card/rendering/spawn_table_card.rs`, add import:
```rust
use crate::card::rendering::gpu_card_mesh::GpuCardMesh;
use engine_render::prelude::RendererRes;
```

After the `baked` mesh is constructed and before inserting into the entity, add:

```rust
let gpu_mesh = world
    .get_resource_mut::<RendererRes>()
    .map(|mut renderer| GpuCardMesh {
        front: renderer.upload_persistent_colored_mesh(
            &baked.front.vertices,
            &baked.front.indices,
        ),
        back: renderer.upload_persistent_colored_mesh(
            &baked.back.vertices,
            &baked.back.indices,
        ),
    });
```

Then in the `world.entity_mut(root).insert(...)` call, include `gpu_mesh` if available. The insert currently does:
```rust
world
    .entity_mut(root)
    .insert((baked, mesh_overlays, ColorMesh(initial_mesh)));
```

Change to:
```rust
let mut entity_mut = world.entity_mut(root);
entity_mut.insert((baked, mesh_overlays));
if let Some(gpu) = gpu_mesh {
    entity_mut.insert(gpu);
}
```

> **Note:** `ColorMesh` is intentionally removed from the insert here. Cards no longer use `ColorMesh` for rendering — the render system reads from `GpuCardMesh` directly. The `ColorMesh` component on cards is dead weight after this change.

- [ ] **Step 5: Run the test to confirm it passes**

```bash
cargo.exe test -p card_game when_card_spawned_then_gpu_card_mesh_uploaded 2>&1 | tail -10
```
Expected: `test ... ok`.

- [ ] **Step 6: Run the full card_game test suite**

```bash
cargo.exe test -p card_game 2>&1 | tail -15
```
Expected: all tests pass. (Some tests may reference `ColorMesh` on spawned cards — they will fail until Task 6 removes the old sync system.)

- [ ] **Step 7: Commit**

```bash
git add crates/card_game/src/card/rendering/gpu_card_mesh.rs crates/card_game/src/card/rendering/mod.rs crates/card_game/src/card/rendering/spawn_table_card.rs crates/card_game/tests/suite/card_rendering_gpu_card_mesh.rs crates/card_game/tests/suite/mod.rs
git commit -m "feat(card-game): upload card meshes to GPU at spawn via GpuCardMesh"
```

---

## Task 5: Add BakedCard draw path to `unified_render_system`

**Files:**
- Modify: `crates/engine_ui/src/unified_render.rs`

- [ ] **Step 1: Add a second test for face selection**

Add to `crates/card_game/tests/suite/card_rendering_gpu_card_mesh.rs`:

```rust
use card_game::card::component::Card;
use card_game::card::component::CardItemForm;
use card_game::card::rendering::baked_mesh::BakedCardMesh;
use card_game::card::rendering::bake::{bake_back_face, bake_front_face};
use card_game::card::component::CardLabel;
use engine_core::prelude::{TextureId, Transform2D};
use engine_render::testing::PersistentMeshCallLog;
use engine_scene::prelude::{GlobalTransform2D, RenderLayer, SortOrder};
use engine_ui::unified_render::unified_render_system;
use engine_render::camera::Camera2D;

fn run_unified_render(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(unified_render_system);
    schedule.run(world);
}

fn spawn_card_for_render(
    world: &mut World,
    face_up: bool,
    front_handle: GpuMeshHandle,
    back_handle: GpuMeshHandle,
) -> Entity {
    let sig = CardSignature::default();
    world
        .spawn((
            Card {
                face_texture: TextureId(0),
                back_texture: TextureId(0),
                face_up,
                signature: sig,
            },
            GpuCardMesh {
                front: front_handle,
                back: back_handle,
            },
            Transform2D::default(),
            GlobalTransform2D(glam::Affine2::IDENTITY),
            RenderLayer::World,
            SortOrder::default(),
        ))
        .id()
}

#[test]
fn when_card_face_up_then_draws_front_handle() {
    // Arrange
    let calls: PersistentMeshCallLog = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(Arc::new(Mutex::new(Vec::new())))
        .with_persistent_mesh_capture(Arc::clone(&calls));
    let mut world = World::new();
    world.insert_resource(RendererRes::new(Box::new(spy)));
    world.spawn(Camera2D { position: glam::Vec2::ZERO, zoom: 1.0 });
    let front_handle = GpuMeshHandle(1);
    let back_handle = GpuMeshHandle(2);
    spawn_card_for_render(&mut world, true, front_handle, back_handle);

    // Act
    run_unified_render(&mut world);

    // Assert
    let recorded = calls.lock().unwrap();
    assert!(
        recorded.iter().any(|(h, _)| *h == front_handle),
        "face-up card must draw front handle"
    );
    assert!(
        !recorded.iter().any(|(h, _)| *h == back_handle),
        "face-up card must not draw back handle"
    );
}

#[test]
fn when_card_face_down_then_draws_back_handle() {
    // Arrange
    let calls: PersistentMeshCallLog = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(Arc::new(Mutex::new(Vec::new())))
        .with_persistent_mesh_capture(Arc::clone(&calls));
    let mut world = World::new();
    world.insert_resource(RendererRes::new(Box::new(spy)));
    world.spawn(Camera2D { position: glam::Vec2::ZERO, zoom: 1.0 });
    let front_handle = GpuMeshHandle(1);
    let back_handle = GpuMeshHandle(2);
    spawn_card_for_render(&mut world, false, front_handle, back_handle);

    // Act
    run_unified_render(&mut world);

    // Assert
    let recorded = calls.lock().unwrap();
    assert!(
        recorded.iter().any(|(h, _)| *h == back_handle),
        "face-down card must draw back handle"
    );
    assert!(
        !recorded.iter().any(|(h, _)| *h == front_handle),
        "face-down card must not draw front handle"
    );
}

#[test]
fn when_card_has_item_form_then_not_drawn() {
    // Arrange
    let calls: PersistentMeshCallLog = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(Arc::new(Mutex::new(Vec::new())))
        .with_persistent_mesh_capture(Arc::clone(&calls));
    let mut world = World::new();
    world.insert_resource(RendererRes::new(Box::new(spy)));
    world.spawn(Camera2D { position: glam::Vec2::ZERO, zoom: 1.0 });
    let sig = CardSignature::default();
    world.spawn((
        Card { face_texture: TextureId(0), back_texture: TextureId(0), face_up: true, signature: sig },
        GpuCardMesh { front: GpuMeshHandle(1), back: GpuMeshHandle(2) },
        CardItemForm,
        Transform2D::default(),
        GlobalTransform2D(glam::Affine2::IDENTITY),
        RenderLayer::World,
        SortOrder::default(),
    ));

    // Act
    run_unified_render(&mut world);

    // Assert — ItemForm cards must not be drawn as full-size cards in world space
    let recorded = calls.lock().unwrap();
    assert!(recorded.is_empty(), "item-form card must not emit a persistent draw call");
}
```

- [ ] **Step 2: Run tests to confirm they fail**

```bash
cargo.exe test -p card_game when_card_face_up_then_draws_front_handle 2>&1 | tail -5
```
Expected: compile error — the system doesn't handle `GpuCardMesh` yet.

- [ ] **Step 3: Add `BakedCardItem` query and `DrawKind::BakedCard` to `unified_render.rs`**

In `crates/engine_ui/src/unified_render.rs`, add imports:

```rust
use card_game::card::component::{Card, CardItemForm};
use card_game::card::rendering::gpu_card_mesh::GpuCardMesh;
```

Add the query type after `ColorMeshItem`:

```rust
type BakedCardItem<'w> = (
    Entity,
    &'w Card,
    &'w GpuCardMesh,
    &'w GlobalTransform2D,
    Option<&'w RenderLayer>,
    Option<&'w SortOrder>,
    Option<&'w EffectiveVisibility>,
    Option<&'w MeshOverlays>,
    Option<&'w CardItemForm>,
);
```

Add variant to `DrawKind`:
```rust
BakedCard,
```

- [ ] **Step 4: Collect BakedCard items in `collect_draw_items`**

Add `baked_card_query: &Query<BakedCardItem>` parameter to `collect_draw_items`, and add the collection loop before `items.sort_by_key`:

```rust
for (entity, _, _, layer, sort, vis, _, _, item_form) in baked_card_query.iter() {
    if item_form.is_some() {
        continue;
    }
    if vis.is_some_and(|v| !v.0) {
        continue;
    }
    items.push(SortedDrawItem {
        entity,
        sort_key: (
            layer.copied().unwrap_or(RenderLayer::World),
            sort.copied().unwrap_or_default(),
        ),
        kind: DrawKind::BakedCard,
    });
}
```

- [ ] **Step 5: Add `baked_card_query` parameter to `unified_render_system` and wire it up**

Add `baked_card_query: Query<BakedCardItem>` to `unified_render_system` parameters.

Pass it to `collect_draw_items`.

- [ ] **Step 6: Handle `DrawKind::BakedCard` in the draw loop**

In the `for item in &items` loop, add the arm:

```rust
DrawKind::BakedCard => {
    let Ok((_, card, gpu_mesh, transform, _, _, _, overlays, _)) =
        baked_card_query.get(item.entity)
    else {
        continue;
    };
    apply_material(&mut **renderer, None, &mut last_shader, &mut last_blend_mode);
    let model = affine2_to_mat4(&transform.0);
    let handle = if card.face_up { gpu_mesh.front } else { gpu_mesh.back };
    renderer.draw_persistent_colored_mesh(handle, model);

    // Draw overlays (small, batched — front_only checked inline)
    if let Some(overlays) = overlays {
        for entry in &overlays.0 {
            if !entry.front_only || card.face_up {
                apply_material(
                    &mut **renderer,
                    Some(&entry.material),
                    &mut last_shader,
                    &mut last_blend_mode,
                );
                renderer.draw_colored_mesh(&entry.mesh.vertices, &entry.mesh.indices, model);
            }
        }
    }
}
```

- [ ] **Step 7: Run the three new tests**

```bash
cargo.exe test -p card_game when_card_face_up_then_draws_front_handle when_card_face_down_then_draws_back_handle when_card_has_item_form_then_not_drawn 2>&1 | tail -10
```
Expected: all three pass.

- [ ] **Step 8: Commit**

```bash
git add crates/engine_ui/src/unified_render.rs crates/card_game/tests/suite/card_rendering_gpu_card_mesh.rs
git commit -m "feat(engine-ui): add BakedCard draw path using persistent GPU mesh handles"
```

---

## Task 6: Delete `baked_card_sync_system` and its tests

**Files:**
- Delete: `crates/card_game/src/card/rendering/baked_render.rs`
- Modify: `crates/card_game/src/card/rendering/mod.rs`
- Modify: `crates/card_game/src/plugin.rs`
- Delete: `crates/card_game/tests/suite/card_rendering_baked_render.rs`
- Modify: `crates/card_game/tests/suite/mod.rs`

- [ ] **Step 1: Remove `baked_card_sync_system` from plugin registration**

In `crates/card_game/src/plugin.rs`:

Remove from imports:
```rust
use crate::card::rendering::baked_render::baked_card_sync_system;
```

Remove from `register_systems`:
```rust
.add_systems(
    Phase::PostUpdate,
    (
        baked_card_sync_system,
        shader_pointer_system,
        // ...
    ),
)
```
→ Remove `baked_card_sync_system,` from the tuple.

- [ ] **Step 2: Remove `baked_render` module**

In `crates/card_game/src/card/rendering/mod.rs`, remove:
```rust
pub mod baked_render;
```

Delete the file:
```bash
rm crates/card_game/src/card/rendering/baked_render.rs
```

- [ ] **Step 3: Remove baked_render tests**

In `crates/card_game/tests/suite/mod.rs`, remove:
```rust
pub mod card_rendering_baked_render;
```

Delete the file:
```bash
rm crates/card_game/tests/suite/card_rendering_baked_render.rs
```

- [ ] **Step 4: Build and test**

```bash
cargo.exe test -p card_game 2>&1 | tail -15
```
Expected: all tests pass. The deleted tests are gone; the new `card_rendering_gpu_card_mesh` tests pass.

- [ ] **Step 5: Build the binary**

```bash
cargo.exe build -p card_game_bin 2>&1 | tail -5
```
Expected: `Finished`. No errors.

- [ ] **Step 6: Commit**

```bash
git add -u
git commit -m "refactor(card-game): remove baked_card_sync_system — GPU card path renders directly from BakedCardMesh"
```

---

## Task 7: Add PostUpdate sub-spans to remaining systems

**Files:**
- Modify: `crates/engine_core/src/scale_spring.rs`
- Modify: `crates/engine_scene/src/sort_propagation.rs`
- Modify: `crates/card_game/src/stash/layout.rs`
- Modify: `crates/card_game/src/hand/layout.rs`

Each system gets `Option<ResMut<FrameProfiler>>` as a parameter. The profiler's `span()` drop-guard records the elapsed time for the system's entire body.

- [ ] **Step 1: Add span to `scale_spring_system`**

In `crates/engine_core/src/scale_spring.rs`, add import:
```rust
use engine_core::profiler::FrameProfiler;
```
(It's in the same crate, so: `use crate::profiler::FrameProfiler;`)

Add `mut profiler: Option<ResMut<FrameProfiler>>` to system params and wrap body:

```rust
pub fn scale_spring_system(
    dt: Res<DeltaTime>,
    mut query: Query<(Entity, &mut Transform2D, &mut ScaleSpring)>,
    mut profiler: Option<ResMut<FrameProfiler>>,
) {
    let _span = profiler.as_deref_mut().map(|p| p.span("scale_spring"));
    // ... existing body unchanged ...
}
```

- [ ] **Step 2: Add span to `hierarchy_sort_system`**

In `crates/engine_scene/src/sort_propagation.rs`, add import:
```rust
use engine_core::profiler::FrameProfiler;
```

Add param and span:
```rust
pub fn hierarchy_sort_system(
    roots: Query<...>,
    children_query: Query<&Children>,
    local_sort_query: Query<Option<&LocalSortOrder>>,
    mut sort_query: Query<&mut SortOrder>,
    mut profiler: Option<ResMut<FrameProfiler>>,
) {
    let _span = profiler.as_deref_mut().map(|p| p.span("hierarchy_sort"));
    // ... existing body unchanged ...
}
```

- [ ] **Step 3: Add span to `stash_layout_system`**

In `crates/card_game/src/stash/layout.rs`, add import and param. The existing signature uses a `SystemParam` struct or individual params — add `mut profiler: Option<ResMut<FrameProfiler>>` and wrap:

```rust
use engine_core::profiler::FrameProfiler;

pub fn stash_layout_system(
    // ... existing params ...
    mut profiler: Option<ResMut<FrameProfiler>>,
) {
    let _span = profiler.as_deref_mut().map(|p| p.span("stash_layout"));
    // ... existing body unchanged ...
}
```

- [ ] **Step 4: Add span to `hand_layout_system`**

Same pattern in `crates/card_game/src/hand/layout.rs`:

```rust
use engine_core::profiler::FrameProfiler;

pub fn hand_layout_system(
    // ... existing params ...
    mut profiler: Option<ResMut<FrameProfiler>>,
) {
    let _span = profiler.as_deref_mut().map(|p| p.span("hand_layout"));
    // ... existing body unchanged ...
}
```

- [ ] **Step 5: Build everything**

```bash
cargo.exe build -p card_game_bin 2>&1 | tail -5
```
Expected: `Finished`.

- [ ] **Step 6: Run all tests**

```bash
cargo.exe test 2>&1 | tail -10
```
Expected: all tests pass.

- [ ] **Step 7: Format**

```bash
cargo.exe fmt --all
```

- [ ] **Step 8: Commit**

```bash
git add crates/engine_core/src/scale_spring.rs crates/engine_scene/src/sort_propagation.rs crates/card_game/src/stash/layout.rs crates/card_game/src/hand/layout.rs
git commit -m "perf(profiling): add PostUpdate sub-spans for scale_spring, hierarchy_sort, stash_layout, hand_layout"
```

---

## Self-Review

**Spec coverage:**
- ✅ Sub-spans replacing parent: Tasks 7 adds `scale_spring`, `hierarchy_sort`, `stash_layout`, `hand_layout` spans (the PostUpdate parent span remains in `app.rs` as the total — consistent with how `Render` coexists with `render_sort`/`render_draw`)
- ✅ Eliminate baked_card_sync_system: Tasks 4–6
- ✅ Persistent GPU buffers: Tasks 1–5
- ✅ CardItemForm visibility maintained: Task 5, Step 6 (render skips ItemForm cards; overlay front_only checked inline)
- ✅ `front_only` overlay logic preserved: Task 5, Step 6 (`!entry.front_only || card.face_up`)
- ✅ NullRenderer + SpyRenderer + VisualRegressionRenderer all implement new trait methods: Tasks 1–2

**Potential gaps:**
- `free_persistent_colored_mesh` is wired in `WgpuRenderer` but no despawn detection system calls it — YAGNI, cards are never despawned in current gameplay. If despawn is added later, add a `RemovedComponents<GpuCardMesh>` system.
- `ColorMesh` is no longer inserted at spawn (Task 4, Step 4) — any code that queries `ColorMesh` on cards will now get no results. Verify no stash/hand render systems query `ColorMesh` on card entities directly. (The stash renders cards as small icons via `StashGrid` positions, not via `ColorMesh`.)
- The `visible` field on `OverlayEntry` is no longer set by any system after `baked_card_sync_system` is removed. The render system uses `front_only + card.face_up` directly. The `visible` field is now unused dead state on overlay entries. This is fine for now (it can be removed in a later cleanup).
