use bevy_ecs::prelude::{Query, ResMut};
use engine_render::material::apply_material;
use engine_render::prelude::RendererRes;
use engine_render::shape::{ColorMesh, MeshOverlays, TessellatedColorMesh, affine2_to_mat4};
use engine_scene::prelude::{EffectiveVisibility, GlobalTransform2D};

use super::baked_mesh::BakedCardMesh;
use super::gpu_card_mesh::GpuCardMesh;
use crate::card::component::Card;
use crate::card::component::CardItemForm;

/// Syncs `BakedCardMesh` → `ColorMesh` based on `card.face_up`.
/// Also hides/shows `MeshOverlays` (art shader) — overlays only render face-up.
/// Runs every frame so that cards with `CardItemForm` always have their mesh
/// zeroed — a change-detection filter would miss cards whose `CardItemForm`
/// was inserted via deferred commands in a prior phase.
#[allow(clippy::type_complexity)]
pub fn baked_card_sync_system(
    mut query: Query<(
        &BakedCardMesh,
        &Card,
        &mut ColorMesh,
        Option<&mut MeshOverlays>,
        Option<&CardItemForm>,
    )>,
) {
    for (baked, card, mut mesh, overlays, item_form) in &mut query {
        if item_form.is_some() {
            mesh.0 = TessellatedColorMesh::default();
            if let Some(mut overlays) = overlays {
                for entry in &mut overlays.0 {
                    entry.visible = false;
                }
            }
            continue;
        }
        let face = if card.face_up {
            &baked.front
        } else {
            &baked.back
        };
        if mesh.0 != *face {
            mesh.0.clone_from(face);
        }
        if let Some(mut overlays) = overlays {
            for entry in &mut overlays.0 {
                entry.visible = !entry.front_only || card.face_up;
            }
        }
    }
}

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
