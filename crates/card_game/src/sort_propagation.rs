use bevy_ecs::prelude::{Component, Query, Without};
use engine_scene::prelude::{Children, SortOrder};
use serde::{Deserialize, Serialize};

/// Local sort order within a parent entity. The sort propagation system
/// computes each child's effective `SortOrder` as:
///   `parent.SortOrder * SORT_STRIDE + LocalSortOrder`
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalSortOrder(pub i32);

/// Multiplier applied to parent `SortOrder` to create space for child sub-layers.
pub const SORT_STRIDE: i32 = 10;

pub fn sort_propagation_system(
    parent_query: Query<(&SortOrder, &Children), Without<LocalSortOrder>>,
    mut child_query: Query<(&LocalSortOrder, &mut SortOrder)>,
) {
    for (parent_sort, children) in &parent_query {
        let base = parent_sort.0 * SORT_STRIDE;
        for &child in &children.0 {
            if let Ok((local, mut sort)) = child_query.get_mut(child) {
                sort.0 = base + local.0;
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use bevy_ecs::prelude::*;
    use engine_scene::prelude::{ChildOf, Children, SortOrder};

    use super::{LocalSortOrder, SORT_STRIDE, sort_propagation_system};

    fn run_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(sort_propagation_system);
        schedule.run(world);
    }

    #[test]
    fn when_parent_sort_zero_then_child_gets_local_sort() {
        // Arrange
        let mut world = World::new();
        let parent = world.spawn(SortOrder(0)).id();
        let child = world
            .spawn((ChildOf(parent), LocalSortOrder(3), SortOrder(0)))
            .id();
        world.entity_mut(parent).insert(Children(vec![child]));

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(world.entity(child).get::<SortOrder>().unwrap().0, 3);
    }

    #[test]
    fn when_parent_sort_five_then_child_gets_stride_times_five_plus_local() {
        // Arrange
        let mut world = World::new();
        let parent = world.spawn(SortOrder(5)).id();
        let child = world
            .spawn((ChildOf(parent), LocalSortOrder(2), SortOrder(0)))
            .id();
        world.entity_mut(parent).insert(Children(vec![child]));

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(
            world.entity(child).get::<SortOrder>().unwrap().0,
            5 * SORT_STRIDE + 2
        );
    }

    #[test]
    fn when_two_parents_then_higher_parent_children_sort_above() {
        // Arrange
        let mut world = World::new();
        let parent_a = world.spawn(SortOrder(1)).id();
        let child_a = world
            .spawn((ChildOf(parent_a), LocalSortOrder(4), SortOrder(0)))
            .id();
        world.entity_mut(parent_a).insert(Children(vec![child_a]));

        let parent_b = world.spawn(SortOrder(2)).id();
        let child_b = world
            .spawn((ChildOf(parent_b), LocalSortOrder(1), SortOrder(0)))
            .id();
        world.entity_mut(parent_b).insert(Children(vec![child_b]));

        // Act
        run_system(&mut world);

        // Assert — child_b (parent sort 2, local 1) > child_a (parent sort 1, local 4)
        let sort_a = world.entity(child_a).get::<SortOrder>().unwrap().0;
        let sort_b = world.entity(child_b).get::<SortOrder>().unwrap().0;
        assert!(
            sort_b > sort_a,
            "child_b ({sort_b}) should sort above child_a ({sort_a})"
        );
    }

    #[test]
    fn when_parent_sort_changes_then_children_update_on_next_run() {
        // Arrange
        let mut world = World::new();
        let parent = world.spawn(SortOrder(1)).id();
        let child = world
            .spawn((ChildOf(parent), LocalSortOrder(2), SortOrder(0)))
            .id();
        world.entity_mut(parent).insert(Children(vec![child]));
        run_system(&mut world);
        assert_eq!(
            world.entity(child).get::<SortOrder>().unwrap().0,
            SORT_STRIDE + 2
        );

        // Act — bump parent sort (like card_pick_system does)
        world.entity_mut(parent).get_mut::<SortOrder>().unwrap().0 = 10;
        run_system(&mut world);

        // Assert
        assert_eq!(
            world.entity(child).get::<SortOrder>().unwrap().0,
            10 * SORT_STRIDE + 2
        );
    }

    #[test]
    fn when_child_has_no_local_sort_then_not_affected() {
        // Arrange
        let mut world = World::new();
        let parent = world.spawn(SortOrder(5)).id();
        let child = world.spawn((ChildOf(parent), SortOrder(99))).id();
        world.entity_mut(parent).insert(Children(vec![child]));

        // Act
        run_system(&mut world);

        // Assert — child without LocalSortOrder keeps its original SortOrder
        assert_eq!(world.entity(child).get::<SortOrder>().unwrap().0, 99);
    }
}
