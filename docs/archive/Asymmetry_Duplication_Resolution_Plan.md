# Asymmetry and Duplication Resolution Plan

This document catalogs the recurring anti-patterns in Axiom2d where the same concept is implemented in multiple places, or where one half of a flow is handled by normal ECS systems while the other half is handled by ad hoc hooks or direct world mutation.

The goal is not to eliminate every duplicate shape immediately. The goal is to make one path canonical for each concept, then push all other paths through that path or delete them.

This plan is the architectural companion to the open backlog items:

- `TD-033` interaction intents and authoritative appliers
- `TD-034` centralized physics ownership
- `TD-035` typed startup schedules instead of hook queues
- `TD-036` normalized raw input ingestion
- `TD-037` render extraction and cached per-frame draw lists

## 1. Startup and splash lifecycle

Current hotspots:

- `crates/axiom2d/src/splash/types.rs:43-95`
- `crates/axiom2d/src/splash/animation.rs:8-68`
- `crates/axiom2d/src/splash/render.rs:25-304`
- `crates/card_game_bin/src/main.rs:87-124`

What is asymmetric:

- `PreloadHooks` and `PostSplashSetup` now wrap typed startup schedules instead of manual closure queues, but they still run outside the normal phase graph.
- `card_game_bin` wires content through those hook queues instead of registering startup systems like the rest of the app.
- The splash code has two almost identical execution paths that differ only by timing.
- The splash subsystem also keeps a separate render module with its own geometry-building helpers, so the lifecycle split extends into rendering instead of staying in one place.

Resolution target:

- Replace hook queues with typed startup schedules or phases.
- [x] The splash startup queues now store typed ECS systems instead of manual `Vec<Box<dyn FnMut(&mut World)>>` lists, and `card_game_bin` registers preload/post-splash work through those schedules.
- [x] The `.add(FnMut)` compatibility shim was removed from both `PreloadHooks` and `PostSplashSetup`. All callers now use `add_systems()`.
- [x] `preload_system` moved to `Phase::Startup` — runs once on the first frame, gated by the App's `startup_executed` flag.
- PostSplashSetup remains in Phase::PreUpdate with its splash-done guard — it needs to run after the splash animation finishes, not on frame 1.
- Make scene setup, asset warmup, and post-splash setup ordinary systems with explicit ordering.

Migration rule:

- If a task can be expressed as a system, it should be a system.
- If a task must happen before or after splash, it should be registered in a typed phase, not pushed into a closure list.

## 2. Input ingestion and frame-state derivation

Current hotspots:

- `crates/engine_app/src/app.rs:84-144`
- `crates/engine_input/src/keyboard/state.rs:8-54`
- `crates/engine_input/src/mouse/state.rs:8-85`
- `crates/engine_input/src/keyboard/system.rs:8-16`
- `crates/engine_input/src/mouse/system.rs:8-23`

What is asymmetric:

- Keyboard and mouse have near-parallel state machines, but they are maintained as separate mini-systems.
- `App` handles raw platform events directly and mutates state resources in the same object that owns scheduling.
- Mouse has extra ad hoc state (`screen_pos`, `world_pos`, `scroll_delta`) that is updated outside the event-drain path.

Resolution target:

- Normalize all raw platform input into a single ingestion path.
- Derive `InputState` and `MouseState` from event streams in one predictable place.
- Keep movement, wheel, and button handling symmetrical unless the hardware genuinely differs.
- [x] `App` now routes cursor movement and wheel input through `EventBus<MouseInputEvent>` instead of mutating `MouseState` directly.

Migration rule:

- Raw events go in once.
- Frame-state gets derived once.
- Game systems should read the derived state, not patch it in parallel.

## 3. Card interaction pipeline

Current hotspots:

- `crates/card_game/src/plugin.rs:106-187`
- `crates/card_game/src/card/interaction/pick.rs:1-84`
- `crates/card_game/src/card/interaction/pick/apply.rs:1-139`
- `crates/card_game/src/card/interaction/release/target.rs:1-87`
- `crates/card_game/src/card/interaction/release/apply.rs:1-110`
- `crates/card_game/src/card/interaction/drag.rs:1-57`
- `crates/card_game/src/card/interaction/damping.rs:1-35`
- `crates/card_game/src/card/reader.rs:1-20`
- `crates/card_game/src/card/reader/pick.rs:1-44`
- `crates/card_game/src/card/reader/drag.rs:1-50`
- `crates/card_game/src/card/reader/insert.rs:1-68`
- `crates/card_game/src/card/reader/eject.rs:1-60`
- `crates/card_game/src/card/screen_device.rs:1-220`
- `crates/card_game/src/stash/boundary.rs:1-59`
- `crates/card_game/src/stash/store.rs:592-756`

What is asymmetric:

- Pick, drag, release, insert, and eject all encode overlapping state transitions in different modules.
- Cards, readers, screens, stash items, and store items all have slightly different versions of the same interaction machinery.
- The plugin wires a long chain of systems whose ordering is the real specification, but the ordering is spread across several modules.
- `store_buy_system`, `store_sell_system`, `stash_boundary_system`, and the reader systems all perform similar zone transition work with different local branches.

Resolution target:

- Introduce interaction intents or events that describe what the player is trying to do.
- Keep a small number of authoritative applier systems that own zone transitions.
- Move shared state transitions into one canonical helper per transition type, not one helper per caller.
- Make the plugin schedule reflect the interaction model directly so ordering becomes easy to inspect and test.

Migration rule:

- Decide the interaction outcome first.
- Apply it once.
- Never let each device or zone invent its own private transition protocol unless it truly has unique rules.

## 4. Physics ownership

Current hotspots:

- `crates/card_game/src/card/rendering/spawn_table_card.rs:34-141`
- `crates/card_game/src/card/interaction/physics_helpers.rs:7-54`
- `crates/card_game/src/card/interaction/drag.rs:11-57`
- `crates/card_game/src/card/interaction/damping.rs:17-35`
- `crates/card_game/src/card/reader/drag.rs:11-44`
- `crates/card_game/src/card/reader/insert.rs:14-68`
- `crates/card_game/src/card/reader/eject.rs:13-60`
- `crates/card_game/src/card/reader/rotation_lock.rs:1-15`
- `crates/card_game/src/stash/boundary.rs:21-59`
- `crates/card_game/src/stash/store.rs:486-499`
- `crates/card_game/src/stash/store.rs:714-749`

What is asymmetric:

- `PhysicsRes` is mutated from many places with different local rules.
- Some sites use `ResMut<PhysicsRes>`, some use `world.get_resource_mut::<PhysicsRes>()`, and some wrap the operations in helpers.
- The same entity can be re-registered, moved, damped, or removed from physics in several modules.

Resolution target:

- Centralize physics mutation behind one reconcile layer.
- Treat gameplay code as intent producers and physics code as the only mutator.
- Reduce the number of direct `PhysicsRes` write sites until each remaining one is clearly justified.

Migration rule:

- Gameplay systems should request a physics change, not perform every low-level physics step themselves.
- If a module is mutating physics and also deciding game rules, split those responsibilities.

## 5. Render pipeline duplication

Current hotspots:

- `crates/engine_render/src/sprite.rs:32-69`
- `crates/engine_render/src/shape/render.rs:51-89`
- `crates/engine_ui/src/unified_render.rs:63-208`
- `crates/card_game/src/card/rendering/baked_render.rs:15-67`
- `crates/card_game/src/card/rendering/art_shader.rs:61-129`
- `crates/card_game/src/stash/render/drag_preview.rs:14-60`
- `crates/card_game/src/stash/render/slots.rs:18-73`
- `crates/card_game/src/stash/hover.rs:65-127`
- `crates/card_game/src/stash/render/helpers.rs:3-6`

What is asymmetric:

- `sprite_render_system`, `shape_render_system`, and `unified_render_system` all do sort, cull, apply-material, draw, but each has its own local data flow.
- The card game has both the normal shape pipeline and the unified pipeline, with `ShapeRenderDisabled` acting as a switch.
- Stash rendering and drag-preview rendering duplicate shader resets and manual shader selection.
- Hover preview uses its own overlay and uniform patching path instead of sharing the same draw-list contract as the rest of render.
- Card art and hover preview both hardcode uniform byte offsets to match `ArtRegionParams`, which makes the shader/layout contract fragile if the struct changes.

Resolution target:

- Introduce a render extraction phase that builds one cached per-frame draw list.
- Make the draw list the shared source of truth for sprites, shapes, text, overlays, and preview rendering.
- Collapse helper render paths onto the same material/shader application rules.

Migration rule:

- One ordering model.
- One material application rule.
- One draw-list contract.
- Specialized renderers may still exist, but they should consume the same extracted data.

## 6. Device-specific duplication inside card game

Current hotspots:

- `crates/card_game/src/stash/store.rs:486-505`
- `crates/card_game/src/card/reader.rs:1-20`
- `crates/card_game/src/card/screen_device.rs:1-220`

What is asymmetric:

- Reader and screen devices are conceptually parallel, but their spawn, pick, drag, release, and render paths are implemented separately.
- The store has separate purchase and sale branches for reader and screen, even when the only difference is the concrete device type.
- The code is still clear today, but the shape of the code encourages drift when a new device type is added.

Resolution target:

- Introduce a small shared device spec or data table for common device behavior.
- Keep only the truly unique behavior in per-device modules.
- Route purchase/sale through the same device lifecycle interface.

Migration rule:

- Add new device kinds by extending the shared lifecycle first.
- Do not copy one device module as the starting point for another unless the duplication is immediately collapsed.

## 7. Practical resolution order

1. Collapse startup hooks into typed phases first. This removes one major asymmetric lifecycle mechanism and makes later startup work easier to reason about.
2. Normalize input ingestion next. That gives every downstream system a cleaner, single source of truth.
3. Extract interaction intents and physics reconciliation together. These two are coupled: interaction code should stop mutating low-level physics directly.
4. Add render extraction after the state-transition cleanup. Once draw data is centralized, the remaining render duplication becomes obvious and removable.
5. Only after the shared infrastructure exists, fold device-specific and stash-specific duplicates into shared lifecycle helpers.

## 8. What counts as a fix

A fix is good when:

- one concept has one canonical owner,
- the remaining call sites are thin adapters,
- the code path is easy to trace from user action to final state,
- and adding a new device, input source, or renderable does not require duplicating the same control flow again.

If a change only moves duplication around, it is not a fix.
