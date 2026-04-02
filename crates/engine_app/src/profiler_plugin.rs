use std::path::PathBuf;

use engine_core::profiler::FrameProfiler;

use crate::app::{App, Plugin};

/// Inserts a [`FrameProfiler`] resource into the app.
///
/// Register in `setup()` before calling `app.run()`:
/// ```ignore
/// app.add_plugin(FrameProfilerPlugin::default());
/// ```
pub struct FrameProfilerPlugin {
    /// Frames between CSV flushes. Must be >= 1. Default: 120.
    pub flush_interval: u64,
    /// Path to the output CSV file. Default: `"perf_log.csv"`.
    pub output_path: PathBuf,
}

impl Default for FrameProfilerPlugin {
    fn default() -> Self {
        Self {
            flush_interval: 120,
            output_path: PathBuf::from("perf_log.csv"),
        }
    }
}

impl Plugin for FrameProfilerPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().insert_resource(FrameProfiler::new(
            self.flush_interval,
            self.output_path.clone(),
        ));
    }
}
