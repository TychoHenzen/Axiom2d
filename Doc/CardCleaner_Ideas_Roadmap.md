# CardCleaner Ideas Roadmap

Ideas mined from [CardCleaner](https://github.com/TychoHenzen/CardCleaner) for Axiom2d's card game.
Not a porting guide — these are conceptual adaptations for our 2D physics-based card engine.

Tech debt items from `Doc/Technical_Debt_Audit.md` are interleaved where they matter — either as prerequisites for upcoming features or as standalone hardening steps.

---

## Engine Hardening (Tech Debt)

Items from the tech debt audit that should be addressed before or alongside new feature work. Ordered by when they become important.

### TD-032 — End-to-End Schedule Tests `[NOT STARTED]`
**Ref:** Technical_Debt_Audit.md
**Why now:** Before building more game systems (I10–I13), establish confidence that the existing 15+ system chain actually works as wired. Currently 330+ tests all run single systems in isolation — no test exercises a pick-drag-release cycle through the real schedule.

- [ ] Test helper: `fn run_frames(app: &mut App, n: usize)` — runs n full schedule ticks
- [ ] Test helper: `fn simulate_click(world, pos: Vec2)` / `simulate_drag(world, from, to)` — programmatic input injection
- [ ] E2E test: pick card from table → drag to hand zone → release → card is in Hand resource
- [ ] E2E test: pick from stash → drag to table → release → card has physics body + CardZone::Table
- [ ] E2E test: flip animation runs to completion over multiple frames → Card.face_up toggled
- [ ] Tests run through real `CardGamePlugin` schedule, not manually constructed World+Schedule

### TD-004 — Shape Tessellation Caching `[NOT STARTED]`
**Ref:** Technical_Debt_Audit.md
**Why now:** Each card has 4–6 child shapes. At 30 cards that's ~180 tessellations per frame. Adding gem sockets (I6) would add 8 more per card. Caching eliminates redundant lyon tessellation.

- [ ] `CachedMesh` component: stores `TessellatedMesh` (vertices + indices)
- [ ] `mesh_cache_system` (Phase::PreUpdate): tessellate on `Added<Shape>` or `Changed<Shape>`, insert/update `CachedMesh`
- [ ] `unified_render_system` and `shape_render_system`: read `CachedMesh` instead of calling `tessellate()` per frame
- [ ] Fallback: if `CachedMesh` absent, tessellate inline (backwards-compatible)
- [ ] Tests: cached mesh matches direct tessellation output, Changed<Shape> triggers re-tessellation

### TD-031 — Silent Failure Observability `[NOT STARTED]`
**Ref:** Technical_Debt_Audit.md
**Why now:** Before building more systems on top of the hardware traits (deck slots with physics, combat, world gen rendering), make failures visible. Currently `Renderer`, `PhysicsBackend`, and `AudioBackend` methods return `()` — failures are invisible.

- [ ] Add `tracing::warn!` calls in `CpalBackend` when audio device unavailable (currently returns `None` silently)
- [ ] Add `tracing::warn!` in `RapierBackend` when operating on unknown entity handles
- [ ] Add `tracing::debug!` in `NullRenderer`/`NullPhysicsBackend`/`NullAudioBackend` for key operations (helps test debugging)
- [ ] Consider `Result` return types for fallible operations: `compile_shader`, `upload_atlas` (API change — evaluate scope)
- [ ] Tests: verify warn-level tracing emitted for degraded operations (use `tracing-test` or capture subscriber)

### TD-001/002/003 — Change Detection Trio `[NOT STARTED]`
**Ref:** Technical_Debt_Audit.md
**Why now:** Not a bottleneck at ~50 card entities, but becomes important before tilemap work (I25) which adds hundreds of static entities. Do this before the tilemap pipeline.

- [ ] `transform_propagation_system`: add `Changed<Transform2D>` filter on roots, propagate only dirty subtrees
- [ ] `hierarchy_maintenance_system`: use `Changed<ChildOf>` / `Added<ChildOf>` to detect actual hierarchy changes, skip rebuild when nothing changed
- [ ] `visibility_system`: add `Changed<Visible>` filter, propagate only when visibility actually changes
- [ ] Edge cases: newly added children, removed parents, reparented entities
- [ ] Tests: unchanged hierarchy produces no GlobalTransform2D writes, changed child triggers parent-chain update

### TD-005 — WgpuRenderer Material GPU Implementation `[NOT STARTED]`
**Ref:** Technical_Debt_Audit.md
**Why later:** ECS-side Material2d wiring is complete. GPU-side is stubs. Not blocking current card game work, but needed before custom shaders for card art (I5 deterministic gen) or post-process effects (TD-015 color grading) can have visual impact.

- [ ] GPU pipeline cache: `HashMap<(ShaderHandle, BlendMode), RenderPipeline>`
- [ ] `set_shader()`: look up registered WGSL source via ShaderRegistry, compile if not cached
- [ ] `set_material_uniforms()`: upload bytes to dynamic GPU buffer
- [ ] `bind_material_texture()`: bind texture to appropriate bind group slot
- [ ] Tests: via HeadlessRenderer visual regression — custom shader produces different output than default

### TD-018 — Physics Interpolation `[NOT STARTED]`
**Ref:** Technical_Debt_Audit.md
**Why later:** Visual stutter when frame rate and physics step rate diverge. Matters for smooth card motion at varying frame rates. Not critical until the game targets release quality.

- [ ] Store previous `Transform2D` position/rotation per physics entity
- [ ] Interpolation system: lerp between previous and current using `accumulator / step_size` alpha
- [ ] Only interpolate entities with `RigidBody` component
- [ ] Tests: interpolated position is between previous and current, alpha=0 gives previous, alpha=1 gives current

### TD-015 — Color Grading Post-Process `[NOT STARTED]`
**Ref:** Technical_Debt_Audit.md
**Why later:** Visual polish for the game. Exposure, contrast, saturation, color temperature via fullscreen quad pass after bloom. Nice to have for mood-setting during combat or exploration phases.

- [ ] `ColorGradeSettings` resource: `exposure: f32`, `contrast: f32`, `saturation: f32`, `temperature: f32`
- [ ] WGSL fullscreen quad shader applying color adjustments
- [ ] `color_grade_system` (Phase::Render, after post_process_system): no-op when resource absent
- [ ] Tests: default settings produce identity transform, zero saturation produces greyscale

---

## Card Identity System

### I1 — Card Signature (Multi-Dimensional Properties) `[NOT STARTED]`
**Inspired by:** `CardSignature.cs` — 8D elemental vector [-1,1] per axis with distance/subtract/intensity operations
**Engine gaps:** None — pure data model in `card_game` crate
**Why:** Gives every card a rich, continuous identity instead of discrete types. Enables emergent rarity, deterministic generation, and cards-as-seeds for world generation. The 8 elemental axes (Solidum, Febris, Ordinem, Lumines, Varias, Inertiae, Subsidium, Spatium) create a space where proximity = similarity, letting us compute card relationships mathematically.

- [ ] `CardSignature` struct: `[f32; 8]` with `Element` enum indexing, clamped to [-1, 1]
- [ ] Operations: `distance_to(&self, other) -> f32` (Euclidean in 8D), `subtract(&self, other) -> CardSignature` (residual energy), `dominant_aspect(element) -> Aspect`, `intensity(element) -> f32`
- [ ] `Element` enum (8 variants) + `Aspect` enum (16 variants, positive/negative per element)
- [ ] `CardSignature::random(rng: &mut ChaCha8Rng) -> Self` for seeded generation
- [ ] Integrate into existing `Card` component as an optional field (backwards-compatible with current texture-only cards)
- [ ] Tests: distance symmetry, subtract produces residual, random stays in bounds, dominant aspect sign-correctness

### I2 — Base Card Types (Template Matching) `[NOT STARTED]`
**Inspired by:** `BaseCardType.cs` — templates with BaseSignature + MatchRadius, inverse-distance weighted matching
**Engine gaps:** None — pure data model
**Why:** Cards aren't randomly typed — they match against defined archetypes by proximity in signature space. A sword card exists because its signature is close to the "Weapon" base type. Edge-of-radius cards are unusual and interesting. Categories (Equipment, Skill, Playstyle) give structure without rigidity.

- [ ] `BaseCardType` struct: `name: String`, `base_signature: CardSignature`, `match_radius: f32`, `category: CardCategory`
- [ ] `CardCategory` enum: `Equipment`, `Skill`, `Playstyle` (extensible)
- [ ] `can_match(signature, max_distance) -> bool` and `match_weight(signature) -> f32` (inverse distance weighting)
- [ ] `BaseCardTypeRegistry` resource: `Vec<BaseCardType>` with lookup by best match
- [ ] Tests: exact match scores highest, outside radius scores zero, multiple candidates resolve by weight

### I3 — Residual Energy (Signature-Derived Stats) `[NOT STARTED]`
**Inspired by:** `ResidualEnergyModifier.cs` — signature minus base = residual, each element axis maps to gameplay stats
**Engine gaps:** None — pure data model
**Why:** A card's stats aren't stored — they're computed from how its signature differs from its base type. The Febris axis difference becomes attack power, Subsidium becomes healing, etc. Cards at the edge of a base type's radius have extreme residuals, making them unusual and powerful. Stats are emergent, not assigned.

- [ ] `ResidualModifier` struct: `source_element: Element`, `modifier_type: ModifierType`, `intensity: f32`, `use_positive: bool`
- [ ] `ModifierType` enum: `Power`, `Cost`, `Duration`, `Range`, `Healing`, `Speed`, `Defense`, `Special`
- [ ] `calculate_effect(residual: &CardSignature) -> f32` — extract element value, apply intensity scaling
- [ ] Attach `Vec<ResidualModifier>` to `BaseCardType`
- [ ] `CardStats` computed struct: derived on-the-fly from signature + base + modifiers (never stored)
- [ ] Tests: zero residual gives base stats, aligned residual boosts stat, misaligned residual returns base only

### I4 — Emergent Rarity `[NOT STARTED]`
**Inspired by:** `SignatureCardHelper.DetermineRarity()` — computed from signature extremity, not random
**Engine gaps:** None
**Why:** Rarity isn't an attribute — it's a consequence of how extreme a card's signature is. Each element's distance from the midpoint (0.5 threshold scale) contributes points, summed and log-scaled into 5 tiers. A card becomes Legendary because its properties are extreme, not because a loot table said so. Players discover rarity by examining the card.

- [ ] `CardRarity` enum: `Common`, `Uncommon`, `Rare`, `Epic`, `Legendary`
- [ ] `determine_rarity(signature: &CardSignature) -> CardRarity` pure function (threshold scoring + log scale + bucketing)
- [ ] Rarity affects card visual rendering (border color, gem intensity — see I6)
- [ ] Tests: all-zero signature is Common, all-extreme signature is Legendary, scoring is deterministic

---

## Card Visuals

### I5 — Deterministic Card Generation `[NOT STARTED]`
**Inspired by:** `SignatureCardGenerator.cs` — seed from signature values, seeded RNG selects all visuals
**Engine gaps:** None — extends existing `spawn_visual_card` shape hierarchy
**Why:** Same signature always produces the same card appearance. Serialize just the signature; reconstruct visuals on load. Tiny save files. No art asset database needed — visual parameters are computed from signature via seeded RNG (texture selection, color mixing, pattern choice).

- [ ] `compute_seed(signature: &CardSignature) -> u64` — deterministic hash from element values
- [ ] `generate_card_visuals(signature: &CardSignature, base_type: &BaseCardType) -> CardVisualParams`
- [ ] `CardVisualParams`: border color, art area fill, pattern index, gem colors/intensities — all derived from seeded ChaCha8Rng
- [ ] Wire into `spawn_visual_card` so shape colors/sizes reflect generated params
- [ ] Tests: same signature produces identical visuals, different signatures produce different visuals

### I6 — Gem Sockets (Signature Visualization) `[NOT STARTED]`
**Inspired by:** `SignatureCardGenerator.SetGemVisuals()` — 8 gem sockets with emission color/strength from signature elements
**Engine gaps:** Possible Shape glow/emission effect (could fake with overlaid translucent shapes)
**Why:** 8 small gem shapes along the card border, one per element. Color encodes positive/negative aspect, brightness encodes intensity. Players learn to read card properties at a glance without UI text. Visually rich without text rendering.

- [ ] 8 gem child entities per card (Circle shapes at fixed positions along border)
- [ ] Gem color derived from Element's positive/negative Aspect color palette
- [ ] Gem size or opacity scaled by `signature.intensity(element)`
- [ ] Dark/dim gem for near-zero intensity, bright/large for extreme values
- [ ] Tests: gem count matches element count, zero-intensity gem is minimal, high-intensity gem is prominent

### I7 — Rarity Visual Indicators `[NOT STARTED]`
**Inspired by:** `RarityVisual` system — per-rarity texture sets for base, border, corners, banner
**Engine gaps:** None — shape color/size variation per rarity tier
**Why:** Rarity should be immediately visible. Border thickness, color saturation, and corner accent shapes scale with rarity tier. Common cards look plain; Legendary cards look ornate. All achievable with the existing shape hierarchy system.

- [ ] `RARITY_COLORS: [Color; 5]` constant array (grey → white → blue → purple → gold)
- [ ] Border shape color keyed by rarity
- [ ] Additional corner accent shapes for Rare+ (small triangles or circles at card corners)
- [ ] Optional subtle scale increase for higher rarity (1.0 → 1.02 → 1.05)
- [ ] Tests: each rarity produces distinct border color, accent shapes present only for Rare+

---

## Inspection & Discovery

### I9 — Card Inspection Mode `[NOT STARTED]`
**Inspired by:** `BlacklightController.cs` — proximity-based reveal of hidden card properties
**Engine gaps:** None — can use existing shape rendering + UI overlay
**Why:** Cards have hidden depth (signature values, residual stats, rarity reasoning). An inspection mode (hold a key while hovering) reveals this information as an overlay or expanded view. Instead of a UV blacklight (3D concept), we use a magnifying-glass metaphor: hover + hold Shift to expand the card and show gem details, stat bars, and type classification.

- [ ] `CardInspectState` resource: `Option<Entity>` tracking inspected card
- [ ] `card_inspect_system` (Phase::Update): Shift+hover over table card → set inspected entity
- [ ] `card_inspect_render_system` (Phase::Render): draw expanded card view at cursor position showing:
  - Larger card face (1.5× scale)
  - 8 stat bars (one per element, colored by aspect)
  - Type name text (from matched BaseCardType)
  - Rarity indicator
- [ ] Guards: only on table cards, not during drag, clear on Shift release
- [ ] Tests: inspect activates on shift+hover, clears on release, no inspect during drag

---

## Zone Mechanics

### I10 — Deck Slots (Physical Consumption Zones) `[NOT STARTED]`
**Inspired by:** `DeckSlot.cs` — physical Area3D zones where cards enter, freeze, stack, and can be consumed to extract signatures
**Engine gaps:** Collision event filtering (engine_physics `CollisionEvent` exists but may need zone-entity association)
**Why:** Dedicated table regions where cards can be placed to trigger actions. Drop a card onto a "Map Seed" slot — it freezes in place, stacks visually. When all slots are filled, consume the cards (extract signatures, destroy entities) to trigger world generation. Physical zones with capacity limits, ejection for overflow, and visual stacking.

- [ ] `DeckSlot` component: `capacity: usize`, `position: Vec2`, `cards: Vec<Entity>`
- [ ] `deck_slot_system` (Phase::Update): detect card release over slot area (AABB), lock card (remove physics, set position, add to slot)
- [ ] Visual stacking: offset each card slightly (SortOrder increment + small position offset)
- [ ] Overflow: eject card with impulse if slot full
- [ ] `consume_all(slot) -> Vec<CardSignature>` — extract signatures, despawn card entities, clear slot
- [ ] Pickup from slot: drag a slotted card out to unlock it (re-add physics body)
- [ ] Tests: card locks into slot, capacity enforced, consume extracts signatures, pickup unlocks card

---

## Game Loop

### I11 — Game Session State Machine `[NOT STARTED]`
**Inspired by:** `GameSessionService.cs` — WaitingForCards → GeneratingMap → Exploring → InCombat → GeneratingLoot → SessionComplete
**Engine gaps:** None — pure ECS state machine
**Why:** Structures the gameplay into distinct phases. Players collect and place cards into deck slots (WaitingForCards). Consuming the cards triggers map generation. Exploration and combat follow. Loot generates new cards that go back to the table/hand/stash. The loop creates purpose for card collection and manipulation.

- [ ] `SessionState` enum: `Collecting`, `Generating`, `Exploring`, `Combat`, `Loot`, `Complete`
- [ ] `GameSession` resource: `state: SessionState`, `map_seeds: Vec<CardSignature>`, `ability_cards: Vec<CardSignature>`
- [ ] `session_system` (Phase::Update): state machine transitions triggered by game events
- [ ] `start_session(seeds, abilities)` — validates inputs, transitions to Generating
- [ ] `reset_session()` — clears all state, returns to Collecting
- [ ] Tests: valid state transitions, invalid transitions rejected, reset clears state

### I12 — Cards-as-Seeds World Generation `[NOT STARTED]`
**Inspired by:** `CardBasedGradient.cs` — 1 card: sphere, 2 cards: capsule interpolation, 3+: closed Bezier loop via De Casteljau
**Engine gaps:** `engine_tilemap` crate (new) for 2D grid maps — or simpler: generate a grid of entities with tile components
**Why:** The cards you sacrifice determine the world you explore. One card creates a uniform environment. Two cards create a gradient between their signatures. Three or more create a complex landscape via Bezier interpolation in 8D space. Multi-card bonuses increase intensity and variation. The signature at each map position determines the biome/terrain via nearest-biome matching.

- [ ] `CardGradient` struct: takes `&[CardSignature]`, implements `signature_at(x, y, map_w, map_h) -> CardSignature`
- [ ] 1-card: sphere sampling (uniform with radius variation)
- [ ] 2-card: capsule (linear interpolation with perpendicular spread)
- [ ] 3+-card: closed Bezier via De Casteljau algorithm
- [ ] Multi-card bonuses: intensity multiplier `1 + (n-1) * 0.3`, variation factor `1 + (n-1) * 0.2`
- [ ] Bilinear interpolation on sample grid (16×16 resolution) for smooth spatial transitions
- [ ] Tests: single card produces uniform field, two cards interpolate, more cards increase intensity

### I13 — Turn-Based Combat (Command Pattern) `[NOT STARTED]`
**Inspired by:** `SimpleCombatSystem.cs` — undoable turn-based combat, player abilities from card signatures, enemy power from signature intensity
**Engine gaps:** None — pure game logic
**Why:** Combat uses the Command pattern for full undo support. Player abilities are derived from their ability cards' signatures (the cards placed in the "ability" deck slot). Enemy stats scale with the map seed's signature intensity. Simple but extensible: attack, heal, and special effects all derived from signature elements.

- [ ] `CombatCommand` struct: `name`, `damage`, `healing`, `description` + `execute(context)` / `undo(context)`
- [ ] `CombatCommand::from_signature(sig: &CardSignature) -> Self` — derives ability from dominant elements
- [ ] `Combatant` struct: `name`, `health`, `max_health`, `attack_power`
- [ ] `CombatSystem` resource: player + enemy combatants, command history for undo, turn alternation
- [ ] `combat_system` (Phase::Update): processes one turn per frame when in Combat state, alternates player/enemy
- [ ] Enemy power: `total_intensity / 8` clamped to [0.1, 1.0], scales HP (30 + power×5) and ATK (8 + power×2)
- [ ] Tests: combat command executes damage, undo reverses damage, enemy power scales with signature, combat ends on death

---

## Persistence

### I14 — Signature-Only Serialization `[NOT STARTED]`
**Inspired by:** CardCleaner's save system — cards serialize as signature + transform, visuals reconstructed deterministically
**Engine gaps:** None — serde already available on Card, CardSignature would need Serialize/Deserialize
**Why:** Since card visuals are deterministically generated from signatures (I5), save files only need to store signatures and positions. Tiny save files, no asset references in saves, reconstructed visuals are identical. Save the hand, stash, and table state as `Vec<(CardSignature, CardZone)>`.

- [ ] Derive `Serialize, Deserialize` on `CardSignature`
- [ ] `SaveState` struct: `cards: Vec<(CardSignature, CardZone, Option<Vec2>)>` (signature + zone + table position if on table)
- [ ] `save_game(world) -> SaveState` — iterate all Card entities, extract signature + zone + position
- [ ] `load_game(world, save: &SaveState)` — despawn existing cards, spawn from signatures using deterministic generation
- [ ] RON format for human-readable saves
- [ ] Tests: roundtrip save/load preserves card count and signatures, table positions restored, hand/stash placement correct

---

## Card Physics & Interaction Polish

### I15 — Card Physics Sleep Enforcement `[NOT STARTED]`
**Inspired by:** `CardSleepEnforcer` component — threshold-based sleep enforcement to prevent jittery card piles from inter-card collision wakeups
**Engine gaps:** `engine_physics` — expose rapier sleep thresholds per body, or add a system-level sleep enforcement query
**Why:** When multiple cards pile on the table, tiny collision impulses between stacked cards keep waking them up, causing perpetual jitter. A sleep enforcer checks linear + angular velocity against thresholds each frame and forces sleep when both are below threshold. Prevents visual noise in card piles without affecting actively moving cards.

- [ ] `SleepEnforcer` component: `linear_threshold: f32`, `angular_threshold: f32` (sane defaults)
- [ ] `sleep_enforcer_system` (Phase::PostUpdate): query `(Entity, &SleepEnforcer, &RigidBody)`, check velocity via `PhysicsBackend`, force sleep when below thresholds
- [ ] Add `body_linear_velocity(entity) -> Vec2` and `body_angular_velocity(entity) -> f32` to `PhysicsBackend` trait
- [ ] Add `set_body_sleeping(entity, bool)` to `PhysicsBackend` trait
- [ ] Tests: card below threshold sleeps, card above threshold stays awake, newly-impulse card wakes up

### I16 — Drop Preview (Landing Indicator) `[NOT STARTED]`
**Inspired by:** `DropPreview.cs` — ImmediateMesh line from card to physics-raycast landing point, shown during drop preparation
**Engine gaps:** None — can use existing Shape rendering for a line/circle indicator
**Why:** When the player is about to release a card, show where it will land. In 2D this translates to a simple downward projection: draw a translucent circle or crosshair at the point directly below the dragged card (or at the physics-projected landing spot). Gives tactile feedback for precise card placement onto deck slots or table regions.

- [ ] `DropPreview` component: marker for the preview indicator entity
- [ ] `drop_preview_system` (Phase::Update): when `DragState` is active, project card position downward (or use gravity direction), draw a Circle shape at the projected landing point
- [ ] Preview shape: translucent circle (alpha 0.3) at landing position, color based on whether a valid drop zone is beneath (green = valid slot, grey = table, red = invalid)
- [ ] Hide preview when not dragging
- [ ] Tests: preview visible during drag, hidden when not dragging, color changes over valid zones

### I17 — Card Highlight System `[NOT STARTED]`
**Inspired by:** `CardHighlighter.cs` + `CardController.cs` — visual feedback on hover/select with outline toggle and interaction orchestration
**Engine gaps:** None — achievable with shape overlay or border color change
**Why:** When the mouse hovers over a card, it should visually respond — a subtle glow, border brightening, or slight scale-up. This is the basic "this is interactive" feedback that makes the card table feel alive. CardCleaner uses outline box visibility toggling; in 2D we can brighten the border shape or add a translucent overlay.

- [ ] `Highlighted` component: marker added/removed by hover detection
- [ ] `card_highlight_system` (Phase::Update): query mouse position against card AABBs, add `Highlighted` to hovered card, remove from previously highlighted
- [ ] `highlight_render_system` (Phase::Render): draw a slightly larger translucent border shape behind highlighted cards (glow effect via Additive blend)
- [ ] Only one card highlighted at a time (closest to cursor wins)
- [ ] No highlight during drag (dragged card is implicitly selected)
- [ ] Tests: hover adds Highlighted, leaving removes it, only one card highlighted at a time, no highlight during drag

### I18 — Batched Card Spawning `[NOT STARTED]`
**Inspired by:** `CardSpawningService.cs` — spawns one card per frame from a queue to avoid frame hitches, with spawned-event for system hookup
**Engine gaps:** None — pure ECS command buffer pattern
**Why:** Spawning many cards at once (e.g., opening a loot pack, loading a save) causes frame spikes because each card involves entity creation + physics body + visual hierarchy. A spawn queue processes one (or N) cards per frame, spreading the cost. The spawned event lets other systems (highlight, slot detection) connect to new cards automatically.

- [ ] `CardSpawnQueue` resource: `VecDeque<CardSpawnRequest>` with `request { signature, position, zone }`
- [ ] `card_spawn_system` (Phase::PreUpdate): dequeue up to `MAX_SPAWNS_PER_FRAME` (default 3), spawn via existing `spawn_visual_card`
- [ ] `CardSpawnedBuffer` event buffer: `Vec<Entity>` of newly spawned cards this frame (drain pattern)
- [ ] `queue_card(queue, signature, position, zone)` helper function
- [ ] Tests: queued cards spawn over multiple frames, spawn count per frame respects limit, empty queue is no-op

---

## World Generation Detail

### I19 — WFC Soft Modifiers `[NOT STARTED]`
**Inspired by:** `CompactnessSoftModifier.cs`, `DiminishingReturnsSoftModifier.cs`, `NoveltySoftModifier.cs` — probability weight modifiers that shape WFC output aesthetics
**Engine gaps:** Requires I12 (WFC world gen) to exist first
**Why:** Raw WFC produces valid but aesthetically random maps. Soft modifiers shape the output: compactness encourages round blob shapes (corner-fill 1.3×, gap-fill 2.0× boost), diminishing returns penalizes over-concentration of one tile type, and novelty encourages variety. These don't add hard constraints — they bias probability weights during entropy-based cell selection.

- [ ] `WfcModifier` trait: `fn weight_modifier(&self, context: &WfcContext) -> f32` (multiplicative, 1.0 = neutral)
- [ ] `CompactnessModifier`: count same-type 4-neighbors → 0-1 neighbors: 1.0×, 2 neighbors: 1.3× (corner fill), 3-4 neighbors: 2.0× (gap fill)
- [ ] `DiminishingReturnsModifier`: track per-tile-type count, apply `1.0 / (1.0 + count * decay_rate)` penalty
- [ ] `NoveltyModifier`: boost tiles that haven't appeared recently in a local window
- [ ] Modifiers stored as `Vec<Box<dyn WfcModifier>>` on the WFC solver, applied multiplicatively during tile selection
- [ ] Tests: compactness boosts gap-fills, diminishing returns reduces dominant tiles, novelty increases variety

### I20 — Biome Distribution Preview `[NOT STARTED]`
**Inspired by:** `BiomeDistributionPreview.cs` — full-screen visualization showing estimated biome % as colored vertical strips based on current deck slot cards
**Engine gaps:** Requires I10 (deck slots) + I12 (world gen) + biome definitions
**Why:** Before committing cards to world generation, players want to see what kind of world they'll get. This preview samples the card gradient at multiple points, computes nearest-biome at each sample, and displays the biome distribution as colored strips. Updates live as cards are added/removed from deck slots. Helps players make informed decisions about which cards to sacrifice.

- [ ] `BiomePreview` resource: `Vec<(String, Color, f32)>` — biome name, display color, percentage
- [ ] `biome_preview_system` (Phase::Update): when deck slots have cards, sample gradient at N points (e.g., 64), compute nearest biome per sample, aggregate percentages
- [ ] `biome_preview_render_system` (Phase::Render): draw colored vertical strips proportional to biome %, with biome name labels
- [ ] Live update: recalculate whenever deck slot contents change
- [ ] Empty state: "Place cards to see biome distribution" text
- [ ] Tests: no cards shows empty state, one card shows uniform biome, adding cards changes distribution

### I21 — Fog of War & Line of Sight `[NOT STARTED]`
**Inspired by:** `FrontierExplorationBehavior.cs` + `SimpleVisibilityChecker.cs` — Bresenham line-of-sight with diagonal corner blocking, seen/visited/visible tile distinction, frontier exploration with blob prioritization
**Engine gaps:** Requires I12 (world gen tilemap) + tile passability/transparency data
**Why:** Exploration needs fog of war. Three tile states: unseen (black), seen but not visible (dim), currently visible (full). Visibility uses Bresenham's line algorithm with diagonal corner blocking (if both adjacent tiles to a diagonal step are opaque, block sight). Tiles "trivially visible" (all neighbors visible + connected to visited) auto-mark as visited without walking there. Significant blob prioritization (threshold 5) targets large unexplored regions first.

- [ ] `TileVisibility` enum: `Unseen`, `Seen`, `Visible` per tile in the map grid
- [ ] `VisibilityMap` resource: `Vec<TileVisibility>` (width × height)
- [ ] `can_see(from, to, map) -> bool`: Bresenham line-of-sight with diagonal corner blocking
- [ ] `update_vision(position, vision_range, map) -> (seen, visible)`: mark currently visible tiles, auto-visit trivially-visible ones
- [ ] `FrontierExplorer`: tracks seen/visited/visible sets, finds next target via largest-unvisited-blob heuristic
- [ ] Render: unseen tiles not drawn, seen tiles drawn at 40% opacity, visible tiles at 100%
- [ ] Tests: direct line-of-sight succeeds, wall blocks sight, diagonal corner blocks sight, trivial-visit optimization works

---

## Tilemap & Auto-Tiling

### I25 — Tilemap Grid System `[NOT STARTED]`
**Inspired by:** `SimpleMapData` + `SimpleMapGenerator` — 2D grid of tile IDs with layers (terrain, foreground, decoration), passability per tile, map-wide queries
**Engine gaps:** New `engine_tilemap` crate or module within `card_game` — grid storage, tile lookup, layer management, rendering integration
**Why:** World generation (I12) produces a grid of tile IDs, but we need a runtime representation to store, query, and render it. A tilemap is a 2D grid where each cell holds a tile ID (or stack of IDs for layers). The grid supports passability queries for pathfinding, neighbor lookups for auto-tiling, and iteration for rendering. In 2D this is the natural way to represent explorable terrain.

- [ ] `TileMap` resource: `width: usize`, `height: usize`, `layers: Vec<TileLayer>` where each layer is `Vec<Option<TileId>>`
- [ ] `TileId(u32)` newtype — index into a tile definition registry
- [ ] `TileMapLayer` enum: `Terrain`, `Foreground`, `Decoration` — determines render order and behavior
- [ ] Grid queries: `get(x, y, layer) -> Option<TileId>`, `set(x, y, layer, tile)`, `is_in_bounds(x, y) -> bool`
- [ ] `neighbors_4(x, y) -> [(i32, i32); 4]` and `neighbors_8(x, y) -> [(i32, i32); 8]` for adjacency lookups
- [ ] `tile_to_world(x, y, tile_size) -> Vec2` and `world_to_tile(pos, tile_size) -> (i32, i32)` coordinate conversion
- [ ] `tilemap_render_system` (Phase::Render): iterate visible tiles, draw as Rect or Sprite per tile, respects camera frustum
- [ ] Tests: get/set roundtrip, out-of-bounds returns None, coordinate conversion roundtrip, neighbor counts at edges

### I26 — Tile Definitions & Registry `[NOT STARTED]`
**Inspired by:** `TileDefinition.cs` + `TileRegistry.cs` — per-tile properties (passability, transparency, elevation, layer, biome restrictions, probability weight), registry with lookup by ID
**Engine gaps:** None — pure data model in `card_game`
**Why:** Each tile type needs defined properties: can you walk on it? Can you see through it? Which biomes allow it? What's its selection weight for WFC? A registry maps tile IDs to definitions, loaded from RON data files. Tile definitions are the bridge between the abstract WFC solver and the concrete rendered world.

- [ ] `TileDefinition` struct: `id: String`, `name: String`, `passability: TilePassability`, `is_transparent: bool`, `color: Color` (for shape-based rendering), `probability: f32` (WFC weight), `allowed_biomes: Option<Vec<String>>`
- [ ] `TilePassability` enum: `Passable`, `Blocked`, `Slow` (affects pathfinding cost)
- [ ] `TileRegistry` resource: `HashMap<TileId, TileDefinition>` with `register()`, `get()`, `is_passable()`, `is_transparent()`
- [ ] `is_allowed_in_biome(tile, biome) -> bool` — None means allowed everywhere
- [ ] Load definitions from RON data (code-defined assets philosophy — no external files, but data-driven within Rust)
- [ ] Tests: registry lookup, passability query, biome filtering, missing tile returns None

### I27 — Dual-Grid Auto-Tiling `[NOT STARTED]`
**Inspired by:** `DualGridAutoTile.cs` + `BitmaskConsistencyValidator.cs` — visual tiles offset by half a cell from data grid, 4-corner sampling → 4-bit bitmask, consistency validation between adjacent visual tiles
**Engine gaps:** Requires I25 (tilemap) + I26 (tile definitions)
**Why:** Raw tile grids look blocky — terrain transitions need smooth edges. The dual-grid technique offsets visual tiles by half a cell so each visual tile sits at the intersection of 4 data cells. A 4-bit bitmask (NE=1, SE=2, SW=4, NW=8) from those 4 corners selects the right transition sprite. This prevents invalid auto-tile states because the visual grid is always derived from the data grid. Three built-in formats: Corner16 (terrain transitions, 16 variants), Edge16 (walls/paths, 16 variants), Blob47 (detailed blobs, 47 variants with corner-requires-edge constraint).

- [ ] `DualGrid` struct: stores visual grid (`(width+1) × (height+1)`) of bitmask values, derived from data grid
- [ ] `compute_bitmask(visual_x, visual_y, data_grid) -> u8`: sample 4 corner data cells, build Corner16 bitmask
- [ ] `compute_all_bitmasks(data_grid) -> Vec<u8>`: batch compute for entire visual grid
- [ ] `visual_tile_position(vx, vy, tile_size) -> Vec2`: offset by -0.5 tiles for rendering
- [ ] `BitmaskFormat` enum: `Corner16`, `Edge16`, `Blob47` with `is_valid_bitmask(mask) -> bool`
- [ ] Blob47 validation: corners only valid when both adjacent edges are present (47 of 256 8-bit combinations)
- [ ] `AutoTileVariants`: maps `(tile_type, bitmask) -> visual_variant` (color/shape/sprite selection)
- [ ] Tests: all-filled corners = bitmask 15, single corner = correct bit, Blob47 rejects invalid masks, visual grid is data+1 in each dimension

### I28 — Biome Definitions & Affinity `[NOT STARTED]`
**Inspired by:** `BiomeDefinition.cs` + `BiomeRegistry.cs` + `BiomeMapGenerator.cs` — biomes with signature affinity, passable/blocked tile pools with weights, nearest-biome-by-signature selection, biome strength grids for smooth transitions
**Engine gaps:** Requires I1 (signatures) + I26 (tile definitions)
**Why:** Biomes are the bridge between card signatures and terrain. Each biome has an affinity signature (a CardSignature representing its "elemental personality"), plus pools of passable and blocked tiles with selection weights. At each map position, the card gradient (I12) produces a signature, and the biome with the closest affinity signature wins. Blocked percentage controls terrain density. This makes card identity directly shape the world — fire-heavy cards produce desert biomes, water-heavy cards produce swamp.

- [ ] `BiomeDefinition` struct: `id: String`, `affinity: CardSignature`, `passable_tiles: Vec<(TileId, f32)>` (weighted pool), `blocked_tiles: Vec<(TileId, f32)>` (weighted pool), `blocked_percentage: f32` (0.0–1.0)
- [ ] `BiomeRegistry` resource: `Vec<BiomeDefinition>` with `find_closest(signature) -> &BiomeDefinition` (min Euclidean distance in 8D space)
- [ ] `select_passable_tile(biome, rng) -> TileId` and `select_blocked_tile(biome, rng) -> TileId` — weighted random selection from pool
- [ ] `BiomeMap`: 2D grid of biome IDs, computed once from card gradient + registry during map generation
- [ ] Fallback biome for positions where no biome matches closely (plains with default tiles)
- [ ] Tests: closest biome by signature distance, weighted tile selection distribution, blocked percentage constrains tile ratio

### I29 — WFC Tile Solver `[NOT STARTED]`
**Inspired by:** `WfcSolver.cs` + `WfcGrid.cs` + `WfcMapGenerator.cs` — weighted entropy solver with possibility sets per cell, constraint propagation, two-phase generation (background + foreground), auto-tile gap enforcement, retry on contradiction
**Engine gaps:** Requires I25 (tilemap) + I26 (tile definitions) + I28 (biomes)
**Why:** Wave Function Collapse generates coherent tilemap terrain from constraints. Instead of hard socket rules, this uses "Soft WFC" — probability-based tile selection with multiplicative weight modifiers. Every tile has some probability everywhere (no hard bans), and modifiers (biome affinity, adjacency, spacing) shape the output. Entropy is weighted (not tile-count), so tiles with extreme probability ratios get resolved first. Two-phase generation: background layer (terrain) first, then foreground layer (auto-tiles with gap constraints). Contradictions trigger local backtracking and retry (up to 5 attempts).

- [ ] `WfcGrid` struct: `width × height` cells, each cell has `possible_tiles: Vec<(TileId, f32)>` (id + weight)
- [ ] `entropy(cell) -> f32`: weighted entropy (`-Σ p·log(p)` where `p = w / Σw`), not tile count
- [ ] `collapse(cell, rng)`: select tile by weighted random from possibilities, reduce to single tile
- [ ] `propagate(cell)`: remove incompatible tiles from neighbors, cascade until stable
- [ ] `solve(rng) -> Result<TileMap, WfcError>`: lowest-entropy-first loop with noise-based tiebreaker
- [ ] `WfcError` enum: `Contradiction { x, y }`, `MaxRetriesExceeded`
- [ ] Auto-tile gap constraint: prevent adjacent different auto-tile types (8-way check) — ensures at most one auto-tile type per 2x2 visual window
- [ ] Connectivity constraint: flood-fill passable regions, reject if disconnected
- [ ] Two-phase generation: background layer first (non-auto-tile terrain), foreground layer second (auto-tiles with gap constraints)
- [ ] Tests: 2×2 grid solves to valid configuration, contradiction triggers retry, connectivity rejects isolated regions, gap constraint prevents adjacent auto-tiles

---

## Game Polish

### I22 — Auto-Save System `[NOT STARTED]`
**Inspired by:** `GameSaveService.cs` — configurable auto-save timer (default 10s), auto-load on startup, deferred card recreation to avoid race conditions
**Engine gaps:** Requires I14 (serialization) to exist first
**Why:** Extends I14 with automatic periodic saving. A timer resource triggers saves at a configurable interval. Auto-load on startup reconstructs the game state. Deferred spawning (via `CardSpawnQueue` from I18) prevents frame hitches during load. The save path is deterministic (`saves/autosave.ron`).

- [ ] `AutoSaveConfig` resource: `interval_seconds: f32` (default 10.0), `enabled: bool`, `auto_load_on_start: bool`
- [ ] `AutoSaveTimer` resource: `elapsed: f32` (accumulates DeltaTime)
- [ ] `auto_save_system` (Phase::PostUpdate): accumulate time, trigger `save_game()` when elapsed >= interval, reset timer
- [ ] `auto_load_system` (startup): if `auto_load_on_start` and save file exists, queue cards via `CardSpawnQueue`
- [ ] Tests: save triggers at interval, save doesn't trigger before interval, disabled config prevents saves

### I23 — Generation Progress UI `[NOT STARTED]`
**Inspired by:** `LoadingScreen.cs` + `CancellableProgressProfiler.cs` — progress bar UI during map generation with phase-weighted progress (WFC 75%, biome 10%, passability 10%, variants 5%)
**Engine gaps:** `engine_ui` ProgressBar component already exists; needs a way to show/hide UI panels based on game state
**Why:** Map generation (I12) may take multiple frames. A loading screen with a progress bar shows the player what's happening. Phase-weighted progress gives accurate estimates (WFC dominates at 75% of total time). The screen auto-shows when entering `Generating` state and auto-hides when generation completes.

- [ ] `GenerationProgress` resource: `phase: GenerationPhase`, `progress: f32` (0.0–1.0)
- [ ] `GenerationPhase` enum with weights: `WfcTerrain(0.75)`, `BiomeMap(0.10)`, `Passability(0.10)`, `Variants(0.05)`
- [ ] `generation_progress_system` (Phase::Update): when `SessionState::Generating`, update progress bar UI entity value from `GenerationProgress.progress`
- [ ] Auto-show loading panel when entering Generating, auto-hide when leaving
- [ ] Tests: progress advances through phases, weights sum to 1.0, UI shows during generation only

### I24 — Pause System `[NOT STARTED]`
**Inspired by:** `PauseController.cs` — Escape key toggles pause, shows/hides pause menu, toggles game UI visibility
**Engine gaps:** `engine_ecs` — needs a `Paused` resource or schedule-skip mechanism to freeze game systems while allowing UI systems
**Why:** Basic game polish. Pressing Escape pauses the game (freezes physics, Update-phase systems) and shows a pause menu overlay. UI systems continue running so the menu is interactive. Unpausing resumes the game. Simple but essential for any playable game.

- [ ] `Paused` resource: `bool` (default false)
- [ ] `pause_system` (Phase::Input): toggle `Paused` on Escape key press
- [ ] Physics and game Update systems check `Paused` and early-return when true (or use a run condition)
- [ ] `PauseMenu` entity (Panel + Button children): visible only when `Paused` is true
- [ ] `pause_menu_render_system` (Phase::Render): draw pause overlay with "Resume" button
- [ ] Tests: escape toggles paused, systems skip when paused, UI responds when paused

---

## Dependency Graph

```
ENGINE HARDENING (tech debt — do first or alongside early features):
    TD-032 (e2e tests) ← do before adding more game systems
    TD-004 (tessellation cache) ← do before I6 (gem sockets add 8 shapes/card)
    TD-031 (observability) ← do before I10+ (more complex system interactions)
    TD-001/002/003 (change detection) ← do before I25 (tilemap adds hundreds of entities)
    TD-005 (material GPU) ← do before I5 (deterministic card gen needs shader visuals)
    TD-018 (physics interp) ← do before release polish
    TD-015 (color grading) ← do alongside I11 (session states benefit from mood lighting)

CARD IDENTITY (pure data, no engine deps):
    I1 (signature) ──→ I2 (base types) ──→ I3 (residual energy)
        │                   │                      │
        │                   └──→ I5 (deterministic gen) ──→ I14 (serialization) ──→ I22 (auto-save)
        │                            │                            │
        │                            └──→ I6 (gem sockets)        └──→ I18 (batched spawn)
        │                                 ↑
        │                           TD-004 (cache tessellation first!)
        │
        ├──→ I4 (rarity) ──→ I7 (rarity visuals)
        │
        ├──→ I9 (inspection) ← after I3 + I6 (needs stats + gems to show)
        │
        ├──→ I28 (biome defs) ← needs I1 + I26
        │
        ├──→ I10 (deck slots) ──→ I11 (session state machine) ──→ I23 (progress UI)
        │         ↑                    │
        │    TD-032 (e2e tests         ├──→ I12 (world gen gradient) ──→ I20 (biome preview)
        │     before more systems)     │
        │                              └──→ I13 (combat) ← needs I3 for ability derivation
        │
        └──→ I15 (sleep enforcer) ← standalone
             I16 (drop preview) ← standalone
             I17 (card highlight) ← standalone
             I24 (pause system) ← standalone

TILEMAP PIPELINE (do TD-001/002/003 change detection first!):
    I25 (tilemap grid) ──→ I26 (tile definitions) ──→ I28 (biome defs) ← needs I1
                                │                           │
                                └──→ I27 (dual-grid auto-tile)
                                │                           │
                                └──→ I29 (WFC solver) ←─────┘
                                          │
                                          ├──→ I19 (WFC soft modifiers)
                                          └──→ I21 (fog of war)

SUGGESTED ORDER:
  Phase 0 (hardening):  TD-032, TD-004, TD-031
  Phase 1 (identity):   I1 → I2 → I3 → I4 → I5 → I6 → I7 → I9
  Phase 2 (game loop):  I10 → I11 → I13, I15–I18, I24 (parallel standalone items)
  Phase 3 (hardening):  TD-001/002/003, TD-005
  Phase 4 (world gen):  I12 → I25 → I26 → I27 → I28 → I29 → I19–I21
  Phase 5 (persistence): I14 → I18 → I22
  Phase 6 (polish):     TD-018, TD-015, I23
```
