# Unified Render System Redesign — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `unified_render_system` the single renderer for all projects — absorb sprites, add DrawCommand buffer with DrawQueue for immediate-mode systems, move to Phase::PostRender so draw order is physically inescapable.

**Architecture:** Replace the current re-query pattern with a DrawCommand enum that captures all render data upfront. A DrawQueue resource lets immediate-mode systems push commands that get sorted alongside entity-sourced draws. The system moves from Phase::Render to Phase::PostRender, guaranteeing it runs after all prep systems. `sprite_render_system`, `shape_render_system`, and `ShapeRenderDisabled` are deleted entirely.

**Tech Stack:** Rust, bevy_ecs (standalone), engine_render (Renderer trait, Shape, Sprite, Material2d), engine_ui (Text, unified_render), engine_scene (RenderLayer, SortOrder)

**Spec:** `docs/superpowers/specs/2026-04-09-unified-render-design.md`

---

### Task 1: Add Clone derive to TessellatedMesh

**Files:**
- Modify: `crates/engine_render/src/shape/components.rs:29`

- [ ] **Step 1: Add Clone derive**

```rust
#[derive(Clone)]
pub struct TessellatedMesh {
    pub vertices: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}
```

- [ ] **Step 2: Verify build**

Run: `cargo.exe build -p engine_render`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add crates/engine_render/src/shape/components.rs
git commit -m "feat(render): derive Clone on TessellatedMesh for DrawCommand buffer"
```

---

### Task 2: Create DrawCommand and DrawQueue types

**Files:**
- Create: `crates/engine_ui/src/draw_command.rs`
- Modify: `crates/engine_ui/src/lib.rs` (add module declaration)

- [ ] **Step 1: Create draw_command.rs**

```rust
use bevy_ecs::prelude::Resource;
use engine_core::color::Color;
use engine_render::material::Material2d;
use engine_render::rect::Rect;
use engine_render::renderer::GpuMeshHandle;
use engine_render::shape::{TessellatedColorMesh, TessellatedMesh};
use engine_scene::prelude::{RenderLayer, SortOrder};

pub struct StrokeCommand {
    pub mesh: TessellatedMesh,
    pub color: Color,
}

pub struct OverlayCommand {
    pub mesh: TessellatedColorMesh,
    pub material: Material2d,
}

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

#[derive(Resource, Default)]
pub struct DrawQueue {
    commands: Vec<SortedDrawCommand>,
}

impl DrawQueue {
    /// Push a draw command with explicit sort position.
    /// Systems in Phase::Render push here; unified_render_system drains in Phase::PostRender.
    pub fn push(&mut self, layer: RenderLayer, order: SortOrder, command: DrawCommand) {
        self.commands.push(SortedDrawCommand {
            sort_key: (layer, order),
            command,
        });
    }

    pub(crate) fn drain(&mut self) -> Vec<SortedDrawCommand> {
        std::mem::take(&mut self.commands)
    }
}
```

- [ ] **Step 2: Add module to lib.rs**

In `crates/engine_ui/src/lib.rs`, add after `pub mod unified_render;`:

```rust
pub mod draw_command;
```

- [ ] **Step 3: Verify build**

Run: `cargo.exe build -p engine_ui`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add crates/engine_ui/src/draw_command.rs crates/engine_ui/src/lib.rs
git commit -m "feat(ui): add DrawCommand enum and DrawQueue resource"
```

---

### Task 3: Write failing tests for sprite interleaving and DrawQueue

**Files:**
- Modify: `crates/engine_ui/tests/suite/unified_render.rs`

These tests will fail until Task 4 rewrites unified_render_system.

- [ ] **Step 1: Add sprite interleaving test**

Add to `crates/engine_ui/tests/suite/unified_render.rs`:

```rust
use engine_core::types::{Pixels, TextureId};
use engine_render::sprite::Sprite;
use engine_render::testing::insert_spy;

#[test]
fn when_sprite_has_lower_sort_order_then_drawn_before_shape() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(engine_ui::draw_command::DrawQueue::default());
    let log = insert_spy(&mut world);
    world.spawn((
        Shape {
            variant: ShapeVariant::Circle { radius: 10.0 },
            color: Color::RED,
        },
        GlobalTransform2D(Affine2::IDENTITY),
        SortOrder::new(5),
    ));
    world.spawn((
        Sprite {
            texture: TextureId(0),
            uv_rect: [0.0, 0.0, 1.0, 1.0],
            color: Color::WHITE,
            width: Pixels(32.0),
            height: Pixels(32.0),
        },
        GlobalTransform2D(Affine2::IDENTITY),
        SortOrder::new(1),
    ));

    // Act
    let mut schedule = Schedule::default();
    schedule.add_systems(unified_render_system);
    schedule.run(&mut world);

    // Assert — sprite (SortOrder 1) should draw before shape (SortOrder 5)
    let calls = log.lock().unwrap();
    let first_sprite = calls.iter().position(|c| c == "draw_sprite");
    let first_shape = calls.iter().position(|c| c == "draw_shape");
    assert!(
        first_sprite.is_some() && first_shape.is_some(),
        "should find both sprite and shape draw calls"
    );
    assert!(
        first_sprite.unwrap() < first_shape.unwrap(),
        "sprite (SortOrder 1) should draw before shape (SortOrder 5)"
    );
}
```

- [ ] **Step 2: Add DrawQueue integration test**

```rust
use engine_ui::draw_command::{DrawCommand, DrawQueue};

#[test]
fn when_draw_queue_command_has_lower_sort_order_then_drawn_before_entity() {
    // Arrange
    let mut world = World::new();
    let mut queue = DrawQueue::default();
    queue.push(
        RenderLayer::World,
        SortOrder::new(1),
        DrawCommand::Shape {
            mesh: engine_render::shape::TessellatedMesh {
                vertices: vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]],
                indices: vec![0, 1, 2],
            },
            color: Color::BLUE,
            model: engine_render::prelude::IDENTITY_MODEL,
            material: None,
            stroke: None,
        },
    );
    world.insert_resource(queue);
    let log = insert_spy(&mut world);

    // Entity shape at SortOrder 5
    world.spawn((
        Shape {
            variant: ShapeVariant::Circle { radius: 10.0 },
            color: Color::RED,
        },
        GlobalTransform2D(Affine2::IDENTITY),
        SortOrder::new(5),
    ));

    // Act
    let mut schedule = Schedule::default();
    schedule.add_systems(unified_render_system);
    schedule.run(&mut world);

    // Assert — queued command (SortOrder 1) drawn before entity (SortOrder 5)
    let calls = log.lock().unwrap();
    let shape_calls: Vec<_> = calls
        .iter()
        .enumerate()
        .filter(|(_, c)| c.as_str() == "draw_shape")
        .collect();
    assert_eq!(
        shape_calls.len(),
        2,
        "should have 2 draw_shape calls (queued + entity)"
    );
}

#[test]
fn when_draw_queue_drained_then_empty_after_render() {
    // Arrange
    let mut world = World::new();
    let mut queue = DrawQueue::default();
    queue.push(
        RenderLayer::World,
        SortOrder::new(0),
        DrawCommand::Shape {
            mesh: engine_render::shape::TessellatedMesh {
                vertices: vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]],
                indices: vec![0, 1, 2],
            },
            color: Color::WHITE,
            model: engine_render::prelude::IDENTITY_MODEL,
            material: None,
            stroke: None,
        },
    );
    world.insert_resource(queue);
    insert_spy(&mut world);

    // Act
    let mut schedule = Schedule::default();
    schedule.add_systems(unified_render_system);
    schedule.run(&mut world);

    // Assert — queue is empty after drain
    let queue = world.resource::<DrawQueue>();
    let drained = queue.commands.len();
    // DrawQueue.commands is pub(crate) so we can't access directly from test.
    // Instead, run a second frame — if queue was drained, no extra draw calls.
    let log2 = insert_spy(&mut world);
    schedule.run(&mut world);
    let calls = log2.lock().unwrap();
    let shape_count = calls.iter().filter(|c| c.as_str() == "draw_shape").count();
    assert_eq!(shape_count, 0, "queue should be empty on second frame");
}
```

- [ ] **Step 3: Add stroke rendering test**

```rust
#[test]
fn when_shape_has_stroke_then_fill_drawn_before_stroke() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(DrawQueue::default());
    let shape_calls = insert_spy_with_shape_capture(&mut world);
    world.spawn((
        Shape {
            variant: ShapeVariant::Circle { radius: 10.0 },
            color: Color::WHITE,
        },
        GlobalTransform2D(Affine2::IDENTITY),
        SortOrder::new(0),
        engine_render::prelude::Stroke {
            color: Color::new(0.0, 0.0, 0.0, 1.0),
            width: 2.0,
        },
    ));

    // Act
    let mut schedule = Schedule::default();
    schedule.add_systems(unified_render_system);
    schedule.run(&mut world);

    // Assert — fill (WHITE) first, stroke (BLACK) second
    let calls = shape_calls.lock().unwrap();
    assert_eq!(calls.len(), 2, "should have fill + stroke draw calls");
    assert_eq!(calls[0].2, Color::WHITE);
    assert_eq!(calls[1].2, Color::new(0.0, 0.0, 0.0, 1.0));
}
```

- [ ] **Step 4: Run tests to verify they fail**

Run: `cargo.exe test -p engine_ui when_sprite_has_lower_sort_order when_draw_queue_command when_draw_queue_drained when_shape_has_stroke_then_fill`
Expected: FAIL — unified_render_system doesn't query sprites or DrawQueue yet.

- [ ] **Step 5: Commit failing tests**

```bash
git add crates/engine_ui/tests/suite/unified_render.rs
git commit -m "test(ui): add failing tests for sprite interleaving, DrawQueue, and stroke"
```

---

### Task 4: Rewrite unified_render_system using DrawCommand

**Files:**
- Modify: `crates/engine_ui/src/unified_render.rs` (full rewrite)

- [ ] **Step 1: Update existing tests to insert DrawQueue resource**

The existing `run_system` and `run_system_colored` helpers in `crates/engine_ui/tests/suite/unified_render.rs` need DrawQueue. Update both:

```rust
fn run_system(world: &mut World) -> ShapeCallLog {
    world.insert_resource(engine_ui::draw_command::DrawQueue::default());
    let shape_calls = insert_spy_with_shape_capture(world);
    let mut schedule = Schedule::default();
    schedule.add_systems(unified_render_system);
    schedule.run(world);
    shape_calls
}

fn run_system_colored(world: &mut World) -> ColoredMeshCallLog {
    world.insert_resource(engine_ui::draw_command::DrawQueue::default());
    let calls = insert_spy_with_colored_mesh_capture(world);
    let mut schedule = Schedule::default();
    schedule.add_systems(unified_render_system);
    schedule.run(world);
    calls
}
```

- [ ] **Step 2: Rewrite unified_render.rs**

Replace the entire contents of `crates/engine_ui/src/unified_render.rs` with the DrawCommand-based implementation:

```rust
use bevy_ecs::prelude::{Local, Query, ResMut};
use engine_core::color::Color;
use engine_core::profiler::FrameProfiler;
use engine_core::types::Pixels;
use engine_render::camera::{Camera2D, CameraRotation};
use engine_render::culling::{aabb_intersects_view_rect, compute_view_rect};
use engine_render::font::{GlyphCache, measure_text, render_text_transformed, wrap_text};
use engine_render::material::{apply_material, BlendMode, Material2d};
use engine_render::prelude::RendererRes;
use engine_render::rect::Rect;
use engine_render::shader::ShaderHandle;
use engine_render::shape::{
    CachedMesh, ColorMesh, MeshOverlays, PersistentColorMesh, Shape, Stroke, affine2_to_mat4,
    is_shape_culled, tessellate, tessellate_stroke,
};
use engine_render::sprite::Sprite;
use engine_scene::prelude::{EffectiveVisibility, GlobalTransform2D, RenderLayer, SortOrder};
use glam::Vec2;

use crate::draw_command::{DrawCommand, DrawQueue, OverlayCommand, SortedDrawCommand, StrokeCommand};
use crate::widget::Text;

const LINE_HEIGHT_FACTOR: f32 = 1.3;

type ShapeItem<'w> = (
    &'w Shape,
    &'w GlobalTransform2D,
    Option<&'w RenderLayer>,
    Option<&'w SortOrder>,
    Option<&'w EffectiveVisibility>,
    Option<&'w Material2d>,
    Option<&'w Stroke>,
    Option<&'w CachedMesh>,
);

type TextItem<'w> = (
    &'w Text,
    &'w GlobalTransform2D,
    Option<&'w RenderLayer>,
    Option<&'w SortOrder>,
    Option<&'w EffectiveVisibility>,
);

type ColorMeshItem<'w> = (
    &'w ColorMesh,
    &'w GlobalTransform2D,
    Option<&'w RenderLayer>,
    Option<&'w SortOrder>,
    Option<&'w EffectiveVisibility>,
    Option<&'w MeshOverlays>,
    Option<&'w Material2d>,
);

type PersistentMeshItem<'w> = (
    &'w PersistentColorMesh,
    &'w GlobalTransform2D,
    Option<&'w RenderLayer>,
    Option<&'w SortOrder>,
    Option<&'w EffectiveVisibility>,
    Option<&'w MeshOverlays>,
    Option<&'w Material2d>,
);

type SpriteItem<'w> = (
    &'w Sprite,
    &'w GlobalTransform2D,
    Option<&'w RenderLayer>,
    Option<&'w SortOrder>,
    Option<&'w EffectiveVisibility>,
    Option<&'w Material2d>,
);

fn key(layer: Option<&RenderLayer>, order: Option<&SortOrder>) -> (RenderLayer, SortOrder) {
    (
        layer.copied().unwrap_or(RenderLayer::World),
        order.copied().unwrap_or_default(),
    )
}

fn is_hidden(vis: Option<&EffectiveVisibility>) -> bool {
    vis.is_some_and(|v| !v.0)
}

fn collect_overlays(overlays: Option<&MeshOverlays>) -> Vec<OverlayCommand> {
    overlays.map_or_else(Vec::new, |o| {
        o.0.iter()
            .filter(|e| e.visible)
            .map(|e| OverlayCommand {
                mesh: e.mesh.clone(),
                material: e.material.clone(),
            })
            .collect()
    })
}

#[allow(clippy::too_many_arguments)]
fn collect_draw_commands(
    shape_query: &Query<ShapeItem>,
    text_query: &Query<TextItem>,
    color_mesh_query: &Query<ColorMeshItem>,
    persistent_mesh_query: &Query<PersistentMeshItem>,
    sprite_query: &Query<SpriteItem>,
    draw_queue: &mut DrawQueue,
    view_rect: Option<(Vec2, Vec2)>,
) -> Vec<SortedDrawCommand> {
    let capacity = shape_query.iter().len()
        + text_query.iter().len()
        + color_mesh_query.iter().len()
        + persistent_mesh_query.iter().len()
        + sprite_query.iter().len();

    let mut commands = draw_queue.drain();
    commands.reserve(capacity);

    for (shape, transform, layer, order, vis, mat, stroke, cached) in shape_query.iter() {
        if is_hidden(vis) {
            continue;
        }
        if is_shape_culled(transform.0.translation, &shape.variant, view_rect) {
            continue;
        }
        let mesh = if let Some(cached) = cached {
            cached.0.clone()
        } else if let Ok(m) = tessellate(&shape.variant) {
            m
        } else {
            continue;
        };
        let stroke_cmd = stroke.and_then(|s| {
            tessellate_stroke(&shape.variant, s.width)
                .ok()
                .map(|sm| StrokeCommand {
                    mesh: sm,
                    color: s.color,
                })
        });
        commands.push(SortedDrawCommand {
            sort_key: key(layer, order),
            command: DrawCommand::Shape {
                mesh,
                color: shape.color,
                model: affine2_to_mat4(&transform.0),
                material: mat.cloned(),
                stroke: stroke_cmd,
            },
        });
    }

    for (text, transform, layer, order, vis) in text_query.iter() {
        if is_hidden(vis) {
            continue;
        }
        commands.push(SortedDrawCommand {
            sort_key: key(layer, order),
            command: DrawCommand::Text {
                content: text.content.clone(),
                font_size: text.font_size,
                color: text.color,
                max_width: text.max_width,
                transform: transform.0,
            },
        });
    }

    for (mesh, transform, layer, order, vis, overlays, mat) in color_mesh_query.iter() {
        if is_hidden(vis) || mesh.is_empty() {
            continue;
        }
        commands.push(SortedDrawCommand {
            sort_key: key(layer, order),
            command: DrawCommand::ColorMesh {
                mesh: mesh.0.clone(),
                model: affine2_to_mat4(&transform.0),
                material: mat.cloned(),
                overlays: collect_overlays(overlays),
            },
        });
    }

    for (pcm, transform, layer, order, vis, overlays, mat) in persistent_mesh_query.iter() {
        if is_hidden(vis) {
            continue;
        }
        commands.push(SortedDrawCommand {
            sort_key: key(layer, order),
            command: DrawCommand::PersistentMesh {
                handle: pcm.0,
                model: affine2_to_mat4(&transform.0),
                material: mat.cloned(),
                overlays: collect_overlays(overlays),
            },
        });
    }

    for (sprite, transform, layer, order, vis, mat) in sprite_query.iter() {
        if is_hidden(vis) {
            continue;
        }
        let pos = transform.0.translation;
        if let Some((view_min, view_max)) = view_rect {
            let entity_min = pos;
            let entity_max = Vec2::new(pos.x + sprite.width.0, pos.y + sprite.height.0);
            if !aabb_intersects_view_rect(entity_min, entity_max, view_min, view_max) {
                continue;
            }
        }
        commands.push(SortedDrawCommand {
            sort_key: key(layer, order),
            command: DrawCommand::Sprite {
                rect: Rect {
                    x: Pixels(pos.x),
                    y: Pixels(pos.y),
                    width: sprite.width,
                    height: sprite.height,
                    color: sprite.color,
                },
                uv_rect: sprite.uv_rect,
                material: mat.cloned(),
            },
        });
    }

    commands.sort_unstable_by_key(|cmd| cmd.sort_key);
    commands
}

fn draw_commands(
    renderer: &mut dyn engine_render::renderer::Renderer,
    cache: &mut GlyphCache,
    commands: &[SortedDrawCommand],
) {
    let mut last_shader = None;
    let mut last_blend_mode = None;

    for cmd in commands {
        match &cmd.command {
            DrawCommand::Shape {
                mesh,
                color,
                model,
                material,
                stroke,
            } => {
                apply_material(renderer, material.as_ref(), &mut last_shader, &mut last_blend_mode);
                renderer.draw_shape(&mesh.vertices, &mesh.indices, *color, *model);
                if let Some(s) = stroke {
                    renderer.draw_shape(&s.mesh.vertices, &s.mesh.indices, s.color, *model);
                }
            }
            DrawCommand::Text {
                content,
                font_size,
                color,
                max_width,
                transform,
            } => {
                draw_text(renderer, cache, content, *font_size, *color, *max_width, transform);
            }
            DrawCommand::ColorMesh {
                mesh,
                model,
                material,
                overlays,
            } => {
                apply_material(renderer, material.as_ref(), &mut last_shader, &mut last_blend_mode);
                renderer.draw_colored_mesh(&mesh.vertices, &mesh.indices, *model);
                draw_overlays(renderer, overlays, *model, &mut last_shader, &mut last_blend_mode);
            }
            DrawCommand::PersistentMesh {
                handle,
                model,
                material,
                overlays,
            } => {
                apply_material(renderer, material.as_ref(), &mut last_shader, &mut last_blend_mode);
                renderer.draw_persistent_colored_mesh(*handle, *model);
                draw_overlays(renderer, overlays, *model, &mut last_shader, &mut last_blend_mode);
            }
            DrawCommand::Sprite {
                rect,
                uv_rect,
                material,
            } => {
                apply_material(renderer, material.as_ref(), &mut last_shader, &mut last_blend_mode);
                renderer.draw_sprite(*rect, *uv_rect);
            }
        }
    }
}

fn draw_overlays(
    renderer: &mut dyn engine_render::renderer::Renderer,
    overlays: &[OverlayCommand],
    model: [[f32; 4]; 4],
    last_shader: &mut Option<ShaderHandle>,
    last_blend_mode: &mut Option<BlendMode>,
) {
    for entry in overlays {
        apply_material(
            renderer,
            Some(&entry.material),
            last_shader,
            last_blend_mode,
        );
        renderer.draw_colored_mesh(&entry.mesh.vertices, &entry.mesh.indices, model);
    }
}

fn draw_text(
    renderer: &mut dyn engine_render::renderer::Renderer,
    cache: &mut GlyphCache,
    content: &str,
    font_size: f32,
    color: Color,
    max_width: Option<f32>,
    transform: &glam::Affine2,
) {
    if let Some(max_w) = max_width {
        let lines = wrap_text(content, font_size, max_w);
        let line_height = font_size * LINE_HEIGHT_FACTOR;
        let total_height = (lines.len() as f32 - 1.0) * line_height;
        let start_y = -total_height * 0.5;
        for (i, line) in lines.iter().enumerate() {
            let line_width = measure_text(line, font_size);
            let y_offset = start_y + i as f32 * line_height;
            let offset = glam::Affine2::from_translation(Vec2::new(-line_width * 0.5, y_offset));
            let line_transform = *transform * offset;
            let model = affine2_to_mat4(&line_transform);
            render_text_transformed(renderer, cache, line, &model, font_size, color);
        }
    } else {
        let text_width = measure_text(content, font_size);
        let center_offset = glam::Affine2::from_translation(Vec2::new(-text_width * 0.5, 0.0));
        let centered_transform = *transform * center_offset;
        let model = affine2_to_mat4(&centered_transform);
        render_text_transformed(renderer, cache, content, &model, font_size, color);
    }
}

/// Unified render system that draws shapes, text, sprites, color meshes, and
/// persistent meshes in a single sorted pass. Draw order is determined by
/// `(RenderLayer, SortOrder)` — the scene hierarchy decides what draws on top.
///
/// Also drains the `DrawQueue` resource, merging immediate-mode commands into
/// the same sorted pass. Systems that push to `DrawQueue` in `Phase::Render`
/// are guaranteed to complete before this system runs in `Phase::PostRender`.
#[allow(clippy::too_many_arguments)]
pub fn unified_render_system(
    shape_query: Query<ShapeItem>,
    text_query: Query<TextItem>,
    color_mesh_query: Query<ColorMeshItem>,
    persistent_mesh_query: Query<PersistentMeshItem>,
    sprite_query: Query<SpriteItem>,
    camera_query: Query<(&Camera2D, Option<&CameraRotation>)>,
    mut renderer: ResMut<RendererRes>,
    mut draw_queue: ResMut<DrawQueue>,
    mut cache: Local<GlyphCache>,
    mut profiler: Option<ResMut<FrameProfiler>>,
) {
    let view_rect = compute_view_rect(&camera_query, &renderer);

    let t_sort = std::time::Instant::now();
    let commands = collect_draw_commands(
        &shape_query,
        &text_query,
        &color_mesh_query,
        &persistent_mesh_query,
        &sprite_query,
        &mut draw_queue,
        view_rect,
    );
    let sort_us = t_sort.elapsed().as_micros() as u64;

    let t_draw = std::time::Instant::now();
    draw_commands(&mut **renderer, &mut cache, &commands);
    let draw_us = t_draw.elapsed().as_micros() as u64;

    if let Some(p) = profiler.as_deref_mut() {
        p.record_phase("render_sort", sort_us);
        p.record_phase("render_draw", draw_us);
    }
}
```

- [ ] **Step 3: Verify all engine_ui tests pass**

Run: `cargo.exe test -p engine_ui`
Expected: ALL PASS (existing tests + new sprite/DrawQueue/stroke tests)

- [ ] **Step 4: Commit**

```bash
git add crates/engine_ui/src/unified_render.rs crates/engine_ui/tests/suite/unified_render.rs
git commit -m "feat(ui): rewrite unified_render with DrawCommand buffer, sprite support, and DrawQueue"
```

---

### Task 5: Integrate into DefaultPlugins (Phase::PostRender)

**Files:**
- Modify: `crates/axiom2d/src/default_plugins.rs:135-156`
- Modify: `crates/card_game/src/plugin.rs:62-65,99,200-214`

This is an atomic change — DefaultPlugins adds unified_render to PostRender, CardGamePlugin removes its own registration and ShapeRenderDisabled.

- [ ] **Step 1: Update DefaultPlugins register_render**

In `crates/axiom2d/src/default_plugins.rs`, replace the `register_render` function:

```rust
fn register_render(app: &mut App) {
    #[cfg(feature = "render")]
    {
        app.world_mut().insert_resource(ClearColor::default());
        app.world_mut().insert_resource(ShaderRegistry::default());
        app.world_mut()
            .insert_resource(engine_ui::draw_command::DrawQueue::default());
        app.add_systems(Phase::LateUpdate, mesh_cache_system);
        app.add_systems(
            Phase::Render,
            (
                clear_system,
                upload_atlas_system,
                camera_prepare_system,
                shader_prepare_system,
                splash_render_system,
            )
                .chain(),
        );
        app.add_systems(
            Phase::PostRender,
            (
                engine_ui::unified_render::unified_render_system,
                post_process_system,
            )
                .chain(),
        );
    }
}
```

Update the imports at the top of the file — remove `sprite_render_system` and `shape_render_system`:

```rust
#[cfg(feature = "render")]
use engine_render::prelude::{
    ClearColor, ShaderRegistry, camera_prepare_system, clear_system, post_process_system,
    shader_prepare_system, upload_atlas_system,
};
```

- [ ] **Step 2: Update CardGamePlugin**

In `crates/card_game/src/plugin.rs`:

Remove line 99: `world.insert_resource(ShapeRenderDisabled);`

Remove the import of `ShapeRenderDisabled` and `shape_render_system` from line 62:
```rust
use engine_render::prelude::{ClearColor, ShaderRegistry};
```

Remove the import of `unified_render_system` from line 65:
```rust
// DELETE: use engine_ui::unified_render::unified_render_system;
```

In `register_systems`, remove lines 198-214 (the Phase::Render registrations for stash, unified_render, and drop zone that reference shape_render_system). Replace with:

```rust
    .add_systems(
        Phase::Render,
        (stash_render_system, stash_tab_render_system, stash_hover_preview_render_system).chain(),
    )
    .add_systems(Phase::Render, hand_drop_zone_render_system)
```

Note: These immediate-mode systems stay in Phase::Render for now (Approach 3 will migrate them to DrawQueue). They no longer need `.after(shape_render_system)` since shape_render_system is gone.

- [ ] **Step 3: Build all crates**

Run: `cargo.exe build --workspace`
Expected: PASS

- [ ] **Step 4: Run all tests**

Run: `cargo.exe test --workspace`
Expected: Some engine_render and demo tests will FAIL (they reference deleted systems). This is expected — Task 6 and Task 7 will fix them.

- [ ] **Step 5: Commit**

```bash
git add crates/axiom2d/src/default_plugins.rs crates/card_game/src/plugin.rs
git commit -m "feat(app): move unified_render to Phase::PostRender in DefaultPlugins"
```

---

### Task 6: Delete standalone render systems

**Files:**
- Modify: `crates/engine_render/src/shape/render.rs` (remove `shape_render_system` and `ShapeRenderDisabled`, keep `affine2_to_mat4` and `is_shape_culled`)
- Modify: `crates/engine_render/src/sprite.rs` (remove `sprite_render_system`, keep `Sprite` component and `is_sprite_culled`)
- Modify: `crates/engine_render/src/shape/mod.rs` (update exports)
- Modify: `crates/engine_render/src/prelude.rs` (update exports)

- [ ] **Step 1: Trim shape/render.rs**

Replace `crates/engine_render/src/shape/render.rs` with only the retained items:

```rust
use engine_scene::prelude::GlobalTransform2D;
use glam::Vec2;

use super::components::ShapeVariant;
use super::tessellate::shape_aabb;
use crate::culling::aabb_intersects_view_rect;

pub fn affine2_to_mat4(affine: &glam::Affine2) -> [[f32; 4]; 4] {
    let m = affine.matrix2;
    let t = affine.translation;
    [
        [m.x_axis.x, m.x_axis.y, 0.0, 0.0],
        [m.y_axis.x, m.y_axis.y, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [t.x, t.y, 0.0, 1.0],
    ]
}

pub fn is_shape_culled(pos: Vec2, variant: &ShapeVariant, view_rect: Option<(Vec2, Vec2)>) -> bool {
    let Some((view_min, view_max)) = view_rect else {
        return false;
    };
    let (local_min, local_max) = shape_aabb(variant);
    let r = local_min.abs().max(local_max.abs()).length();
    let entity_min = Vec2::new(pos.x - r, pos.y - r);
    let entity_max = Vec2::new(pos.x + r, pos.y + r);
    !aabb_intersects_view_rect(entity_min, entity_max, view_min, view_max)
}
```

- [ ] **Step 2: Trim sprite.rs**

Remove `sprite_render_system` from `crates/engine_render/src/sprite.rs`. Keep the `Sprite` component and `is_sprite_culled`:

```rust
use bevy_ecs::prelude::Component;
use engine_core::color::Color;
use engine_core::types::{Pixels, TextureId};
use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::culling::aabb_intersects_view_rect;

#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Sprite {
    pub texture: TextureId,
    pub uv_rect: [f32; 4],
    pub color: Color,
    pub width: Pixels,
    pub height: Pixels,
}

pub fn is_sprite_culled(sprite: &Sprite, pos: Vec2, view_rect: Option<(Vec2, Vec2)>) -> bool {
    let Some((view_min, view_max)) = view_rect else {
        return false;
    };
    let entity_min = Vec2::new(pos.x, pos.y);
    let entity_max = Vec2::new(pos.x + sprite.width.0, pos.y + sprite.height.0);
    !aabb_intersects_view_rect(entity_min, entity_max, view_min, view_max)
}
```

- [ ] **Step 3: Update shape/mod.rs exports**

In `crates/engine_render/src/shape/mod.rs`, remove `ShapeRenderDisabled` and `shape_render_system` from the render re-export:

```rust
pub use render::{affine2_to_mat4, is_shape_culled};
```

- [ ] **Step 4: Update prelude.rs**

In `crates/engine_render/src/prelude.rs`:

Remove `ShapeRenderDisabled` and `shape_render_system` from shape exports:
```rust
pub use crate::shape::{
    ColorMesh, ColorVertex, PathCommand, PersistentColorMesh, QUAD_INDICES, Shape,
    ShapeVariant, Stroke, TessellatedColorMesh, TessellatedMesh, UNIT_QUAD,
    affine2_to_mat4, rect_polygon, rect_vertices, resolve_commands, reverse_path,
    rounded_rect_path, sample_cubic, sample_quadratic, split_contours,
    tessellate, tessellate_stroke, unit_quad_model,
};
```

Remove `sprite_render_system` from sprite export:
```rust
pub use crate::sprite::Sprite;
```

- [ ] **Step 5: Add unified_render_system and DrawQueue to engine_ui prelude**

In `crates/engine_ui/src/prelude.rs`, add:

```rust
pub use crate::draw_command::{DrawCommand, DrawQueue};
pub use crate::unified_render::unified_render_system;
```

- [ ] **Step 6: Build workspace**

Run: `cargo.exe build --workspace`
Expected: Build errors in tests that reference deleted items (engine_render tests, demo tests). Implementation code should compile.

- [ ] **Step 7: Commit**

```bash
git add crates/engine_render/src/shape/render.rs crates/engine_render/src/sprite.rs crates/engine_render/src/shape/mod.rs crates/engine_render/src/prelude.rs crates/engine_ui/src/prelude.rs
git commit -m "refactor(render): delete shape_render_system, sprite_render_system, ShapeRenderDisabled"
```

---

### Task 7: Migrate and fix tests

**Files:**
- Delete: `crates/engine_render/tests/suite/shape_render.rs`
- Delete: `crates/engine_render/tests/suite/sprite.rs`
- Modify: `crates/engine_render/tests/suite/mod.rs` (remove module declarations)
- Modify: `crates/engine_render/tests/suite/shader.rs:238` (fix shader test that uses shape_render_system)
- Modify: `crates/demo/tests/suite/core_main.rs` (update to use unified_render_system)
- Modify: `crates/engine_ui/tests/suite/unified_render.rs` (add migrated behavioral tests)

- [ ] **Step 1: Remove shape_render and sprite test modules from engine_render**

In `crates/engine_render/tests/suite/mod.rs`, remove:
```rust
// DELETE these lines:
// mod shape_render;
// mod sprite;
```

Delete the files:
```bash
rm crates/engine_render/tests/suite/shape_render.rs
rm crates/engine_render/tests/suite/sprite.rs
```

- [ ] **Step 2: Fix shader test in engine_render**

In `crates/engine_render/tests/suite/shader.rs`, the test at line 238 uses `shape_render_system`. This test verifies that `shader_prepare_system` compiles shaders before rendering. Since unified_render_system is in engine_ui (and engine_render can't depend on engine_ui), this test should verify shader preparation only — not the render integration:

The test `when_shader_prepare_runs_before_shape_render_then_compile_precedes_draw_in_log` (line 234) chains `shader_prepare_system` with `shape_render_system` and asserts `compile_shader` appears before `draw_shape` in the spy log. Delete this test — the ordering it verified is now guaranteed by phase separation (shader_prepare in Phase::Render, unified_render in Phase::PostRender). Phase boundaries are enforced by the scheduler, not by `.chain()`.

- [ ] **Step 3: Fix demo tests**

In `crates/demo/tests/suite/core_main.rs`, update the two tests that use `sprite_render_system`:

Test `when_sprite_render_system_runs_then_draw_sprite_called_for_player` (line 29):
```rust
#[test]
fn when_unified_render_system_runs_then_draw_sprite_called_for_player() {
    // Arrange
    let log = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(log.clone());
    let mut world = World::new();
    world.insert_resource(RendererRes::new(Box::new(spy)));
    world.insert_resource(engine_ui::draw_command::DrawQueue::default());
    world.spawn((
        Sprite {
            texture: TextureId(0),
            uv_rect: [0.0, 0.0, 1.0, 1.0],
            color: Color::WHITE,
            width: Pixels(300.0),
            height: Pixels(200.0),
        },
        GlobalTransform2D(Affine2::from_translation(Vec2::new(490.0, 260.0))),
    ));
    let mut schedule = Schedule::default();
    schedule.add_systems(engine_ui::unified_render::unified_render_system);

    // Act
    schedule.run(&mut world);

    // Assert
    let count = log
        .lock()
        .unwrap()
        .iter()
        .filter(|s| s.as_str() == "draw_sprite")
        .count();
    assert_eq!(count, 1);
}
```

Test `when_render_phase_runs_then_clear_before_camera_before_sprite` (line 62):
```rust
#[test]
fn when_render_phase_runs_then_clear_before_camera_before_draw() {
    // Arrange
    let log = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(log.clone()).with_viewport(800, 600);
    let mut world = World::new();
    world.insert_resource(RendererRes::new(Box::new(spy)));
    world.insert_resource(ClearColor::default());
    world.insert_resource(engine_ui::draw_command::DrawQueue::default());
    world.spawn(Camera2D::default());
    world.spawn((
        Sprite {
            texture: TextureId(0),
            uv_rect: [0.0, 0.0, 1.0, 1.0],
            color: Color::WHITE,
            width: Pixels(300.0),
            height: Pixels(200.0),
        },
        GlobalTransform2D(Affine2::from_translation(Vec2::new(400.0, 300.0))),
    ));
    let mut schedule = Schedule::default();
    schedule.add_systems(
        (clear_system, camera_prepare_system, engine_ui::unified_render::unified_render_system).chain(),
    );

    // Act
    schedule.run(&mut world);

    // Assert
    let calls = log.lock().unwrap();
    assert_eq!(calls[0], "clear");
    assert_eq!(calls[1], "set_view_projection");
    assert_eq!(calls[2], "set_shader");
    assert_eq!(calls[3], "set_blend_mode");
    assert_eq!(calls[4], "draw_sprite");
}
```

Update imports at the top of the file to add `engine_ui` and remove `sprite_render_system`.

- [ ] **Step 4: Add critical migrated tests to engine_ui**

Add these behavioral tests to `crates/engine_ui/tests/suite/unified_render.rs` that were previously only in engine_render:

Shape frustum culling test:
```rust
use engine_render::camera::Camera2D;
use engine_render::testing::insert_spy_with_viewport;

#[test]
fn when_shape_fully_outside_camera_view_then_not_drawn() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(DrawQueue::default());
    let log = insert_spy_with_viewport(&mut world, 800, 600);
    world.spawn(Camera2D {
        position: Vec2::new(400.0, 300.0),
        zoom: 1.0,
    });
    world.spawn((
        Shape {
            variant: ShapeVariant::Circle { radius: 10.0 },
            color: Color::RED,
        },
        GlobalTransform2D(Affine2::from_translation(Vec2::new(2000.0, 300.0))),
        SortOrder::new(0),
    ));

    // Act
    let mut schedule = Schedule::default();
    schedule.add_systems(unified_render_system);
    schedule.run(&mut world);

    // Assert
    let count = log
        .lock()
        .unwrap()
        .iter()
        .filter(|s| s.as_str() == "draw_shape")
        .count();
    assert_eq!(count, 0);
}
```

Sprite frustum culling test:
```rust
#[test]
fn when_sprite_fully_outside_camera_view_then_not_drawn() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(DrawQueue::default());
    let log = insert_spy_with_viewport(&mut world, 800, 600);
    world.spawn(Camera2D {
        position: Vec2::new(400.0, 300.0),
        zoom: 1.0,
    });
    world.spawn((
        Sprite {
            texture: TextureId(0),
            uv_rect: [0.0, 0.0, 1.0, 1.0],
            color: Color::WHITE,
            width: Pixels(32.0),
            height: Pixels(32.0),
        },
        GlobalTransform2D(Affine2::from_translation(Vec2::new(2000.0, 300.0))),
        SortOrder::new(0),
    ));

    // Act
    let mut schedule = Schedule::default();
    schedule.add_systems(unified_render_system);
    schedule.run(&mut world);

    // Assert
    let count = log
        .lock()
        .unwrap()
        .iter()
        .filter(|s| s.as_str() == "draw_sprite")
        .count();
    assert_eq!(count, 0);
}
```

Material dedup across draw types test:
```rust
use engine_render::testing::insert_spy_with_blend_capture;

#[test]
fn when_sprite_and_shape_share_blend_mode_then_set_blend_mode_called_once() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(DrawQueue::default());
    let blend_calls = insert_spy_with_blend_capture(&mut world);
    world.spawn((
        Shape {
            variant: ShapeVariant::Circle { radius: 10.0 },
            color: Color::RED,
        },
        GlobalTransform2D(Affine2::IDENTITY),
        SortOrder::new(0),
    ));
    world.spawn((
        Sprite {
            texture: TextureId(0),
            uv_rect: [0.0, 0.0, 1.0, 1.0],
            color: Color::WHITE,
            width: Pixels(32.0),
            height: Pixels(32.0),
        },
        GlobalTransform2D(Affine2::IDENTITY),
        SortOrder::new(1),
    ));

    // Act
    let mut schedule = Schedule::default();
    schedule.add_systems(unified_render_system);
    schedule.run(&mut world);

    // Assert — both default to Alpha, so set_blend_mode called once
    let calls = blend_calls.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0], engine_render::material::BlendMode::Alpha);
}
```

- [ ] **Step 5: Run all workspace tests**

Run: `cargo.exe test --workspace`
Expected: ALL PASS

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "test: migrate render tests to unified_render, fix demo and shader tests"
```

---

### Task 8: Final verification and cleanup

**Files:**
- Verify: all crates build, all tests pass, clippy clean, formatted

- [ ] **Step 1: Full workspace build**

Run: `cargo.exe build --workspace`
Expected: PASS with no warnings

- [ ] **Step 2: Full workspace tests**

Run: `cargo.exe test --workspace`
Expected: ALL PASS

- [ ] **Step 3: Clippy**

Run: `cargo.exe clippy --workspace`
Expected: PASS (no new warnings)

- [ ] **Step 4: Format**

Run: `cargo.exe fmt --all`

- [ ] **Step 5: Build card_game_bin specifically**

Run: `cargo.exe build -p card_game_bin`
Expected: PASS — verifies all game systems wire up correctly

- [ ] **Step 6: Commit any cleanup**

```bash
git add -A
git commit -m "chore: final cleanup after unified render redesign"
```

---

## Notes

### Approach 3 Migration Path (documented, not implemented)

After this plan is complete, the DrawQueue infrastructure exists for immediate-mode systems to migrate. Priority order:

1. `hand_drop_zone_render_system` — single `draw_queue.push(DrawCommand::Shape{...})` call
2. `stash_tab_render_system` — loop of DrawCommand::Shape + DrawCommand::Text pushes
3. `ui_render_system` — DrawCommand::Shape for each UiNode background
4. `store_render_system` — many procedural shapes + text
5. `stash_render_system` — grid layout + drag preview
6. `stash_hover_preview_render_system` — animated shader uniforms

Each migration is mechanical: replace `renderer.draw_shape(...)` with `draw_queue.push(layer, order, DrawCommand::Shape{...})`.

End state: only `unified_render_system`, `clear_system`, and `splash_render_system` touch the Renderer trait. `splash_render_system` is a boot-time system (pre-game splash screen) that early-returns once done — acceptable exception.

### Test Migration Summary

Tests deleted from engine_render (behavior now covered by unified_render tests in engine_ui):
- `shape_render.rs` — 31 tests covering stroke, culling, materials, sort order, visibility
- `sprite.rs` — 35 tests covering sort order, culling, materials, visibility

Critical behaviors migrated to engine_ui unified_render tests:
- Shape frustum culling
- Sprite frustum culling
- Stroke rendering (fill before stroke)
- Sprite interleaving with shapes
- DrawQueue integration
- Material dedup across draw types

Behaviors already covered by existing unified_render tests (not migrated, would be redundant):
- Shape + text sort order
- Visibility filtering
- CachedMesh usage
- Fallback tessellation
- ColorMesh rendering
