use bevy_ecs::prelude::{Component, Resource, World};
use engine_core::prelude::Color;

#[derive(Resource)]
pub struct SplashScreen {
    pub elapsed: f32,
    pub duration: f32,
    pub done: bool,
}

impl SplashScreen {
    pub fn new(duration: f32) -> Self {
        Self {
            elapsed: 0.0,
            duration,
            done: false,
        }
    }
}

pub const SPLASH_DURATION: f32 = 2.5;
pub(crate) const SPLASH_BG_ORDER: i32 = 10_000;
pub(crate) const SPLASH_SIDE_BASE: i32 = 10_001;
pub(crate) const SPLASH_LETTER_ORDER: i32 = 11_000;
pub(crate) const SPLASH_ACCENT_ORDER: i32 = 11_001;

pub(crate) const LOGO_COLOR: Color = Color {
    r: 0.85,
    g: 0.85,
    b: 0.95,
    a: 1.0,
};
pub(crate) const ACCENT_COLOR: Color = Color {
    r: 0.4,
    g: 0.5,
    b: 0.9,
    a: 1.0,
};

#[derive(Component)]
pub struct SplashEntity;

pub(crate) type PreloadHook = Box<dyn FnMut(&mut World) + Send + Sync>;

#[derive(Resource)]
pub struct PreloadHooks {
    pub(crate) hooks: Vec<PreloadHook>,
    pub executed: bool,
}

impl PreloadHooks {
    pub fn new() -> Self {
        Self {
            hooks: Vec::new(),
            executed: false,
        }
    }

    pub fn add(&mut self, hook: impl FnMut(&mut World) + Send + Sync + 'static) {
        self.hooks.push(Box::new(hook));
    }
}

impl Default for PreloadHooks {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) type PostSplashHook = Box<dyn FnMut(&mut World) + Send + Sync>;

#[derive(Resource)]
pub struct PostSplashSetup {
    pub(crate) hooks: Vec<PostSplashHook>,
    pub executed: bool,
}

impl PostSplashSetup {
    pub fn new() -> Self {
        Self {
            hooks: Vec::new(),
            executed: false,
        }
    }

    pub fn add(&mut self, hook: impl FnMut(&mut World) + Send + Sync + 'static) {
        self.hooks.push(Box::new(hook));
    }
}

impl Default for PostSplashSetup {
    fn default() -> Self {
        Self::new()
    }
}

/// Marker resource to skip the splash screen.
/// Insert this before calling `app.add_plugin(DefaultPlugins)`.
#[derive(Resource)]
pub struct SkipSplash;
