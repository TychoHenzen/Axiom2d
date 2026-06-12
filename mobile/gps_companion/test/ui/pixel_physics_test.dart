import 'package:flutter_test/flutter_test.dart';
import 'package:gps_companion/domain/grain_pixel.dart';
import 'package:gps_companion/ui/pixel_physics.dart';

void main() {
  final phys = TubePhysics();

  group('stepFalling', () {
    test('when_pixel_falls_then_velocity_increases_downward', () {
      // Arrange
      final pixel = TubePixel(
        x: 10,
        y: 0,
        color: const AspectColor(0.5, 0.5, 0.5),
        vx: 0,
        vy: 0,
      );

      // Act
      stepFalling(pixel, phys, 0.016); // ~60fps frame

      // Assert — gravity pulls pixel down.
      expect(pixel.vy, greaterThan(0));
      expect(pixel.y, greaterThan(0));
    });

    test('when_multiple_frames_then_velocity_accumulates', () {
      // Arrange
      final pixel = TubePixel(
        x: 10,
        y: 0,
        color: const AspectColor(0.5, 0.5, 0.5),
      );
      final v0 = pixel.vy;

      // Act
      stepFalling(pixel, phys, 0.016);
      final v1 = pixel.vy;
      stepFalling(pixel, phys, 0.016);
      final v2 = pixel.vy;

      // Assert — velocity increases each frame.
      expect(v1, greaterThan(v0));
      expect(v2, greaterThan(v1));
    });

    test('when_hits_left_wall_then_bounces_back', () {
      // Arrange
      final pixel = TubePixel(
        x: 0.1,
        y: 10,
        color: const AspectColor(0.5, 0.5, 0.5),
        vx: -50.0,
        vy: 5.0,
      );
      final originalVx = pixel.vx;

      // Act
      stepFalling(pixel, phys, 0.05);

      // Assert — bounced off left wall, vx reversed and damped.
      expect(pixel.x, greaterThanOrEqualTo(0));
      expect(pixel.vx, greaterThan(originalVx)); // less negative
    });

    test('when_hits_right_wall_then_bounces_back', () {
      // Arrange
      final pixel = TubePixel(
        x: phys.tubeWidth - 0.1,
        y: 10,
        color: const AspectColor(0.5, 0.5, 0.5),
        vx: 50.0,
        vy: 5.0,
      );
      final originalVx = pixel.vx;

      // Act
      stepFalling(pixel, phys, 0.05);

      // Assert — bounced off right wall, vx reversed and damped.
      expect(pixel.x, lessThanOrEqualTo(phys.tubeWidth));
      expect(pixel.vx, lessThan(originalVx)); // less positive
    });

    test('when_hits_tube_bottom_then_stops', () {
      // Arrange
      final pixel = TubePixel(
        x: 10,
        y: phys.tubeHeight - 2,
        color: const AspectColor(0.5, 0.5, 0.5),
        vy: 20.0,
      );

      // Act
      stepFalling(pixel, phys, 0.05);

      // Assert — clamped to tube bottom.
      expect(pixel.y, lessThanOrEqualTo(phys.tubeHeight));
    });
  });

  group('shouldBeginSettling', () {
    test('when_above_sand_surface_then_does_not_settle', () {
      // Arrange
      final pixel = TubePixel(
        x: 10,
        y: 0,
        color: const AspectColor(0.5, 0.5, 0.5),
      );
      final settled = <TubePixel>[];

      // Act
      final result = shouldBeginSettling(pixel, settled, phys, 1.0);

      // Assert
      expect(result, isFalse);
    });

    test('when_near_sand_surface_and_slow_then_settles', () {
      // Arrange
      final pixel = TubePixel(
        x: 10,
        y: 48,
        color: const AspectColor(0.5, 0.5, 0.5),
        vy: 0.2,
      );
      final settled = [
        TubePixel(
          x: 10,
          y: 50,
          color: const AspectColor(0.5, 0.5, 0.5),
          phase: PixelPhase.settled,
        ),
      ];

      // Act
      final result = shouldBeginSettling(pixel, settled, phys, 2.0);

      // Assert — pixel is close to sand surface and slow → begin settling.
      expect(result, isTrue);
    });

    test('when_fast_then_does_not_settle_even_near_surface', () {
      // Arrange
      final pixel = TubePixel(
        x: 10,
        y: 48,
        color: const AspectColor(0.5, 0.5, 0.5),
        vy: 10.0, // fast
      );
      final settled = [
        TubePixel(
          x: 10,
          y: 50,
          color: const AspectColor(0.5, 0.5, 0.5),
          phase: PixelPhase.settled,
        ),
      ];

      // Act
      final result = shouldBeginSettling(pixel, settled, phys, 2.0);

      // Assert — too fast to settle.
      expect(result, isFalse);
    });

    test('when_near_bottom_with_no_settled_pixels_then_settles', () {
      // Arrange
      final pixel = TubePixel(
        x: 10,
        y: phys.tubeHeight - 1,
        color: const AspectColor(0.5, 0.5, 0.5),
        vy: 0.1,
      );

      // Act
      final result = shouldBeginSettling(pixel, [], phys, 1.0);

      // Assert — near tube bottom, should begin settling.
      expect(result, isTrue);
    });
  });

  group('lockSettled', () {
    test('when_locked_then_velocity_zero_and_phase_settled', () {
      // Arrange
      final pixel = TubePixel(
        x: 5,
        y: 30,
        color: const AspectColor(0.5, 0.5, 0.5),
        vx: 1.0,
        vy: -2.0,
      );

      // Act
      lockSettled(pixel);

      // Assert
      expect(pixel.vx, 0);
      expect(pixel.vy, 0);
      expect(pixel.phase, PixelPhase.settled);
    });
  });
}
