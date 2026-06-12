/// Shared app state: the player's inventory plus persistence. A ChangeNotifier
/// so screens rebuild when grains are collected or packs forged.
library;

import 'package:flutter/foundation.dart';

import '../data/store.dart';
import '../domain/biome.dart';
import '../domain/coverage.dart';
import '../domain/grain.dart';
import '../domain/inventory.dart';
import '../domain/leyline.dart';
import '../domain/route_log.dart';

class AppState extends ChangeNotifier {
  AppState({
    required this.store,
    required this.inventory,
    required this.routeLogStore,
    required this.routeLog,
  });

  final InventoryStore store;
  final Inventory inventory;
  final RouteLogStore routeLogStore;

  /// Persisted GPS datapoint log. Points after [RouteLog.lastPackForgedAtIndex]
  /// are replayed into coverage on load; after each forge the marker advances.
  RouteLog routeLog;

  /// Painted ground for the current week. Resets when the week rolls over,
  /// matching the leyline cadence. Grain seeds are day-based (they still expire
  /// at midnight UTC), but the painted overlay and accumulated signature persist
  /// across days within a week via the route log.
  final CoverageMap coverage = CoverageMap();
  int _coverageWeek = -1;

  static Future<AppState> load() async {
    final store = InventoryStore();
    final inventory = await store.load();
    final routeLogStore = RouteLogStore();
    var routeLog = await routeLogStore.load();
    final currentWeek = weekNumber(DateTime.now().toUtc());
    routeLog = routeLogStore.maybePrune(routeLog, currentWeek);
    final state = AppState(
      store: store,
      inventory: inventory,
      routeLogStore: routeLogStore,
      routeLog: routeLog,
    );
    // Replay unprocessed points from the persisted log to rebuild coverage.
    if (routeLog.unprocessed.isNotEmpty) {
      state._replayRouteLog(currentWeek);
    }
    return state;
  }

  int get grainCount => inventory.grainCount;
  int get packCount => inventory.boosters.length;
  List<Booster> get boosters => inventory.boosters;

  /// Percent of the next booster gathered (0..100): whole loose grains plus the
  /// sub-grain volume still accumulating in the field.
  double get packProgress =>
      (inventory.grainCount + coverage.volumeRemainder).clamp(0.0, 100.0);

  /// Replay unprocessed route-log points into coverage (called on startup).
  void _replayRouteLog(int currentWeek) {
    final result = coverage.replayFromLog(
      routeLog.unprocessed,
      day: DateTime.now().toUtc().difference(DateTime.utc(1970, 1, 1)).inDays,
      week: currentWeek,
    );
    if (result.grains.isNotEmpty) {
      inventory.collectAll(
        result.grains,
        now: DateTime.now().toUtc(),
        lat: routeLog.unprocessed.last.lat,
        lon: routeLog.unprocessed.last.lon,
      );
    }
    routeLog.markProcessed();
    _coverageWeek = currentWeek;
  }

  /// Vacuum the brush radius at (lat, lon): harvest freshly-covered ground,
  /// collect its grains, persist, notify. Returns boosters forged and the ids
  /// of cells painted this step (for the map overlay).
  Future<({int forged, List<String> newCells, List<Grain> grains})> vacuum({
    required double lat,
    required double lon,
    required int day,
    required int week,
    Biome biome = Biome.noData,
  }) async {
    if (week != _coverageWeek) {
      coverage.reset();
      routeLog.clear();
      _coverageWeek = week;
    }
    final harvest = coverage.harvest(
      lat: lat,
      lon: lon,
      day: day,
      week: week,
      biome: biome,
    );
    if (harvest.isEmpty) {
      return (forged: 0, newCells: const <String>[], grains: const <Grain>[]);
    }
    final forged = harvest.grains.isEmpty
        ? 0
        : inventory.collectAll(
            harvest.grains,
            now: DateTime.now().toUtc(),
            lat: lat,
            lon: lon,
          );
    if (harvest.grains.isNotEmpty) {
      await store.save(inventory);
      // Advance forge marker so replayed points aren't double-counted.
      routeLog.markProcessed();
      await routeLogStore.save(routeLog);
    }
    // Notify on any freshly painted ground so the pack-progress % ticks even
    // before a whole grain mints.
    notifyListeners();
    return (forged: forged, newCells: harvest.newCells, grains: harvest.grains);
  }

  /// Append a GPS fix to the route log and persist periodically.
  Future<void> logPosition({
    required double lat,
    required double lon,
    required double speed,
    required DateTime timestamp,
    bool isProjected = false,
  }) async {
    routeLog.add(GpsPoint(
      lat: lat,
      lon: lon,
      speed: speed,
      timestamp: timestamp,
      isProjected: isProjected,
    ));
    // Persist every ~10 points to limit I/O while keeping log reasonably fresh.
    if (routeLog.points.length % 10 == 0) {
      await routeLogStore.save(routeLog);
    }
  }
}
