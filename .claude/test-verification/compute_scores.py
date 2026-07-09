#!/usr/bin/env python3
"""Compute scores from static analysis + heuristics for LLM-dependent dimensions.
Outputs manifest-compatible JSON with scores, metrics, and classifications."""
import json, hashlib, os, sys
from pathlib import Path
from datetime import datetime, timezone

# Fix Windows cp1252 encoding
sys.stdout.reconfigure(encoding='utf-8')

ROOT = Path(os.getcwd())

def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()

# ── Scoring Formulas (from rubric) ──

def score_assertion_quality(m):
    tf = max(m['test_function_count'], 1)
    ta = m['total_assertions']
    sp = m['specific_assertions']
    za = m['zero_assertion_functions']
    density = min(3.5, (ta / tf) * 1.8)
    specificity = min(4, (sp / max(ta, 1)) * 4)
    coverage = min(2.5, ((tf - max(0, za - 1)) / tf) * 2.5)
    return round(1 + density + specificity + coverage, 1)

def score_determinism(m):
    violations = 0
    if m.get('has_real_time'): violations += 1
    if m.get('has_unseeded_random'): violations += 1
    violations += m.get('sleep_wait_count', 0)
    if m.get('has_real_filesystem'): violations += 1
    if m.get('has_real_network'): violations += 1
    if m.get('has_real_database'): violations += 1
    if m.get('has_shared_mutable_state'): violations += 0.5
    return round(max(2, 10 - violations * 1.3), 1)

def score_isolation(m, classifications):
    score = 7.0
    if m.get('has_setup_teardown'): score += 1.5
    score -= min(2, max(0, 0 - 2) * 0.75)  # module_mutable_count not tracked for Rust
    if m.get('has_test_only'): score -= 2
    if classifications.get('creates_own_fixtures', True): score += 1
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

def score_coverage_depth(m, classifications):
    tf = max(m['test_function_count'], 1)
    ta = m['total_assertions']
    err = classifications.get('error_path_functions', m.get('should_panic_count', 0))
    edge = classifications.get('edge_case_functions', 0)
    score = 3
    score += min(5, (err / tf) * 8)
    score += min(2, (edge / tf) * 4)
    score += min(1, (ta / tf) > 3 and 1 or ((ta / tf) / 3))
    return round(max(1, min(10, score)), 1)

def score_speed(m):
    violations = m.get('sleep_wait_count', 0) * 2 + m.get('real_io_count', 0) * 0.5
    return round(max(2, 10 - violations * 1.0), 1)

def score_diagnostics(m, classifications):
    ta = max(m['total_assertions'], 1)
    msg = m.get('assertions_with_message', 0)
    score = 2
    score += min(4.5, (msg / ta) * 5.5)
    if m.get('framework_shows_diff', True): score += 2.5
    if classifications.get('has_custom_matchers', False): score += 1
    return round(max(1, min(10, score)), 1)

def score_assertion_triviality(m):
    ta = max(m['total_assertions'], 1)
    triv = m.get('trivial_assertions', 0)
    if ta == 0: return 1
    if triv == 0: return 10
    return round(max(1, 10 - (triv / ta) * 10), 1)

def overall(a, d, i, c, cd, s, di, at):
    return round((a*2 + d*2 + i*1 + c*1 + cd*2 + s*1 + di*1 + at*1) / 11, 1)

# ── LLM heuristic classifications (for now — will be refined by actual LLM agents) ──

def heuristic_classifications(m, filepath):
    """Provide best-guess classifications from static data until LLM refinement."""
    tf = max(m['test_function_count'], 1)
    ta = m['total_assertions']

    # Coverage depth: use should_panic + name patterns
    sp = m.get('should_panic_count', 0)
    en = m.get('error_name_count', 0)
    edges = m.get('edge_name_count', 0)
    happy = tf - sp - edges
    error_path = max(sp, en)  # use higher of should_panic count vs name-based

    # Magic numbers: classify well-known ones
    magic = []
    well_known = {0, 1, 2, -1, 42, 100, 200, 201, 204, 301, 302, 400, 401, 403, 404, 409, 422, 500, 502, 503, 1000, 1024, 2048, 4096}

    # Multi-behavior: estimate from assertion density
    multi = 0
    if ta / tf > 8:  # very high assertion density suggests multi-behavior tests
        multi = max(0, int(tf * 0.15))

    # Custom matchers: check for helper functions in the file
    has_custom = filepath.endswith('helpers.rs') or 'test_helpers' in filepath

    return {
        "truthiness_assertions": m.get('truthiness_assertions', 0),
        "specific_assertions": m.get('specific_assertions', 0),
        "real_time_mocked": not m.get('has_real_time', False),  # if no real time, no issue
        "random_mocked": not m.get('has_unseeded_random', False),
        "filesystem_mocked": not m.get('has_real_filesystem', False),
        "network_mocked": not m.get('has_real_network', False),
        "database_mocked": not m.get('has_real_database', False),
        "has_effective_setup": None,
        "creates_own_fixtures": True,  # Rust idiom
        "magic_numbers": magic,
        "multi_behavior_count": multi,
        "happy_path_functions": max(0, happy),
        "error_path_functions": error_path,
        "edge_case_functions": edges,
        "has_custom_matchers": has_custom,
        "zero_assertion_functions": m.get('zero_assertion_functions', 0),
    }

# ── Main ──

def main():
    with open('.claude/test-verification/static_results.json') as f:
        static = json.load(f)
    with open('.claude/test-verification/file_lists.json') as f:
        lists = json.load(f)

    metrics = static['test_metrics']
    source_quality = static['source_quality']
    coverage = static['coverage']

    files = {}
    now = datetime.now(timezone.utc).isoformat()

    # Compute scores for each test file (exclude benches)
    for filepath in lists['test_files']:
        if '/benches/' in filepath:
            continue
        if filepath not in metrics:
            print(f"WARNING: {filepath} not in metrics, skipping")
            continue
        m = metrics[filepath]
        cls = heuristic_classifications(m, filepath)

        a_score = score_assertion_quality(m)
        d_score = score_determinism(m)
        i_score = score_isolation(m, cls)
        c_score = score_clarity(m, cls)
        cd_score = score_coverage_depth(m, cls)
        s_score = score_speed(m)
        di_score = score_diagnostics(m, cls)
        at_score = score_assertion_triviality(m)
        ov = overall(a_score, d_score, i_score, c_score, cd_score, s_score, di_score, at_score)

        # Generate findings
        findings = []
        if a_score <= 5:
            findings.append({"severity": "high", "category": "assertion_quality", "location": "file", "detail": f"Assertion quality {a_score}: {m.get('truthiness_assertions',0)}/{m['total_assertions']} truthiness", "suggestion": "Replace truthiness assertions with specific value checks"})
        if d_score <= 5:
            findings.append({"severity": "high", "category": "determinism", "location": "file", "detail": f"Determinism {d_score}", "suggestion": "Mock time/random/IO sources"})
        if cd_score <= 5:
            findings.append({"severity": "high", "category": "coverage_depth", "location": "file", "detail": f"Coverage depth {cd_score}: {cls['error_path_functions']}/{m['test_function_count']} error path tests", "suggestion": "Add error path and edge case tests"})
        if di_score <= 3:
            findings.append({"severity": "medium", "category": "diagnostics", "location": "file", "detail": f"Diagnostics {di_score}: {m['assertions_with_message']}/{m['total_assertions']} assertions with messages", "suggestion": "Add diagnostic messages to assertions"})
        if at_score <= 5:
            findings.append({"severity": "high", "category": "assertion_triviality", "location": "file", "detail": f"Trivial assertions: {m['trivial_assertions']}", "suggestion": "Replace constant-on-constant assertions"})
        if m.get('doc_annotation_count', 0) == 0 and m['test_function_count'] > 2:
            findings.append({"severity": "low", "category": "clarity", "location": "file", "detail": "No @doc: annotations on test functions", "suggestion": "Add /// @doc: annotations to describe test behaviors"})

        files[filepath] = {
            "hash": sha256(Path(filepath)),
            "last_verified": now,
            "scores": {
                "assertion_quality": a_score,
                "determinism": d_score,
                "isolation": i_score,
                "clarity": c_score,
                "coverage_depth": cd_score,
                "speed": s_score,
                "diagnostics": di_score,
                "assertion_triviality": at_score,
            },
            "overall": ov,
            "metrics": m,
            "classifications": cls,
            "findings": findings
        }

    # Stats
    overalls = [f['overall'] for f in files.values()]
    avg = sum(overalls) / max(len(overalls), 1)
    at_target = sum(1 for o in overalls if o >= 8.0)
    below_min = sum(1 for o in overalls if o < 6.5)
    below_8 = sum(1 for o in overalls if o < 8.0)

    print(f"Files: {len(files)}")
    print(f"Avg score: {avg:.1f}")
    print(f"At target (≥8.0): {at_target}/{len(files)} ({at_target/len(files)*100:.0f}%)")
    print(f"Below minimum (<6.5): {below_min}")
    print(f"Below 8.0: {below_8}")

    # Top 10 worst
    worst = sorted(files.items(), key=lambda x: x[1]['overall'])[:10]
    print(f"\nWorst 10 files:")
    for p, f in worst:
        sc = f['scores']
        print(f"  {f['overall']} {p}")
        print(f"    AQ:{sc['assertion_quality']} D:{sc['determinism']} I:{sc['isolation']} CL:{sc['clarity']} CD:{sc['coverage_depth']} SP:{sc['speed']} DG:{sc['diagnostics']} AT:{sc['assertion_triviality']}")

    # Manifest
    manifest = {
        "project_root": str(ROOT.absolute()),
        "cycle": {
            "phase": "baseline",
            "cycle_number": 1,
            "last_file_processed": None,
            "files_verified_this_cycle": list(files.keys()),
            "files_fixed_this_cycle": [],
            "started_at": now,
            "last_updated": now,
            "tooling": {
                "language": "rust",
                "test_framework": "cargo-test",
                "full_test_command": "cargo test",
                "compile_check_command": "cargo check"
            },
            "target_average": 8.0,
            "target_min_file": 6.5,
            "target_coverage": 0.80,
            "target_line_coverage": 0.70,
            "target_branch_coverage": 0.50,
            "target_source_obs": 7,
            "target_source_brevity": 7,
            "results": {
                "test_avg_score": round(avg, 1),
                "file_coverage_pct": round(coverage['tested'] / max(coverage['total_source_files'], 1) * 100, 1),
                "source_observability_avg": None,
                "source_brevity_avg": None,
                "files_at_target": at_target,
                "files_below_minimum": below_min,
                "files_below_threshold": below_8,
                "total_files": len(files),
                "line_coverage_pct": None,
                "branch_coverage_pct": None
            }
        },
        "last_full_run": now,
        "last_partial_run": None,
        "files": files,
        "coverage": coverage,
        "source_quality": source_quality
    }

    with open('.claude/test-verification/manifest.json', 'w') as f:
        json.dump(manifest, f, indent=2)

    print(f"\nManifest written: .claude/test-verification/manifest.json")

if __name__ == '__main__':
    main()
