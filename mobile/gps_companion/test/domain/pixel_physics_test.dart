// Tests that stepFalling() correctly indicates whether a pixel is still
// actively falling.
//
// TDD RED (before fix): stepFalling() always returns false —
//   `expect(result, isTrue)` fails.
// TDD GREEN (after fix): stepFalling() returns true when pixel velocity
//   exceeds the settle threshold.
import 'package:flutter_test/flutter_test.dart';
import 'package:gps_companion/domain/grain_pixel.dart';
import 'package:gps_companion/ui/pixel_physics.dart';

void main() {
  group('stepFalling — return value', () {
    test(
        'when_pixel_has_significant_velocity_then_stepFalling_returns_true',
        () {
      // Arrange — pixel actively falling with velocity well above settle threshold.
      final pixel = TubePixel(
        x: 10,
        y: 10,
        color: const AspectColor(0.5, 0.5, 0.5),
        vy: 50.0, // 50 px/s — far above settleThreshold (0.5)
      );
      final phys = TubePhysics();

      // Act
      final result = stepFalling(pixel, phys, 0.016);

      // Assert — actively moving pixel should return true.
      expect(result, isTrue);
    });

    test(
        'when_pixel_reaches_tube_bottom_and_stops_then_stepFalling_returns_false',
        () {
      // Arrange — pixel at tube bottom with zero velocity.
      final phys = TubePhysics();
      final pixel = TubePixel(
        x: 10,
        y: phys.tubeHeight,
        color: const AspectColor(0.5, 0.5, 0.5),
        vy: 0.0, // fully stopped
      );

      // Act
      final result = stepFalling(pixel, phys, 0.016);

      // Assert — pixel at rest at tube bottom returns false.
      expect(result, isFalse);
    });

    test(
        'when_TubePixel_phase_is_not_falling_then_stepFalling_returns_false',
        () {
      // Arrange — settled pixel should not be stepped.
      final pixel = TubePixel(
        x: 10,
        y: 100,
        color: const AspectColor(0.5, 0.5, 0.5),
        phase: PixelPhase.settled,
        vy: 50.0,
      );

      // Act
      final result = stepFalling(pixel, TubePhysics(), 0.016);

      // Assert — non-falling phase: early-exit returns false.
      expect(result, isFalse);
    });
  });
}
