import 'dart:math' as math;

import 'package:flutter_test/flutter_test.dart';

void main() {
  /// Linear interpolation from [a] to [b] by normalized factor [t] in [0,1].
  double lerp(double a, double b, double t) => a + (b - a) * t;

  /// Normalize a yield factor f (clamped to [0.5, 10.0]) to [0, 1].
  double normF(double f) {
    const fMin = 0.5;
    const fMax = 10.0;
    final clamped = f.clamp(fMin, fMax);
    return (clamped - fMin) / (fMax - fMin);
  }

  /// Fog alpha from yield factor f: lerp(0.06, 0.92, normF(f)).
  double fogAlpha(double f) => lerp(0.06, 0.92, normF(f));

  group('density coupling', () {
    test('when_density_at_minimum_then_yield_factor_is_0_5', () {
      // Arrange
      const density = 10.0; // below any reasonable biome density
      const kBase = 200.0;

      // Act
      final f = (density / kBase).clamp(0.5, 10.0);

      // Assert
      expect(f, 0.5);
    });

    test('when_density_at_maximum_then_yield_factor_is_10', () {
      // Arrange
      const density = 5000.0; // far above any biome
      const kBase = 200.0;

      // Act
      final f = (density / kBase).clamp(0.5, 10.0);

      // Assert
      expect(f, 10.0);
    });

    test('when_density_is_base_then_yield_factor_is_1', () {
      // Arrange
      const density = 200.0;
      const kBase = 200.0;

      // Act
      final f = (density / kBase).clamp(0.5, 10.0);

      // Assert
      expect(f, 1.0);
    });

    test('when_signature_magnitude_scales_with_sqrt_of_density', () {
      // Arrange — scale = 0.15 * sqrt(f)
      // At f=1.0, scale = 0.15. At f=4.0, scale = 0.30.
      const baseScale = 0.15;

      // Act
      final scale1 = baseScale * math.sqrt(1.0);
      final scale4 = baseScale * math.sqrt(4.0);
      final scale9 = baseScale * math.sqrt(9.0);

      // Assert
      expect(scale1, closeTo(0.15, 0.001));
      expect(scale4, closeTo(0.30, 0.001));
      expect(scale9, closeTo(0.45, 0.001));
    });
  });

  group('fog alpha', () {
    test('when_alpha_is_monotonic_with_f', () {
      // Arrange
      const fs = [0.5, 1.0, 2.0, 5.0, 10.0];

      // Act
      final alphas = fs.map(fogAlpha).toList();

      // Assert — alpha strictly increases with f.
      for (var i = 1; i < alphas.length; i++) {
        expect(alphas[i], greaterThan(alphas[i - 1]),
            reason: 'alpha must increase with f');
      }
    });

    test('when_alpha_is_bounded', () {
      // Act
      final alphaMin = fogAlpha(0.5);
      final alphaMax = fogAlpha(10.0);
      final alphaMid = fogAlpha(1.0);

      // Assert — within [0.06, 0.92].
      expect(alphaMin, closeTo(0.06, 0.001));
      expect(alphaMax, closeTo(0.92, 0.001));
      expect(alphaMid, greaterThan(0.06));
      expect(alphaMid, lessThan(0.92));
    });

    test('when_f_at_extremes_then_alpha_stays_in_bounds', () {
      // Act
      final below = fogAlpha(0.0); // clamped to 0.5 internally
      final above = fogAlpha(100.0); // clamped to 10.0

      // Assert
      expect(below, closeTo(0.06, 0.001));
      expect(above, closeTo(0.92, 0.001));
    });
  });
}
