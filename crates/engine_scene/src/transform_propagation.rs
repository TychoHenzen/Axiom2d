use bevy_ecs::prelude::*;
use engine_core::prelude::Transform2D;
use glam::Affine2;

use crate::hierarchy::{ChildOf, Children};

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct GlobalTransform2D(pub Affine2);

pub fn transform_propagation_system(
    roots: Query<(Entity, &Transform2D, Option<&GlobalTransform2D>), Without<ChildOf>>,
    children_query: Query<&Children>,
    transforms: Query<&Transform2D>,
    dirty_transforms: Query<&Transform2D, Or<(Added<Transform2D>, Changed<Transform2D>)>>,
    globals: Query<&GlobalTransform2D>,
    mut commands: Commands,
) {
    for (entity, transform, existing_global) in &roots {
        let root_dirty = dirty_transforms.get(entity).is_ok() || existing_global.is_none();
        let global = if root_dirty {
            let global = GlobalTransform2D(transform.to_affine2());
            commands.entity(entity).insert(global);
            global
        } else {
            *existing_global.expect("clean root must already have GlobalTransform2D")
        };
        propagate_to_children(
            entity,
            &global,
            root_dirty,
            &children_query,
            &transforms,
            &dirty_transforms,
            &globals,
            &mut commands,
        );
    }
}

fn propagate_to_children(
    parent: Entity,
    parent_global: &GlobalTransform2D,
    ancestor_dirty: bool,
    children_query: &Query<&Children>,
    transforms: &Query<&Transform2D>,
    dirty_transforms: &Query<&Transform2D, Or<(Added<Transform2D>, Changed<Transform2D>)>>,
    globals: &Query<&GlobalTransform2D>,
    commands: &mut Commands,
) {
    if let Ok(children) = children_query.get(parent) {
        for &child in &children.0 {
            if let Ok(local) = transforms.get(child) {
                let child_dirty = ancestor_dirty
                    || dirty_transforms.get(child).is_ok()
                    || globals.get(child).is_err();
                let global = if child_dirty {
                    let global = GlobalTransform2D(parent_global.0 * local.to_affine2());
                    commands.entity(child).insert(global);
                    global
                } else {
                    *globals
                        .get(child)
                        .expect("clean child must already have GlobalTransform2D")
                };
                propagate_to_children(
                    child,
                    &global,
                    child_dirty,
                    children_query,
                    transforms,
                    dirty_transforms,
                    globals,
                    commands,
                );
            }
        }
    }
}
