import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:gps_companion/ui/map_screen.dart';

void main() {
  group('HUD integration', () {
    testWidgets('when_tube_widget_renders_then_found_in_tree', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(body: TubeHud(fillLevel: 0.5)),
        ),
      );

      expect(find.byType(TubeHud), findsOneWidget);
    });

    testWidgets('when_stack_widget_renders_then_found_in_tree', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(body: PackStackHud()),
        ),
      );

      expect(find.byType(PackStackHud), findsOneWidget);
    });

    test('when_tube_fill_level_is_set_then_matches_provided_value', () {
      const tube = TubeHud(fillLevel: 0.75);
      expect(tube.fillLevel, 0.75);
    });

    test('when_tube_has_zero_fill_then_fill_level_is_zero', () {
      const tube = TubeHud();
      expect(tube.fillLevel, 0.0);
    });

    test('when_stack_has_no_boosters_then_renders_empty', () {
      const stack = PackStackHud();
      expect(stack.boosters, isEmpty);
    });
  });
}
