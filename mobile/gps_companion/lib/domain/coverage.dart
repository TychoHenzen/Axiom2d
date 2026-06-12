/// Continuous "brush" collection. Instead of discrete tappable grains, the
/// player's collection radius vacuums up ground: the world is a fine grid of
/// paint cells, and entering a cell harvests its (deterministic) grains exactly
/// once. The covered set is the painted overlay. Coverage is day-scoped — grain
/// seeds are day-based ("grains expire at midnight UTC"), so each new day starts
/// on a fresh, unpainted field.
library;

import 'dart:math' as math;

import 'biome.dart';
import 'grain.dart';
import 'route_log.dart';
import 'spawn.dart';

/// Paint-cell size in degrees (~5.5 m latitude). Fine grid so the brush feels
/// continuous and the painted squares are small.
const double kPaintCellDegrees = 0.00005;

/// Collection radius in metres (spec: ~20 m brush).
const double kCollectRadiusMeters = 20;

/// Game-balance knob: scales raw spec density (grains/km²/day) into the brush
/// yield. ~20 ≈ one 100-grain booster per ~1 km walked at base density. Raise
/// to forge faster, lower for more scarcity.
const double kYieldMultiplier = 20.0;

/// Reference biome density (grains/km²/day) for yield factor normalization.
/// Yield factor f = clamp(density / kBaseDensity, 0.5, 10.0).
const double kBaseDensity = 200.0;

const double _metersPerDegLat = 111320.0;

int paintCellX(double lon) => (lon / kPaintCellDegrees).floor();
int paintCellY(double lat) => (lat / kPaintCellDegrees).floor();
String paintCellId(int ix, int iy) => '$ix:$iy';
double paintCellCenterLat(int iy) => (iy + 0.5) * kPaintCellDegrees;
double paintCellCenterLon(int ix) => (ix + 0.5) * kPaintCellDegrees;

/// Approximate ground area of one paint cell at the given latitude.
double paintCellAreaM2(double lat) {
  final latM = kPaintCellDegrees * _metersPerDegLat;
  final lonM =
      kPaintCellDegrees * _metersPerDegLat * math.cos(lat * math.pi / 180.0).abs();
  return latM * lonM;
}

/// Equirectangular metre distance — good enough at grain scale, and pure Dart
/// so the domain stays testable without platform plugins.
double metersBetween(double lat1, double lon1, double lat2, double lon2) {
  const r = 6371000.0;
  final dLat = (lat2 - lat1) * math.pi / 180.0;
  final mLat = (lat1 + lat2) / 2.0 * math.pi / 180.0;
  final dLon = (lon2 - lon1) * math.pi / 180.0;
  final x = dLon * math.cos(mLat);
  return r * math.sqrt(dLat * dLat + x * x);
}

/// Paint cells whose centre lies within [radiusM] of (lat, lon).
List<({int ix, int iy})> cellsWithinRadius(
  double lat,
  double lon,
  double radiusM,
) {
  final latSpanDeg = radiusM / _metersPerDegLat;
  final cosLat = math.cos(lat * math.pi / 180.0).abs();
  final lonSpanDeg = radiusM / (_metersPerDegLat * (cosLat == 0 ? 1e-9 : cosLat));
  final minIx = paintCellX(lon - lonSpanDeg);
  final maxIx = paintCellX(lon + lonSpanDeg);
  final minIy = paintCellY(lat - latSpanDeg);
  final maxIy = paintCellY(lat + latSpanDeg);
  final out = <({int ix, int iy})>[];
  for (var ix = minIx; ix <= maxIx; ix++) {
    for (var iy = minIy; iy <= maxIy; iy++) {
      if (metersBetween(lat, lon, paintCellCenterLat(iy), paintCellCenterLon(ix)) <=
          radiusM) {
        out.add((ix: ix, iy: iy));
      }
    }
  }
  return out;
}

/// What a single brush step harvested.
class HarvestResult {
  HarvestResult({required this.grains, required this.newCells});

  /// Grains vacuumed from cells freshly covered this step.
  final List<Grain> grains;

  /// Ids of cells painted this step (for overlay rendering).
  final List<String> newCells;

  bool get isEmpty => newCells.isEmpty;
}

int _hash(int a, int b) {
  var h = (a * 0x9e3779b1) & 0xffffffff;
  h = ((h ^ b) * 0x85ebca6b) & 0xffffffff;
  return (h ^ (h >> 13)) & 0xffffffff;
}

/// Tracks the player's harvest for the current day: which cells are painted,
/// the accumulated per-axis signature mass, and how many grains have minted.
class CoverageMap {
  CoverageMap({Set<String>? covered}) : covered = covered ?? <String>{};

  final Set<String> covered;

  /// Accumulated signature mass per axis. Each axis fills from biome/leyline
  /// weights as the player paints ground; the total is the "volume" — when it
  /// crosses 1.0 a grain crystallises from the accumulated signature.
  final List<double> signature = List<double>.filled(kAxisCount, 0.0);

  int get cellCount => covered.length;

  /// Fractional progress toward the next grain (0..1): the accumulated volume
  /// not yet minted into a whole grain.
  double get volumeRemainder => signature.fold<double>(0, (s, v) => s + v);

  /// Approximate vacuumed area, using the cell size at [latRef].
  double areaM2(double latRef) => covered.length * paintCellAreaM2(latRef);

  /// Clears all daily state (called when the day rolls over).
  void reset() {
    covered.clear();
    for (var i = 0; i < signature.length; i++) {
      signature[i] = 0.0;
    }
  }

  /// Harvest every not-yet-covered cell within the brush radius. Volume and
  /// signature accumulate continuously (no per-cell rounding), and whole grains
  /// mint as the accumulated volume crosses integers. Re-entering painted ground
  /// yields nothing.
  ///
  /// New cells are processed in sorted order for deterministic minting;
  /// mint seeds derive from day, week, and cell id rather than a mutable
  /// counter so that deferred biome resolution cannot perturb grain identity.
  HarvestResult harvest({
    required double lat,
    required double lon,
    required int day,
    required int week,
    double radiusM = kCollectRadiusMeters,
    Biome biome = Biome.noData,
  }) {
    final grains = <Grain>[];
    final fresh = <String>[];

    // Collect newly-covered cell ids, then sort for deterministic accrual.
    final newIds = <String>[];
    for (final c in cellsWithinRadius(lat, lon, radiusM)) {
      final id = paintCellId(c.ix, c.iy);
      if (!covered.contains(id)) {
        newIds.add(id);
      }
    }
    if (newIds.isEmpty) return HarvestResult(grains: grains, newCells: fresh);
    newIds.sort();

    // Per-cell accumulation + per-cell minting so seeds are cell-scoped.
    for (final id in newIds) {
      covered.add(id);
      fresh.add(id);
      final parts = id.split(':');
      final ix = int.parse(parts[0]);
      final iy = int.parse(parts[1]);

      final density = biome.densityPerKm2;
      final f = (density / kBaseDensity).clamp(0.5, 10.0);

      final vol = cellExpectedYield(
            cellY: iy,
            biome: biome,
            cellDegrees: kPaintCellDegrees,
          ) *
          kYieldMultiplier;
      final w = axisWeights(
        cellX: ix,
        cellY: iy,
        week: week,
        biome: biome,
        cellDegrees: kPaintCellDegrees,
      );
      for (var a = 0; a < kAxisCount; a++) {
        signature[a] += vol * w[a];
      }

      // Mint whole grains — seed from day, week, and cell id.
      var grainFromCell = 0;
      while (volumeRemainder >= 1.0) {
        final total = volumeRemainder;
        final factor = 1.0 / total;
        final consumed = List<double>.generate(
          kAxisCount,
          (a) => signature[a] * factor,
        );
        for (var a = 0; a < kAxisCount; a++) {
          signature[a] -= consumed[a];
        }
        grains.add(mintGrainFromSignature(
          signature: consumed,
          seed: _hash(_hash(day, week), _hash(id.hashCode, grainFromCell++)),
          densityFactor: f,
        ));
      }
    }
    return HarvestResult(grains: grains, newCells: fresh);
  }

  /// Replay a list of GPS points through the harvest pipeline to rebuild
  /// coverage state (used on app restart to restore previously covered cells
  /// and minted grains from the persisted route log).
  ///
  /// Returns the total grains collected during replay (for minting into
  /// inventory) and the cells painted.
  ({List<Grain> grains, List<String> newCells}) replayFromLog(
    List<GpsPoint> points, {
    required int day,
    required int week,
    Biome biome = Biome.noData,
  }) {
    final allGrains = <Grain>[];
    final allCells = <String>[];
    for (final p in points) {
      final h = harvest(
        lat: p.lat,
        lon: p.lon,
        day: day,
        week: week,
        biome: biome,
      );
      allGrains.addAll(h.grains);
      allCells.addAll(h.newCells);
    }
    return (grains: allGrains, newCells: allCells);
  }
}
