use bevy_ecs::prelude::Query;
use engine_scene::prelude::{Children, RenderLayer};

use crate::card::component::Card;
use crate::card::component::CardFaceSide;

/// Propagates the parent card's `RenderLayer` to all card-face children.
///
/// When a card transitions between zones (Table → Hand → Stash), its
/// `RenderLayer` changes but children spawned by `spawn_visual_card` keep
/// their original layer. This system syncs them each frame.
pub fn card_render_layer_system(
    cards: Query<(&RenderLayer, &Children), bevy_ecs::prelude::With<Card>>,
    mut faces: Query<
        &mut RenderLayer,
        (
            bevy_ecs::prelude::With<CardFaceSide>,
            bevy_ecs::prelude::Without<Card>,
        ),
    >,
) {
    for (layer, children) in &cards {
        for &child in &children.0 {
            if let Ok(mut child_layer) = faces.get_mut(child)
                && *child_layer != *layer
            {
                *child_layer = *layer;
            }
        }
    }
}
