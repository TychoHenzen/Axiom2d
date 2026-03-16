use bevy_ecs::prelude::Query;
use engine_scene::prelude::{Children, Visible};

use crate::card::Card;
use crate::card_face_side::CardFaceSide;

pub fn card_face_visibility_sync_system(
    card_query: Query<(&Card, &Children)>,
    mut face_query: Query<(&CardFaceSide, &mut Visible)>,
) {
    for (card, children) in &card_query {
        for &child in &children.0 {
            if let Ok((side, mut vis)) = face_query.get_mut(child) {
                vis.0 = match side {
                    CardFaceSide::Front => card.face_up,
                    CardFaceSide::Back => !card.face_up,
                };
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use bevy_ecs::prelude::*;
    use engine_core::prelude::TextureId;
    use engine_scene::prelude::{ChildOf, Children, Visible};

    use super::card_face_visibility_sync_system;
    use crate::card::Card;
    use crate::card_face_side::CardFaceSide;

    fn run_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(card_face_visibility_sync_system);
        schedule.run(world);
    }

    fn spawn_card_with_children(
        world: &mut World,
        face_up: bool,
        front_visible: bool,
        back_visible: bool,
    ) -> (Entity, Entity, Entity) {
        let root = world
            .spawn(Card {
                face_texture: TextureId(1),
                back_texture: TextureId(2),
                face_up,
            })
            .id();
        let front_child = world
            .spawn((ChildOf(root), CardFaceSide::Front, Visible(front_visible)))
            .id();
        let back_child = world
            .spawn((ChildOf(root), CardFaceSide::Back, Visible(back_visible)))
            .id();
        let mut children = vec![front_child, back_child];
        children.sort();
        world.entity_mut(root).insert(Children(children));
        (root, front_child, back_child)
    }

    #[test]
    fn when_face_up_true_then_front_children_become_visible() {
        // Arrange
        let mut world = World::new();
        let (_, front, _) = spawn_card_with_children(&mut world, true, false, true);

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(world.entity(front).get::<Visible>().unwrap().0, true);
    }

    #[test]
    fn when_face_up_true_then_back_children_become_hidden() {
        // Arrange
        let mut world = World::new();
        let (_, _, back) = spawn_card_with_children(&mut world, true, false, true);

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(world.entity(back).get::<Visible>().unwrap().0, false);
    }

    #[test]
    fn when_face_up_false_then_front_hidden_back_visible() {
        // Arrange
        let mut world = World::new();
        let (_, front, back) = spawn_card_with_children(&mut world, false, true, false);

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(world.entity(front).get::<Visible>().unwrap().0, false);
        assert_eq!(world.entity(back).get::<Visible>().unwrap().0, true);
    }

    #[test]
    fn when_card_has_no_children_then_sync_system_no_panic() {
        // Arrange
        let mut world = World::new();
        world.spawn(Card::face_down(TextureId(1), TextureId(2)));

        // Act + Assert (no panic)
        run_system(&mut world);
    }

    #[test]
    fn when_card_has_non_card_face_children_then_unrelated_children_untouched() {
        // Arrange
        let mut world = World::new();
        let root = world
            .spawn(Card {
                face_texture: TextureId(1),
                back_texture: TextureId(2),
                face_up: true,
            })
            .id();
        let front_child = world
            .spawn((ChildOf(root), CardFaceSide::Front, Visible(false)))
            .id();
        let unrelated = world.spawn((ChildOf(root), Visible(false))).id();
        let mut children = vec![front_child, unrelated];
        children.sort();
        world.entity_mut(root).insert(Children(children));

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(world.entity(front_child).get::<Visible>().unwrap().0, true);
        assert_eq!(
            world.entity(unrelated).get::<Visible>().unwrap().0,
            false,
            "unrelated child should not be modified"
        );
    }
}
