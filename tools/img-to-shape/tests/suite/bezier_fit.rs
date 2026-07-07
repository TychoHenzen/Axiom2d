#![allow(clippy::unwrap_used)]

// bezier_fit is a private module (not in pub mod list of lib.rs).
// All tests live inline in src/bezier_fit.rs: 10 tests covering fit_bezier_path and fit_bezier_segment.
// This module exists to satisfy the single-binary test consolidation pattern.
