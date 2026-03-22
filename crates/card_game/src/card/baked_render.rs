use bevy_ecs::prelude::{Changed, Or, Query, ResMut};
use engine_render::material::apply_material;
use engine_render::prelude::RendererRes;
use engine_render::shape::{ColorMesh, affine2_to_mat4};
use engine_scene::prelude::GlobalTransform2D;

use super::baked_mesh::{BakedCardMesh, CardOverlays};
use super::component::Card;
use super::visual_params::generate_card_visuals;

const QUAD_INDICES: [u32; 6] = [0, 1, 2, 0, 2, 3];

/// Draws art shader overlays on top of baked card meshes.
/// Runs after `unified_render_system` in the Render phase.
pub fn card_art_overlay_system(
    query: Query<(&Card, &CardOverlays, &GlobalTransform2D)>,
    mut renderer: ResMut<RendererRes>,
) {
    let mut last_shader = None;
    let mut last_blend_mode = None;

    for (card, overlays, transform) in &query {
        if !card.face_up {
            continue;
        }
        let Some(art) = &overlays.art else {
            continue;
        };
        let visuals = generate_card_visuals(&card.signature);
        let color = engine_core::color::Color {
            r: visuals.art_color.r,
            g: visuals.art_color.g,
            b: visuals.art_color.b,
            a: visuals.art_color.a,
        };
        apply_material(
            &mut **renderer,
            Some(&art.material),
            &mut last_shader,
            &mut last_blend_mode,
        );
        let model = affine2_to_mat4(&transform.0);
        let vertices: [[f32; 2]; 4] = [
            art.quad[0].into(),
            art.quad[1].into(),
            art.quad[2].into(),
            art.quad[3].into(),
        ];
        renderer.draw_shape(&vertices, &QUAD_INDICES, color, model);
    }
}

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
