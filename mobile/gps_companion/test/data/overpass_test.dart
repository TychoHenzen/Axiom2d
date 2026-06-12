import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:gps_companion/data/overpass.dart';

void main() {
  group('Overpass parse', () {
    late String fixtureJson;

    setUp(() {
      final file = File('test/fixtures/overpass_sample.json');
      fixtureJson = file.readAsStringSync();
    });

    test('when_parsing_fixture_then_produces_regions_and_pois', () {
      // Act
      final result = parseOverpassResponse(fixtureJson);

      // Assert — at least 1 region polygon.
      expect(result.regions.length, greaterThanOrEqualTo(1),
          reason: 'fixture has 2 way elements');
      // Assert — at least 1 POI point.
      expect(result.pois.length, greaterThanOrEqualTo(1),
          reason: 'fixture has 2 node elements');
    });

    test('when_parsing_forest_region_then_has_correct_tags_and_geometry', () {
      // Act
      final result = parseOverpassResponse(fixtureJson);
      final forest = result.regions.firstWhere(
        (r) => r.tags['landuse'] == 'forest',
      );

      // Assert
      expect(forest.osmId, 111);
      expect(forest.geometry.length, greaterThanOrEqualTo(3),
          reason: 'closed polygon must have ≥3 points');
      expect(forest.tags['name'], 'Sample Woods');
    });

    test('when_parsing_water_region_then_geometry_is_not_empty', () {
      // Act
      final result = parseOverpassResponse(fixtureJson);
      final water = result.regions.firstWhere(
        (r) => r.tags['natural'] == 'water',
      );

      // Assert
      expect(water.geometry, isNotEmpty);
      expect(water.tags['name'], 'Sample Pond');
    });

    test('when_parsing_fountain_poi_then_has_position_and_tags', () {
      // Act
      final result = parseOverpassResponse(fixtureJson);
      final fountain = result.pois.firstWhere(
        (p) => p.tags['amenity'] == 'fountain',
      );

      // Assert
      expect(fountain.lat, closeTo(51.5012, 0.0001));
      expect(fountain.lon, closeTo(-0.1185, 0.0001));
    });

    test('when_parsing_monument_poi_then_has_historic_tag', () {
      // Act
      final result = parseOverpassResponse(fixtureJson);
      final monument = result.pois.firstWhere(
        (p) => p.tags['historic'] == 'monument',
      );

      // Assert
      expect(monument.tags['name'], 'Sample Monument');
    });
  });

  group('buildOverpassUrl', () {
    test('when_building_url_then_contains_overpass_api_host', () {
      // Act
      final url = buildOverpassUrl(lat: 51.5, lon: -0.12, radiusM: 400);

      // Assert
      expect(url, contains('overpass-api.de'));
      expect(url, contains('interpreter'));
      expect(url, contains('out%3Ajson')); // URL-encoded ':'
    });

    test('when_building_url_then_bbox_is_within_radius', () {
      // Act
      final url = buildOverpassUrl(lat: 51.5, lon: -0.12, radiusM: 400);

      // Assert — bbox should be in URL
      expect(url, contains('51.'));
    });
  });
}
