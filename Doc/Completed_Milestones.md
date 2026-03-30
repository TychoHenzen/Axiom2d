# Completed Milestones

Reference of completed implementation work. For active future work, see `Doc/Work_Backlog.md`.

---

## Engine (12 crates, 900+ tests)

All engine phases are complete. Implementation details formerly tracked in `Implementation_Roadmap.md`.

| Phase | Scope | Key Deliverables |
|-------|-------|-----------------|
| 1 | Time & Input | DeltaTime, FixedTimestep, FakeClock, InputState, MouseState, ActionMap |
| 2 | Scene Graph | ChildOf/Children hierarchy, Transform propagation (GlobalTransform2D), Visibility inheritance |
| 3 | Rendering | Instanced quad rendering, TextureAtlas (guillotiere), Sprite system, Camera2D + frustum culling, Shape system (lyon), Bloom post-processing, Material2d + BlendMode + ShaderRegistry |
| 4 | Audio | AudioBackend trait + CpalBackend, fundsp synthesis, SoundLibrary, MixerTrack volumes, Spatial 2D audio |
| 5 | Physics | PhysicsBackend trait + RapierBackend, RigidBody/Collider, CollisionEvents, physics-transform sync |
| 6 | Assets | Serde + RON on all data types, AssetServer\<T\> with Handle\<T\> + ref counting |
| 7 | UI | UiNode + Anchor + FlexLayout, Button/Panel/ProgressBar widgets, Interaction system, UiTheme |
| 8 | Quality | Property-based testing (proptest, 7 crates), Snapshot testing (insta, 3 crates), Visual regression (HeadlessRenderer + SSIM), DefaultPlugins + feature flags |

### Not started (deferred by design)

- Gamepad support (gilrs) — keyboard+mouse covers current needs
- Hot-reloading (assets_manager) — restart-based workflow is fine at current scale
- Examples directory — demo crate serves as reference

---

## Card Game (card_game + card_game_bin, 330+ tests)

All 8 phases complete. Implementation details formerly tracked in `Card_Game_Roadmap.md`.

| Phase | Scope | Key Deliverables |
|-------|-------|-----------------|
| A | Engine Extensions | add_force_at_point, set_damping, body_point_to_world on PhysicsBackend |
| B | Data Model | Card + CardZone components, Hand resource, StashGrid resource |
| C | Drag & Drop | DragState, card_pick_system (topmost AABB hit), card_drag_system (spring force at grab point), card_release_system |
| D | Table Physics | spawn_visual_card, card_damping_system, CameraDragState + camera_drag/zoom systems |
| E | Card Flip | CardFaceSide visibility sync, FlipAnimation (scale.x tween), right-click detection |
| F | Hand Inventory | hand_layout_system (screen-bottom row), pick/release zone transitions (Hand↔Table) |
| G | Stash Grid | Stash rendering + toggle, drag-and-drop with CardItemForm, hover preview, page navigation |
| H | Integration | CardGamePlugin (all systems wired), drag visual feedback (ScaleSpring, highlight, snap-back) |

### Post-completion additions

- **TD-032** â€” end-to-end `CardGamePlugin` schedule tests now cover multi-frame input sequences and zone transitions.
- **TD-031** â€” silent physics failures in card interaction/spawn paths now emit tracing warnings instead of being discarded.

- **Unified render system** — DFS-based hierarchy_sort_system replaced sort_propagation_system; unified_render_system in engine_ui draws shapes+text in sort order
- **Vector text rendering** — ttf-parser + lyon tessellation through shape pipeline (card labels)
- **Tracing instrumentation** — tracing crate added at hardware boundaries (engine_render, engine_physics, engine_audio)
- **Stash boundary system** — cursor-follow drag for stash-origin cards with physics transitions
- **ZoneConfig** — data-driven zone transition pattern
