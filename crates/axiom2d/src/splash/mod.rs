mod animation;
#[cfg(feature = "render")]
pub(crate) mod render;
mod types;

pub use animation::{SplashPlugin, preload_system, splash_tick_system};
#[cfg(feature = "render")]
pub use render::splash_render_system;
pub use types::{PreloadHooks, SplashEntity, SplashScreen};
