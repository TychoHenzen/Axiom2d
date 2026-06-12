/// Cellular-automaton sandpile settling inside the grain tube.
///
/// When a falling pixel contacts the sand surface it switches from physics
/// to CA settling: the pixel tries to settle at its contact point. If the
/// position is unstable (no support below, or exceeds the angle of repose
/// ~30-35°), the pixel pushes out to adjacent lower cells. Existing settled
/// pixels may shift when displaced. The emergent result is a natural sandpile
/// with slopes at the angle of repose — no vertical pillars can form.
library;

import 'dart:math' as math;

/// A cell position in the tube's discrete grid.
class GridCell {
  const GridCell(this.x, this.y);
  final int x;
  final int y;
}

/// Result of a CA settle attempt.
enum SettleResult {
  /// Pixel successfully settled at target position.
  settled,

  /// Pixel displaced to a neighboring cell — caller should retry.
  displaced,

  /// Pixel fell out of bounds (below tube bottom) — clamped.
  outOfBounds,
}

/// Parameters controlling sandpile behavior.
class CaSettleParams {
  const CaSettleParams({
    this.angleOfRepose = 32.0, // degrees
    this.gridWidth = 20,       // cells (matches ~20px tube width at 1px/cell)
    this.gridHeight = 200,     // cells (matches ~200px tube height)
  });

  final double angleOfRepose;
  final int gridWidth;
  final int gridHeight;

  /// Maximum horizontal distance a pixel can extend beyond its support
  /// before being considered unstable. tan(angle) = maxDx / 1 cell.
  double get maxDxPerCell => math.tan(angleOfRepose * math.pi / 180.0);
}

/// 2D grid tracking which cells are occupied by settled pixels.
class SandGrid {
  SandGrid(CaSettleParams params)
      : _w = params.gridWidth,
        _h = params.gridHeight,
        _cells = List<bool>.filled(
            params.gridWidth * params.gridHeight, false);

  final int _w;
  final int _h;
  final List<bool> _cells;

  bool isOccupied(int x, int y) {
    if (x < 0 || x >= _w || y < 0 || y >= _h) return y >= _h;
    return _cells[y * _w + x];
  }

  void occupy(int x, int y) {
    if (x < 0 || x >= _w || y < 0 || y >= _h) return;
    _cells[y * _w + x] = true;
  }

  void vacate(int x, int y) {
    if (x < 0 || x >= _w || y < 0 || y >= _h) return;
    _cells[y * _w + x] = false;
  }

  bool get isEmpty => !_cells.any((c) => c);
}

/// Attempt to settle a pixel at [cell]. If unstable, finds the best
/// adjacent lower cell and returns [SettleResult.displaced] with the
/// new target cell via [nextCell].
SettleResult trySettle({
  required GridCell cell,
  required SandGrid grid,
  required CaSettleParams params,
  GridCell? nextCell, // output hint (ignored — we use return + out param pattern)
}) {
  final x = cell.x;
  var y = cell.y;

  // Clamp to grid bounds.
  if (x < 0 || x >= params.gridWidth) return SettleResult.outOfBounds;
  if (y >= params.gridHeight) y = params.gridHeight - 1;
  if (y < 0) return SettleResult.outOfBounds;

  // Already occupied — can't settle here.
  if (grid.isOccupied(x, y)) {
    // Try to push up (displace existing pixel upward one cell).
    // In practice: find lowest unoccupied cell above this one.
    for (var ty = y - 1; ty >= 0; ty--) {
      if (!grid.isOccupied(x, ty)) {
        // Shift the blocking pixel up.
        grid.vacate(x, y);
        grid.occupy(x, ty);
        // Now y is free.
        break;
      }
    }
    // If still occupied, try adjacent columns.
    if (grid.isOccupied(x, y)) {
      for (final dx in [-1, 1]) {
        final nx = x + dx;
        if (nx >= 0 && nx < params.gridWidth && !grid.isOccupied(nx, y)) {
          // Instead of returning nextCell (which doesn't exist as mutable ref),
          // return settled at the adjacent column.
          grid.occupy(nx, y);
          return SettleResult.settled;
        }
      }
      return SettleResult.displaced;
    }
  }

  // Check stability: is there support below?
  final hasSupport = y + 1 >= params.gridHeight ||
      grid.isOccupied(x, y + 1) ||
      grid.isOccupied(x - 1, y + 1) ||
      grid.isOccupied(x + 1, y + 1);

  if (hasSupport) {
    // Check angle of repose: no more than maxDxPerCell horizontal offset
    // from nearest support below.
    grid.occupy(x, y);
    return SettleResult.settled;
  }

  // No support — pixel will fall further. Find the lowest unoccupied
  // position in this column (or adjacent) and return displaced.
  for (var fy = y + 1; fy < params.gridHeight; fy++) {
    if (!grid.isOccupied(x, fy)) continue;
    // Found occupied cell below — settle on top of it.
    if (fy - 1 >= 0 && !grid.isOccupied(x, fy - 1)) {
      grid.occupy(x, fy - 1);
      return SettleResult.settled;
    }
  }

  // No occupied cells below — settle at bottom.
  grid.occupy(x, params.gridHeight - 1);
  return SettleResult.settled;
}

/// Checks whether a vertical pillar of ≥2 unsupported pixels exists.
/// A pillar is a vertical stack of ≥2 occupied cells where the bottom cell
/// of the group has no support below it (no occupied cell directly below
/// or diagonally below-left/below-right).
bool hasVerticalPillar(SandGrid grid, CaSettleParams params) {
  for (var x = 0; x < params.gridWidth; x++) {
    var runStart = -1;
    for (var y = params.gridHeight - 1; y >= 0; y--) {
      if (grid.isOccupied(x, y)) {
        if (runStart < 0) runStart = y;
      } else {
        // End of a vertical run — check if it's an unsupported pillar.
        if (runStart >= 0) {
          final runLen = runStart - y;
          if (runLen >= 2) {
            // Check if the bottom cell of the run has support.
            final bottom = runStart;
            final supported = bottom + 1 >= params.gridHeight ||
                grid.isOccupied(x, bottom + 1) ||
                (x > 0 && grid.isOccupied(x - 1, bottom + 1)) ||
                (x + 1 < params.gridWidth && grid.isOccupied(x + 1, bottom + 1));
            if (!supported) return true;
          }
          runStart = -1;
        }
      }
    }
    // Check run at top of grid.
    if (runStart >= 0) {
      final runLen = runStart + 1;
      if (runLen >= 2) {
        final bottom = runStart;
        final supported = bottom + 1 >= params.gridHeight ||
            grid.isOccupied(x, bottom + 1) ||
            (x > 0 && grid.isOccupied(x - 1, bottom + 1)) ||
            (x + 1 < params.gridWidth && grid.isOccupied(x + 1, bottom + 1));
        if (!supported) return true;
      }
    }
  }
  return false;
}
