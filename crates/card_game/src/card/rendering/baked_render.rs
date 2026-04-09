use bevy_ecs::prelude::{Commands, Entity, Query, With, Without};
use engine_render::shape::{MeshOverlays, PersistentColorMesh};

use super::gpu_card_mesh::GpuCardMesh;
use crate::card::component::Card;
use crate::card::component::CardItemForm;

/// Keeps the `PersistentColorMesh` handle in sync with the card's face-up state
/// and manages overlay visibility for front-only overlays. This replaces the old
/// `baked_card_render_system` — cards now render through `unified_render_system`
/// via `PersistentColorMesh`, participating in proper `SortOrder`-based z-sorting.
#[allow(clippy::type_complexity)]
pub fn sync_card_persistent_mesh(
    mut active: Query<
        (
            &Card,
            &GpuCardMesh,
            &mut PersistentColorMesh,
            Option<&mut MeshOverlays>,
        ),
        Without<CardItemForm>,
    >,
    needs_pcm: Query<
        (Entity, &Card, &GpuCardMesh),
        (Without<PersistentColorMesh>, Without<CardItemForm>),
    >,
    remove_pcm: Query<Entity, (With<PersistentColorMesh>, With<CardItemForm>)>,
    mut commands: Commands,
) {
    for (card, gpu, mut pcm, overlays) in &mut active {
        let target = if card.face_up { gpu.front } else { gpu.back };
        pcm.0 = target;
        if let Some(mut overlays) = overlays {
            for entry in &mut overlays.0 {
                if entry.front_only {
                    entry.visible = card.face_up;
                }
            }
        }
    }
    for (entity, card, gpu) in &needs_pcm {
        let handle = if card.face_up { gpu.front } else { gpu.back };
        commands.entity(entity).insert(PersistentColorMesh(handle));
    }
    for entity in &remove_pcm {
        commands.entity(entity).remove::<PersistentColorMesh>();
    }
}
