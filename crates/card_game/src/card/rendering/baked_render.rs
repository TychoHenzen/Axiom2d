use bevy_ecs::prelude::Query;
use engine_render::shape::{ColorMesh, MeshOverlays, TessellatedColorMesh};

use super::baked_mesh::BakedCardMesh;
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
