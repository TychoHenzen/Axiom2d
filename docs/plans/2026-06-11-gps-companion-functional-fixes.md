# GPS Companion — Functional Fixes (Screen On, Background GPS, Coverage Persistence, Polling Rate) — Requirements Spec

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

**Goal:** Make GPS Companion usable for real driving: screen stays on (Map tab), GPS tracks continuously via foreground service (opt-in), coverage persists across app restarts via GPS datapoint log with weekly reset, and polling gaps are filled with dead reckoning projection.

**Date:** 2026-06-11
**Target:** `C:\Users\siriu\RustroverProjects\Axiom2d\mobile\gps_companion`
**DoD ID:** `ac73c1aa-b73e-4f81-bd6d-b9253eb0dbb0`
**Last check:** PASS (2026-06-11T13:07:31.097Z)

---

## Decisions (locked with user)

1. **Background GPS:** `flutter_background_service` (free/MIT) with `AndroidForegroundType.location`. NOT `flutter_background_geolocation` (commercial). Explicit opt-in toggle, auto-cancels when app re-opened.
2. **Polling:** moderate rate increase (1s interval) + dead reckoning projection between fixes. Both.
3. **Coverage persistence:** GPS datapoint log (lat, lon, speed, timestamp) + lastPackForgedAtIndex marker. Replay on app reopen to rebuild CoverageMap state.
4. **Coverage reset:** weekly (matching leyline cadence), NOT daily.
5. **Screen keep-on:** Map tab only via wakelock_plus. Release on tab switch / dispose.
6. **Log retention:** weekly pruning. Points before last-pack marker retained for route display, cleared on week rollover.

## Current state

Verified 2026-06-11 against code:
- `coverage.dart`: CoverageMap is in-memory only, day-scoped, resets at midnight UTC. `_mintCount` for mint seeds (accrual-order-dependent).
- `app_state.dart`: CoverageMap created fresh on load (`CoverageMap()`), `_coverageDay = -1`. No persistence of covered cells or signature accumulator.
- `map_screen.dart`: GPS via `geolocator.getPositionStream()` with `distanceFilter: 5`, no `intervalDuration`. No wakelock. No dead reckoning.
- `main.dart`: No background service configuration.
- `AndroidManifest.xml`: `ACCESS_FINE_LOCATION` + `ACCESS_COARSE_LOCATION` only. No `FOREGROUND_SERVICE` or `FOREGROUND_SERVICE_LOCATION` permissions.
- `pubspec.yaml`: `geolocator: ^14.0.2`, no `wakelock_plus`, no `flutter_background_service`.
- 31 Dart tests pass, flutter analyze clean.
- `store.dart`: persists `Inventory` (grains + boosters) only. No route/coverage persistence.

## Requirements

### 1. Keep screen on (Map tab only)
- Add `wakelock_plus` package dependency.
- Enable wakelock when MapScreen is active (initState / tab switch to Map).
- Disable wakelock when leaving Map tab or disposing MapScreen.
- Gallery and Transfer tabs use normal screen timeout.

### 2. Background GPS tracking (opt-in foreground service)
- Add `flutter_background_service` package dependency.
- Configure Android foreground service in `main.dart` with `AndroidForegroundType.location`.
- Add a toggle button on MapScreen: "Track in Background".
- When enabled: start foreground service with notification "GPS Companion is tracking your location".
- Background isolate: run `geolocator.getPositionStream()` and append points to shared datapoint log.
- When app re-opens (MapScreen becomes active while service running): auto-stop service, replay datapoints to rebuild coverage.
- Service stops if GPS permission revoked or location services disabled.

### 3. Coverage persistence via GPS datapoint log
- New file `lib/domain/route_log.dart`: `GpsPoint { double lat, lon, speed; DateTime timestamp }` + JSON codec.
- New key `route_log_v1` in `shared_preferences`: stores `{ points: [...], lastPackForgedAtIndex: int|null }`.
- `AppState` loads route log on init, replays points after `lastPackForgedAtIndex` through `CoverageMap.harvest()`.
- On each booster forge: update `lastPackForgedAtIndex` to current point index.
- On each GPS fix: append point to log, persist periodically (debounced, or on forge).
- Weekly prune: clear log when week number changes.

### 4. Weekly coverage reset
- Replace `_coverageDay` with `_coverageWeek` in AppState.
- Reset coverage + route log when `weekNumber` changes.
- Week number uses same `weekNumber()` function from `leyline.dart`.

### 5. Dead reckoning position projection
- In `map_screen.dart` / new domain file `lib/domain/projection.dart`:
  - When GPS fix arrives with `speed >= 1.0 m/s` AND `heading` (bearing) available:
    - Calculate time delta since last fix.
    - Project intermediate positions at ~1-second intervals along bearing.
    - Run each projected point through `vacuum()` harvest pipeline.
    - Store projected points in route log (flagged `isProjected: true`).
  - When next real fix arrives: stop projecting, use real position.
- Projected points count toward coverage but do NOT trigger duplicate cell harvests (CoverageMap already deduplicates).

### 6. Increase GPS polling rate
- In `map_screen.dart` `_startLocation()`: add `intervalDuration: const Duration(seconds: 1)` to `LocationSettings`.
- Keep `distanceFilter: 5` (meters).
- Background service uses same settings.

### 7. Android manifest changes
- Add `FOREGROUND_SERVICE` permission.
- Add `FOREGROUND_SERVICE_LOCATION` permission.
- Add `<service>` declaration for `flutter_background_service.BackgroundService` with `foregroundServiceType="location"`.

### 8. Privacy policy update
- Add bullet to `docs/plans/gps_companion_privacy.md`: background location mode uses foreground service with persistent notification; location data stays on-device, never transmitted.

### What it does NOT do:
- No full background tracking when app swiped away (foreground service only, killable by OEM).
- No route-line visualization on map (datapoint storage enables it, out of scope).
- No biome-based density coupling (separate DoD: biome_overlay).

## Research Notes

- `flutter_background_service` (MIT, free): configures foreground service in `main()` before `runApp()`. Requires `AndroidForegroundType.location` on Android 14+. Uses separate isolate for background Dart code. Communication via `service.invoke()` / `service.on()`. Must declare `<service>` in manifest + `FOREGROUND_SERVICE` + `FOREGROUND_SERVICE_LOCATION` permissions.
- `wakelock_plus` (BSD-3, free): `WakelockPlus.enable()` / `WakelockPlus.disable()`. Simple enable/disable. Works on Android via `FLAG_KEEP_SCREEN_ON`.
- Android location rate: `intervalDuration` in `LocationSettings` maps to Android's `LocationRequest.setInterval()`. 1000ms is default for high-accuracy; reducing below doesn't guarantee faster fixes but prevents throttling.
- Dead reckoning formula: `newLat = lat + (speed * dt * cos(bearing)) / 111320`, `newLon = lon + (speed * dt * sin(bearing)) / (111320 * cos(lat))`. Speed in m/s, bearing in radians, dt in seconds.
- `shared_preferences` size limit: ~1-2MB on Android depending on device. 10K GPS points ≈ ~500KB JSON. Weekly pruning keeps well under limit.
- Files to create: `lib/domain/route_log.dart`, `lib/domain/projection.dart`.
- Files to edit: `pubspec.yaml`, `lib/main.dart`, `lib/ui/map_screen.dart`, `lib/ui/app_state.dart`, `lib/domain/coverage.dart`, `lib/data/store.dart`, `android/app/src/main/AndroidManifest.xml`, `docs/plans/gps_companion_privacy.md`.

## Open Questions

- Desktop import plan (grain_unpack.rs) — separate plan, not started.
- Route-line map visualization — future feature, not scoped here.

---

## Definition of Done

### Step 1: Add wakelock_plus and flutter_background_service dependencies [x]

- [x] Proof: `rtk grep "wakelock_plus:" pubspec.yaml` → wakelock_plus present in pubspec.yaml dependencies
- [x] Proof: `rtk grep "flutter_background_service:" pubspec.yaml` → flutter_background_service present in pubspec.yaml dependencies
- [x] Proof: `C:\Users\siriu\flutter\bin\flutter pub get` → flutter pub get succeeds with new deps

### Step 2: Keep screen on — Map tab wakelock [x]

- [x] Proof: `rtk grep "WakelockPlus" lib/ui/map_screen.dart` → WakelockPlus.enable() called in MapScreen initState or tab activation
- [x] Proof: `rtk grep "WakelockPlus" lib/ui/map_screen.dart` → WakelockPlus.disable() called in MapScreen dispose or tab deactivation
- [x] Proof: `rtk grep "WakelockPlus" lib/ui/gallery_screen.dart lib/ui/transfer_screen.dart` → Wakelock NOT used in Gallery or Transfer screens

### Step 3: GPS datapoint log model + persistence [x]

- [x] Proof: `rtk grep "class GpsPoint" lib/domain/route_log.dart` → GpsPoint class exists with lat, lon, speed, timestamp fields
- [x] Proof: `rtk grep "lastPackForgedAtIndex" lib/domain/route_log.dart` → lastPackForgedAtIndex field in route log model
- [x] Proof: `C:\Users\siriu\flutter\bin\flutter test test/domain/route_log_test.dart` → Route log JSON roundtrip test passes

### Step 4: Coverage replay from datapoint log + weekly reset [x]

- [x] Proof: `rtk grep "_coverageWeek" lib/ui/app_state.dart` → AppState uses _coverageWeek (not _coverageDay)
- [x] Proof: `rtk grep "weekNumber" lib/ui/app_state.dart` → weekNumber check for coverage reset
- [x] Proof: `C:\Users\siriu\flutter\bin\flutter test test/domain/coverage_test.dart` → Coverage replay test: harvest path → serialize log → fresh CoverageMap → replay log → identical covered cells + minted grains
- [x] Proof: `rtk grep "replayRouteLog\|replayFromLog" lib/domain/coverage.dart` → CoverageMap has replay/restore method from GpsPoint list

### Step 5: Dead reckoning position projection [x]

- [x] Proof: `rtk grep "class Projection\|projectIntermediate" lib/domain/projection.dart` → Projection logic exists in domain/projection.dart
- [x] Proof: `C:\Users\siriu\flutter\bin\flutter test test/domain/projection_test.dart` → Projection test: given two GPS fixes 5s apart at 20 m/s, produces ~5 intermediate points along bearing
- [x] Proof: `rtk grep "isProjected" lib/domain/route_log.dart` → GpsPoint has isProjected flag
- [x] Proof: `rtk grep "projectIntermediate\|_projectPositions" lib/ui/map_screen.dart` → MapScreen calls projection logic on GPS fixes

### Step 6: Increase GPS polling rate [x]

- [x] Proof: `rtk grep "intervalDuration" lib/ui/map_screen.dart` → LocationSettings includes intervalDuration
- [x] Proof: `rtk grep "Duration.*seconds.*1" lib/ui/map_screen.dart` → intervalDuration set to 1 second

### Step 7: Background GPS service with opt-in toggle [x]

- [x] Proof: `rtk grep "FlutterBackgroundService" lib/main.dart` → Background service configured in main.dart
- [x] Proof: `rtk grep "AndroidForegroundType" lib/main.dart` → AndroidForegroundType.location declared
- [x] Proof: `rtk grep "Track in Background\|backgroundToggle\|_backgroundEnabled" lib/ui/map_screen.dart` → MapScreen has background tracking toggle button or control
- [x] Proof: `C:\Users\siriu\flutter\bin\flutter test test/data/transfer_store_test.dart` → Existing transfer/store tests still pass (no regression)

### Step 8: Android manifest — foreground service permissions + declaration [x]

- [x] Proof: `rtk grep "FOREGROUND_SERVICE" android/app/src/main/AndroidManifest.xml` → FOREGROUND_SERVICE permission declared
- [x] Proof: `rtk grep "FOREGROUND_SERVICE_LOCATION" android/app/src/main/AndroidManifest.xml` → FOREGROUND_SERVICE_LOCATION permission declared
- [x] Proof: `rtk grep "flutter_background_service.BackgroundService" android/app/src/main/AndroidManifest.xml` → BackgroundService declared in manifest with foregroundServiceType=location

### Step 9: Privacy policy update for background location [x]

- [x] Proof: `rtk grep -i "background\|foreground service" ../../docs/plans/gps_companion_privacy.md` → Privacy policy mentions background/foreground service location usage

### Step 10: Full build + full test suite passes [x]

- [x] Proof: `C:\Users\siriu\flutter\bin\flutter analyze` → flutter analyze — no errors or warnings
- [x] Proof: `C:\Users\siriu\flutter\bin\flutter test` → flutter test — full suite passes (existing 31 + new tests)
- [x] Proof: `C:\Users\siriu\flutter\bin\flutter build apk --debug` → Debug APK builds successfully

## Open risks

- Aggressive OEM battery optimization (Samsung, Xiaomi, Huawei) may kill foreground service under memory pressure. Mitigation: notification helps, but not guaranteed. Datapoint log survives — coverage replays on next app open.
- `shared_preferences` load on app start may be slow if log grows large. Mitigation: weekly pruning keeps log bounded to ~1 week of driving data.
- Dead reckoning accuracy degrades with turns. Mitigation: only project ≤5 seconds ahead; real fix resets position.

## Amendment log

- **2026-06-11T13:06:00.665Z** [step-1/proof-1-3] modified: flutter not on MCP PATH; use absolute path to flutter.bat
- **2026-06-11T13:06:02.232Z** [step-3/proof-3-3] modified: flutter not on MCP PATH; use absolute path
- **2026-06-11T13:06:03.747Z** [step-4/proof-4-3] modified: flutter not on MCP PATH; use absolute path
- **2026-06-11T13:06:05.244Z** [step-5/proof-5-2] modified: flutter not on MCP PATH; use absolute path
- **2026-06-11T13:06:06.754Z** [step-7/proof-7-4] modified: flutter not on MCP PATH; use absolute path
- **2026-06-11T13:06:08.578Z** [step-9/proof-9-1] modified: Relative path must account for cwd being mobile/gps_companion
- **2026-06-11T13:06:10.053Z** [step-10/proof-10-1] modified: flutter not on MCP PATH; use absolute path
- **2026-06-11T13:06:11.513Z** [step-10/proof-10-2] modified: flutter not on MCP PATH; use absolute path
- **2026-06-11T13:06:12.849Z** [step-10/proof-10-3] modified: flutter not on MCP PATH; use absolute path
