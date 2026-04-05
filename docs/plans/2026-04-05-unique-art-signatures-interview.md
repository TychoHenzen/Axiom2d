# Unique Art Signatures — Requirements Spec

> **For Claude:** This spec was produced by /interview. Use /tdd to implement directly.

**Goal:** Replace the broken `seed_signature_from_name` hash in `codegen.rs` with one that guarantees unique, well-distributed signatures across similar names.

**Date:** 2026-04-05

---

## Requirements

- **Root cause:** `seed_signature_from_name` runs 8 near-identical FNV-1a loops from correlated initial seeds, then loses ~40 bits of precision in the `u64 → f32` conversion. Similar names (same long prefix, different short suffix) produce hash values within `2^40` of each other, collapsing to identical f32 values across all 8 axes.
- **Fix:** Replace with: (1) hash the name once to a u64 seed, then (2) drive splitmix64 8 times to get 8 independent u64 values, then (3) extract the top 24 bits for maximum f32 precision. Splitmix64 has strong avalanche and no inter-axis correlation.
- No changes to entries with explicit non-zero `signature_axes` in the manifest.
- Fix is in `tools/img-to-shape/src/codegen.rs` only. `repository.rs` stays untouched (user re-runs the GUI).
- Must remain deterministic: same name → same 8 axes every run.

## Subtask Checklist

- [ ] Replace `seed_signature_from_name` with splitmix64-based implementation
- [ ] Add tests: uniqueness across 130 `barbarian_icons_*_t`-style names, and cross-axis independence

## Research Notes

- `seed_signature_from_name` is at line 1084 in `tools/img-to-shape/src/codegen.rs`
- Called from `generate_repository_module` when `entry.signature_axes == [0.0; 8]`
- Existing tests in `tools/img-to-shape/tests/suite/codegen.rs`
- splitmix64: seed with `name_hash.wrapping_add(0x9e3779b97f4a7c15)`, mix with two multiply-xorshift rounds, repeat 8×
- Top-24-bit float extraction: `(v >> 40) as f32 / (1u64 << 24) as f32 * 2.0 - 1.0`
