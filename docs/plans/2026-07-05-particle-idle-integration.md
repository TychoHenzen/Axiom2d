# ParticleIdle - Integrate Research Discoveries into PoC - Requirements Spec

<claude_instructions>
**For the implementer:** Work through each task below.
1. Mark a task `[>]` when you begin working on it.
2. Call `dod_check` to verify proofs - do NOT mark proofs manually.
3. A task group is complete when ALL its concrete proofs pass via `dod_check`.
4. Use `dod_refine` to turn a draft leaf into a concrete proof or subdivide into child tasks.
5. If a proof cannot be met, use `dod_amend` to modify it with a reason.
6. Continue until `dod_check` returns PASS - then stop and report done.

**Behavioral predicates only.** Each proof is a concrete behavioral claim.
Read failure diagnoses carefully - they tell you WHAT went wrong and what to fix.
Proofs run on the HOST OS - write OS-correct commands (no bash on Windows).

**CWD:** `C:\Users\siriu\RustroverProjects\Axiom2d`

**Anti-cheat:** Proofs stored canonically in MCP storage.
`dod_check` executes commands from the canonical copy, not this markdown file.
</claude_instructions>

**Goal:** Fix DEM-to-PBD documentation drift, remove dead spring_k/damping code, add Morton code reordering, replace hardcoded reactions with data-driven interaction matrix, add CLI substep control - all verified against the existing 100k/60fps benchmark.

**Date:** 2026-07-04
**Target:** `C:\Users\siriu\RustroverProjects\Axiom2d`
**DoD ID:** `bda7683c-cb04-4879-865c-689e735bd9b2`
**Last check:** PASS (2026-07-04T23:25:31.222Z)

---

## Decisions (locked with user)

<decisions>
- DoD strategy: New standalone DoD. Old PoC DoD stays as historical artifact.
- Morton sort: Reorder sorted_indices only, not full particle buffer reorder.
- Interaction matrix: 8x8 dense flat array, indexed result[species_A * 8 + species_B]. Default: Red+Blue to Green.
- Substep profiling: --substeps N flag on existing benchmark/diagnose modes.
- PBD reference: Ten Minute Physics by Matthias Muller at matthias-research.github.io/pages/tenMinutePhysics/
</decisions>

## Current state

<current_state>
PoC is working 100k-particle GPU PBD solver at 60fps. All 10 steps pass. Code uses PBD with SOR_OMEGA=1.2, 16 substeps. Documentation drift: plan says DEM but code is PBD. Dead spring_k/damping fields in 3 WGSL files and Rust.
</current_state>

## Requirements

<requirements>
Step 1: Fix docs (DEM to PBD), remove dead spring_k/damping, add lessons learned to ParticleIdle.md, add PL-001 through PL-004 to BACKLOG.md.
Step 2: Morton code reordering of sorted_indices - new morton_sort.wgsl, morton_keys buffer, --no-morton flag.
Step 3: Data-driven interaction matrix - ReactionMatrix uniform 8x8, modify reaction.wgsl.
Step 4: --substeps N CLI flag, parameterize SUB_STEPS.
Non-functional: benchmark must still PASS, no spring_k/damping references, cargo clippy clean, cargo test passes.
</requirements>

## Research Notes

<research_notes>
Reference implementations: Ten Minute Physics (Muller PBD tutorials), gpu-physics-engine (Morton reorder every 4s), RDPE (spatial hashing + composable rules). Morton: Z-order curve, interleave grid coords to u32 key. Codebase: main.rs uses std::env::args for CLI (manual parsing), SUB_STEPS is hardcoded const, Params struct duplicated across 3 WGSL files.
</research_notes>

---

## Definition of Done

<definition_of_done>

### Step 1: Fix documentation drift, remove dead fields, add lessons learned and backlog [x]

  - [x] Proof: `findstr /c:"PBD" C:\Users\siriu\RustroverProjects\Axiom2d\docs\plans\2026-07-04-particle-idle-poc.md` -> PoC plan Decisions section references PBD not DEM <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `findstr /c:"spring_k" crates\particle_poc\src\shaders\integrate.wgsl crates\particle_poc\src\shaders\project.wgsl crates\particle_poc\src\shaders\reaction.wgsl crates\particle_poc\src\main.rs` -> No spring_k references remain in any WGSL or Rust <!--p:{"type":"exit_code","value":1}-->
  - [x] Proof: `findstr /c:"damping" crates\particle_poc\src\shaders\integrate.wgsl crates\particle_poc\src\shaders\project.wgsl crates\particle_poc\src\shaders\reaction.wgsl crates\particle_poc\src\main.rs` -> No damping references remain in any source <!--p:{"type":"exit_code","value":1}-->
  - [x] Proof: `findstr /c:"Ten Minute Physics" C:\Users\siriu\RustroverProjects\Axiom2d\docs\ParticleIdle.md` -> ParticleIdle.md contains PBD reference link <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `findstr /c:"PL-001" C:\Users\siriu\RustroverProjects\Axiom2d\docs\BACKLOG.md` -> BACKLOG.md contains PL-001 entry <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `findstr /c:"PL-004" C:\Users\siriu\RustroverProjects\Axiom2d\docs\BACKLOG.md` -> BACKLOG.md contains PL-004 entry <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `cargo check -p particle_poc` -> particle_poc compiles after field removal <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `cargo test --workspace` -> Full workspace test suite passes <!--p:{"type":"exit_code","value":0}-->

### Step 2: Implement Morton code reordering of sorted_indices [x]

  - [x] Proof: `findstr /c:"morton" crates\particle_poc\src\shaders\morton_sort.wgsl` -> morton_sort.wgsl exists with Morton logic <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `findstr /c:"morton" crates\particle_poc\src\main.rs` -> main.rs creates Morton buffer and pipeline <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `findstr /c:"morton_sort" crates\particle_poc\src\main.rs` -> main.rs dispatches morton_sort pipeline (flag removed — Morton always active) <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `findstr /c:"morton_sort" crates\particle_poc\src\main.rs` -> Morton pipeline dispatched in sim loop <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `cargo check -p particle_poc` -> Compiles with Morton code <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `cargo build --release -p particle_poc` -> Release build succeeds <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `cargo run --release -p particle_poc -- --benchmark` -> Benchmark passes with Morton active <!--p:{"type":"output_contains","value":"Result: PASS"}-->
  - [x] Proof: `cargo test --workspace` -> Full workspace test suite passes <!--p:{"type":"exit_code","value":0}-->

### Step 3: Replace hardcoded reaction with data-driven interaction matrix [x]

  - [x] Proof: `findstr /c:"matrix.results" crates\particle_poc\src\shaders\reaction.wgsl` -> reaction.wgsl reads from matrix <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `findstr /c:"ReactionMatrix" crates\particle_poc\src\main.rs` -> ReactionMatrix buffer in main.rs <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `findstr /c:"MAX_SPECIES" crates\particle_poc\src\main.rs` -> MAX_SPECIES constant defined <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `findstr /c:"si == 0u" crates\particle_poc\src\shaders\reaction.wgsl` -> No hardcoded Red species check remains <!--p:{"type":"exit_code","value":1}-->
  - [x] Proof: `findstr /c:"reaction_matrix" crates\particle_poc\src\main.rs` -> Matrix bound to reaction bind group <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `cargo check -p particle_poc` -> Compiles with matrix <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `cargo build --release -p particle_poc` -> Release build succeeds <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `cargo run --release -p particle_poc -- --benchmark` -> Benchmark passes with matrix <!--p:{"type":"output_contains","value":"Result: PASS"}-->
  - [x] Proof: `cargo test --workspace` -> Full workspace test suite passes <!--p:{"type":"exit_code","value":0}-->

### Step 4: Add --substeps N CLI flag with runtime SUB_STEPS [x]

  - [x] Proof: `findstr /c:"substeps" crates\particle_poc\src\main.rs` -> --substeps CLI flag parsed <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `findstr /c:"sub_steps" crates\particle_poc\src\main.rs` -> sub_steps variable drives sim loop <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `cargo run --release -p particle_poc -- --benchmark --substeps 16` -> Benchmark at 16 substeps passes <!--p:{"type":"output_contains","value":"Result: PASS"}-->
  - [x] Proof: `cargo run --release -p particle_poc -- --benchmark --substeps 8` -> Benchmark at 8 substeps produces valid result <!--p:{"type":"output_contains","value":"PASS"}-->
  - [x] Proof: `cargo check -p particle_poc` -> Compiles with --substeps <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `cargo build --release -p particle_poc` -> Release build succeeds <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `cargo test --workspace` -> Full workspace test suite passes <!--p:{"type":"exit_code","value":0}-->

</definition_of_done>

## Amendment log

- **2026-07-04T23:04:15.232Z** [undefined] modified: Removed --no-morton CLI flag per project convention (no compatibility shims). Proof 2-3 (no-morton check) replaced with proof that morton_sort dispatch exists.
