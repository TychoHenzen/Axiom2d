# Project Cleanup Pass — Requirements Spec

> **For Claude (/goal):** Work through each incomplete step below.
> 1. Mark a step `[>]` when you begin working on it.
> 2. Verify each proof by running the stated command/process and confirming the expected outcome.
> 3. Mark each proof `[x]` only when the claim has been tested and matches the expected value.
> 4. A step may only be marked `[x]` once ALL its proofs are `[x]` or `[~]`.
> 5. If a proof cannot be met because requirements changed or the original condition is unreasonable:
>    - Mark it `[~]` with the original condition struck through.
>    - Add a bullet underneath: `  - Met instead: [what was actually achieved]`
>    - The step can still be `[x]` once all proofs are resolved (either `[x]` or `[~]`).
> 6. Continue until every step is `[x]` — then stop and report done.
>
> **Self-contained.** No external context needed. Run the commands listed in proofs directly.

**Goal:** Remove obsolete files/folders, fix misleading config comment, consolidate docs, quick-triage doc relevance against code.

**Date:** 2026-05-30

---

## Requirements

### R1: Delete obsolete local folders
Remove these gitignored/local-only items from the filesystem:
- `.agents/` — 5 skills duplicated in `.claude/skills/` (old agent declaration format)
- `.serena/` — memories/cache/config from a no-longer-used plugin
- `.superpowers/` — brainstorm session cache (HTML artifacts, server logs)
- `mutants.out/` and `mutants.out.old/` — mutation testing output

### R2: Untrack AGENTS.md
`AGENTS.md` is in `.gitignore` but still tracked by git (committed before the gitignore entry). Remove from git tracking and delete the file.

### R3: Fix Cargo.toml incremental comment
Line 77 currently reads:
```toml
incremental = false     # keep for iteration speed, cargo clean periodically
```
Comment is misleading — reads as arguing FOR incremental, but value is `false`. Reword to clearly state incremental is disabled and why.

### R4: Verify target size stability
Manually verify that `target/` does not significantly grow between successive builds:
1. Run `cargo.exe build` (clean state assumed — currently 0.57 GB)
2. Record `target/` size
3. Make a trivial source change (e.g., add a comment to a `.rs` file)
4. Run `cargo.exe build` again
5. Record `target/` size again
6. Confirm delta is negligible (< 5 MB growth)

### R5: Consolidate docs structure
Move contents of `docs/plans/` and `docs/superpowers/` into either `docs/` root (if still active) or `docs/archive/` (if completed). Classification based on quick code triage (R6).

Also move `TECH_DEBT_LEDGER.md` from project root into `docs/`.

Delete empty `docs/plans/` and `docs/superpowers/` directories after moves.

**Exception:** This spec file (`2026-05-30-project-cleanup-interview.md`) should be moved to `docs/archive/` after all steps complete, since it will be historical at that point.

### R6: Quick triage of plan/spec docs
For each doc in `docs/plans/` and `docs/superpowers/`, check whether the main feature described exists in the codebase:
- **Feature exists in code** → move to `docs/archive/`
- **Feature not implemented** → move to `docs/` root (still relevant)
- **Partially implemented** → move to `docs/` root with a note

Files to triage:

**docs/plans/ (4 files):**
- `2026-04-05-unique-art-signatures-interview.md`
- `2026-04-07-geometric-wire-interview.md`
- `2026-04-08-screen-panel-catmull-rom-interview.md`
- `2026-04-09-noisy-art-selection-interview.md`

**docs/superpowers/plans/ (5 files):**
- `2026-04-06-hybrid-rope-physics.md`
- `2026-04-08-signal-combining.md`
- `2026-04-08-booster-pack.md`
- `2026-04-09-unified-render.md`
- `2026-04-16-terrain-viewer.md`

**docs/superpowers/specs/ (5 files):**
- `2026-04-06-hybrid-rope-physics-design.md`
- `2026-04-08-signal-combining-design.md`
- `2026-04-08-booster-pack-design.md`
- `2026-04-09-unified-render-design.md`
- `2026-04-15-terrain-viewer-design.md`

### R7: Update CLAUDE.md cross-references
If any doc paths referenced in `CLAUDE.md` changed due to moves, update the references. Known references:
- `docs/Axiom_Blueprint.md` (not moving — stays)
- `docs/Completed_Milestones.md` (not moving — stays)
- `docs/BACKLOG.md` (not moving — stays)
- `docs/architecture_bible.md` (not moving — stays)
- `docs/archive/` (stays as-is, just gets more files)

### R8: Keep untouched
These are explicitly out of scope:
- `.code-review-graph/` — active plugin, keep
- Root files: `release.ps1`, `upx.exe`, `shape list.json` — keep as-is
- `docs/archive/` existing contents — already archived
- `.cargo/`, `.github/`, `.claude/` — active config

---

## Research Notes

- `.agents/skills/` contains: livingdoc, mine-ideas, mutant-hunt, quality-check, tdd — exact duplicates exist in `.claude/skills/`
- `.serena/` has project.yml, memories/, cache/ — from Serena plugin no longer installed
- `.superpowers/` has 6 brainstorm sessions with HTML content and server state files
- `AGENTS.md` tracked by git despite `.gitignore` line 48 — needs `git rm --cached` then delete
- `TECH_DEBT_LEDGER.md` at root: 390 files tracked, last run 2026-04-02, average score 6.55
- Cargo.toml line 77: `incremental = false` — value correct, comment misleading
- Target/ currently 0.57 GB: debug/deps 320MB + debug/build 259MB. No incremental artifacts (0 MB in incremental/ dir)
- `docs/plans/` has 4 interview specs from April 2026
- `docs/superpowers/` has 5 plans + 5 design specs from April 2026
- All docs moves should use `git mv` to preserve history

## Open Questions

- Full backlog verification and new TODO creation deferred to a separate task
- Whether `TECH_DEBT_LEDGER.md` needs re-running (last run 2026-04-02, ~2 months old) — separate concern

---

## Definition of Done

### Step 1: Delete obsolete local folders

Remove `.agents/`, `.serena/`, `.superpowers/`, `mutants.out/`, `mutants.out.old/` from filesystem.

- [x] Proof: `powershell -Command "Test-Path '.agents'"` → `False`
- [x] Proof: `powershell -Command "Test-Path '.serena'"` → `False`
- [x] Proof: `powershell -Command "Test-Path '.superpowers'"` → `False`
- [x] Proof: `powershell -Command "Test-Path 'mutants.out'"` → `False`
- [x] Proof: `powershell -Command "Test-Path 'mutants.out.old'"` → `False`

### Step 2: Untrack and delete AGENTS.md

Run `git rm AGENTS.md` to remove from tracking and filesystem.

- [x] Proof: `rtk git status` → shows `deleted: AGENTS.md` in staged changes
- [x] Proof: `powershell -Command "Test-Path 'AGENTS.md'"` → `False`

### Step 3: Fix Cargo.toml incremental comment

Change line 77 comment from `# keep for iteration speed, cargo clean periodically` to something accurate like `# disabled — bloat not worth marginal rebuild speed`.

- [x] Proof: `rtk grep "incremental = false" Cargo.toml` → line contains clear comment about being disabled intentionally
- [x] Proof: `rtk grep "keep for iteration speed" Cargo.toml` → no matches (old comment gone)

### Step 4: Verify target size stability

Build twice with a trivial change between builds. Confirm target/ size delta is negligible.

- [x] Proof: Run `cargo.exe build`, record target/ size in MB — 1987.4 MB
- [x] Proof: Add a trivial comment to any `.rs` file, run `cargo.exe build` again, record target/ size — 1987.4 MB
- [x] Proof: Delta between the two measurements is < 5 MB — 0 MB delta
- [x] Proof: Remove the trivial comment after verification (leave no junk)

### Step 5: Quick triage plan/spec docs

For each of the 14 files in `docs/plans/` and `docs/superpowers/`, grep the codebase for the main feature. Record classification (implemented → archive, not implemented → docs root).

- [x] Proof: For each file, document: filename, feature keyword searched, result (found/not found), classification — all 14 files triaged as implemented → archive
- [~] Proof: ~~No files remain in `docs/plans/` (dir deleted or empty)~~
  - Met instead: Only this spec file remains; will be moved to `docs/archive/` as final step per R5 exception
- [x] Proof: No files remain in `docs/superpowers/` (dir deleted or empty)

### Step 6: Move docs per triage results

Execute `git mv` for each file based on Step 5 classification. Also `git mv TECH_DEBT_LEDGER.md docs/`.

- [x] Proof: `powershell -Command "Test-Path 'TECH_DEBT_LEDGER.md'"` → `False` (moved from root)
- [x] Proof: `powershell -Command "Test-Path 'docs\TECH_DEBT_LEDGER.md'"` → `True` (now in docs/)
- [~] Proof: ~~`powershell -Command "Test-Path 'docs\plans'"` → `False` (directory removed)~~
  - Met instead: `docs/plans/` still contains this spec file; will be removed as final step per R5
- [x] Proof: `powershell -Command "Test-Path 'docs\superpowers'"` → `False` (directory removed)
- [x] Proof: `rtk git status` → all moved files show as `renamed:` in staged changes

### Step 7: Update CLAUDE.md cross-references

Check if any paths in `CLAUDE.md` reference moved files. Update if needed.

- [x] Proof: `rtk grep "TECH_DEBT_LEDGER\|docs/plans\|docs/superpowers" CLAUDE.md` → no matches referencing old paths (or no references existed)
- [x] Proof: `cargo.exe build` → exit 0 (nothing broken by file moves)
- [x] Proof: `cargo.exe test` → exit 0 (nothing broken by file moves)

### Step 8: Final verification

Confirm clean state — no orphaned references, build works, git status shows only intended changes.

- [x] Proof: `rtk git status` → only expected staged changes (deletions, renames, Cargo.toml edit)
- [x] Proof: `rtk git diff --cached --stat` → shows only the files we intended to change (17 files, 1 insertion, 153 deletions)
- [x] Proof: `cargo.exe clippy` → no new warnings introduced
