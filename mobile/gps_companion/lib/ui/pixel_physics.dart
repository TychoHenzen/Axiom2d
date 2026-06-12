/// Simple kinematic pixel physics for grain particles falling inside the
/// glass test-tube. Handles gravity, wall collision, and sand-surface
/// contact detection. When a pixel contacts the sand surface it switches
/// to CA settling phase.
library;

import '../domain/grain_pixel.dart';

/// Phase of a grain pixel inside the tube.
enum PixelPhase {
  /// Falling under gravity toward the sand surface.
  falling,

  /// Transitioning: CA settling in progress.
  settling,

  /// Locked in final position.
  settled,
}

/// A single grain pixel inside the tube with physics state.
class TubePixel {
  TubePixel({
    required this.x,
    required this.y,
    required this.color,
    this.phase = PixelPhase.falling,
    this.vx = 0.0,
    this.vy = 0.0,
  });

  double x;
  double y;
  double vx;
  double vy;
  AspectColor color;
  PixelPhase phase;
}

/// Physics parameters for the tube simulation.
class TubePhysics {
  const TubePhysics({
    this.gravity = 300.0, // px/s²
    this.tubeWidth = 20.0,
    this.tubeHeight = 200.0,
    this.wallDamping = 0.2, // velocity retained on wall bounce
    this.settleThreshold = 0.5, // |vy| below this → check settling
  });

  final double gravity;
  final double tubeWidth;
  final double tubeHeight;
  final double wallDamping;
  final double settleThreshold;
}

/// Simulate one physics step for a falling pixel.
///
/// Returns true if the pixel transitioned to [PixelPhase.settling] (hit sand
/// surface), false if still falling or already settled.
bool stepFalling(TubePixel pixel, TubePhysics phys, double dt) {
  if (pixel.phase != PixelPhase.falling) return false;

  // Gravity.
  pixel.vy += phys.gravity * dt;

  // Integrate position.
  pixel.x += pixel.vx * dt;
  pixel.y += pixel.vy * dt;

  // Wall collision — left wall.
  if (pixel.x < 0) {
    pixel.x = 0;
    pixel.vx = -pixel.vx * phys.wallDamping;
  }
  // Wall collision — right wall.
  if (pixel.x > phys.tubeWidth) {
    pixel.x = phys.tubeWidth;
    pixel.vx = -pixel.vx * phys.wallDamping;
  }

  // Bottom of tube.
  if (pixel.y > phys.tubeHeight) {
    pixel.y = phys.tubeHeight;
    pixel.vy = 0;
  }

  return false;
}

/// Check whether [pixel] has contacted the top surface of the settled sand
/// mass. [settledPixels] is the list of already-settled pixels (CA lock).
/// [cellSize] is the approximate pixel diameter (~1-2 px).
///
/// Returns the y-coordinate of the sand surface at pixel.x, or null if
/// no surface exists at that x.
double? sandSurfaceAt(double x, List<TubePixel> settledPixels, double cellSize) {
  double? top;
  for (final p in settledPixels) {
    final dx = (p.x - x).abs();
    if (dx < cellSize * 1.5) {
      if (top == null || p.y < top) {
        top = p.y;
      }
    }
  }
  return top;
}

/// Determine if the pixel should switch from falling to settling.
/// Triggers when the pixel is close to the sand surface or tube bottom
/// and its downward velocity is low enough.
bool shouldBeginSettling(
  TubePixel pixel,
  List<TubePixel> settledPixels,
  TubePhysics phys,
  double cellSize,
) {
  if (pixel.phase != PixelPhase.falling) return false;
  if (pixel.vy.abs() > phys.settleThreshold) return false;

  // Check against sand surface.
  final surface = sandSurfaceAt(pixel.x, settledPixels, cellSize);
  if (surface != null && pixel.y >= surface - cellSize) return true;

  // Check against tube bottom.
  if (pixel.y >= phys.tubeHeight - cellSize) return true;

  return false;
}

/// Lock a pixel into its final settled position (velocity zero).
void lockSettled(TubePixel pixel) {
  pixel.vx = 0;
  pixel.vy = 0;
  pixel.phase = PixelPhase.settled;
}
