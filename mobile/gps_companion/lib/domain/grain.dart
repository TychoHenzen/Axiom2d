/// Grain data model — a miniature 8-float `CardSignature` fragment.
///
/// Mirrors `docs/plans/gps_companion_app.md`. The mobile app never builds cards;
/// it only collects grains. The desktop game is the sole card factory.
library;

/// 8 elements, one per signature axis. Index = signature axis index.
enum GrainType {
  earth('Earth', 0),
  febris('Febris', 1),
  urban('Urban', 2),
  lumines('Lumines', 3),
  water('Water', 4),
  inertiae('Inertiae', 5),
  nature('Nature', 6),
  arcane('Arcane', 7);

  const GrainType(this.json, this.axis);

  /// PascalCase name used in the LAN transfer payload (desktop contract).
  final String json;

  /// Signature axis index this grain type maps to.
  final int axis;

  static GrainType fromJson(String s) =>
      values.firstWhere((t) => t.json == s, orElse: () => GrainType.nature);

  /// Grain type for a signature axis index (0..7).
  static GrainType forAxis(int axis) =>
      values.firstWhere((t) => t.axis == axis);

  /// The three leyline (overlay-only) types never spawn from base biome.
  bool get isLeyline =>
      this == GrainType.febris ||
      this == GrainType.lumines ||
      this == GrainType.inertiae;
}

enum GrainRarity {
  common('Common'),
  uncommon('Uncommon'),
  rare('Rare'),
  epic('Epic'),
  legendary('Legendary');

  const GrainRarity(this.json);

  final String json;

  static GrainRarity fromJson(String s) =>
      values.firstWhere((r) => r.json == s, orElse: () => GrainRarity.common);

  static GrainRarity fromLevel(int level) =>
      values[level.clamp(0, values.length - 1)];
}

/// Number of signature axes.
const int kAxisCount = 8;

/// A single collectible grain.
class Grain {
  Grain({required this.axes, required this.type, required this.rarity})
    : assert(axes.length == kAxisCount, 'grain must have $kAxisCount axes');

  final List<double> axes;
  final GrainType type;
  final GrainRarity rarity;

  /// Index of the dominant axis (largest magnitude).
  static int dominantAxis(List<double> axes) {
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

  Map<String, dynamic> toJson() => {
    'axes': axes,
    'grain_type': type.json,
    'rarity': rarity.json,
  };

  static Grain fromJson(Map<String, dynamic> j) => Grain(
    axes: (j['axes'] as List).map((e) => (e as num).toDouble()).toList(),
    type: GrainType.fromJson(j['grain_type'] as String),
    rarity: GrainRarity.fromJson(j['rarity'] as String),
  );
}
