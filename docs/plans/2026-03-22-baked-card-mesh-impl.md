# Baked Card Mesh Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Collapse 17 child entities per card into a single pre-tessellated mesh, eliminating per-frame tessellation and reducing entity count from 1,800 to 100 at 100 cards.

**Architecture:** At spawn time, tessellate all static card geometry (border, strips, gems, text glyphs) into a single vertex/index buffer with per-vertex colors. Store as a `BakedCardMesh` component on the root card entity. Add a `draw_colored_mesh` method to the Renderer trait for drawing pre-colored vertices. The unified render system gets a baked-card code path.

**Tech Stack:** Lyon (tessellation), ttf-parser (font outlines), bevy_ecs (components), engine_render (renderer trait, shape pipeline), engine_ui (unified render system), card_game (card spawn/layout).

---

### Task 1: Add `ColorVertex` and `TessellatedColorMesh` to engine_render

These types store pre-tessellated geometry with per-vertex color, used by the bake process.

**Files:**
- Modify: `crates/engine_render/src/shape/components.rs` (add types after `TessellatedMesh`)
- Modify: `crates/engine_render/src/shape/mod.rs` (re-export new types)
- Modify: `crates/engine_render/src/prelude.rs` (re-export new types)

**Step 1: Write the failing test**

In `crates/engine_render/src/shape/components.rs`, add to the test module:

```rust
#[test]
fn when_color_vertex_size_checked_then_exactly_24_bytes() {
    // Act
    let size = std::mem::size_of::<ColorVertex>();

    // Assert
    assert_eq!(size, 24);
}

#[test]
fn when_colored_mesh_merge_two_triangles_then_indices_offset_correctly() {
    // Arrange
    let mut mesh = TessellatedColorMesh::new();
    let white = [1.0, 1.0, 1.0, 1.0];
    let red = [1.0, 0.0, 0.0, 1.0];
    mesh.push_vertices(
        &[[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]],
        &[0, 1, 2],
        white,
    );

    // Act
    mesh.push_vertices(
        &[[2.0, 0.0], [3.0, 0.0], [2.5, 1.0]],
        &[0, 1, 2],
        red,
    );

    // Assert
    assert_eq!(mesh.vertices.len(), 6);
    assert_eq!(mesh.indices.len(), 6);
    assert_eq!(mesh.indices[3..], [3, 4, 5]);
    assert_eq!(mesh.vertices[0].color, white);
    assert_eq!(mesh.vertices[3].color, red);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo.exe test -p engine_render when_color_vertex_size`
Expected: FAIL — `ColorVertex` and `TessellatedColorMesh` don't exist yet.

**Step 3: Write minimal implementation**

In `crates/engine_render/src/shape/components.rs`, add:

```rust
/// Vertex with baked position and RGBA color.
/// Layout matches `ShapeVertex` in the wgpu renderer (24 bytes).
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ColorVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

/// Pre-tessellated mesh with per-vertex color.
/// Used by `BakedCardMesh` to store card geometry that never changes.
#[derive(Clone, Debug, Default)]
pub struct TessellatedColorMesh {
    pub vertices: Vec<ColorVertex>,
    pub indices: Vec<u32>,
}

impl TessellatedColorMesh {
    pub fn new() -> Self {
        Self::default()
    }

    /// Append position-only vertices with a uniform color, offsetting indices.
    pub fn push_vertices(
        &mut self,
        positions: &[[f32; 2]],
        indices: &[u32],
        color: [f32; 4],
    ) {
        let base = self.vertices.len() as u32;
        self.vertices.extend(
            positions.iter().map(|&position| ColorVertex { position, color }),
        );
        self.indices.extend(indices.iter().map(|&i| i + base));
    }

    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }
}
```

Re-export from `crates/engine_render/src/shape/mod.rs` and `crates/engine_render/src/prelude.rs`.

**Step 4: Run test to verify it passes**

Run: `cargo.exe test -p engine_render when_color_vertex_size && cargo.exe test -p engine_render when_colored_mesh_merge`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/engine_render/src/shape/components.rs crates/engine_render/src/shape/mod.rs crates/engine_render/src/prelude.rs
git commit -m "feat(engine_render): add ColorVertex and TessellatedColorMesh types"
```

---

### Task 2: Add `draw_colored_mesh` to Renderer trait

The current `draw_shape` takes `&[[f32; 2]]` positions + uniform `Color`. The baked mesh needs to send pre-colored vertices directly. The GPU pipeline already supports per-vertex color (`ShapeVertex` has `position + color`), so we just need a new trait method that bypasses the color-expansion step.

**Files:**
- Modify: `crates/engine_render/src/renderer.rs` (add method to `Renderer` trait, `NullRenderer`)
- Modify: `crates/engine_render/src/testing/mod.rs` (add to `SpyRenderer` + capture type)
- Modify: `crates/engine_render/src/wgpu_renderer/types.rs` (add `push_colored` to `ShapeBatch`)

**Step 1: Write the failing test**

In `crates/engine_render/src/wgpu_renderer/types.rs` test module:

```rust
#[test]
fn when_colored_vertices_pushed_then_colors_preserved_per_vertex() {
    // Arrange
    let mut batch = ShapeBatch::new();
    let vertices = [
        ShapeVertex { position: [0.0, 0.0], color: [1.0, 0.0, 0.0, 1.0] },
        ShapeVertex { position: [1.0, 0.0], color: [0.0, 1.0, 0.0, 1.0] },
        ShapeVertex { position: [0.5, 1.0], color: [0.0, 0.0, 1.0, 1.0] },
    ];

    // Act
    batch.push_colored(&vertices, &[0, 1, 2]);

    // Assert
    assert_eq!(batch.vertex_count(), 3);
    assert_eq!(batch.vertices()[0].color, [1.0, 0.0, 0.0, 1.0]);
    assert_eq!(batch.vertices()[1].color, [0.0, 1.0, 0.0, 1.0]);
    assert_eq!(batch.vertices()[2].color, [0.0, 0.0, 1.0, 1.0]);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo.exe test -p engine_render when_colored_vertices_pushed`
Expected: FAIL — `push_colored` doesn't exist.

**Step 3: Write minimal implementation**

In `ShapeBatch` (`types.rs`), add:

```rust
pub(crate) fn push_colored(&mut self, vertices: &[ShapeVertex], indices: &[u32]) {
    let base = self.vertices.len() as u32;
    self.vertices.extend_from_slice(vertices);
    self.indices.extend(indices.iter().map(|&i| i + base));
}
```

In `Renderer` trait (`renderer.rs`), add after `draw_shape`:

```rust
fn draw_colored_mesh(
    &mut self,
    vertices: &[ColorVertex],
    indices: &[u32],
    model: [[f32; 4]; 4],
);
```

Add `use crate::shape::ColorVertex;` at the top of `renderer.rs`.

Implement in `NullRenderer`:

```rust
fn draw_colored_mesh(
    &mut self,
    _vertices: &[ColorVertex],
    _indices: &[u32],
    _model: [[f32; 4]; 4],
) {}
```

Implement in `SpyRenderer` (`testing/mod.rs`):

Add capture type: `pub type ColoredMeshCallLog = Arc<Mutex<Vec<(Vec<ColorVertex>, Vec<u32>, [[f32; 4]; 4])>>>;`

Add field to `SpyRenderer`: `colored_mesh_calls: Option<ColoredMeshCallLog>`

Add builder: `pub fn with_colored_mesh_capture(mut self, calls: ColoredMeshCallLog) -> Self { ... }`

Add impl:

```rust
fn draw_colored_mesh(
    &mut self,
    vertices: &[ColorVertex],
    indices: &[u32],
    model: [[f32; 4]; 4],
) {
    self.log_call("draw_colored_mesh");
    if let Some(capture) = &self.colored_mesh_calls {
        capture.lock().expect("colored mesh capture poisoned").push((
            vertices.to_vec(),
            indices.to_vec(),
            model,
        ));
    }
}
```

Implement in `WgpuRenderer` — locate the `draw_shape` implementation and add `draw_colored_mesh` next to it. Since `ColorVertex` has the same layout as `ShapeVertex` (both are `[f32;2] + [f32;4]` = 24 bytes), cast via `bytemuck` or copy:

```rust
fn draw_colored_mesh(
    &mut self,
    vertices: &[ColorVertex],
    indices: &[u32],
    model: [[f32; 4]; 4],
) {
    // ColorVertex has the same layout as ShapeVertex
    let shape_verts: Vec<ShapeVertex> = vertices.iter().map(|v| ShapeVertex {
        position: v.position,
        color: v.color,
    }).collect();
    self.shape_batch.push_colored(&shape_verts, indices);
    // ... store model transform same as draw_shape does
}
```

Note: Check the exact WgpuRenderer `draw_shape` impl to match the model-transform storage pattern. The `draw_colored_mesh` should follow the same pattern but skip the uniform-color expansion.

**Step 4: Run tests**

Run: `cargo.exe test -p engine_render when_colored_vertices_pushed && cargo.exe build`
Expected: PASS, no compile errors.

**Step 5: Commit**

```bash
git add crates/engine_render/
git commit -m "feat(engine_render): add draw_colored_mesh to Renderer trait"
```

---

### Task 3: Add text baking helper to engine_render

Text baking needs to tessellate glyphs and merge them into a `TessellatedColorMesh` with pre-applied offsets. This is similar to `render_text_transformed` but outputs geometry instead of draw calls.

**Files:**
- Modify: `crates/engine_render/src/font.rs` (add `bake_text_into_mesh`)

**Step 1: Write the failing test**

In `crates/engine_render/src/font.rs` test module:

```rust
#[test]
fn when_bake_text_single_char_then_mesh_is_nonempty() {
    // Arrange
    let mut mesh = TessellatedColorMesh::new();
    let color = [0.1, 0.1, 0.1, 1.0];

    // Act
    bake_text_into_mesh(&mut mesh, "A", 16.0, color, 0.0, 0.0);

    // Assert
    assert!(!mesh.is_empty());
    assert!(mesh.vertices.iter().all(|v| v.color == color));
}

#[test]
fn when_bake_text_two_chars_then_second_glyph_vertices_offset_right() {
    // Arrange
    let mut mesh = TessellatedColorMesh::new();
    let color = [1.0, 1.0, 1.0, 1.0];

    // Act
    bake_text_into_mesh(&mut mesh, "AB", 32.0, color, 0.0, 0.0);

    // Assert — vertices should span a wider range than single-char
    let mut single = TessellatedColorMesh::new();
    bake_text_into_mesh(&mut single, "A", 32.0, color, 0.0, 0.0);
    let max_x = mesh.vertices.iter().map(|v| v.position[0]).fold(f32::NEG_INFINITY, f32::max);
    let single_max_x = single.vertices.iter().map(|v| v.position[0]).fold(f32::NEG_INFINITY, f32::max);
    assert!(max_x > single_max_x, "two chars should extend further right");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo.exe test -p engine_render when_bake_text_single_char`
Expected: FAIL — `bake_text_into_mesh` doesn't exist.

**Step 3: Write minimal implementation**

In `crates/engine_render/src/font.rs`:

```rust
use crate::shape::TessellatedColorMesh;

/// Tessellate text glyphs and append them to an existing mesh with pre-applied
/// position offsets and uniform color. Used for baking text into card meshes.
pub fn bake_text_into_mesh(
    mesh: &mut TessellatedColorMesh,
    text: &str,
    font_size: f32,
    color: [f32; 4],
    base_x: f32,
    base_y: f32,
) {
    let face = ttf_parser::Face::parse(FONT_BYTES, 0).expect("embedded font is valid");
    let text_width = measure_text_with_face(&face, text, font_size);
    let center_x = base_x - text_width * 0.5;
    let glyphs = layout_text(&face, text, font_size);
    let mut cache = GlyphCache::new();
    for glyph in &glyphs {
        let glyph_mesh = cache.get_or_tessellate(&face, glyph.glyph_id, font_size);
        if glyph_mesh.vertices.is_empty() {
            continue;
        }
        // Pre-transform vertices by glyph offset
        let offset_x = center_x + glyph.x_offset;
        let transformed: Vec<[f32; 2]> = glyph_mesh
            .vertices
            .iter()
            .map(|&[x, y]| [x + offset_x, y + base_y])
            .collect();
        mesh.push_vertices(&transformed, &glyph_mesh.indices, color);
    }
}

/// Same as `bake_text_into_mesh` but with word wrapping.
pub fn bake_wrapped_text_into_mesh(
    mesh: &mut TessellatedColorMesh,
    text: &str,
    font_size: f32,
    color: [f32; 4],
    base_x: f32,
    base_y: f32,
    max_width: f32,
) {
    let lines = wrap_text(text, font_size, max_width);
    let line_height = font_size * 1.3;
    let total_height = (lines.len() as f32 - 1.0) * line_height;
    let start_y = base_y - total_height * 0.5;
    for (i, line) in lines.iter().enumerate() {
        let y_offset = start_y + i as f32 * line_height;
        bake_text_into_mesh(mesh, line, font_size, color, base_x, y_offset);
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo.exe test -p engine_render when_bake_text`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/engine_render/src/font.rs
git commit -m "feat(engine_render): add bake_text_into_mesh for pre-tessellating text"
```

---

### Task 4: Add `BakedCardMesh` and `CardOverlays` components

**Files:**
- Create: `crates/card_game/src/card/baked_mesh.rs`
- Modify: `crates/card_game/src/card/mod.rs` (add module)
- Modify: `crates/card_game/src/prelude.rs` (re-export)

**Step 1: Write the failing test**

In `crates/card_game/src/card/baked_mesh.rs`:

```rust
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn when_baked_card_mesh_default_then_both_faces_empty() {
        // Act
        let baked = BakedCardMesh::default();

        // Assert
        assert!(baked.front.is_empty());
        assert!(baked.back.is_empty());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo.exe test -p card_game when_baked_card_mesh_default`
Expected: FAIL — module doesn't exist.

**Step 3: Write minimal implementation**

```rust
use bevy_ecs::prelude::Component;
use engine_render::shape::TessellatedColorMesh;
use engine_render::material::Material2d;
use glam::Vec2;

/// Pre-tessellated card geometry. Built once at spawn, never mutated.
#[derive(Component, Clone, Debug, Default)]
pub struct BakedCardMesh {
    pub front: TessellatedColorMesh,
    pub back: TessellatedColorMesh,
}

/// A shader-driven overlay quad (art area, foil effect, or back face).
#[derive(Clone, Debug)]
pub struct CardOverlay {
    pub quad: [Vec2; 4],
    pub material: Material2d,
}

/// Shader-driven overlay layers drawn on top of the baked mesh.
#[derive(Component, Clone, Debug, Default)]
pub struct CardOverlays {
    pub art: Option<CardOverlay>,
    pub foil: Option<CardOverlay>,
    pub back: Option<CardOverlay>,
}
```

Wire into `mod.rs` and `prelude.rs`.

**Step 4: Run test to verify it passes**

Run: `cargo.exe test -p card_game when_baked_card_mesh_default`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/card_game/src/card/baked_mesh.rs crates/card_game/src/card/mod.rs crates/card_game/src/prelude.rs
git commit -m "feat(card_game): add BakedCardMesh and CardOverlays components"
```

---

### Task 5: Implement `bake_card_mesh` — core baking logic

This is the heart of the feature. It reuses the existing layout/color computation from `face_layout.rs`, `gem_sockets.rs`, and `visual_params.rs` but outputs a merged `TessellatedColorMesh` instead of spawning entities.

**Files:**
- Create: `crates/card_game/src/card/bake.rs`
- Modify: `crates/card_game/src/card/mod.rs` (add module)

**Step 1: Write the failing test**

In `crates/card_game/src/card/bake.rs`:

```rust
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::card::card_name::CardName;
    use crate::card::signature::CardSignature;

    fn test_signature() -> CardSignature {
        CardSignature::default()
    }

    #[test]
    fn when_bake_front_then_mesh_has_vertices_and_valid_indices() {
        // Arrange
        let sig = test_signature();
        let card_size = Vec2::new(60.0, 90.0);
        let name = CardName {
            name: "Test".to_owned(),
            description: "A test card".to_owned(),
        };

        // Act
        let mesh = bake_front_face(&sig, card_size, &name);

        // Assert
        assert!(!mesh.is_empty(), "front mesh should have geometry");
        assert_eq!(mesh.indices.len() % 3, 0, "indices should form triangles");
        let vcount = mesh.vertices.len() as u32;
        for &i in &mesh.indices {
            assert!(i < vcount, "index {i} out of bounds ({vcount} vertices)");
        }
    }

    #[test]
    fn when_bake_front_then_contains_gem_geometry() {
        // Arrange — signature with non-zero intensities produces visible gems
        let mut sig = CardSignature::default();
        sig.set(crate::card::signature::Element::Febris, 0.8);
        let card_size = Vec2::new(60.0, 90.0);
        let name = CardName {
            name: "Gem Test".to_owned(),
            description: "Has gems".to_owned(),
        };

        // Act
        let mesh = bake_front_face(&sig, card_size, &name);

        // Assert — mesh should have significantly more vertices than just 4 rectangles
        // (4 rects = ~16 verts; 8 circles add ~80+ verts depending on tessellation)
        assert!(
            mesh.vertices.len() > 30,
            "expected gems to add substantial geometry, got {} vertices",
            mesh.vertices.len()
        );
    }

    #[test]
    fn when_bake_back_then_mesh_has_vertices() {
        // Arrange
        let card_size = Vec2::new(60.0, 90.0);

        // Act
        let mesh = bake_back_face(card_size);

        // Assert
        assert!(!mesh.is_empty());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo.exe test -p card_game when_bake_front_then_mesh`
Expected: FAIL — `bake.rs` and functions don't exist.

**Step 3: Write implementation**

In `crates/card_game/src/card/bake.rs`:

```rust
use engine_render::font::{bake_text_into_mesh, bake_wrapped_text_into_mesh};
use engine_render::shape::{TessellatedColorMesh, tessellate};
use glam::Vec2;

use super::card_name::CardName;
use super::face_layout::FRONT_FACE_REGIONS;
use super::gem_sockets::{aspect_color, gem_desc_positions, gem_radius};
use super::signature::{CardSignature, Element};
use super::spawn_table_card::{rounded_rect_path, rect_polygon, CARD_CORNER_RADIUS};
use super::visual_params::generate_card_visuals;

const TEXT_COLOR: [f32; 4] = [0.1, 0.1, 0.1, 1.0];
const BACK_OUTER_COLOR: [f32; 4] = [0.3, 0.3, 0.4, 1.0];
const BACK_INNER_COLOR: [f32; 4] = [0.2, 0.2, 0.3, 1.0];

/// Tessellate all front-face geometry into a single mesh.
/// Geometry is appended back-to-front (painter's order):
/// border → name strip → art area bg → desc strip → text → gems
pub fn bake_front_face(
    signature: &CardSignature,
    card_size: Vec2,
    name: &CardName,
) -> TessellatedColorMesh {
    let mut mesh = TessellatedColorMesh::new();
    let (w, h) = (card_size.x, card_size.y);
    let half_w = w * 0.5;
    let half_h = h * 0.5;

    let visuals = generate_card_visuals(signature);

    // --- Shapes (border, strips) ---
    for (i, region) in FRONT_FACE_REGIONS.iter().enumerate() {
        let (reg_hw, reg_hh, offset_y) = region.resolve(w, h);
        let color = match i {
            0 => [1.0, 1.0, 1.0, 1.0], // border: white
            2 => color_to_array(visuals.art_color), // art area
            _ => color_to_array(region.color),
        };
        let variant = if i == 0 {
            rounded_rect_path(reg_hw, reg_hh, CARD_CORNER_RADIUS)
        } else {
            rect_polygon(reg_hw, reg_hh)
        };
        if let Ok(tess) = tessellate(&variant) {
            // Offset vertices by region position
            let offset: Vec<[f32; 2]> = tess.vertices
                .iter()
                .map(|&[x, y]| [x, y + offset_y])
                .collect();
            mesh.push_vertices(&offset, &tess.indices, color);
        }
    }

    // --- Name text ---
    let name_region = &FRONT_FACE_REGIONS[1];
    let (_name_hw, name_hh, name_oy) = name_region.resolve(w, h);
    let name_font_size = name_hh; // approximate: fit in strip height
    bake_text_into_mesh(
        &mut mesh,
        &name.name,
        name_font_size,
        TEXT_COLOR,
        0.0,
        name_oy,
    );

    // --- Description text (wrapped) ---
    let desc_region = &FRONT_FACE_REGIONS[3];
    let (desc_hw, _desc_hh, desc_oy) = desc_region.resolve(w, h);
    let desc_font_size = h / 20.0;
    let desc_max_width = desc_hw * 2.0 * 0.9;
    bake_wrapped_text_into_mesh(
        &mut mesh,
        &name.description,
        desc_font_size,
        TEXT_COLOR,
        0.0,
        desc_oy,
        desc_max_width,
    );

    // --- Gems ---
    let positions = gem_desc_positions(card_size);
    let elements = [
        Element::Solidum, Element::Febris, Element::Ordinem, Element::Lumines,
        Element::Varias, Element::Inertiae, Element::Subsidium, Element::Spatium,
    ];
    for (i, element) in elements.iter().enumerate() {
        let intensity = signature.intensity(*element);
        let aspect = signature.dominant_aspect(*element);
        let gem_color = color_to_array(aspect_color(aspect));
        let radius = gem_radius(intensity);
        let variant = engine_render::shape::ShapeVariant::Circle { radius };
        if let Ok(tess) = tessellate(&variant) {
            let pos = positions[i];
            let offset: Vec<[f32; 2]> = tess.vertices
                .iter()
                .map(|&[x, y]| [x + pos.x, y + pos.y])
                .collect();
            mesh.push_vertices(&offset, &tess.indices, gem_color);
        }
    }

    mesh
}

/// Tessellate back-face geometry into a single mesh.
pub fn bake_back_face(card_size: Vec2) -> TessellatedColorMesh {
    let mut mesh = TessellatedColorMesh::new();
    let half_w = card_size.x * 0.5;
    let half_h = card_size.y * 0.5;

    // Outer border (rounded)
    let outer = rounded_rect_path(half_w, half_h, CARD_CORNER_RADIUS);
    if let Ok(tess) = tessellate(&outer) {
        mesh.push_vertices(&tess.vertices, &tess.indices, BACK_OUTER_COLOR);
    }

    // Inner panel
    let inner = rect_polygon(half_w * 0.85, half_h * 0.85);
    if let Ok(tess) = tessellate(&inner) {
        mesh.push_vertices(&tess.vertices, &tess.indices, BACK_INNER_COLOR);
    }

    mesh
}

fn color_to_array(c: engine_core::color::Color) -> [f32; 4] {
    [c.r, c.g, c.b, c.a]
}
```

**Important:** The exact function names `rounded_rect_path`, `rect_polygon`, and constant `CARD_CORNER_RADIUS` need to be made `pub(crate)` in `spawn_table_card.rs` if they aren't already. Similarly, `FRONT_FACE_REGIONS` fields and the `resolve` method, `aspect_color`, `gem_desc_positions`, `gem_radius` all need to be accessible. Check visibility and adjust as needed.

**Step 4: Run tests**

Run: `cargo.exe test -p card_game when_bake_front && cargo.exe test -p card_game when_bake_back`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/card_game/src/card/bake.rs crates/card_game/src/card/mod.rs
git commit -m "feat(card_game): implement bake_front_face and bake_back_face"
```

---

### Task 6: Add baked-card rendering path to unified_render_system

The unified render system needs to detect `BakedCardMesh` entities and draw them using `draw_colored_mesh` instead of tessellating children.

**Files:**
- Modify: `crates/engine_ui/src/unified_render.rs`

**Step 1: Write the failing test**

In `crates/engine_ui/src/unified_render.rs` test module:

```rust
#[test]
fn when_baked_card_mesh_entity_then_draw_colored_mesh_called() {
    // Arrange
    let mut world = World::new();
    // Insert spy that captures draw_colored_mesh calls
    // ... (need ColoredMeshCallLog from engine_render::testing)
    let mut front = engine_render::shape::TessellatedColorMesh::new();
    front.push_vertices(
        &[[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]],
        &[0, 1, 2],
        [1.0, 0.0, 0.0, 1.0],
    );
    world.spawn((
        card_game::card::baked_mesh::BakedCardMesh {
            front,
            back: engine_render::shape::TessellatedColorMesh::new(),
        },
        card_game::card::component::Card { face_up: true, .. },
        GlobalTransform2D(Affine2::IDENTITY),
        SortOrder(0),
        RenderLayer::World,
    ));

    // Act
    let calls = run_system(&mut world);

    // Assert — should have called draw_colored_mesh, not draw_shape via tessellation
    // (verify through spy captures)
}
```

Note: The exact test setup depends on how `SpyRenderer` captures `draw_colored_mesh`. This test needs the spy infrastructure from Task 2. The test verifies that the baked path is taken.

**Step 2: Implement the baked-card rendering path**

Add a new query type and `DrawKind` variant:

```rust
use crate::card::baked_mesh::BakedCardMesh;
use crate::card::component::Card;

type BakedCardItem<'w> = (
    Entity,
    &'w BakedCardMesh,
    &'w Card,
    &'w GlobalTransform2D,
    Option<&'w RenderLayer>,
    Option<&'w SortOrder>,
    Option<&'w EffectiveVisibility>,
    Option<&'w CardOverlays>,
);
```

Add `DrawKind::BakedCard` variant.

In `collect_draw_items`, add collection of baked card entities.

In the main render loop, add:

```rust
DrawKind::BakedCard => {
    let Ok((_, baked, card, transform, _, _, _, overlays)) =
        baked_query.get(item.entity)
    else {
        continue;
    };
    let model = affine2_to_mat4(&transform.0);
    let face_mesh = if card.face_up { &baked.front } else { &baked.back };
    if !face_mesh.is_empty() {
        renderer.draw_colored_mesh(&face_mesh.vertices, &face_mesh.indices, model);
    }
    // TODO: draw overlays (art, foil, back shader) — future task
}
```

**Important:** The `unified_render_system` function signature needs a new query parameter for baked cards. This is a system function — bevy_ecs allows up to ~16 query parameters.

**Step 3: Run tests**

Run: `cargo.exe test -p engine_ui` and `cargo.exe build`
Expected: PASS, existing tests still pass, new test passes.

**Step 4: Commit**

```bash
git add crates/engine_ui/src/unified_render.rs
git commit -m "feat(engine_ui): add baked-card rendering path to unified_render_system"
```

---

### Task 7: Update `spawn_visual_card` to use baking

Replace child entity spawning with bake calls.

**Files:**
- Modify: `crates/card_game/src/card/spawn_table_card.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn when_spawn_visual_card_then_entity_has_baked_card_mesh() {
    // Arrange
    let mut world = World::new();
    // ... setup physics, etc.

    // Act
    let entity = spawn_visual_card(&mut world, &def, pos, size, true, signature);

    // Assert
    assert!(world.get::<BakedCardMesh>(entity).is_some());
    assert!(world.get::<CardOverlays>(entity).is_some());
}

#[test]
fn when_spawn_visual_card_then_no_child_entities_exist() {
    // Arrange
    let mut world = World::new();
    // ... setup

    // Act
    let entity = spawn_visual_card(&mut world, &def, pos, size, true, signature);

    // Assert — no entities with ChildOf pointing to this card
    let children: Vec<_> = world.query::<&ChildOf>()
        .iter(&world)
        .filter(|c| c.parent() == entity)
        .collect();
    assert!(children.is_empty(), "baked cards should have no children");
}
```

**Step 2: Modify `spawn_visual_card`**

Replace the four `spawn_*_children` calls with:

```rust
let baked = BakedCardMesh {
    front: bake_front_face(&signature, card_size, &label),
    back: bake_back_face(card_size),
};
let overlays = CardOverlays::default(); // art/foil overlays added later
```

Add `baked` and `overlays` to the entity's component tuple.

Remove calls to `spawn_front_face_children`, `spawn_back_face_children`, `spawn_text_children`, `spawn_gem_children`, and the `StashIcon` child spawn.

**Step 3: Run tests**

Run: `cargo.exe test -p card_game` — many existing tests will fail because they assert on child entities. Fix or remove those tests (see Task 8).

Run: `cargo.exe build -p card_game_bin` — verify binary compiles.

**Step 4: Commit**

```bash
git add crates/card_game/src/card/spawn_table_card.rs
git commit -m "feat(card_game): spawn_visual_card uses baked mesh instead of child entities"
```

---

### Task 8: Update flip animation for baked cards

The flip animation currently relies on toggling `Visible` on child entities with `CardFaceSide::Front/Back`. With baked cards, the render system reads `card.face_up` directly — no visibility toggling needed. The flip animation just needs to toggle `card.face_up` at the midpoint (which it already does).

**Files:**
- Modify: `crates/card_game/src/card/flip_animation.rs` (remove child visibility toggling if any)
- Modify: `crates/card_game/src/card/item_form.rs` (remove `card_item_form_visibility_system` child toggles)
- Verify: `crates/card_game/src/card/flip.rs` still works (only touches root entity)

**Step 1: Verify flip animation still works**

Run: `cargo.exe test -p card_game flip`
Check which tests fail and fix them — most should still pass since the animation operates on the root `Card` component.

**Step 2: Remove child-visibility systems that no longer have children to toggle**

The `card_item_form_visibility_system` (in `item_form.rs`) iterates children and toggles `Visible`. Since there are no children, it either becomes a no-op or should be removed. Check if it does anything else useful.

**Step 3: Commit**

```bash
git commit -m "refactor(card_game): simplify flip animation for baked cards"
```

---

### Task 9: Update stash rendering for baked cards

The stash previously rendered a `StashIcon` child entity. Now it should render the baked card at a scaled-down transform.

**Files:**
- Modify: `crates/card_game/src/stash/render.rs` or `layout.rs`

**Step 1: Check stash system behavior**

The stash layout system positions cards by setting `Transform2D.position`. If the unified render system already draws baked cards at their transform, the stash just needs to set the right position and scale. The card will render itself via the baked-card path.

Check if extra work is needed for the scale-down (stash slots are smaller than table cards).

**Step 2: Remove `StashIcon` component and related code**

**Step 3: Run stash-related tests**

Run: `cargo.exe test -p card_game stash`

**Step 4: Commit**

```bash
git commit -m "refactor(card_game): stash renders baked cards directly, remove StashIcon"
```

---

### Task 10: Clean up dead code

Remove types, functions, and systems that are no longer used after baking migration.

**Files:**
- Modify: `crates/card_game/src/card/spawn_table_card.rs` (remove `spawn_*_children` functions, `FaceChildDef`)
- Modify: `crates/card_game/src/card/face_side.rs` (remove `CardFaceSide` if unused)
- Modify: `crates/card_game/src/stash/icon.rs` (remove `StashIcon` if unused)
- Modify: `crates/card_game/src/card/item_form.rs` (remove child visibility system if unused)
- Modify: `crates/card_game/src/plugin.rs` (unregister removed systems)
- Modify: `crates/card_game/src/prelude.rs` (remove dead re-exports)
- Modify: `crates/card_game_bin/src/main.rs` (remove any system registrations for deleted systems)

**Step 1: Search for usages of each candidate**

Use `cargo.exe build` to find compile errors after removing. The compiler is the best dead-code detector.

**Step 2: Remove and verify**

Run: `cargo.exe build && cargo.exe test -p card_game && cargo.exe clippy -p card_game`

**Step 3: Commit**

```bash
git commit -m "refactor(card_game): remove child-entity spawn code and CardFaceSide"
```

---

### Task 11: Update and fix remaining tests

After all the structural changes, run the full test suite and fix any failures.

**Files:**
- Various test modules in `crates/card_game/src/card/spawn_table_card.rs`
- Various test modules in `crates/card_game/src/stash/`
- Integration tests

**Step 1: Run full test suite**

Run: `cargo.exe test --workspace`

**Step 2: Fix failing tests**

- Tests asserting on child entity counts → rewrite to assert on `BakedCardMesh` contents
- Tests asserting on `CardFaceSide` → remove or rewrite
- Tests asserting on `StashIcon` → remove or rewrite

**Step 3: Run clippy**

Run: `cargo.exe clippy --workspace`

**Step 4: Format**

Run: `cargo.exe fmt --all`

**Step 5: Commit**

```bash
git commit -m "test(card_game): update tests for baked card mesh architecture"
```

---

### Task 12: Wire baked card rendering into game binary

Ensure the binary compiles and runs with the new baked-card path.

**Files:**
- Modify: `crates/card_game_bin/src/main.rs` (if system registration changes needed)
- Modify: `crates/card_game/src/plugin.rs` (if new systems need registration)

**Step 1: Build the binary**

Run: `cargo.exe build -p card_game_bin`

**Step 2: Verify no runtime panics**

The binary should spawn cards with baked meshes and render them through the baked-card path. Verify by running with debug spawn (key 1 for single card, key 3 for 100 cards).

**Step 3: Commit**

```bash
git commit -m "feat(card_game_bin): wire baked card rendering into game binary"
```
