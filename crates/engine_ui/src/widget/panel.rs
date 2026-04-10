// EVOLVE-BLOCK-START
use bevy_ecs::component::Component;
use bevy_ecs::prelude::{Query, ResMut};
use engine_core::prelude::{Color, Pixels};
use engine_render::prelude::{Rect, RendererRes};
use engine_scene::prelude::{EffectiveVisibility, GlobalTransform2D};
use serde::{Deserialize, Serialize};

use super::node::UiNode;
use crate::is_hidden;
use crate::layout::anchor_offset;

#[derive(Component, Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Panel {
    pub border_color: Option<Color>,
    pub border_width: f32,
}

pub fn panel_render_system(
    panels: Query<(
        &Panel,
        &UiNode,
        &GlobalTransform2D,
        Option<&EffectiveVisibility>,
    )>,
    mut renderer: ResMut<RendererRes>,
) {
    for (panel, node, transform, visibility) in &panels {
        if is_hidden(visibility) {
            continue;
        }

        if let Some(border_color) = panel.border_color {
            let offset = anchor_offset(node.anchor, node.size);
            let top_left = transform.0.translation + offset;
            draw_borders(
                &mut renderer,
                top_left,
                node.size,
                panel.border_width,
                border_color,
            );
        }
    }
}

fn draw_borders(
    renderer: &mut RendererRes,
    top_left: glam::Vec2,
    size: glam::Vec2,
    bw: f32,
    color: Color,
) {
    renderer.draw_rect(Rect {
        x: Pixels(top_left.x),
        y: Pixels(top_left.y),
        width: Pixels(size.x),
        height: Pixels(bw),
        color,
    });
    renderer.draw_rect(Rect {
        x: Pixels(top_left.x),
        y: Pixels(top_left.y + size.y - bw),
        width: Pixels(size.x),
        height: Pixels(bw),
        color,
    });
    renderer.draw_rect(Rect {
        x: Pixels(top_left.x),
        y: Pixels(top_left.y + bw),
        width: Pixels(bw),
        height: Pixels(size.y - 2.0 * bw),
        color,
    });
    renderer.draw_rect(Rect {
        x: Pixels(top_left.x + size.x - bw),
        y: Pixels(top_left.y + bw),
        width: Pixels(bw),
        height: Pixels(size.y - 2.0 * bw),
        color,
    });
}
// EVOLVE-BLOCK-END
