# Art Contour Glow — Requirements Spec

> **For Claude:** This spec was produced by /interview. Use /writing-plans to expand into an implementation plan, or /tdd to implement directly.

**Goal:** Replace the rect-based glow shader (Rare rarity) with a contour-following bloom that traces the actual combined art shape silhouette using lyon stroke tessellation.

**Date:** 2026-03-27

---

## Requirements

### What it does
- Generates a stroke mesh around **all** art shapes' paths combined, using lyon's `tessellate_stroke()` with a wide line width (~20px)
- Uses `StrokeVertex::side()` to encode alpha in the vertex color: inner edge = full opacity, outer edge = transparent — creating a smooth falloff
- Glow color is derived from the art shapes' dominant/average color
- This stroke mesh becomes a new overlay entry rendered **behind** the art (lower sort order) so the glow appears to emanate outward
- Replaces the current `glow.wgsl` which uses a rect SDF
- Combined silhouette: all shapes treated as one mass, not per-shape glow

### What it does NOT do
- No per-shape individual glow — one unified silhouette
- Not composable with other rarity shaders — Rare only
- No animation or pointer reactivity (yet)
- No time uniforms

### Visual spec
- Glow width: medium, 15-25px at default card size
- Glow color: derived from the art shapes' dominant/average color
- Falloff: smooth alpha gradient from inner edge (opaque) to outer edge (transparent)

## Subtask Checklist

- [ ] 1. Add `tessellate_art_glow(shapes: &[Shape], line_width: f32) -> TessellatedColorMesh` in `card_game::card::art` — strokes all art shape paths using lyon's `tessellate_stroke()`, using `StrokeVertex::side()` to set vertex alpha (inner=1.0, outer=0.0), with glow color derived from average of all shape colors. Y-flip must match `tessellate_art_shapes()`.
- [ ] 2. Update `build_mesh_overlays()` in `spawn_table_card.rs` — when `ShaderVariant::Glow`, generate the glow stroke mesh via `tessellate_art_glow()`, fit it to the art region with `fit_art_mesh_to_region()`, and add it as an overlay entry rendered behind the art.
- [ ] 3. Simplify `glow.wgsl` — remove the rect SDF distance computation. The shader now just passes through vertex color with vertex alpha (falloff is baked into the mesh geometry). Keep the discard for zero-alpha to avoid fill-rate waste.
- [ ] 4. Wire up art shapes access in `build_mesh_overlays()` — the function currently receives `front_mesh` but not the original `Shape` data. Pass the art shapes through so we can stroke-tessellate them.
- [ ] 5. Tests: verify glow mesh has non-zero vertices, all alpha values in [0,1], glow vertices don't overlap art vertices, and the mesh fits within art region bounds after fitting.

## Research Notes

### Lyon stroke tessellation
- `tessellate_stroke()` already exists in `engine_render::shape::tessellate` (line 167)
- `StrokeVertex` provides: `position()`, `normal()`, `side()`, `advancement()`
- `side()` returns which side of the path (positive/negative) — this encodes inner vs outer edge
- Current stroke vertex callback only captures `position().to_array()` — needs to also capture side info to encode alpha

### Vertex format
- `ColorVertex { position: [f32;2], color: [f32;4], uv: [f32;2] }` — alpha channel in color[3] can carry the falloff
- UV should be `[0.0, 0.0]` for glow vertices (non-art) to avoid triggering art-specific shader logic

### Art data flow
- Art shapes are hydrated from `ShapeRepository` → `Vec<Shape>` with path commands
- `tessellate_art_shapes()` does fill tessellation + Y-flip + AABB UV computation
- `fit_art_mesh_to_region()` uniformly scales and centers the mesh in the art region
- The glow mesh must go through the same fitting so contours align

### Current glow.wgsl
- Uses rect SDF (`abs(local_pos - art_center) - art_half`) for distance
- Gaussian falloff: `exp(-dist*dist / (spread*spread))`
- Only renders on non-UV vertices (card background), transparent on art vertices
- This approach will be replaced entirely

## Open Questions

- Should overlapping glow from stacked cards blend additively or use max blending? (Deferred)
- Future: could add time-based pulse animation via a time uniform (not in scope)
