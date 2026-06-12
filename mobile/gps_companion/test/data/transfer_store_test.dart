import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:gps_companion/data/store.dart';
import 'package:gps_companion/data/transfer.dart';
import 'package:gps_companion/domain/grain.dart';
import 'package:gps_companion/domain/inventory.dart';

Grain _grain(GrainType t) =>
    Grain(axes: List<double>.filled(8, 0.01), type: t, rarity: GrainRarity.common);

void main() {
  group('InventoryStore encode/decode', () {
    test('roundtrip_preserves_loose_and_boosters', () {
      final inv = Inventory(
        loose: [_grain(GrainType.nature), _grain(GrainType.urban)],
        boosters: [
          Booster(
            grains: [_grain(GrainType.water)],
            forgedAt: DateTime.utc(2026, 6, 6),
            locationName: 'Park',
          ),
        ],
      );
      final back = InventoryStore.decode(InventoryStore.encode(inv));
      expect(back.loose.length, 2);
      expect(back.boosters.length, 1);
      expect(back.boosters.first.locationName, 'Park');
    });
  });

  group('PairingInfo.fromQr', () {
    test('parses_ip_port_token', () {
      final info = PairingInfo.fromQr(
        '{"ip":"192.168.1.42","port":9876,"token":"a1b2c3"}',
      );
      expect(info.ip, '192.168.1.42');
      expect(info.port, 9876);
      expect(info.token, 'a1b2c3');
      expect(info.wsUri.toString(), 'ws://192.168.1.42:9876');
    });
  });

  group('buildPayload', () {
    test('matches_desktop_contract', () {
      final payload = buildPayload(
        boosters: [
          Booster(grains: [_grain(GrainType.earth)], forgedAt: DateTime.utc(2026)),
        ],
        deviceId: 'abc123',
        token: 'tok',
      );
      final j = jsonDecode(payload) as Map<String, dynamic>;
      expect(j['version'], 1);
      expect(j['device_id'], 'abc123');
      expect(j['token'], 'tok');
      expect((j['boosters'] as List).length, 1);
      final g = (j['boosters'][0]['grains'][0]) as Map<String, dynamic>;
      expect(g['grain_type'], 'Earth');
    });
  });
}
