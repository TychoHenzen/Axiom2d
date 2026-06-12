import 'package:flutter_test/flutter_test.dart';
import 'package:gps_companion/ui/pack_stack.dart';

void main() {
  group('PackStackState', () {
    test('when_created_then_empty_by_default', () {
      // Arrange & Act
      final state = PackStackState();

      // Assert
      expect(state.count, 0);
      expect(state.packs, isEmpty);
    });

    test('when_initial_count_then_pre_populated', () {
      // Arrange & Act
      final state = PackStackState(initialCount: 3);

      // Assert
      expect(state.count, 3);
      expect(state.packs.length, 3);
    });

    test('when_pushing_pack_then_adds_to_bottom', () {
      // Arrange
      final state = PackStackState(initialCount: 2);
      final beforeCount = state.count;

      // Act
      final entry = state.pushPack();

      // Assert — new pack added, count increases.
      expect(state.count, beforeCount + 1);
      expect(entry.animProgress, 0.0); // starts animating
      // New pack is last in list (bottom).
      expect(state.packs.last.id, entry.id);
    });

    test('when_multiple_pushes_then_existing_packs_shift_up', () {
      // Arrange
      final state = PackStackState(initialCount: 2);
      final firstId = state.packs.first.id;
      final secondId = state.packs[1].id;

      // Act
      state.pushPack();

      // Assert — existing packs remain in order (shifted up).
      expect(state.packs[0].id, firstId);
      expect(state.packs[1].id, secondId);
      expect(state.packs.length, 3);
    });

    test('when_tick_then_anim_progress_increases', () {
      // Arrange
      final state = PackStackState();
      final entry = state.pushPack();
      expect(entry.animProgress, 0.0);

      // Act
      state.tick(0.25); // half of 0.5s animation

      // Assert
      final updated = state.packs.last;
      expect(updated.animProgress, greaterThan(0.0));
      expect(updated.animProgress, lessThan(1.0));
    });

    test('when_tick_to_completion_then_progress_reaches_one', () {
      // Arrange
      final state = PackStackState();
      state.pushPack();

      // Act
      state.tick(0.6); // past 0.5s animation

      // Assert
      expect(state.packs.last.animProgress, 1.0);
    });

    test('when_count_matches_push_count', () {
      // Arrange
      final state = PackStackState();

      // Act
      for (var i = 0; i < 5; i++) {
        state.pushPack();
      }

      // Assert
      expect(state.count, 5);
    });
  });
}
