# Particle Idle Phase 4 — Sandbox Game Foundation — Requirements Spec

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
4. Use `dod_refine` to turn a draft leaf into a concrete proof (mode=concretize) or subdivide into child tasks (mode=subdivide).
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

**Goal:** Evolve particle_poc from technical benchmark into a playable sandbox game: player-placed spawners with batched delivery, draggable capsule conveyor (endpoint-defined), draggable machines, drawable SDF walls, kill barrier, and mode-toggle input.

**Date:** 2026-07-13
**Target:** `C:\Users\siriu\RustroverProjects\Axiom2d`
**DoD ID:** `2bd01c25-86bf-4658-9309-d2ac0b33dcb5`
**Last check:** INCOMPLETE (2026-07-13T13:50:41.665Z)

---

## Decisions (locked with user)

<decisions>
### Technical Decisions
- **SDF approach**: Drawn strokes rendered into GPU SDF texture. Compute shader samples SDF for collision. Chosen over Rapier2D colliders (too many bodies) and raw pixel buffer (no smooth normals).
- **Conveyor model**: Full capsule between two draggable endpoints. Preserves current paddle system. Rotation + length computed from endpoints.
- **In-place evolution**: All changes land in `crates/particle_poc/`. No new crate.
- **Mode toggle**: Hotkey (Tab) switches between Drag mode and Draw mode. Distinct visual cursors.
- **Spawners replace hoppers**: Two spawners pre-placed at hopper positions. Old `spawn()` function removed.
- **Fixed viewport**: 1280×720 fixed camera. Scrollable world deferred.
</decisions>

## Current state

<current_state>
### Pre-existing State
- `crates/particle_poc/`: GPU PBD solver (100k particles @ 60fps), Rapier2D machines, recipe layer, reactions, polymer bonds
- Phases 1-3 complete per `docs/ParticleIdle.md`
- 12 tests (11 pass, 1 flake: `when_10k_particles_at_conveyor_bottom_then_no_paddle_phasing`)
- Clippy: 1 warning (clean). Format: 0 files dirty (clean).
- Quality metrics tooling setup documented in `docs/QUALITY_METRICS_TOOLING.md` (complexity/coverage/duplication not yet wired into CI)
</current_state>

## Requirements

<requirements>
## Requirements

### Spawner System
- `MachineKind::Spawner` — placeable machine with species, batch_size, interval fields
- Two spawners pre-placed at current hopper positions (Red=species 0, Blue=species 1)
- Batched delivery: each spawner accumulates dt, fires batch when timer >= interval
- Batch respects MAX_PARTICLES cap — skips if full, retries next cycle
- Old hopper code (`spawn()` function) removed entirely
- Spawner has Rapier2D body for drag interaction and distinct visual (species-color border)

### SDF Wall System
- 256×256 or 512×512 SDF texture buffer allocated on GPU as read-only storage
- Draw mode mouse drag paints signed distance values into CPU-side SDF grid
- CPU SDF grid uploaded to GPU each frame
- Erase brush (secondary action in Draw mode) clears SDF cells
- `project.wgsl` samples SDF texture for wall collision distance + normal
- SDF walls replace hardcoded wall_min/max uniform collision

### Draggable Conveyor
- Conveyor stores two endpoint positions instead of fixed pivot+angle
- Capsule geometry (center, angle, half-lengths) derived from endpoints each frame
- Paddle positions recalculated from new capsule geometry
- Hit-test against endpoint visual handles in Drag mode
- Drag updates endpoint position; Rapier2D body recreated/repositioned

### Draggable Machines
- Grinder + heater bodies repositionable via drag in Drag mode
- GPU machine params buffer updated after drag
- Hit-testing OBBs for all machine types (conveyor, grinder, heater, spawner)

### Kill Barrier
- `SimParams` gets `kill_y` field
- Particles with `pos.y < kill_y` removed from alive list (atomic decrement in compute)
- Red kill-zone line rendered at kill_y position

### Mode Toggle & Input
- `Mode { Drag, Draw }` state tracked in app
- Hotkey toggles between modes
- Draw mode shows brush cursor (circle outline at mouse world position)
- Drag mode highlights hovered draggable targets

### Scope Boundaries
- NO scrollable viewport (deferred)
- NO machine/spawner placement UI (only drag existing)
- NO recipe changes
- NO economy, logic/automation, or LLM discovery
- NO save/load
</requirements>

## Research Notes

<research_notes>
### From Phase 1 Research
- `docs/ParticleIdle.md`: Full blueprint — Phase 4 is "Economy" but user wants sandbox foundation first
- `docs/BACKLOG.md`: No particle-idle tasks currently tracked (PL-001 through PL-004 are lessons, not tasks)
- `crates/particle_poc/src/lib.rs`: `MAX_MACHINES=16`, `MAX_PARTICLES=100000`, `SPAWN_RATE=130`
- `crates/particle_poc/src/main.rs`: winit ApplicationHandler pattern, no mouse input handling yet
- `crates/particle_poc/src/state.rs`: `State` struct, `update_machines()`, `spawn()`, `simulate()`, `render()`
- Current collision: wall_min/max uniform in `SimParams`, checked in `project.wgsl`
- Current conveyor: fixed `CONVEYOR_ANGLE_DEG=45`, `CAPSULE_HALF_LEN=0.22`, pivot `(0.0, -0.22)`
- Current machines: 1 conveyor (kinematic), 1 grinder (static sensor), 1 heater (static sensor)
- Pre-existing test failure: `when_10k_particles_at_conveyor_bottom_then_no_paddle_phasing` — paddle stability flake, known and not in scope
- LINT_BASELINE: 1 clippy warning. FORMAT_BASELINE: 0 files dirty.
- Test count: 24 across entire workspace, 12 in particle_poc
</research_notes>

## Open Questions

<open_questions>
### Deferred
- What exact brush radius for SDF drawing? Default to 10× particle_radius, tunable.
- Exact spawner batch parameters? Default: 1000 particles, 60s interval, tunable via const.
- Kill barrier Y position? Default to `wall_min_y` (bottom of current world bounds).
- Should SDF walls also affect machine physics (Rapier2D colliders)? Deferred — SDF is GPU-side only for particles; machines don't collide with walls yet.
</open_questions>

---

## Definition of Done

<definition_of_done>

### Code Quality [ ]

  - [x] Proof: `cargo clippy --all-targets --all-features` → Clippy passes with zero warnings (current: 1 warning, fix it)
  - [x] Proof: `cargo fmt --all -- --check` → All files properly formatted
  - [ ] Proof: `cargo test --all` → All workspace tests pass

### Spawner System [~]

  - [ ] Proof: `grep "Spawner" crates/particle_poc/src/lib.rs` → MachineKind::Spawner variant defined in lib.rs
  - [~] **Draft**: Spawner accumulates dt, fires batch of N particles when timer >= interval. Respects MAX_PARTICLES cap — skips if full, retries next cycle.
  - [~] **Draft**: Two spawners placed at (HOPPER_LEFT_X, HOPPER_Y) and (HOPPER_RIGHT_X, HOPPER_Y). Red spawner = species 0, Blue spawner = species 1. Old hopper spawn() function removed.
  - [~] **Draft**: Spawner renders as colored rectangle with species-color border and output arrow. Uses existing GpuMachineRender pipeline.

### SDF Wall System [~]

  - [~] **Draft**: SDF texture buffer (256x256) allocated on GPU as read-only storage. Bound in particle compute bind group. New SdfParams uniform with resolution and world-bounds mapping.
  - [ ] Proof: `grep "SdfParams\|sdf_tex\|SDF" crates/particle_poc/src/lib.rs` → SDF-related types defined in public API
  - [~] **Draft**: In Draw mode, mouse drag writes signed distance values into CPU-side SDF grid. Brush radius = 10 * particle_radius. Erase brush (right-click) clears cells. CPU grid uploaded to GPU each frame on dirty.
  - [~] **Draft**: project.wgsl samples SDF texture for wall collision: if SDF value < 0 (inside wall), project particle to surface using SDF gradient as normal. Replaces wall_min/max uniform collision.
  - [~] **Draft**: Test: spawn particles above a drawn SDF wall line. After N frames, particles rest on the wall (not fall through). Verify positions are above wall surface.

### Draggable Conveyor Endpoints [~]

  - [~] **Draft**: Conveyor stores endpoint_a and endpoint_b (vec2 world positions). Capsule center = midpoint, angle = atan2(dy,dx), half_length = 0.5 * distance. Geometry recomputed each frame in update_machines().
  - [~] **Draft**: In Drag mode, mouse hit-test against circular handles at each endpoint. Drag updates endpoint position. Rapier2D body repositioned to new capsule geometry. Paddle positions recalculated.
  - [~] **Draft**: Small circular grab handles rendered at each conveyor endpoint. Highlighted on hover in Drag mode.

### Draggable Machines [~]

  - [~] **Draft**: In Drag mode, click on grinder/heater/spawner OBB and drag to reposition. Rapier2D body updated + GPU machine params buffer rewritten.
  - [~] **Draft**: Hovered machine in Drag mode shows highlight (brightness boost or outline). Cursor changes to grab on hover.

### Kill Barrier [~]

  - [~] **Draft**: SimParams gets kill_y field. After apply pass or integrated into apply, particles with pos.y < kill_y are removed via atomic decrement of alive count. Position zeroed to avoid stale rendering.
  - [~] **Draft**: Red horizontal line rendered at kill_y across full world width. Drawn as part of machine render pass or a dedicated debug-draw pass.
  - [~] **Draft**: Test: spawn particles above kill_y with downward velocity. After particles cross kill_y, alive count = 0.

### Mode Toggle & Input State [~]

  - [x] Proof: `grep "enum Mode\|Mode::" crates/particle_poc/src/main.rs crates/particle_poc/src/state.rs` → Mode enum (Drag/Draw) defined in app state
  - [~] **Draft**: Tab key toggles between Drag and Draw modes. Current mode displayed in window title or on-screen text.
  - [~] **Draft**: WindowEvent::MouseInput and CursorMoved handled. In Drag mode: hit-test + drag machines/endpoints. In Draw mode: paint/erase SDF cells. Mouse position converted to world coordinates.
  - [~] **Draft**: Draw mode renders circle outline at mouse world position showing brush radius. Circle color matches brush mode (solid = white, erase = red).

### Integration [~]

  - [ ] Proof: `grep -i "spawner\|MachineKind::Spawner" crates/particle_poc/src/main.rs crates/particle_poc/src/state.rs` → Spawner code integrated into app entry point
  - [ ] Proof: `grep -i "sdf\|SdfParams" crates/particle_poc/src/state.rs crates/particle_poc/src/lib.rs` → SDF buffer + params wired into simulate() create_buffers() pipeline
  - [~] **Draft**: cargo run -p particle_poc -- --no-benchmark starts, shows two spawners, particles emit in batches. Window title shows mode. Tab toggles. Exit code 0.
  - [~] **Draft**: cargo run -p particle_poc: switch to Draw mode, paint wall, switch to Drag mode, observe particles collide with drawn wall. Exit code 0.

### Manual Verification [x]

  - [~] Proof: `manual` → Peer review of Phase 4 sandbox implementation
  - [~] Proof: Manual — Run game, verify: spawners emit particles, conveyor moves them, machines process them, SDF walls block them, kill barrier removes them, drag + mode toggle works end-to-end _(awaiting human verification)_

</definition_of_done>

## Open risks

<open_risks>
### Risks
- **SDF texture resolution tradeoff**: 256×256 = 256KB, 512×512 = 1MB. Higher res = finer walls but more GPU memory + upload bandwidth. Start with 256×256, measure.
- **GPU→CPU SDF upload overhead**: Every frame upload of SDF texture may cost 0.1-0.5ms. Mitigation: dirty-flag, only upload when drawn.
- **Endpoint-drag UX complexity**: Capsule geometry math from two points is straightforward but getting the hit-test and drag feel right may need iteration.
- **MAX_MACHINES ceiling**: Current 16. 2 spawners + 1 conveyor + 2 machines = 5 base machines + 10 paddles = 15. Room for 1 more. Spawner count increase may need bump.
- **Test instability**: Pre-existing paddle phasing test failure — this is a known flake unrelated to Phase 4 changes.
</open_risks>

## Amendment log

- **2026-07-13T13:47:13.025Z** [__meta__] modified: Adding missing skip_reasons for tdd (GPU compute shaders are tested via behavioral integration, not unit-level red-green cycles) and brevity (existing WGSL shaders are already compact; no new Rust functions exceed 30 lines expected)
