/// Base biome layer (spec Layer 1). Each biome yields a grain-type
/// distribution over the five terrestrial types; leyline types come only from
/// the procedural overlay (see leyline.dart), never from the base biome.
library;

import 'grain.dart';

enum Biome {
  forestPark,
  urbanResidential,
  waterCoast,
  mountainDesert,
  historicCultural,
  noData;

  /// Grains per km² per day for this biome (spec "Spawn Algorithm").
  double get densityPerKm2 => switch (this) {
    Biome.urbanResidential => 200,
    Biome.forestPark => 300,
    Biome.waterCoast => 150,
    Biome.mountainDesert => 100,
    Biome.historicCultural => 80,
    Biome.noData => 120,
  };
}

/// Terrestrial grain-type weights per biome (spec Layer 1 table). Sums to 1.0.
const Map<Biome, Map<GrainType, double>> kBiomeDistribution = {
  Biome.forestPark: {
    GrainType.nature: 0.65,
    GrainType.urban: 0.05,
    GrainType.water: 0.10,
    GrainType.earth: 0.10,
    GrainType.arcane: 0.10,
  },
  Biome.urbanResidential: {
    GrainType.nature: 0.10,
    GrainType.urban: 0.60,
    GrainType.water: 0.05,
    GrainType.earth: 0.10,
    GrainType.arcane: 0.15,
  },
  Biome.waterCoast: {
    GrainType.nature: 0.15,
    GrainType.urban: 0.05,
    GrainType.water: 0.55,
    GrainType.earth: 0.10,
    GrainType.arcane: 0.15,
  },
  Biome.mountainDesert: {
    GrainType.nature: 0.05,
    GrainType.urban: 0.05,
    GrainType.water: 0.05,
    GrainType.earth: 0.70,
    GrainType.arcane: 0.15,
  },
  Biome.historicCultural: {
    GrainType.nature: 0.10,
    GrainType.urban: 0.15,
    GrainType.water: 0.05,
    GrainType.earth: 0.05,
    GrainType.arcane: 0.65,
  },
  Biome.noData: {
    GrainType.nature: 0.25,
    GrainType.urban: 0.25,
    GrainType.water: 0.15,
    GrainType.earth: 0.20,
    GrainType.arcane: 0.15,
  },
};

/// Weighted pick of a grain type from a distribution given a uniform `u` in
/// `[0, 1)`. Deterministic for a given `u`.
GrainType pickFromDistribution(Map<GrainType, double> dist, double u) {
  var acc = 0.0;
  GrainType last = dist.keys.first;
  for (final entry in dist.entries) {
    acc += entry.value;
    last = entry.key;
    if (u < acc) return entry.key;
  }
  return last; // rounding fallthrough
}
