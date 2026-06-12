/// Procedural leyline overlay (spec Layer 2). A weekly-seeded noise field
/// amplifies certain elements' spawn chances and makes the three leyline-only
/// grain types (Febris/Lumines/Inertiae) appear. Self-contained deterministic
/// value noise — no external noise dependency, so results are reproducible.
library;

import 'dart:math' as math;

import 'grain.dart';

/// Week index since the Unix epoch (UTC). Stable weekly seed source.
int weekNumber(DateTime when) {
  final days = when.toUtc().difference(DateTime.utc(1970, 1, 1)).inDays;
  return days ~/ 7;
}

int _hashInt(int x) {
  var h = x & 0xffffffff;
  h = (h ^ (h >> 16)) * 0x45d9f3b & 0xffffffff;
  h = (h ^ (h >> 16)) * 0x45d9f3b & 0xffffffff;
  h = h ^ (h >> 16);
  return h & 0xffffffff;
}

double _hashUnit(int a, int b, int c) {
  final h = _hashInt(a * 73856093 ^ b * 19349663 ^ c * 83492791);
  return h / 0xffffffff; // [0,1]
}

double _smooth(double t) => t * t * (3 - 2 * t);

/// Deterministic 3D value noise in `[-1, 1]`.
double noise3(double x, double y, double z) {
  final xi = x.floor(), yi = y.floor(), zi = z.floor();
  final xf = x - xi, yf = y - yi, zf = z - zi;
  double corner(int dx, int dy, int dz) =>
      _hashUnit(xi + dx, yi + dy, zi + dz);
  final u = _smooth(xf), v = _smooth(yf), w = _smooth(zf);
  double lerp(double a, double b, double t) => a + (b - a) * t;
  final x00 = lerp(corner(0, 0, 0), corner(1, 0, 0), u);
  final x10 = lerp(corner(0, 1, 0), corner(1, 1, 0), u);
  final x01 = lerp(corner(0, 0, 1), corner(1, 0, 1), u);
  final x11 = lerp(corner(0, 1, 1), corner(1, 1, 1), u);
  final y0 = lerp(x00, x10, v);
  final y1 = lerp(x01, x11, v);
  return lerp(y0, y1, w) * 2 - 1;
}

/// The thematic alignment for a given week.
class WeeklyTheme {
  WeeklyTheme({required this.dominantAxes, required this.intensity});

  /// Signature axis indices of the dominant leyline element(s) this week.
  final List<int> dominantAxes;

  /// Global multiplier on overlay magnitude (0.5x .. 2.0x).
  final double intensity;

  String get label {
    final names = dominantAxes.map((a) => GrainType.forAxis(a).json).join(' / ');
    return '$names alignment';
  }
}

/// Leyline axes (the three overlay-only elements).
const List<int> kLeylineAxes = [1, 3, 5]; // Febris, Lumines, Inertiae

/// Derive the weekly theme from the week seed.
WeeklyTheme weeklyTheme(int week) {
  final r = _hashInt(week);
  // 1-2 dominant leyline axes.
  final primary = kLeylineAxes[r % kLeylineAxes.length];
  final twoActive = (r >> 8) % 2 == 0;
  final secondary = kLeylineAxes[(r >> 4) % kLeylineAxes.length];
  final axes = <int>{primary, if (twoActive) secondary}.toList();
  final intensity = 0.5 + (_hashUnit(week, 7, 11)) * 1.5; // 0.5..2.0
  return WeeklyTheme(dominantAxes: axes, intensity: intensity);
}

/// Coarse leyline grid cell size in degrees (~1 km at mid-latitudes).
/// Hotspot positions are permanent per (cellX, cellY) and rotate element
/// weekly so the same spot shows a different leyline each week.
const double kLeylineCellDegrees = 0.009;

/// Effective radius of a single hotspot in metres.
const double kLeylineHotspotRadiusM = 500.0;

/// 8-element overlay modifier at a coordinate for a week.
///
/// Placement strategy:
/// - World divided into ~1 km coarse cells.
/// - ~1/3 of cells host a hotspot (seeded by permanent cell coordinate).
/// - Each hotspot has a fixed position within its cell and an element
///   that rotates weekly (so the character of a place shifts over time).
/// - Only the cell the player is in plus its 8 direct neighbours are
///   checked — O(9) deterministic hash lookups, no continuous noise eval.
/// - The weekly theme boosts hotspots whose element matches the season.
List<double> leylineOverlay(double lat, double lon, int week) {
  final out = List<double>.filled(8, 0.0);
  final cx = (lon / kLeylineCellDegrees).floor();
  final cy = (lat / kLeylineCellDegrees).floor();
  final theme = weeklyTheme(week);
  final cosLat = math.cos(lat * math.pi / 180.0).abs();

  for (var dy = -1; dy <= 1; dy++) {
    for (var dx = -1; dx <= 1; dx++) {
      final ncx = cx + dx;
      final ncy = cy + dy;

      // Permanent per-cell seed (independent of week).
      final pSeed = _hashInt(ncx * 73856093 ^ ncy * 19349663);

      // ~33% of cells have a hotspot.
      if (pSeed % 3 != 0) continue;

      // Element rotates weekly so the same spot changes character over time.
      final axis = kLeylineAxes[(pSeed + week) % kLeylineAxes.length];

      // Stable hotspot centre within the cell (fractional offset 0..1).
      final hotLat =
          (ncy + ((pSeed >> 8) & 0xff) / 255.0) * kLeylineCellDegrees;
      final hotLon =
          (ncx + ((pSeed >> 16) & 0xff) / 255.0) * kLeylineCellDegrees;

      // Metres from player to hotspot.
      final dlat = (lat - hotLat) * 111320.0;
      final dlon = (lon - hotLon) * 111320.0 * cosLat;
      final distM = math.sqrt(dlat * dlat + dlon * dlon);
      if (distM >= kLeylineHotspotRadiusM) continue;

      // Hotspots whose element matches this week's theme are stronger.
      final axisBoost = theme.dominantAxes.contains(axis) ? 1.4 : 0.7;
      out[axis] +=
          (1.0 - distM / kLeylineHotspotRadiusM) * theme.intensity * axisBoost * 0.5;
    }
  }
  return out;
}
