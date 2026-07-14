# Phase 4 Code Review — 2026-07-13

**Scope**: particle_poc Phase 4 + CI quality.yml changes. 23 files, 1387 insertions committed + 805 insertions uncommitted.
**Effort**: max recall (10 angles × up to 8 candidates → 1-vote verify → sweep)

---

## Critical (2)

### 1. SDF texture never uploaded to GPU
**File**: `crates/particle_poc/src/state.rs:335`
**Failure**: `sdf_grid` CPU vec initialized to all-1.0, `sdf_dirty` starts `false`. `render()` L1761 only uploads when `sdf_dirty == true`. Until user paints, `project.wgsl` L320 `textureLoad(sdf_tex, ...)` reads undefined GPU memory. `sdf_params_buf` same — never populated until first SDF dirty path.

Same defect in `HeadlessCapture::try_new()` (capture.rs L140) — never uploads SDF at all, struct lacks `sdf_grid` field.
**Fix**: Upload initial all-1.0 grid + SdfParams at end of `State::new()` and `HeadlessCapture::try_new()`.

### 2. update_machines() called twice per frame in paddle test
**File**: `crates/particle_poc/src/state.rs:1675,1702`
**Failure**: `render()` L1675 calls `update_machines()` unconditionally. L1702 calls it again inside `test_paddle_stability` branch. Machine time advances at 2× speed, Rapier2D stepped 2/60s per frame, second call reads stale counter staging data (no intervening GPU copy).
**Fix**: Remove duplicate call at L1702.

---

## High (2)

### 3. CI soft-gate crashes on empty coverage output
**File**: `.github/workflows/quality.yml:335`
**Failure**: Coverage job fails → `needs.coverage.outputs.coverage_pct = ""`. Bash: `echo ' < 78' | bc -l` → syntax error, `cov_compare=""`. Then `[ "" -eq 1 ]` → `[: integer expression expected` → exit 2 under `set -e` (L296). Soft-gate step fails spuriously.
**Fix**: Add `if [ -z "$cov_current" ]; then echo "::error::coverage job missing"; exit 1; fi` before bc calls.

### 4. HeadlessCapture uses hardcoded conveyor constants
**File**: `crates/particle_poc/src/capture.rs:837`
**Failure**: `update_machines()` uses `CONVEYOR_ANGLE_DEG.to_radians()` (L837) and `CAPSULE_HALF_LEN` (L839,858,884,910,915). `State::update_machines` uses dynamic endpoints (L897-905). HeadlessCapture struct also lacks `conveyor_endpoints` field. Geometry diverges if endpoints != default.
**Fix**: Port endpoint-based computation to capture.rs, or extract shared function.

---

## Medium (3)

### 5. SDF textureLoad OOB on boundary
**File**: `crates/particle_poc/src/shaders/project.wgsl:318`
**Failure**: Guard `u <= 1.0` allows `u==1.0` → `u32(256.0) = 256` OOB for 256×256 texture. Wall clamp at L310 caps `u ≤ 0.99875` with current params — latent bug. WGSL OOB returns zero (silent gap).
**Fix**: Change `<= 1.0` to `< 1.0` on both checks, or clamp `sample_coord`.

### 6. Blue spawner Recipe input_species mismatch
**File**: `crates/particle_poc/src/lib.rs:1429`
**Failure**: Blue spawner (species 1) Recipe has `input_species: 0` while `MachineDef.input_species: 1` at L1426. Currently harmless (`input_count==0` skips recipe), but data is wrong.
**Fix**: `input_species: 1` on Blue spawner Recipe.

### 7. spawner_timers hardcoded size 2
**File**: `crates/particle_poc/src/state.rs:465`
**Failure**: `spawner_timers: vec![0.0; 2]` — if 3rd spawner added, 3rd's timer never advances (bounds guard at L1062). Silent non-spawning.
**Fix**: Size from spawner count after `init_machines()`.

---

## Low (3)

### 8. Any button release cancels drag
**File**: `crates/particle_poc/src/state.rs:94`
**Failure**: In Drag mode, any `MouseInput` release clears `dragging`/`dragging_endpoint`. Right-click while left-dragging → release drops grip.
**Fix**: Store which button started the drag, only clear on that button's release.

### 9. paint_sdf brush assumes square world
**File**: `crates/particle_poc/src/state.rs:225`
**Failure**: `brush_px` uses `BRUSH_RADIUS / (2.0 * half_w)` for both X and Y pixel conversion. For non-square bounds (half_w ≠ half_h), brush becomes elliptical in world space. Currently correct at 0.8×0.8.
**Fix**: Use `half_w` for X, `half_h` for Y, or add `BRUSH_RADIUS_PX` constant.

### 10. paint_sdf hardcoded blend factor
**File**: `crates/particle_poc/src/state.rs:244`
**Failure**: Blend factor `t = 0.3` hardcoded. No mechanism to adjust brush opacity.
**Fix**: Extract as `const SDF_BRUSH_BLEND: f32 = 0.3;`.

---

## Refuted

- **SimParams 68-byte alignment**: WGSL scalar-only struct aligns to max member alignment (4), not vec4(16). 68 % 4 = 0. Pre-existing 56-byte struct also worked.
- **`hit_test_machine` null pointer panic**: Code at L134-136 uses `let Some(body) = ... else { continue; }`, not `?` operator. Correct pattern.
- **SDF not wired in WGSL**: `project.wgsl` L58-59 declares `sdf_tex` at binding 11 and `sdf_params` at binding 12. SDF is wired correctly for GPU consumption — the missing piece is the CPU→GPU upload path (#1).
- **Kill barrier corpse accumulation**: `paint_sdf` returns `return` from apply() — particle position reset to top, NOT zeroed. Same particle slot reused. `particle_count` stays constant; correct for recycling not needing decrement.

## Already addressed (pre-existing, not Phase 4)

- `capture.rs spawn_grid` single-row bug: pre-existing, `let _ = i / cols` at L286 discards row. Not introduced by this diff.
- `no_benchmark` field: read at state.rs L1754 `if !self.no_benchmark { self.spawn(); }` — correctly wired.
- `kind as u32 == 0` magic numbers: state.rs L924-929 already replaced with `MachineKind::Conveyor`/`MachineKind::Spawner` comparisons.
- `SPAWN_RATE`: already has `#[allow(dead_code)]` annotation at lib.rs L14.

## Verification

- `grep -n "update_machines" crates/particle_poc/src/state.rs` → L1675 + L1702 confirmed
- `grep -n "sdf_dirty" crates/particle_poc/src/state.rs` → set at L250, checked at L1761
- `grep -n "write_texture" crates/particle_poc/src/state.rs` → only at L1764, inside `if self.sdf_dirty`
- `grep "cov_" .github/workflows/quality.yml` → no null guard at L335-338
- `grep "CONVEYOR_ANGLE_DEG\|CAPSULE_HALF_LEN" crates/particle_poc/src/capture.rs` → L837,839,858,884,910,915
- `grep "texture_2d" crates/particle_poc/src/shaders/project.wgsl` → L58 present
- `grep "sdf_tex\|SdfParams\|binding(11)\|binding(12)" crates/particle_poc/src/shaders/project.wgsl` → L58-59, L320-327 present
