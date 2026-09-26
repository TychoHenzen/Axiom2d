import 'package:flutter/material.dart';

import '../domain/grain_pixel.dart';
import '../domain/inventory.dart';
import 'app_state.dart';
import 'pixel_physics.dart';
import 'tube_painter.dart';

class PlayerDot extends StatelessWidget {
  const PlayerDot({super.key});

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: BoxDecoration(
        color: Colors.blueAccent,
        shape: BoxShape.circle,
        border: Border.all(color: Colors.white, width: 3),
      ),
    );
  }
}

class MapHud extends StatelessWidget {
  const MapHud({
    super.key,
    required this.state,
    required this.themeLabel,
    required this.status,
    required this.backgroundEnabled,
    required this.onToggleBackground,
  });

  final AppState state;
  final String themeLabel;
  final String status;
  final bool backgroundEnabled;
  final ValueChanged<bool> onToggleBackground;

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: state,
      builder: (context, _) {
        return Padding(
          padding: const EdgeInsets.all(8),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              _chip(themeLabel),
              if (status.isNotEmpty) ...[
                const SizedBox(height: 4),
                _chip(status),
              ],
              const SizedBox(height: 8),
              Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  const Icon(Icons.gps_fixed, color: Colors.white70, size: 16),
                  const SizedBox(width: 6),
                  const Text(
                    'Track in Background',
                    style: TextStyle(color: Colors.white70, fontSize: 12),
                  ),
                  const SizedBox(width: 4),
                  SizedBox(
                    height: 24,
                    child: Switch(
                      value: backgroundEnabled,
                      onChanged: onToggleBackground,
                      materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
                    ),
                  ),
                ],
              ),
            ],
          ),
        );
      },
    );
  }

  Widget _chip(String text) => Container(
    padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 5),
    decoration: BoxDecoration(
      color: Colors.black.withValues(alpha: 0.6),
      borderRadius: BorderRadius.circular(12),
    ),
    child: Text(
      text,
      style: const TextStyle(color: Colors.white, fontSize: 13),
    ),
  );
}

/// Animated grain tube — RIGHT side of the map screen.
///
/// Displays falling grain pixels (colored by grain aspect), a settled sand
/// pile growing from the bottom, and the piston head when compressing.
/// Fill level drives from [AppState.packProgress] (0..100 → 0..0.9 tube fill).
class TubeHud extends StatelessWidget {
  const TubeHud({
    super.key,
    this.height = 200.0,
    this.fillLevel = 0.0,
    this.pixels = const [],
    this.showPiston = false,
    this.pistonProgress = 0.0,
  });

  final double height;
  final double fillLevel;
  final List<TubePixel> pixels;
  final bool showPiston;
  final double pistonProgress;

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: 20,
      height: height,
      child: CustomPaint(
        painter: TubePainter(
          pixels: pixels,
          fillLevel: fillLevel,
          showPiston: showPiston,
          pistonProgress: pistonProgress,
          tubeHeight: height,
        ),
      ),
    );
  }
}

/// Pack stack widget — sits to the LEFT of the tube, bottom-aligned.
///
/// Each sealed booster pack is a 5px-high × 16px-wide bar colored by the
/// pack's average grain aspect. New packs appear at the bottom; the stack
/// grows upward.
class PackStackHud extends StatelessWidget {
  const PackStackHud({super.key, this.boosters = const []});

  final List<Booster> boosters;

  /// Average the aspect colors of all grains in a booster.
  static Color _packColor(Booster b) {
    if (b.grains.isEmpty) return const Color(0x80FFFFFF);
    var r = 0.0, g = 0.0, bl = 0.0;
    for (final grain in b.grains) {
      final c = grainPixelColor(grain);
      r += c.r;
      g += c.g;
      bl += c.b;
    }
    final n = b.grains.length.toDouble();
    return Color.fromRGBO(
      (r / n * 255).round().clamp(0, 255),
      (g / n * 255).round().clamp(0, 255),
      (bl / n * 255).round().clamp(0, 255),
      0.85,
    );
  }

  @override
  Widget build(BuildContext context) {
    // Cap visible packs so the column doesn't extend past the tube top.
    final visible = boosters.length.clamp(0, 200);
    return Column(
      mainAxisSize: MainAxisSize.min,
      verticalDirection: VerticalDirection.up,
      children: [
        for (var i = 0; i < visible; i++)
          Container(
            width: 16,
            height: 5,
            margin: const EdgeInsets.only(top: 1),
            color: _packColor(boosters[i]),
          ),
      ],
    );
  }
}
