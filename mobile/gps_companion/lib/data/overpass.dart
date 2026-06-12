/// Overpass API client: builds queries, fetches JSON, and parses responses into
/// region polygons (ways) and POI points (nodes) for the biome service.
library;

import 'dart:convert';

/// A lat/lon point from an Overpass geometry.
class OverpassPoint {
  const OverpassPoint({required this.lat, required this.lon});
  final double lat;
  final double lon;
}

/// A tagged region polygon (closed way) from Overpass.
class OverpassRegion {
  const OverpassRegion({
    required this.osmId,
    required this.tags,
    required this.geometry,
  });
  final int osmId;
  final Map<String, String> tags;
  final List<OverpassPoint> geometry;
}

/// A tagged POI node from Overpass.
class OverpassPoi {
  const OverpassPoi({
    required this.osmId,
    required this.lat,
    required this.lon,
    required this.tags,
  });
  final int osmId;
  final double lat;
  final double lon;
  final Map<String, String> tags;
}

/// Parsed Overpass API response.
class OverpassResponse {
  const OverpassResponse({required this.regions, required this.pois});
  final List<OverpassRegion> regions;
  final List<OverpassPoi> pois;
}

/// Build the Overpass API URL for a bbox ~[radiusM] metres around (lat, lon).
/// Queries landuse, natural, leisure, waterway, wetland ways + amenity,
/// historic, tourism, leisure, natural nodes.
String buildOverpassUrl({
  required double lat,
  required double lon,
  double radiusM = 400,
}) {
  // Approximate degree offsets.
  const mPerDeg = 111_320.0;
  final dLat = radiusM / mPerDeg;
  final cosLat = _cosDeg(lat);
  final dLon = radiusM / (mPerDeg * cosLat);
  final s = (dLat < dLon ? dLat : dLon); // use tighter bound for square bbox
  final bbox = '${lat - s},${lon - s},${lat + s},${lon + s}';

  final query = '''
[out:json];
(
  way["landuse"]($bbox);
  way["natural"]($bbox);
  way["leisure"]($bbox);
  way["waterway"]($bbox);
  way["wetland"]($bbox);
  node["amenity"]($bbox);
  node["historic"]($bbox);
  node["tourism"]($bbox);
  node["leisure"]($bbox);
  node["natural"]($bbox);
);
out geom;
'''
      .replaceAll('\n', '')
      .trim();

  return 'https://overpass-api.de/api/interpreter?data=${Uri.encodeComponent(query)}';
}

double _cosDeg(double degrees) {
  final r = (degrees * 3.141592653589793 / 180.0).abs();
  final c = _cos(r);
  return c < 1e-9 ? 1e-9 : c;
}

// Chebyshev approximation for cos(x) on [-pi/2, pi/2].
double _cos(double x) {
  // Reduce to [-pi, pi].
  const pi = 3.141592653589793;
  x = x % (2 * pi);
  if (x < -pi) x += 2 * pi;
  if (x > pi) x -= 2 * pi;
  final x2 = x * x;
  return 1.0 +
      x2 * (-0.4999999963 +
          x2 * (0.0416666418 +
              x2 * (-0.0013888397 +
                  x2 * (2.47609e-5 + x2 * (-2.605e-7)))));
}

/// Parse raw Overpass JSON into [OverpassResponse].
OverpassResponse parseOverpassResponse(String json) {
  final root = jsonDecode(json) as Map<String, dynamic>;
  final elements = root['elements'] as List<dynamic>? ?? [];
  final regions = <OverpassRegion>[];
  final pois = <OverpassPoi>[];

  for (final e in elements) {
    final map = e as Map<String, dynamic>;
    final type = map['type'] as String? ?? '';
    final id = (map['id'] as num).toInt();
    final rawTags = map['tags'] as Map<String, dynamic>? ?? {};
    final tags = rawTags.map((k, v) => MapEntry(k, v.toString()));

    if (type == 'way') {
      final geomRaw = map['geometry'] as List<dynamic>? ?? [];
      if (geomRaw.isEmpty) continue;
      final geom = geomRaw.map((pt) {
        final p = pt as Map<String, dynamic>;
        return OverpassPoint(
          lat: (p['lat'] as num).toDouble(),
          lon: (p['lon'] as num).toDouble(),
        );
      }).toList();
      if (geom.length >= 3) {
        regions.add(OverpassRegion(osmId: id, tags: tags, geometry: geom));
      }
    } else if (type == 'node') {
      final lat = (map['lat'] as num?)?.toDouble();
      final lon = (map['lon'] as num?)?.toDouble();
      if (lat == null || lon == null || tags.isEmpty) continue;
      pois.add(OverpassPoi(osmId: id, lat: lat, lon: lon, tags: tags));
    }
  }
  return OverpassResponse(regions: regions, pois: pois);
}
