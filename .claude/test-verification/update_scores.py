import json, hashlib, os, datetime

root = r'C:/Users/siriu/RustroverProjects/Axiom2d'
with open(os.path.join(root, '.claude/test-verification/manifest.json')) as f:
    m = json.load(f)

now = datetime.datetime.now(datetime.timezone.utc).isoformat()

def update_file(path, scores, findings, summary):
    full = os.path.join(root, path)
    with open(full, 'rb') as f:
        h = hashlib.sha256(f.read()).hexdigest()
    aq = scores['assertion_quality']
    det = scores['determinism']
    iso = scores['isolation']
    cla = scores['clarity']
    cd = scores['coverage_depth']
    spd = scores['speed']
    diag = scores['diagnostics']
    triv = scores['assertion_triviality']
    ov = (aq*2 + det*2 + iso*1 + cla*1 + cd*2 + spd*1 + diag*1 + triv*1) / 11.0
    ov = round(ov, 1)
    m['files'][path] = {
        'hash': h,
        'last_verified': now,
        'scores': scores,
        'overall': ov,
        'findings': findings,
        'summary': summary
    }
    return ov

# ---- tools/img-to-shape-gui ----
update_file('tools/img-to-shape-gui/tests/suite/loader.rs',
    {'assertion_quality':8,'determinism':10,'isolation':10,'clarity':9,'coverage_depth':7,'speed':10,'diagnostics':8,'assertion_triviality':10},
    [{'severity':'medium','category':'coverage_depth','detail':'Only 2 tests: valid PNG and invalid bytes. Missing: empty bytes, truncated PNG, very large image, non-PNG format'}],
    'Two focused loader tests with good error handling. Coverage depth could expand to edge cases.')

update_file('tools/img-to-shape-gui/tests/suite/preview.rs',
    {'assertion_quality':7,'determinism':10,'isolation':10,'clarity':9,'coverage_depth':7,'speed':9,'diagnostics':8,'assertion_triviality':10},
    [{'severity':'medium','category':'assertion_quality','detail':'Uses !shapes.is_empty() rather than verifying shape count or type'},
     {'severity':'medium','category':'coverage_depth','detail':'Missing: non-Path variant shapes, offset testing, empty color edge case'}],
    'Good preview conversion coverage from path commands to egui shapes. Assertions could verify shape structure.')

update_file('tools/img-to-shape-gui/tests/suite/state.rs',
    {'assertion_quality':9,'determinism':10,'isolation':10,'clarity':9,'coverage_depth':8,'speed':9,'diagnostics':9,'assertion_triviality':10},
    [{'severity':'low','category':'coverage_depth','detail':'Missing: loading 0x0 image, loading after conversion already run, double-conversion'}],
    'Solid AppState lifecycle tests covering defaults, image load, conversion, and export. Minor edge case gaps.')

# ---- tools/img-to-shape ----
update_file('tools/img-to-shape/tests/suite/bezier_fit.rs',
    {'assertion_quality':5,'determinism':10,'isolation':10,'clarity':8,'coverage_depth':3,'speed':10,'diagnostics':3,'assertion_triviality':10},
    [{'severity':'low','category':'structure','detail':'Structural placeholder for single-binary test consolidation. All tests live inline in src/bezier_fit.rs (10 tests). This file cannot test private module.'}],
    'Structural shim for test binary consolidation. Real tests (10) live inline in src/bezier_fit.rs.')

update_file('tools/img-to-shape/tests/suite/boundary_graph.rs',
    {'assertion_quality':5,'determinism':10,'isolation':10,'clarity':8,'coverage_depth':3,'speed':10,'diagnostics':3,'assertion_triviality':10},
    [{'severity':'low','category':'structure','detail':'Structural placeholder for single-binary test consolidation. All tests live inline in src/boundary_graph.rs (6 tests).'}],
    'Structural shim for test binary consolidation. Real tests (6) live inline in src/boundary_graph.rs.')

update_file('tools/img-to-shape/tests/suite/manifest.rs',
    {'assertion_quality':8,'determinism':9,'isolation':8,'clarity':9,'coverage_depth':7,'speed':9,'diagnostics':9,'assertion_triviality':10},
    [{'severity':'medium','category':'determinism','detail':'save_manifest writes to real temp_dir. Isolation concern - parallel test runs may collide on directory'},
     {'severity':'medium','category':'coverage_depth','detail':'Only tests empty manifest roundtrip. Missing: non-empty entries, overwrite existing file, invalid path'}],
    'Clean manifest load/save roundtrip test. Uses real filesystem for temp file, missing non-empty data tests.')

update_file('tools/img-to-shape/tests/suite/scale2x.rs',
    {'assertion_quality':9,'determinism':10,'isolation':10,'clarity':9,'coverage_depth':8,'speed':10,'diagnostics':9,'assertion_triviality':10},
    [{'severity':'low','category':'coverage_depth','detail':'Missing: 2x2 input, non-square input, palette with more than 2 colors'}],
    'Excellent scale2x tests covering 1x1 replication, empty input, and multicolor channel preservation.')

update_file('tools/img-to-shape/tests/suite/segment.rs',
    {'assertion_quality':5,'determinism':10,'isolation':10,'clarity':8,'coverage_depth':3,'speed':10,'diagnostics':3,'assertion_triviality':10},
    [{'severity':'low','category':'structure','detail':'Structural placeholder for single-binary test consolidation. All tests live inline in src/segment.rs (6 tests).'}],
    'Structural shim for test binary consolidation. Real tests (6) live inline in src/segment.rs.')

update_file('tools/img-to-shape/tests/suite/simplify.rs',
    {'assertion_quality':5,'determinism':10,'isolation':10,'clarity':8,'coverage_depth':3,'speed':10,'diagnostics':3,'assertion_triviality':10},
    [{'severity':'low','category':'structure','detail':'Structural placeholder for single-binary test consolidation. All tests live inline in src/simplify.rs (8 tests).'}],
    'Structural shim for test binary consolidation. Real tests (8) live inline in src/simplify.rs.')

# ---- tools/tiled-to-shapes ----
update_file('tools/tiled-to-shapes/tests/suite/codegen.rs',
    {'assertion_quality':8,'determinism':10,'isolation':10,'clarity':9,'coverage_depth':6,'speed':9,'diagnostics':8,'assertion_triviality':10},
    [{'severity':'medium','category':'coverage_depth','detail':'Single test for empty tileset only. Missing: non-empty tileset codegen, output compiles check, file naming in output'}],
    'Single focused codegen test for empty tileset. Needs non-empty tileset tests for real coverage.')

update_file('tools/tiled-to-shapes/tests/suite/extract_tile.rs',
    {'assertion_quality':9,'determinism':10,'isolation':10,'clarity':9,'coverage_depth':8,'speed':9,'diagnostics':9,'assertion_triviality':10},
    [{'severity':'low','category':'coverage_depth','detail':'Missing: tile at edge of image, 1x1 tile, single-tile sheet, tile with partial alpha'}],
    'Excellent tile extraction tests covering known position and OOB errors with precise pixel verification.')

update_file('tools/tiled-to-shapes/tests/suite/normalize.rs',
    {'assertion_quality':9,'determinism':10,'isolation':10,'clarity':9,'coverage_depth':7,'speed':9,'diagnostics':8,'assertion_triviality':10},
    [{'severity':'medium','category':'coverage_depth','detail':'Missing: zero-size image normalization, multiple shapes, non-square image'}],
    'Good normalize tests with exact coordinate math validation. Missing edge cases for zero-size images.')

update_file('tools/tiled-to-shapes/tests/suite/parse_tsx.rs',
    {'assertion_quality':9,'determinism':10,'isolation':10,'clarity':10,'coverage_depth':8,'speed':9,'diagnostics':9,'assertion_triviality':10},
    [{'severity':'low','category':'coverage_depth','detail':'Missing: multiple wangsets, wangset with mixed corner/edge types, invalid tile dimensions'}],
    'Excellent TSX parsing tests with happy path, no-wangset error, and malformed XML error paths.')

update_file('tools/tiled-to-shapes/tests/suite/pipeline.rs',
    {'assertion_quality':9,'determinism':10,'isolation':10,'clarity':9,'coverage_depth':8,'speed':10,'diagnostics':8,'assertion_triviality':10},
    [{'severity':'low','category':'coverage_depth','detail':'Missing: edge passability values, null/empty input to passability_to_tags'}],
    'Strong pipeline test with config validation and all passability tag mappings including unknown fallback.')

update_file('tools/tiled-to-shapes/tests/suite/scaffold.rs',
    {'assertion_quality':9,'determinism':10,'isolation':10,'clarity':9,'coverage_depth':8,'speed':9,'diagnostics':8,'assertion_triviality':10},
    [{'severity':'low','category':'coverage_depth','detail':'Missing: partial properties (only some present), invalid XML input to scaffold'}],
    'Excellent scaffold tests covering property injection, idempotency, and no-corner-wangset no-op.')

# ---- engine crates ----
update_file('crates/card_game/tests/suite/terrain.rs',
    {'assertion_quality':9,'determinism':10,'isolation':10,'clarity':8,'coverage_depth':7,'speed':10,'diagnostics':8,'assertion_triviality':10},
    [{'severity':'medium','category':'coverage_depth','detail':'Tests only empty-codegen path. Missing: non-empty tileset when TSX assets present, tile field access, adjacency_rules structure'}],
    'Valid terrain empty-codegen tests with good diagnostics. Limited to empty path since no TSX assets exist.')

update_file('crates/engine_audio/tests/suite/backend_cpal.rs',
    {'assertion_quality':8,'determinism':8,'isolation':10,'clarity':9,'coverage_depth':8,'speed':9,'diagnostics':8,'assertion_triviality':10},
    [{'severity':'medium','category':'determinism','detail':'One test uses real CPAL hardware init. Remaining 9 tests use NullAudioBackend, which is deterministic.'},
     {'severity':'low','category':'coverage_depth','detail':'Missing: CPAL backend stop/volume tests. All non-trivial path tests are on Null backend'}],
    'Solid backend tests with 10 tests covering CPAL uniqueness and Null backend full lifecycle. One hardware-dependent test.')

update_file('crates/engine_render/tests/suite/wgpu_renderer_types.rs',
    {'assertion_quality':1,'determinism':10,'isolation':10,'clarity':6,'coverage_depth':1,'speed':10,'diagnostics':1,'assertion_triviality':10},
    [{'severity':'high','category':'coverage_depth','detail':'Zero test functions. Source module has zero pub items - all types are pub(crate). Types tested indirectly through renderer integration tests.'},
     {'severity':'low','category':'structure','detail':'If types are never promoted to pub, this file can be removed or permanently documented as untestable.'}],
    'Placeholder - source types are all pub(crate), tested indirectly through integration tests. No public API to test.')

# Compute new stats
test_files = {k:v for k,v in m['files'].items() if not k.endswith('mod.rs') and not k.endswith('helpers.rs')}
scores = [v['overall'] for v in test_files.values()]
avg = sum(scores)/len(scores)
below_65 = [s for s in scores if s < 6.5]
below_80 = [s for s in scores if s < 8.0]

count_updated = sum(1 for k in m['files'] if m['files'][k].get('last_verified') == now)
print(f'Updated {count_updated} files')
print(f'New average: {avg:.2f}  (was 7.94)')
print(f'Below 6.5: {len(below_65)}  Below 8.0: {len(below_80)}')

# Update cycle
m['cycle']['current_average'] = round(avg, 2)
m['last_partial_run'] = now

with open(os.path.join(root, '.claude/test-verification/manifest.json'), 'w') as f:
    json.dump(m, f, indent=2)
print('Manifest written.')
