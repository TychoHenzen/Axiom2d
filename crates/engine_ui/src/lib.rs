pub mod interaction;
pub mod layout;
pub mod prelude;
pub mod render;
pub mod text_render;
pub mod theme;
pub mod ui_event;
pub mod unified_render;
pub mod widget;

use engine_scene::prelude::EffectiveVisibility;

pub(crate) fn is_hidden(visibility: Option<&EffectiveVisibility>) -> bool {
    visibility.is_some_and(|v| !v.0)
}
