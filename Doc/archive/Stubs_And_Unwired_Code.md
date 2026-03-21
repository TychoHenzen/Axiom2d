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

### 3. `ShaderRegistry` / `ShaderHandle` — FIXED

- **Was**: `ShaderRegistry` stored shader source strings keyed by `ShaderHandle`, but neither `WgpuRenderer` nor any system read them — custom shaders could not be used at runtime.
- **Fix applied**: `ShaderRegistry` derives `Resource` for ECS access. `ShaderHandle` derives `PartialOrd`/`Ord` for sort-key use. New `effective_shader_handle()` helper (mirrors `effective_blend_mode()`). Renderer trait gained `set_shader(ShaderHandle)`. Render systems call `set_shader()` with deduplication (skip redundant calls for consecutive entities with same shader). Sort key expanded to `(RenderLayer, ShaderHandle, BlendMode, SortOrder)` — shader is the outer grouping key since shader switches are more expensive than blend state switches on GPU. WgpuRenderer has stub impl (pipeline cache pending GPU-side work).

### 4. `Material2d.uniforms` / `Material2d.textures` — FIXED

- **Was**: `Material2d` stored `uniforms: Vec<u8>` and `textures: Vec<TextureBinding>`, but render systems only read `blend_mode`. Custom uniform data and texture bindings were silently ignored.
- **Fix applied**: Renderer trait gained `set_material_uniforms(&[u8])` and `bind_material_texture(TextureId, u32)`. Render systems (sprite + shape) call `set_material_uniforms()` when uniforms is non-empty, and `bind_material_texture()` for each entry in textures Vec (preserving declaration order). Extracted shared `apply_material()` function to deduplicate the material-forwarding logic between both render systems. WgpuRenderer has stub impls (GPU buffer upload pending GPU-side work).

## Remaining GPU-Side Work

### 5. `WgpuRenderer` custom shader pipeline cache — NOT STARTED

- **File**: `crates/engine_render/src/wgpu_renderer.rs`
- **Impact**: `WgpuRenderer::set_shader()`, `set_material_uniforms()`, and `bind_material_texture()` are empty stubs. The ECS systems now forward all Material2d data to the Renderer trait, but the GPU backend ignores it — all entities still render with the built-in quad/shape shaders.
- **Fix**: Build a `HashMap<(ShaderHandle, BlendMode), RenderPipeline>` cache. On `set_shader()`, look up the registered WGSL source via `ShaderRegistry`, compile if not cached, and select the pipeline. Upload uniform bytes to a dynamic GPU buffer. Bind extra textures to the appropriate bind group slots. Requires GPU device — test via headless wgpu / visual regression.

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
