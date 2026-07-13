use engine_core::prelude::WindowConfig;

/// Configuration for the render subsystem.
///
/// `RenderPlugin` is a data-only struct — it does not implement the `Plugin`
/// trait because engine crates cannot depend on `engine_app` (circular dep).
/// Hook registration is performed by `DefaultPlugins` in the `axiom2d` facade.
pub struct RenderPlugin {
    config: WindowConfig,
}

impl RenderPlugin {
    pub fn new(config: WindowConfig) -> Self {
        Self { config }
    }

    /// The window configuration this renderer was created with.
    pub fn config(&self) -> &WindowConfig {
        &self.config
    }
}

impl Default for RenderPlugin {
    fn default() -> Self {
        Self::new(WindowConfig::default())
    }
}
