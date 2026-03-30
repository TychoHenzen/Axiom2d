use std::collections::HashMap;

use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct ChildOf(pub Entity);

#[derive(Component, Debug, Clone, PartialEq)]
pub struct Children(pub Vec<Entity>);

#[allow(clippy::implicit_hasher)]
pub fn hierarchy_maintenance_system(
    child_query: Query<(Entity, &ChildOf)>,
    parent_query: Query<Entity, With<Children>>,
    dirty_children: Query<Entity, Or<(Added<ChildOf>, Changed<ChildOf>)>>,
    mut removed_children: RemovedComponents<ChildOf>,
    mut commands: Commands,
    mut cache: Local<HashMap<Entity, Vec<Entity>>>,
    mut initialized: Local<bool>,
) {
    let has_changes = !dirty_children.is_empty() || removed_children.read().next().is_some();
    if *initialized && !has_changes {
        return;
    }
    *initialized = true;

    cache.clear();
    for (child, child_of) in &child_query {
        cache.entry(child_of.0).or_default().push(child);
    }
    for children in cache.values_mut() {
        children.sort();
    }
    for parent in &parent_query {
        if !cache.contains_key(&parent) {
            commands.entity(parent).remove::<Children>();
        }
    }
    for (parent, children) in cache.drain() {
        commands.entity(parent).insert(Children(children));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::run_hierarchy_system;
    use bevy_ecs::schedule::IntoScheduleConfigs;

    #[derive(Resource, Default)]
    struct ChangedChildrenCapture(usize);

    fn capture_changed_children(
        mut capture: ResMut<ChangedChildrenCapture>,
        query: Query<Entity, Changed<Children>>,
    ) {
        capture.0 = query.iter().count();
    }

    #[test]
    fn when_entity_has_child_of_then_hierarchy_system_adds_it_to_parent_children() {
        // Arrange
        let mut world = World::new();
        let parent = world.spawn_empty().id();
        let child = world.spawn(ChildOf(parent)).id();

        // Act
        run_hierarchy_system(&mut world);

        // Assert
        let children = world
            .get::<Children>(parent)
            .expect("parent should have Children component");
        assert!(children.0.contains(&child));
    }

    #[test]
    fn when_two_children_share_same_parent_then_children_contains_both() {
        // Arrange
        let mut world = World::new();
        let parent = world.spawn_empty().id();
        let child_a = world.spawn(ChildOf(parent)).id();
        let child_b = world.spawn(ChildOf(parent)).id();

        // Act
        run_hierarchy_system(&mut world);

        // Assert
        let children = world
            .get::<Children>(parent)
            .expect("parent should have Children");
        assert_eq!(children.0.len(), 2);
        assert!(children.0.contains(&child_a));
        assert!(children.0.contains(&child_b));
    }

    #[test]
    fn when_two_parents_each_have_one_child_then_each_parent_children_is_independent() {
        // Arrange
        let mut world = World::new();
        let parent_a = world.spawn_empty().id();
        let parent_b = world.spawn_empty().id();
        let child_x = world.spawn(ChildOf(parent_a)).id();
        let child_y = world.spawn(ChildOf(parent_b)).id();

        // Act
        run_hierarchy_system(&mut world);

        // Assert
        let children_a = world
            .get::<Children>(parent_a)
            .expect("parent_a should have Children");
        assert_eq!(children_a.0, vec![child_x]);
        let children_b = world
            .get::<Children>(parent_b)
            .expect("parent_b should have Children");
        assert_eq!(children_b.0, vec![child_y]);
    }

    #[test]
    fn when_multiple_children_belong_to_parent_then_children_vec_is_sorted_by_entity() {
        // Arrange
        let mut world = World::new();
        let parent = world.spawn_empty().id();
        let child_a = world.spawn(ChildOf(parent)).id();
        let child_b = world.spawn(ChildOf(parent)).id();
        let child_c = world.spawn(ChildOf(parent)).id();

        // Act
        run_hierarchy_system(&mut world);

        // Assert
        let children = world
            .get::<Children>(parent)
            .expect("parent should have Children");
        let mut sorted = children.0.clone();
        sorted.sort();
        assert_eq!(children.0, sorted);
        assert_eq!(children.0, vec![child_a, child_b, child_c]);
    }

    #[test]
    fn when_system_runs_twice_with_no_changes_then_children_remains_stable() {
        // Arrange
        let mut world = World::new();
        let parent = world.spawn_empty().id();
        let child = world.spawn(ChildOf(parent)).id();

        // Act
        run_hierarchy_system(&mut world);
        run_hierarchy_system(&mut world);

        // Assert
        let children = world
            .get::<Children>(parent)
            .expect("parent should have Children");
        assert_eq!(children.0, vec![child]);
    }

    /// @doc: `hierarchy_maintenance_system` rebuilds `Children` from scratch each frame — reparenting is automatic
    #[test]
    fn when_child_of_is_removed_then_parent_children_no_longer_contains_that_child() {
        // Arrange
        let mut world = World::new();
        let parent = world.spawn_empty().id();
        let child_a = world.spawn(ChildOf(parent)).id();
        let child_b = world.spawn(ChildOf(parent)).id();
        run_hierarchy_system(&mut world);
        world.entity_mut(child_a).remove::<ChildOf>();

        // Act
        run_hierarchy_system(&mut world);

        // Assert
        let children = world
            .get::<Children>(parent)
            .expect("parent should have Children");
        assert_eq!(children.0, vec![child_b]);
    }

    /// @doc: Stale `Children` components are cleaned up when no `ChildOf` references remain for that parent
    #[test]
    fn when_last_child_of_is_removed_then_parent_children_component_is_removed() {
        // Arrange
        let mut world = World::new();
        let parent = world.spawn_empty().id();
        let child = world.spawn(ChildOf(parent)).id();
        run_hierarchy_system(&mut world);
        world.entity_mut(child).remove::<ChildOf>();

        // Act
        run_hierarchy_system(&mut world);

        // Assert
        assert!(world.get::<Children>(parent).is_none());
    }

    #[test]
    fn when_child_entity_is_despawned_then_parent_children_no_longer_contains_that_child() {
        // Arrange
        let mut world = World::new();
        let parent = world.spawn_empty().id();
        let child_a = world.spawn(ChildOf(parent)).id();
        let child_b = world.spawn(ChildOf(parent)).id();
        run_hierarchy_system(&mut world);
        world.despawn(child_a);

        // Act
        run_hierarchy_system(&mut world);

        // Assert
        let children = world
            .get::<Children>(parent)
            .expect("parent should have Children");
        assert_eq!(children.0, vec![child_b]);
    }

    proptest::proptest! {
        #[test]
        fn when_arbitrary_child_of_assignments_then_children_vec_is_sorted(
            child_count in 2usize..=10,
            parent_count in 1usize..=3,
        ) {
            // Arrange
            let mut world = World::new();
            let parents: Vec<Entity> = (0..parent_count)
                .map(|_| world.spawn_empty().id())
                .collect();
            for i in 0..child_count {
                let parent = parents[i % parents.len()];
                world.spawn(ChildOf(parent));
            }

            // Act
            run_hierarchy_system(&mut world);

            // Assert
            for &parent in &parents {
                if let Some(children) = world.get::<Children>(parent) {
                    let sorted = {
                        let mut v = children.0.clone();
                        v.sort();
                        v
                    };
                    assert_eq!(
                        children.0, sorted,
                        "Children vec should be sorted for parent {parent:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn when_only_child_is_despawned_then_parent_children_component_is_removed() {
        // Arrange
        let mut world = World::new();
        let parent = world.spawn_empty().id();
        let child = world.spawn(ChildOf(parent)).id();
        run_hierarchy_system(&mut world);
        world.despawn(child);

        // Act
        run_hierarchy_system(&mut world);

        // Assert
        assert!(world.get::<Children>(parent).is_none());
    }

    #[test]
    fn when_child_is_reparented_then_old_parent_loses_child_and_new_parent_gains_it() {
        // Arrange
        let mut world = World::new();
        let old_parent = world.spawn_empty().id();
        let new_parent = world.spawn_empty().id();
        let child = world.spawn(ChildOf(old_parent)).id();
        run_hierarchy_system(&mut world);
        world.entity_mut(child).insert(ChildOf(new_parent));

        // Act
        run_hierarchy_system(&mut world);

        // Assert
        assert!(world.get::<Children>(old_parent).is_none());
        let new_children = world
            .get::<Children>(new_parent)
            .expect("new parent should have Children");
        assert_eq!(new_children.0, vec![child]);
    }

    #[test]
    fn when_hierarchy_system_reruns_without_changes_then_children_are_not_marked_changed() {
        // Arrange
        let mut world = World::new();
        let parent = world.spawn_empty().id();
        let child = world.spawn(ChildOf(parent)).id();
        world.insert_resource(ChangedChildrenCapture::default());

        let mut schedule = Schedule::default();
        schedule.add_systems((hierarchy_maintenance_system, capture_changed_children).chain());

        schedule.run(&mut world);
        assert_eq!(
            world.resource::<ChangedChildrenCapture>().0,
            1,
            "initial hierarchy build should mark Children as changed"
        );

        // Act
        schedule.run(&mut world);

        // Assert
        let changed = world.resource::<ChangedChildrenCapture>().0;
        assert_eq!(
            changed, 0,
            "unchanged hierarchy should not rewrite Children, but {changed} entities were marked changed"
        );
        let children = world
            .get::<Children>(parent)
            .expect("parent should still have Children");
        assert_eq!(children.0, vec![child]);
    }
}
