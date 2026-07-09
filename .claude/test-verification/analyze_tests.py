#!/usr/bin/env python3
"""Static analysis of Rust test files — extracts metrics via regex for test-verification skill."""
import json, re, hashlib, os, sys
from pathlib import Path
from collections import defaultdict

PROJECT_ROOT = Path(r"C:\Users\siriu\RustroverProjects\Axiom2d")

# Patterns for Rust test files
TEST_FN_RE = re.compile(r'#\s*\[(?:cfg\(test\)|test)\]\s*(?://[^\n]*\n\s*)*\s*fn\s+(\w+)')
FN_RE = re.compile(r'\bfn\s+(\w+)')
ASSERT_RE = re.compile(r'\b(assert!|assert_eq!|assert_ne!|assert_approx_eq!)')
TRUTHINESS_RE = re.compile(r'\bassert!\s*\(\s*[a-z_]\w*\s*\)')  # assert!(x) bare boolean
TRIVIAL_RE = re.compile(r'(?:assert_eq!\s*\(\s*(?:true|false|\d+(?:\.\d+)?)\s*,\s*(?:true|false|\d+(?:\.\d+)?)\s*\)|assert!\s*\(\s*(?:true|false)\s*\))')
ASSERT_WITH_MSG_RE = re.compile(r'(?:assert_eq!|assert_ne!|assert!)\s*\([^)]*,\s*"[^"]*"\s*\)')
SHOULD_PANIC_RE = re.compile(r'#\s*\[should_panic')
IS_ERR_RE = re.compile(r'\.is_err\(\)|\.unwrap_err\(\)|assert!\([^)]*\.is_err\(\)')
REAL_TIME_RE = re.compile(r'(?:Instant::now|SystemTime::now|std::time::Instant)')
RANDOM_RE = re.compile(r'(?:rand::random|ChaCha8Rng|thread_rng|StdRng)')
SEEDED_RNG_RE = re.compile(r'(?:ChaCha8Rng::seed_from_u64|StdRng::seed_from_u64|SeedableRng)')
SLEEP_RE = re.compile(r'(?:thread::sleep|tokio::time::sleep|sleep\s*\()')
FS_RE = re.compile(r'(?:std::fs::|File::open|fs::read|fs::write|tempfile|tempdir|TempDir)')
NETWORK_RE = re.compile(r'(?:reqwest|ureq|hyper|TcpStream|UdpSocket)')
DB_RE = re.compile(r'(?:sqlx|diesel|rusqlite|mongodb|redis)')
STATIC_MUT_RE = re.compile(r'(?:static\s+mut|static\s+[A-Z]|lazy_static!|once_cell::sync::Lazy|Lazy::new|Mutex::new\([^)]*\)\s*;|RwLock::new\([^)]*\)\s*;)')
SETUP_RE = re.compile(r'fn\s+(?:setup|teardown|set_up|tear_down|before_each|after_each)')
IGNORE_RE = re.compile(r'#\s*\[ignore\]')
AAA_MARKER_RE = re.compile(r'//\s*(?:Arrange|Act|Assert)')
GENERIC_NAME_RE = re.compile(r'fn\s+(?:test_\d+|test_1|test_basic|test_simple|test_works|test_foo|test_bar)\b')
MAGIC_NUMBER_RE = re.compile(r'(?:assert_eq!|assert_ne!|assert!)\s*\([^)]*\b(\d+(?:\.\d+)?)\b[^)]*\)')

# Source analysis patterns
LOG_RE = re.compile(r'(?:log::(?:info|warn|error|debug|trace)!|tracing::(?:info|warn|error|debug|trace)!|println!|eprintln!|console::log)')
CATCH_RE = re.compile(r'(?:\.map_err\(|\.or_else\(|match\s+\w+\s*\{[^}]*Err|if\s+let\s+Err)')
EMPTY_CATCH_RE = re.compile(r'\.map_err\(\s*\|_\|\s*\{?\s*\}?\s*\)')
SWALLOWED_ERR_RE = re.compile(r'(?:\.unwrap_or_default\(\)|\.unwrap_or\([^)]*\)\s*$|\.ok\(\)\s*$|let\s+_\s*=\s*\w+\s*;)')
BARELOG_RE = re.compile(r'(?:log::(?:info|warn|error|debug|trace)!\s*\("[^"{]*"\))')

# Well-known magic numbers
WELL_KNOWN = {0, 1, -1, 2, 10, 100, 200, 201, 204, 301, 302, 400, 401, 403, 404, 500, 502, 503, 42, 255, 256, 1024, 60, 3600}

def compute_cyclomatic_complexity(lines):
    """Count branching points in a function body: if, for, while, loop, match, &&, ||, ?"""
    cc = 1  # base
    for line in lines:
        cc += len(re.findall(r'\b(?:if|for|while|loop|match)\b', line))
        cc += line.count('&&')
        cc += line.count('||')
        cc += line.count('?')
    return cc

def extract_test_functions(content):
    """Extract individual test function bodies with line numbers."""
    fns = []
    for m in TEST_FN_RE.finditer(content):
        name = m.group(1)
        start = m.start()
        # Find the function body by tracking braces
        brace_start = content.find('{', m.end())
        if brace_start == -1:
            continue
        depth = 0
        end = brace_start
        for i in range(brace_start, len(content)):
            if content[i] == '{':
                depth += 1
            elif content[i] == '}':
                depth -= 1
                if depth == 0:
                    end = i + 1
                    break
        body = content[brace_start:end]
        line_start = content[:start].count('\n') + 1
        fns.append({
            'name': name,
            'line': line_start,
            'body': body,
            'body_lines': body.count('\n')
        })
    return fns

def analyze_test_file(filepath):
    """Extract all static metrics from a Rust test file."""
    try:
        content = filepath.read_text(encoding='utf-8')
    except Exception:
        return None

    lines = content.split('\n')
    total_lines = len(lines)

    test_fns = extract_test_functions(content)
    test_fn_count = len(test_fns)
    test_fn_names = [f['name'] for f in test_fns]

    # Assertions
    all_assertions = ASSERT_RE.findall(content)
    total_assertions = len(all_assertions)
    truthiness_count = len(TRUTHINESS_RE.findall(content))
    trivial_count = len(TRIVIAL_RE.findall(content))
    messaged_count = len(ASSERT_WITH_MSG_RE.findall(content))
    unmessaged_count = total_assertions - messaged_count

    # Per-function assertion counts
    zero_assertion_fns = 0
    happy_path = 0
    error_path = 0
    edge_case = 0
    for fn in test_fns:
        fn_assertions = len(ASSERT_RE.findall(fn['body']))
        if fn_assertions == 0:
            zero_assertion_fns += 1

        # Error path detection
        is_error = bool(SHOULD_PANIC_RE.search(fn['name']) or IS_ERR_RE.search(fn['body']))
        # Also check function name hints
        name_lower = fn['name'].lower()
        if any(kw in name_lower for kw in ['error', 'fail', 'invalid', 'panic', 'reject', 'none', 'empty']):
            is_error = True
        if is_error:
            error_path += 1
        elif any(kw in name_lower for kw in ['edge', 'boundary', 'zero', 'max', 'min', 'overflow', 'underflow']):
            edge_case += 1
        else:
            happy_path += 1

    # Determinism flags
    has_real_time = bool(REAL_TIME_RE.search(content))
    has_random = bool(RANDOM_RE.search(content))
    has_seeded = bool(SEEDED_RNG_RE.search(content))
    has_unseeded = has_random and not has_seeded
    has_sleep = bool(SLEEP_RE.search(content))
    has_fs = bool(FS_RE.search(content))
    has_network = bool(NETWORK_RE.search(content))
    has_db = bool(DB_RE.search(content))
    has_shared_mutable = bool(STATIC_MUT_RE.search(content))

    # Isolation
    has_setup = bool(SETUP_RE.search(content))
    module_mutable_count = len(re.findall(r'static\s+mut', content)) + len(re.findall(r'static\s+[A-Z][A-Z_]+\s*:', content))
    has_ignore = bool(IGNORE_RE.search(content))

    # Clarity
    generic_names = [m.group(1) for m in GENERIC_NAME_RE.finditer(content)]
    has_aaa = bool(AAA_MARKER_RE.search(content))
    magic_numbers = []
    for m in MAGIC_NUMBER_RE.finditer(content):
        try:
            val = float(m.group(1))
            if val == int(val):
                val = int(val)
        except ValueError:
            continue
        if val not in WELL_KNOWN:
            magic_numbers.append(val)

    # Speed
    sleep_count = len(SLEEP_RE.findall(content))
    io_count = len(FS_RE.findall(content)) + len(NETWORK_RE.findall(content)) + len(DB_RE.findall(content))
    io_count = min(io_count, 5)

    # Coverage hints
    specific_assertions = total_assertions - truthiness_count - trivial_count

    return {
        'test_function_count': test_fn_count,
        'test_function_names': test_fn_names,
        'total_assertions': total_assertions,
        'specific_assertions': max(0, specific_assertions),
        'truthiness_assertions': truthiness_count,
        'trivial_assertions': trivial_count,
        'zero_assertion_functions': zero_assertion_fns,
        'assertions_with_message': messaged_count,
        'assertions_without_message': unmessaged_count,
        'has_real_time': has_real_time,
        'has_unseeded_random': has_unseeded,
        'has_sleep': has_sleep,
        'has_real_filesystem': has_fs,
        'has_real_network': has_network,
        'has_real_database': has_db,
        'has_shared_mutable_state': has_shared_mutable,
        'has_setup_teardown': has_setup,
        'module_mutable_count': module_mutable_count,
        'has_test_only': has_ignore,
        'creates_own_fixtures': None,  # LLM
        'generic_names': generic_names,
        'has_aaa_markers': has_aaa,
        'magic_number_count': len(magic_numbers),
        'multi_behavior_count': 0,  # LLM
        'happy_path_functions': happy_path,
        'error_path_functions': error_path,
        'edge_case_functions': edge_case,
        'sleep_wait_count': sleep_count,
        'real_io_count': io_count,
        'framework_shows_diff': True,  # Rust assert_eq! shows diff
        'has_custom_matchers': False,  # LLM
        'total_lines': total_lines,
        'magic_numbers_list': magic_numbers[:20],
    }

def analyze_source_file(filepath):
    """Extract observability and brevity metrics from a Rust source file."""
    try:
        content = filepath.read_text(encoding='utf-8')
    except Exception:
        return None

    lines = content.split('\n')
    total_lines = len(lines)
    file_too_long = total_lines > 300

    # Log statements
    log_count = len(LOG_RE.findall(content))

    # Error handlers
    catch_count = len(CATCH_RE.findall(content))
    empty_catch_count = len(EMPTY_CATCH_RE.findall(content))
    swallowed_count = len(SWALLOWED_ERR_RE.findall(content))
    bare_log_count = len(BARELOG_RE.findall(content))

    # Anti-patterns
    anti_patterns = []
    if empty_catch_count > 0:
        anti_patterns.append(f"{empty_catch_count} empty error handlers (silently discard errors)")
    if swallowed_count > 0:
        anti_patterns.append(f"{swallowed_count} swallowed errors (.unwrap_or_default(), .ok(), let _ =)")
    if bare_log_count > 0:
        anti_patterns.append(f"{bare_log_count} bare log statements (no variable interpolation)")

    # Observability score
    obs_penalties = empty_catch_count * 1.5 + swallowed_count * 0.5 + bare_log_count * 0.5
    # Penalize zero-log files
    if log_count == 0 and total_lines > 20:
        obs_penalties += 3
    if log_count == 0 and catch_count > 0:
        obs_penalties += 2  # error handlers with no logging
    obs_score = max(1, 10 - obs_penalties)

    # Brevity
    long_lines = sum(1 for l in lines if len(l) > 120)

    # Extract functions and their bounds
    fn_starts = [(m.group(1), m.start()) for m in re.finditer(r'\bfn\s+(\w+)\s*[<(]', content)]
    functions = []
    for i, (name, start) in enumerate(fn_starts):
        brace_start = content.find('{', start)
        if brace_start == -1:
            continue
        depth = 0
        end = brace_start
        for j in range(brace_start, len(content)):
            if content[j] == '{':
                depth += 1
            elif content[j] == '}':
                depth -= 1
                if depth == 0:
                    end = j
                    break
        body = content[brace_start:end]
        body_lines = body.count('\n')
        fn_lines = lines[content[:brace_start].count('\n'):content[:end].count('\n')+1]
        cc = compute_cyclomatic_complexity(fn_lines)
        functions.append({
            'name': name,
            'lines': body_lines,
            'cyclomatic_complexity': cc
        })

    fn_count = len(functions)
    long_fns = [f for f in functions if f['lines'] > 30]
    high_cc_fns = [f for f in functions if f['cyclomatic_complexity'] > 5]

    # Unnecessary else detection
    unnecessary_else = 0
    else_avoidable = 0
    for i, line in enumerate(lines):
        stripped = line.strip()
        if stripped == 'else {' or stripped.startswith('else {'):
            # Check if preceding block ends with return/break/continue
            prev_lines = '\n'.join(lines[max(0,i-5):i])
            if re.search(r'\b(?:return|break|continue)\s*[;}]', prev_lines):
                unnecessary_else += 1
            else:
                else_avoidable += 1

    brevity_penalties = (
        (long_lines * 0.3) +
        (len(long_fns) * 1) +
        (file_too_long * 2) +
        (len(high_cc_fns) * 1) +
        (unnecessary_else * 0.5) +
        (else_avoidable * 0.3)
    )
    brevity_score = max(1, 10 - brevity_penalties)

    return {
        'lines': total_lines,
        'functions': fn_count,
        'observability': {
            'score': round(obs_score, 1),
            'log_statements': log_count,
            'error_handlers': catch_count,
            'error_handlers_logged': catch_count - empty_catch_count,
            'anti_patterns': anti_patterns
        },
        'brevity': {
            'score': round(brevity_score, 1),
            'long_lines': long_lines,
            'long_functions': len(long_fns),
            'long_function_names': [f['name'] for f in long_fns],
            'file_too_long': file_too_long,
            'high_complexity_functions': len(high_cc_fns),
            'high_cc_function_names': [f['name'] for f in high_cc_fns],
            'unnecessary_else_count': unnecessary_else,
            'else_avoidable_count': else_avoidable,
        }
    }

def compute_scores(metrics):
    """Compute all 8 dimension scores from metrics using rubric formulas."""
    if metrics is None:
        return None

    m = metrics
    tfc = max(m['test_function_count'], 1)
    ta = max(m['total_assertions'], 1)

    # 1. Assertion Quality
    density = min(3.5, (m['total_assertions'] / tfc) * 1.8)
    specificity = min(4, (m['specific_assertions'] / ta) * 4)
    coverage = min(2.5, ((tfc - max(0, m['zero_assertion_functions'] - 1)) / tfc) * 2.5)
    assertion_quality = min(10, max(1, 1 + density + specificity + coverage))

    # 2. Determinism
    violations = (
        m['has_sleep'] * 1 +
        m['has_shared_mutable_state'] * 0.5 +
        m['has_real_time'] * 1 +
        m['has_unseeded_random'] * 1 +
        m['has_real_filesystem'] * 1 +
        m['has_real_network'] * 1 +
        m['has_real_database'] * 1
    )
    determinism = max(2, 10 - violations * 1.3)

    # 3. Isolation
    isolation = 7
    if m['has_setup_teardown']:
        isolation += 1.5
    isolation -= min(2, max(0, m['module_mutable_count'] - 2) * 0.75)
    if m['has_test_only']:
        isolation -= 2
    if m.get('creates_own_fixtures'):
        isolation += 1
    isolation = min(10, max(1, isolation))

    # 4. Clarity
    name_quality = (tfc - len(m['generic_names'])) / tfc
    clarity = 2
    clarity += min(3, name_quality * 3)
    if m['has_aaa_markers']:
        clarity += 2
    clarity -= min(2.5, max(0, m['magic_number_count'] - 1) * 0.5)
    clarity -= min(2, m.get('multi_behavior_count', 0))
    clarity = min(10, max(1, clarity))

    # 5. Coverage Depth
    cd = 3
    cd += min(5, (m['error_path_functions'] / tfc) * 8)
    cd += min(2, (m['edge_case_functions'] / tfc) * 4)
    cd += min(1, (m['total_assertions'] / tfc) > 3 and 1 or (m['total_assertions'] / tfc) / 3)
    coverage_depth = min(10, max(1, cd))

    # 6. Speed
    speed_violations = m['sleep_wait_count'] * 2 + m['real_io_count'] * 0.5
    speed = max(2, 10 - speed_violations * 1.0)

    # 7. Diagnostics
    diag = 2
    diag += min(4.5, (m['assertions_with_message'] / ta) * 5.5)
    if m['framework_shows_diff']:
        diag += 2.5
    if m.get('has_custom_matchers'):
        diag += 1
    diagnostics = min(10, max(1, diag))

    # 8. Assertion Triviality
    if ta == 0:
        triviality = 1
    elif m['trivial_assertions'] == 0:
        triviality = 10
    else:
        triviality = max(1, 10 - int(m['trivial_assertions'] / ta * 10))

    # Overall
    overall = (
        assertion_quality * 2 +
        determinism * 2 +
        isolation * 1 +
        clarity * 1 +
        coverage_depth * 2 +
        speed * 1 +
        diagnostics * 1 +
        triviality * 1
    ) / 11

    return {
        'assertion_quality': round(assertion_quality, 1),
        'determinism': round(determinism, 1),
        'isolation': round(isolation, 1),
        'clarity': round(clarity, 1),
        'coverage_depth': round(coverage_depth, 1),
        'speed': round(speed, 1),
        'diagnostics': round(diagnostics, 1),
        'assertion_triviality': round(triviality, 1),
        'overall': round(overall, 1),
    }

def sha256(filepath):
    h = hashlib.sha256()
    with open(filepath, 'rb') as f:
        for chunk in iter(lambda: f.read(65536), b''):
            h.update(chunk)
    return h.hexdigest()

def find_test_files():
    """Find all Rust test files, excluding mod.rs, main.rs, and helpers."""
    test_files = []
    for pattern in ['crates/*/tests/**/*.rs', 'tools/*/tests/**/*.rs']:
        for fp in PROJECT_ROOT.glob(pattern):
            name = fp.name
            if name in ('mod.rs', 'main.rs'):
                continue
            if 'helpers' in name.lower() or name == 'testing_mod.rs':
                continue
            rel = str(fp.relative_to(PROJECT_ROOT)).replace('\\', '/')
            test_files.append((fp, rel))
    return sorted(test_files, key=lambda x: x[1])

def find_source_files():
    """Find all production source files."""
    source_files = []
    for pattern in ['crates/*/src/**/*.rs', 'tools/*/src/**/*.rs']:
        for fp in PROJECT_ROOT.glob(pattern):
            name = fp.name
            if name in ('mod.rs', 'prelude.rs', 'lib.rs'):
                # Include these — they contain production code
                pass
            # Skip test helpers
            if 'test_helpers' in str(fp) or 'testing_mod' in str(fp):
                continue
            rel = str(fp.relative_to(PROJECT_ROOT)).replace('\\', '/')
            source_files.append((fp, rel))
    return sorted(source_files, key=lambda x: x[1])

def match_test_to_source(source_path):
    """Check if a source file has a matching test file."""
    # Extract the meaningful part of the path
    # e.g., crates/engine_core/src/color.rs -> look for engine_core/tests/**/color.rs
    parts = source_path.split('/')
    if len(parts) < 4:
        return None, 'unknown'

    crate = parts[1]
    # Remove src/ from path
    if 'src/' in source_path:
        after_src = source_path.split('src/')[1]
    else:
        after_src = '/'.join(parts[3:])

    stem = Path(after_src).stem

    # Check common patterns
    test_dir = PROJECT_ROOT / 'crates' / crate / 'tests'
    if not test_dir.exists():
        return None, 'untested'

    # Look for matching test files
    matches = []
    for tf in test_dir.rglob('*.rs'):
        if tf.name in ('mod.rs', 'main.rs'):
            continue
        if stem in tf.name or stem.replace('_', '') in tf.name.replace('_', ''):
            matches.append(str(tf.relative_to(PROJECT_ROOT)).replace('\\', '/'))

    if matches:
        return matches, 'covered'

    # Also check the content of mod.rs files for module declarations matching stem
    suite_mod = test_dir / 'suite' / 'mod.rs'
    if suite_mod.exists():
        try:
            content = suite_mod.read_text(encoding='utf-8')
            if f'mod {stem};' in content or f'pub mod {stem};' in content:
                return [f'{crate}/tests/suite/{stem}.rs'], 'covered'
        except Exception:
            pass

    return None, 'untested'

def estimate_risk(source_path, line_count):
    """Estimate risk level for untested source files."""
    # Entry points
    if source_path.endswith('main.rs'):
        return 'low'
    # Core business logic
    if line_count > 50:
        return 'high'
    elif line_count > 20:
        return 'medium'
    return 'low'

def main():
    print("=== Test Verification: Static Analysis ===", flush=True)

    test_files = find_test_files()
    print(f"\nFound {len(test_files)} test files", flush=True)

    source_files = find_source_files()
    print(f"Found {len(source_files)} source files", flush=True)

    # Analyze test files
    print("\n--- Analyzing test files ---", flush=True)
    file_results = {}
    for fp, rel in test_files:
        metrics = analyze_test_file(fp)
        if metrics is None:
            continue
        scores = compute_scores(metrics)
        if scores is None:
            continue
        fhash = sha256(fp)
        file_results[rel] = {
            'hash': fhash,
            'metrics': {k: v for k, v in metrics.items() if k != 'test_function_names' and k != 'magic_numbers_list'},
            'scores': scores,
            'overall': scores['overall'],
        }
        print(f"  {rel}: overall={scores['overall']:.1f}, assertions={metrics['total_assertions']}, fns={metrics['test_function_count']}", flush=True)

    # Analyze source files
    print("\n--- Analyzing source files ---", flush=True)
    source_results = {}
    coverage_data = {'tested': [], 'possibly_tested': [], 'untested': []}

    for fp, rel in source_files:
        sq = analyze_source_file(fp)
        if sq is None:
            continue
        fhash = sha256(fp)

        # Coverage mapping
        match, status = match_test_to_source(rel)
        line_count = sq['lines']
        risk = estimate_risk(rel, line_count) if status == 'untested' else ('low' if status == 'covered' else 'medium')

        cov_entry = {
            'path': rel,
            'lines': line_count,
            'test_match': status,
            'risk': risk,
            'line_coverage_pct': None,
            'branch_coverage_pct': None,
        }
        if match:
            cov_entry['matching_test'] = match

        if status == 'covered':
            coverage_data['tested'].append(cov_entry)
        elif status == 'possibly_tested':
            coverage_data['possibly_tested'].append(cov_entry)
        else:
            coverage_data['untested'].append(cov_entry)

        source_results[rel] = {
            'hash': fhash,
            'lines': line_count,
            'functions': sq['functions'],
            'observability': sq['observability'],
            'brevity': sq['brevity'],
        }

        if sq['observability']['score'] < 6 or sq['brevity']['score'] < 6:
            print(f"  {rel}: obs={sq['observability']['score']}, brevity={sq['brevity']['score']} !!", flush=True)

    # Compute averages
    all_scores = [r['overall'] for r in file_results.values()]
    avg = sum(all_scores) / max(len(all_scores), 1)
    below_65 = [(p, r['overall']) for p, r in file_results.items() if r['overall'] < 6.5]
    below_70 = [(p, r['overall']) for p, r in file_results.items() if r['overall'] < 7.0]

    print(f"\n=== Summary ===", flush=True)
    print(f"Test files: {len(file_results)}", flush=True)
    print(f"Average score: {avg:.1f}/10", flush=True)
    print(f"Below 6.5: {len(below_65)} files", flush=True)
    for p, s in sorted(below_65):
        print(f"  {p}: {s:.1f}", flush=True)
    print(f"Below 7.0: {len(below_70)} files", flush=True)

    # Output JSON for manifest
    output = {
        'test_file_count': len(file_results),
        'source_file_count': len(source_results),
        'average_overall': round(avg, 1),
        'below_min_65': [{'path': p, 'score': s} for p, s in sorted(below_65)],
        'below_threshold_70': [{'path': p, 'score': s} for p, s in sorted(below_70)],
        'files': file_results,
        'coverage': {
            'total_source_files': len(source_results),
            'tested': len(coverage_data['tested']),
            'possibly_tested': len(coverage_data['possibly_tested']),
            'untested': len(coverage_data['untested']),
            'line_coverage_available': False,
            'untested_files': [e for e in coverage_data['untested'] if e['risk'] in ('high', 'medium')],
            'weakly_covered': coverage_data['possibly_tested'],
            'entry_points_untested': [e for e in coverage_data['untested'] if e['risk'] == 'low'],
        },
        'source_quality': source_results,
    }

    outpath = PROJECT_ROOT / '.claude' / 'test-verification' / 'static_analysis.json'
    outpath.parent.mkdir(parents=True, exist_ok=True)
    outpath.write_text(json.dumps(output, indent=2), encoding='utf-8')
    print(f"\nOutput: {outpath}", flush=True)

if __name__ == '__main__':
    main()
