/// Deterministic grain spawning (spec "Spawn Algorithm"). Grains for a given
/// cell + day + week seed are fully reproducible, so every device sees the same
/// grains at the same place and time without any server.
library;

import 'dart:math' as math;

import 'biome.dart';
import 'grain.dart';
import 'leyline.dart';
import 'rarity.dart';

/// Approximate cell size in degrees (~300 m at mid latitudes).
const double kCellDegrees = 0.0027;

/// A grain placed at a world coordinate.
class SpawnedGrain {
  SpawnedGrain({required this.grain, required this.lat, required this.lon, required this.id});

  final Grain grain;
  final double lat;
  final double lon;
  final String id;
}

/// Day index since the Unix epoch (UTC). Grains expire at midnight UTC.
int dayNumber(DateTime when) =>
    when.toUtc().difference(DateTime.utc(1970, 1, 1)).inDays;

int _mix(int a, int b) {
  var h = (a * 0x9e3779b1) & 0xffffffff;
  h ^= b & 0xffffffff;
  h = (h ^ (h >> 15)) * 0x85ebca6b & 0xffffffff;
  h = (h ^ (h >> 13)) * 0xc2b2ae35 & 0xffffffff;
  return (h ^ (h >> 16)) & 0xffffffff;
}

/// Seed for a spawn cell.
int cellSeed(int cellX, int cellY, int day, int week) =>
    _mix(_mix(_mix(cellX, cellY), day), week);

/// Poisson-ish count from an expected value using a seeded RNG.
int _sampleCount(double expected, math.Random rng) {
  if (expected <= 0) return 0;
  // Knuth's Poisson sampler.
  final l = math.exp(-expected);
  var k = 0;
  var p = 1.0;
  do {
    k++;
    p *= rng.nextDouble();
  } while (p > l);
  return k - 1;
}

/// Spawn all grains for one cell. [cellDegrees] lets the same deterministic
/// model run at a finer resolution (the continuous "field" uses small cells).
List<SpawnedGrain> spawnCell({
  required int cellX,
  required int cellY,
  required int day,
  required int week,
  Biome biome = Biome.noData,
  double cellDegrees = kCellDegrees,
}) {
  final seed = cellSeed(cellX, cellY, day, week);
  final rng = math.Random(seed);
  final cellAreaKm2 = _cellAreaKm2(cellY, cellDegrees);
  final expected = biome.densityPerKm2 * cellAreaKm2;
  final count = _sampleCount(expected, rng);

  final dist = kBiomeDistribution[biome]!;
  final originLat = cellY * cellDegrees;
  final originLon = cellX * cellDegrees;
  final overlay = leylineOverlay(originLat, originLon, week);

  final out = <SpawnedGrain>[];
  for (var i = 0; i < count; i++) {
    final lat = originLat + rng.nextDouble() * cellDegrees;
    final lon = originLon + rng.nextDouble() * cellDegrees;

    final type = _pickType(dist, overlay, rng);
    final axes = _axesFor(type, overlay, rng);
    final grainSeed = _mix(seed, i);
    final grain = Grain(
      axes: axes,
      type: type,
      rarity: grainRarity(axes, grainSeed),
    );
    out.add(SpawnedGrain(grain: grain, lat: lat, lon: lon, id: '$seed-$i'));
  }
  return out;
}

/// Expected (fractional) grain yield of a cell: area × biome density. Unlike
/// [spawnCell] this does NOT Poisson-round — the continuous brush accumulates
/// these fractions so tiny per-cell yields are never lost to rounding.
double cellExpectedYield({
  required int cellY,
  Biome biome = Biome.noData,
  double cellDegrees = kCellDegrees,
}) => biome.densityPerKm2 * _cellAreaKm2(cellY, cellDegrees);

/// Per-axis weights (sum 1.0) for a cell: the biome's type distribution mapped
/// onto axes, blended with the leyline overlay. The continuous field multiplies
/// these by a cell's volume to accumulate signature mass per dimension.
List<double> axisWeights({
  required int cellX,
  required int cellY,
  required int week,
  Biome biome = Biome.noData,
  double cellDegrees = kCellDegrees,
}) {
  final w = List<double>.filled(8, 0.0);
  final overlay = leylineOverlay(cellY * cellDegrees, cellX * cellDegrees, week);
  final leyBoost = kLeylineAxes.fold<double>(0, (s, a) => s + overlay[a]);
  final leyFrac = leyBoost > 0 ? leyBoost / (1 + leyBoost) : 0.0;
  kBiomeDistribution[biome]!.forEach((type, weight) {
    w[type.axis] += (1 - leyFrac) * weight;
  });
  if (leyFrac > 0) {
    for (final a in kLeylineAxes) {
      w[a] += leyFrac * (overlay[a] / leyBoost);
    }
  }
  return w;
}

/// Mint one grain from an accumulated signature vector (one grain's worth of
/// per-axis mass, summing to ~1). The signature direction becomes the grain's
/// 8-axis vector; a concentrated (pure-biome) signature yields a higher
/// magnitude — and thus rarer — grain than a spread-out (mixed) one.
///
/// [densityFactor] scales the signature magnitude: scale = 0.15 * sqrt(f).
Grain mintGrainFromSignature({
  required List<double> signature,
  required int seed,
  double densityFactor = 1.0,
}) {
  final rng = math.Random(seed);
  final total = signature.fold<double>(0, (s, v) => s + v);
  final norm = total > 0 ? total : 1.0;
  final scale = 0.15 * math.sqrt(densityFactor);
  var dom = 0;
  final axes = List<double>.filled(8, 0.0);
  for (var a = 0; a < 8; a++) {
    if (signature[a] > signature[dom]) dom = a;
    final sign = rng.nextBool() ? 1.0 : -1.0;
    axes[a] = sign * (signature[a] / norm) * scale;
  }
  return Grain(
    axes: axes,
    type: GrainType.forAxis(dom),
    rarity: grainRarity(axes, seed),
  );
}

double _cellAreaKm2(int cellY, double cellDegrees) {
  final latDeg = cellY * cellDegrees;
  final latKm = cellDegrees * 111.0;
  final lonKm = cellDegrees * 111.0 * math.cos(latDeg * math.pi / 180.0).abs();
  return latKm * lonKm;
}

/// Pick a grain type, allowing leyline overlay to inject leyline-only types.
GrainType _pickType(Map<GrainType, double> dist, List<double> overlay, math.Random rng) {
  // Leyline injection chance proportional to total leyline boost this cell.
  final leylineBoost = kLeylineAxes.fold<double>(0, (s, a) => s + overlay[a]);
  if (leylineBoost > 0 && rng.nextDouble() < leylineBoost / (1 + leylineBoost)) {
    // Choose among leyline axes weighted by their boost.
    final total = kLeylineAxes.fold<double>(0, (s, a) => s + overlay[a]);
    var t = rng.nextDouble() * total;
    for (final a in kLeylineAxes) {
      t -= overlay[a];
      if (t <= 0) return GrainType.forAxis(a);
    }
    return GrainType.forAxis(kLeylineAxes.first);
  }
  return pickFromDistribution(dist, rng.nextDouble());
}

/// Build the 8-axis vector: dominant axis for the type, tiny noise elsewhere.
List<double> _axesFor(GrainType type, List<double> overlay, math.Random rng) {
  final axes = List<double>.filled(8, 0.0);
  // Dominant magnitude ~0.01..0.15, skewed low (rare high values).
  final roll = rng.nextDouble();
  final magnitude = 0.01 + math.pow(roll, 6).toDouble() * 0.14;
  final sign = rng.nextBool() ? 1.0 : -1.0;
  axes[type.axis] = sign * magnitude;
  for (var i = 0; i < 8; i++) {
    if (i == type.axis) continue;
    axes[i] = (rng.nextDouble() - 0.5) * 0.01;
  }
  return axes;
}
