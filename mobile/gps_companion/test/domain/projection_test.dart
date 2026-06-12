import 'package:flutter_test/flutter_test.dart';
import 'package:gps_companion/domain/projection.dart';
import 'package:gps_companion/domain/route_log.dart';

void main() {
  group('projectIntermediate', () {
    test('two fixes 5s apart at 20 m/s produces ~5 intermediate points along bearing', () {
      // Arrange
      final last = GpsPoint(
        lat: 51.0,
        lon: 5.0,
        speed: 20.0,
        timestamp: DateTime.utc(2026, 6, 11, 12, 0, 0),
      );
      final current = GpsPoint(
        lat: 51.0,
        lon: 5.0,
        speed: 20.0,
        timestamp: DateTime.utc(2026, 6, 11, 12, 0, 5),
      );
      const heading = 90.0; // due east

      // Act
      final projected = projectIntermediate(
        last: last,
        current: current,
        headingDegrees: heading,
      );

      // Assert — ~4 backfilled intermediate points between the two fixes.
      // (at 1s intervals: 1s, 2s, 3s, 4s after last fix)
      final backfilled = projected.where((p) =>
        p.timestamp.isBefore(current.timestamp) &&
        p.timestamp.isAfter(last.timestamp)
      ).toList();
      expect(backfilled.length, greaterThanOrEqualTo(3));
      expect(backfilled.length, lessThanOrEqualTo(5));

      // All projected points are marked as such.
      for (final p in projected) {
        expect(p.isProjected, true);
      }

      // Backfilled points lie between the two fixes along the bearing.
      for (final p in backfilled) {
        // Due east: lon should increase, lat stays about the same.
        expect(p.lon, greaterThan(last.lon));
        expect(p.lat, closeTo(last.lat, 0.0001));
      }
    });

    test('speed below 1 m/s produces no projected points', () {
      final last = GpsPoint(
        lat: 51.0, lon: 5.0, speed: 0.5,
        timestamp: DateTime.utc(2026, 6, 11, 12, 0, 0),
      );
      final current = GpsPoint(
        lat: 51.0, lon: 5.0, speed: 0.5,
        timestamp: DateTime.utc(2026, 6, 11, 12, 0, 5),
      );
      final projected = projectIntermediate(
        last: last, current: current, headingDegrees: 90,
      );
      expect(projected, isEmpty);
    });

    test('fixes less than 1s apart produce no intermediate points', () {
      final last = GpsPoint(
        lat: 51.0, lon: 5.0, speed: 20.0,
        timestamp: DateTime.utc(2026, 6, 11, 12, 0, 0, 500000),
      );
      final current = GpsPoint(
        lat: 51.0, lon: 5.0, speed: 20.0,
        timestamp: DateTime.utc(2026, 6, 11, 12, 0, 1, 0),
      );
      final projected = projectIntermediate(
        last: last, current: current, headingDegrees: 90,
      );
      expect(projected, isEmpty);
    });

    test('northward bearing increases latitude', () {
      final last = GpsPoint(
        lat: 51.0, lon: 5.0, speed: 20.0,
        timestamp: DateTime.utc(2026, 6, 11, 12, 0, 0),
      );
      final current = GpsPoint(
        lat: 51.0, lon: 5.0, speed: 20.0,
        timestamp: DateTime.utc(2026, 6, 11, 12, 0, 5),
      );
      final projected = projectIntermediate(
        last: last, current: current, headingDegrees: 0, // due north
      );
      expect(projected, isNotEmpty);
      for (final p in projected) {
        expect(p.lat, greaterThan(last.lat));
      }
    });
  });
}
