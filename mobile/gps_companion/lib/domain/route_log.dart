/// GPS datapoint log: stores the player's position history so coverage can be
/// replayed after app restart. Persisted via shared_preferences under key
/// `route_log_v1`. Weekly pruning matches the leyline cadence.
library;

import 'dart:convert';

import 'package:shared_preferences/shared_preferences.dart';

/// A single GPS fix recorded in the route log.
class GpsPoint {
  GpsPoint({
    required this.lat,
    required this.lon,
    required this.speed,
    required this.timestamp,
    this.isProjected = false,
  });

  final double lat;
  final double lon;

  /// Speed in m/s at this fix (0 if unknown).
  final double speed;

  /// UTC timestamp of this fix.
  final DateTime timestamp;

  /// True when this point was produced by dead reckoning, not a real GPS fix.
  final bool isProjected;

  Map<String, dynamic> toJson() => {
    'lat': lat,
    'lon': lon,
    'speed': speed,
    'ts': timestamp.toIso8601String(),
    'proj': isProjected,
  };

  static GpsPoint fromJson(Map<String, dynamic> j) => GpsPoint(
    lat: (j['lat'] as num).toDouble(),
    lon: (j['lon'] as num).toDouble(),
    speed: (j['speed'] as num).toDouble(),
    timestamp: DateTime.parse(j['ts'] as String),
    isProjected: j['proj'] as bool? ?? false,
  );
}

/// Route log: persisted GPS history with a marker tracking which points have
/// already been processed for booster pack forging.
class RouteLog {
  RouteLog({
    List<GpsPoint>? points,
    this.lastPackForgedAtIndex,
    this.week = -1,
  }) : points = points ?? <GpsPoint>[];

  final List<GpsPoint> points;
  int? lastPackForgedAtIndex;
  int week;

  bool get isEmpty => points.isEmpty;

  /// Points after the last-forged marker that still need replay.
  List<GpsPoint> get unprocessed {
    if (lastPackForgedAtIndex == null) return points;
    if (lastPackForgedAtIndex! >= points.length - 1) return <GpsPoint>[];
    return points.sublist(lastPackForgedAtIndex! + 1);
  }

  /// Append a real GPS fix.
  void add(GpsPoint p) => points.add(p);

  /// Mark all points up to the current end as processed (called after forge).
  void markProcessed() {
    lastPackForgedAtIndex = points.isEmpty ? null : points.length - 1;
  }

  /// Drop all points and reset the forge marker.
  void clear() {
    points.clear();
    lastPackForgedAtIndex = null;
  }

  Map<String, dynamic> toJson() => {
    'points': points.map((p) => p.toJson()).toList(),
    'lastPackForgedAtIndex': lastPackForgedAtIndex,
    'week': week,
  };

  static RouteLog fromJson(Map<String, dynamic> j) => RouteLog(
    points: (j['points'] as List)
        .map((e) => GpsPoint.fromJson(e as Map<String, dynamic>))
        .toList(),
    lastPackForgedAtIndex: j['lastPackForgedAtIndex'] as int?,
    week: j['week'] as int? ?? -1,
  );
}

/// Persisted route log store.
class RouteLogStore {
  static const String _key = 'route_log_v1';

  Future<RouteLog> load() async {
    final prefs = await SharedPreferences.getInstance();
    final raw = prefs.getString(_key);
    if (raw == null) return RouteLog();
    return decode(raw);
  }

  Future<void> save(RouteLog log) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_key, encode(log));
  }

  /// Weekly prune: if the current week differs from the stored week, clear log.
  RouteLog maybePrune(RouteLog log, int currentWeek) {
    if (log.week != currentWeek) {
      return RouteLog(week: currentWeek);
    }
    return log;
  }

  static String encode(RouteLog log) => jsonEncode(log.toJson());

  static RouteLog decode(String raw) {
    final j = jsonDecode(raw) as Map<String, dynamic>;
    return RouteLog.fromJson(j);
  }
}
