# Phase 1 Completion: Fix Polymer Bonds + Tests - Requirements Spec

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

**Goal:** Fix solve_bonds instability (unilateral bonds → energy explosion), enable dispatch, pass stability benchmark, add bond lifecycle tests.

**Date:** 2026-07-06
**Target:** `C:\Users\siriu\RustroverProjects\Axiom2d`
**DoD ID:** `5a2e4026-a4f5-4d49-83ba-6c83bd61aa97`
**Last check:** PASS (2026-07-06T09:36:16.232Z)

---

## Decisions (locked with user)

<decisions>
Fix approach: make bonds mutual (two-pass form_bonds: proposal pass + resolve pass) so both particles in a bond pair apply equal-opposite corrections. Run solve_bonds inside substep loop after apply with re-clamp to avoid stale PBD contact corrections. Bond visualization and PBF density constraint deferred.
</decisions>

## Current state

<current_state>
form_bonds compute pipeline active (runs after substep loop). solve_bonds pipeline created but dispatch commented out (lines 1250-1251). Known failure: unilateral bonds cause asymmetric forces → energy injection → clumping and NaN explosion. Bonds form once per frame, not per substep.
</current_state>

## Requirements

<requirements>
## Requirements

1. Fix solve_bonds instability: unilateral bonds cause asymmetric correction forces → center-of-mass drift → energy ratchet → explosion. Fix via mutual bonds (two-pass form_bonds: proposal + resolve) or symmetric midpoint solver.
2. Fix substep ordering: run solve_bonds after apply with re-clamp so PBD contact corrections aren't invalidated.
3. Enable solve_bonds dispatch in substep loop.
4. Stability benchmark passes with bonds enabled (100k particles, <16.67ms, no NaN/OOB/escape).
5. Three new test modes: --test-bond-form, --test-bond-constrain, --test-bond-break.
6. Build clean — no warnings.
</requirements>

## Research Notes

<research_notes>
### Failure analysis
- solve_bonds.wgsl: tension-only distance constraints. Each thread processes own slots, applies 0.5 * error * n * BOND_COMPLIANCE toward partner. Unilateral bonds = only one side corrects = systematic center-of-mass drift in chains.
- form_bonds.wgsl: each green particle writes unilateral bond to own slot only. No reciprocal.
- main.rs: form_bonds after substep loop (once per frame). solve_bonds was between project and apply (commented out).

### Fix options
1. Two-pass form_bonds (proposal buffer + mutual resolve pass) — bonds only created if both particles agree.
2. Separate bond_corrections buffer with midpoint solver — stable even with unilateral bonds.
3. Run form_bonds inside substep loop for continuous re-bonding.
</research_notes>

## Open Questions

<open_questions>
None — confirmed with user.
</open_questions>

---

## Definition of Done

<definition_of_done>

### Fix bond solver — make bonds mutual, fix substep ordering, enable solve_bonds dispatch [x]

  - [x] Proof: `cargo build -p particle_poc` -> Build succeeds with solve_bonds enabled <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `grep "solve_bonds" crates/particle_poc/src/main.rs` -> solve_bonds dispatch uncommented and active in substep loop <!--p:{"type":"output_matches","value":"pass\\.set_pipeline\\(&self\\.pipelines\\.solve_bonds\\)"}-->
  - [x] Proof: `grep -E "reciprocal|mutual|proposal" crates/particle_poc/src/shaders/form_bonds.wgsl` -> form_bonds implements reciprocal/mutual bond mechanism <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `grep "bond_slot_a\|bond_slot_b" crates/particle_poc/src/main.rs` -> Bond buffers wired into particle bind group (bgl entry + bind_group entry) <!--p:{"type":"output_matches","value":"bond_slot_a.*as_entire_binding"}-->
  - [x] Proof: `cargo run --bin particle_poc -- --benchmark` -> Run benchmark with bonds active — the actual binary entry point, not a test harness <!--p:{"type":"output_contains","value":"Result: PASS"}-->

### Bond formation test mode: spawn green particles, verify bonds form [x]

  - [x] Proof (TDD AWAITING RED): `cargo run --bin particle_poc -- --test-bond-form` -> TDD: test must fail before bond fix, then pass after. Verifies green particles form bonds when within range. <!--p:{"type":"tdd","value":0}-->

### Bond constraint test mode: bonded pair pulled apart, verify constraint pulls together [x]

  - [x] Proof: `cargo run --bin particle_poc -- --test-bond-constrain` -> Bond constraint pulls stretched pair back toward rest length. Exit 0 confirms constraint works. <!--p:{"type":"exit_code","value":0}-->

### Bond breakage test mode: bonded pair overstretched, verify bonds clear [x]

  - [x] Proof: `cargo run --bin particle_poc -- --test-bond-break` -> Bonds clear to INVALID when stretched beyond 3x rest length. Exit 0 confirms breakage. <!--p:{"type":"exit_code","value":0}-->

### Build clean — no warnings on particle_poc [x]

  - [x] Proof: `cargo build -p particle_poc` -> No compiler warnings in particle_poc build output <!--p:{"type":"output_not_contains","value":"warning"}-->

</definition_of_done>

## Open risks

<open_risks>
Cross-thread writes in two-pass form_bonds may need atomics. WGSL atomics limited to u32/i32 scalars — may need to restructure bond data as separate partner_a/rest_a/partner_b/rest_b arrays.

Performance: solve_bonds per substep may push frame time over 16.67ms. Bonds only apply to green particles (subset). Mitigation: reduce workgroup coverage or run every Nth substep if needed.
</open_risks>

## Amendment log

- **2026-07-06T08:05:26.402Z** [undefined] modified: Workspace has multiple binaries; must specify --bin particle_poc to disambiguate
- **2026-07-06T08:05:27.989Z** [undefined] modified: Workspace has multiple binaries; must specify --bin particle_poc to disambiguate
- **2026-07-06T08:05:29.613Z** [undefined] modified: Workspace has multiple binaries; must specify --bin particle_poc to disambiguate
- **2026-07-06T08:05:31.006Z** [undefined] modified: Workspace has multiple binaries; must specify --bin particle_poc to disambiguate
