import 'package:flutter_test/flutter_test.dart';
import 'package:gps_companion/data/biome_service.dart';
import 'package:gps_companion/data/overpass.dart';
import 'package:gps_companion/domain/micro_biome.dart';

void main() {
  group('BiomeService sampleAt', () {
    late BiomeService service;

    setUp(() {
      service = BiomeService();
    });

    test('when_cache_miss_then_returns_null', () {
      // Act — sample at a location with no seeded data.
      final result = service.sampleAt(51.5, -0.12);

      // Assert
      expect(result, isNull,
          reason: 'no data seeded → null, not a noData sample');
    });

    test('when_cache_hit_in_region_then_returns_biome_sample', () {
      // Arrange — seed a forest polygon.
      service.seedTile(
        51.5,
        -0.12,
        regions: [
          (
            geometry: [
              OverpassPoint(lat: 51.49, lon: -0.13),
              OverpassPoint(lat: 51.51, lon: -0.13),
              OverpassPoint(lat: 51.51, lon: -0.11),
              OverpassPoint(lat: 51.49, lon: -0.11),
            ],
            osmKey: 'landuse',
            osmValue: 'forest',
          ),
        ],
      );

      // Act — sample inside the polygon.
      final result = service.sampleAt(51.50, -0.12);

      // Assert
      expect(result, isNotNull);
      expect(result!.key, 'landuse=forest');
      expect(result.densityPerKm2, 320);
      expect(result.dist.length, 5);
    });

    test('when_poi_override_then_beats_polygon', () {
      // Arrange — seed both a polygon and a closer POI.
      // Use coordinates that stay within the same coarse tile (~-0.125, 51.5).
      service.seedTile(
        51.5,
        -0.125,
        regions: [
          (
            geometry: [
              OverpassPoint(lat: 51.49, lon: -0.135),
              OverpassPoint(lat: 51.51, lon: -0.135),
              OverpassPoint(lat: 51.51, lon: -0.115),
              OverpassPoint(lat: 51.49, lon: -0.115),
            ],
            osmKey: 'landuse',
            osmValue: 'forest',
          ),
        ],
        pois: [
          (
            lat: 51.5001,
            lon: -0.1251,
            category: MicroBiomeCategory.waterFeature,
          ),
        ],
      );

      // Act — sample near the POI (within its radius, same tile).
      final result = service.sampleAt(51.5001, -0.1251);

      // Assert — POI override wins, returns water-feature sample.
      expect(result, isNotNull);
      expect(result!.key, 'poi=waterFeature');
      // Water feature dominant grain is water (index 2).
      expect(result.dist[2], greaterThan(0.5));
    });

    test('when_outside_polygon_and_no_poi_then_returns_null', () {
      // Arrange — seed a polygon that does not cover the query point.
      service.seedTile(
        51.5,
        -0.12,
        regions: [
          (
            geometry: [
              OverpassPoint(lat: 51.49, lon: -0.13),
              OverpassPoint(lat: 51.51, lon: -0.13),
              OverpassPoint(lat: 51.51, lon: -0.11),
              OverpassPoint(lat: 51.49, lon: -0.11),
            ],
            osmKey: 'landuse',
            osmValue: 'forest',
          ),
        ],
      );

      // Act — sample far outside the polygon.
      final result = service.sampleAt(52.0, 0.0);

      // Assert
      expect(result, isNull);
    });

    test('when_different_tile_then_returns_null', () {
      // Arrange — seed tile A.
      service.seedTile(
        51.5,
        -0.12,
        regions: [
          (
            geometry: [
              OverpassPoint(lat: 51.49, lon: -0.13),
              OverpassPoint(lat: 51.51, lon: -0.13),
              OverpassPoint(lat: 51.51, lon: -0.11),
              OverpassPoint(lat: 51.49, lon: -0.11),
            ],
            osmKey: 'landuse',
            osmValue: 'forest',
          ),
        ],
      );

      // Act — sample in a different tile (far away).
      final result = service.sampleAt(40.0, -74.0);

      // Assert — different tile, no cache.
      expect(result, isNull);
    });
  });
}
