use bevy_ecs::prelude::{Changed, Or, Query};
use engine_render::shape::ColorMesh;

use super::baked_mesh::BakedCardMesh;
use super::component::Card;

/// Syncs `BakedCardMesh` → `ColorMesh` based on `card.face_up`.
/// Runs when `Card` or `BakedCardMesh` changes so the unified render system
/// always has the correct face mesh to draw.
pub fn baked_card_sync_system(
    mut query: Query<
        (&BakedCardMesh, &Card, &mut ColorMesh),
        Or<(Changed<Card>, Changed<BakedCardMesh>)>,
    >,
) {
    for (baked, card, mut mesh) in &mut query {
        let face = if card.face_up {
            &baked.front
        } else {
            &baked.back
        };
        if mesh.0 != *face {
            mesh.0.clone_from(face);
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use bevy_ecs::prelude::*;
    use engine_render::shape::ColorMesh;
    use glam::Vec2;

    use super::*;
    use crate::card::bake::{bake_back_face, bake_front_face};
    use crate::card::label::CardLabel;
    use crate::card::signature::CardSignature;

    fn make_baked() -> BakedCardMesh {
        let label = CardLabel {
            name: "Test".to_owned(),
            description: "Desc".to_owned(),
        };
        let size = Vec2::new(60.0, 90.0);
        let sig = CardSignature::default();
        BakedCardMesh {
            front: bake_front_face(&sig, size, &label),
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
