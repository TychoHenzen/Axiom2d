use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

use bevy_ecs::prelude::Resource;

struct Record {
    frame: u64,
    scope: &'static str,
    duration_us: u64,
}

/// Per-frame performance profiler. Insert as an ECS resource via `FrameProfilerPlugin`.
///
/// Records phase timings (set by `App::handle_redraw`) and manual spans from systems.
/// Flushes to CSV every `flush_interval` frames.
///
/// ## Analysing the output
/// ```python
/// import pandas as pd
/// df = pd.read_csv("perf_log.csv")
/// print(df.groupby("scope")["duration_us"].agg(["median", lambda x: x.quantile(0.99)]))
/// ```
#[derive(Resource)]
pub struct FrameProfiler {
    frame: u64,
    flush_interval: u64,
    records: Vec<Record>,
    output_path: PathBuf,
}

impl FrameProfiler {
    /// `flush_interval` must be >= 1. Default: 120 (≈2 s at 60 fps).
    pub fn new(flush_interval: u64, output_path: PathBuf) -> Self {
        assert!(flush_interval >= 1, "flush_interval must be >= 1");
        Self {
            frame: 0,
            flush_interval,
            records: Vec::new(),
            output_path,
        }
    }

    /// Record a named duration in microseconds. Called by `App::handle_redraw` for
    /// phase timings; also usable inside systems for manual instrumentation.
    pub fn record_phase(&mut self, name: &'static str, duration_us: u64) {
        self.records.push(Record {
            frame: self.frame,
            scope: name,
            duration_us,
        });
    }

    /// Start a named span. The elapsed time is recorded when the returned guard is dropped.
    ///
    /// Spans cannot be nested — the `&mut` borrow prevents holding two guards simultaneously.
    /// Drop the first guard (or use `drop(span)`) before starting another.
    pub fn span(&mut self, name: &'static str) -> ProfileScope<'_> {
        ProfileScope {
            profiler: self,
            name,
            start: Instant::now(),
        }
    }

    /// Advance the frame counter. Flushes to disk when `frame % flush_interval == 0`.
    /// Call once per frame, after all schedules have run.
    pub fn end_frame(&mut self) {
        self.frame += 1;
        if self.frame % self.flush_interval == 0 {
            self.flush();
        }
    }

    /// Number of buffered records not yet flushed. Used in tests.
    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    fn flush(&mut self) {
        if let Err(e) = self.write_records() {
            eprintln!("[FrameProfiler] flush error: {e}");
        }
        self.records.clear();
    }

    fn write_records(&self) -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.output_path)?;

        if file.metadata()?.len() == 0 {
            writeln!(file, "frame,scope,duration_us")?;
        }

        for r in &self.records {
            writeln!(file, "{},{},{}", r.frame, r.scope, r.duration_us)?;
        }

        Ok(())
    }
}

/// Drop guard that records elapsed time into a [`FrameProfiler`] on drop.
/// Obtained from [`FrameProfiler::span`].
pub struct ProfileScope<'a> {
    profiler: &'a mut FrameProfiler,
    name: &'static str,
    start: Instant,
}

impl Drop for ProfileScope<'_> {
    fn drop(&mut self) {
        let duration_us = self.start.elapsed().as_micros() as u64;
        self.profiler.records.push(Record {
            frame: self.profiler.frame,
            scope: self.name,
            duration_us,
        });
    }
}
