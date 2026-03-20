mod animation;
mod letters;
#[cfg(feature = "render")]
pub(crate) mod render;
mod types;

pub use animation::{SplashPlugin, post_splash_setup_system, preload_system, splash_tick_system};
#[cfg(feature = "render")]
pub use render::splash_render_system;
pub use types::{PostSplashSetup, PreloadHooks, SkipSplash, SplashEntity, SplashScreen};
