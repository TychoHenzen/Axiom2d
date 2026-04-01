#![allow(clippy::unwrap_used)]

use engine_core::types::TextureId;
use engine_render::wgpu_renderer::renderer::{
    PackedTextureBinding, PendingMaterialBindings, pack_material_bindings,
};
use std::collections::HashMap;

#[test]
fn when_material_bindings_recorded_then_take_returns_uniforms_and_textures_in_order() {
    let mut pending = PendingMaterialBindings::default();
    pending.set_uniforms(&[1, 2, 3]);
    pending.bind_texture(TextureId(7), 0);
    pending.bind_texture(TextureId(9), 2);

    let uniforms = pending.take_uniforms();
    let textures = pending.take_textures();

    assert_eq!(uniforms, vec![1, 2, 3]);
    assert_eq!(textures, vec![(TextureId(7), 0), (TextureId(9), 2)]);
}

#[test]
fn when_material_bindings_taken_then_subsequent_take_is_empty() {
    let mut pending = PendingMaterialBindings::default();
    pending.set_uniforms(&[4, 5]);
    pending.bind_texture(TextureId(11), 1);

    let _ = pending.take_uniforms();
    let _ = pending.take_textures();

    assert!(pending.take_uniforms().is_empty());
    assert!(pending.take_textures().is_empty());
}

#[test]
fn when_material_bindings_packed_then_texture_lookup_data_appended_after_uniforms() {
    let mut lookups = HashMap::new();
    lookups.insert(TextureId(7), [0.1, 0.2, 0.3, 0.4]);
    let packed = pack_material_bindings(&[9, 8, 7], &[(TextureId(7), 2)], &lookups);

    let tex = bytemuck::from_bytes::<PackedTextureBinding>(&packed[32..64]);
    assert_eq!(tex.texture_id, 7);
    assert_eq!(tex.binding, 2);
    assert_eq!(tex.uv_rect, [0.1, 0.2, 0.3, 0.4]);
}
