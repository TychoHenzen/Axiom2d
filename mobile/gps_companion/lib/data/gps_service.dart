/// GPS stream abstraction so widget tests can inject a fake stream
/// without requiring platform location services.
library;

import 'package:geolocator/geolocator.dart';

/// Abstract GPS position stream. Inject into MapScreen for testability.
abstract class GpsService {
  /// Continuous stream of GPS position updates.
  Stream<Position> get positionStream;
}

/// Production implementation backed by the device Geolocator plugin.
class GeolocatorGpsService implements GpsService {
  const GeolocatorGpsService();

  @override
  Stream<Position> get positionStream => Geolocator.getPositionStream(
        locationSettings: AndroidSettings(
          accuracy: LocationAccuracy.high,
          distanceFilter: 5,
          intervalDuration: const Duration(seconds: 1),
        ),
      );
}
