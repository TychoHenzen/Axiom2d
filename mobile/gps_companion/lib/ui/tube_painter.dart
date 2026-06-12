/// Glass test-tube CustomPainter: renders a vertical cylinder with rounded
/// bottom, transparent/translucent walls (~20px wide). The interior shows
/// settled grain mass (colored pixels filling from bottom) and empty space
/// above proportional to fill level.
library;

import 'package:flutter/material.dart';

import 'pixel_physics.dart';

/// Paints the glass test-tube on the RIGHT side of the map screen.
class TubePainter extends CustomPainter {
  TubePainter({
    required this.pixels,
    required this.fillLevel, // 0.0 .. 1.0
    this.showPiston = false,
    this.pistonProgress = 0.0,
    this.tubeWidth = 20.0,
    this.tubeHeight = 200.0,
  });

  final List<TubePixel> pixels;
  final double fillLevel;
  final bool showPiston;
  final double pistonProgress;
  final double tubeWidth;
  final double tubeHeight;

  @override
  void paint(Canvas canvas, Size size) {
    _drawTube(canvas, Offset(tubeWidth / 2, 0));
    _drawSettledGrains(canvas, Offset(tubeWidth / 2, 0));
    if (showPiston) {
      _drawPiston(canvas, Offset(tubeWidth / 2, 0));
    }
  }

  void _drawTube(Canvas canvas, Offset origin) {
    final tubePaint = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.5
      ..color = Colors.white.withValues(alpha: 0.3);

    final glassPaint = Paint()
      ..style = PaintingStyle.fill
      ..color = Colors.white.withValues(alpha: 0.05);

    final wallRect = Rect.fromLTWH(
        origin.dx - tubeWidth / 2, origin.dy, tubeWidth, tubeHeight);

    canvas.drawRect(wallRect, glassPaint);
    canvas.drawRect(wallRect, tubePaint);
  }

  void _drawSettledGrains(Canvas canvas, Offset origin) {
    for (final p in pixels) {
      if (p.phase == PixelPhase.falling) continue;
      final paint = Paint()
        ..style = PaintingStyle.fill
        ..color = Color.fromRGBO(
          (p.color.r * 255).round(),
          (p.color.g * 255).round(),
          (p.color.b * 255).round(),
          1.0,
        );
      canvas.drawRect(
        Rect.fromLTWH(
          origin.dx - tubeWidth / 2 + p.x,
          origin.dy + p.y,
          1.5,
          1.5,
        ),
        paint,
      );
    }
  }

  void _drawPiston(Canvas canvas, Offset origin) {
    final pistonY = origin.dy + pistonProgress * tubeHeight;
    final pistonPaint = Paint()
      ..style = PaintingStyle.fill
      ..color = Colors.grey.shade400;

    // Piston head: horizontal bar across tube width.
    canvas.drawRect(
      Rect.fromLTWH(
        origin.dx - tubeWidth / 2 + 1,
        pistonY - 4,
        tubeWidth - 2,
        6,
      ),
      pistonPaint,
    );
  }

  @override
  bool shouldRepaint(TubePainter oldDelegate) =>
      oldDelegate.fillLevel != fillLevel ||
      oldDelegate.showPiston != showPiston ||
      oldDelegate.pistonProgress != pistonProgress;
}
