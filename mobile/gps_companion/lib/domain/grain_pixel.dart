/// Grain pixel color model: maps every (GrainType, sign) pair to one of the
/// 16 Aspect colors defined in the desktop `gem_sockets.rs` aspect_color()
/// palette. Positive dominant-axis value → warm hue; negative → cool hue.
library;

import 'grain.dart';

/// RGB triple from the desktop aspect_color() palette.
class AspectColor {
  const AspectColor(this.r, this.g, this.b);
  final double r;
  final double g;
  final double b;
}

/// The 16 Aspect colors, matching `crates/card_game/src/card/identity/gem_sockets.rs`.
/// Positive = warm (r > b or g > b), negative = cool (b > r).
const _warm = [
  AspectColor(0.85, 0.55, 0.20), // Solid — amber
  AspectColor(0.95, 0.25, 0.10), // Heat — red-orange
  AspectColor(0.90, 0.80, 0.10), // Order — gold
  AspectColor(0.98, 0.95, 0.40), // Light — bright yellow
  AspectColor(0.70, 0.85, 0.10), // Change — yellow-green
  AspectColor(0.90, 0.40, 0.05), // Force — deep orange
  AspectColor(0.20, 0.80, 0.20), // Growth — green
  AspectColor(0.60, 0.90, 0.30), // Expansion — lime
];

const _cool = [
  AspectColor(0.30, 0.50, 0.85), // Fragile — periwinkle
  AspectColor(0.10, 0.70, 0.95), // Cold — ice blue
  AspectColor(0.55, 0.10, 0.80), // Chaos — violet
  AspectColor(0.15, 0.05, 0.40), // Dark — deep indigo
  AspectColor(0.20, 0.60, 0.80), // Stasis — steel blue
  AspectColor(0.10, 0.75, 0.70), // Calm — teal
  AspectColor(0.35, 0.20, 0.60), // Decay — muted purple
  AspectColor(0.05, 0.20, 0.70), // Contraction — navy
];

/// Index of the axis with largest absolute magnitude in [axes].
int dominantAxisIndex(List<double> axes) {
  var best = 0;
  var bestMag = axes[0].abs();
  for (var i = 1; i < axes.length; i++) {
    final m = axes[i].abs();
    if (m > bestMag) {
      bestMag = m;
      best = i;
    }
  }
  return best;
}

/// Color for a grain pixel, matching the desktop aspect_color() palette.
///
/// The mapping is determined by the desktop Element → Aspect assignment:
///   Solidum(0): +Solid(amber) / -Fragile(periwinkle)
///   Febris(1):  +Heat(red-orange) / -Cold(ice-blue)
///   Ordinem(2): +Order(gold) / -Chaos(violet)
///   Lumines(3): +Light(yellow) / -Dark(indigo)
///   Varias(4):  +Change(yellow-green) / -Stasis(steel-blue)
///   Inertiae(5): +Force(orange) / -Calm(teal)
///   Subsidium(6): +Growth(green) / -Decay(muted-purple)
///   Spatium(7):  +Expansion(lime) / -Contraction(navy)
///
/// Where GrainType axis matches Element index: earth=0, febris=1,
/// urban=2, lumines=3, water=4, inertiae=5, nature=6, arcane=7.
AspectColor grainPixelColor(Grain grain) {
  final axis = dominantAxisIndex(grain.axes);
  final sign = grain.axes[axis] >= 0;
  // Each axis maps to one warm + one cool color at the same index.
  return sign ? _warm[axis] : _cool[axis];
}

/// Convenience: returns (GrainType, isPositive) → AspectColor for the
/// grain's dominant axis, without needing a full [Grain] instance.
AspectColor aspectColorFor(GrainType type, bool positive) =>
    positive ? _warm[type.axis] : _cool[type.axis];

/// All 16 distinct (GrainType, sign) color pairs.
List<({GrainType type, bool positive, AspectColor color})> get allGrainColors =>
    GrainType.values.expand((t) => [
          (type: t, positive: true, color: _warm[t.axis]),
          (type: t, positive: false, color: _cool[t.axis]),
        ]).toList();
