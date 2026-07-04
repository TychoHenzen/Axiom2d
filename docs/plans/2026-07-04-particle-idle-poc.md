# Particle Idle PoC — GPU DEM Solver — Requirements Spec

<claude_instructions>
**For Claude (/goal):** Work through each incomplete step below.
1. Mark a step `[>]` when you begin working on it.
2. Call `dod_check` to verify proofs — do NOT mark proofs manually.
   While iterating on one step, pass `step: N` to verify just that step fast (other steps are carried, not re-run). A scoped run returns INCOMPLETE, never PASS.
3. A step is complete when ALL its proofs pass via `dod_check`.
3b. For `manual`/`review` proofs: `dod_check` never auto-prompts — it only reports what's already
    on record (`skipped` = not yet verified, holds overall at INCOMPLETE). Call
    `dod_verify(dod_id, proof_id)` explicitly once verification is actually relevant — typically
    right after implementing that step — then re-run `dod_check` to fold in the verdict.
4. If a proof cannot be met, use `dod_amend` to modify it with a reason.
4b. Proof commands run on the HOST OS — write OS-correct commands (no bash on Windows).
4c. After a step's proofs all pass, commit that step before starting the next — one commit per step (clean, bisectable history).
5. Continue until `dod_check` returns PASS — then stop and report done.

**Self-contained.** All commands run from `C:\Users\siriu\RustroverProjects\Axiom2d` unless noted.

**🔒 Anti-cheat:** Proofs are stored canonically in MCP storage (dod-guard).
`dod_check` executes commands from the canonical copy, not this markdown file.
Editing proof text here has no effect on verification.
Store tampering is **logged and detectable** — each check prints a proof-set fingerprint.
Manual/review proofs are confirmed by the human directly (popup / elicitation) via `dod_verify` —
Claude cannot self-confirm them, and an unrequested one holds the DoD at INCOMPLETE, never PASS.
A confirmed verdict is recorded until the proof changes.
</claude_instructions>

**Goal:** Prove a GPU compute DEM particle solver sustains 100k particles at 60 FPS with gravity, containment, reactions, and a Rapier2D kinematic conveyor on the Axiom2d wgpu stack.

**Date:** 2026-07-04
**Target:** `C:\Users\siriu\RustroverProjects\Axiom2d`
**DoD ID:** `ed6f2e7a-fe08-4aa7-8b91-ec62c4121ada`

---

## Decisions (locked with user)

<decisions>
- **Algorithm**: GPU DEM (spring-dashpot normal + Coulomb friction), not PBD/XPBD
- **Spatial search**: Count-sort + prefix-scan spatial hash on GPU
- **Rendering**: Instanced colored circles, species→color
- **Scene**: Hopper spawns Red/Blue particles into a box; 1 reaction (Red+Blue→Green)
- **Interaction**: Fully passive (no user input)
- **Conveyor (step 2)**: Rapier2D kinematic body, collider transform fed to compute shader
- **Crate location**: `crates/particle_poc/` — standalone binary, own wgpu instance
- **Performance overlay**: FPS + particle count + sim ms displayed as text
</decisions>

## Requirements

<requirements>
## Functional
- Particles spawn from a hopper region at screen top, fall under gravity into a box container
- Two species: Red (0) and Blue (1). Reaction: Red + Blue within contact radius → Green (2)
- DEM contact forces: Hertzian spring-dashpot normal force + Coulomb tangential friction
- Spatial hash grid (cell size ≥ interaction radius) for O(n) neighbor search
- Box boundaries (4 walls) as collision constraints
- Particle cap: 100k
- Step 2: Rapier2D kinematic conveyor oscillating horizontally at box bottom, agitating particles

## Non-Functional
- 100k particles at 60 FPS sustained (avg frame time < 16.67ms) on RTX 3060+
- SoA GPU buffer layout (position, velocity, species — no CPU readback in hot path)
- Fixed timestep sub-stepping for DEM stability
- Performance overlay: FPS, particle count, sim time per frame
- Windows native (x86_64-pc-windows-msvc, DX12 via wgpu)

## Out of Scope
- Economy, recipes, game logic
- User interaction (passive demo)
- Audio, save/load
- Integration with existing engine_render or card_game
</requirements>

## Research Notes

<research_notes>
- **DEM on GPU**: Spring-dashpot (k_n, damping_n) normal + (k_t, mu) friction. Smaller timesteps than PBD needed for stiff contacts — use sub-stepping (e.g. 4 substeps per frame at dt=4.17ms each).
- **Spatial hash perf**: MDPI 2025 study shows count-sort + prefix-scan spatial hash achieves O(n), 168k particles/ms throughput, 5.7-6.0ms frame times from 10k-1M particles.
- **wgpu compute**: Use `ComputePipeline`, `BindGroup` with `StorageBuffer`. Dispatch with `dispatch_workgroups(ceil(N/256), 1, 1)`. Indirect dispatch via `DispatchIndirect` buffer to avoid CPU readback of particle count.
- **Rendering**: Instanced quads expanded in vertex shader. 6 vertices per particle (2 triangles), position + species from storage buffer, circle SDF in fragment shader.
- **Rapier2D kinematic**: `RigidBodyType::KinematicPositionBased`, set position each frame with `set_next_kinematic_position`. Extract collider shape + transform, upload as uniform buffer to compute shader.
- **Existing deps available**: wgpu (24), winit, pollster, bytemuck, rapier2d — all in workspace already.
</research_notes>

---

## Definition of Done

<definition_of_done>

### Step 1: Scaffold crate with winit + wgpu window [x]

- [x] Proof: `cargo check -p particle_poc` → Crate compiles successfully with all dependencies
- [x] Proof: `findstr /c:"wgpu" crates\particle_poc\Cargo.toml` → wgpu dependency declared in Cargo.toml
- [x] Proof: `findstr /c:"winit" crates\particle_poc\Cargo.toml` → winit dependency declared in Cargo.toml
- [x] Proof: `cargo pkgid particle_poc` → particle_poc is a recognized workspace member

### Step 2: GPU storage buffers (SoA layout, 100k capacity) + compute pipeline skeleton [x]

- [x] Proof: `cargo check -p particle_poc` → Compiles with buffer and pipeline code
- [x] Proof: `findstr /s /c:"STORAGE" crates\particle_poc\src\*.rs` → Storage buffer usage flags present in Rust source
- [x] Proof: `findstr /s /c:"ComputePipeline" crates\particle_poc\src\*.rs` → Compute pipeline creation exists

### Step 3: Spatial hash compute shader (cell assignment + count sort + prefix scan) [x]

- [x] Proof: `cargo check -p particle_poc` → Compiles with spatial hash shader
- [x] Proof: `findstr /s /c:"cell" crates\particle_poc\src\shaders\*.wgsl` → Spatial hash shader references grid cells
- [x] Proof: `findstr /s /c:"prefix" crates\particle_poc\src\shaders\*.wgsl` → Prefix scan logic exists in shader

### Step 4: DEM contact solver compute shader (forces + gravity + wall collision) [x]

- [x] Proof: `cargo check -p particle_poc` → Compiles with DEM solver
- [x] Proof: `findstr /s /c:"spring" crates\particle_poc\src\shaders\*.wgsl` → Spring-dashpot force model in shader
- [x] Proof: `findstr /s /c:"gravity" crates\particle_poc\src\shaders\*.wgsl` → Gravity applied in solver shader
- [x] Proof: `findstr /s /c:"friction" crates\particle_poc\src\shaders\*.wgsl` → Friction force computation in shader

### Step 5: Hopper spawner (Red/Blue particles from top region) [x]

- [x] Proof: `cargo check -p particle_poc` → Compiles with spawner logic
- [x] Proof: `findstr /s /c:"spawn" crates\particle_poc\src\*.rs` → Spawn logic exists in Rust source
- [x] Proof: `findstr /s /c:"100000" crates\particle_poc\src\*.rs` → 100k particle cap defined

### Step 6: Instanced particle renderer (colored circles, species to color) [x]

- [x] Proof: `cargo check -p particle_poc` → Compiles with render pipeline
- [x] Proof: `findstr /s /c:"RenderPipeline" crates\particle_poc\src\*.rs` → Render pipeline created for particle drawing
- [x] Proof: `findstr /s /c:"species" crates\particle_poc\src\shaders\*.wgsl` → Vertex/fragment shader reads species for color mapping

### Step 7: Inter-particle reaction (Red + Blue within radius transmutes to Green) [x]

- [x] Proof: `cargo check -p particle_poc` → Compiles with reaction logic
- [x] Proof: `findstr /s /c:"transmute" crates\particle_poc\src\shaders\*.wgsl` → Transmutation logic in compute shader

### Step 8: Performance overlay (FPS, particle count, sim time) [x]

- [x] Proof: `cargo check -p particle_poc` → Compiles with overlay code
- [x] Proof: `findstr /s /i /c:"fps" crates\particle_poc\src\*.rs` → FPS tracking exists in source

### Step 9: 100k particle benchmark passes at 60 FPS [x]

- [x] Proof: `cargo build --release -p particle_poc` → Release build succeeds
- [x] Proof: `cargo run --release -p particle_poc -- --benchmark` → Benchmark mode runs 300 frames at 100k particles, reports PASS when avg frame time < 16.67ms
- [x] Proof: `cargo run --release -p particle_poc -- --benchmark` → Benchmark confirms full 100k particle count was reached

### Step 10: Rapier2D kinematic conveyor (oscillating, feeds transform to shader) [ ]

- [ ] Proof: `cargo check -p particle_poc` → Compiles with Rapier2D conveyor
- [ ] Proof: `findstr /c:"rapier" crates\particle_poc\Cargo.toml` → rapier2d dependency in Cargo.toml
- [ ] Proof: `findstr /s /c:"Kinematic" crates\particle_poc\src\*.rs` → Kinematic rigid body type used for conveyor
- [ ] Proof: `findstr /s /c:"conveyor" crates\particle_poc\src\shaders\*.wgsl` → Conveyor collision geometry referenced in compute shader
- [ ] Proof: `cargo run --release -p particle_poc -- --benchmark` → Benchmark still passes with conveyor active
- [ ] Proof: `cargo test --workspace` → Full workspace test suite passes — no regressions from new crate

</definition_of_done>

## Open risks

<open_risks>
- DEM with stiff contacts may need very small sub-steps, eating into frame budget. Fallback: softer springs or switch to position-based collision resolve.
- Spatial hash prefix scan on GPU requires careful synchronization (workgroup barriers). May need multiple dispatch passes.
- Text overlay rendering without engine_render/engine_ui — may need a simple CPU-side bitmap font or wgpu_text crate.
- wgpu DX12 backend compute shader debugging is limited. If shaders misbehave, diagnosis is hard.
</open_risks>

## Lessons (2026-07-04 physics rework)

- **DEM spring-dashpot is not viable at this scale.** Supporting a ~225-layer pile requires k ≈ 5e6, which needs dt ≈ 1e-4 (160+ substeps/frame). Soft springs explode instead (ω·dt > 2 once contacts stack). Replaced with PBD (position-based dynamics) per this plan's documented fallback: predict → hash → project (ω/n-averaged Jacobi, positional Coulomb friction) → apply with v = (x−x_prev)/dt.
- **Scale rule that ends tunneling:** max speed must stay ≤ 1 particle radius per substep. r=0.002, dt=1/960 → v_max=1.9. Gravity chosen as −1.2 so full-height free-fall peaks exactly there. Raising g back toward 9.81 without shrinking dt re-introduces tunneling and bottom-layer crush.
- **Jacobi corrections must be averaged (ω/n), never summed.** Summed corrections overshoot ~3× at 6 contacts → oscillation → KE grows 10× ("fountain"). ω=1.2 stable; ω=1.5 ejects sparse-contact surface particles (measured vmax spikes 9–14 at rest).
- **Never write velocities in a pass that also reads neighbor velocities** — race breaks Newton's 3rd law pairwise (measured as momentum drift to ±8700 with zero horizontal forcing).
- **GPU prefix scan must not be single-thread.** The workgroup_size(1) serial scan over 25 600 cells × 16 substeps was the 2 FPS cause; replaced with 256-thread chunked Hillis-Steele scan.
- **Diagnostics before theory.** `--diagnose` mode (GPU readback → NaN/vmax/KE/momentum/overlap stats per 10 frames) found every one of the above; guessing found none. Keep it.

## Amendment log

- **2026-07-04T09:13:20.594Z** [step-1/proof-1-4] modified: Root Cargo.toml uses glob pattern `members = ["crates/*"]` — the string "particle_poc" never appears literally. Amended to verify cargo can locate the package, which proves workspace membership.
- **2026-07-04T09:13:39.609Z** [step-1/proof-1-4] modified: cargo locate-project doesn't support -p flag. Using cargo pkgid which exits 0 only if the package is a workspace member.
- **2026-07-04T18:55:44.925Z** [step-9/proof-9-2] modified: Proof predicate doesn't match actual benchmark output format. Output clearly contains 'Result: PASS' but proof fails. Amending to output_contains with 'Result: PASS' to match the actual println format.
- **2026-07-04T18:55:46.221Z** [step-9/proof-9-3] modified: Proof predicate doesn't match actual benchmark output format. Output clearly contains '100000 particles' but proof fails. Amending to output_contains with '100000 particles' to match the actual println format.
