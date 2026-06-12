import 'package:flutter_test/flutter_test.dart';
import 'package:gps_companion/domain/grain.dart';
import 'package:gps_companion/domain/inventory.dart';

Grain _grain(GrainType type, [GrainRarity rarity = GrainRarity.common]) => Grain(
  axes: List<double>.filled(8, 0.0),
  type: type,
  rarity: rarity,
);

void main() {
  final now = DateTime.utc(2026, 6, 6, 12);

  group('Inventory auto-forge', () {
    test('forges_a_booster_exactly_at_100_grains', () {
      final inv = Inventory();
      var forged = 0;
      for (var i = 0; i < 99; i++) {
        forged += inv.collect(_grain(GrainType.nature), now: now);
      }
      expect(forged, 0);
      expect(inv.grainCount, 99);
      expect(inv.boosters, isEmpty);

      forged += inv.collect(_grain(GrainType.nature), now: now);
      expect(forged, 1);
      expect(inv.boosters.length, 1);
      expect(inv.boosters.first.grains.length, 100);
      expect(inv.grainCount, 0);
    });

    test('collectAll_forges_multiple_boosters_and_keeps_remainder', () {
      final inv = Inventory();
      // One brush batch of 250 grains => 2 boosters, 50 loose.
      final batch = [for (var i = 0; i < 250; i++) _grain(GrainType.earth)];
      final forged = inv.collectAll(batch, now: now);
      expect(forged, 2);
      expect(inv.boosters.length, 2);
      expect(inv.grainCount, 50);
    });

    test('collectAll_of_empty_batch_is_a_noop', () {
      final inv = Inventory();
      expect(inv.collectAll(const [], now: now), 0);
      expect(inv.grainCount, 0);
    });

    test('spills_remainder_into_next_pack', () {
      final inv = Inventory();
      // 150 grains => 1 booster of 100, 50 loose remaining.
      for (var i = 0; i < 150; i++) {
        inv.collect(_grain(GrainType.urban), now: now);
      }
      expect(inv.boosters.length, 1);
      expect(inv.grainCount, 50);
    });

    test('booster_carries_provenance', () {
      final inv = Inventory();
      for (var i = 0; i < 100; i++) {
        inv.collect(
          _grain(GrainType.water),
          now: now,
          lat: 51.5,
          lon: -0.12,
          locationName: 'Hyde Park',
        );
      }
      final b = inv.boosters.single;
      expect(b.locationName, 'Hyde Park');
      expect(b.lat, 51.5);
      expect(b.forgedAt, now);
    });
  });

  group('BoosterTheme', () {
    test('pure_pack_when_single_type_at_90_percent', () {
      final grains = [
        for (var i = 0; i < 95; i++) _grain(GrainType.nature),
        for (var i = 0; i < 5; i++) _grain(GrainType.urban),
      ];
      final theme = BoosterTheme.of(grains);
      expect(theme.dominantType, GrainType.nature);
      expect(theme.isPure, isTrue);
      expect(theme.isMixed, isFalse);
    });

    test('mixed_pack_when_no_type_reaches_50_percent', () {
      final grains = [
        for (var i = 0; i < 30; i++) _grain(GrainType.nature),
        for (var i = 0; i < 30; i++) _grain(GrainType.urban),
        for (var i = 0; i < 40; i++) _grain(GrainType.water),
      ];
      final theme = BoosterTheme.of(grains);
      expect(theme.isMixed, isTrue);
      expect(theme.isPure, isFalse);
    });

    test('gold_shimmer_when_five_or_more_legendary', () {
      final grains = [
        for (var i = 0; i < 5; i++)
          _grain(GrainType.arcane, GrainRarity.legendary),
        for (var i = 0; i < 95; i++) _grain(GrainType.arcane),
      ];
      expect(BoosterTheme.of(grains).hasGoldShimmer, isTrue);
    });
  });

  group('Booster JSON', () {
    test('roundtrip_preserves_grains_and_location', () {
      final b = Booster(
        grains: [_grain(GrainType.earth), _grain(GrainType.arcane)],
        forgedAt: now,
        lat: 1.0,
        lon: 2.0,
        locationName: 'Somewhere',
      );
      final back = Booster.fromJson(b.toJson());
      expect(back.grains.length, 2);
      expect(back.lat, 1.0);
      expect(back.locationName, 'Somewhere');
      expect(back.forgedAt.toUtc(), now);
    });
  });
}
