import 'package:flutter_test/flutter_test.dart';
import 'package:gps_companion/ui/piston_animation.dart';

void main() {
  group('PistonState', () {
    test('when_idle_then_trigger_starts_descending', () {
      // Arrange
      final state = PistonState();

      // Act
      state.trigger();

      // Assert
      expect(state.phase, PistonPhase.descending);
      expect(state.progress, 0.0);
    });

    test('when_descending_completes_then_transitions_to_compressing', () {
      // Arrange
      final state = PistonState();
      state.trigger(); // → descending

      // Act — advance past 0.3s descent time.
      state.tick(0.35);

      // Assert
      expect(state.phase, PistonPhase.compressing);
    });

    test('when_compressing_completes_then_block_visible_and_retracting', () {
      // Arrange
      final state = PistonState();
      state.trigger();
      state.tick(0.35); // → compressing

      // Act
      final hadBlock = state.tick(0.25); // past 0.2s compression

      // Assert
      expect(state.phase, PistonPhase.retracting);
      expect(state.blockVisible, isTrue);
      expect(hadBlock, isFalse); // block not yet transferred
    });

    test('when_retracting_completes_then_tube_emptied_and_done', () {
      // Arrange
      final state = PistonState();
      state.trigger();
      state.tick(0.35); // → compressing
      state.tick(0.25); // → retracting

      // Act
      final hadBlock = state.tick(0.25); // past 0.2s retraction

      // Assert
      expect(state.phase, PistonPhase.done);
      expect(state.tubeEmptied, isTrue);
      expect(hadBlock, isTrue); // pack block ready for stack
    });

    test('when_reset_then_back_to_idle', () {
      // Arrange
      final state = PistonState();
      state.trigger();
      state.tick(0.35);
      state.tick(0.25);
      state.tick(0.25);

      // Act
      state.reset();

      // Assert
      expect(state.phase, PistonPhase.idle);
      expect(state.blockVisible, isFalse);
      expect(state.tubeEmptied, isFalse);
    });

    test('when_done_then_trigger_restarts_cycle', () {
      // Arrange
      final state = PistonState();
      state.trigger();
      state.tick(0.8); // full cycle → done

      // Act
      state.trigger();

      // Assert
      expect(state.phase, PistonPhase.descending);
    });

    test('when_idle_tick_does_nothing', () {
      // Arrange
      final state = PistonState();

      // Act
      final hadBlock = state.tick(1.0);

      // Assert
      expect(state.phase, PistonPhase.idle);
      expect(hadBlock, isFalse);
    });
  });
}
