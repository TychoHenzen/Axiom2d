/// OSM tag to [BiomeSample] mappings. Each entry maps a single OSM key=value
/// pair to a grain distribution, spawn density, and fog color. The lookup
/// function uses the compound `"key=value"` format used by Overpass responses.
library;

import 'biome_def.dart';

// ---------------------------------------------------------------------------
// Distribution index reference (same order as BiomeSample.dist):
//   0 = nature, 1 = urban, 2 = water, 3 = earth, 4 = arcane
// ---------------------------------------------------------------------------

/// All known OSM tag -> BiomeSample mappings, exposed as a const list for
/// test enumeration and iteration.
const List<BiomeSample> kOsmBiomeSamples = [
  // ── landuse/* ──────────────────────────────────────────────────────────
  BiomeSample(
    key: 'landuse=forest',
    densityPerKm2: 320,
    dist: [0.65, 0.02, 0.08, 0.15, 0.10],
    colorArgb: 0x40228B22, // semi-transparent forest green
  ),
  BiomeSample(
    key: 'landuse=residential',
    densityPerKm2: 200,
    dist: [0.08, 0.62, 0.05, 0.12, 0.13],
    colorArgb: 0x40A0A0A0, // grey
  ),
  BiomeSample(
    key: 'landuse=commercial',
    densityPerKm2: 180,
    dist: [0.05, 0.70, 0.03, 0.07, 0.15],
    colorArgb: 0x40C8A000, // amber
  ),
  BiomeSample(
    key: 'landuse=industrial',
    densityPerKm2: 150,
    dist: [0.03, 0.72, 0.02, 0.13, 0.10],
    colorArgb: 0x40808080, // dark grey
  ),
  BiomeSample(
    key: 'landuse=farmland',
    densityPerKm2: 250,
    dist: [0.45, 0.05, 0.05, 0.35, 0.10],
    colorArgb: 0x40DAA520, // goldenrod
  ),
  BiomeSample(
    key: 'landuse=meadow',
    densityPerKm2: 280,
    dist: [0.55, 0.03, 0.07, 0.25, 0.10],
    colorArgb: 0x4090EE90, // light green
  ),
  BiomeSample(
    key: 'landuse=orchard',
    densityPerKm2: 270,
    dist: [0.60, 0.04, 0.06, 0.20, 0.10],
    colorArgb: 0x4032CD32, // lime green
  ),
  BiomeSample(
    key: 'landuse=cemetery',
    densityPerKm2: 100,
    dist: [0.15, 0.10, 0.02, 0.08, 0.65],
    colorArgb: 0x40483D8B, // dark slate blue
  ),
  BiomeSample(
    key: 'landuse=quarry',
    densityPerKm2: 90,
    dist: [0.02, 0.15, 0.02, 0.71, 0.10],
    colorArgb: 0x40A0522D, // sienna
  ),
  BiomeSample(
    key: 'landuse=vineyard',
    densityPerKm2: 260,
    dist: [0.50, 0.05, 0.05, 0.28, 0.12],
    colorArgb: 0x40800080, // purple
  ),

  // ── natural/* ──────────────────────────────────────────────────────────
  BiomeSample(
    key: 'natural=wood',
    densityPerKm2: 330,
    dist: [0.68, 0.02, 0.07, 0.13, 0.10],
    colorArgb: 0x40006400, // dark green
  ),
  BiomeSample(
    key: 'natural=water',
    densityPerKm2: 160,
    dist: [0.10, 0.02, 0.65, 0.08, 0.15],
    colorArgb: 0x404169E1, // royal blue
  ),
  BiomeSample(
    key: 'natural=grassland',
    densityPerKm2: 270,
    dist: [0.50, 0.03, 0.07, 0.30, 0.10],
    colorArgb: 0x407CFC00, // lawn green
  ),
  BiomeSample(
    key: 'natural=scrub',
    densityPerKm2: 220,
    dist: [0.42, 0.03, 0.05, 0.40, 0.10],
    colorArgb: 0x406B8E23, // olive drab
  ),
  BiomeSample(
    key: 'natural=beach',
    densityPerKm2: 140,
    dist: [0.08, 0.05, 0.35, 0.42, 0.10],
    colorArgb: 0x40F4A460, // sandy brown
  ),
  BiomeSample(
    key: 'natural=bare_rock',
    densityPerKm2: 80,
    dist: [0.03, 0.02, 0.02, 0.78, 0.15],
    colorArgb: 0x40696969, // dim grey
  ),
  BiomeSample(
    key: 'natural=wetland',
    densityPerKm2: 200,
    dist: [0.30, 0.02, 0.45, 0.13, 0.10],
    colorArgb: 0x402E8B57, // sea green
  ),

  // ── leisure/* ──────────────────────────────────────────────────────────
  BiomeSample(
    key: 'leisure=park',
    densityPerKm2: 300,
    dist: [0.55, 0.12, 0.08, 0.10, 0.15],
    colorArgb: 0x4000FF7F, // spring green
  ),
  BiomeSample(
    key: 'leisure=garden',
    densityPerKm2: 290,
    dist: [0.58, 0.10, 0.07, 0.10, 0.15],
    colorArgb: 0x4098FB98, // pale green
  ),
  BiomeSample(
    key: 'leisure=playground',
    densityPerKm2: 210,
    dist: [0.20, 0.45, 0.05, 0.10, 0.20],
    colorArgb: 0x40FFA500, // orange
  ),
  BiomeSample(
    key: 'leisure=sports_centre',
    densityPerKm2: 190,
    dist: [0.10, 0.60, 0.05, 0.10, 0.15],
    colorArgb: 0x40FF6347, // tomato
  ),
  BiomeSample(
    key: 'leisure=nature_reserve',
    densityPerKm2: 350,
    dist: [0.70, 0.02, 0.08, 0.10, 0.10],
    colorArgb: 0x40008000, // green
  ),

  // ── waterway/* ─────────────────────────────────────────────────────────
  BiomeSample(
    key: 'waterway=river',
    densityPerKm2: 170,
    dist: [0.12, 0.03, 0.60, 0.10, 0.15],
    colorArgb: 0x401E90FF, // dodger blue
  ),
  BiomeSample(
    key: 'waterway=stream',
    densityPerKm2: 150,
    dist: [0.18, 0.02, 0.55, 0.12, 0.13],
    colorArgb: 0x4087CEEB, // sky blue
  ),
  BiomeSample(
    key: 'waterway=canal',
    densityPerKm2: 140,
    dist: [0.08, 0.15, 0.52, 0.10, 0.15],
    colorArgb: 0x404682B4, // steel blue
  ),

  // ── wetland/* ──────────────────────────────────────────────────────────
  BiomeSample(
    key: 'wetland=bog',
    densityPerKm2: 180,
    dist: [0.25, 0.02, 0.40, 0.18, 0.15],
    colorArgb: 0x40556B2F, // dark olive green
  ),
  BiomeSample(
    key: 'wetland=marsh',
    densityPerKm2: 190,
    dist: [0.30, 0.02, 0.42, 0.16, 0.10],
    colorArgb: 0x403CB371, // medium sea green
  ),
  BiomeSample(
    key: 'wetland=swamp',
    densityPerKm2: 210,
    dist: [0.35, 0.02, 0.38, 0.10, 0.15],
    colorArgb: 0x40006400, // dark green
  ),
];

/// Fast lookup index built lazily from [kOsmBiomeSamples].
final Map<String, BiomeSample> _index = {
  for (final s in kOsmBiomeSamples) s.key: s,
};

/// Look up the [BiomeSample] for an OSM tag. [key] is the tag key
/// (e.g. `"landuse"`), [value] is the tag value (e.g. `"forest"`).
/// Returns `null` for tags with no mapping.
BiomeSample? osmTagToBiome(String key, String value) =>
    _index['$key=$value'];
