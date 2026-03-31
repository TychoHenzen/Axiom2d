Using `$refactor-planning` here. This should be done as a sequence of behavior-preserving extractions, not a single “split giant files” pass.

**Why**
The main problem is not just line count. These files combine public API, internal orchestration, helpers, and very large inline test modules in one place, which raises review cost and makes safe edits harder. The good news is that the public surface is already narrow in most cases, so this is a good candidate for staged module extraction without behavior changes.

**Dependency Map**
- [crates/card_game/src/card/rendering/spawn_table_card.rs](/mnt/c/Users/siriu/RustroverProjects/Axiom2d/crates/card_game/src/card/rendering/spawn_table_card.rs)
    - Public surface is essentially `spawn_visual_card`.
    - Depends on card identity generation, baked rendering, art selection, shader resources, physics setup.
    - Called from `card_game_bin`, stash hover, debug spawn, and integration tests.
    - Internal seam already visible: `build_mesh_overlays`.
- [crates/card_game/src/card/reader.rs](/mnt/c/Users/siriu/RustroverProjects/Axiom2d/crates/card_game/src/card/reader.rs)
    - Mixed concerns: components/resources, overlap helper, 8 systems, intent types, and tests.
    - Coupled to drag state, physics helpers, collision constants from `pick`, and `CardZone`.
    - Highest orchestration density among the listed files.
- [crates/card_game/src/card/interaction/pick.rs](/mnt/c/Users/siriu/RustroverProjects/Axiom2d/crates/card_game/src/card/interaction/pick.rs)
    - Public surface is `card_pick_intent_system` and `apply_card_pick_intents_system`.
    - Internal seams: source identification, stash pick path, table/hand pick path, hit testing, hand-to-table transition.
    - Shared constants consumed by `reader` and `release`.
- [crates/card_game/src/card/interaction/release.rs](/mnt/c/Users/siriu/RustroverProjects/Axiom2d/crates/card_game/src/card/interaction/release.rs)
    - Public surface is `card_drop_intent_system` and `apply_card_drop_intents_system`.
    - Internal seams: drop target resolution, hand drop, stash drop, table drop.
    - Coupled to drag state, physics helpers, stash grid, hand, item form, flip animation.
- [crates/card_game/src/card/identity/signature.rs](/mnt/c/Users/siriu/RustroverProjects/Axiom2d/crates/card_game/src/card/identity/signature.rs)
    - Mixed pure domain types plus algorithms plus a very large test body.
    - External consumers mostly depend on `CardSignature`, `compute_seed`, and rarity/tier methods.
    - Lowest runtime risk because it is pure logic, but high semantic risk because many behaviors derive from it.
- [crates/card_game/src/stash/render.rs](/mnt/c/Users/siriu/RustroverProjects/Axiom2d/crates/card_game/src/stash/render.rs)
    - Public surface is `stash_render_system`.
    - Internal seams: camera/grid background setup, occupied slot rendering, empty slot rendering, drag preview rendering, shader reset/model helpers.
    - Mostly rendering orchestration; good candidate for extraction by draw responsibility.

**Recommended Order**
1. `signature.rs`
2. `stash/render.rs`
3. `spawn_table_card.rs`
4. `pick.rs`
5. `release.rs`
6. `reader.rs`

That order starts with pure logic and single-entry rendering modules, and leaves the most cross-coupled interaction/system orchestration for last.

**Step-by-Step Plan**
1. Refactor `signature.rs` into submodules under `card/identity/signature/`.
    - Keep `signature.rs` as the stable facade re-exporting the same public items.
    - Extract `types.rs` for `RarityTierConfig`, `Rarity`, `Element`, `Aspect`, `CardSignature`.
    - Extract `algorithms.rs` for `compute_seed`, `geometric_level`, and rarity/tier computation helpers.
    - Extract `tests.rs`.
    - Validation: run targeted tests for signature and direct dependents, then `cargo.exe test -p card_game signature`.

2. Refactor `stash/render.rs` into a directory module `stash/render/`.
    - Keep `render/mod.rs` with `stash_render_system` signature unchanged.
    - Extract `models.rs` for `miniature_card_model` and any render math.
    - Extract `slot_render.rs` for empty/occupied slot drawing.
    - Extract `drag_preview.rs` for dragged-card preview logic.
    - Extract `tests.rs`.
    - Validation: run stash render tests, then `cargo.exe test -p card_game stash::render`.

3. Refactor `spawn_table_card.rs` into `card/rendering/spawn_table_card/`.
    - Keep `mod.rs` exporting only `spawn_visual_card` and `CARD_CORNER_RADIUS`.
    - Extract `spawn.rs` for entity creation/physics insertion.
    - Extract `overlay.rs` for `build_mesh_overlays` and overlay-specific mesh/uniform assembly.
    - Extract `tests.rs`.
    - If overlay creation is still large, split further into `art_overlay.rs`, `variant_overlay.rs`, `tier_overlay.rs`.
    - Validation: run local module tests plus integration tests that call `spawn_visual_card`, then `cargo.exe test -p card_game spawn_visual_card`.

4. Refactor `pick.rs` into `card/interaction/pick/`.
    - Keep public systems and collision constants stable in `mod.rs`.
    - Extract `intent.rs` for `CardPickIntent`, `PickSource`, and intent identification.
    - Extract `hit_test.rs` for `find_card_under_cursor`.
    - Extract `apply.rs` for `pick_from_stash`, `pick_from_card`, `transition_hand_to_table`.
    - Extract `tests.rs`.
    - Validation: run pick-specific tests and interaction integration tests, then `cargo.exe test -p card_game pick`.

5. Refactor `release.rs` into `card/interaction/release/`.
    - Keep public systems stable in `mod.rs`.
    - Extract `target.rs` for `DropTarget`, `is_hand_drop_zone`, `resolve_drop_target`.
    - Extract `apply.rs` for `drop_on_hand`, `drop_on_stash`, `drop_on_table`.
    - Extract `tests.rs`.
    - Validation: run release-specific tests, then `cargo.exe test -p card_game release`.

6. Refactor `reader.rs` last, because it spans data definitions and several cooperating systems.
    - Convert to `card/reader/`.
    - Keep `mod.rs` as the facade for public components/resources/systems/constants.
    - Extract `components.rs` for `CardReader`, `OutputJack`, `ReaderDragState`, intent types.
    - Extract `pick.rs` for reader pick/release drag systems.
    - Extract `insert.rs` for reader insert intent/apply.
    - Extract `eject.rs` for eject intent/apply.
    - Extract `tests.rs`, potentially split by concern if still large.
    - Validation: run reader tests, then card interaction integration tests, then `cargo.exe test -p card_game reader`.

7. After each file-level refactor, run formatting and a narrow compile/build check.
    - `cargo.exe fmt --all`
    - `cargo.exe test -p card_game <targeted filter>`
    - At the end of each milestone group, `cargo.exe build -p card_game_bin`

**Risks and Mitigations**
- Shared private helpers becoming cyclic.
    - Mitigation: keep `mod.rs` as the only outward facade and move shared internals into one `apply.rs`/`helpers.rs` within the same directory, not cross-imported sibling modules.
- System registration accidentally changing.
    - Mitigation: do not rename exported system fns during refactor; keep signatures and import paths stable until the final cleanup pass.
- Interaction modules drifting apart on shared constants.
    - Mitigation: leave `CARD_COLLISION_GROUP` and `CARD_COLLISION_FILTER` in one canonical place until both `pick` and `release` are refactored, then consider a dedicated `collision.rs` only if duplication appears.
- Test moves causing accidental coverage loss.
    - Mitigation: move tests as-is first. Do not rewrite tests while restructuring.
- Large “mod.rs” facades regrowing.
    - Mitigation: enforce that facades only re-export and define public entrypoints, not internal implementation.

**Completion Criteria**
- Each oversized file is replaced by a directory module with a small facade file.
- Public APIs and schedule wiring remain unchanged.
- No behavioral changes are mixed into the refactor.
- Module-specific tests still pass after each extraction.
- Final verification passes:
    - `cargo.exe fmt --all`
    - `cargo.exe test -p card_game`
    - `cargo.exe build -p card_game_bin`

If you want execution, the safest first slice is `signature.rs`; it has the cleanest seams and the lowest cross-system risk.