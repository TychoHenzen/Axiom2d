use bevy_ecs::prelude::Query;
use engine_scene::prelude::{Children, RenderLayer};

use crate::card::Card;
use crate::card_face_side::CardFaceSide;

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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use bevy_ecs::prelude::*;
    use engine_core::prelude::TextureId;
    use engine_scene::prelude::{ChildOf, RenderLayer};

    use super::*;
    use crate::card::Card;
    use crate::card_face_side::CardFaceSide;

    fn run_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(
            (
                engine_scene::prelude::hierarchy_maintenance_system,
                card_render_layer_system,
            )
                .chain(),
        );
        schedule.run(world);
    }

    fn spawn_card_with_face(world: &mut World, parent_layer: RenderLayer) -> (Entity, Entity) {
        let card = Card::face_down(TextureId(1), TextureId(2));
        let root = world
            .spawn((card, parent_layer, engine_scene::prelude::SortOrder(0)))
            .id();
        let face = world
            .spawn((
                ChildOf(root),
                CardFaceSide::Front,
                RenderLayer::World,
                engine_scene::prelude::SortOrder(0),
            ))
            .id();
        (root, face)
    }

    #[test]
    fn when_parent_ui_and_child_world_then_child_becomes_ui() {
        // Arrange
        let mut world = World::new();
        let (_, face) = spawn_card_with_face(&mut world, RenderLayer::UI);

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(*world.get::<RenderLayer>(face).unwrap(), RenderLayer::UI);
    }

    #[test]
    fn when_parent_world_and_child_world_then_child_stays_world() {
        // Arrange
        let mut world = World::new();
        let (_, face) = spawn_card_with_face(&mut world, RenderLayer::World);

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(*world.get::<RenderLayer>(face).unwrap(), RenderLayer::World);
    }

    #[test]
    fn when_parent_changes_layer_then_child_follows() {
        // Arrange
        let mut world = World::new();
        let (root, face) = spawn_card_with_face(&mut world, RenderLayer::World);
        run_system(&mut world);
        assert_eq!(*world.get::<RenderLayer>(face).unwrap(), RenderLayer::World);

        // Act
        *world.get_mut::<RenderLayer>(root).unwrap() = RenderLayer::UI;
        run_system(&mut world);

        // Assert
        assert_eq!(*world.get::<RenderLayer>(face).unwrap(), RenderLayer::UI);
    }

    #[test]
    fn when_child_has_no_card_face_side_then_not_affected() {
        // Arrange — non-face child (e.g., StashIcon) keeps its own layer
        let mut world = World::new();
        let card = Card::face_down(TextureId(1), TextureId(2));
        let root = world
            .spawn((
                card,
                RenderLayer::World,
                engine_scene::prelude::SortOrder(0),
            ))
            .id();
        let non_face = world
            .spawn((
                ChildOf(root),
                RenderLayer::UI,
                engine_scene::prelude::SortOrder(0),
            ))
            .id();

        // Act
        run_system(&mut world);

        // Assert — no CardFaceSide, so layer unchanged
        assert_eq!(
            *world.get::<RenderLayer>(non_face).unwrap(),
            RenderLayer::UI
        );
    }
}
