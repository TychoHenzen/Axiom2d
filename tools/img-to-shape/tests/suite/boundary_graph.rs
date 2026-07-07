#![allow(clippy::unwrap_used)]

// boundary_graph is a private module (not in pub mod list of lib.rs).
// All tests live inline in src/boundary_graph.rs: 6 tests covering ChainData, ChainRef,
// Face, BoundaryGraph, face_vertices, and extract_region_faces.
// This module exists to satisfy the single-binary test consolidation pattern.
