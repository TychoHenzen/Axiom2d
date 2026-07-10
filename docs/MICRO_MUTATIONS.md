# Micro-Mutation Tracking

Stochastic mutation testing — one random source file per daily CI run.
Over weeks, covers the codebase without combinatorial explosion.

**Cumulative (all runs)**: 3 mutants | 2 caught | 0 missed | 0 timeout | 1 unviable | **catch rate: 66.7%** | 1 runs

**How to read**: Each row = one CI run. A single random source file is selected
and all mutants generated for it are tested. Over time, this builds a picture
of mutation coverage across the workspace.

**Last run**: 2026-07-10 (`293aa4f`)

---

## Run Log

| Date | Commit | Total | Caught | Missed | Timeout | Unviable |
|------|--------|-------|--------|--------|---------|----------|
| 2026-07-10 | `293aa4f` | 3 | 2 | 0 | 0 | 1 |
\n<!-- detail: crates/engine_audio/src/sound/library.rs → 2/3 caught (66.7%) -->
<!-- Runs appended by scripts/micro-mutations.sh -->
