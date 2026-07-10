# Micro-Mutation Tracking

Stochastic mutation testing — one random source file per daily CI run.
Over weeks, covers the codebase without combinatorial explosion.

**Cumulative (all runs)**: 0 mutants | 0 caught | 0 missed | 0 timeout | 0 unviable | **catch rate: N/A** | 0 runs

**How to read**: Each row = one CI run. A single random source file is selected
and all mutants generated for it are tested. Over time, this builds a picture
of mutation coverage across the workspace.

**Last run**: —

---

## Run Log

| Date | Commit | Total | Caught | Missed | Timeout | Unviable |
|------|--------|-------|--------|--------|---------|----------|
<!-- Runs appended by scripts/micro-mutations.sh -->
