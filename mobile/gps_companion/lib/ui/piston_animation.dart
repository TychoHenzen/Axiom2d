/// Piston animation controller: descends from above the tube, compresses
/// the pixel mass into a solid block, then retracts. The block emerges at
/// the tube position and is handed off to the pack-stack animation.
library;

/// Phase of the piston cycle.
enum PistonPhase {
  /// Waiting for tube to fill (packProgress < 100%).
  idle,

  /// Piston head moving downward into the tube.
  descending,

  /// Piston compressing pixel mass into a solid block.
  compressing,

  /// Piston head moving back up.
  retracting,

  /// Cycle complete — pack block has emerged, tube is empty.
  done,
}

/// State for the piston animation.
class PistonState {
  PistonState();

  PistonPhase phase = PistonPhase.idle;

  /// 0..1 progress within the current phase.
  double progress = 0.0;

  /// Whether a pack block is visible (emerged after compression).
  bool blockVisible = false;

  /// Whether the tube was just emptied (piston cycle completed).
  bool tubeEmptied = false;

  /// Advance the piston by [dt] seconds. Returns true when a new pack
  /// block has emerged (signaling the caller to start pack animation).
  bool tick(double dt) {
    tubeEmptied = false;
    var remaining = dt;
    while (remaining > 0) {
      final consumed = _tickPhase(remaining);
      remaining -= consumed;
      if (phase == PistonPhase.idle || phase == PistonPhase.done) break;
    }
    return phase == PistonPhase.done && tubeEmptied;
  }

  double _tickPhase(double dt) {
    switch (phase) {
      case PistonPhase.idle:
      case PistonPhase.done:
        return dt; // consume all remaining time

      case PistonPhase.descending:
        final needed = 0.3 - progress * 0.3;
        if (dt >= needed) {
          progress = 0.0;
          phase = PistonPhase.compressing;
          return needed;
        }
        progress += dt / 0.3;
        return dt;

      case PistonPhase.compressing:
        final needed = 0.2 - progress * 0.2;
        if (dt >= needed) {
          progress = 0.0;
          blockVisible = true;
          phase = PistonPhase.retracting;
          return needed;
        }
        progress += dt / 0.2;
        return dt;

      case PistonPhase.retracting:
        final needed = 0.2 - progress * 0.2;
        if (dt >= needed) {
          progress = 0.0;
          blockVisible = false;
          tubeEmptied = true;
          phase = PistonPhase.done;
          return needed;
        }
        progress += dt / 0.2;
        return dt;
    }
  }

  /// Start the piston cycle (called when tube is full).
  void trigger() {
    if (phase == PistonPhase.idle || phase == PistonPhase.done) {
      phase = PistonPhase.descending;
      progress = 0.0;
      blockVisible = false;
      tubeEmptied = false;
    }
  }

  /// Reset to idle after pack block has been transferred.
  void reset() {
    phase = PistonPhase.idle;
    progress = 0.0;
    blockVisible = false;
    tubeEmptied = false;
  }
}
