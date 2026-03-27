use bevy_ecs::prelude::{Changed, Or, Query};
use engine_render::shape::{ColorMesh, MeshOverlays, TessellatedColorMesh};

use super::baked_mesh::BakedCardMesh;
use crate::card::component::Card;
use crate::card::component::CardItemForm;

/// Syncs `BakedCardMesh` → `ColorMesh` based on `card.face_up`.
/// Also hides/shows `MeshOverlays` (art shader) — overlays only render face-up.
/// Runs when `Card` or `BakedCardMesh` changes so the unified render system
/// always has the correct face mesh to draw.
pub fn baked_card_sync_system(
    mut query: Query<
        (
            &BakedCardMesh,
            &Card,
            &mut ColorMesh,
            Option<&mut MeshOverlays>,
            Option<&CardItemForm>,
        ),
        Or<(Changed<Card>, Changed<BakedCardMesh>)>,
    >,
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
                entry.visible = card.face_up;
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use bevy_ecs::prelude::*;
    use engine_render::shape::{ColorMesh, MeshOverlays, OverlayEntry};
    use glam::Vec2;

    use super::*;
    use crate::card::component::CardItemForm;
    use crate::card::component::CardLabel;
    use crate::card::identity::signature::CardSignature;
    use crate::card::rendering::bake::{bake_back_face, bake_front_face};

    fn make_baked() -> BakedCardMesh {
        let label = CardLabel {
            name: "Test".to_owned(),
            description: "Desc".to_owned(),
        };
        let size = Vec2::new(60.0, 90.0);
        let sig = CardSignature::default();
        BakedCardMesh {
            front: bake_front_face(&sig, size, &label, None),
            back: bake_back_face(size),
        }
    }

    fn make_card(face_up: bool) -> Card {
        Card {
            face_texture: engine_core::prelude::TextureId(0),
            back_texture: engine_core::prelude::TextureId(0),
            face_up,
            signature: CardSignature::default(),
        }
    }

    fn run(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(baked_card_sync_system);
        schedule.run(world);
    }

    #[test]
    fn when_face_up_then_color_mesh_matches_front() {
        // Arrange
        let mut world = World::new();
        let baked = make_baked();
        let expected_len = baked.front.vertices.len();
        world.spawn((baked, make_card(true), ColorMesh::default()));

        // Act
        run(&mut world);

        // Assert
        let mut q = world.query::<&ColorMesh>();
        let mesh = q.single(&world).unwrap();
        assert_eq!(mesh.vertices.len(), expected_len);
    }

    #[test]
    fn when_card_has_item_form_then_color_mesh_is_empty() {
        // Arrange
        let mut world = World::new();
        world.spawn((
            make_baked(),
            make_card(true),
            ColorMesh::default(),
            CardItemForm,
        ));

        // Act
        run(&mut world);

        // Assert
        let mut q = world.query::<&ColorMesh>();
        let mesh = q.single(&world).unwrap();
        assert!(
            mesh.is_empty(),
            "item-form card ColorMesh must have no vertices"
        );
    }

    #[test]
    fn when_card_has_item_form_then_overlays_hidden() {
        // Arrange
        let mut world = World::new();
        let overlay = MeshOverlays(vec![OverlayEntry {
            mesh: engine_render::shape::TessellatedColorMesh::new(),
            material: engine_render::material::Material2d::default(),
            visible: true,
        }]);
        world.spawn((
            make_baked(),
            make_card(true),
            ColorMesh::default(),
            overlay,
            CardItemForm,
        ));

        // Act
        run(&mut world);

        // Assert
        let mut q = world.query::<&MeshOverlays>();
        let overlays = q.single(&world).unwrap();
        assert!(
            overlays.0.iter().all(|e| !e.visible),
            "item-form card overlays must all be hidden"
        );
    }

    #[test]
    fn when_card_has_item_form_face_down_then_color_mesh_is_empty() {
        // Arrange
        let mut world = World::new();
        world.spawn((
            make_baked(),
            make_card(false),
            ColorMesh::default(),
            CardItemForm,
        ));

        // Act
        run(&mut world);

        // Assert
        let mut q = world.query::<&ColorMesh>();
        let mesh = q.single(&world).unwrap();
        assert!(
            mesh.is_empty(),
            "item-form face-down card must have no vertices"
        );
    }

    #[test]
    fn when_item_form_removed_then_mesh_restored_on_next_sync() {
        // Arrange — spawn card with ItemForm, run system to clear mesh
        let mut world = World::new();
        let baked = make_baked();
        let expected_len = baked.front.vertices.len();
        let entity = world
            .spawn((baked, make_card(true), ColorMesh::default(), CardItemForm))
            .id();
        run(&mut world);
        // Confirm mesh is empty after first sync
        let mut q = world.query::<&ColorMesh>();
        assert!(q.get(&world, entity).unwrap().is_empty());

        // Act — remove ItemForm and touch Card to trigger change detection
        world.entity_mut(entity).remove::<CardItemForm>();
        world.entity_mut(entity).get_mut::<Card>().unwrap().face_up = true;
        run(&mut world);

        // Assert
        let mut q = world.query::<&ColorMesh>();
        let mesh = q.get(&world, entity).unwrap();
        assert_eq!(
            mesh.vertices.len(),
            expected_len,
            "mesh must be restored after leaving stash"
        );
    }

    #[test]
    fn when_face_down_then_color_mesh_matches_back() {
        // Arrange
        let mut world = World::new();
        let baked = make_baked();
        let expected_len = baked.back.vertices.len();
        world.spawn((baked, make_card(false), ColorMesh::default()));

        // Act
        run(&mut world);

        // Assert
        let mut q = world.query::<&ColorMesh>();
        let mesh = q.single(&world).unwrap();
        assert_eq!(mesh.vertices.len(), expected_len);
    }
}
