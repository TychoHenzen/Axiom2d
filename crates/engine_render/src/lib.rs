pub mod prelude;
pub mod rect;
pub mod renderer;
pub mod window;

mod wgpu_renderer;

use std::sync::Arc;
use winit::window::Window;

use crate::renderer::Renderer;
use crate::window::WindowConfig;

pub fn create_renderer(window: Arc<Window>, config: &WindowConfig) -> Box<dyn Renderer> {
    Box::new(wgpu_renderer::WgpuRenderer::new(window, config))
}
