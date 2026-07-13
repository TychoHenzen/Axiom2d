/// Configuration for the UI subsystem.
///
/// `UIPlugin` is a data-only struct — it does not implement the `Plugin`
/// trait because engine crates cannot depend on `engine_app` (circular dep).
/// Registration is performed by `DefaultPlugins` in the `axiom2d` facade.
pub struct UIPlugin;

impl Default for UIPlugin {
    fn default() -> Self {
        Self
    }
}
