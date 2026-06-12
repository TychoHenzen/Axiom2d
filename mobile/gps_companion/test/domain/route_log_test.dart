import 'package:flutter_test/flutter_test.dart';
import 'package:gps_companion/domain/route_log.dart';

void main() {
  group('GpsPoint JSON roundtrip', () {
    test('real fix survives encode → decode', () {
      // Arrange
      final original = GpsPoint(
        lat: 51.5074,
        lon: -0.1278,
        speed: 12.5,
        timestamp: DateTime.utc(2026, 6, 11, 14, 30, 0),
        isProjected: false,
      );

      // Act
      final json = original.toJson();
      final restored = GpsPoint.fromJson(json);

      // Assert
      expect(restored.lat, original.lat);
      expect(restored.lon, original.lon);
      expect(restored.speed, original.speed);
      expect(restored.timestamp, original.timestamp);
      expect(restored.isProjected, original.isProjected);
    });

    test('projected point survives encode → decode', () {
      // Arrange
      final original = GpsPoint(
        lat: 48.8566,
        lon: 2.3522,
        speed: 20.0,
        timestamp: DateTime.utc(2026, 6, 11, 15, 0, 0),
        isProjected: true,
      );

      // Act
      final json = original.toJson();
      final restored = GpsPoint.fromJson(json);

      // Assert
      expect(restored.lat, original.lat);
      expect(restored.lon, original.lon);
      expect(restored.speed, original.speed);
      expect(restored.isProjected, true);
    });
  });

  group('RouteLog JSON roundtrip', () {
    test('full log with points and marker survives encode → decode', () {
      // Arrange
      final log = RouteLog(week: 2900);
      log.add(GpsPoint(
        lat: 51.0, lon: 5.0, speed: 10.0,
        timestamp: DateTime.utc(2026, 6, 11, 12, 0, 0),
      ));
      log.add(GpsPoint(
        lat: 51.1, lon: 5.1, speed: 15.0,
        timestamp: DateTime.utc(2026, 6, 11, 12, 0, 5),
        isProjected: true,
      ));
      log.markProcessed();

      // Act
      final encoded = RouteLogStore.encode(log);
      final restored = RouteLogStore.decode(encoded);

      // Assert
      expect(restored.points.length, 2);
      expect(restored.points[0].lat, 51.0);
      expect(restored.points[1].isProjected, true);
      expect(restored.lastPackForgedAtIndex, 1);
      expect(restored.week, 2900);
      expect(restored.unprocessed, isEmpty);
    });

    test('empty log survives encode → decode', () {
      // Arrange
      final log = RouteLog(week: 2900);

      // Act
      final encoded = RouteLogStore.encode(log);
      final restored = RouteLogStore.decode(encoded);

      // Assert
      expect(restored.points, isEmpty);
      expect(restored.lastPackForgedAtIndex, isNull);
    });
  });

  group('RouteLog.unprocessed', () {
    test('all points unprocessed when marker is null', () {
      final log = RouteLog();
      log.add(GpsPoint(
        lat: 51.0, lon: 5.0, speed: 0, timestamp: DateTime.now(),
      ));
      expect(log.unprocessed.length, 1);
    });

    test('empty when all points processed', () {
      final log = RouteLog();
      log.add(GpsPoint(
        lat: 51.0, lon: 5.0, speed: 0, timestamp: DateTime.now(),
      ));
      log.markProcessed();
      expect(log.unprocessed, isEmpty);
    });

    test('only returns points after the marker', () {
      final log = RouteLog();
      log.add(GpsPoint(
        lat: 51.0, lon: 5.0, speed: 0, timestamp: DateTime.now(),
      ));
      log.add(GpsPoint(
        lat: 51.1, lon: 5.1, speed: 0, timestamp: DateTime.now(),
      ));
      log.add(GpsPoint(
        lat: 51.2, lon: 5.2, speed: 0, timestamp: DateTime.now(),
      ));
      log.lastPackForgedAtIndex = 0;
      expect(log.unprocessed.length, 2);
    });
  });

  group('RouteLogStore.maybePrune', () {
    test('clears log when week changes', () {
      final store = RouteLogStore();
      final log = RouteLog(week: 100);
      log.add(GpsPoint(
        lat: 51.0, lon: 5.0, speed: 0, timestamp: DateTime.now(),
      ));
      final pruned = store.maybePrune(log, 101);
      expect(pruned.points, isEmpty);
      expect(pruned.lastPackForgedAtIndex, isNull);
      expect(pruned.week, 101);
    });

    test('keeps log when week is the same', () {
      final store = RouteLogStore();
      final log = RouteLog(week: 100);
      log.add(GpsPoint(
        lat: 51.0, lon: 5.0, speed: 0, timestamp: DateTime.now(),
      ));
      final pruned = store.maybePrune(log, 100);
      expect(pruned.points.length, 1);
    });
  });
}
