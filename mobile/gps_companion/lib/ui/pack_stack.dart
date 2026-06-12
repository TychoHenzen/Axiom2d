/// Pack stack widget: vertical column of booster pack blocks on the LEFT
/// side of the map screen. Each pack is a thin rectangle (~5×20px). New
/// packs add to the bottom, bumping previous packs upward. The stack grows
/// unbounded — packs scroll off the top of the screen.
library;

/// Represents one pack in the stack.
class PackEntry {
  const PackEntry({required this.id, required this.animProgress});
  final String id;

  /// 0..1 progress of the slide-in animation (pack arriving from right).
  final double animProgress;
}

/// State for the pack stack widget.
class PackStackState {
  PackStackState({int initialCount = 0}) : _packs = [] {
    for (var i = 0; i < initialCount; i++) {
      _packs.add(PackEntry(id: 'pack_$i', animProgress: 1.0));
    }
  }

  final List<PackEntry> _packs;
  int _nextId = 0;

  List<PackEntry> get packs => List.unmodifiable(_packs);
  int get count => _packs.length;

  /// Add a new pack to the bottom of the stack. Returns the new entry
  /// so callers can animate its arrival.
  PackEntry pushPack() {
    _nextId++;
    final entry = PackEntry(id: 'pack_$_nextId', animProgress: 0.0);
    // New pack goes to the bottom (end of list = bottom of stack visually).
    _packs.add(entry);
    return entry;
  }

  /// Call each frame to update pack animation progress.
  void tick(double dt) {
    for (var i = 0; i < _packs.length; i++) {
      if (_packs[i].animProgress < 1.0) {
        final newProgress = (_packs[i].animProgress + dt / 0.5)
            .clamp(0.0, 1.0); // 0.5s slide-in
        _packs[i] = PackEntry(
          id: _packs[i].id,
          animProgress: newProgress,
        );
      }
    }
  }
}
