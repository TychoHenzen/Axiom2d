// EVOLVE-BLOCK-START
use bevy_ecs::prelude::{Query, ResMut};
use engine_render::prelude::RendererRes;
use engine_scene::prelude::{EffectiveVisibility, GlobalTransform2D};

use crate::is_hidden;
use crate::widget::{UiNode, node_rect};

pub fn ui_render_system(
    nodes: Query<(&UiNode, &GlobalTransform2D, Option<&EffectiveVisibility>)>,
    mut renderer: ResMut<RendererRes>,
) {
    for (node, global_transform, visibility) in &nodes {
        if is_hidden(visibility) {
            continue;
        }

        let Some(color) = node.background else {
            continue;
        };

        renderer.draw_rect(node_rect(node, global_transform, color));
    }
}
// EVOLVE-BLOCK-END
