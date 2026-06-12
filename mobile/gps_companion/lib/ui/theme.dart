/// Visual mapping of grain types and rarities to colours/sizes.
library;

import 'package:flutter/material.dart';

import '../domain/grain.dart';

/// Representative colour per grain type (spec wrapper-theming palette).
Color grainColor(GrainType type) => switch (type) {
  GrainType.nature => const Color(0xFF4CAF50), // green
  GrainType.urban => const Color(0xFF9E9E9E), // gray
  GrainType.water => const Color(0xFF2196F3), // blue
  GrainType.earth => const Color(0xFF795548), // brown
  GrainType.arcane => const Color(0xFF9C27B0), // purple
  GrainType.febris => const Color(0xFFF44336), // red (Heat/Cold)
  GrainType.lumines => const Color(0xFFEEEEEE), // white-ish
  GrainType.inertiae => const Color(0xFFFF9800), // orange
};

/// Marker radius in logical pixels, larger for rarer grains.
double rarityRadius(GrainRarity rarity) => switch (rarity) {
  GrainRarity.common => 10,
  GrainRarity.uncommon => 13,
  GrainRarity.rare => 16,
  GrainRarity.epic => 20,
  GrainRarity.legendary => 26,
};
