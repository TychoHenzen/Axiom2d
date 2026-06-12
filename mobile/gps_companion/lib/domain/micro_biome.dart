/// POI micro-biome overrides (spec Layer 1b). When the player is near a
/// point of interest the base biome distribution shifts toward the POI's
/// thematic grain type, with density falling off monotonically from the center.
library;

import 'dart:math' as math;

import 'biome_def.dart';

/// Simple lat/lon pair for domain-layer distance calculations. Avoids a
/// dependency on `latlong2` or `flutter_map` in the domain model.
typedef LatLon = ({double lat, double lon});

/// POI category that determines the grain bias of a micro-biome.
enum MicroBiomeCategory {
  /// Fountain, pond, swimming pool — water bias.
  waterFeature(
    dist: [0.08, 0.05, 0.62, 0.10, 0.15],
    densityBump: 80,
    radiusM: 8.0,
    colorArgb: 0x604169E1,
  ),

  /// Statue, monument, historical marker — arcane bias.
  monument(
    dist: [0.05, 0.10, 0.05, 0.10, 0.70],
    densityBump: 60,
    radiusM: 6.0,
    colorArgb: 0x60483D8B,
  ),

  /// Tree, flowerbed, garden feature — nature bias.
  greenery(
    dist: [0.65, 0.05, 0.08, 0.12, 0.10],
    densityBump: 70,
    radiusM: 7.0,
    colorArgb: 0x60228B22,
  ),

  /// Bus stop, bench, kiosk — urban bias.
  streetFurniture(
    dist: [0.05, 0.65, 0.05, 0.10, 0.15],
    densityBump: 50,
    radiusM: 5.0,
    colorArgb: 0x60A0A0A0,
  ),

  /// Rock outcrop, boulder, geological marker — earth bias.
  geological(
    dist: [0.05, 0.03, 0.05, 0.72, 0.15],
    densityBump: 55,
    radiusM: 6.0,
    colorArgb: 0x60A0522D,
  );

  const MicroBiomeCategory({
    required this.dist,
    required this.densityBump,
    required this.radiusM,
    required this.colorArgb,
  });

  /// 5-way grain distribution `[nature, urban, water, earth, arcane]`.
  final List<double> dist;

  /// Extra grains/km-squared/day added at POI center.
  final double densityBump;

  /// Effective radius in metres (5-10 m).
  final double radiusM;

  /// Fog overlay ARGB color for the micro-biome ring.
  final int colorArgb;
}

/// Approximate distance in metres between two lat/lon points using
/// the equirectangular projection (accurate enough at <100 m scale).
double _distanceMetres(LatLon a, LatLon b) {
  const metersPerDegLat = 111_320.0;
  final dLat = (a.lat - b.lat) * metersPerDegLat;
  final avgLat = (a.lat + b.lat) / 2.0;
  final dLon = (a.lon - b.lon) * metersPerDegLat * math.cos(avgLat * math.pi / 180.0);
  return math.sqrt(dLat * dLat + dLon * dLon);
}

/// Linear falloff from 1.0 at center to 0.0 at [radius]. Always in `[0, 1]`;
/// returns 0 outside radius. Monotonically decreasing with distance.
double _falloff(double distance, double radius) {
  if (distance >= radius) return 0.0;
  return 1.0 - distance / radius;
}

/// Returns a [BiomeSample] if [query] is within the micro-biome radius of
/// [poi] for the given [category]. Density is scaled by distance falloff
/// (closer = stronger). Returns `null` when outside the radius.
BiomeSample? microBiomeAt(
  LatLon poi,
  LatLon query,
  MicroBiomeCategory category,
) {
  final d = _distanceMetres(poi, query);
  final f = _falloff(d, category.radiusM);
  if (f <= 0.0) return null;

  return BiomeSample(
    key: 'poi=${category.name}',
    densityPerKm2: category.densityBump * f,
    dist: category.dist,
    colorArgb: category.colorArgb,
  );
}
