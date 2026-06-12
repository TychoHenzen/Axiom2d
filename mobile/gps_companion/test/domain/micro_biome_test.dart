import 'package:flutter_test/flutter_test.dart';
import 'package:gps_companion/domain/micro_biome.dart';

void main() {
  // A reference POI at an arbitrary location.
  const LatLon poi = (lat: 51.5074, lon: -0.1278);

  group('microBiomeAt radius behaviour', () {
    test('when_query_at_poi_center_then_returns_non_null_with_full_density', () {
      // Act
      final result = microBiomeAt(poi, poi, MicroBiomeCategory.waterFeature);

      // Assert
      expect(result, isNotNull);
      expect(result!.densityPerKm2, MicroBiomeCategory.waterFeature.densityBump,
          reason: 'at center, falloff should be 1.0 => full density bump');
    });

    test('when_query_far_outside_radius_then_returns_null', () {
      // Arrange — ~100 m away (well beyond any 5-10 m radius)
      const LatLon far = (lat: 51.5084, lon: -0.1278);

      // Act
      final result = microBiomeAt(poi, far, MicroBiomeCategory.monument);

      // Assert
      expect(result, isNull);
    });

    test('when_query_just_inside_radius_then_returns_non_null', () {
      // Arrange — nudge by ~3 m (within the 8 m waterFeature radius).
      // 1 degree lat ~ 111320 m, so 3 m ~ 0.0000269 degrees.
      const LatLon near = (lat: 51.5074 + 0.0000269, lon: -0.1278);

      // Act
      final result = microBiomeAt(poi, near, MicroBiomeCategory.waterFeature);

      // Assert
      expect(result, isNotNull);
      expect(result!.densityPerKm2, greaterThan(0));
      expect(result.densityPerKm2,
          lessThan(MicroBiomeCategory.waterFeature.densityBump),
          reason: 'not at center, so density < full bump');
    });
  });

  group('microBiomeAt grain distribution', () {
    test('when_water_feature_poi_then_dist_has_water_bias', () {
      // Act
      final result = microBiomeAt(poi, poi, MicroBiomeCategory.waterFeature);

      // Assert
      expect(result, isNotNull);
      // Water is index 2
      expect(result!.dist[2], greaterThan(0.5),
          reason: 'waterFeature should bias toward water grains');
    });

    test('when_monument_poi_then_dist_has_arcane_bias', () {
      // Act
      final result = microBiomeAt(poi, poi, MicroBiomeCategory.monument);

      // Assert
      expect(result, isNotNull);
      // Arcane is index 4
      expect(result!.dist[4], greaterThan(0.5),
          reason: 'monument should bias toward arcane grains');
    });

    test('when_any_category_then_dist_sums_to_one', () {
      for (final cat in MicroBiomeCategory.values) {
        // Act
        final result = microBiomeAt(poi, poi, cat);

        // Assert
        expect(result, isNotNull);
        final sum = result!.dist.fold(0.0, (a, b) => a + b);
        expect(sum, closeTo(1.0, 0.001),
            reason: '${cat.name} dist sums to $sum, expected ~1.0');
      }
    });
  });

  group('microBiomeAt falloff monotonicity', () {
    test('when_moving_away_from_poi_then_density_decreases_monotonically', () {
      // Arrange — sample at increasing distances within the radius.
      // greenery radius = 7 m. Sample at 0, 1, 2, 3, 4, 5, 6 metres.
      const category = MicroBiomeCategory.greenery;
      final densities = <double>[];

      for (var metres = 0.0; metres < category.radiusM; metres += 1.0) {
        // 1 m in latitude ~ 1 / 111320 degrees
        final offset = metres / 111320.0;
        final query = (lat: poi.lat + offset, lon: poi.lon);
        final result = microBiomeAt(poi, query, category);
        expect(result, isNotNull, reason: 'should be within radius at $metres m');
        densities.add(result!.densityPerKm2);
      }

      // Assert — each density is >= the next (strictly monotonic decreasing)
      for (var i = 0; i < densities.length - 1; i++) {
        expect(densities[i], greaterThanOrEqualTo(densities[i + 1]),
            reason:
                'density at step $i (${densities[i]}) should be >= step ${i + 1} (${densities[i + 1]})');
      }
      // First should be strictly greater than last
      expect(densities.first, greaterThan(densities.last),
          reason: 'center density must be strictly greater than edge density');
    });
  });

  group('MicroBiomeCategory constraints', () {
    test('when_checking_radii_then_all_between_5_and_10_metres', () {
      for (final cat in MicroBiomeCategory.values) {
        expect(cat.radiusM, inInclusiveRange(5.0, 10.0),
            reason: '${cat.name} radius must be 5-10 m');
      }
    });
  });
}
