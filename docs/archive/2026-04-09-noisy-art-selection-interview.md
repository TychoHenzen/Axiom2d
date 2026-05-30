# Noisy Art Selection — Requirements Spec

> **For Claude:** This spec was produced by /interview. Use /writing-plans to expand into an implementation plan, or /tdd to implement directly.

**Goal:** Replace deterministic art selection with signature-seeded Gaussian-weighted probabilistic sampling so booster packs produce visually diverse cards.

**Date:** 2026-04-09

---

## Requirements

**What it does:**
- `select_art_for_signature` becomes probabilistic: seeds a `ChaCha8Rng` from the card's signature bytes, computes Gaussian weights `exp(-d²/(2σ²))` for every art entry by signature distance, then samples one entry from that weighted distribution
- At distance 0 (exact signature match), the closest art has ~25% probability — so in a 10-card booster, ~2–3 cards get the matching art and the rest scatter across nearby entries
- Deterministic per-signature: same signature always picks the same art (hash-like)
- Neighboring signatures diverge chaotically due to the hash-seeded RNG
- Independent per-card: no pack-level dedup or repulsion

**What it does NOT do:**
- No new config structs or tuning parameters exposed to callers
- No changes to the `ShapeRepository` API
- No changes to the sampling/booster code — only art selection changes

**Function signature:** `select_art_for_signature(&CardSignature, &ShapeRepository) -> Option<&ArtEntry>` stays the same. The RNG is created internally from the signature.

**Tuning:** Single `const SELECTION_SIGMA: f32` hardcoded in `art_selection.rs`, tuned so the peak probability at distance 0 is ~25% given 361 entries.

**Algorithm:** Signature-seeded Gaussian soft-Voronoi
- Deterministic: `ChaCha8Rng` seeded from signature bytes
- Gaussian falloff: `weight = exp(-distance² / (2σ²))`
- Sharp cutoff is desired — distant art getting near-zero probability is fine

## Subtask Checklist

- [ ] Subtask 1: Add `rand` + `rand_chacha` imports to `art_selection.rs`, seed `ChaCha8Rng` from signature axes bytes
- [ ] Subtask 2: Compute Gaussian weights for all repo entries: `exp(-dist² / (2σ²))`
- [ ] Subtask 3: Weighted random sampling from the weight distribution using the seeded RNG
- [ ] Subtask 4: Tune `SELECTION_SIGMA` so peak probability at distance 0 ≈ 25% with 361 entries
- [ ] Subtask 5: Update existing tests in `card_art_selection.rs` to match new probabilistic behavior
- [ ] Subtask 6: Add test: exact-match signature produces the matching art with expected ~25% frequency over many trials
- [ ] Subtask 7: Add test: two nearby but distinct signatures produce different art selections (chaos property)
- [ ] Subtask 8: Verify build with `cargo.exe build -p card_game_bin`

## Research Notes

- **Production call site:** `crates/card_game/src/card/rendering/spawn_table_card.rs:115`
- **Test call sites:** `crates/card_game/tests/suite/card_art_selection.rs` (3 tests)
- **Repository:** 361 art entries in `ShapeRepository`, stored in `BTreeMap<&'static str, ArtEntry>`
- **Distance:** Euclidean in 8D, `CardSignature::distance_to` at `signature/types.rs:124`
- **Existing deps:** `rand` + `rand_chacha` already in workspace deps
- **Signature axes:** 8 × f32 in [-1, 1], accessible via `CardSignature::axes() -> [f32; 8]`

## Open Questions

None — all requirements confirmed.
