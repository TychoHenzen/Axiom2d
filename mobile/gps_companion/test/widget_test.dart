import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:gps_companion/data/store.dart';
import 'package:gps_companion/domain/grain.dart';
import 'package:gps_companion/domain/inventory.dart';
import 'package:gps_companion/domain/route_log.dart';
import 'package:gps_companion/ui/app_state.dart';
import 'package:gps_companion/ui/gallery_screen.dart';

void main() {
  testWidgets('gallery shows empty-state prompt when no packs', (tester) async {
    final state = AppState(
      store: InventoryStore(),
      inventory: Inventory(),
      routeLogStore: RouteLogStore(),
      routeLog: RouteLog(),
    );
    await tester.pumpWidget(MaterialApp(home: GalleryScreen(state: state)));
    expect(find.textContaining('No packs yet'), findsOneWidget);
  });

  testWidgets('gallery renders a tile per forged pack', (tester) async {
    final grains = List.generate(
      100,
      (_) => Grain(
        axes: List<double>.filled(8, 0.01),
        type: GrainType.nature,
        rarity: GrainRarity.common,
      ),
    );
    final state = AppState(
      store: InventoryStore(),
      inventory: Inventory(
        boosters: [Booster(grains: grains, forgedAt: DateTime.utc(2026, 6, 6))],
      ),
      routeLogStore: RouteLogStore(),
      routeLog: RouteLog(),
    );
    await tester.pumpWidget(MaterialApp(home: GalleryScreen(state: state)));
    expect(find.text('Pack #1'), findsOneWidget);
  });
}
