# GPS Companion — Biome Overlay + Grain Tube Visual - Requirements Spec

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

**CWD:** `C:\Users\siriu\RustroverProjects\Axiom2d\mobile\gps_companion`

**Anti-cheat:** Proofs stored canonically in MCP storage.
`dod_check` executes commands from the canonical copy, not this markdown file.
</claude_instructions>

**Goal:** Invert map paint to biome-colored fog with live OSM biomes + replace text HUD with grain tube (test-tube hourglass with CA sandpile settling), piston crush, and pack stack on map screen. Mobile only.

**Date:** 2026-06-11
**Target:** `C:\Users\siriu\RustroverProjects\Axiom2d\mobile\gps_companion`
**DoD ID:** `6bb01d92-c739-456f-8ae8-dc816868428f`
**Last check:** PASS (2026-06-11T15:14:44.176Z)

---

## Decisions (locked with user)

<decisions>
1. Biome source: live Overpass API + persistent per-region cache (deterministic on revisit, offline after first fetch). Reverses original embedded-GeoJSON design.
2. Density couples to grain signature magnitude on mobile (denser ground → higher-magnitude → rarer grains). Desktop is OUT of scope.
3. 10× = yield multiplier displayed AS overlay opacity — one density knob drives both grains/step and fog alpha; opacity reads as farmability. Range 0.5×…10×.
4. Minting is deferred until biome resolves — final minted grain set must be independent of Overpass network timing (strict determinism preserved).
5. Grain tube replaces "X% to next pack" and "# packs" text labels with visual tube (right) + pack stack (left). Purely visual — existing transfer mechanism unchanged.
6. Pixel colors: reuse desktop gem aspect_color() palette (16 Aspect colors).
7. Tube capacity: tied to packProgress %, ~90% full when progress = 100%, tube ~20px wide.
8. Piston: literal piston head descends, compresses pixel mass into block (5×20px thin rectangle), retracts.
9. Pack stack: vertical, new packs add to bottom bumping previous up, unbounded off top of screen.
</decisions>

## Current state

<current_state>
Verified 2026-06-08: coverage.dart CoverageMap.harvest() is pure/sync; paints covered cells; mints grains from accumulated signature. kYieldMultiplier = 20.0. vacuum() always defaults Biome.noData — biome detection does not exist. map_screen.dart renders _painted cell centres as amber polygons, capped _maxPaintedRendered = 2500. mintGrainFromSignature normalizes magnitude to scale = 0.15. Mint seed is _hash(_hash(day, week), _mintCount++) — accrual-order-dependent. Biome enum has 6 entries; kBiomeDistribution maps each to 5-way grain dist. Grain toJson includes axes.

Verified 2026-06-11: map_screen.dart HUD shows Column with _chip('X% to next pack'), _chip('N packs'), theme, status, background toggle. Desktop has no grain import (crates/card_game/src/import/ does not exist). Desktop BoosterMachine takes SignatureSpace via cable/jack, seals into BoosterPack. Desktop gem_sockets.rs defines aspect_color() with 16 Aspect→Color mappings.
</current_state>

## Requirements

<requirements>
## Feature A: Biome Overlay (from existing plan)

### A1. Biome data model
- BiomeSample { key, densityPerKm2, dist (5-way, sums to 1.0±0.001), colorArgb }
- osm_tags.dart: ≥25 distinct OSM tag→BiomeSample mappings (landuse/*, natural/*, leisure/*, waterway/*, wetland/*)
- micro_biome.dart: POI category → grain bias + density bump, radius 5–10m, falloff monotonic with distance

### A2. BiomeService (Overpass + cache)
- Overpass query: bbox ~300–500m around player, pulls landuse/natural/leisure ways + POI nodes
- Parse to region polygons + POI points → BiomeSample grid
- Persist parsed region samples (shared_preferences, keyed by coarse tile id)
- sampleAt(lat,lon) sync from cache: POI override → polygon → null (pending/unknown)
- prefetch(lat,lon) debounced background-fill, respects Overpass rate limits
- Add http as direct dependency

### A3. Density coupling + deferred deterministic minting
- harvest() takes BiomeSample resolver (not fixed Biome)
- Yield factor f = clamp(density/kBaseDensity, 0.5, 10.0)
- Signature magnitude: scale = 0.15 * sqrt(f)
- Opacity: alpha = lerp(0.06, 0.92, norm(f))
- Deferred minting: cache-miss cells queued, accrued in sorted cell-id order when resolved
- Mint seed from day+week+cell_id (not _mintCount++)
- Determinism: same path = same grain list regardless of biome resolution timing

### A4. Render inversion
- Remove walked-path amber polygons
- Coarse ~25m fog grid, colored sample.colorArgb at density-driven alpha
- Fog tiles skipped where harvest cells are covered
- Viewport-culled, capped 2500 tiles
- Keep player dot, brush circle, recenter, HUD

### A5. Privacy disclosure
- One bullet in gps_companion_privacy.md: Overpass requests include approximate viewport bbox, no account/identifier, cached on-device after first fetch.

## Feature B: Grain Tube Visual (new)

### B1. Grain pixel color model
- Map (GrainType, sign of dominant axis) → Color using the 16-color aspect_color() palette from desktop gem_sockets.rs
- Dominant axis = axis index with largest absolute magnitude
- Positive axis → warm aspect color, negative axis → cool aspect color

### B2. Tube widget
- Glass test-tube shape: vertical cylinder with rounded bottom, transparent/translucent walls, ~20px wide
- CustomPainter on map screen, positioned on RIGHT side
- Tube walls visible, interior shows settled grain mass + falling pixels

### B3. Pixel physics
- Each harvested grain = 1 colored pixel (1-2px) appears at tube opening (top)
- Small random initial velocity, falls under gravity (~0.3s typical drop)
- Collision detection against:
  - Tube walls (left/right boundaries) — bounce/stop
  - Top surface of settled sand mass — transition to CA settling phase
- Physics is simple kinematic: position += velocity * dt, velocity += gravity * dt
- Individual pixel grains remain visible (color distinguishable)

### B4. CA settling
- Tube interior discretized into pixel-sized grid cells
- On contact with sand surface, pixel switches from physics to CA settling:
  - Try settle at contact point
  - If unstable (no support below, or exceeds angle of repose ~30-35°) → push out to adjacent lower cells (below-left, below-right)
  - Existing settled pixels may shift (push-out displacement)
  - Repeat until stable
- Natural sandpile emerges — slopes form at angle of repose, no vertical pillars

### B5. Piston animation
- Tube reaches ~90% full (= packProgress 100%) → piston triggers automatically
- Literal piston head shape descends from above tube
- Compresses pixel mass into solid block
- Retracts back up
- Tube resets to empty (packProgress resets)

### B6. Pack block animation
- Crushed block (~5×20px thin rectangle) emerges at tube position
- Animates from RIGHT side (tube position) to LEFT side (stack position)
- Duration: ~0.5s

### B7. Stack widget
- Vertical stack on LEFT side of map screen
- Each pack = thin rectangle (~5×20px), matches pack block size
- New packs add to bottom, bumping previous packs up
- Stack grows unbounded off top of screen
- Replaces "N packs" text label

### B8. Map screen integration
- Remove "X% to next pack" _chip → replace with tube widget (RIGHT side)
- Remove "N packs" _chip → replace with stack widget (LEFT side)
- Theme, status, background toggle remain unchanged
- Tube fill level drives from state.packProgress
- Stack count drives from state.packCount
- Piston fires when packProgress wraps to 0 (new pack created)
- Pack block animation triggered on piston completion
- Stack adds new pack on block arrival

### What it does NOT do
- No desktop grain import
- No change to grain transfer mechanism
- No per-day coverage persistence across app restarts
- No sum-into-clamp-saturation
</requirements>

## Research Notes

<research_notes>
Files to create (biome): lib/domain/biome_def.dart, lib/domain/osm_tags.dart, lib/domain/micro_biome.dart, lib/data/overpass.dart, lib/data/biome_service.dart
Files to edit (biome): lib/domain/coverage.dart, lib/domain/spawn.dart, lib/ui/app_state.dart, lib/ui/map_screen.dart, pubspec.yaml, docs/plans/gps_companion_privacy.md

Files to create (grain tube): lib/domain/grain_pixel.dart (color model), lib/ui/tube_painter.dart (CustomPainter), lib/ui/pixel_physics.dart (gravity + collision), lib/ui/ca_settling.dart (sandpile algorithm), lib/ui/piston_animation.dart, lib/ui/pack_stack.dart
Files to edit (grain tube): lib/ui/map_screen.dart (HUD replacement)

Determinism hazard: current mint seed _hash(_hash(day, week), _mintCount++) is accrual-order-dependent. Deferred minting changes accrual order → make mint seed position-derived OR accrue queued cells in fixed sorted order.

Desktop aspect_color() palette (16 colors): Solid=amber(0.85,0.55,0.20), Heat=red-orange(0.95,0.25,0.10), Order=gold(0.90,0.80,0.10), Light=yellow(0.98,0.95,0.40), Change=yellow-green(0.70,0.85,0.10), Force=orange(0.90,0.40,0.05), Growth=green(0.20,0.80,0.20), Expansion=lime(0.60,0.90,0.30), Fragile=periwinkle(0.30,0.50,0.85), Cold=ice-blue(0.10,0.70,0.95), Chaos=violet(0.55,0.10,0.80), Dark=indigo(0.15,0.05,0.40), Stasis=steel-blue(0.20,0.60,0.80), Calm=teal(0.10,0.75,0.70), Decay=muted-purple(0.35,0.20,0.60), Contraction=navy(0.05,0.20,0.70)

Mobile Grain model: axes[8], GrainType (8 types = 8 Elements), GrainRarity, dominantAxis = index of largest |axis value|. Sign of dominant axis value determines positive/negative Aspect.

Existing map_screen.dart HUD: Column with _chip('packProgress% to next pack'), _chip('packCount packs'), themeLabel, status, background toggle. Positioned top-left via Padding.
</research_notes>

## Open Questions

<open_questions>
Desktop import plan: build crates/card_game/src/import/grain_unpack.rs honoring Desktop Contract (magnitude-preserving aggregation). Separate plan; not started.

Per-day coverage persistence (closing app mid-day loses painted overlay) — unchanged from current caveat, out of scope.
</open_questions>

---

## Definition of Done

<definition_of_done>

### Biome data model — biome_def.dart, osm_tags.dart, micro_biome.dart [x]

  - [x] Proof: `C:\Users\siriu\flutter\bin\flutter analyze lib/domain/biome_def.dart lib/domain/osm_tags.dart lib/domain/micro_biome.dart` -> flutter analyze exits 0, no errors/warnings on biome data model files <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `C:\Users\siriu\flutter\bin\flutter test test/domain/osm_tags_test.dart` -> ≥25 distinct tag→BiomeSample mappings, every dist sums to 1.0 ±0.001 <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `C:\Users\siriu\flutter\bin\flutter test test/domain/micro_biome_test.dart` -> POI within radius blends bias, POI outside radius has zero effect, falloff monotonic with distance <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `rtk grep "class BiomeSample" lib/domain/biome_def.dart` -> BiomeSample has fields key, densityPerKm2, dist, colorArgb <!--p:{"type":"exit_code","value":0}-->

### BiomeService — Overpass parse + cache + sync sample [x]

  - [x] Proof: `rtk grep "http:" pubspec.yaml` -> http present under direct dependencies (not only transitive) <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `C:\Users\siriu\flutter\bin\flutter test test/data/overpass_test.dart` -> parses committed Overpass JSON fixture into ≥1 region polygon and ≥1 POI point <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `C:\Users\siriu\flutter\bin\flutter test test/data/biome_service_test.dart` -> cache hit returns sample, POI override beats polygon, miss returns null (NOT noData sample) <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `rtk grep "Future<void> prefetch" lib/data/biome_service.dart` -> debounced prefetch method exists <!--p:{"type":"exit_code","value":0}-->

### Density coupling + deferred deterministic minting (coverage.dart, spawn.dart) [x]

  - [x] Proof: `C:\Users\siriu\flutter\bin\flutter test test/domain/coverage_test.dart` -> yield factor clamps to [0.5, 10.0] at extreme densities; signature magnitude scales with density (scale = 0.15*sqrt(f)) <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `C:\Users\siriu\flutter\bin\flutter test test/domain/coverage_test.dart` -> determinism test: same path with biome resolved up-front vs deferred produces identical minted-grain list (same axes, type, rarity, order) <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `rtk grep "_mintCount" lib/domain/coverage.dart` -> mint seed no longer depends on _mintCount++ order; seed derived from day, week, and cell id (OR queued cells accrue in sorted order) <!--p:{"type":"exit_code","value":1}-->
  - [x] Proof: `rtk grep "sqrt" lib/domain/spawn.dart` -> mintGrainFromSignature scales magnitude by sqrt(f) <!--p:{"type":"exit_code","value":0}-->

### Grain pixel color model — dominant aspect → Color mapping (grain_pixel.dart) [x]

  - [x] Proof: `C:\Users\siriu\flutter\bin\flutter analyze lib/domain/grain_pixel.dart` -> flutter analyze exits 0, no errors/warnings <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `C:\Users\siriu\flutter\bin\flutter test test/domain/grain_pixel_test.dart` -> dominantAxis correctly identifies axis with largest absolute magnitude; positive value maps to warm aspect color, negative to cool aspect color <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `C:\Users\siriu\flutter\bin\flutter test test/domain/grain_pixel_test.dart` -> all 8 GrainType × 2 signs = 16 distinct color outputs; each matches desktop aspect_color() palette <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `rtk grep "aspectColor\|aspect_color\|grainColor" lib/domain/grain_pixel.dart` -> color lookup function exists mapping (GrainType, sign) → Color <!--p:{"type":"exit_code","value":0}-->

### Tube widget — glass test-tube CustomPainter (tube_painter.dart) [x]

  - [x] Proof: `C:\Users\siriu\flutter\bin\flutter analyze lib/ui/tube_painter.dart` -> flutter analyze exits 0, no errors/warnings <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `C:\Users\siriu\flutter\bin\flutter test test/ui/tube_widget_test.dart` -> tube renders with transparent/translucent walls, rounded bottom, ~20px width <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `C:\Users\siriu\flutter\bin\flutter test test/ui/tube_widget_test.dart` -> interior shows settled grain mass (colored pixels filling from bottom) and empty space above proportional to fill level <!--p:{"type":"exit_code","value":0}-->

### Pixel physics — gravity, wall collision, sand surface collision (pixel_physics.dart) [x]

  - [x] Proof: `C:\Users\siriu\flutter\bin\flutter analyze lib/ui/pixel_physics.dart` -> flutter analyze exits 0, no errors/warnings <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `C:\Users\siriu\flutter\bin\flutter test test/ui/pixel_physics_test.dart` -> pixel accelerates downward under gravity; velocity increases each frame <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `C:\Users\siriu\flutter\bin\flutter test test/ui/pixel_physics_test.dart` -> pixel bounces/stops at tube wall boundaries (left/right) <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `C:\Users\siriu\flutter\bin\flutter test test/ui/pixel_physics_test.dart` -> pixel stops falling when it contacts top surface of settled sand mass; switches to CA settling phase <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `C:\Users\siriu\flutter\bin\flutter test test/ui/pixel_physics_test.dart` -> pixel locks position when velocity drops below threshold AND it has settled; no further motion <!--p:{"type":"exit_code","value":0}-->

### CA settling — sandpile with angle of repose (ca_settling.dart) [x]

  - [x] Proof: `C:\Users\siriu\flutter\bin\flutter analyze lib/ui/ca_settling.dart` -> flutter analyze exits 0, no errors/warnings <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `C:\Users\siriu\flutter\bin\flutter test test/ui/ca_settling_test.dart` -> pixel placed on flat surface settles directly above supporting pixel; no lateral displacement <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `C:\Users\siriu\flutter\bin\flutter test test/ui/ca_settling_test.dart` -> pixel placed on steep slope (exceeding angle of repose ~30-35°) pushes out to adjacent lower cell until stable <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `C:\Users\siriu\flutter\bin\flutter test test/ui/ca_settling_test.dart` -> natural sandpile forms with slope at angle of repose; no vertical pillars (≥2 pixels unsupported cannot remain) <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `C:\Users\siriu\flutter\bin\flutter test test/ui/ca_settling_test.dart` -> existing settled pixels may shift when push-out displacement requires it; final configuration is stable <!--p:{"type":"exit_code","value":0}-->

### Piston animation — descend, compress, retract + pack block emerges [x]

  - [x] Proof: `C:\Users\siriu\flutter\bin\flutter analyze lib/ui/piston_animation.dart` -> flutter analyze exits 0, no errors/warnings <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `C:\Users\siriu\flutter\bin\flutter test test/ui/piston_test.dart` -> when tube is full (packProgress ≥ 100%), piston triggers: piston head descends from above tube <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `C:\Users\siriu\flutter\bin\flutter test test/ui/piston_test.dart` -> piston compresses pixel mass into solid block (removing individual grain visibility); retracts after compression <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `C:\Users\siriu\flutter\bin\flutter test test/ui/piston_test.dart` -> after piston retracts, a pack block (thin rectangle ~5×20px) appears at tube position; tube is empty <!--p:{"type":"exit_code","value":0}-->

### Pack block animation — right-to-left movement to stack [x]

  - [x] Proof: `C:\Users\siriu\flutter\bin\flutter test test/ui/piston_test.dart` -> pack block animates from tube position (right side) to stack position (left side) over ~0.5s <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `C:\Users\siriu\flutter\bin\flutter test test/ui/piston_test.dart` -> pack block disappears at tube position and reappears at stack position in a continuous animated movement <!--p:{"type":"exit_code","value":0}-->

### Stack widget — vertical pack stack on left side (pack_stack.dart) [x]

  - [x] Proof: `C:\Users\siriu\flutter\bin\flutter analyze lib/ui/pack_stack.dart` -> flutter analyze exits 0, no errors/warnings <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `C:\Users\siriu\flutter\bin\flutter test test/ui/pack_stack_test.dart` -> stack renders on LEFT side of screen; packs displayed as thin rectangles (~5×20px) in vertical column <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `C:\Users\siriu\flutter\bin\flutter test test/ui/pack_stack_test.dart` -> new pack adds to bottom of stack, existing packs shift upward <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `C:\Users\siriu\flutter\bin\flutter test test/ui/pack_stack_test.dart` -> stack count matches state.packCount; stack grows unbounded (packs scroll off top of screen) <!--p:{"type":"exit_code","value":0}-->

### Map screen HUD integration — replace text labels with tube + stack [x]

  - [x] Proof: `rtk grep "to next pack" lib/ui/map_screen.dart` -> old 'X% to next pack' text chip removed <!--p:{"type":"exit_code","value":1}-->
  - [x] Proof: `rtk grep "packs'" lib/ui/map_screen.dart` -> old 'N packs' text chip removed <!--p:{"type":"exit_code","value":1}-->
  - [x] Proof: `rtk grep "TubePainter\|GrainTube\|tube_painter" lib/ui/map_screen.dart` -> tube widget integrated on right side of map screen <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `rtk grep "PackStack\|pack_stack" lib/ui/map_screen.dart` -> stack widget integrated on left side of map screen <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `C:\Users\siriu\flutter\bin\flutter test test/ui/grain_hud_test.dart` -> theme label, status, and background toggle remain present and unchanged <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `C:\Users\siriu\flutter\bin\flutter test test/ui/grain_hud_test.dart` -> tube fill level tracks state.packProgress; stack count tracks state.packCount <!--p:{"type":"exit_code","value":0}-->

### Render inversion — fog over uncovered, cleared on vacuum (map_screen.dart) [x]

  - [x] Proof: `rtk grep "amber" lib/ui/map_screen.dart` -> walked-path amber polygons removed <!--p:{"type":"exit_code","value":1}-->
  - [x] Proof: `C:\Users\siriu\flutter\bin\flutter test test/domain/density_test.dart` -> alpha = lerp(0.06,0.92,norm(f)) is monotonic and bounded within [0.06, 0.92]; yield factor clamps <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `rtk grep "2500\|_maxFogTiles\|cull" lib/ui/map_screen.dart` -> fog render is viewport-culled and capped at 2500 tiles <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `C:\Users\siriu\flutter\bin\flutter test test/domain/coverage_test.dart` -> covered cells are excluded from fog set (fog tile whose underlying harvest cells are all covered is not emitted) <!--p:{"type":"exit_code","value":0}-->

### Wire-up + privacy + full suite [x]

  - [x] Proof: `rtk grep "biome" lib/ui/app_state.dart` -> vacuum threads a BiomeService resolver and triggers prefetch on meaningful move <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `rtk grep -i "overpass" ../../docs/plans/gps_companion_privacy.md` -> privacy-policy disclosure bullet for Overpass egress present <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `C:\Users\siriu\flutter\bin\flutter analyze` -> flutter analyze exits 0, no errors/warnings across the project <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `C:\Users\siriu\flutter\bin\flutter test` -> flutter test exits 0, full suite passes (existing + new tests) <!--p:{"type":"exit_code","value":0}-->
  - [x] Proof: `rtk grep "TODO\|FIXME" lib/ui/tube_painter.dart lib/ui/pixel_physics.dart lib/ui/ca_settling.dart lib/ui/piston_animation.dart lib/ui/pack_stack.dart lib/domain/grain_pixel.dart` -> no TODO or FIXME stubs in grain tube source files <!--p:{"type":"exit_code","value":1}-->

</definition_of_done>

## Open risks

<open_risks>
Overpass latency/rate limits → debounce + cache + deferred-mint (no UI block).

Fog polygon count at low zoom → coarse grid + viewport cull + 2500 cap; measure frame cost.

Determinism diverges if OSM edited between visits → acceptable; cache pins first read. Deferred minting handles within-session timing; cross-session OSM edits out of scope.

Grain tube physics: ~100 pixels max in tube. O(n) per-frame gravity + O(k) CA settling per new grain. Should be well within frame budget. CA settling worst-case: grain cascades down steep pile edge; bound by tube height in pixels.

Map screen touches both biome fog AND grain HUD → coordinate edits to map_screen.dart to avoid merge conflicts between features.
</open_risks>

## Amendment log

- **2026-06-11T14:18:21.561Z** [undefined] modified: flutter not on PATH in dod-guard MCP environment; using full path to flutter.bat
- **2026-06-11T14:19:06.082Z** [undefined] modified: flutter not on PATH in dod-guard MCP environment
- **2026-06-11T14:19:10.409Z** [undefined] modified: flutter not on PATH in dod-guard MCP environment
- **2026-06-11T14:19:15.214Z** [undefined] modified: flutter not on PATH in dod-guard MCP environment
- **2026-06-11T14:19:19.658Z** [undefined] modified: flutter not on PATH in dod-guard MCP environment
- **2026-06-11T14:19:37.769Z** [undefined] modified: flutter not on PATH in dod-guard MCP environment
- **2026-06-11T14:19:40.667Z** [undefined] modified: flutter not on PATH in dod-guard MCP environment
- **2026-06-11T14:19:44.927Z** [undefined] modified: flutter not on PATH in dod-guard MCP environment
- **2026-06-11T14:19:48.880Z** [undefined] modified: flutter not on PATH in dod-guard MCP environment
- **2026-06-11T14:20:17.262Z** [undefined] modified: flutter not on PATH in dod-guard MCP environment
- **2026-06-11T14:20:21.660Z** [undefined] modified: flutter not on PATH in dod-guard MCP environment
- **2026-06-11T14:20:25.904Z** [undefined] modified: flutter not on PATH in dod-guard MCP environment
- **2026-06-11T14:24:39.817Z** [undefined] modified: flutter not on PATH in current dod-guard session
- **2026-06-11T14:24:45.047Z** [undefined] modified: flutter not on PATH in current dod-guard session
- **2026-06-11T14:24:49.847Z** [undefined] modified: flutter not on PATH in current dod-guard session
- **2026-06-11T14:24:54.588Z** [undefined] modified: flutter not on PATH in current dod-guard session
- **2026-06-11T14:24:59.776Z** [undefined] modified: flutter not on PATH in current dod-guard session
- **2026-06-11T14:31:39.486Z** [undefined] modified: flutter not on PATH in dod-guard MCP environment; use full path
- **2026-06-11T14:31:41.137Z** [undefined] modified: flutter not on PATH in dod-guard MCP environment; use full path
- **2026-06-11T14:31:42.699Z** [undefined] modified: flutter not on PATH in dod-guard MCP environment; use full path
- **2026-06-11T14:31:44.147Z** [undefined] modified: flutter not on PATH in dod-guard MCP environment; use full path
- **2026-06-11T14:49:30.788Z** [undefined] modified: flutter not on PATH in dod-guard MCP environment; use full path
- **2026-06-11T14:49:32.658Z** [undefined] modified: flutter not on PATH in dod-guard MCP environment; use full path
- **2026-06-11T14:49:34.538Z** [undefined] modified: flutter not on PATH in dod-guard MCP environment; use full path
- **2026-06-11T14:49:36.242Z** [undefined] modified: flutter not on PATH in dod-guard MCP environment; use full path
- **2026-06-11T15:02:03.384Z** [undefined] modified: flutter not on PATH in dod-guard MCP; use full path
- **2026-06-11T15:02:04.763Z** [undefined] modified: flutter not on PATH in dod-guard MCP; use full path
- **2026-06-11T15:02:06.125Z** [undefined] modified: flutter not on PATH in dod-guard MCP; use full path
- **2026-06-11T15:02:07.366Z** [undefined] modified: flutter not on PATH in dod-guard MCP; use full path
- **2026-06-11T15:02:14.717Z** [undefined] modified: flutter not on PATH in dod-guard MCP; use full path
- **2026-06-11T15:02:16.301Z** [undefined] modified: fix corrupted canonical: was pixel_physics_test, should be ca_settling_test
- **2026-06-11T15:02:17.711Z** [undefined] modified: fix corrupted canonical: was ca_settling.dart, should be piston_animation.dart
- **2026-06-11T15:02:19.038Z** [undefined] modified: fix corrupted canonical: was ca_settling_test, should be piston_test
- **2026-06-11T15:02:24.261Z** [undefined] modified: fix corrupted canonical: was ca_settling_test, should be piston_test
- **2026-06-11T15:02:25.664Z** [undefined] modified: flutter not on PATH in dod-guard MCP; use full path
- **2026-06-11T15:02:27.065Z** [undefined] modified: flutter not on PATH in dod-guard MCP; use full path
- **2026-06-11T15:02:28.288Z** [undefined] modified: flutter not on PATH in dod-guard MCP; use full path
- **2026-06-11T15:02:32.384Z** [undefined] modified: flutter not on PATH in dod-guard MCP; use full path
- **2026-06-11T15:02:33.854Z** [undefined] modified: flutter not on PATH in dod-guard MCP; use full path
- **2026-06-11T15:02:35.209Z** [undefined] modified: flutter not on PATH in dod-guard MCP; use full path
- **2026-06-11T15:02:36.489Z** [undefined] modified: flutter not on PATH in dod-guard MCP; use full path
- **2026-06-11T15:02:41.846Z** [undefined] modified: flutter not on PATH in dod-guard MCP; use full path
- **2026-06-11T15:02:43.233Z** [undefined] modified: flutter not on PATH in dod-guard MCP; use full path
- **2026-06-11T15:02:44.613Z** [undefined] modified: flutter not on PATH in dod-guard MCP; use full path
- **2026-06-11T15:02:45.838Z** [undefined] modified: flutter not on PATH in dod-guard MCP; use full path
- **2026-06-11T15:02:50.352Z** [undefined] modified: flutter not on PATH in dod-guard MCP; use full path
- **2026-06-11T15:02:51.521Z** [undefined] modified: flutter not on PATH in dod-guard MCP; use full path
