# Mobile: Journey Integration Tests + Fix Update Lag & Grain Physics — Requirements Spec

> **For Claude (/goal):** Work through each incomplete step below.
> 1. Mark a step `[>]` when you begin working on it.
> 2. Call `dod_check` to verify proofs — do NOT mark proofs manually.
> 3. A step is complete when ALL its proofs pass via `dod_check`.
> 4. If a proof cannot be met, use `dod_amend` to modify it with a reason.
> 5. Continue until `dod_check` returns PASS — then stop and report done.
>
> **Self-contained.** All commands run from `C:\Users\siriu\RustroverProjects\Axiom2d\mobile\gps_companion` unless noted.
>
> **🔒 Anti-cheat:** Proofs are stored canonically in MCP storage (dod-guard).
> `dod_check` executes commands from the canonical copy, not this markdown file.
> Editing proof text here has no effect on verification.
> Store tampering is **logged and detectable** — each check prints a proof-set fingerprint.

**Goal:** Build a Flutter widget test harness that simulates a GPS journey in Central London, exposes and fixes the 30s update lag bug and the grains-not-falling bug, and verifies a full booster is forged end-to-end.

**Date:** 2026-06-12
**Target:** `C:\Users\siriu\RustroverProjects\Axiom2d\mobile\gps_companion`
**DoD ID:** `d5d4a333-845a-4cd2-aa84-081b55f2afd6`
**Last check:** PASS (2026-06-12T10:24:41.178Z)

---

## Decisions (locked with user)

- Test tier: Flutter widget tests (headless, flutter_test) — no device needed, runs in CI
- Biome data: real Overpass API (no mocking) — tests require network, ~15-30s expected
- Journey location: Central London (51.51°N, -0.12°W) — dense OSM coverage
- Journey length: long enough to forge 1 booster (100 grains)
- Bug scope: write failing tests first (TDD RED), then fix, tests become regression guards
- AppState/MapScreen GPS: extract GpsService abstraction so tests inject fake GPS without hardware

## Current state

Zero integration tests. All 15+ test files are unit-level. Two confirmed bugs:
1. 30s update lag: `_onPosition()` in map_screen.dart awaits vacuum() which calls biome_service.prefetch() → _doFetch() → unbounded http.get() with no timeout. Also called in a loop for projected intermediate GPS points.
2. Grains not falling: stepFalling() in pixel_physics.dart always returns false (line 88). _stepSimulation() only calls setState() when changed=true (set only on settle, never during active fall). Physics state updates but UI never redraws.

## Requirements

## Core requirements

1. **GpsService abstraction** — abstract class injected into MapScreen so widget tests supply fake GPS stream without hardware
2. **FakeGpsService** — test helper that accepts emitted GpsPoints programmatically via a stream controller
3. **Bug 1 fix: 30s update lag**
   - Add `.timeout(Duration(seconds: 8))` to the `http.get()` call in `biome_service.dart:_doFetch()`
   - Decouple biome prefetch from blocking vacuum(): fire prefetch as fire-and-forget in `_onPosition()`, vacuum() must not await it
   - Failing test written first (TDD RED) asserting vacuum() completes within 5s on cold tile
4. **Bug 2 fix: grains not falling**
   - Fix `stepFalling()` in `pixel_physics.dart` to return `true` when the pixel has non-negligible velocity or hasn't settled yet
   - Fix `_stepSimulation()` in `map_screen.dart` to call `setState()` whenever any pixel is still active (not just on settle)
   - Failing test written first (TDD RED) asserting stepFalling() returns true for an airborne pixel
5. **Journey integration test** — `test/integration/journey_test.dart`
   - Injects Central London GPS path (~200+ points in a grid/zigzag pattern to cover paint cells)
   - Uses real Overpass API for biome data
   - Asserts `inventory.boosters.length >= 1` after journey completes
   - Tagged `@Tags(['integration'])` so it can be excluded from fast CI runs
6. **All non-integration tests still pass** after changes

## What is NOT in scope
- Device-level integration_test / flutter drive
- Mocking the Overpass API
- WebSocket transfer tests
- Background GPS service (isolated Dart context)

## Research Notes

## Bug 1: 30s update lag — call chain
- `map_screen.dart:345` — `_onPosition()` async, awaits vacuum multiple times
- `map_screen.dart:377-394` — projection loop: awaits vacuum() + logPosition() for each intermediate point
- `map_screen.dart:410-416` — final vacuum() call, also awaited
- `app_state.dart:89-129` — vacuum() itself returns quickly, but...
- `biome_service.dart:203-227` — prefetch() starts a 200ms debounce timer
- `biome_service.dart:258-296` — _doFetch() contains `await _http.get(url)` with NO timeout
- Result: first Overpass tile fetch blocks entire _onPosition() callback for however long the HTTP request takes (30s+ on slow connections)

## Bug 2: grains not falling — call chain
- `map_screen.dart:111` — ticker created and started (ticker IS running)
- `map_screen.dart:133-150` — _onTick() calls _stepSimulation() if dt > 0
- `map_screen.dart:152-192` — _stepSimulation() calls stepFalling(px, ...) line 160, then checks `changed || anyActive` for setState at line 191
- `pixel_physics.dart:61-88` — stepFalling() modifies pixel position and velocity correctly, but ALWAYS returns false at line 88
- Result: `changed` stays false every tick, setState() never called during fall, UI frozen
- Fix: stepFalling() should return true when abs(vy) > settle_threshold OR pixel hasn't reached settled y position

## Architecture
- GPS stream: `Geolocator.getPositionStream()` called directly in `_MapScreenState` (map_screen.dart:100)
- AppState: `lib/ui/app_state.dart` — ChangeNotifier, vacuum() at line 89, logPosition() at line 132
- Coverage: `lib/domain/coverage.dart:140` — harvest() mints grains from painted cells
- Grain tube: pixels in `map_screen.dart._pixels` list, physics in `pixel_physics.dart`, CA settling in `ca_settling.dart`, rendered by `TubePainter`
- Existing tests: 15+ unit test files, zero integration tests
- Test location: `test/` directory (not `tests/` — Flutter convention)

---

## Definition of Done

### Step 1: Extract GpsService abstraction and inject into MapScreen [x]

- [x] Proof: `grep -n "abstract class GpsService" lib/data/gps_service.dart` → GpsService abstract class exists in lib/data/gps_service.dart
- [x] Proof: `grep -n "Stream.*Position" lib/data/gps_service.dart` → GpsService exposes a Stream<Position>
- [x] Proof: `grep -n "GpsService" lib/ui/map_screen.dart` → MapScreen accepts/uses GpsService (injection wired up)
- [x] Proof: `grep -n "Geolocator.getPositionStream" lib/ui/map_screen.dart` → exit 1 — direct Geolocator call removed from MapScreen (routed through GpsService)
- [x] Proof: `C:\Users\siriu\flutter\bin\flutter.bat analyze lib/` → No analysis errors after refactor

### Step 2: Create FakeGpsService test helper [x]

- [x] Proof: `grep -n "class FakeGpsService" test/helpers/fake_gps_service.dart` → FakeGpsService class exists in test/helpers/
- [x] Proof: `grep -n "StreamController\|emit\|add" test/helpers/fake_gps_service.dart` → FakeGpsService has a stream controller and emit/add mechanism
- [x] Proof: `grep -n "implements GpsService" test/helpers/fake_gps_service.dart` → FakeGpsService implements the GpsService interface

### Step 3: Write RED test: vacuum() completes within 5s on cold Overpass tile (TDD) [x]

- [x] Proof: `grep -n "Duration\|seconds\|timeout\|elapsed\|stopwatch" test/integration/vacuum_latency_test.dart` → vacuum_latency_test.dart has timing/duration assertions
- [x] Proof: `grep -n "expect\|lessThan\|within\|completes" test/integration/vacuum_latency_test.dart` → vacuum_latency_test.dart has real assertions about completion time
- [x] Proof (TDD 🟢 GREEN): `C:\Users\siriu\flutter\bin\flutter.bat test test/integration/vacuum_latency_test.dart` → TDD: test must fail first (no timeout = 30s hang), then pass after fix

### Step 4: Fix Bug 1: add 8s timeout to Overpass HTTP and decouple prefetch from vacuum() [x]

- [x] Proof: `grep -n "timeout" lib/data/biome_service.dart` → biome_service.dart has a .timeout() on the HTTP call
- [x] Proof: `grep -n "\.timeout(Duration" lib/data/biome_service.dart` → Timeout is between 5-19 seconds (not too short, not unbounded)
- [x] Proof: `grep -n "await.*vacuum\|await.*prefetch" lib/ui/map_screen.dart` → exit 1 — vacuum() and prefetch() no longer awaited together in position handler (decoupled)
- [x] Proof: `C:\Users\siriu\flutter\bin\flutter.bat analyze lib/` → No analysis errors after fix

### Step 5: Write RED test: stepFalling() returns true for airborne pixel (TDD) [x]

- [x] Proof: `grep -n "isTrue\|stepFalling.*true\|returns true" test/domain/pixel_physics_test.dart` → pixel_physics_test.dart has assertion that stepFalling returns true for active pixel
- [x] Proof: `grep -n "TubePixel\|TubePhysics" test/domain/pixel_physics_test.dart` → Test uses real TubePixel and TubePhysics types
- [x] Proof (TDD 🟢 GREEN): `C:\Users\siriu\flutter\bin\flutter.bat test test/domain/pixel_physics_test.dart` → TDD: test must fail first (stepFalling always returns false), then pass after fix

### Step 6: Fix Bug 2: stepFalling() returns true when falling, _stepSimulation() calls setState during active fall [x]

- [x] Proof: `grep -n "return true" lib/ui/pixel_physics.dart` → stepFalling() now has a return true path for active pixels
- [x] Proof: `grep -n "return false" lib/ui/pixel_physics.dart` → stepFalling() still returns false when pixel is settled
- [x] Proof: `grep -n "setState" lib/ui/map_screen.dart` → _stepSimulation calls setState when pixels are actively falling, not only on settle
- [x] Proof: `C:\Users\siriu\flutter\bin\flutter.bat analyze lib/` → No analysis errors after fix

### Step 7: Write journey integration test: Central London path forges ≥1 booster [x]

- [x] Proof: `grep -n "51\.5\|london\|London\|-0\.1" test/integration/journey_test.dart` → Journey test uses Central London coordinates
- [x] Proof: `grep -n "boosters\|booster" test/integration/journey_test.dart` → Journey test asserts on booster count
- [x] Proof: `grep -n "FakeGpsService\|gpsService\|emit\|inject" test/integration/journey_test.dart` → Journey test injects fake GPS stream
- [x] Proof: `grep -n "Tags\|integration" test/integration/journey_test.dart` → Journey test is tagged 'integration' so it can be excluded from fast CI
- [x] Proof: `C:\Users\siriu\flutter\bin\flutter.bat test test/integration/journey_test.dart` → Journey test passes end-to-end (requires network for real Overpass)

### Step 8: All non-integration tests pass with no regressions [x]

- [x] Proof: `C:\Users\siriu\flutter\bin\flutter.bat test --exclude-tags=integration` → Full unit + widget test suite passes (integration tests excluded — they need network)
- [x] Proof: `C:\Users\siriu\flutter\bin\flutter.bat analyze` → flutter analyze clean across the whole project

## Amendment log

- **2026-06-12T10:18:08.207Z** [step-1/proof-1-5] modified: flutter not on PATH for dod-guard process; using absolute path to flutter.bat
- **2026-06-12T10:18:09.771Z** [step-3/proof-3-3] modified: flutter not on PATH for dod-guard process; using absolute path to flutter.bat
- **2026-06-12T10:18:11.298Z** [step-4/proof-4-4] modified: flutter not on PATH for dod-guard process; using absolute path to flutter.bat
- **2026-06-12T10:18:12.842Z** [step-5/proof-5-3] modified: flutter not on PATH for dod-guard process; using absolute path to flutter.bat
- **2026-06-12T10:18:14.311Z** [step-6/proof-6-4] modified: flutter not on PATH for dod-guard process; using absolute path to flutter.bat
- **2026-06-12T10:18:15.808Z** [step-7/proof-7-5] modified: flutter not on PATH for dod-guard process; using absolute path to flutter.bat
- **2026-06-12T10:18:17.384Z** [step-8/proof-8-1] modified: flutter not on PATH for dod-guard process; using absolute path to flutter.bat
- **2026-06-12T10:18:19.094Z** [step-8/proof-8-2] modified: flutter not on PATH for dod-guard process; using absolute path to flutter.bat
