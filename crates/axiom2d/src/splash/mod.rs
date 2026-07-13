mod animation;
#[cfg(feature = "render")]
mod letters;
#[cfg(feature = "render")]
pub(crate) mod render;
mod types;

pub use animation::{SplashPlugin, post_splash_setup_system, preload_system, splash_tick_system};
#[cfg(feature = "render")]
pub use letters::{letter_a, letter_i, letter_m, letter_o, letter_x};
#[cfg(feature = "render")]
pub use render::splash_render_system;
pub use types::{ACCENT_COLOR, LOGO_COLOR};
pub use types::{
    PostSplashSetup, PreloadHooks, SPLASH_DURATION, SkipSplash, SplashEntity, SplashScreen,
};
