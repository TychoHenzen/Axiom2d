import 'package:flutter/material.dart';
import 'package:flutter_background_service/flutter_background_service.dart';
import 'package:geolocator/geolocator.dart';

import 'domain/route_log.dart';
import 'ui/app.dart';
import 'ui/app_state.dart';

/// Entry point for the background isolate. Runs geolocator position stream
/// and appends points to the shared route log.
@pragma('vm:entry-point')
Future<void> backgroundServiceEntry(ServiceInstance service) async {
  WidgetsFlutterBinding.ensureInitialized();

  service.on('stopService').listen((_) => service.stopSelf());

  final permission = await Geolocator.checkPermission();
  if (permission == LocationPermission.denied ||
      permission == LocationPermission.deniedForever) {
    service.stopSelf();
    return;
  }

  final store = RouteLogStore();
  var log = await store.load();

  Geolocator.getPositionStream(
    locationSettings: AndroidSettings(
      accuracy: LocationAccuracy.high,
      distanceFilter: 5,
      intervalDuration: const Duration(seconds: 1),
    ),
  ).listen((p) async {
    log.add(GpsPoint(
      lat: p.latitude,
      lon: p.longitude,
      speed: p.speed,
      timestamp: DateTime.now().toUtc(),
    ));
    // Persist every ~10 points.
    if (log.points.length % 10 == 0) {
      await store.save(log);
    }
  });
}

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();

  // Configure foreground service for background GPS tracking.
  final service = FlutterBackgroundService();
  await service.configure(
    iosConfiguration: IosConfiguration(
      autoStart: false,
      onForeground: null,
      onBackground: null,
    ),
    androidConfiguration: AndroidConfiguration(
      onStart: backgroundServiceEntry,
      autoStart: false,
      isForegroundMode: true,
      foregroundServiceTypes: [AndroidForegroundType.location],
      notificationChannelId: 'gps_companion_channel',
      initialNotificationTitle: 'GPS Companion',
      initialNotificationContent: 'Tracking your location',
    ),
  );

  final state = await AppState.load();
  runApp(GpsCompanionApp(state: state));
}
