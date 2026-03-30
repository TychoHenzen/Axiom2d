use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::hierarchy::{ChildOf, Children};

#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Visible(pub bool);

impl Default for Visible {
    fn default() -> Self {
        Visible(true)
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct EffectiveVisibility(pub bool);

pub fn visibility_system(
    roots: Query<(Entity, Option<&Visible>, Option<&EffectiveVisibility>), Without<ChildOf>>,
    children_query: Query<&Children>,
    visible_query: Query<Option<&Visible>>,
    dirty_visible: Query<&Visible, Or<(Added<Visible>, Changed<Visible>)>>,
    effective_query: Query<&EffectiveVisibility>,
    mut commands: Commands,
) {
    for (entity, visible, existing_effective) in &roots {
        let root_dirty = dirty_visible.get(entity).is_ok() || existing_effective.is_none();
        let is_visible = if root_dirty {
            let is_visible = visible.is_none_or(|v| v.0);
            commands
                .entity(entity)
                .insert(EffectiveVisibility(is_visible));
            is_visible
        } else {
            existing_effective
                .expect("clean root must already have EffectiveVisibility")
                .0
        };
        propagate_visibility(
            entity,
            is_visible,
            root_dirty,
            &children_query,
            &visible_query,
            &dirty_visible,
            &effective_query,
            &mut commands,
        );
    }
}

fn propagate_visibility(
    parent: Entity,
    parent_effective: bool,
    ancestor_dirty: bool,
    children_query: &Query<&Children>,
    visible_query: &Query<Option<&Visible>>,
    dirty_visible: &Query<&Visible, Or<(Added<Visible>, Changed<Visible>)>>,
    effective_query: &Query<&EffectiveVisibility>,
    commands: &mut Commands,
) {
    if let Ok(children) = children_query.get(parent) {
        for &child in &children.0 {
            let child_visible = visible_query.get(child).ok().flatten().is_none_or(|v| v.0);
            let child_dirty = ancestor_dirty
                || dirty_visible.get(child).is_ok()
                || effective_query.get(child).is_err();
            let effective = if child_dirty {
                let effective = parent_effective && child_visible;
                commands
                    .entity(child)
                    .insert(EffectiveVisibility(effective));
                effective
            } else {
                effective_query
                    .get(child)
                    .expect("clean child must already have EffectiveVisibility")
                    .0
            };
            propagate_visibility(
                child,
                effective,
                child_dirty,
                children_query,
                visible_query,
                dirty_visible,
                effective_query,
                commands,
            );
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_helpers::run_visibility_system;
    use bevy_ecs::schedule::IntoScheduleConfigs;

    #[derive(Resource, Default)]
    struct ChangedEffectiveVisibilityCapture(usize);

    fn capture_changed_effective_visibility(
        mut capture: ResMut<ChangedEffectiveVisibilityCapture>,
        query: Query<Entity, Changed<EffectiveVisibility>>,
    ) {
        capture.0 = query.iter().count();
    }

    #[test]
    fn when_root_entity_has_default_visible_then_visibility_system_inserts_effective_visibility_true()
     {
        // Arrange
        let mut world = World::new();
        let entity = world.spawn(Visible::default()).id();

        // Act
        run_visibility_system(&mut world);

        // Assert
        let effective = world.get::<EffectiveVisibility>(entity).unwrap();
        assert_eq!(*effective, EffectiveVisibility(true));
    }

    #[test]
    fn when_root_entity_has_visible_false_then_visibility_system_inserts_effective_visibility_false()
     {
        // Arrange
        let mut world = World::new();
        let entity = world.spawn(Visible(false)).id();

        // Act
        run_visibility_system(&mut world);

        // Assert
        let effective = world.get::<EffectiveVisibility>(entity).unwrap();
        assert_eq!(*effective, EffectiveVisibility(false));
    }

    /// @doc: Visible is opt-in — entities without it default to visible (no component = no hiding)
    #[test]
    fn when_root_entity_has_no_visible_component_then_visibility_system_inserts_effective_visibility_true()
     {
        // Arrange
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        // Act
        run_visibility_system(&mut world);

        // Assert
        let effective = world.get::<EffectiveVisibility>(entity).unwrap();
        assert_eq!(*effective, EffectiveVisibility(true));
    }

    #[test]
    fn when_visible_parent_has_visible_child_then_child_effective_visibility_is_true() {
        // Arrange
        let mut world = World::new();
        let parent = world.spawn(Visible(true)).id();
        let child = world.spawn((Visible(true), ChildOf(parent))).id();
        world.entity_mut(parent).insert(Children(vec![child]));

        // Act
        run_visibility_system(&mut world);

        // Assert
        let effective = world.get::<EffectiveVisibility>(child).unwrap();
        assert_eq!(*effective, EffectiveVisibility(true));
    }

    /// @doc: AND-logic propagation: `EffectiveVisibility` = `parent_effective` AND `child_visible`
    #[test]
    fn when_parent_is_hidden_and_child_is_visible_then_child_effective_visibility_is_false() {
        // Arrange
        let mut world = World::new();
        let parent = world.spawn(Visible(false)).id();
        let child = world.spawn((Visible(true), ChildOf(parent))).id();
        world.entity_mut(parent).insert(Children(vec![child]));

        // Act
        run_visibility_system(&mut world);

        // Assert
        let effective = world.get::<EffectiveVisibility>(child).unwrap();
        assert_eq!(*effective, EffectiveVisibility(false));
    }

    #[test]
    fn when_parent_is_visible_and_child_is_hidden_then_child_effective_visibility_is_false() {
        // Arrange
        let mut world = World::new();
        let parent = world.spawn(Visible(true)).id();
        let child = world.spawn((Visible(false), ChildOf(parent))).id();
        world.entity_mut(parent).insert(Children(vec![child]));

        // Act
        run_visibility_system(&mut world);

        // Assert
        let effective = world.get::<EffectiveVisibility>(child).unwrap();
        assert_eq!(*effective, EffectiveVisibility(false));
    }

    #[test]
    fn when_three_level_hierarchy_with_hidden_root_then_grandchild_effective_visibility_is_false() {
        // Arrange
        let mut world = World::new();
        let root = world.spawn(Visible(false)).id();
        let child = world.spawn((Visible(true), ChildOf(root))).id();
        let grandchild = world.spawn((Visible(true), ChildOf(child))).id();
        world.entity_mut(root).insert(Children(vec![child]));
        world.entity_mut(child).insert(Children(vec![grandchild]));

        // Act
        run_visibility_system(&mut world);

        // Assert
        let effective = world.get::<EffectiveVisibility>(grandchild).unwrap();
        assert_eq!(*effective, EffectiveVisibility(false));
    }

    #[test]
    fn when_two_siblings_one_hidden_then_each_gets_independent_effective_visibility() {
        // Arrange
        let mut world = World::new();
        let parent = world.spawn(Visible(true)).id();
        let child_a = world.spawn((Visible(true), ChildOf(parent))).id();
        let child_b = world.spawn((Visible(false), ChildOf(parent))).id();
        world
            .entity_mut(parent)
            .insert(Children(vec![child_a, child_b]));

        // Act
        run_visibility_system(&mut world);

        // Assert
        let a_effective = world.get::<EffectiveVisibility>(child_a).unwrap();
        assert_eq!(*a_effective, EffectiveVisibility(true));
        let b_effective = world.get::<EffectiveVisibility>(child_b).unwrap();
        assert_eq!(*b_effective, EffectiveVisibility(false));
    }

    #[test]
    fn when_hierarchy_system_runs_before_visibility_system_then_children_receive_effective_visibility()
     {
        // Arrange
        let mut world = World::new();
        let parent = world.spawn(Visible(true)).id();
        let child = world.spawn((Visible(true), ChildOf(parent))).id();

        // Act
        crate::test_helpers::run_hierarchy_system(&mut world);
        run_visibility_system(&mut world);

        // Assert
        let effective = world.get::<EffectiveVisibility>(child).unwrap();
        assert_eq!(*effective, EffectiveVisibility(true));
    }

    #[test]
    fn when_parent_visibility_changed_and_system_reruns_then_child_effective_visibility_updates() {
        // Arrange
        let mut world = World::new();
        let parent = world.spawn(Visible(false)).id();
        let child = world.spawn((Visible(true), ChildOf(parent))).id();
        world.entity_mut(parent).insert(Children(vec![child]));
        run_visibility_system(&mut world);
        assert_eq!(
            *world.get::<EffectiveVisibility>(child).unwrap(),
            EffectiveVisibility(false)
        );
        world.entity_mut(parent).insert(Visible(true));

        // Act
        run_visibility_system(&mut world);

        // Assert
        let effective = world.get::<EffectiveVisibility>(child).unwrap();
        assert_eq!(*effective, EffectiveVisibility(true));
    }

    #[test]
    fn when_child_has_no_visible_component_and_parent_is_visible_then_child_effective_visibility_is_true()
     {
        // Arrange
        let mut world = World::new();
        let parent = world.spawn(Visible(true)).id();
        let child = world.spawn(ChildOf(parent)).id();
        world.entity_mut(parent).insert(Children(vec![child]));

        // Act
        run_visibility_system(&mut world);

        // Assert
        let effective = world.get::<EffectiveVisibility>(child).unwrap();
        assert_eq!(*effective, EffectiveVisibility(true));
    }

    #[test]
    fn when_child_has_no_visible_component_and_parent_is_hidden_then_child_effective_visibility_is_false()
     {
        // Arrange
        let mut world = World::new();
        let parent = world.spawn(Visible(false)).id();
        let child = world.spawn(ChildOf(parent)).id();
        world.entity_mut(parent).insert(Children(vec![child]));

        // Act
        run_visibility_system(&mut world);

        // Assert
        let effective = world.get::<EffectiveVisibility>(child).unwrap();
        assert_eq!(*effective, EffectiveVisibility(false));
    }

    #[test]
    fn when_visibility_system_reruns_without_changes_then_effective_visibility_is_not_marked_changed()
     {
        // Arrange
        let mut world = World::new();
        let parent = world.spawn(Visible(true)).id();
        let child = world.spawn((Visible(true), ChildOf(parent))).id();
        world.entity_mut(parent).insert(Children(vec![child]));
        world.insert_resource(ChangedEffectiveVisibilityCapture::default());

        let mut schedule = Schedule::default();
        schedule.add_systems((visibility_system, capture_changed_effective_visibility).chain());
        schedule.run(&mut world);

        // Act
        schedule.run(&mut world);

        // Assert
        let changed = world.resource::<ChangedEffectiveVisibilityCapture>().0;
        assert!(
            changed == 0,
            "unchanged hierarchy should not rewrite EffectiveVisibility, but {changed} entities were marked changed"
        );

        let parent_effective = world.get::<EffectiveVisibility>(parent).unwrap();
        assert_eq!(*parent_effective, EffectiveVisibility(true));
        let child_effective = world.get::<EffectiveVisibility>(child).unwrap();
        assert_eq!(*child_effective, EffectiveVisibility(true));
    }
}
