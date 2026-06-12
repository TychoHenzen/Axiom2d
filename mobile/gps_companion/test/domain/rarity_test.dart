import 'package:flutter_test/flutter_test.dart';
import 'package:gps_companion/domain/rarity.dart';

void main() {
  group('geometricLevel (port of Rust geometric_level)', () {
    test('when_rate_zero_then_always_returns_first_level', () {
      for (final v in [0.0, 0.1, 0.5, 0.9, 1.0]) {
        expect(geometricLevel(v, 0.0, 5), 0);
      }
    });

    test('when_rate_one_then_always_returns_max_level', () {
      for (final v in [0.0, 0.25, 0.5, 0.75, 0.99]) {
        expect(geometricLevel(v, 1.0, 5), 4);
      }
    });

    test('when_value_below_rate_then_advances_past_first_level', () {
      // value 0.1 < rate 0.5 → must climb above level 0.
      expect(geometricLevel(0.1, 0.5, 5), greaterThan(0));
    });

    test('when_value_at_or_above_rate_then_stays_at_level_zero', () {
      expect(geometricLevel(0.7, 0.5, 5), 0);
    });

    test('when_value_increases_then_level_is_monotonically_non_increasing', () {
      var prev = 5;
      for (var i = 0; i <= 100; i++) {
        final level = geometricLevel(i / 100.0, 0.3, 5);
        expect(level, lessThanOrEqualTo(prev));
        prev = level;
      }
    });
  });

  group('grainRarity', () {
    test('is_deterministic_for_same_axes_and_seed', () {
      final axes = [0.02, -0.01, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
      expect(grainRarity(axes, 42), grainRarity(axes, 42));
    });

    test('different_seeds_can_yield_different_rarities', () {
      final axes = [0.05, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
      final results = {for (var s = 0; s < 200; s++) grainRarity(axes, s)};
      // Over 200 seeds we should observe more than one rarity tier.
      expect(results.length, greaterThan(1));
    });
  });
}
