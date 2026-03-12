pub mod atlas;
pub mod bloom;
pub mod camera;
pub mod clear;
pub mod material;
pub mod prelude;
pub mod rect;
pub mod renderer;
pub mod shape;
pub mod sprite;
#[cfg(any(test, feature = "testing"))]
pub mod testing;
#[cfg(any(test, feature = "testing"))]
pub mod visual_regression;
pub mod window;

mod wgpu_renderer;

use std::sync::Arc;
use winit::window::Window;

use crate::renderer::Renderer;
use crate::window::WindowConfig;

pub fn create_renderer(
    window: Arc<Window>,
    config: &WindowConfig,
) -> Box<dyn Renderer + Send + Sync> {
    Box::new(wgpu_renderer::WgpuRenderer::new(window, config))
}
