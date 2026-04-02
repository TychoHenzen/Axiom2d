use bevy_ecs::component::Component;
use bevy_ecs::prelude::{Query, Res, ResMut};
use engine_core::prelude::Pixels;
use engine_render::prelude::{Rect, RendererRes};
use engine_scene::prelude::{EffectiveVisibility, GlobalTransform2D};
use serde::{Deserialize, Serialize};

use super::node::UiNode;
use crate::is_hidden;
use crate::layout::anchor_offset;
use crate::theme::UiTheme;

#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ProgressBar {
    pub value: f32,
    pub max: f32,
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self {
            value: 0.0,
            max: 100.0,
        }
    }
}

pub fn progress_bar_render_system(
    bars: Query<(
        &ProgressBar,
        &UiNode,
        &GlobalTransform2D,
        Option<&EffectiveVisibility>,
    )>,
    theme: Res<UiTheme>,
    mut renderer: ResMut<RendererRes>,
) {
    for (bar, node, transform, visibility) in &bars {
        if is_hidden(visibility) {
            continue;
        }

        // Background is handled by `ui_render_system`.

        let fill_width = fill_width(*bar, node.size.x);

        if fill_width > 0.0 {
            let offset = anchor_offset(node.anchor, node.size);
            let top_left = transform.0.translation + offset;
            renderer.draw_rect(Rect {
                x: Pixels(top_left.x),
                y: Pixels(top_left.y),
                width: Pixels(fill_width),
                height: Pixels(node.size.y),
                color: theme.normal_color,
            });
        }
    }
}

fn fill_width(bar: ProgressBar, node_width: f32) -> f32 {
    if bar.max == 0.0 {
        0.0
    } else {
        (bar.value / bar.max).clamp(0.0, 1.0) * node_width
    }
}
