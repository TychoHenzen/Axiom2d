import 'dart:async';

import 'package:geolocator/geolocator.dart';
import 'package:gps_companion/data/gps_service.dart';

/// Test double for GpsService. Accepts [Position] objects via [emit];
/// each call adds a fix to the position stream.
class FakeGpsService implements GpsService {
  final StreamController<Position> _controller =
      StreamController<Position>.broadcast();

  @override
  Stream<Position> get positionStream => _controller.stream;

  /// Inject a GPS fix into the stream.
  void emit(Position position) => _controller.add(position);

  /// Close the stream when the journey is complete.
  Future<void> close() => _controller.close();
}
