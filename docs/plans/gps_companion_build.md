# GPS Companion App — Build & Distribution Plan

Companion to `gps_companion_app.md` (feature spec). This doc tracks the **build path to an
Android APK for Google Play**.

## Decisions (locked 2026-06-06)

- **Stack: Flutter** (plan's stated production choice for app-store distribution). True
  offline native APK, matches the spec's "no backend / fully offline" stance.
- **Success target: a signed *release* APK (and AAB) that builds locally and is
  Play-submission-ready** — correct manifest/permissions, release signing config,
  privacy-policy stub, data-safety notes.
- **Out of scope for this agent (human/external gates):** paid Google Play Developer
  account, identity verification, hosting the privacy policy, the data-safety form, and
  Google's review queue. The owner does these.
- **Platform: Android only.** No iOS, web, desktop targets in the Flutter project.
- **Location:** `mobile/gps_companion/` (Flutter project), separate from the Rust workspace.

## Toolchain status (probed 2026-06-06)

| Tool | Status |
|------|--------|
| Java | ✅ 21.0.10 LTS |
| Node | ✅ v22.15.0 |
| Android SDK / cmdline-tools | ❌ absent |
| NDK | ❌ absent (Flutter doesn't need it unless we add native Rust later) |
| Flutter SDK | ❌ absent |
| Gradle | ❌ absent (Flutter ships a Gradle wrapper per-project) |
| adb | ❌ absent (comes with platform-tools) |

## Status (2026-06-06)

**Signed release APK + AAB building from the real app; 31 Dart tests pass; analyze clean.**
- P0 toolchain ✅  P1 scaffold ✅  P2 core app ✅ (domain+data+UI)  P3 release readiness ✅
- targetSdk 36 / minSdk 24 (≥ Play's targetSdk floor of 35). apksigner verify on
  `app-release.apk` exits 0 with release key `CN=GPS Companion`.
- **Runtime not yet verified on-device.** `flutter devices` shows only Windows desktop +
  Edge web — no Android emulator/phone was available, so the app has never been launched.
  The build chain and signing are proven; first-launch behaviour (GPS stream, QR scan,
  flutter_map tiles, async `main()` I/O) is unverified. To verify: create an Android AVD
  (`flutter emulators --create` + system image) or plug in a phone, then `flutter run`.
- Remaining polish (not blocking submission): launcher icon art, offline map tiles, R8
  minify with tested keep-rules, biome point-in-polygon from embedded GeoJSON.

### Continuous field collection (brush model) — added 2026-06-06
Replaces discrete tap-to-collect grains with a "vacuum" brush: the world is a fine
~5.5 m paint grid (`domain/coverage.dart`, `kPaintCellDegrees`), and the collection
radius harvests each freshly-covered cell exactly once. Covered cells = the painted
overlay (small amber squares). Basemap swapped to CARTO `voyager_nolabels` (colored
landuse, no house numbers/POIs). Camera follows player; pan (not zoom) breaks follow;
recenter FAB re-enables.

Two accumulators (no per-cell rounding — fixes "fraction of a grain rounds to 0
forever"):
- **Volume** = Σ(cell area × biome density × `kYieldMultiplier`). Controls *how many*
  grains. A whole grain mints each time volume crosses 1.0.
- **Signature** (8-axis vector) = volume distributed across axes by biome/leyline
  weights (`spawn.axisWeights`). Controls *what each grain is*: a minted grain's
  signature is one grain's worth carved off the accumulated vector
  (`spawn.mintGrainFromSignature`). Walk through forest → nature-heavy grains; through
  a leyline → leyline-axis grains. Concentrated (pure) signatures mint rarer grains.
- HUD shows `% to next pack` (`AppState.packProgress` = loose grains + sub-grain
  volume remainder), ticking continuously as you walk.
- `kYieldMultiplier = 20` ≈ one booster per ~1 km walked at base density. Pure
  balance knob — raise to forge faster.

Caveats / follow-ups:
- Coverage is **in-memory** and **day-scoped** (resets at midnight UTC, matching
  day-based grain seeds). Closing the app mid-day loses the painted overlay (grains
  already forged persist). Persist per-day coverage if this matters.
- Biome is hard-coded `noData` (density 120) — no point-in-polygon biome lookup yet.
- Paint overlay render is capped at 2500 cells (`_maxPaintedRendered`) for frame budget;
  older painted cells stop rendering beyond that (still counted as collected).

## Build phases

### P0 — Toolchain (blocker)
1. Install Flutter SDK (stable) outside the repo.
2. Install Android cmdline-tools → `sdkmanager` → platform-tools, build-tools, platform
   android-34, accept licenses.
3. `flutter config --android-sdk <path>`; `flutter doctor` green for Android toolchain.

### P1 — Project scaffold
4. `flutter create --platforms=android --org com.<owner> mobile/gps_companion`.
5. Strip non-Android platform folders. Set applicationId, minSdk, app name.
6. Confirm `flutter build apk --debug` produces a runnable APK.

### P2 — Core app (feature spec drives this)
7. State/persistence: grains + boosters (shared_preferences/Hive), auto-forge at 100.
8. Map screen: GPS (geolocator), map (flutter_map), grain nodes, counters.
9. Spawn algorithm: deterministic S2-cell grains, biome lookup, leyline simplex overlay.
10. Booster gallery screen with wrapper theming.
11. Transfer screen: QR scanner (mobile_scanner) + WebSocket send.

### P3 — Release readiness
12. App icon, splash, version.
13. Release signing: generate keystore, `key.properties`, gradle signingConfig.
14. Permissions (location, camera, internet for LAN) + rationale strings.
15. Privacy-policy stub + data-safety notes (GPS/location, no network egress).
16. `flutter build apk --release` and `flutter build appbundle --release` succeed; verify
    signed with `apksigner`.

## Flutter module + package map

Packages (pubspec):
- `geolocator` — GPS position stream (foreground only)
- `flutter_map` + `latlong2` — OSM raster/vector map, offline-capable tiles
- `mobile_scanner` — QR scan on Transfer screen
- `web_socket_channel` — LAN transfer to desktop
- `hive` + `hive_flutter` (or `shared_preferences`) — local persistence of grains/boosters
- `fast_noise` (or hand-rolled simplex) — leyline overlay noise
- dev: `flutter_test`, `build_runner` (if Hive adapters)

`lib/` layout (pure-Dart logic separated from widgets for unit testing):
- `domain/grain.dart` — Grain, GrainType, GrainRarity + JSON
- `domain/rarity.dart` — geometric_level rarity (mirrors CardSignature::rarity_with_config)
- `domain/biome.dart` — biome → grain-type distribution table + point-in-polygon
- `domain/spawn.dart` — deterministic S2-cell spawn (seeded hash, poisson, jitter)
- `domain/leyline.dart` — weekly simplex overlay + theme
- `domain/inventory.dart` — collect, auto-forge at 100, booster theming
- `data/store.dart` — persistence
- `data/transfer.dart` — QR payload + WebSocket send (payload format from spec)
- `ui/map_screen.dart`, `ui/gallery_screen.dart`, `ui/transfer_screen.dart`, `ui/app.dart`
- `main.dart`

`test/` mirrors `domain/` — rarity math, spawn determinism, auto-forge spillover,
biome distribution, transfer payload roundtrip. Logic is the TDD target; widgets smoke-tested.

## Notes / lessons
- git-for-Windows prints MSYS paths (`/home/siriu/...`) in clone progress but writes to the
  real Windows path. Not a WSL routing bug — verify the Windows path directly.
- [LESSON] The `Write` tool in this environment appends a stray literal `</content>` line to
  every file. After each Write, immediately Edit to strip the trailing `</content>` (or it
  breaks compilation). Discovered when Dart files failed to parse at EOF.
- [LESSON] `sdkmanager.bat` license/accept prompts can't be fed via PowerShell pipe; use
  `Start-Process -RedirectStandardInput <file-of-y's>`. Also `cmd` on PATH may resolve to a
  msys2 stub — use `Start-Process` directly, not `cmd /c`.
- Flutter 3.44 stable requires Android **SDK 36** (platforms;android-36); SDK 35 alone fails
  `flutter doctor`. build-tools 36.0.0 installed too.
- Toolchain installed: Flutter `C:\Users\siriu\flutter`, Android SDK `C:\Users\siriu\Android\Sdk`.
- [LESSON] `flutter install` does NOT recompile — it deploys the existing
  `build/app/outputs/flutter-apk/app-release.apk`. After editing Dart you MUST
  `flutter build apk --release` (or `flutter run`) first, then install. Symptom of
  skipping it: device keeps running stale code while analyze/build look clean. Confirm a
  real rebuild via the APK size delta + the `√ Built` line.
- [LESSON] Device `R9WT50BV6TV` (Samsung SM-A226B) — connect over USB, `adb` at
  `C:\Users\siriu\Android\Sdk\platform-tools\adb.exe`. Launch headless via
  `adb -s <id> shell monkey -p com.tychohenzen.gps_companion -c android.intent.category.LAUNCHER 1`.
- [LESSON] `flutter run --debug` may fail with "Gradle build daemon disappeared" (JVM
  crash, `hs_err_pid*.log`) under memory pressure (daemon `-Xmx8G`). Release `build apk`
  succeeds; close RustRover to free RAM if the debug daemon keeps crashing.
- [LESSON] Android 9+ blocks cleartext (`http://`/`ws://`) by default. The LAN booster
  transfer uses `ws://<lan-ip>:<port>`, which silently fails in a release build unless a
  network-security-config allows it. Fixed via
  `android/app/src/main/res/xml/network_security_config.xml`
  (`<base-config cleartextTrafficPermitted="true"/>`) referenced from the manifest
  `application` tag. HTTPS map tiles are unaffected. Confirmed present in the merged
  release manifest.
</content>
</invoke>
