// EVOLVE-BLOCK-START
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
// EVOLVE-BLOCK-END
