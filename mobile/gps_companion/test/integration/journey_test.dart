/// Journey integration test: simulates a GPS walk through Central London,
/// uses real Overpass API for biome data, and asserts that at least one
/// booster pack is forged after covering enough ground.
///
/// Uses FakeGpsService to inject GPS fixes without hardware.
/// Excludes from fast CI runs via the 'integration' tag.
@Tags(['integration'])
library;

import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:geolocator/geolocator.dart';
import 'package:gps_companion/data/biome_service.dart';
import 'package:gps_companion/data/store.dart';
import 'package:gps_companion/domain/biome.dart';
import 'package:gps_companion/domain/biome_def.dart';
import 'package:gps_companion/domain/inventory.dart';
import 'package:gps_companion/domain/route_log.dart';
import 'package:gps_companion/domain/leyline.dart' show weekNumber;
import 'package:gps_companion/domain/spawn.dart' show dayNumber;
import 'package:gps_companion/ui/app_state.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../helpers/fake_gps_service.dart';

/// Map a BiomeSample dominant index to the Biome enum.
Biome _sampleToBiome(BiomeSample sample) {
  final dist = sample.dist;
  var maxIdx = 0;
  for (var i = 1; i < dist.length; i++) {
    if (dist[i] > dist[maxIdx]) maxIdx = i;
  }
  return switch (maxIdx) {
    0 => Biome.forestPark,
    1 => Biome.urbanResidential,
    2 => Biome.waterCoast,
    3 => Biome.mountainDesert,
    4 => Biome.historicCultural,
    _ => Biome.noData,
  };
}

/// Build a minimal [Position] for testing — only lat/lon/speed matter.
Position _makePosition(double lat, double lon) => Position(
      latitude: lat,
      longitude: lon,
      timestamp: DateTime.now().toUtc(),
      accuracy: 1.0,
      altitude: 0.0,
      heading: 0.0,
      speed: 1.5,
      speedAccuracy: 0.0,
      altitudeAccuracy: 0.0,
      headingAccuracy: 0.0,
    );

void main() {
  setUpAll(() {
    TestWidgetsFlutterBinding.ensureInitialized();
  });

  setUp(() {
    SharedPreferences.setMockInitialValues({});
  });

  test(
    'central_london_journey_forges_at_least_one_booster',
    () async {
      // ── Arrange ──────────────────────────────────────────────────────────────
      // AppState backed by in-memory SharedPreferences mock.
      final state = AppState(
        store: InventoryStore(),
        inventory: Inventory(),
        routeLogStore: RouteLogStore(),
        routeLog: RouteLog(week: weekNumber(DateTime.now().toUtc())),
      );

      // BiomeService with real Overpass API for Central London.
      final biomeService = BiomeService();

      // FakeGpsService injects GPS fixes programmatically.
      final gpsService = FakeGpsService();

      // Central London grid: 51.51°N, -0.12°W — dense OSM coverage.
      // 15 × 15 = 225 points at ~33 m spacing covers ~0.0042° × 0.0042°.
      // Expected yield: >200 grains → at least 2 booster packs.
      const double baseLat = 51.51;
      const double baseLon = -0.12;
      const double step = 0.0003; // ~33 m latitude, ~20 m longitude

      // Prefetch the relevant Overpass tiles (1–4 tile queries for this area).
      // Allow up to 30 s for real network response.
      final tileCompleter = Completer<void>();
      biomeService.onTileLoaded = () {
        if (!tileCompleter.isCompleted) tileCompleter.complete();
      };
      unawaited(biomeService.prefetch(baseLat, baseLon));
      await tileCompleter.future
          .timeout(const Duration(seconds: 30), onTimeout: () {});

      // ── Act ───────────────────────────────────────────────────────────────────
      final now = DateTime.now().toUtc();
      final day = dayNumber(now);
      final week = weekNumber(now);

      // Collect vacuum futures so we can await them all after the journey.
      final vacuumFutures = <Future<dynamic>>[];

      // Listen to the FakeGpsService stream and call vacuum for each fix.
      final sub = gpsService.positionStream.listen((pos) {
        final sample = biomeService.sampleAt(pos.latitude, pos.longitude);
        final biome = sample != null ? _sampleToBiome(sample) : Biome.noData;
        vacuumFutures.add(
          state.vacuum(
            lat: pos.latitude,
            lon: pos.longitude,
            day: day,
            week: week,
            biome: biome,
          ),
        );
      });

      // Emit the Central London grid path through FakeGpsService.
      for (var i = 0; i < 15; i++) {
        for (var j = 0; j < 15; j++) {
          final lat = baseLat + i * step;
          final lon = baseLon + j * step;
          gpsService.emit(_makePosition(lat, lon));
        }
      }

      await gpsService.close();
      await sub.cancel();

      // Wait for all vacuum calls to complete.
      for (final f in vacuumFutures) {
        await f;
      }

      // ── Assert ────────────────────────────────────────────────────────────────
      expect(
        state.inventory.boosters.length,
        greaterThanOrEqualTo(1),
        reason: 'A 225-point Central London journey should forge ≥1 booster',
      );
    },
    timeout: const Timeout(Duration(minutes: 2)),
  );
}
