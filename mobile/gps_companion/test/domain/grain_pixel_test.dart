import 'package:flutter_test/flutter_test.dart';
import 'package:gps_companion/domain/grain.dart';
import 'package:gps_companion/domain/grain_pixel.dart';

void main() {
  group('dominantAxisIndex', () {
    test('when_all_axes_zero_then_returns_zero', () {
      // Arrange
      final axes = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];

      // Act
      final result = dominantAxisIndex(axes);

      // Assert
      expect(result, 0);
    });

    test('when_one_axis_dominant_then_returns_that_index', () {
      // Arrange
      final axes = [0.01, -0.02, 0.15, -0.03, 0.01, 0.0, 0.0, 0.0];

      // Act
      final result = dominantAxisIndex(axes);

      // Assert — axis 2 has the largest absolute value (0.15).
      expect(result, 2);
    });

    test('when_negative_dominant_then_returns_correct_index', () {
      // Arrange
      final axes = [0.01, -0.5, 0.02, 0.01, 0.0, 0.0, 0.0, 0.0];

      // Act
      final result = dominantAxisIndex(axes);

      // Assert — axis 1 has |−0.5| = 0.5, the largest magnitude.
      expect(result, 1);
    });
  });

  group('grainPixelColor', () {
    test('when_positive_dominant_axis_then_returns_warm_color', () {
      // Arrange — grain with dominant positive axis 0 (earth → Solid/amber).
      final grain = Grain(
        axes: [0.15, 0.01, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        type: GrainType.earth,
        rarity: GrainRarity.common,
      );

      // Act
      final color = grainPixelColor(grain);

      // Assert — Solid = amber (r > g > b, warm).
      expect(color.r, closeTo(0.85, 0.001));
      expect(color.g, closeTo(0.55, 0.001));
      expect(color.b, closeTo(0.20, 0.001));
      expect(color.r, greaterThan(color.b)); // warm: r > b
    });

    test('when_negative_dominant_axis_then_returns_cool_color', () {
      // Arrange — grain with dominant negative axis 1 (febris → Cold/ice-blue).
      final grain = Grain(
        axes: [0.01, -0.15, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        type: GrainType.febris,
        rarity: GrainRarity.common,
      );

      // Act
      final color = grainPixelColor(grain);

      // Assert — Cold = ice blue (b > r, cool).
      expect(color.r, closeTo(0.10, 0.001));
      expect(color.g, closeTo(0.70, 0.001));
      expect(color.b, closeTo(0.95, 0.001));
      expect(color.b, greaterThan(color.r)); // cool: b > r
    });
  });

  group('aspectColorFor', () {
    test('when_all_16_combinations_then_distinct_and_match_desktop_palette', () {
      // Arrange — expected (r,g,b) for each (GrainType, sign) pair.
      // Matches desktop gem_sockets.rs aspect_color() via Element→Aspect table.
      const expected = <(GrainType, bool, double, double, double)>[
        // Warm (positive)
        (GrainType.earth, true, 0.85, 0.55, 0.20),     // Solid
        (GrainType.febris, true, 0.95, 0.25, 0.10),    // Heat
        (GrainType.urban, true, 0.90, 0.80, 0.10),     // Order
        (GrainType.lumines, true, 0.98, 0.95, 0.40),   // Light
        (GrainType.water, true, 0.70, 0.85, 0.10),     // Change
        (GrainType.inertiae, true, 0.90, 0.40, 0.05),  // Force
        (GrainType.nature, true, 0.20, 0.80, 0.20),    // Growth
        (GrainType.arcane, true, 0.60, 0.90, 0.30),    // Expansion
        // Cool (negative)
        (GrainType.earth, false, 0.30, 0.50, 0.85),    // Fragile
        (GrainType.febris, false, 0.10, 0.70, 0.95),   // Cold
        (GrainType.urban, false, 0.55, 0.10, 0.80),    // Chaos
        (GrainType.lumines, false, 0.15, 0.05, 0.40),  // Dark
        (GrainType.water, false, 0.20, 0.60, 0.80),    // Stasis
        (GrainType.inertiae, false, 0.10, 0.75, 0.70), // Calm
        (GrainType.nature, false, 0.35, 0.20, 0.60),   // Decay
        (GrainType.arcane, false, 0.05, 0.20, 0.70),   // Contraction
      ];

      // Assert — 16 entries.
      expect(expected.length, 16, reason: '8 types × 2 signs = 16 colors');

      // Act & Assert — each maps to correct color.
      for (final (type, positive, r, g, b) in expected) {
        final color = aspectColorFor(type, positive);
        expect(color.r, closeTo(r, 0.001),
            reason: '${type.name} ${positive ? "+" : "-"} r');
        expect(color.g, closeTo(g, 0.001),
            reason: '${type.name} ${positive ? "+" : "-"} g');
        expect(color.b, closeTo(b, 0.001),
            reason: '${type.name} ${positive ? "+" : "-"} b');
        // Warm check: positive → r > b or g > b; negative → b > r.
        if (positive) {
          final isWarm = color.r > color.b || color.g > color.b;
          expect(isWarm, isTrue,
              reason: '${type.name}+ should be warm');
        } else {
          expect(color.b, greaterThan(color.r),
              reason: '${type.name}- should be cool (b > r)');
        }
      }
    });

    test('when_all_16_colors_are_distinct', () {
      // Act
      final colors = allGrainColors;

      // Assert — 16 distinct colors.
      expect(colors.length, 16);
      final seen = <String>{};
      for (final entry in colors) {
        final key = '${entry.color.r},${entry.color.g},${entry.color.b}';
        expect(seen.contains(key), isFalse,
            reason: 'duplicate color for ${entry.type.name} ${entry.positive ? '+' : '-'}');
        seen.add(key);
      }
    });
  });
}
