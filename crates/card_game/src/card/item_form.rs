use std::collections::HashMap;

use bevy_ecs::prelude::{Component, Entity, Query};
use engine_scene::prelude::{ChildOf, Visible};

use crate::card::component::Card;
use crate::card::face_side::CardFaceSide;
use crate::stash::icon::StashIcon;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct CardItemForm;

pub fn card_item_form_visibility_system(
    cards: Query<(Entity, &Card, Option<&CardItemForm>)>,
    mut children: Query<(
        &ChildOf,
        Option<&CardFaceSide>,
        Option<&StashIcon>,
        &mut Visible,
    )>,
) {
    let card_states: HashMap<Entity, (bool, bool)> = cards
        .iter()
        .map(|(entity, card, item_form)| (entity, (item_form.is_some(), card.face_up)))
        .collect();

    for (child_of, face_side, stash_icon, mut visible) in &mut children {
        let Some(&(has_item_form, face_up)) = card_states.get(&child_of.0) else {
            continue;
        };
        if has_item_form {
            visible.0 = false;
        } else if stash_icon.is_some() {
            visible.0 = false;
        } else if let Some(side) = face_side {
            visible.0 = match side {
                CardFaceSide::Front => face_up,
                CardFaceSide::Back => !face_up,
            };
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use bevy_ecs::prelude::*;
    use engine_core::prelude::TextureId;
    use engine_scene::prelude::{ChildOf, Visible};

    use super::{CardItemForm, card_item_form_visibility_system};
    use crate::card::component::Card;
    use crate::card::face_side::CardFaceSide;
    use crate::stash::icon::StashIcon;

    // ---------------------------------------------------------------------------
    // Shared setup
    // ---------------------------------------------------------------------------

    /// Spawns a minimal card hierarchy without physics:
    ///   root (Card + optional `CardItemForm`)
    ///   ├── 4 × Front children (`CardFaceSide::Front` + Visible)
    ///   ├── 2 × Back  children (`CardFaceSide::Back`  + Visible)
    ///   └── 1 × `StashIcon` child (`StashIcon` + Visible)
    ///
    /// Returns `(root, front_entities, back_entities, stash_icon_entity)`.
    fn make_card_with_children(
        world: &mut World,
        face_up: bool,
    ) -> (Entity, Vec<Entity>, Vec<Entity>, Entity) {
        let root = world
            .spawn(Card {
                face_texture: TextureId(1),
                back_texture: TextureId(2),
                face_up,
                signature: None,
            })
            .id();

        let front_children: Vec<Entity> = (0..4)
            .map(|_| {
                world
                    .spawn((ChildOf(root), CardFaceSide::Front, Visible(face_up)))
                    .id()
            })
            .collect();

        let back_children: Vec<Entity> = (0..2)
            .map(|_| {
                world
                    .spawn((ChildOf(root), CardFaceSide::Back, Visible(!face_up)))
                    .id()
            })
            .collect();

        let stash_icon = world.spawn((ChildOf(root), StashIcon, Visible(false))).id();

        (root, front_children, back_children, stash_icon)
    }

    fn run_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(card_item_form_visibility_system);
        schedule.run(world);
    }

    // ---------------------------------------------------------------------------
    // TC006 – CardItemForm present → StashIcon becomes Visible(true)
    // ---------------------------------------------------------------------------

    #[test]
    fn when_card_item_form_added_then_stash_icon_child_becomes_hidden() {
        // Arrange — all children hidden when in item form (stash_render_system handles visuals)
        let mut world = World::new();
        let (root, _, _, stash_icon) = make_card_with_children(&mut world, false);
        world.entity_mut(root).insert(CardItemForm);

        // Act
        run_system(&mut world);

        // Assert
        assert!(
            !world.entity(stash_icon).get::<Visible>().unwrap().0,
            "StashIcon should be Visible(false) when CardItemForm is present"
        );
    }

    // ---------------------------------------------------------------------------
    // TC007 – CardItemForm present → ALL CardFaceSide children become Visible(false)
    // ---------------------------------------------------------------------------

    #[test]
    fn when_card_item_form_added_then_all_card_face_side_children_become_hidden() {
        // Arrange
        let mut world = World::new();
        let (root, front_children, back_children, _) = make_card_with_children(&mut world, true);
        world.entity_mut(root).insert(CardItemForm);

        // Act
        run_system(&mut world);

        // Assert
        let all_face_entities: Vec<Entity> =
            front_children.into_iter().chain(back_children).collect();
        for entity in all_face_entities {
            let vis = world.entity(entity).get::<Visible>().unwrap();
            assert!(
                !vis.0,
                "CardFaceSide child {entity:?} should be Visible(false) when CardItemForm is present"
            );
        }
    }

    // ---------------------------------------------------------------------------
    // TC008 – CardItemForm removed → StashIcon becomes Visible(false)
    // Precondition: stash icon is already Visible(true), simulating a prior
    // run of the system while CardItemForm was present.
    // ---------------------------------------------------------------------------

    #[test]
    fn when_card_item_form_removed_then_stash_icon_child_becomes_hidden() {
        // Arrange — icon starts Visible(true) as if CardItemForm had been active
        let mut world = World::new();
        let (_root, _, _, stash_icon) = make_card_with_children(&mut world, false);
        world.entity_mut(stash_icon).insert(Visible(true));

        // Act
        run_system(&mut world);

        // Assert
        assert!(
            !world.entity(stash_icon).get::<Visible>().unwrap().0,
            "StashIcon should be Visible(false) when root has no CardItemForm"
        );
    }

    // ---------------------------------------------------------------------------
    // TC009 – CardItemForm removed from face-up card → Front visible, Back hidden
    // Precondition: all face children are Visible(false) and icon is Visible(true),
    // simulating the state after a prior run with CardItemForm active.
    // ---------------------------------------------------------------------------

    #[test]
    fn when_card_item_form_removed_from_face_up_card_then_front_visible_back_hidden() {
        // Arrange — face-up card, face children all hidden, icon shown (CardItemForm was active)
        let mut world = World::new();
        let (_root, front_children, back_children, _) = make_card_with_children(&mut world, true);
        for &e in front_children.iter().chain(back_children.iter()) {
            world.entity_mut(e).insert(Visible(false));
        }
        // root does NOT have CardItemForm

        // Act
        run_system(&mut world);

        // Assert
        for entity in &front_children {
            let vis = world.entity(*entity).get::<Visible>().unwrap();
            assert!(
                vis.0,
                "Front child {entity:?} should be Visible(true) for face-up card with no CardItemForm"
            );
        }
        for entity in &back_children {
            let vis = world.entity(*entity).get::<Visible>().unwrap();
            assert!(
                !vis.0,
                "Back child {entity:?} should be Visible(false) for face-up card with no CardItemForm"
            );
        }
    }

    // ---------------------------------------------------------------------------
    // TC010 – No entities have CardItemForm → no Visible changes (no-op)
    // ---------------------------------------------------------------------------

    #[test]
    fn when_no_card_has_card_item_form_then_visible_components_are_unchanged() {
        // Arrange
        let mut world = World::new();
        let (_, front_children, back_children, stash_icon) =
            make_card_with_children(&mut world, true);
        // Record initial visibility before running the system
        let initial_front: Vec<bool> = front_children
            .iter()
            .map(|&e| world.entity(e).get::<Visible>().unwrap().0)
            .collect();
        let initial_back: Vec<bool> = back_children
            .iter()
            .map(|&e| world.entity(e).get::<Visible>().unwrap().0)
            .collect();
        let initial_stash = world.entity(stash_icon).get::<Visible>().unwrap().0;

        // Act
        run_system(&mut world);

        // Assert
        for (i, &entity) in front_children.iter().enumerate() {
            assert_eq!(
                world.entity(entity).get::<Visible>().unwrap().0,
                initial_front[i],
                "Front child {entity:?} Visible should be unchanged when no CardItemForm present"
            );
        }
        for (i, &entity) in back_children.iter().enumerate() {
            assert_eq!(
                world.entity(entity).get::<Visible>().unwrap().0,
                initial_back[i],
                "Back child {entity:?} Visible should be unchanged when no CardItemForm present"
            );
        }
        assert_eq!(
            world.entity(stash_icon).get::<Visible>().unwrap().0,
            initial_stash,
            "StashIcon Visible should be unchanged when no CardItemForm present"
        );
    }
}
