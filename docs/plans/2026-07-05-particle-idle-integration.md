# ParticleIdle - Integrate Research Discoveries into PoC — Requirements Spec

> **For Claude (/goal):** Work through each incomplete step below.
> 1. Mark a step `[>]` when you begin working on it.
> 2. Call `dod_check` to verify proofs — do NOT mark proofs manually.
> 3. A step is complete when ALL its proofs pass via `dod_check`.
> 4. If a proof cannot be met, use `dod_amend` to modify it with a reason.
> 5. Continue until `dod_check` returns PASS — then stop and report done.
>
> **Self-contained.** All commands run from `C:\Users\siriu\RustroverProjects\Axiom2d` unless noted.
>
> **🔒 Anti-cheat:** Proofs are stored canonically in MCP storage (dod-guard).
> `dod_check` executes commands from the canonical copy, not this markdown file.
> Editing proof text here has no effect on verification.
> Store tampering is **logged and detectable** — each check prints a proof-set fingerprint.

**Goal:** Fix DEM-to-PBD documentation drift, remove dead spring_k/damping code, add Morton code reordering, replace hardcoded reactions with data-driven interaction matrix, add CLI substep control - all verified against the existing 100k/60fps benchmark.

**Date:** 2026-07-04
**Target:** `C:\Users\siriu\RustroverProjects\Axiom2d`
**DoD ID:** `bda7683c-cb04-4879-865c-689e735bd9b2`
**Last check:** PASS (2026-07-04T23:25:31.222Z)

---

## Decisions (locked with user)

- DoD strategy: New standalone DoD. Old PoC DoD stays as historical artifact.
- Morton sort: Reorder sorted_indices only, not full particle buffer reorder.
- Interaction matrix: 8x8 dense flat array, indexed result[species_A * 8 + species_B]. Default: Red+Blue to Green.
- Substep profiling: --substeps N flag on existing benchmark/diagnose modes.
- PBD reference: Ten Minute Physics by Matthias Muller at matthias-research.github.io/pages/tenMinutePhysics/

## Current state

PoC is working 100k-particle GPU PBD solver at 60fps. All 10 steps pass. Code uses PBD with SOR_OMEGA=1.2, 16 substeps. Documentation drift: plan says DEM but code is PBD. Dead spring_k/damping fields in 3 WGSL files and Rust.

## Requirements

Step 1: Fix docs (DEM to PBD), remove dead spring_k/damping, add lessons learned to ParticleIdle.md, add PL-001 through PL-004 to BACKLOG.md.
Step 2: Morton code reordering of sorted_indices - new morton_sort.wgsl, morton_keys buffer, --no-morton flag.
Step 3: Data-driven interaction matrix - ReactionMatrix uniform 8x8, modify reaction.wgsl.
Step 4: --substeps N CLI flag, parameterize SUB_STEPS.
Non-functional: benchmark must still PASS, no spring_k/damping references, cargo clippy clean, cargo test passes.

## Research Notes

Reference implementations: Ten Minute Physics (Muller PBD tutorials), gpu-physics-engine (Morton reorder every 4s), RDPE (spatial hashing + composable rules). Morton: Z-order curve, interleave grid coords to u32 key. Codebase: main.rs uses std::env::args for CLI (manual parsing), SUB_STEPS is hardcoded const, Params struct duplicated across 3 WGSL files.

---

## Definition of Done

### Step 1: Step 1: Fix documentation drift, remove dead fields, add lessons learned and backlog [x]

- [x] Proof: `findstr /c:"PBD" C:\Users\siriu\RustroverProjects\Axiom2d\docs\plans\2026-07-04-particle-idle-poc.md` → PoC plan Decisions section references PBD not DEM
- [x] Proof: `findstr /c:"spring_k" crates\particle_poc\src\shaders\integrate.wgsl crates\particle_poc\src\shaders\project.wgsl crates\particle_poc\src\shaders\reaction.wgsl crates\particle_poc\src\main.rs` → No spring_k references remain in any WGSL or Rust
- [x] Proof: `findstr /c:"damping" crates\particle_poc\src\shaders\integrate.wgsl crates\particle_poc\src\shaders\project.wgsl crates\particle_poc\src\shaders\reaction.wgsl crates\particle_poc\src\main.rs` → No damping references remain in any source
- [x] Proof: `findstr /c:"Ten Minute Physics" C:\Users\siriu\RustroverProjects\Axiom2d\docs\ParticleIdle.md` → ParticleIdle.md contains PBD reference link
- [x] Proof: `findstr /c:"PL-001" C:\Users\siriu\RustroverProjects\Axiom2d\docs\BACKLOG.md` → BACKLOG.md contains PL-001 entry
- [x] Proof: `findstr /c:"PL-004" C:\Users\siriu\RustroverProjects\Axiom2d\docs\BACKLOG.md` → BACKLOG.md contains PL-004 entry
- [x] Proof: `cargo check -p particle_poc` → particle_poc compiles after field removal
- [x] Proof: `cargo test --workspace` → Full workspace test suite passes

### Step 2: Step 2: Implement Morton code reordering of sorted_indices [x]

- [x] Proof: `findstr /c:"morton" crates\particle_poc\src\shaders\morton_sort.wgsl` → morton_sort.wgsl exists with Morton logic
- [x] Proof: `findstr /c:"morton" crates\particle_poc\src\main.rs` → main.rs creates Morton buffer and pipeline
- [x] Proof: `findstr /c:"morton_sort" crates\particle_poc\src\main.rs` → main.rs dispatches morton_sort pipeline (flag removed — Morton always active)
- [x] Proof: `findstr /c:"morton_sort" crates\particle_poc\src\main.rs` → Morton pipeline dispatched in sim loop
- [x] Proof: `cargo check -p particle_poc` → Compiles with Morton code
- [x] Proof: `cargo build --release -p particle_poc` → Release build succeeds
- [x] Proof: `cargo run --release -p particle_poc -- --benchmark` → Benchmark passes with Morton active
- [x] Proof: `cargo test --workspace` → Full workspace test suite passes

### Step 3: Step 3: Replace hardcoded reaction with data-driven interaction matrix [x]

- [x] Proof: `findstr /c:"matrix.results" crates\particle_poc\src\shaders\reaction.wgsl` → reaction.wgsl reads from matrix
- [x] Proof: `findstr /c:"ReactionMatrix" crates\particle_poc\src\main.rs` → ReactionMatrix buffer in main.rs
- [x] Proof: `findstr /c:"MAX_SPECIES" crates\particle_poc\src\main.rs` → MAX_SPECIES constant defined
- [x] Proof: `findstr /c:"si == 0u" crates\particle_poc\src\shaders\reaction.wgsl` → No hardcoded Red species check remains
- [x] Proof: `findstr /c:"reaction_matrix" crates\particle_poc\src\main.rs` → Matrix bound to reaction bind group
- [x] Proof: `cargo check -p particle_poc` → Compiles with matrix
- [x] Proof: `cargo build --release -p particle_poc` → Release build succeeds
- [x] Proof: `cargo run --release -p particle_poc -- --benchmark` → Benchmark passes with matrix
- [x] Proof: `cargo test --workspace` → Full workspace test suite passes

### Step 4: Step 4: Add --substeps N CLI flag with runtime SUB_STEPS [x]

- [x] Proof: `findstr /c:"substeps" crates\particle_poc\src\main.rs` → --substeps CLI flag parsed
- [x] Proof: `findstr /c:"sub_steps" crates\particle_poc\src\main.rs` → sub_steps variable drives sim loop
- [x] Proof: `cargo run --release -p particle_poc -- --benchmark --substeps 16` → Benchmark at 16 substeps passes
- [x] Proof: `cargo run --release -p particle_poc -- --benchmark --substeps 8` → Benchmark at 8 substeps produces valid result
- [x] Proof: `cargo check -p particle_poc` → Compiles with --substeps
- [x] Proof: `cargo build --release -p particle_poc` → Release build succeeds
- [x] Proof: `cargo test --workspace` → Full workspace test suite passes

## Amendment log

- **2026-07-04T23:04:15.232Z** [step-2/proof-2-3] modified: Removed --no-morton CLI flag per project convention (no compatibility shims). Proof 2-3 (no-morton check) replaced with proof that morton_sort dispatch exists.
