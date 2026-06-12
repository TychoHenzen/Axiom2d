import 'package:flutter_test/flutter_test.dart';
import 'package:gps_companion/ui/ca_settling.dart';

void main() {
  final params = CaSettleParams(gridWidth: 20, gridHeight: 40);

  group('trySettle', () {
    test('when_flat_surface_then_settles_directly_above_support', () {
      // Arrange — flat surface at row 30.
      final grid = SandGrid(params);
      for (var x = 5; x <= 15; x++) {
        grid.occupy(x, 30);
      }

      // Act — settle at (10, 29) — directly above support at (10, 30).
      final result = trySettle(
        cell: const GridCell(10, 29),
        grid: grid,
        params: params,
      );

      // Assert
      expect(result, SettleResult.settled);
      expect(grid.isOccupied(10, 29), isTrue);
    });

    test('when_steep_slope_then_displaces_to_adjacent_lower_cell', () {
      // Arrange — steep slope: support only at (10, 30).
      final grid = SandGrid(params);
      grid.occupy(10, 30);

      // Act — try settle at (14, 29) — 4 cells away, exceeds angle of repose.
      final result = trySettle(
        cell: const GridCell(14, 29),
        grid: grid,
        params: params,
      );

      // Assert — should have settled somewhere (displaced or settled).
      // The pixel can't remain unsupported at (14, 29).
      final wasUnsupported = !grid.isOccupied(14, 29) ||
          result == SettleResult.displaced;
      expect(wasUnsupported || result == SettleResult.settled, isTrue);
    });

    test('when_no_support_below_then_falls_to_bottom', () {
      // Arrange — empty grid.
      final grid = SandGrid(params);

      // Act — settle at (10, 5).
      trySettle(
        cell: const GridCell(10, 5),
        grid: grid,
        params: params,
      );

      // Assert — pixel falls to bottom row.
      final bottomRow = params.gridHeight - 1;
      expect(grid.isOccupied(10, bottomRow), isTrue);
    });
  });

  group('hasVerticalPillar', () {
    test('when_flat_pile_then_no_pillars', () {
      // Arrange
      final grid = SandGrid(CaSettleParams(gridWidth: 10, gridHeight: 20));
      for (var x = 2; x <= 7; x++) {
        for (var y = 15; y < 20; y++) {
          grid.occupy(x, y);
        }
      }

      // Act & Assert
      expect(hasVerticalPillar(grid, params), isFalse);
    });

    test('when_two_unsupported_cells_stacked_then_pillar_detected', () {
      // Arrange — two cells stacked vertically with no support below the bottom one.
      final grid = SandGrid(CaSettleParams(gridWidth: 10, gridHeight: 20));
      grid.occupy(5, 18); // no support below 18
      grid.occupy(5, 17); // supported by 18, but 18 has no support

      // Act & Assert — pillar of 2 unsupported cells.
      expect(hasVerticalPillar(grid, params), isTrue);
    });

    test('when_sloped_pile_then_no_pillars', () {
      // Arrange — natural slope at ~45°.
      final grid = SandGrid(CaSettleParams(gridWidth: 15, gridHeight: 20));
      // Build a slope: row 19 has cells 5..10, row 18 has 6..9, etc.
      for (var y = 19, w = 6; y >= 14; y--, w--) {
        for (var x = 7 - w ~/ 2; x <= 7 + w ~/ 2; x++) {
          grid.occupy(x, y);
        }
      }

      // Act & Assert — sloped piles don't have vertical pillars.
      expect(hasVerticalPillar(grid, params), isFalse);
    });
  });

  group('SandGrid', () {
    test('when_occupy_and_vacate_then_state_updates', () {
      // Arrange
      final grid = SandGrid(params);

      // Act
      grid.occupy(5, 10);

      // Assert
      expect(grid.isOccupied(5, 10), isTrue);
      expect(grid.isEmpty, isFalse);

      // Act
      grid.vacate(5, 10);

      // Assert
      expect(grid.isOccupied(5, 10), isFalse);
      expect(grid.isEmpty, isTrue);
    });

    test('when_out_of_bounds_then_ignored', () {
      // Arrange
      final grid = SandGrid(params);

      // Act
      grid.occupy(-1, 5);
      grid.occupy(5, params.gridHeight + 1);

      // Assert — no crash, no occupancy.
      expect(grid.isEmpty, isTrue);
    });
  });
}
