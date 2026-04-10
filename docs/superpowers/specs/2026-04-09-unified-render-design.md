# Unified Render System Redesign

## Problem

The `unified_render_system` exists to enforce a single draw order determined by `(RenderLayer, SortOrder)` — the scene hierarchy decides what draws on top, not system registration order. But the current implementation has gaps that cause new render systems to be created, breaking draw order:

- Sprites have their own `sprite_render_system` — can't interleave with shapes/text
- Adding a new draw type requires surgery in 5 places (DrawKind, query type, system params, collection loop, match dispatch)
- 6+ immediate-mode systems bypass unified sort entirely
- No mechanism for procedural/immediate-mode draws to participate in unified sort

## Solution: Approach 2 (DrawCommand buffer + sprites + phase separation)

### DrawCommand enum

All render data is captured upfront during collection into owned `DrawCommand` values. The draw phase is pure dispatch — no ECS queries, no tessellation.

```rust
pub enum DrawCommand {
    Shape {
        mesh: TessellatedMesh,
        color: Color,
        model: [[f32; 4]; 4],
        material: Option<Material2d>,
        stroke: Option<StrokeCommand>,
    },
    Text {
        content: String,
        font_size: f32,
        color: Color,
        max_width: Option<f32>,
        transform: glam::Affine2,
    },
    ColorMesh {
        mesh: TessellatedColorMesh,
        model: [[f32; 4]; 4],
        material: Option<Material2d>,
        overlays: Vec<OverlayCommand>,
    },
    PersistentMesh {
        handle: GpuMeshHandle,
        model: [[f32; 4]; 4],
        material: Option<Material2d>,
        overlays: Vec<OverlayCommand>,
    },
    Sprite {
        rect: Rect,
        uv_rect: [f32; 4],
        material: Option<Material2d>,
    },
}

pub struct SortedDrawCommand {
    pub sort_key: (RenderLayer, SortOrder),
    pub command: DrawCommand,
}

struct StrokeCommand {
    mesh: TessellatedMesh,
    color: Color,
}

struct OverlayCommand {
    mesh: TessellatedColorMesh,
    material: Material2d,
}
```

### DrawQueue resource

Shared command buffer for immediate-mode systems. Systems push commands with explicit `(RenderLayer, SortOrder)`. `unified_render_system` drains the queue each frame and merges with entity-sourced commands before sorting.

```rust
#[derive(Resource, Default)]
pub struct DrawQueue {
    commands: Vec<SortedDrawCommand>,
}

impl DrawQueue {
    pub fn push(&mut self, layer: RenderLayer, order: SortOrder, command: DrawCommand) { ... }
    fn drain(&mut self) -> Vec<SortedDrawCommand> { std::mem::take(&mut self.commands) }
}
```

### Phase separation

`unified_render_system` moves from `Phase::Render` to `Phase::PostRender`.

```
Phase::Render       <- all prep: clear_system, entity component setup, DrawQueue pushes
Phase::PostRender   <- unified_render_system ONLY: drain queue, collect entities, sort, draw
                    -> renderer.present()
```

No `.before()` / `.after()` constraints needed. Phase boundaries enforce the order. Any system in `Phase::Render` that pushes to DrawQueue or sets up entity components is guaranteed to complete before unified_render runs.

### sprite_render_system and shape_render_system are deleted

There is no standalone mode. Unified render is **the** render system for every project — `demo`, `card_game_bin`, any future binary. `ShapeRenderDisabled` is deleted. If an entity has a `Shape` or `Sprite` component, unified_render draws it.

### Collection phase

Single function collects from 5 entity queries + drains DrawQueue:
- Pre-allocates Vec with total entity count + queue size
- Tessellation happens during collection (Shape without CachedMesh, Stroke)
- `sort_unstable_by_key` on `(RenderLayer, SortOrder)` — no need for stable sort
- Entity visibility filtering (`EffectiveVisibility(false)`) during collection

### Draw phase

Pure dispatch loop over sorted commands:
- Material state tracking (`last_shader`, `last_blend_mode`) flows across all draw types
- Frustum culling for shapes (mesh AABB) and sprites (rect bounds)
- Overlay drawing extracted into shared helper for ColorMesh and PersistentMesh
- No ECS queries, no tessellation, no branching on entity presence

### Clone costs

| Variant | Cost | Reason |
|---------|------|--------|
| PersistentMesh | Trivial | u32 handle + matrix |
| Sprite | Trivial | All Copy types |
| Text | Cheap | String clone + scalars |
| Shape | Moderate | Tessellated vertices (typically small: circles, rects, polygons) |
| ColorMesh | Moderate | Rarely used in entity path; cards use PersistentMesh |
| Material2d | Cheap | Shader handle + blend mode + small vecs |

## Testing

**Test:**
- Unified sort order across all 5 draw types
- DrawQueue commands interleave correctly with entity-sourced commands
- Visibility filtering produces no DrawCommands
- Material state dedup tracks across draw type boundaries
- Frustum culling for shapes and sprites
- DrawQueue drains each frame

**Don't test:**
- sort_unstable_by_key correctness (stdlib)
- Exact Renderer trait call arguments (spy implementation details)

## Future: Approach 3 (documented intent)

The end goal is that **no system except `unified_render_system` and `clear_system` touches the Renderer trait**. DrawQueue becomes the only way to issue immediate-mode draws.

### Systems to migrate (priority order)

These systems currently call `renderer.draw_*` directly, bypassing unified sort:

1. **hand_drop_zone_render_system** — trivial, single glow rect -> `DrawCommand::Shape`
2. **stash_tab_render_system** — simple, few rects + text -> `DrawCommand::Shape` + `DrawCommand::Text`
3. **ui_render_system** — simple, background rects -> `DrawCommand::Shape`
4. **store_render_system** — medium, many procedural shapes + text
5. **stash_render_system** — complex, grid layout + drag preview
6. **stash_hover_preview_render_system** — complex, animated shader uniforms

### Migration pattern

Each is a mechanical transformation: replace `renderer.draw_shape(...)` with `draw_queue.push(layer, order, DrawCommand::Shape { ... })`. The DrawQueue infrastructure from Approach 2 makes this trivial. No architectural changes needed.

### End state

Unified sort is physically inescapable. The Renderer trait is an implementation detail of `unified_render_system`, not a public API for game systems.
