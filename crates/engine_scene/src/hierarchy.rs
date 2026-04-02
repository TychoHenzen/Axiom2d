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
