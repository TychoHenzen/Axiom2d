/// Configuration for the audio subsystem.
///
/// `AudioPlugin` is a data-only struct — it does not implement the `Plugin`
/// trait because engine crates cannot depend on `engine_app` (circular dep).
/// Registration is performed by `DefaultPlugins` in the `axiom2d` facade.
///
/// Consumer override: insert `AudioRes<CpalBackend>` before `DefaultPlugins`
/// runs, and the plugin will skip its default `NullAudioBackend`.
pub struct AudioPlugin;

impl Default for AudioPlugin {
    fn default() -> Self {
        Self
    }
}
