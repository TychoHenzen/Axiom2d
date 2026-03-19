# Vector Text Rendering Plan

## Goal
Implement scale-independent text rendering in WgpuRenderer using vector glyph tessellation. Text on cards should look sharp at any zoom level.

## What's Already Done
- `draw_text(&mut self, text: &str, x: f32, y: f32, font_size: f32, color: Color)` on Renderer trait
- NullRenderer, SpyRenderer (with TextCallLog capture), HeadlessRenderer: all implemented
- WgpuRenderer: stub (no-op)
- `CardLabel { name, description }` component
- `spawn_visual_card` spawns Text children when CardLabel is provided
- `card_text_render_system` queries Text + GlobalTransform2D, calls draw_text
- Wired into card_game_bin Phase::Render

## Architecture

### New Dependency
- `ttf-parser` (workspace dev-dep or regular dep): zero-alloc TTF/OTF outline parser (~50KB, no unsafe, no allocations). Provides glyph outlines as moveto/lineto/curveto commands.

### Font Embedding
- Embed a small open-source monospace font (e.g., JetBrains Mono, Fira Code, or a subset) via `include_bytes!` in a new `engine_render::font` module. Fits the code-defined-assets philosophy (bytes live in Rust source).

### Glyph Pipeline
1. **Parse**: On init, parse the embedded TTF with `ttf-parser::Face`
2. **Tessellate**: For each unique (glyph_id, font_size) pair, tessellate the outline using lyon `FillTessellator` (same as ShapeVariant::Polygon). Cache the tessellated mesh.
3. **Render**: In `WgpuRenderer::draw_text`, for each character:
   - Look up glyph mesh from cache (tessellate on first use)
   - Compute character position using advance widths from the font
   - Submit as shape draw via existing shape pipeline (ShapeBatch + ShapeDrawRecord)

### Key Design Decisions
- **Glyph cache**: `HashMap<(GlyphId, u16), TessellatedGlyph>` where u16 is font_size rounded to nearest integer. Cache lives on WgpuRenderer.
- **Coordinate mapping**: TTF uses font units (typically 1000 or 2048 units per em). Scale factor = font_size / units_per_em. Apply scale when tessellating.
- **Advance/kerning**: Use `ttf-parser::Face::glyph_hor_advance` for character spacing. Kerning optional (nice-to-have).
- **Reuse existing pipeline**: Text glyphs render through the same shape pipeline as Shape components. No new GPU pipeline needed. Color passed per-glyph via ShapeBatch.

### Files to Create/Modify
| File | Change |
|---|---|
| `Cargo.toml` (root) | Add `ttf-parser = "0.24"` to workspace deps |
| `crates/engine_render/Cargo.toml` | Add `ttf-parser = { workspace = true }` |
| `crates/engine_render/src/font.rs` | New module: embedded font bytes, `GlyphCache`, `tessellate_glyph()`, `layout_text()` |
| `crates/engine_render/src/wgpu_renderer/renderer_trait.rs` | Replace draw_text stub with real implementation using GlyphCache + ShapeBatch |
| `crates/engine_render/src/wgpu_renderer/renderer.rs` | Add `glyph_cache: GlyphCache` field to WgpuRenderer |
| `crates/engine_render/src/visual_regression.rs` | Implement draw_text on HeadlessRenderer (same as WgpuRenderer) |

### TDD Test Cases
1. `font.rs`: Parse embedded font -> glyph count > 0
2. `font.rs`: Tessellate 'A' -> non-empty vertices/indices
3. `font.rs`: Layout "AB" -> two glyph positions with correct advance
4. `font.rs`: Cache hit on second tessellation of same glyph
5. `wgpu_renderer`: draw_text("Hi") -> 2 ShapeDrawRecords in shape_draws
6. `visual_regression`: draw_text renders non-background pixels at text position

### Scope Estimate
- font.rs module: ~150 lines (embed font, parse, tessellate, cache, layout)
- WgpuRenderer changes: ~30 lines
- Tests: ~100 lines
- Total: ~280 lines of new code

### Risks
- Font outline complexity: some glyphs have cubic beziers that lyon must approximate with line segments. Quality depends on tolerance setting.
- Cache memory: each glyph at each font size is cached independently. For card games with limited text, this is fine.
- No text wrapping or line breaking in this phase — single-line rendering only.
