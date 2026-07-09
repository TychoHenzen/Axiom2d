#!/usr/bin/env python3
"""Static analysis for Rust test and source files. Regex-based, deterministic.
Covers: Steps 1e (test metrics), 1d (source quality), 1c (coverage mapping)."""

import json
import os
import re
import hashlib
from pathlib import Path

ROOT = Path(os.getcwd())

def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

# ── Rust test file patterns ──

# Assertion patterns
ASSERT_MACRO = re.compile(r'(assert!\s*\(|assert_eq!\s*\(|assert_ne!\s*\(|debug_assert!\s*\(|debug_assert_eq!\s*\(|debug_assert_ne!\s*\()')
ASSERT_WITH_MESSAGE = re.compile(r'assert[_!][a-z_]*!\s*\([^,)]*,\s*"')

# Rust-specific: methods that indicate a SPECIFIC value check (not truthiness)
SPECIFIC_METHODS = re.compile(r'\.(is_ok|is_err|is_some|is_none|is_empty|contains|is_finite|is_nan|is_normal|is_sign_positive|is_sign_negative|is_ascii|is_alphanumeric|is_digit|is_lowercase|is_uppercase|is_whitespace|starts_with|ends_with|eq|ne|lt|le|gt|ge)\(')
# assert! with a comparison operator = specific
COMPARISON_ASSERT = re.compile(r'assert!\s*\([^)]*[=<>!+\-*/%&|^]')
# Pure truthiness: assert!(bare_boolean) or assert!(!bare_boolean)
TRUTHINESS_PURE = re.compile(r'assert!\s*\(\s*!?\s*\w+\s*\)')

# Determinism
INSTANT_NOW = re.compile(r'Instant::now\(\)')
RAND_RANDOM = re.compile(r'rand::random\(\)|\.gen::<|thread_rng\(\)')
SLEEP = re.compile(r'sleep\(|thread::sleep\(|tokio::time::sleep\(|Delay::new\(')
FS_OPEN = re.compile(r'File::open\(|fs::read\(|fs::write\(|OpenOptions::new\(')
NET_CALL = re.compile(r'TcpStream::connect\(|reqwest::|ureq::|hyper::')
DB_CALL = re.compile(r'sqlx::|rusqlite::|diesel::|mongodb::')
SHARED_STATE = re.compile(r'static\s+mut\b|static\s+ref\b')

# Test functions
TEST_FN = re.compile(r'#\[test\]\s*\n\s*fn\s+(\w+)')
SHOULD_PANIC = re.compile(r'#\[should_panic')
TEST_ONLY_IGNORE = re.compile(r'#\[ignore\]')
GENERIC_NAMES = re.compile(r'fn\s+(test_\d+|test_basic|test_simple|test_works|test_test|test_foo|test_bar)\b')

# AAA markers
AAA_ARRANGE = re.compile(r'//\s*Arrange')
AAA_ACT = re.compile(r'//\s*Act')
AAA_ASSERT = re.compile(r'//\s*Assert')

# Setup/teardown
SETUP_FN = re.compile(r'fn\s+(setup|set_up|before_each|before_all)\b')
TEARDOWN_FN = re.compile(r'fn\s+(teardown|tear_down|after_each|after_all)\b')

# Magic numbers (in assertions, not 0,1,-1,2,42,100,200,404,500)
MAGIC_NUM = re.compile(r'assert[_!][a-z_]*!\s*\(.*?(\d+\.?\d*).*?\)')

# Trivial assertions (constant on constant)
TRIVIAL = re.compile(r'assert[_!][a-z_]*!\s*\(\s*(true|false|\d+\.?\d*)\s*,\s*(true|false|\d+\.?\d*)\s*\)')

# Logging patterns for observability
LOG_CALL = re.compile(r'(log::\w+!|tracing::\w+!|println!|eprintln!|info!|warn!|error!|debug!|trace!)')
EMPTY_CATCH = re.compile(r'\}\s*_?\s*=>\s*\{?\s*\}?\s*$')
ERROR_RETURN = re.compile(r'Err\([^)]*\)\s*\)\s*;?\s*$|return\s+Err\(')

# Function detection for brevity
FN_DEF = re.compile(r'^\s*(pub\s+)?(async\s+)?fn\s+(\w+)\s*\(')
IF_FOR_WHILE = re.compile(r'\b(if|for|while|match|loop)\b')
ELSE_AFTER_RETURN = re.compile(r'(return|break|continue);\s*\n\s*\}?\s*\n\s*else\s*\{')

# @doc annotations
DOC_ANNOTATION = re.compile(r'///\s*@doc:')

# Log message has variable interpolation
LOG_WITH_VARS = re.compile(r'(log::\w+!|tracing::\w+!|info!|warn!|error!|debug!|trace!)\s*\(\s*"[^"]*\{')

def analyze_test_file(filepath):
    """Run all regex metrics on a single Rust test file."""
    content = Path(filepath).read_text(encoding='utf-8', errors='ignore')
    lines = content.split('\n')

    # Count assertions by macro type
    # assert_eq!, assert_ne!, debug_assert_eq!, debug_assert_ne! = always specific
    eq_ne_count = len(re.findall(r'(?:debug_)?assert_eq!\s*\(', content)) + len(re.findall(r'(?:debug_)?assert_ne!\s*\(', content))

    # assert!, debug_assert! — classify: specific if contains method call (.is_ok(), etc.), operator, or message
    # Count all bare assert! macros
    bare_assert_total = len(re.findall(r'(?:debug_)?assert!\s*\(', content))

    # For each assert! call, look at full content including following lines
    bare_specific = 0
    for m in re.finditer(r'(?:debug_)?assert!\s*\(', content):
        start = m.start()
        # Capture from the '(' to the matching ');' — scan forward up to 500 chars
        rest = content[m.end()-1:]  # from '(' onward
        depth = 0
        body = ''
        for ch in rest:
            body += ch
            if ch == '(':
                depth += 1
            elif ch == ')':
                depth -= 1
                if depth == 0:
                    # found matching ), check for ;
                    break
        # Classify: specific if contains method call, operator, or custom message string
        is_specific = False
        if SPECIFIC_METHODS.search(body):
            is_specific = True
        elif re.search(r'[=<>!+\-*/%&|^]', body):
            is_specific = True
        elif re.search(r'"[^"]*"', body):  # has a custom message string, ergo checking specific value
            is_specific = True
        if is_specific:
            bare_specific += 1

    bare_truthiness = max(0, bare_assert_total - bare_specific)

    specific_count = eq_ne_count + bare_specific
    truthiness_count = bare_truthiness
    total_assertions = specific_count + truthiness_count

    # Messages
    messaged = len(ASSERT_WITH_MESSAGE.findall(content))
    unmessaged = max(0, total_assertions - messaged)

    # Trivial
    trivial = len(TRIVIAL.findall(content))

    # Test functions
    test_fns = TEST_FN.findall(content)
    test_fn_count = len(test_fns)
    should_panic_count = len(SHOULD_PANIC.findall(content))

    # Error path detection via should_panic + name patterns
    error_names = len([n for n in test_fns if any(kw in n.lower() for kw in ('error', 'fail', 'invalid', 'panic', 'reject', 'none', 'empty'))])

    # Edge case names
    edge_names = len([n for n in test_fns if any(kw in n.lower() for kw in ('edge', 'boundary', 'zero', 'empty', 'null', 'max', 'min', 'overflow'))])

    # Determinism flags
    has_real_time = bool(INSTANT_NOW.search(content))
    has_unseeded_random = bool(RAND_RANDOM.search(content))
    has_sleep = bool(SLEEP.search(content))
    has_real_fs = bool(FS_OPEN.search(content))
    has_real_net = bool(NET_CALL.search(content))
    has_real_db = bool(DB_CALL.search(content))
    has_shared_state = bool(SHARED_STATE.search(content))

    # Isolation
    has_setup = bool(SETUP_FN.search(content))
    has_teardown = bool(TEARDOWN_FN.search(content))
    has_test_only = bool(TEST_ONLY_IGNORE.search(content))

    # Clarity
    generic = len(GENERIC_NAMES.findall(content))
    has_aaa = bool(AAA_ARRANGE.search(content)) and bool(AAA_ACT.search(content)) and bool(AAA_ASSERT.search(content))

    # Magic numbers
    magic_count = 0
    well_known = {0, 1, 2, -1, 42, 100, 200, 201, 204, 301, 302, 400, 401, 403, 404, 409, 422, 500, 502, 503}
    for m in MAGIC_NUM.finditer(content):
        num_str = m.group(1)
        try:
            num = float(num_str)
            if num == int(num):
                num = int(num)
            if num not in well_known:
                magic_count += 1
        except ValueError:
            pass

    # Speed
    sleep_count = len(SLEEP.findall(content))
    io_count = (len(FS_OPEN.findall(content)) + len(NET_CALL.findall(content)) + len(DB_CALL.findall(content)))

    # Zero-assertion functions
    # Count test functions that have no assertions in their body
    zero_assert_count = 0
    if test_fn_count > 0:
        # Simple heuristic: flag test functions without any assertion macro
        fn_pattern = re.compile(r'#\[test\]\s*(?:\n\s*#\[[^\]]*\])*\s*\n\s*(?:pub\s+)?fn\s+(\w+)\s*\([^)]*\)\s*\{(.*?)\n\}', re.DOTALL)
        for fn_match in fn_pattern.finditer(content):
            fn_body = fn_match.group(2)
            if not ASSERT_MACRO.search(fn_body) and 'should_panic' not in content[fn_match.start():fn_match.start()+200]:
                zero_assert_count += 1

    # @doc annotations
    doc_count = len(DOC_ANNOTATION.findall(content))

    return {
        "test_function_count": test_fn_count,
        "total_assertions": total_assertions,
        "trivial_assertions": trivial,
        "truthiness_assertions": truthiness_count,
        "specific_assertions": specific_count,
        "zero_assertion_functions": zero_assert_count,
        "assertions_with_message": messaged,
        "assertions_without_message": unmessaged,
        "has_real_time": has_real_time,
        "has_unseeded_random": has_unseeded_random,
        "has_sleep": has_sleep,
        "has_real_filesystem": has_real_fs,
        "has_real_network": has_real_net,
        "has_real_database": has_real_db,
        "has_shared_mutable_state": has_shared_state,
        "has_setup_teardown": has_setup or has_teardown,
        "has_test_only": has_test_only,
        "generic_names_count": generic,
        "has_aaa_markers": has_aaa,
        "magic_number_count": magic_count,
        "sleep_wait_count": sleep_count,
        "real_io_count": io_count,
        "framework_shows_diff": True,  # Rust assert_eq! shows diff
        "has_custom_matchers": False,  # Requires LLM
        "creates_own_fixtures": True,  # Requires LLM verification
        "should_panic_count": should_panic_count,
        "error_name_count": error_names,
        "edge_name_count": edge_names,
        "doc_annotation_count": doc_count,
        "line_count": len(lines),
    }

def analyze_source_file(filepath):
    """Observability + brevity analysis for source files."""
    content = Path(filepath).read_text(encoding='utf-8', errors='ignore')
    lines = content.split('\n')
    line_count = len(lines)

    # ── Observability ──
    log_calls = LOG_CALL.findall(content)
    log_count = len(log_calls)

    # Error handlers: match blocks with Err / .map_err / .or_else etc
    err_pattern = re.compile(r'Err\(|\.map_err\(|\.or_else\(|if\s+let\s+Err\(|match\s+\w+\s*\{[^}]*Err')
    error_handlers = err_pattern.findall(content)
    error_handler_count = len(error_handlers)

    # Error handlers that log: check if there's a log call within error handling blocks
    error_handlers_logged = 0
    for m in re.finditer(r'(match\s+\w+\s*\{|if\s+let\s+Err\()', content):
        # Search within next 500 chars for a log call
        nearby = content[m.start():m.start()+500]
        if LOG_CALL.search(nearby):
            error_handlers_logged += 1
    # Cap at error_handler_count
    error_handlers_logged = min(error_handlers_logged, max(error_handler_count, 1))

    # Anti-patterns
    anti_patterns = []

    # Empty catch-equivalent (empty match arm for Err)
    empty_err = re.findall(r'Err\([^)]*\)\s*=>\s*\{\s*\}', content)
    if empty_err:
        anti_patterns.append({"type": "empty_catch", "count": len(empty_err)})

    # Swallowed errors (Err returned/ignored without logging)
    if not LOG_CALL.search(content) and error_handler_count > 0 and line_count > 20:
        anti_patterns.append({"type": "no_logs_with_errors", "detail": f"{error_handler_count} error handlers, 0 logs"})

    # Bare static log messages
    log_msgs = re.findall(r'(log::\w+!|tracing::\w+!|info!|warn!|error!|debug!)\s*\(\s*"([^"]*)"', content)
    bare_static = 0
    for level, msg in log_msgs:
        if '{' not in msg:
            bare_static += 1
    if bare_static > 0 and len(log_msgs) > 0 and bare_static / len(log_msgs) > 0.5:
        anti_patterns.append({"type": "bare_static_logs", "count": bare_static})

    # Score: start at 10, subtract
    obs_score = 10
    obs_score -= min(3, len(anti_patterns) * 1.5)
    if log_count == 0 and line_count > 50:
        obs_score -= 2
    if error_handler_count > 0 and error_handlers_logged < error_handler_count:
        obs_score -= min(2, (error_handler_count - error_handlers_logged))
    obs_score = max(1, int(obs_score))

    # ── Brevity ──
    # Long lines
    long_lines = sum(1 for l in lines if len(l) > 120 and not l.strip().startswith('//') and not l.strip().startswith('///'))

    # Function detection and length
    fn_starts = []
    for i, line in enumerate(lines):
        if re.match(r'^\s*(pub\s+)?(async\s+)?fn\s+', line):
            fn_starts.append((i, line.strip()))

    long_fns = []
    high_cc_fns = []
    unnecessary_else = 0

    for fn_i, (start_idx, sig) in enumerate(fn_starts):
        # Find matching closing brace
        depth = 0
        end_idx = start_idx
        started = False
        for j in range(start_idx, min(line_count, start_idx + 500)):
            l = lines[j]
            for ch in l:
                if ch == '{':
                    depth += 1
                    started = True
                elif ch == '}':
                    depth -= 1
            if started and depth == 0:
                end_idx = j
                break
        fn_len = end_idx - start_idx + 1
        fn_body = '\n'.join(lines[start_idx:end_idx+1])

        if fn_len > 30:
            name_match = re.search(r'fn\s+(\w+)', sig)
            fn_name = name_match.group(1) if name_match else 'unknown'
            long_fns.append({"name": fn_name, "line": start_idx + 1, "length": fn_len})

        # Cyclomatic complexity
        cc = 1  # base
        cc += len(re.findall(r'\bif\b', fn_body))
        cc += len(re.findall(r'\bwhile\b', fn_body))
        cc += len(re.findall(r'\bfor\b', fn_body))
        cc += len(re.findall(r'\bmatch\b', fn_body))
        cc += len(re.findall(r'\bloop\b', fn_body))
        cc += len(re.findall(r'&&', fn_body))
        cc += len(re.findall(r'\|\|', fn_body))
        cc += len(re.findall(r'\?\?', fn_body))

        if cc > 5:
            name_match = re.search(r'fn\s+(\w+)', sig)
            fn_name = name_match.group(1) if name_match else 'unknown'
            high_cc_fns.append({"name": fn_name, "line": start_idx + 1, "cc": cc})

        # Unnecessary else (else after return/break/continue)
        if re.search(r'(return|break|continue)\s*;', fn_body) and 'else' in fn_body:
            unnecessary_else += 1

    # File too long
    file_too_long = line_count > 300

    # Brevity score
    brevity_score = 10
    if long_lines > 0:
        brevity_score -= min(2, long_lines * 0.2)
    brevity_score -= min(4, len(long_fns) * 1.5)
    brevity_score -= min(3, len(high_cc_fns) * 0.75)
    if file_too_long:
        brevity_score -= 1.5
    brevity_score -= min(2, unnecessary_else * 0.75)
    brevity_score = max(1, int(brevity_score))

    return {
        "hash": sha256(Path(filepath)),
        "lines": line_count,
        "functions": len(fn_starts),
        "observability": {
            "score": obs_score,
            "log_statements": log_count,
            "error_handlers": max(error_handler_count, 1),
            "error_handlers_logged": error_handlers_logged,
            "anti_patterns": anti_patterns
        },
        "brevity": {
            "score": brevity_score,
            "long_lines": long_lines,
            "long_functions": long_fns,
            "file_too_long": file_too_long,
            "high_complexity_functions": high_cc_fns,
            "unnecessary_else_count": unnecessary_else,
        }
    }

def map_coverage(source_files, test_files):
    """Map source files to test files."""
    coverage = []
    untested = []
    possibly_tested = []

    test_basenames = {}
    for tf in test_files:
        base = Path(tf).stem
        # Strip _test suffix variants
        clean = base.replace('_test', '').replace('test_', '')
        test_basenames[clean] = tf
        test_basenames[base] = tf

    for sf in source_files:
        s_stem = Path(sf).stem
        s_path = Path(sf)
        s_dir = s_path.parent

        # Strategy 1: direct name match
        found = False
        if s_stem in test_basenames:
            found = True

        # Strategy 2: check if test file exists in tests/ mirroring src/
        if not found:
            # e.g., src/foo/bar.rs -> tests/suite/bar.rs or tests/suite/foo_bar.rs
            pkg = s_path.parts[1] if len(s_path.parts) > 2 else 'unknown'
            suite_dir = Path(f'crates/{pkg}/tests/suite')
            if suite_dir.exists():
                for tf in suite_dir.glob('*.rs'):
                    tf_stem = tf.stem
                    if s_stem in tf_stem or tf_stem in s_stem:
                        found = True
                        test_basenames[s_stem] = str(tf)
                        break

        lines = sum(1 for _ in open(sf, encoding='utf-8', errors='ignore'))

        if found:
            matched = test_basenames.get(s_stem, '')
            entry = {
                "path": sf,
                "lines": lines,
                "test_match": "covered",
                "matching_test": matched,
                "risk": "low"
            }
            coverage.append(entry)
        else:
            risk = "low"
            if lines > 50:
                risk = "high"
            elif lines > 20:
                risk = "medium"

            entry = {
                "path": sf,
                "lines": lines,
                "test_match": "untested",
                "matching_test": None,
                "risk": risk
            }
            untested.append(entry)

    return coverage, untested, possibly_tested


def main():
    with open('.claude/test-verification/file_lists.json') as f:
        data = json.load(f)

    test_files = data['test_files']
    source_files = data['source_files']

    print(f"Analyzing {len(test_files)} test files and {len(source_files)} source files...")

    # Step 1e: Test file metrics
    test_metrics = {}
    for i, tf in enumerate(test_files):
        if i % 20 == 0:
            print(f"  Test file {i+1}/{len(test_files)}...")
        try:
            test_metrics[tf] = analyze_test_file(tf)
        except Exception as e:
            print(f"  ERROR {tf}: {e}")
            test_metrics[tf] = {"error": str(e)}

    # Step 1d: Source file analysis
    source_quality = {}
    for i, sf in enumerate(source_files):
        if i % 50 == 0:
            print(f"  Source file {i+1}/{len(source_files)}...")
        try:
            source_quality[sf] = analyze_source_file(sf)
        except Exception as e:
            print(f"  ERROR {sf}: {e}")
            source_quality[sf] = {"error": str(e)}

    # Step 1c: Coverage mapping
    coverage, untested, possibly_tested = map_coverage(source_files, test_files)

    total_source = len(source_files)
    tested = len([c for c in coverage if c["test_match"] == "covered"])

    print(f"\nResults:")
    print(f"  Test files: {len(test_files)}")
    print(f"  Source files: {total_source}")
    print(f"  Covered: {tested}")
    print(f"  Untested: {len(untested)}")
    print(f"  Possibly tested: {len(possibly_tested)}")

    # Check source quality
    obs_below_7 = [(p, d['observability']['score']) for p, d in source_quality.items() if d['observability']['score'] < 7]
    brev_below_7 = [(p, d['brevity']['score']) for p, d in source_quality.items() if d['brevity']['score'] < 7]
    print(f"  Observability below 7: {len(obs_below_7)}")
    print(f"  Brevity below 7: {len(brev_below_7)}")

    # Save
    output = {
        "test_metrics": test_metrics,
        "source_quality": source_quality,
        "coverage": {
            "total_source_files": total_source,
            "tested": tested,
            "possibly_tested": len(possibly_tested),
            "untested": len(untested),
            "untested_files": untested,
            "weakly_covered": [],
            "entry_points_untested": []
        }
    }

    with open('.claude/test-verification/static_results.json', 'w') as f:
        json.dump(output, f, indent=2)

    print(f"\nSaved to .claude/test-verification/static_results.json")

if __name__ == '__main__':
    main()
