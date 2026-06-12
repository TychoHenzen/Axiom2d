/// Collection + forging. Collecting 100 grains auto-forges a booster pack;
/// leftover grains spill over toward the next pack. See spec "Collection & Forging".
library;

import 'grain.dart';

/// Number of grains required to forge one booster pack.
const int kGrainsPerBooster = 100;

/// A forged booster pack: exactly [kGrainsPerBooster] grains plus provenance.
class Booster {
  Booster({
    required this.grains,
    required this.forgedAt,
    this.lat,
    this.lon,
    this.locationName,
  });

  final List<Grain> grains;
  final DateTime forgedAt;
  final double? lat;
  final double? lon;
  final String? locationName;

  Map<String, dynamic> toJson() => {
    'forged_at': forgedAt.toUtc().toIso8601String(),
    if (lat != null && lon != null) 'location': {'lat': lat, 'lon': lon},
    if (locationName != null) 'location_name': locationName,
    'grains': grains.map((g) => g.toJson()).toList(),
  };

  static Booster fromJson(Map<String, dynamic> j) {
    final loc = j['location'] as Map<String, dynamic>?;
    return Booster(
      forgedAt: DateTime.parse(j['forged_at'] as String),
      lat: loc == null ? null : (loc['lat'] as num).toDouble(),
      lon: loc == null ? null : (loc['lon'] as num).toDouble(),
      locationName: j['location_name'] as String?,
      grains: (j['grains'] as List)
          .map((e) => Grain.fromJson(e as Map<String, dynamic>))
          .toList(),
    );
  }
}

/// Cosmetic theming for a booster wrapper, derived from its grain mix.
class BoosterTheme {
  BoosterTheme({
    required this.dominantType,
    required this.dominantRatio,
    required this.isMixed,
    required this.isPure,
    required this.legendaryCount,
  });

  final GrainType dominantType;
  final double dominantRatio;
  final bool isMixed; // no type reaches 50%
  final bool isPure; // a type reaches 90% -> guarantees Rare+ card
  final int legendaryCount;

  bool get hasGoldShimmer => legendaryCount >= 5;

  static BoosterTheme of(List<Grain> grains) {
    final counts = <GrainType, int>{};
    var legendary = 0;
    for (final g in grains) {
      counts[g.type] = (counts[g.type] ?? 0) + 1;
      if (g.rarity == GrainRarity.legendary) legendary++;
    }
    var dominant = GrainType.nature;
    var dominantCount = -1;
    counts.forEach((t, c) {
      if (c > dominantCount) {
        dominantCount = c;
        dominant = t;
      }
    });
    final total = grains.isEmpty ? 1 : grains.length;
    final ratio = dominantCount <= 0 ? 0.0 : dominantCount / total;
    return BoosterTheme(
      dominantType: dominant,
      dominantRatio: ratio,
      isMixed: ratio < 0.5,
      isPure: ratio >= 0.9,
      legendaryCount: legendary,
    );
  }
}

/// Mutable collection state. Persistence is handled separately (data/store).
class Inventory {
  Inventory({List<Grain>? loose, List<Booster>? boosters})
    : loose = loose ?? [],
      boosters = boosters ?? [];

  /// Grains collected but not yet forged into a pack.
  final List<Grain> loose;

  /// Forged booster packs (no limit).
  final List<Booster> boosters;

  int get grainCount => loose.length;

  /// Collect one grain. Auto-forges a booster every time loose reaches 100,
  /// spilling any remainder into the next pack. Returns boosters forged (0+).
  ///
  /// [now] and provenance are injected so forging is deterministic and testable.
  int collect(
    Grain grain, {
    required DateTime now,
    double? lat,
    double? lon,
    String? locationName,
  }) {
    loose.add(grain);
    return _forge(now: now, lat: lat, lon: lon, locationName: locationName);
  }

  /// Collect many grains at once (the continuous brush harvests a batch per
  /// step). Forges as many boosters as the new total allows; returns the count.
  int collectAll(
    List<Grain> grains, {
    required DateTime now,
    double? lat,
    double? lon,
    String? locationName,
  }) {
    if (grains.isEmpty) return 0;
    loose.addAll(grains);
    return _forge(now: now, lat: lat, lon: lon, locationName: locationName);
  }

  int _forge({
    required DateTime now,
    double? lat,
    double? lon,
    String? locationName,
  }) {
    var forged = 0;
    while (loose.length >= kGrainsPerBooster) {
      final packGrains = loose.sublist(0, kGrainsPerBooster);
      loose.removeRange(0, kGrainsPerBooster);
      boosters.add(
        Booster(
          grains: packGrains,
          forgedAt: now,
          lat: lat,
          lon: lon,
          locationName: locationName,
        ),
      );
      forged++;
    }
    return forged;
  }
}
