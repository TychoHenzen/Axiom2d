#![allow(clippy::unwrap_used)]

// simplify is a private module (not in pub mod list of lib.rs).
// All tests live inline in src/simplify.rs: 8 tests covering rdp_open and rdp_simplify_closed.
// This module exists to satisfy the single-binary test consolidation pattern.
