/// Dead reckoning position projection. When GPS delivers a fix with speed
/// and bearing, intermediate positions are projected at ~1-second intervals
/// so coverage vacuuming feels smooth even between infrequent GPS updates.
library;

import 'dart:math' as math;

import 'route_log.dart';

/// Earth's meridional radius in metres (WGS84 approximation).
const double _metersPerDegLat = 111320.0;

/// Maximum seconds to project ahead of the last real fix. Beyond this horizon
/// the position error from unaccounted turns grows too large.
const double kMaxProjectionHorizonSeconds = 5.0;

/// Project intermediate positions between two GPS fixes using dead reckoning.
///
/// [last] is the previous real fix, [current] is the latest real fix. The
/// bearing (heading) from [current] is used for forward projection.
///
/// Returns a list of projected points at ~1-second intervals between the
/// two fixes, plus forward-projected points if [current] is the latest known
/// fix (extend ahead up to [kMaxProjectionHorizonSeconds]).
List<GpsPoint> projectIntermediate({
  required GpsPoint last,
  required GpsPoint current,
  required double headingDegrees,
  DateTime? nowOverride,
}) {
  final now = nowOverride ?? DateTime.now().toUtc();
  if (current.speed < 1.0) return <GpsPoint>[];

  final dtSeconds = current.timestamp.difference(last.timestamp).inMilliseconds / 1000.0;
  if (dtSeconds <= 1.0) return <GpsPoint>[];

  final bearingRad = headingDegrees * math.pi / 180.0;
  final cosLat = math.cos(current.lat * math.pi / 180.0).abs();
  if (cosLat < 1e-9) return <GpsPoint>[];

  final out = <GpsPoint>[];

  // Backfill: project intermediate points forward from last fix toward current.
  var t = 1.0;
  while (t < dtSeconds) {
    final d = current.speed * t;
    final dLat = d * math.cos(bearingRad) / _metersPerDegLat;
    final dLon = d * math.sin(bearingRad) / (_metersPerDegLat * cosLat);
    out.add(GpsPoint(
      lat: last.lat + dLat,
      lon: last.lon + dLon,
      speed: current.speed,
      timestamp: last.timestamp.add(Duration(milliseconds: (t * 1000).round())),
      isProjected: true,
    ));
    t += 1.0;
  }

  // Forward projection: extend ahead up to horizon from the current fix.
  var ahead = 1.0;
  final elapsed = now.difference(current.timestamp).inMilliseconds / 1000.0;
  final horizon = (kMaxProjectionHorizonSeconds - elapsed).clamp(0.0, kMaxProjectionHorizonSeconds);
  while (ahead <= horizon) {
    final d = current.speed * ahead;
    final dLat = d * math.cos(bearingRad) / _metersPerDegLat;
    final dLon = d * math.sin(bearingRad) / (_metersPerDegLat * cosLat);
    out.add(GpsPoint(
      lat: current.lat + dLat,
      lon: current.lon + dLon,
      speed: current.speed,
      timestamp: current.timestamp.add(Duration(milliseconds: (ahead * 1000).round())),
      isProjected: true,
    ));
    ahead += 1.0;
  }

  return out;
}
