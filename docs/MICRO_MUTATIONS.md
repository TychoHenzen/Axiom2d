# Micro-Mutation Tracking

Stochastic mutation testing — one random source file per daily CI run.
Over weeks, covers the codebase without combinatorial explosion.

**Cumulative (all runs)**: 118 mutants | 106 caught | 2 missed | 0 timeout | 10 unviable | **catch rate: 89.8%** | 4 runs

**How to read**: Each row = one CI run. A single random source file is selected
and all mutants generated for it are tested. Over time, this builds a picture
of mutation coverage across the workspace.

**Last run**: 2026-07-10 (`dce8fd8`)

---

## Run Log

| Date | Commit | Total | Caught | Missed | Timeout | Unviable |
|------|--------|-------|--------|--------|---------|----------|
| 2026-07-10 | `dce8fd8` | 29 | 26 | 2 | 0 | 1 |
\n<!-- detail: crates/card_game/src/card/identity/name_pools/adjectives.rs → 2/3 caught (66.7%) -->\n<!-- detail: crates/card_game/src/card/interaction/flip_animation.rs → 21/23 caught (91.3%) -->\n<!-- detail: crates/card_game/src/card/reader/pick.rs → 3/3 caught (100.0%) -->
| 2026-07-10 | `60616b7` | 86 | 78 | 0 | 0 | 8 |
\n<!-- detail: crates/card_game/src/card/identity/signature/types.rs → 39/46 caught (84.8%) -->\n<!-- detail: crates/engine_audio/src/spatial.rs → 39/40 caught (97.5%) -->\n<!-- detail: crates/engine_input/src/mouse/buffer.rs → 0/0 caught (error) -->
| 2026-07-10 | `8d4b09d` | 0 | 0 | 0 | 0 | 0 |
\n<!-- detail: crates/card_game/src/card/interaction/intent.rs → 0/0 caught (error) -->
| 2026-07-10 | `293aa4f` | 3 | 2 | 0 | 0 | 1 |
\n<!-- detail: crates/engine_audio/src/sound/library.rs → 2/3 caught (66.7%) -->
<!-- Runs appended by scripts/micro-mutations.sh -->
