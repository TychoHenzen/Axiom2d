use bevy_ecs::prelude::Query;
use engine_core::prelude::Transform2D;
use engine_scene::prelude::{Children, GlobalTransform2D};

use super::flex::{FlexLayout, compute_flex_offsets};
use super::margin::Margin;
use crate::widget::UiNode;

pub fn ui_layout_system(
    parents: Query<(&FlexLayout, &Children, &GlobalTransform2D)>,
    mut children: Query<(&UiNode, &mut Transform2D)>,
) {
    for (layout, child_entities, parent_global) in &parents {
        let child_data: Vec<(glam::Vec2, Margin)> = child_entities
            .0
            .iter()
            .filter_map(|&e| {
                children
                    .get(e)
                    .ok()
                    .map(|(node, _)| (node.size, node.margin))
            })
            .collect();

        let offsets = compute_flex_offsets(layout, &child_data);
        let origin = parent_global.0.translation;

        for (entity, offset) in child_entities.0.iter().zip(offsets) {
            if let Ok((_, mut transform)) = children.get_mut(*entity) {
                transform.position = origin + offset;
            }
        }
    }
}
