use bevy_ecs::prelude::{Res, ResMut, Resource};
use engine_core::color::Color;

use crate::renderer::RendererRes;

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct ClearColor(pub Color);

impl Default for ClearColor {
    fn default() -> Self {
        Self(Color::new(0.392, 0.584, 0.929, 1.0))
    }
}

pub fn clear_system(color: Res<ClearColor>, mut renderer: ResMut<RendererRes>) {
    renderer.clear(color.0);
}
