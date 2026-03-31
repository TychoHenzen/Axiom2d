use engine_render::prelude::{BlendMode, ShaderHandle};

pub(crate) fn reset_default_shader(renderer: &mut dyn engine_render::prelude::Renderer) {
    renderer.set_shader(ShaderHandle(0));
    renderer.set_blend_mode(BlendMode::Alpha);
}
