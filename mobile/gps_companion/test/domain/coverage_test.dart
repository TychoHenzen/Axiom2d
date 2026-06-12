import 'package:flutter_test/flutter_test.dart';
import 'package:gps_companion/domain/biome.dart';
import 'package:gps_companion/domain/coverage.dart';
import 'package:gps_companion/domain/grain.dart';
import 'package:gps_companion/domain/route_log.dart';

void main() {
  // A fixed spot/time so every assertion is deterministic.
  const lat = 51.35;
  const lon = 5.46;
  const day = 20000;
  const week = 2900;

  group('metersBetween', () {
    test('one paint-cell of latitude matches the degree size', () {
      final d = metersBetween(lat, lon, lat + kPaintCellDegrees, lon);
      expect(d, closeTo(kPaintCellDegrees * 111320, 0.3));
    });
  });

  group('cellsWithinRadius', () {
    test('every returned cell centre is within the radius', () {
      final cells = cellsWithinRadius(lat, lon, kCollectRadiusMeters);
      for (final c in cells) {
        final d = metersBetween(
          lat,
          lon,
          paintCellCenterLat(c.iy),
          paintCellCenterLon(c.ix),
        );
        expect(d, lessThanOrEqualTo(kCollectRadiusMeters));
      }
    });

    test('a larger radius covers strictly more cells', () {
      final small = cellsWithinRadius(lat, lon, 10).length;
      final big = cellsWithinRadius(lat, lon, 30).length;
      expect(big, greaterThan(small));
    });

    test('the player position is always inside the brush', () {
      expect(cellsWithinRadius(lat, lon, kCollectRadiusMeters), isNotEmpty);
    });
  });

  group('CoverageMap.harvest', () {
    test('first sweep paints cells; re-sweeping the same spot yields nothing', () {
      // Arrange
      final cov = CoverageMap();

      // Act
      final first = cov.harvest(lat: lat, lon: lon, day: day, week: week);
      final second = cov.harvest(lat: lat, lon: lon, day: day, week: week);

      // Assert — no double-collection of already-painted ground.
      expect(first.newCells, isNotEmpty);
      expect(second.newCells, isEmpty);
      expect(second.grains, isEmpty);
    });

    test('covered set grows by exactly the freshly painted cell count', () {
      final cov = CoverageMap();
      final r = cov.harvest(lat: lat, lon: lon, day: day, week: week);
      expect(cov.cellCount, r.newCells.length);
    });

    test('moving to a far patch paints new ground', () {
      final cov = CoverageMap();
      cov.harvest(lat: lat, lon: lon, day: day, week: week);
      final before = cov.cellCount;
      // ~1 km east — disjoint from the first brush.
      final moved = cov.harvest(lat: lat, lon: lon + 0.015, day: day, week: week);
      expect(moved.newCells, isNotEmpty);
      expect(cov.cellCount, greaterThan(before));
    });

    test('grain yield is deterministic for the same spot, day and week', () {
      final a = CoverageMap().harvest(lat: lat, lon: lon, day: day, week: week);
      final b = CoverageMap().harvest(lat: lat, lon: lon, day: day, week: week);
      expect(b.grains.length, a.grains.length);
      if (a.grains.isNotEmpty) {
        expect(b.grains.first.type, a.grains.first.type);
      }
    });

    test('a tiny sweep accumulates fractional volume without minting a grain', () {
      // Arrange — a brush too small to gather a whole grain in one step.
      final cov = CoverageMap();

      // Act
      final r = cov.harvest(lat: lat, lon: lon, day: day, week: week, radiusM: 6);

      // Assert — no rounding to zero: progress is captured as a fraction.
      expect(r.newCells, isNotEmpty);
      expect(r.grains, isEmpty);
      expect(cov.volumeRemainder, greaterThan(0));
      expect(cov.volumeRemainder, lessThan(1.0));
    });

    test('after minting, the leftover volume is below one grain', () {
      final cov = CoverageMap();
      cov.harvest(lat: lat, lon: lon, day: day, week: week, radiusM: 80);
      expect(cov.volumeRemainder, lessThan(1.0));
    });

    test('grain signatures cluster by the biome walked through', () {
      // Grains harvested from one biome share a dominant signature axis (not
      // random) — the accumulated biome/leyline signature shapes each grain.
      final r = CoverageMap().harvest(
        lat: lat,
        lon: lon,
        day: day,
        week: week,
        radiusM: 80,
        biome: Biome.forestPark,
      );
      expect(r.grains, isNotEmpty);
      final counts = <GrainType, int>{};
      for (final g in r.grains) {
        counts[g.type] = (counts[g.type] ?? 0) + 1;
      }
      final top = counts.values.fold(0, (m, c) => c > m ? c : m);
      expect(top, greaterThan(r.grains.length / 2));
    });

    test('reset clears painted ground and accumulated signature', () {
      final cov = CoverageMap();
      cov.harvest(lat: lat, lon: lon, day: day, week: week, radiusM: 40);
      cov.reset();
      expect(cov.cellCount, 0);
      expect(cov.volumeRemainder, 0.0);
    });

    test('a denser biome yields at least as many grains as a sparse one', () {
      // Sum over a wide brush so the Poisson means are large enough to order.
      var dense = 0;
      var sparse = 0;
      for (var k = 0; k < 12; k++) {
        dense += CoverageMap()
            .harvest(
              lat: lat,
              lon: lon + k * 0.02,
              day: day,
              week: week,
              radiusM: 80,
              biome: Biome.forestPark, // 300/km²
            )
            .grains
            .length;
        sparse += CoverageMap()
            .harvest(
              lat: lat,
              lon: lon + k * 0.02,
              day: day,
              week: week,
              radiusM: 80,
              biome: Biome.historicCultural, // 80/km²
            )
            .grains
            .length;
      }
      expect(dense, greaterThan(sparse));
    });
  });

  group('areaM2', () {
    test('scales with the number of painted cells', () {
      final cov = CoverageMap();
      cov.harvest(lat: lat, lon: lon, day: day, week: week);
      expect(cov.areaM2(lat), closeTo(cov.cellCount * paintCellAreaM2(lat), 1e-6));
    });
  });

  group('replayFromLog', () {
    test('harvest path → replay log → identical covered cells + minted grains', () {
      // Arrange — harvest a path on the first CoverageMap.
      final original = CoverageMap();
      final path = [
        GpsPoint(lat: lat, lon: lon, speed: 5, timestamp: DateTime.now()),
        GpsPoint(lat: lat + 0.001, lon: lon, speed: 5, timestamp: DateTime.now()),
        GpsPoint(lat: lat + 0.002, lon: lon, speed: 5, timestamp: DateTime.now()),
        GpsPoint(lat: lat + 0.003, lon: lon, speed: 5, timestamp: DateTime.now()),
        GpsPoint(lat: lat + 0.004, lon: lon, speed: 5, timestamp: DateTime.now()),
      ];
      for (final p in path) {
        original.harvest(lat: p.lat, lon: p.lon, day: day, week: week);
      }
      final coveredBefore = Set<String>.from(original.covered);
      final volumeBefore = original.volumeRemainder;

      // Act — replay the same path on a fresh CoverageMap.
      final replayed = CoverageMap();
      replayed.replayFromLog(path, day: day, week: week);

      // Assert — covered cells match exactly.
      expect(replayed.covered, coveredBefore);
      // Volume remainder matches (same cells × same yields).
      expect(replayed.volumeRemainder, closeTo(volumeBefore, 1e-9));
    });

    test('replay on already-covered map adds no new cells', () {
      final cov = CoverageMap();
      final points = [
        GpsPoint(lat: lat, lon: lon, speed: 5, timestamp: DateTime.now()),
        GpsPoint(lat: lat + 0.001, lon: lon, speed: 5, timestamp: DateTime.now()),
      ];
      // Harvest directly first.
      for (final p in points) {
        cov.harvest(lat: p.lat, lon: p.lon, day: day, week: week);
      }
      final countBefore = cov.cellCount;
      // Replay same points — no new cells.
      final result = cov.replayFromLog(points, day: day, week: week);
      expect(result.newCells, isEmpty);
      expect(cov.cellCount, countBefore);
    });
  });
}
