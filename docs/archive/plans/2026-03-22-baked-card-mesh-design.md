# Baked Card Mesh Design

## Problem

At 100 cards, FPS is terrible. Each card spawns 1 root + 17 child entities = 18 entities per card (1,800 total). Every frame, `unified_render_system` tessellates ~1,500 shapes from scratch using Lyon, sorts them all, and issues ~17 draw calls per card. Tessellation is the dominant cost — ~3,000 heap allocations per frame for geometry that never changes.

## Solution: Bake cards at spawn time

Since a card's visual appearance is fully determined by its immutable signature, all static geometry (border, strips, gems, text glyphs) is tessellated once at spawn and stored as a single pre-built vertex/index buffer. Child entities are eliminated entirely.

## Data Model

### New components

```rust
/// Pre-tessellated mesh containing all static card geometry.
/// Built once at spawn, never mutated.
pub struct BakedCardMesh {
    pub front: TessellatedColorMesh,
    pub back: TessellatedColorMesh,
}

/// A tessellated mesh with per-vertex color.
pub struct TessellatedColorMesh {
    pub vertices: Vec<ColorVertex>,  // position + RGBA
    pub indices: Vec<u32>,
}

pub struct ColorVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

/// Shader-driven overlay layers.
pub struct CardOverlays {
    pub art: Option<CardOverlay>,   // art area shader, clipped to art region
    pub foil: Option<CardOverlay>,  // fullcard overlay, additive blend
    pub back: Option<CardOverlay>,  // back-face shader
}

pub struct CardOverlay {
    pub quad: [Vec2; 4],
    pub material: Material2d,
}
```

### Card entity after baking

```
Card (root entity)
  Components: Card, CardDefinition, CardLabel, CardZone, Transform2D,
              RigidBody::Dynamic, Collider::Aabb, RenderLayer::World,
              SortOrder, BakedCardMesh, CardOverlays, Visible
```

No child entities. No ChildOf relationships for rendering.

## Bake Process

```rust
pub fn bake_card_mesh(
    card_def: &CardDefinition,
    signature: &CardSignature,
    profile: &SignatureProfile,
) -> BakedCardMesh
```

1. Computes all geometry using existing layout functions (face_layout, gem_sockets, visual_params)
2. Tessellates each piece immediately instead of spawning entities
3. Assigns per-vertex colors during tessellation
4. Merges all pieces into a single vertex/index buffer, appended back-to-front (painter's algorithm baked into buffer order): border -> strips -> text -> gems
5. Builds back-face mesh the same way (outer border + inner panel)

`spawn_visual_card()` calls `bake_card_mesh` + `build_card_overlays` and spawns a single entity.

## Render Integration

The unified render system gains a baked-card code path:

**Face-up draw sequence:**
1. Draw `baked.front` — single draw_indexed call with pre-built buffers + card's model matrix
2. Draw `overlays.art` — art area quad with shader
3. Draw `overlays.foil` — if present, fullcard quad with additive blend

**Face-down draw sequence:**
1. Draw `baked.back` if it has geometry
2. Draw `overlays.back` — back-face shader quad

**Sort order:** Baked cards use the same `(RenderLayer, SortOrder)` sorting. Internal layer ordering is guaranteed by vertex buffer order.

**Coexistence:** Non-baked entities continue through the existing tessellate-per-frame path. The system checks for `BakedCardMesh` first; if absent, falls back to current behavior.

**New vertex format:** Per-vertex color requires a `ColorVertex` struct, a matching WGSL shader, and a `draw_colored_mesh` method on the renderer trait.

## Performance Impact

| Metric | Before | After |
|--------|--------|-------|
| Entities (100 cards) | 1,800 | 100 |
| Tessellations/frame | ~1,500 | 0 |
| Draw calls (100 cards) | ~1,700 | ~300 |
| Sort list size | ~1,500 | ~100 |
| Heap allocs/frame (shapes) | ~3,000 | 0 |

## Migration & Cleanup

### Removed
- `spawn_front_face_children`, `spawn_back_face_children`, `spawn_text_children`, `spawn_gem_children` — replaced by `bake_card_mesh`
- `CardFaceSide` component — face visibility handled at root level
- `LocalSortOrder` — no children to sort
- `StashIcon` as child entity — stash shows scaled-down baked card
- `flip_visibility_system` for children — render system checks `card.face_up` directly

### Preserved
- `face_layout.rs`, `gem_sockets.rs`, `visual_params.rs`, `signature.rs` — layout/color computation reused by `bake_card_mesh`
- Physics body/collider on root
- Card zones, hand/stash inventory systems
- `CardDefinition`, `CardLabel`, `CardSignature`, `SignatureProfile`

## Stash Rendering

Stash shows the full baked card at a scaled-down transform. No special case needed — the baked mesh is just vertex data, scaling is free.

## Future: Foil/Hologram Effects

The foil overlay is a fullcard quad with an additive/overlay blend shader that responds to view angle or time. It's a `CardOverlay` entry — adding or removing it is just attaching/detaching the overlay. This keeps the effect dynamic without re-tessellating anything.
