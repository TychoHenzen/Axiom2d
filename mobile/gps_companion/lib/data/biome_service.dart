/// Biome service: resolves lat/lon → [BiomeSample] via cached Overpass data.
///
/// Cache is coarse-tile-keyed (shared_preferences). On cache miss,
/// [prefetch] triggers a debounced background Overpass fetch for the tile.
/// [sampleAt] is synchronous: POI override → polygon containment → null.
library;

import 'dart:async';
import 'dart:convert';

import 'package:http/http.dart' as http;
import 'package:shared_preferences/shared_preferences.dart';

import '../domain/biome_def.dart';
import '../domain/micro_biome.dart';
import '../domain/osm_tags.dart';
import 'overpass.dart';

/// Coarse-tile resolution in degrees (~1.1 km at mid-latitudes).
const double kCacheTileDegrees = 0.01;

/// Minimum interval between Overpass fetches for the same tile (debounce).
const Duration kPrefetchDebounce = Duration(seconds: 60);

String _tileId(double lat, double lon) {
  final tx = (lon / kCacheTileDegrees).floor();
  final ty = (lat / kCacheTileDegrees).floor();
  return 'biome_${tx}_$ty';
}

/// Cached region entry: polygon boundary + OSM tag.
class _CachedRegion {
  _CachedRegion({
    required this.geometry,
    required this.osmKey,
    required this.osmValue,
  });
  final List<OverpassPoint> geometry;
  final String osmKey;
  final String osmValue;

  Map<String, dynamic> toJson() => {
        'g': geometry.map((p) => [p.lat, p.lon]).toList(),
        'k': osmKey,
        'v': osmValue,
      };

  factory _CachedRegion.fromJson(Map<String, dynamic> j) => _CachedRegion(
        geometry: (j['g'] as List)
            .map((e) {
              final arr = e as List;
              return OverpassPoint(
                  lat: (arr[0] as num).toDouble(),
                  lon: (arr[1] as num).toDouble());
            })
            .toList(),
        osmKey: j['k'] as String,
        osmValue: j['v'] as String,
      );
}

/// Cached POI entry: position + category.
class _CachedPoi {
  _CachedPoi({
    required this.lat,
    required this.lon,
    required this.category,
  });
  final double lat;
  final double lon;
  final MicroBiomeCategory category;

  Map<String, dynamic> toJson() => {
        't': [lat, lon],
        'c': category.index,
      };

  factory _CachedPoi.fromJson(Map<String, dynamic> j) => _CachedPoi(
        lat: ((j['t'] as List)[0] as num).toDouble(),
        lon: ((j['t'] as List)[1] as num).toDouble(),
        category: MicroBiomeCategory.values[(j['c'] as num).toInt()],
      );
}

/// Per-tile cache payload.
class _TileCache {
  _TileCache({required this.regions, required this.pois});
  final List<_CachedRegion> regions;
  final List<_CachedPoi> pois;
}

/// Point-in-polygon test via ray casting. [testLat] is Y, [testLon] is X.
bool _pointInPolygon(double testLat, double testLon, List<OverpassPoint> polygon) {
  var inside = false;
  final n = polygon.length;
  for (var i = 0, j = n - 1; i < n; j = i++) {
    final yi = polygon[i].lat;
    final yj = polygon[j].lat;
    final xi = polygon[i].lon;
    final xj = polygon[j].lon;
    if ((yi > testLat) != (yj > testLat) &&
        testLon < (xj - xi) * (testLat - yi) / (yj - yi) + xi) {
      inside = !inside;
    }
  }
  return inside;
}

/// Maps OSM POI tags → [MicroBiomeCategory].
MicroBiomeCategory? _poiTagToCategory(Map<String, String> tags) {
  final amenity = tags['amenity'];
  if (amenity != null) {
    switch (amenity) {
      case 'fountain':
        return MicroBiomeCategory.waterFeature;
      case 'bench':
      case 'waste_basket':
        return MicroBiomeCategory.streetFurniture;
      default:
        break;
    }
  }
  final historic = tags['historic'];
  if (historic != null) {
    switch (historic) {
      case 'monument':
      case 'memorial':
        return MicroBiomeCategory.monument;
      default:
        break;
    }
  }
  final natural = tags['natural'];
  if (natural != null) {
    switch (natural) {
      case 'tree':
        return MicroBiomeCategory.greenery;
      case 'rock':
      case 'stone':
        return MicroBiomeCategory.geological;
      default:
        break;
    }
  }
  final tourism = tags['tourism'];
  if (tourism != null) {
    switch (tourism) {
      case 'artwork':
        return MicroBiomeCategory.monument;
      default:
        break;
    }
  }
  return null;
}

/// Singleton biome resolver. Holds in-memory cache; persists fetched tiles
/// via shared_preferences.
class BiomeService {
  BiomeService({http.Client? httpClient})
      : _http = httpClient ?? http.Client();

  /// Called whenever a new tile finishes loading from the network or prefs.
  /// Use this to mark the fog dirty and recolor cells with the new data.
  void Function()? onTileLoaded;

  final http.Client _http;
  final Map<String, _TileCache> _cache = {};
  final Map<String, DateTime> _lastFetch = {};
  Timer? _debounceTimer;
  String? _pendingTile;

  /// Synchronous biome sample at (lat, lon). Returns null when no cached
  /// data exists for the tile — caller should have called [prefetch] first.
  BiomeSample? sampleAt(double lat, double lon) {
    final tile = _tileId(lat, lon);
    final c = _cache[tile];
    if (c == null) return null;

    // 1. POI override — strongest signal.
    for (final poi in c.pois) {
      final sample = microBiomeAt(
        (lat: poi.lat, lon: poi.lon),
        (lat: lat, lon: lon),
        poi.category,
      );
      if (sample != null) return sample;
    }

    // 2. Polygon containment.
    for (final region in c.regions) {
      if (_pointInPolygon(lat, lon, region.geometry)) {
        return osmTagToBiome(region.osmKey, region.osmValue);
      }
    }

    // 3. No data for this point.
    return null;
  }

  /// Debounced background fetch for the tile covering (lat, lon). Does not
  /// re-fetch a tile fetched within [kPrefetchDebounce].
  Future<void> prefetch(double lat, double lon) async {
    final tile = _tileId(lat, lon);
    if (_cache.containsKey(tile)) return; // already in memory
    final last = _lastFetch[tile];
    if (last != null &&
        DateTime.now().difference(last) < kPrefetchDebounce) {
      return;
    }

    // Try loading from persisted cache first.
    final loaded = await _loadFromPrefs(tile);
    if (loaded != null) {
      _cache[tile] = loaded;
      onTileLoaded?.call();
      return;
    }

    // Debounce: only fire one fetch per event-loop turn.
    if (_debounceTimer != null && _pendingTile == tile) return;
    _pendingTile = tile;
    _debounceTimer?.cancel();
    _debounceTimer = Timer(const Duration(milliseconds: 200), () {
      _doFetch(tile, lat, lon);
    });
  }

  /// Pre-populate a tile cache entry. Used by tests to set up known biome
  /// regions and POIs without network calls.
  void seedTile(
    double lat,
    double lon, {
    List<({List<OverpassPoint> geometry, String osmKey, String osmValue})>
        regions = const [],
    List<({double lat, double lon, MicroBiomeCategory category})> pois =
        const [],
  }) {
    final tile = _tileId(lat, lon);
    _cache[tile] = _TileCache(
      regions: regions
          .map((r) => _CachedRegion(
                geometry: r.geometry,
                osmKey: r.osmKey,
                osmValue: r.osmValue,
              ))
          .toList(),
      pois: pois
          .map((p) => _CachedPoi(
                lat: p.lat,
                lon: p.lon,
                category: p.category,
              ))
          .toList(),
    );
  }

  Future<void> _doFetch(String tile, double lat, double lon) async {
    _lastFetch[tile] = DateTime.now();
    try {
      final url = buildOverpassUrl(lat: lat, lon: lon);
      final resp = await _http.get(Uri.parse(url));
      if (resp.statusCode != 200) return;
      final parsed = parseOverpassResponse(resp.body);

      final regions = <_CachedRegion>[];
      for (final r in parsed.regions) {
        for (final tagKey in r.tags.keys) {
          final tagVal = r.tags[tagKey]!;
          if (osmTagToBiome(tagKey, tagVal) != null) {
            regions.add(_CachedRegion(
              geometry: r.geometry,
              osmKey: tagKey,
              osmValue: tagVal,
            ));
            break; // first matching tag per region
          }
        }
      }

      final pois = <_CachedPoi>[];
      for (final p in parsed.pois) {
        final cat = _poiTagToCategory(p.tags);
        if (cat != null) {
          pois.add(_CachedPoi(lat: p.lat, lon: p.lon, category: cat));
        }
      }

      final tc = _TileCache(regions: regions, pois: pois);
      _cache[tile] = tc;
      await _saveToPrefs(tile, tc);
      onTileLoaded?.call();
    } catch (_) {
      // Network errors are non-fatal — retry on next prefetch.
    }
  }

  // ---------------------------------------------------------------------------
  // Persistence helpers
  // ---------------------------------------------------------------------------

  static const String _prefPrefix = 'biome_tile_';

  Future<void> _saveToPrefs(String tile, _TileCache tc) async {
    try {
      final prefs = await SharedPreferences.getInstance();
      final json = jsonEncode({
        'r': tc.regions.map((r) => r.toJson()).toList(),
        'p': tc.pois.map((p) => p.toJson()).toList(),
      });
      await prefs.setString('$_prefPrefix$tile', json);
    } catch (_) {
      // shared_preferences write failure is non-fatal.
    }
  }

  Future<_TileCache?> _loadFromPrefs(String tile) async {
    try {
      final prefs = await SharedPreferences.getInstance();
      final raw = prefs.getString('$_prefPrefix$tile');
      if (raw == null) return null;
      final j = jsonDecode(raw) as Map<String, dynamic>;
      return _TileCache(
        regions: (j['r'] as List)
            .map((e) => _CachedRegion.fromJson(e as Map<String, dynamic>))
            .toList(),
        pois: (j['p'] as List)
            .map((e) => _CachedPoi.fromJson(e as Map<String, dynamic>))
            .toList(),
      );
    } catch (_) {
      return null;
    }
  }
}
