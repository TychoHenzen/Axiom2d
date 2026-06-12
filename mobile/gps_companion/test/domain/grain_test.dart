import 'package:flutter_test/flutter_test.dart';
import 'package:gps_companion/domain/grain.dart';

void main() {
  group('GrainType axis mapping', () {
    test('forAxis_matches_spec_table', () {
      expect(GrainType.forAxis(0), GrainType.earth);
      expect(GrainType.forAxis(2), GrainType.urban);
      expect(GrainType.forAxis(4), GrainType.water);
      expect(GrainType.forAxis(6), GrainType.nature);
      expect(GrainType.forAxis(7), GrainType.arcane);
    });

    test('leyline_types_are_febris_lumines_inertiae', () {
      expect(GrainType.febris.isLeyline, isTrue);
      expect(GrainType.lumines.isLeyline, isTrue);
      expect(GrainType.inertiae.isLeyline, isTrue);
      expect(GrainType.nature.isLeyline, isFalse);
      expect(GrainType.earth.isLeyline, isFalse);
    });
  });

  group('Grain.dominantAxis', () {
    test('returns_index_of_largest_magnitude_including_negatives', () {
      final axes = [0.01, -0.2, 0.05, 0.0, 0.0, 0.0, 0.0, 0.0];
      expect(Grain.dominantAxis(axes), 1);
    });
  });

  group('Grain JSON payload contract', () {
    test('toJson_uses_desktop_field_names_and_pascalcase', () {
      final g = Grain(
        axes: [0.02, -0.01, 0, 0, 0, 0, 0, 0],
        type: GrainType.nature,
        rarity: GrainRarity.common,
      );
      final j = g.toJson();
      expect(j['grain_type'], 'Nature');
      expect(j['rarity'], 'Common');
      expect((j['axes'] as List).length, 8);
    });

    test('fromJson_roundtrip_preserves_type_and_rarity', () {
      final g = Grain(
        axes: [0, 0, 0, 0.1, 0, 0, 0, 0],
        type: GrainType.lumines,
        rarity: GrainRarity.epic,
      );
      final back = Grain.fromJson(g.toJson());
      expect(back.type, GrainType.lumines);
      expect(back.rarity, GrainRarity.epic);
      expect(back.axes[3], 0.1);
    });
  });
}
