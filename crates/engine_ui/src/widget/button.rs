// EVOLVE-BLOCK-START
use bevy_ecs::component::Component;
use bevy_ecs::prelude::{Query, Res, ResMut};
use engine_render::prelude::RendererRes;
use engine_scene::prelude::{EffectiveVisibility, GlobalTransform2D};
use serde::{Deserialize, Serialize};

use super::node::UiNode;
use super::node_rect;
use crate::interaction::Interaction;
use crate::is_hidden;
use crate::theme::UiTheme;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Button {
    pub disabled: bool,
}

#[allow(clippy::type_complexity)]
pub fn button_render_system(
    buttons: Query<(
        &Button,
        &UiNode,
        &GlobalTransform2D,
        Option<&Interaction>,
        Option<&EffectiveVisibility>,
    )>,
    theme: Res<UiTheme>,
    mut renderer: ResMut<RendererRes>,
) {
    for (button, node, transform, interaction, visibility) in &buttons {
        if is_hidden(visibility) {
            continue;
        }

        let color = if button.disabled {
            theme.disabled_color
        } else {
            match interaction.copied().unwrap_or_default() {
                Interaction::Pressed => theme.pressed_color,
                Interaction::Hovered => theme.hovered_color,
                Interaction::None => theme.normal_color,
            }
        };

        renderer.draw_rect(node_rect(node, transform, color));
    }
}
// EVOLVE-BLOCK-END
