/// Configuration for the physics subsystem.
///
/// `PhysicsPlugin` is a data-only struct — it does not implement the `Plugin`
/// trait because engine crates cannot depend on `engine_app` (circular dep).
/// Registration is performed by `DefaultPlugins` in the `axiom2d` facade.
///
/// Consumer override: insert `PhysicsRes<RapierBackend>` before `DefaultPlugins`
/// runs, and the plugin will skip its default `NullPhysicsBackend`.
pub struct PhysicsPlugin;

impl Default for PhysicsPlugin {
    fn default() -> Self {
        Self
    }
}
