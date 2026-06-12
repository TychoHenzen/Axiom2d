/// Rarity assignment — faithful port of `geometric_level` from the desktop
/// `card_game::CardSignature` pipeline (see signature/algorithms.rs).
library;

import 'grain.dart';

/// Default rarity advance rate (matches Rust `RarityTierConfig::default`).
const double kRarityAdvanceRate = 0.3;

/// Number of rarity levels (Common..Legendary).
const int kRarityLevels = 5;

/// Port of Rust `geometric_level(value, advance_rate, max_levels)`.
///
/// Returns a level in `0..max_levels`. Higher `value` (closer to the advance
/// rate) keeps the level low; small values cascade up to higher levels.
int geometricLevel(double value, double advanceRate, int maxLevels) {
  var remaining = value;
  for (var level = 0; level < maxLevels - 1; level++) {
    if (remaining >= advanceRate) return level;
    remaining /= advanceRate;
  }
  return maxLevels - 1;
}

/// Deterministic hash of the axes plus a seed into the unit interval `[0, 1)`.
///
/// Mobile-only: grain rarity is cosmetic (the desktop recomputes the real
/// rarity from the summed signature), so this need not match the Rust f32 hash
/// bit-for-bit — only be deterministic and well-distributed.
double unitHash(List<double> axes, int seed) {
  var h = 0x9e3779b97f4a7c15 ^ (seed & 0x7fffffffffffffff);
  for (final a in axes) {
    // Quantize the float to a stable integer before mixing.
    final q = (a * 1e6).round();
    h ^= q & 0xffffffffffffffff;
    h = (h * 0x100000001b3) & 0x7fffffffffffffff;
    h ^= h >> 29;
  }
  // Map the low 53 bits to [0, 1).
  return (h & 0x1fffffffffffff) / 0x20000000000000;
}

/// Rarity for a grain with the given axes, seeded deterministically.
GrainRarity grainRarity(List<double> axes, int seed) {
  final level = geometricLevel(unitHash(axes, seed), kRarityAdvanceRate, kRarityLevels);
  return GrainRarity.fromLevel(level);
}
