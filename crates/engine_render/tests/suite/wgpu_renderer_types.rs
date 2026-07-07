#![allow(clippy::unwrap_used)]

// Source file crates/engine_render/src/wgpu_renderer/types.rs has zero `pub` items.
// Every type is pub(crate) or pub(super) — no API surface accessible from test crate.
// All types are tested indirectly through renderer integration tests.
//
// If types are promoted to `pub`, add tests here for:
//   - QuadVertex, ShapeVertex, Instance memory layout (bytemuck Pod/Zeroable)
//   - ShapeBatch push/index_count/clear/is_empty
//   - rect_to_instance output shape
//   - compute_batch_ranges correctness
//   - blend_mode_to_blend_state mappings
//   - TextureData and create_texture_bind_group
