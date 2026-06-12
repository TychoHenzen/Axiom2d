/// Local persistence of the inventory (loose grains + forged boosters) via
/// shared_preferences. All data stays on-device.
library;

import 'dart:convert';

import 'package:shared_preferences/shared_preferences.dart';

import '../domain/grain.dart';
import '../domain/inventory.dart';

class InventoryStore {
  static const String _key = 'inventory_v1';

  Future<Inventory> load() async {
    final prefs = await SharedPreferences.getInstance();
    final raw = prefs.getString(_key);
    if (raw == null) return Inventory();
    return decode(raw);
  }

  Future<void> save(Inventory inv) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_key, encode(inv));
  }

  /// Pure encode (testable without platform channels).
  static String encode(Inventory inv) => jsonEncode({
    'loose': inv.loose.map((g) => g.toJson()).toList(),
    'boosters': inv.boosters.map((b) => b.toJson()).toList(),
  });

  /// Pure decode (testable without platform channels).
  static Inventory decode(String raw) {
    final j = jsonDecode(raw) as Map<String, dynamic>;
    return Inventory(
      loose: (j['loose'] as List)
          .map((e) => Grain.fromJson(e as Map<String, dynamic>))
          .toList(),
      boosters: (j['boosters'] as List)
          .map((e) => Booster.fromJson(e as Map<String, dynamic>))
          .toList(),
    );
  }
}
