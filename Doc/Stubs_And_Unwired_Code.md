# Stubs & Unwired Code Audit

Cataloged: 2026-03-11

Production code only. Excludes placeholder crates (engine_audio, engine_physics, engine_assets, engine_ui), NullRenderer (intentionally no-op), and SpyRenderer (test helper).

## Fixed (previously stubs/dead code)

### 1. `WgpuRenderer::set_blend_mode()` — FIXED

- **Was**: Empty no-op. All draw calls used the same pipeline regardless of BlendMode.
- **Fix applied**: WgpuRenderer now stores 3 quad pipelines + 3 shape pipelines (one per BlendMode: Alpha, Additive, Multiply). `set_blend_mode()` records the current mode; `draw_rect()`/`draw_sprite()`/`draw_shape()` track blend mode per item. `draw_scene_to()` uses `compute_batch_ranges()` to partition draws and switch pipelines at blend mode boundaries. Pure function `blend_mode_to_blend_state()` maps BlendMode to the correct `wgpu::BlendState`. Also added `BlendMode::ALL` and `BlendMode::index()`.

### 2. `WgpuRenderer::upload_atlas()` — FIXED

- **Was**: Method body existed but was `#[allow(dead_code)]` — nothing called it.
- **Fix applied**: `upload_atlas()` moved to the `Renderer` trait (implemented on NullRenderer, SpyRenderer, WgpuRenderer). New `upload_atlas_system` in atlas.rs gates on `Option<Res<TextureAtlas>>` + `AtlasUploaded` marker for one-shot upload. Registered in DefaultPlugins Render chain before sprite_render_system.

## Unwired Types

### 3. `ShaderRegistry` / `ShaderHandle` — defined, exported, unused by GPU

- **File**: `crates/engine_render/src/material.rs`
- **Impact**: `ShaderRegistry` stores shader source strings keyed by `ShaderHandle`. `Material2d` carries a `shader: ShaderHandle` field. Neither `WgpuRenderer` nor any system reads these — custom shaders cannot be used at runtime.
- **Fix**: Integrate `ShaderRegistry` as a `Resource`, look up the shader source during pipeline creation or caching, and compile variant pipelines from the registered WGSL source.

### 4. `Material2d.uniforms` / `Material2d.textures` — stored but never read

- **File**: `crates/engine_render/src/material.rs`
- **Impact**: `Material2d` stores `uniforms: Vec<u8>` and `textures: Vec<TextureBinding>`, but the render systems only read `blend_mode`. Custom uniform data and additional texture bindings are silently ignored.
- **Fix**: Upload `uniforms` to a per-material GPU buffer (or a dynamic uniform buffer with offsets). Bind `textures` entries to the appropriate bind group slots during draw calls.

## Not Stubs (Verified Functional)

The following were checked and confirmed to have real implementations:

- `WgpuRenderer::set_blend_mode()` — records blend mode, selects correct pipeline per BlendMode in draw_scene_to()
- `WgpuRenderer::upload_atlas()` — trait method called by upload_atlas_system on first TextureAtlas insertion
- `WgpuRenderer::apply_post_process()` — sets `post_process_pending` flag, triggers bloom pipeline in `present()`
- `WgpuRenderer::set_view_projection()` — writes matrix to GPU camera uniform buffer
- `WgpuRenderer::draw_shape()` — pushes to `ShapeBatch`, rendered via dedicated shape pipeline
- `WgpuRenderer::draw_sprite()` / `draw_rect()` — push `Instance` data, rendered via instanced quad pipeline
- All engine_scene systems (hierarchy, transform propagation, visibility)
- All engine_input systems (input_system, action map lookups)
- All engine_core systems (time_system, FixedTimestep)
