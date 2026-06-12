import 'package:flutter_test/flutter_test.dart';
import 'package:gps_companion/domain/biome.dart';
import 'package:gps_companion/domain/grain.dart';
import 'package:gps_companion/domain/leyline.dart';
import 'package:gps_companion/domain/spawn.dart';

void main() {
  group('spawnCell determinism', () {
    test('same_inputs_produce_identical_grains', () {
      final a = spawnCell(cellX: 100, cellY: 200, day: 20000, week: 2900, biome: Biome.forestPark);
      final b = spawnCell(cellX: 100, cellY: 200, day: 20000, week: 2900, biome: Biome.forestPark);
      expect(a.length, b.length);
      for (var i = 0; i < a.length; i++) {
        expect(a[i].id, b[i].id);
        expect(a[i].lat, b[i].lat);
        expect(a[i].grain.type, b[i].grain.type);
        expect(a[i].grain.axes, b[i].grain.axes);
      }
    });

    test('different_day_changes_spawn', () {
      final d1 = spawnCell(cellX: 5, cellY: 5, day: 20000, week: 2900, biome: Biome.urbanResidential);
      final d2 = spawnCell(cellX: 5, cellY: 5, day: 20001, week: 2900, biome: Biome.urbanResidential);
      // Overwhelmingly likely the id sequences differ.
      final ids1 = d1.map((g) => g.id).toList();
      final ids2 = d2.map((g) => g.id).toList();
      expect(ids1, isNot(equals(ids2)));
    });
  });

  group('spawnCell biome behaviour', () {
    test('dominant_axis_matches_grain_type', () {
      final grains = spawnCell(cellX: 1, cellY: 1, day: 19000, week: 2700, biome: Biome.mountainDesert);
      for (final sg in grains) {
        final dom = Grain.dominantAxis(sg.grain.axes);
        expect(dom, sg.grain.type.axis,
            reason: 'dominant axis should equal the grain type axis');
      }
    });

    test('higher_density_biome_yields_more_grains_on_average', () {
      var forest = 0;
      var historic = 0;
      for (var c = 0; c < 60; c++) {
        forest += spawnCell(cellX: c, cellY: 3, day: 18000, week: 2600, biome: Biome.forestPark).length;
        historic += spawnCell(cellX: c, cellY: 3, day: 18000, week: 2600, biome: Biome.historicCultural).length;
      }
      // Forest density 300 vs historic 80 — forest should produce more overall.
      expect(forest, greaterThan(historic));
    });
  });

  group('leyline overlay', () {
    test('weekly_theme_is_deterministic', () {
      final t1 = weeklyTheme(2900);
      final t2 = weeklyTheme(2900);
      expect(t1.dominantAxes, t2.dominantAxes);
      expect(t1.intensity, t2.intensity);
    });

    test('weekly_theme_intensity_in_range', () {
      for (var w = 2800; w < 2820; w++) {
        final t = weeklyTheme(w);
        expect(t.intensity, inInclusiveRange(0.5, 2.0));
        expect(t.dominantAxes.every((a) => kLeylineAxes.contains(a)), isTrue);
      }
    });

    test('noise3_is_deterministic_and_bounded', () {
      expect(noise3(1.5, 2.5, 3.5), noise3(1.5, 2.5, 3.5));
      for (var i = 0; i < 50; i++) {
        final n = noise3(i * 0.3, i * 0.7, i * 1.1);
        expect(n, inInclusiveRange(-1.0, 1.0));
      }
    });
  });
}
