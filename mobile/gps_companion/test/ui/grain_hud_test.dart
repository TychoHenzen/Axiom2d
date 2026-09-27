import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:gps_companion/data/store.dart';
import 'package:gps_companion/domain/inventory.dart';
import 'package:gps_companion/domain/route_log.dart';
import 'package:gps_companion/ui/app_state.dart';
import 'package:gps_companion/ui/map_hud.dart';

void main() {
  group('HUD integration', () {
    testWidgets('when_tube_widget_renders_then_found_in_tree', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(home: Scaffold(body: TubeHud(fillLevel: 0.5))),
      );

      expect(find.byType(TubeHud), findsOneWidget);
    });

    testWidgets('when_stack_widget_renders_then_found_in_tree', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(home: Scaffold(body: PackStackHud())),
      );

      expect(find.byType(PackStackHud), findsOneWidget);
    });

    testWidgets('when_map_hud_renders_then_shows_status_and_toggle', (
      tester,
    ) async {
      final state = AppState(
        store: InventoryStore(),
        inventory: Inventory(),
        routeLogStore: RouteLogStore(),
        routeLog: RouteLog(),
      );
      var enabled = false;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: MapHud(
              state: state,
              themeLabel: 'This week: Nature',
              status: 'Acquiring GPS…',
              backgroundEnabled: false,
              onToggleBackground: (value) => enabled = value,
            ),
          ),
        ),
      );

      expect(find.text('This week: Nature'), findsOneWidget);
      expect(find.text('Acquiring GPS…'), findsOneWidget);
      expect(find.text('Track in Background'), findsOneWidget);

      await tester.tap(find.byType(Switch));
      expect(enabled, isTrue);
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
