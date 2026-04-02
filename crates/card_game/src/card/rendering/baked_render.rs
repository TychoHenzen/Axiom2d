use bevy_ecs::prelude::{Query, ResMut};
use engine_render::material::apply_material;
use engine_render::prelude::RendererRes;
use engine_render::shape::{MeshOverlays, affine2_to_mat4};
use engine_scene::prelude::{EffectiveVisibility, GlobalTransform2D};

use super::gpu_card_mesh::GpuCardMesh;
use crate::card::component::Card;
use crate::card::component::CardItemForm;

/// Draws cards that have a `GpuCardMesh` component using persistent GPU buffers.
/// Cards with `CardItemForm` are skipped — they are rendered as stash grid slots,
/// not as full-size world cards. Cards with `EffectiveVisibility(false)` are also skipped.
#[allow(clippy::type_complexity)]
pub fn baked_card_render_system(
    query: Query<(
        &Card,
        &GpuCardMesh,
        &GlobalTransform2D,
        Option<&EffectiveVisibility>,
        Option<&MeshOverlays>,
        Option<&CardItemForm>,
    )>,
    mut renderer: ResMut<RendererRes>,
) {
    let mut last_shader = None;
    let mut last_blend_mode = None;

    for (card, gpu_mesh, transform, vis, overlays, item_form) in query.iter() {
        if item_form.is_some() {
            continue;
        }
        if vis.is_some_and(|v| !v.0) {
            continue;
        }
        apply_material(
            &mut **renderer,
            None,
            &mut last_shader,
            &mut last_blend_mode,
        );
        let model = affine2_to_mat4(&transform.0);
        let handle = if card.face_up {
            gpu_mesh.front
        } else {
            gpu_mesh.back
        };
        renderer.draw_persistent_colored_mesh(handle, model);

        if let Some(overlays) = overlays {
            for entry in &overlays.0 {
                if !entry.visible {
                    continue;
                }
                if entry.front_only && !card.face_up {
                    continue;
                }
                apply_material(
                    &mut **renderer,
                    Some(&entry.material),
                    &mut last_shader,
                    &mut last_blend_mode,
                );
                renderer.draw_colored_mesh(&entry.mesh.vertices, &entry.mesh.indices, model);
            }
        }
    }
}
