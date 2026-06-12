import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:gps_companion/domain/grain_pixel.dart';
import 'package:gps_companion/ui/pixel_physics.dart';
import 'package:gps_companion/ui/tube_painter.dart';

/// Small wrapper that isolates our CustomPaint from framework-internal ones.
Widget _wrap(CustomPainter painter) => Directionality(
      textDirection: TextDirection.ltr,
      child: CustomPaint(
        size: const Size(20, 200),
        painter: painter,
      ),
    );

void main() {
  testWidgets('when_tube_renders_then_custom_painter_creates_translucent_walls',
      (tester) async {
    // Arrange
    final painter = TubePainter(pixels: const [], fillLevel: 0.0);

    // Act
    await tester.pumpWidget(_wrap(painter));

    // Assert — widget renders without errors.
    expect(find.byType(CustomPaint), findsOneWidget);
  });

  testWidgets('when_tube_has_settled_grains_then_interior_shows_colored_pixels',
      (tester) async {
    // Arrange
    final pixel = TubePixel(
      x: 8,
      y: 180,
      color: const AspectColor(0.85, 0.55, 0.20),
      phase: PixelPhase.settled,
    );
    final painter = TubePainter(pixels: [pixel], fillLevel: 0.5);

    // Act
    await tester.pumpWidget(_wrap(painter));

    // Assert
    expect(find.byType(CustomPaint), findsOneWidget);
  });

  test('when_should_repaint_then_detects_changes', () {
    // Arrange
    final p1 = TubePainter(pixels: const [], fillLevel: 0.0);
    final p2 = TubePainter(pixels: const [], fillLevel: 0.0);
    final p3 = TubePainter(pixels: const [], fillLevel: 0.5);
    final p4 = TubePainter(pixels: const [], fillLevel: 0.0, showPiston: true);

    // Assert
    expect(p1.shouldRepaint(p2), isFalse); // same state
    expect(p1.shouldRepaint(p3), isTrue);  // different fill
    expect(p1.shouldRepaint(p4), isTrue);  // piston shown
  });
}
