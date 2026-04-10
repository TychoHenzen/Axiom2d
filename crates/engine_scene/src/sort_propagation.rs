// EVOLVE-BLOCK-START
use bevy_ecs::prelude::{Component, Entity, Query, ResMut, With, Without};
use engine_core::profiler::FrameProfiler;
use serde::{Deserialize, Serialize};

use crate::hierarchy::{ChildOf, Children};
use crate::render_order::SortOrder;

/// Sibling order within a parent. Controls position among siblings during
/// depth-first traversal. Lower values render first (behind higher values).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalSortOrder(pub i32);

/// Walks the scene graph depth-first, sorting siblings by `LocalSortOrder`,
/// and assigns an incrementing `SortOrder` to each entity visited.
#[allow(clippy::type_complexity)]
pub fn hierarchy_sort_system(
    roots: Query<(Entity, Option<&LocalSortOrder>), (With<SortOrder>, Without<ChildOf>)>,
    children_query: Query<&Children>,
    local_sort_query: Query<Option<&LocalSortOrder>>,
    mut sort_query: Query<&mut SortOrder>,
    mut profiler: Option<ResMut<FrameProfiler>>,
) {
    let _span = profiler.as_deref_mut().map(|p| p.span("hierarchy_sort"));
    let mut root_list: Vec<_> = roots.iter().collect();
    root_list.sort_by_key(|(_, local)| local.map_or(0, |l| l.0));

    let mut counter: i32 = 0;
    for (entity, _) in &root_list {
        assign_sort(
            *entity,
            &mut counter,
            &children_query,
            &local_sort_query,
            &mut sort_query,
        );
    }
}

fn assign_sort(
    entity: Entity,
    counter: &mut i32,
    children_query: &Query<&Children>,
    local_sort_query: &Query<Option<&LocalSortOrder>>,
    sort_query: &mut Query<&mut SortOrder>,
) {
    if let Ok(mut sort) = sort_query.get_mut(entity) {
        sort.set(*counter);
    }
    *counter += 1;

    if let Ok(children) = children_query.get(entity) {
        let mut sorted_children: Vec<Entity> = children.0.clone();
        sorted_children.sort_by_key(|&child| {
            local_sort_query
                .get(child)
                .ok()
                .flatten()
                .map_or(0, |l| l.0)
        });
        for child in sorted_children {
            assign_sort(child, counter, children_query, local_sort_query, sort_query);
        }
    }
}
// EVOLVE-BLOCK-END
