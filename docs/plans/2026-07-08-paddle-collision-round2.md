# Fix Paddle Collision — Proper Dissipation + Per-Frame Outlier Detection — Requirements Spec

<claude_instructions>
**For Claude (/goal):** Work through each incomplete task below.
1. Mark a task `[>]` when you begin working on it.
2. Call `dod_check` to verify proofs — do NOT mark proofs manually.
   While iterating on one subtree, pass `nodePath` to verify just that part fast (others are carried, not re-run). A scoped run returns INCOMPLETE, never PASS.
3. A task group is complete when ALL its concrete proofs pass via `dod_check`.
3b. For `manual`/`review` proofs: `dod_check` never auto-prompts — call
    `dod_verify(dod_id, proof_id)` explicitly when verification is actually relevant.
3c. **Manual verification is a HARD GATE.** DoD cannot PASS without it.
    Proofs can pass against wrong code. Visual verification catches what metrics miss.
4. Use `dod_refine` to turn a draft leaf into a concrete proof with a command.
4b. **Refine incrementally per task group, not all at once.** Scoped dod_check is faster
    than full runs — use it. Refining 7 drafts at session end = rubber-stamping.
4c. Use `dod_add_node` to add new nodes discovered during implementation.
5. If a proof cannot be met, use `dod_amend` to modify it with a reason.
5b. **Amending a proof 3+ times is a red flag** — you're probably tuning proofs to pass
    rather than fixing the bug. Re-examine the approach.
5c. Proof commands run on the HOST OS — write OS-correct commands (no bash on Windows).
6. Continue until `dod_check` returns PASS (zero drafts, all proofs pass, manuals verified) — then stop and report done.
6b. **If the approach isn't working, stop and re-interview.** Don't silently pivot to
    a different implementation while keeping the old DoD. The DoD must match what you're doing.

**Self-contained.** All commands run from `C:\Users\siriu\RustroverProjects\Axiom2d` unless noted.

**🔒 Anti-cheat:** Proofs are stored canonically in MCP storage (dod-guard).
`dod_check` executes commands from the canonical copy, not this markdown file.
Editing proof text here has no effect on verification.
Store tampering is **logged and detectable** — each check prints a proof-set fingerprint.
Manual/review proofs are confirmed by the human directly (popup / elicitation) via `dod_verify` —
Claude cannot self-confirm them, and an unrequested one holds the DoD at INCOMPLETE, never PASS.
</claude_instructions>

**Goal:** Fix paddle collision energy injection and phasing by moving machine contacts into project pass with Coulomb friction, SOR averaging, and per-substep correction cap. Add GPU-side per-frame velocity outlier and phasing detection so tests catch problems throughout entire benchmark, not just final frame.

**Date:** 2026-07-08
**Target:** `C:\Users\siriu\RustroverProjects\Axiom2d`
**DoD ID:** `071ff682-aaf7-425c-bdf2-cd925f0a9c0b`
**Last check:** INCOMPLETE (2026-07-08T19:16:38.737Z)

---

## Decisions (locked with user)

<decisions>
## Decisions

1. **Move to project pass**: Machine contacts (kind 0 capsule, kind 3 paddle) write to corrections[i] in project pass, integrated with particle-particle PBD solve. Single dissipation model.

2. **Coulomb friction**: μ=0.3 friction on tangential velocity component of machine contacts, matching particle-particle friction.

3. **SOR averaging**: ω=1.2 / total_contact_count including both particle-particle and machine contacts.

4. **Per-substep correction cap**: Machine corrections capped at particle_radius per substep.

5. **Back-edge expansion**: Paddle OBB expanded on both +tangent and -tangent edges by frame_displacement. Safe now because proper dissipation prevents energy accumulation.

6. **Tangential sweep as friction-limited impulse**: Instead of unbounded position offset, tangential velocity is friction-damped like particle-particle contacts.

7. **Dynamic frame_displacement**: angular_velocity * dt * f32(sub_steps) — not hardcoded * 16.0.

8. **disable_velocity_cap uniform**: Runtime flag to disable MAX_SPEED velocity cap for TDD root-cause testing. Default: cap enabled (safety net).

9. **Sensor volumes unchanged**: Grinder (kind 1) and Heater (kind 2) remain in apply pass — no collision response needed.

10. **Accept all 5 contrarian additions**: TDD root-cause test, streamline verification, observability checks, performance gate at 100k particles, duplication check.
</decisions>

## Current state

<current_state>
## Current State

**Previous fix** (2026-07-07) added MAX_SPEED=1.9 velocity cap in apply pass as band-aid. Machine collision stayed in apply pass with compliance=0.5, no friction, no SOR, no correction cap. frame_displacement hardcoded to dt * 16.0. Test samples velocities every 60 frames and checks only final frame + monotonic tracked_max. The velocity cap masks energy injection — tracked_max sits flat at 1.900 because the cap clips everything. Particles still get launched; they just can't exceed 1.9 in stored velocity. Paddles still phase through.
</current_state>

## Requirements

<requirements>
## Requirements

### Symptoms
- Particles in conveyor buckets launch out with excessive force
- Paddles phase through particles in bulk pile
- Test passes because MAX_SPEED=1.9 cap clips velocity — energy injection hidden, not fixed

### Fix
- Move Capsule (kind=0) and Paddle (kind=3) OBB collision from apply pass into project pass
- Machine contacts use Coulomb friction μ=0.3, SOR ω=1.2 averaging, per-substep correction cap at particle_radius
- Expand paddle OBB on both +tangent and -tangent edges by dynamic frame_displacement
- Tangential sweep as friction-limited velocity impulse
- frame_displacement uses dynamic sub_steps value
- Runtime uniform flag to disable velocity cap for TDD testing

### Test Improvements
- GPU-side compute pass scans all velocities each frame, finds top 20 outliers, writes to small buffer
- GPU-side compute pass checks particles inside paddle OBBs each frame
- CPU reads outlier + phasing buffers every frame (~1KB, negligible cost)
- CPU logs outlier trajectories with frame numbers for debugging
- Test asserts: no speed > 2.0 at ANY frame, zero paddle containment at ANY frame, KE stable throughout 10s run

### NOT in scope
- Machine transform frequency (still 1x/frame on CPU)
- PBF density constraints
- Debug visualization
- Bond formation/breaking changes
- Reaction pass
</requirements>

## Research Notes

<research_notes>
## Research Notes

### Codebase findings
- All particle sim in crates/particle_poc/src/main.rs (~2628 lines) + project.wgsl (~254 lines)
- Machine collision currently in machine_push() called from apply pass (line 238)
- PBD project pass uses SOR ω=1.2, Coulomb friction μ=0.3, correction cap = particle_radius — particle-particle only
- apply pass velocity recompute: (pos - prev) / dt where dt = 1/960 → 960x amplification
- Simulate: 16 substeps × (predict → morton → clear → count → scan → scatter → project → solve_bonds → apply)
- 10 paddles orbiting capsule at CONVEYOR_SPEED=0.45
- Paddle: half_width=0.012, half_height=0.035
- Current test: sample_max_speed() every 60 frames, verify_stability() checks final frame + tracked_max
- MAX_SPEED=1.9 cap in apply pass — band-aid, clips output not input
- frame_displacement = angular_velocity * dt * 16.0 (hardcoded, not sub_steps)

### Contrarian findings
- TDD root-cause test needed — current test proves cap works, not physics
- Streamline — hardcoded 16.0 must be fixed; old machine_push may remain as dead code
- Observability — per-frame GPU→CPU readback is the entire verification strategy
- Performance — 16x cost increase from moving to project pass; must pin particle count
- Duplication — OBB transform math risk of appearing in both project and apply
</research_notes>

## Open Questions

<open_questions>
## Open Questions

None — all design decisions resolved during interview.
</open_questions>

---

## Definition of Done

<definition_of_done>

### Code Quality [x]

  - [x] Proof: `cargo clippy -p particle_poc --no-deps -- -D warnings -A clippy::struct_excessive_bools` → Clippy passes with zero warnings on particle_poc crate (struct_excessive_bools allowed: pre-existing on App struct)
  - [x] Proof: `cargo fmt --all -- --check` → All code properly formatted
  - [x] Proof: `cargo test` → All 1725+ tests pass, no regressions

### Paddle Collision Fix [~]

  **TDD Root-Cause Test** [~]

    - [ ] Proof: `findstr /C:"test-paddle-root-cause" crates\particle_poc\src\main.rs` → --test-paddle-root-cause CLI flag defined and wired to disable velocity cap uniform
    - [~] **Draft**: cargo run --release -p particle_poc -- --test-paddle-root-cause exits non-zero on current code because velocity spikes exceed 2.0 without MAX_SPEED cap. Proves bug exists independent of band-aid.
    - [~] **Draft**: cargo run --release -p particle_poc -- --test-paddle-root-cause exits 0 after fix because Coulomb friction + SOR + correction cap limit energy transfer without needing velocity cap.
    - [~] **Draft**: Root-cause test verifies: nan_count==0, max_speed<2.0, tracked_max<2.0, zero OOB. Prints explicit FAIL messages and exits 1 on failure. Assertions check physics output, not cap output.
  **GPU Outlier Detection** [~]

    - [~] **Draft**: New compute shader entry point @compute @workgroup_size(256) fn detect_outliers — scans all particle velocities, writes top 20 (particle_index, pos_x, pos_y, vel_x, vel_y, speed) to outlier_buffer. Uses atomic compare-and-swap for top-k selection.
    - [~] **Draft**: Every frame in test mode: copy outlier_buffer to staging, map_async, read top-20 data. Not every 60 frames — EVERY frame during measurement window.
    - [~] **Draft**: Each frame prints frame number + top outliers: index, position (x,y), velocity (vx,vy), speed. Builds per-particle launch trajectory over time. Printed even below threshold for trend visibility.
    - [~] **Draft**: Test asserts max tracked speed across ALL frames is < 2.0. If any outlier exceeds threshold at any frame, test fails with frame number, particle index, position, velocity. Uses outlier buffer data, not end-of-benchmark readback.
  **GPU Phasing Detection** [~]

    - [~] **Draft**: New compute shader entry point fn detect_phasing — for each paddle (kind 3), checks if any particle center is inside expanded paddle OBB. Writes (paddle_index, particle_index, penetration_depth) to phasing_buffer. Uses atomic counter for count.
    - [~] **Draft**: Every frame in test mode: copy phasing_buffer to staging, map_async, read phasing count and entries. Logs phasing events with frame number, paddle index, particle index, penetration depth.
    - [~] **Draft**: Test asserts phasing_count == 0 at EVERY frame. If any particle is inside any paddle OBB at any time, test fails with full diagnostic information (frame, paddle, particle, penetration).
  **Energy Stability Tracking** [~]

    - [~] **Draft**: Track total kinetic energy (0.5 * m * v^2 summed over all particles) per frame. Log KE each frame during measurement window. Store KE history for trend analysis.
    - [~] **Draft**: Test asserts KE does not monotonically increase over 10s run. In dissipative system, KE trend must be flat or decreasing. If KE trends upward, test fails with KE-vs-time data.
  **OBB Response Math Redesign** [~]

    - [~] **Draft**: Machine contacts (kind 0 capsule, kind 3 paddle) processed in project pass alongside particle-particle contacts. Write to same corrections[i] buffer. SOR omega multiplies combined correction. Part of unified contact solve.
    - [~] **Draft**: Tangential component of machine contact velocity damped by Coulomb friction coefficient mu=0.3. Same friction model as particle-particle contacts. Prevents energy injection from tangential sweep.
    - [~] **Draft**: Total contact count for SOR omega=1.2 multiplier includes both particle-particle and machine contacts. correction *= SOR_OMEGA / f32(total_contacts). Prevents over-correction when particle contacts multiple machines.
    - [~] **Draft**: Machine correction magnitude capped at particle_radius per substep. Prevents single large penetration from producing unbounded correction. Same cap as particle-particle contacts.
    - [~] **Draft**: Paddle OBB expanded on both +tangent and -tangent edges by frame_displacement. Catches particles entering from pile side. Previously regressed KE but now safe with Coulomb friction + SOR + correction cap.
    - [~] **Draft**: Conveyor carry converted from unbounded position offset to friction-limited velocity impulse. Tangential velocity component clamped by Coulomb friction limit: max_tangential_correction = mu * normal_correction. surf_speed integrated into velocity delta, not raw position offset.
    - [~] **Draft**: frame_displacement = mach.angular_velocity * params.dt * f32(sub_steps). Uses actual sub_steps from uniform, not hardcoded 16.0.
    - [~] **Draft**: Uniform flag in sim params struct. When set (disable_velocity_cap != 0), MAX_SPEED check skipped in apply pass. Default 0 (cap enabled). Used by --test-paddle-root-cause to test physics without band-aid.
    - [~] **Draft**: Grinder (kind 1) and Heater (kind 2) machine collision remains in apply pass or is skipped. Read-only sensor volumes used by reaction pass — no collision response needed.
  **Stability Verification** [~]

    - [~] **Draft**: cargo run --release -p particle_poc -- --test-paddle-stability exits 0. All new checks pass: zero outliers above 2.0, zero phasing events, KE stable. 10-second run with 1000 particles near conveyor.
    - [~] **Draft**: cargo run --release -p particle_poc -- --benchmark exits 0. verify_stability() passes: NaN=0, OOB=0, KE outliers=0, max_speed<2.0, tracked_max<2.0. 10-second run with default particle count.
    - [x] Proof: `cargo run --release -p particle_poc -- --test-bond-form && cargo run --release -p particle_poc -- --test-bond-constrain && cargo run --release -p particle_poc -- --test-bond-break` → All three bond tests (form, constrain, break) exit 0 — machine collision changes don't break bond physics
  **Streamline** [ ]

    - [ ] Proof: `findstr /C:"16.0" crates\particle_poc\src\shaders\project.wgsl` → frame_displacement uses f32(sub_steps) not hardcoded 16.0 — findstr returns exit code 1 (no matches for 16.0 literal in project.wgsl)
    - [ ] Proof: `findstr /C:"disable_velocity_cap" crates\particle_poc\src\main.rs crates\particle_poc\src\shaders\project.wgsl` → Uniform flag exists in both Rust struct definition and WGSL shader — cap is controllable runtime flag, not always-on band-aid
  **Observability** [ ]

    - [ ] Proof: `findstr /C:"detect_outliers" crates\particle_poc\src\shaders\project.wgsl` → detect_outliers compute entry point declared in WGSL shader
    - [ ] Proof: `findstr /C:"detect_phasing" crates\particle_poc\src\shaders\project.wgsl` → detect_phasing compute entry point declared in WGSL shader
    - [ ] Proof: `findstr /C:"outlier" crates\particle_poc\src\main.rs` → Main.rs contains outlier buffer readback code with staging buffer, map_async, and println logging
    - [ ] Proof: `findstr /C:"phasing" crates\particle_poc\src\main.rs` → Main.rs contains phasing buffer readback code with staging buffer, map_async, and println logging
  **Performance** [~]

    - [~] **Draft**: cargo run --release -p particle_poc -- --benchmark --particles 100000 reports average frame time <= 16.67ms (60 FPS) with new per-substep machine collision. Must hold at full 100k particle count.
  **Duplication** [~]

    - [~] **Draft**: findstr /C:"cos_angle" project.wgsl shows OBB transform math in one location only (shared helper or project pass), not duplicated in both project and apply passes. Machine collision logic lives in exactly one place.
  **Integration** [~]

    - [ ] Proof: `findstr /C:"detect_outliers" crates\particle_poc\src\main.rs` → detect_outliers compute pipeline created and dispatch called in simulate() loop
    - [ ] Proof: `findstr /C:"detect_phasing" crates\particle_poc\src\main.rs` → detect_phasing compute pipeline created and dispatch called in simulate() loop
    - [ ] Proof: `findstr /C:"disable_velocity_cap" crates\particle_poc\src\main.rs` → disable_velocity_cap uniform value written to GPU buffer, controlled by --test-paddle-root-cause flag
    - [~] **Draft**: cargo run --release -p particle_poc -- --test-paddle-stability exits 0 with all new detection passes (outlier, phasing, KE tracking). Exercises full system end-to-end: WGSL compute passes -> GPU buffers -> CPU readback -> assertions.

### Manual Verification [x]

  - [~] Proof: `manual` → Peer review of WGSL shader changes (project pass restructuring, Coulomb friction, SOR averaging), Rust test additions (outlier/phasing detection), and frame_displacement fix
  - [~] Proof: Manual — Run particle_poc without flags. Observe conveyor: particles stay in bucket between paddles, no launching, no phasing through paddles. Verify rendering looks correct. _(awaiting human verification)_

</definition_of_done>

## Open risks

<open_risks>
## Risks

- Moving machine collision to project pass runs OBB checks 16x per substep instead of once per frame. With 13 machines and 100k particles: ~20.8M OBB checks per frame. Particle-particle neighbor search already ~40M checks per frame — machine overhead should be < 33% increase.
- WGSL changes need careful testing. GPU shader bugs produce silent corruption (NaN, position jumps) not Rust panics.
- Back-edge expansion re-added after previously regressing KE from 67 to 191 outliers. Now safe because: (1) Coulomb friction dissipates tangential energy, (2) SOR averaging limits per-contact energy transfer, (3) per-substep cap prevents large single-step corrections. Previous regression was from unbounded energy injection — dissipation fixes root cause.
</open_risks>
