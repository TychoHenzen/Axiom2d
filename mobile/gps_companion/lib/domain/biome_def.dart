/// A resolved biome sample from OSM data. Replaces the fixed six-entry [Biome]
/// enum with tag-level granularity: each distinct OSM tag maps to its own
/// density, grain distribution, and fog-overlay color.
library;

/// A resolved biome sample from OSM data.
///
/// [dist] is a 5-element list: `[nature, urban, water, earth, arcane]`,
/// matching the five terrestrial grain types. The three leyline-only types
/// (Febris, Lumines, Inertiae) never appear in the base biome distribution.
class BiomeSample {
  const BiomeSample({
    required this.key,
    required this.densityPerKm2,
    required this.dist,
    required this.colorArgb,
  });

  /// OSM tag key=value string (e.g. `"landuse=forest"`).
  final String key;

  /// Grains per km-squared per day for this biome.
  final double densityPerKm2;

  /// 5-way grain distribution `[nature, urban, water, earth, arcane]`.
  /// Elements sum to 1.0 (within +/- 0.001).
  final List<double> dist;

  /// Fog overlay color as ARGB int (no dependency on `dart:ui`).
  final int colorArgb;
}
