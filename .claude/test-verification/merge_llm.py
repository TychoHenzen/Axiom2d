#!/usr/bin/env python3
"""Merge LLM classification results into manifest and recompute scores."""
import json, sys
sys.stdout.reconfigure(encoding='utf-8')

# All LLM classification results
llm_batches = [
    # Batch 1
    {"crates/engine_input/tests/suite/keyboard_state.rs": {"error_path_functions":0,"edge_case_functions":2,"happy_path_functions":14,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0},
     "crates/engine_physics/tests/suite/collider.rs": {"error_path_functions":0,"edge_case_functions":0,"happy_path_functions":2,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0},
     "crates/card_game/tests/suite/stash_constants.rs": {"error_path_functions":0,"edge_case_functions":5,"happy_path_functions":8,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":True,"zero_assertion_functions":0},
     "crates/engine_input/tests/suite/mouse_state.rs": {"error_path_functions":0,"edge_case_functions":1,"happy_path_functions":18,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0},
     "crates/axiom2d/tests/suite/splash_animation.rs": {"error_path_functions":0,"edge_case_functions":7,"happy_path_functions":14,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":True,"zero_assertion_functions":0},
     "crates/card_game/tests/suite/card_reader_spawn.rs": {"error_path_functions":0,"edge_case_functions":0,"happy_path_functions":6,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0},
     "crates/card_game/tests/suite/card_rendering_debug_spawn.rs": {"error_path_functions":0,"edge_case_functions":1,"happy_path_functions":4,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0},
     "crates/card_game/tests/suite/stash_toggle.rs": {"error_path_functions":0,"edge_case_functions":0,"happy_path_functions":2,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0}},
    # Batch 2
    {"crates/engine_audio/tests/suite/backend_traits.rs": {"error_path_functions":0,"edge_case_functions":0,"happy_path_functions":4,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0},
     "crates/engine_audio/tests/suite/sound_data.rs": {"error_path_functions":0,"edge_case_functions":0,"happy_path_functions":2,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0},
     "crates/engine_ecs/tests/schedule.rs": {"error_path_functions":0,"edge_case_functions":1,"happy_path_functions":2,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0},
     "crates/engine_input/tests/suite/action_map.rs": {"error_path_functions":0,"edge_case_functions":0,"happy_path_functions":2,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0},
     "crates/engine_input/tests/suite/keyboard_system.rs": {"error_path_functions":0,"edge_case_functions":0,"happy_path_functions":7,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0},
     "crates/engine_input/tests/suite/mouse_system.rs": {"error_path_functions":0,"edge_case_functions":0,"happy_path_functions":8,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0},
     "crates/engine_render/tests/suite/clear.rs": {"error_path_functions":0,"edge_case_functions":0,"happy_path_functions":1,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0},
     "crates/engine_ui/tests/suite/layout_system.rs": {"error_path_functions":0,"edge_case_functions":0,"happy_path_functions":6,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0}},
    # Batch 3
    {"crates/engine_ui/tests/suite/theme.rs": {"error_path_functions":0,"edge_case_functions":0,"happy_path_functions":1,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0},
     "crates/engine_ui/tests/suite/widget_node.rs": {"error_path_functions":0,"edge_case_functions":0,"happy_path_functions":3,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0},
     "crates/axiom2d/tests/suite/default_plugins.rs": {"error_path_functions":0,"edge_case_functions":0,"happy_path_functions":17,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0},
     "crates/card_game/tests/suite/card_identity_name_pools_syllables.rs": {"error_path_functions":0,"edge_case_functions":1,"happy_path_functions":4,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0},
     "crates/card_game/tests/suite/card_interaction_intent_apply.rs": {"error_path_functions":0,"edge_case_functions":1,"happy_path_functions":14,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0},
     "crates/card_game/tests/suite/card_interaction_release_target.rs": {"error_path_functions":0,"edge_case_functions":3,"happy_path_functions":3,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0},
     "crates/card_game/tests/suite/card_reader_glow.rs": {"error_path_functions":0,"edge_case_functions":2,"happy_path_functions":3,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0},
     "crates/engine_audio/tests/suite/mixer_engine.rs": {"error_path_functions":0,"edge_case_functions":2,"happy_path_functions":6,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0}},
    # Batch 4
    {"crates/card_game/tests/suite/card_rendering_spawn_table_card.rs": {"error_path_functions":0,"edge_case_functions":7,"happy_path_functions":22,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0},
     "crates/card_game/tests/suite/card_zone_config.rs": {"error_path_functions":0,"edge_case_functions":0,"happy_path_functions":3,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0},
     "crates/card_game/tests/suite/stash_hover.rs": {"error_path_functions":0,"edge_case_functions":8,"happy_path_functions":5,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0},
     "crates/engine_physics/tests/suite/physics_step_system.rs": {"error_path_functions":0,"edge_case_functions":1,"happy_path_functions":3,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0},
     "crates/engine_ui/tests/suite/layout_anchor.rs": {"error_path_functions":0,"edge_case_functions":2,"happy_path_functions":6,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0},
     "crates/engine_ui/tests/suite/interaction.rs": {"error_path_functions":0,"edge_case_functions":6,"happy_path_functions":13,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0},
     "crates/engine_render/tests/suite/camera.rs": {"error_path_functions":0,"edge_case_functions":6,"happy_path_functions":15,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0},
     "crates/card_game/tests/suite/card_identity_nouns.rs": {"error_path_functions":0,"edge_case_functions":6,"happy_path_functions":4,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0}},
    # Batch 5
    {"crates/card_game/tests/suite/card_interaction_apply.rs": {"error_path_functions":0,"edge_case_functions":1,"happy_path_functions":3,"multi_behavior_count":2,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0},
     "crates/card_game/tests/suite/card_interaction_drag_state.rs": {"error_path_functions":0,"edge_case_functions":0,"happy_path_functions":6,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0},
     "crates/card_game/tests/suite/card_interaction_flip.rs": {"error_path_functions":0,"edge_case_functions":4,"happy_path_functions":5,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0},
     "crates/card_game/tests/suite/card_rendering_render_layer.rs": {"error_path_functions":0,"edge_case_functions":1,"happy_path_functions":3,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0},
     "crates/card_game/tests/suite/card_reader.rs": {"error_path_functions":1,"edge_case_functions":6,"happy_path_functions":14,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0},
     "crates/card_game/tests/suite/card_reader_components.rs": {"error_path_functions":0,"edge_case_functions":6,"happy_path_functions":2,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0},
     "crates/card_game/tests/suite/card_reader_signature_space.rs": {"error_path_functions":0,"edge_case_functions":3,"happy_path_functions":8,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0},
     "crates/card_game/tests/suite/card_reader_volume.rs": {"error_path_functions":0,"edge_case_functions":6,"happy_path_functions":4,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0}},
    # Batch 6
    {"crates/card_game/tests/suite/card_interaction_sleep.rs": {"error_path_functions":0,"edge_case_functions":5,"happy_path_functions":2,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0},
     "crates/card_game/tests/suite/card_interaction_click_resolve.rs": {"error_path_functions":0,"edge_case_functions":2,"happy_path_functions":4,"multi_behavior_count":1,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0},
     "crates/card_game/tests/suite/stash_pages.rs": {"error_path_functions":0,"edge_case_functions":9,"happy_path_functions":10,"multi_behavior_count":1,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0},
     "crates/engine_render/tests/suite/atlas.rs": {"error_path_functions":5,"edge_case_functions":8,"happy_path_functions":15,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0},
     "crates/card_game/tests/suite/card_identity_templates.rs": {"error_path_functions":0,"edge_case_functions":1,"happy_path_functions":17,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0},
     "crates/card_game/tests/suite/card_combiner_device.rs": {"error_path_functions":0,"edge_case_functions":1,"happy_path_functions":7,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0},
     "crates/card_game/tests/suite/card_rendering_bake.rs": {"error_path_functions":0,"edge_case_functions":1,"happy_path_functions":8,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0},
     "crates/card_game/tests/suite/card_rendering_overlay.rs": {"error_path_functions":0,"edge_case_functions":3,"happy_path_functions":5,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0},
     "crates/card_game/tests/suite/card_identity_definition.rs": {"error_path_functions":0,"edge_case_functions":1,"happy_path_functions":9,"multi_behavior_count":0,"creates_own_fixtures":True,"has_custom_matchers":False,"zero_assertion_functions":0}},
]

# Merge all batches into one dict
llm_all = {}
for batch in llm_batches:
    llm_all.update(batch)

print(f"LLM classifications available for {len(llm_all)} files")

# Load manifest
with open('.claude/test-verification/manifest.json') as f:
    manifest = json.load(f)

# Update classifications in manifest
updated = 0
for filepath, cls in llm_all.items():
    if filepath in manifest['files']:
        manifest['files'][filepath]['classifications'] = cls
        updated += 1
    else:
        print(f"WARNING: {filepath} not in manifest")

print(f"Updated {updated} files with LLM classifications")

# ── Recompute scores with LLM data ──

def score_coverage_depth(m, classifications):
    tf = max(m['test_function_count'], 1)
    ta = m['total_assertions']
    err = classifications.get('error_path_functions', 0)
    edge = classifications.get('edge_case_functions', 0)
    score = 3
    score += min(5, (err / tf) * 8)
    score += min(2, (edge / tf) * 4)
    score += min(1, (ta / tf) > 3 and 1 or ((ta / tf) / 3))
    return round(max(1, min(10, score)), 1)

def score_clarity(m, classifications):
    tf = max(m['test_function_count'], 1)
    gn = m.get('generic_names_count', 0)
    name_quality = (tf - gn) / tf
    score = 2 + min(3, name_quality * 3)
    if m.get('has_aaa_markers'): score += 2
    magic = classifications.get('magic_numbers', [])
    magic_bad = [x for x in magic if x.get('classification') == 'should_be_named']
    score -= min(2.5, max(0, len(magic_bad) - 1) * 0.5)
    mb = classifications.get('multi_behavior_count', 0)
    score -= min(2, mb)
    return round(max(1, min(10, score)), 1)

def score_diagnostics(m, classifications):
    ta = max(m['total_assertions'], 1)
    msg = m.get('assertions_with_message', 0)
    score = 2
    score += min(4.5, (msg / ta) * 5.5)
    if m.get('framework_shows_diff', True): score += 2.5
    if classifications.get('has_custom_matchers', False): score += 1
    return round(max(1, min(10, score)), 1)

def score_isolation(m, classifications):
    score = 7.0
    if m.get('has_setup_teardown'): score += 1.5
    if m.get('has_test_only'): score -= 2
    if classifications.get('creates_own_fixtures', True): score += 1
    return round(max(1, min(10, score)), 1)

def overall(a, d, i, c, cd, s, di, at):
    return round((a*2 + d*2 + i*1 + c*1 + cd*2 + s*1 + di*1 + at*1) / 11, 1)

for filepath, file_entry in manifest['files'].items():
    m = file_entry['metrics']
    cls = file_entry.get('classifications', {})
    sc = file_entry['scores']

    # Recompute LLM-dependent scores
    sc['coverage_depth'] = score_coverage_depth(m, cls)
    sc['clarity'] = score_clarity(m, cls)
    sc['diagnostics'] = score_diagnostics(m, cls)
    sc['isolation'] = score_isolation(m, cls)

    file_entry['overall'] = overall(
        sc['assertion_quality'], sc['determinism'], sc['isolation'],
        sc['clarity'], sc['coverage_depth'], sc['speed'], sc['diagnostics'],
        sc['assertion_triviality']
    )

# Stats
overalls = [f['overall'] for f in manifest['files'].values()]
avg = sum(overalls) / max(len(overalls), 1)
at_target = sum(1 for o in overalls if o >= 8.0)
below_min = sum(1 for o in overalls if o < 6.5)
below_8 = sum(1 for o in overalls if o < 8.0)

print(f"\nAfter LLM classification merge:")
print(f"Files: {len(manifest['files'])}")
print(f"Avg score: {avg:.2f}")
print(f"At target (>=8.0): {at_target}/{len(manifest['files'])} ({at_target/len(manifest['files'])*100:.0f}%)")
print(f"Below minimum (<6.5): {below_min}")
print(f"Below 8.0: {below_8}")

# Update cycle results
manifest['cycle']['results']['test_avg_score'] = round(avg, 1)
manifest['cycle']['results']['files_at_target'] = at_target
manifest['cycle']['results']['files_below_minimum'] = below_min
manifest['cycle']['results']['files_below_threshold'] = below_8
manifest['cycle']['results']['total_files'] = len(manifest['files'])

# Show worst 10
worst = sorted(manifest['files'].items(), key=lambda x: x[1]['overall'])[:10]
print(f"\nWorst 10 files after LLM merge:")
for p, f in worst:
    sc = f['scores']
    print(f"  {f['overall']} {p}")

# Compute source quality stats
src = manifest['source_quality']
obs_scores = [d['observability']['score'] for d in src.values()]
brev_scores = [d['brevity']['score'] for d in src.values()]
obs_avg = sum(obs_scores) / max(len(obs_scores), 1)
brev_avg = sum(brev_scores) / max(len(brev_scores), 1)
obs_below = sum(1 for s in obs_scores if s < 7)
brev_below = sum(1 for s in brev_scores if s < 7)
obs_min = min(obs_scores) if obs_scores else None

manifest['cycle']['results']['source_observability_avg'] = round(obs_avg, 1)
manifest['cycle']['results']['source_observability_min'] = obs_min
manifest['cycle']['results']['source_brevity_avg'] = round(brev_avg, 1)
manifest['cycle']['results']['source_brevity_below_7'] = brev_below

# Save
with open('.claude/test-verification/manifest.json', 'w') as f:
    json.dump(manifest, f, indent=2)

print(f"\nManifest updated: .claude/test-verification/manifest.json")
print(f"\nSource quality summary:")
print(f"  Observability: avg {obs_avg:.1f}, min {obs_min}, below 7: {obs_below}")
print(f"  Brevity: avg {brev_avg:.1f}, below 7: {brev_below}")
