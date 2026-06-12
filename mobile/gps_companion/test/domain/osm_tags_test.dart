import 'package:flutter_test/flutter_test.dart';
import 'package:gps_companion/domain/osm_tags.dart';

void main() {
  group('OSM tag coverage', () {
    test('when_counting_mappings_then_at_least_25_distinct_tags', () {
      // Arrange
      final keys = kOsmBiomeSamples.map((s) => s.key).toSet();

      // Act / Assert
      expect(keys.length, greaterThanOrEqualTo(25),
          reason: 'spec requires >= 25 distinct OSM tag mappings');
    });

    test('when_checking_tag_categories_then_all_required_prefixes_present', () {
      // Arrange
      final keys = kOsmBiomeSamples.map((s) => s.key).toList();

      // Assert — each required OSM category has at least one entry
      expect(keys.any((k) => k.startsWith('landuse=')), isTrue,
          reason: 'must have landuse/* mappings');
      expect(keys.any((k) => k.startsWith('natural=')), isTrue,
          reason: 'must have natural/* mappings');
      expect(keys.any((k) => k.startsWith('leisure=')), isTrue,
          reason: 'must have leisure/* mappings');
      expect(keys.any((k) => k.startsWith('waterway=')), isTrue,
          reason: 'must have waterway/* mappings');
      expect(keys.any((k) => k.startsWith('wetland=')), isTrue,
          reason: 'must have wetland/* mappings');
    });
  });

  group('Distribution validity', () {
    test('when_summing_each_dist_then_all_sum_to_one_within_tolerance', () {
      for (final sample in kOsmBiomeSamples) {
        // Arrange
        final sum = sample.dist.fold(0.0, (a, b) => a + b);

        // Assert
        expect(sum, closeTo(1.0, 0.001),
            reason: '${sample.key} dist sums to $sum, expected ~1.0');
      }
    });

    test('when_checking_dist_length_then_all_have_five_elements', () {
      for (final sample in kOsmBiomeSamples) {
        expect(sample.dist.length, 5,
            reason: '${sample.key} dist should have 5 elements');
      }
    });

    test('when_checking_densities_then_all_positive', () {
      for (final sample in kOsmBiomeSamples) {
        expect(sample.densityPerKm2, greaterThan(0),
            reason: '${sample.key} density must be positive');
      }
    });
  });

  group('osmTagToBiome lookup', () {
    test('when_querying_known_tag_then_returns_correct_sample', () {
      // Act
      final result = osmTagToBiome('landuse', 'forest');

      // Assert
      expect(result, isNotNull);
      expect(result!.key, 'landuse=forest');
      expect(result.densityPerKm2, 320);
    });

    test('when_querying_water_tag_then_returns_water_biased_dist', () {
      // Act
      final result = osmTagToBiome('natural', 'water');

      // Assert
      expect(result, isNotNull);
      // Water is index 2 in dist; should be the dominant element
      final waterWeight = result!.dist[2];
      expect(waterWeight, greaterThan(0.5),
          reason: 'natural=water should have water as dominant grain');
    });

    test('when_querying_unknown_tag_then_returns_null', () {
      // Act
      final result = osmTagToBiome('landuse', 'nonexistent_value');

      // Assert
      expect(result, isNull);
    });

    test('when_querying_unknown_key_then_returns_null', () {
      // Act
      final result = osmTagToBiome('unknown_key', 'forest');

      // Assert
      expect(result, isNull);
    });

    test('when_querying_each_known_tag_then_lookup_matches_list', () {
      // Every entry in the const list should be reachable via the lookup.
      for (final sample in kOsmBiomeSamples) {
        // Arrange — split "key=value"
        final parts = sample.key.split('=');

        // Act
        final found = osmTagToBiome(parts[0], parts[1]);

        // Assert
        expect(found, isNotNull, reason: '${sample.key} not found via lookup');
        expect(found!.key, sample.key);
      }
    });
  });
}
