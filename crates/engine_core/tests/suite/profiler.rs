#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

use engine_core::profiler::FrameProfiler;

// ── record_phase ─────────────────────────────────────────────────────────────

#[test]
fn when_record_phase_called_then_record_count_increases() {
    // Arrange
    let mut profiler = FrameProfiler::new(9999, PathBuf::from("unused.csv"));

    // Act
    profiler.record_phase("Input", 100);
    profiler.record_phase("Update", 200);

    // Assert
    assert_eq!(profiler.record_count(), 2);
}

// ── span ─────────────────────────────────────────────────────────────────────

#[test]
fn when_span_dropped_then_record_count_increases() {
    // Arrange
    let mut profiler = FrameProfiler::new(9999, PathBuf::from("unused.csv"));

    // Act
    {
        let _s = profiler.span("my_span");
    }

    // Assert
    assert_eq!(profiler.record_count(), 1);
}

// ── end_frame / flush ─────────────────────────────────────────────────────────

#[test]
fn when_end_frame_reaches_flush_interval_then_csv_written() {
    // Arrange
    let path = std::env::temp_dir().join("axiom2d_profiler_test_flush.csv");
    let _ = std::fs::remove_file(&path);
    let mut profiler = FrameProfiler::new(1, path.clone());
    profiler.record_phase("Input", 100);
    profiler.record_phase("Update", 200);

    // Act — frame increments to 1, 1 % 1 == 0, triggers flush
    profiler.end_frame();

    // Assert
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("frame,scope,duration_us"), "missing header");
    assert!(content.contains("0,Input,100"), "missing Input record");
    assert!(content.contains("0,Update,200"), "missing Update record");

    let _ = std::fs::remove_file(&path);
}

#[test]
fn when_flush_occurs_then_in_memory_records_cleared() {
    // Arrange
    let path = std::env::temp_dir().join("axiom2d_profiler_test_clear.csv");
    let _ = std::fs::remove_file(&path);
    let mut profiler = FrameProfiler::new(1, path.clone());
    profiler.record_phase("Input", 100);

    // Act
    profiler.end_frame();

    // Assert
    assert_eq!(profiler.record_count(), 0);

    let _ = std::fs::remove_file(&path);
}

#[test]
fn when_flush_interval_not_reached_then_records_not_cleared() {
    // Arrange
    let mut profiler = FrameProfiler::new(10, PathBuf::from("unused.csv"));
    profiler.record_phase("Input", 100);

    // Act — frame becomes 1, 1 % 10 != 0
    profiler.end_frame();

    // Assert
    assert_eq!(profiler.record_count(), 1);
}

#[test]
fn when_flush_path_invalid_then_no_panic() {
    // Arrange — path inside a non-existent directory
    let path = std::env::temp_dir()
        .join("axiom2d_no_such_dir_xyz")
        .join("profiler.csv");
    let mut profiler = FrameProfiler::new(1, path);
    profiler.record_phase("Input", 100);

    // Act + Assert — IO error must not panic
    profiler.end_frame();
}

#[test]
fn when_second_flush_appends_to_existing_file_without_duplicate_header() {
    // Arrange
    let path = std::env::temp_dir().join("axiom2d_profiler_test_append.csv");
    let _ = std::fs::remove_file(&path);
    let mut profiler = FrameProfiler::new(1, path.clone());
    profiler.record_phase("Input", 100);
    profiler.end_frame(); // frame 1 → flush

    profiler.record_phase("Update", 200);
    profiler.end_frame(); // frame 2 → flush

    // Assert — header appears exactly once
    let content = std::fs::read_to_string(&path).unwrap();
    let header_count = content.matches("frame,scope,duration_us").count();
    assert_eq!(header_count, 1, "header must appear exactly once");
    assert!(content.contains("1,Update,200"), "second flush records frame 1");

    let _ = std::fs::remove_file(&path);
}
