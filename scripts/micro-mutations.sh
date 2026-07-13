#!/usr/bin/env bash
# Micro-mutation testing — weighted random file selection per CI run.
# Biases toward stale, large, high-churn files. Over weeks, covers the codebase stochastically.
#
# Usage:
#   ./scripts/micro-mutations.sh            # Run with defaults (1 file, weighted random)
#   MICRO_SAMPLE_SIZE=3 ./scripts/micro-mutations.sh  # Run on 3 files
#   ./scripts/micro-mutations.sh --print-exclusions   # Show excluded paths
#   ./scripts/micro-mutations.sh --init-state          # Seed state file from scratch
#
# CI: called from quality.yml "micro-mutations" job.
# Reads/writes quality/.micro_mutation_state.json (machine-readable).
# Regenerates quality/MICRO_MUTATIONS.md (human-readable) each run.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
STATE_FILE="$PROJECT_ROOT/quality/.micro_mutation_state.json"
TRACKING_DOC="$PROJECT_ROOT/quality/MICRO_MUTATIONS.md"
MUTANTS_OUT_DIR="$PROJECT_ROOT/mutants.out"
SAMPLE_SIZE="${MICRO_SAMPLE_SIZE:-1}"
TIMEOUT="${MICRO_TIMEOUT:-45}"

# ─── Exclusions (mirrors .cargo/mutants.toml exclude_globs) ─────────────────────
EXCLUDE_PATTERNS=(
    '*/demo/*'
    '*/card_game_bin/*'
    '*/wgpu_renderer/*'
    '*/art/generated/*'
    '*/card_back.rs'
    '*/repository.rs'
    '*/hydrate.rs'
    '*/tests/*'
)

# ─── Parse args ─────────────────────────────────────────────────────────────────

if [[ "${1:-}" == "--print-exclusions" ]]; then
    echo "Excluded path patterns:"
    for pat in "${EXCLUDE_PATTERNS[@]}"; do
        echo "  $pat"
    done
    exit 0
fi

# ─── Helpers ─────────────────────────────────────────────────────────────────────

log() { echo ":: $*" >&2; }
warn() { echo "::warning::$*" >&2; }

eligible_files() {
    local find_args=("crates" "-name" "*.rs" "-type" "f")
    for pat in "${EXCLUDE_PATTERNS[@]}"; do
        find_args+=("-not" "-path" "$pat")
    done
    find "${find_args[@]}" 2>/dev/null || true
}

# ─── Python: state management + markdown generation ──────────────────────────────

PYTHON_BIN=""
find_python() {
    for cmd in python3 python; do
        if command -v "$cmd" >/dev/null 2>&1; then
            PYTHON_BIN="$cmd"
            return
        fi
    done
    echo "ERROR: neither python3 nor python found on PATH" >&2
    exit 1
}

run_python() {
    find_python
    # All heavy lifting in Python: file scanning, scoring, selection, state update, markdown gen.
    # Pass env vars through instead of args to keep quoting simple.
    "$PYTHON_BIN" - "$@" << 'PYEOF'
import json, os, sys, re, random, subprocess
from datetime import date, datetime, timezone, timedelta
from pathlib import Path
from collections import defaultdict

PROJECT_ROOT = Path(os.environ["PROJECT_ROOT"])
STATE_FILE = Path(os.environ["STATE_FILE"])
TRACKING_DOC = Path(os.environ["TRACKING_DOC"])
MUTANTS_OUT_DIR = Path(os.environ["MUTANTS_OUT_DIR"])
SAMPLE_SIZE = int(os.environ["SAMPLE_SIZE"])
TIMEOUT = int(os.environ["TIMEOUT"])

# Windows: force UTF-8 for all I/O (default is cp1252 which loses Unicode)
FILE_ENCODING = "utf-8"
SUBPROCESS_ENCODING = "utf-8"

EXCLUDE_PATTERNS = [
    '*/demo/*', '*/card_game_bin/*', '*/wgpu_renderer/*',
    '*/art/generated/*', '*/card_back.rs', '*/repository.rs',
    '*/hydrate.rs', '*/tests/*',
]

MODE = sys.argv[1] if len(sys.argv) > 1 else "run"


# ═══════════════════════════════════════════════════════════════════════════════════
# State file operations
# ═══════════════════════════════════════════════════════════════════════════════════

def load_state():
    if STATE_FILE.exists():
        with open(STATE_FILE, encoding=FILE_ENCODING) as f:
            return json.load(f)
    return {"files": {}, "cumulative": {"total_mutants": 0, "caught": 0, "missed": 0,
              "timeout": 0, "unviable": 0, "zero_mutant": 0, "error": 0,
              "runs": 0, "files_tested": 0},
            "runs": []}

def save_state(state):
    STATE_FILE.parent.mkdir(parents=True, exist_ok=True)
    with open(STATE_FILE, 'w', encoding=FILE_ENCODING) as f:
        json.dump(state, f, indent=2, sort_keys=True, ensure_ascii=False)


# ═══════════════════════════════════════════════════════════════════════════════════
# File discovery & scoring
# ═══════════════════════════════════════════════════════════════════════════════════

def scan_eligible_files():
    """Find all .rs files not matching exclusion patterns. Returns list of relative paths."""
    import fnmatch
    files = []
    for rs_file in PROJECT_ROOT.glob("crates/**/*.rs"):
        if not rs_file.is_file():
            continue
        rel = str(rs_file.relative_to(PROJECT_ROOT)).replace("\\", "/")
        excluded = False
        for pat in EXCLUDE_PATTERNS:
            # fnmatch uses Unix-style matching
            if fnmatch.fnmatch(rel, pat) or fnmatch.fnmatch(f"/{rel}", pat):
                excluded = True
                break
        if not excluded:
            files.append(rel)
    return sorted(files)


def count_lines(rel_path):
    """Count non-empty lines in a source file."""
    try:
        p = PROJECT_ROOT / rel_path
        with open(p, encoding=FILE_ENCODING, errors='ignore') as f:
            return sum(1 for line in f if line.strip())
    except Exception:
        return 0


# ─── Batch git churn (single subprocess, not one per file) ──────────────────────

def build_churn_map(days=90):
    """Return dict mapping file path -> commit count in last N days.
    Uses a single git subprocess call for all files."""
    churn = defaultdict(int)
    try:
        result = subprocess.run(
            ["git", "log", "--since", f"{days}.days.ago", "--name-only", "--oneline", "--", "crates/"],
            capture_output=True, text=True,
            cwd=str(PROJECT_ROOT), timeout=30,
            encoding=SUBPROCESS_ENCODING, errors='replace'
        )
        for line in result.stdout.split('\n'):
            line = line.strip()
            if line and not line.startswith(' ') and ' ' not in line:
                # It's a filename (oneline format: "hash msg" then filename on its own line)
                # Actually --name-only puts filenames after each commit line
                pass
        # Re-parse: --name-only output is: commit_hash commit_msg\n\nfile1\nfile2\n\ncommit_hash...
        # Simpler approach: use --format='' to only output filenames
    except Exception:
        pass

    # Use a simpler format: just list all changed files, one per line
    try:
        result = subprocess.run(
            ["git", "log", f"--since={days}.days.ago", "--format=", "--name-only", "--", "crates/"],
            capture_output=True, text=True,
            cwd=str(PROJECT_ROOT), timeout=30,
            encoding=SUBPROCESS_ENCODING, errors='replace'
        )
        for line in result.stdout.split('\n'):
            line = line.strip()
            if line:
                # Convert backslash paths to forward slash
                line = line.replace('\\', '/')
                churn[line] += 1
    except Exception:
        pass

    return dict(churn)


def compute_score(file_info, now_str):
    """Compute selection score for a file. Higher = more likely to be picked.

    staleness: days since last tested (0-90, capped). Untested = 90.
    size: log2(lines). Bigger files have more mutation surface.
    churn: commits in last 90d. More churn = more likely to have fresh bugs.
    """
    last_tested = file_info.get("last_tested")
    if last_tested:
        try:
            # Parse as dates (not datetimes) to avoid timezone-naive vs aware issues
            last_date = date.fromisoformat(last_tested[:10])  # "2026-07-10" or "2026-07-10T..."
            now_date = date.fromisoformat(now_str[:10])
            staleness = min((now_date - last_date).days, 90)
        except Exception:
            staleness = 90
    else:
        staleness = 90  # never tested = maximum staleness

    lines = file_info.get("lines", 0)
    if lines > 0:
        import math
        size_score = math.log2(lines)  # log2(50)=5.6, log2(500)=9.0, log2(2000)=11.0
    else:
        size_score = 1.0

    churn = file_info.get("recent_commits", 0)

    # Normalize to roughly 0-1 range for combining
    staleness_norm = staleness / 90.0              # 0.0 - 1.0
    size_norm = min(size_score / 11.0, 1.0)        # 0.0 - 1.0 (11 ~= log2(2000))
    churn_norm = min(churn / 10.0, 1.0)            # 0.0 - 1.0

    # Weights: staleness dominates, then size, then churn
    score = staleness_norm * 0.50 + size_norm * 0.30 + churn_norm * 0.20

    return {
        "score": round(score, 4),
        "staleness_days": staleness,
        "lines": lines,
        "churn_90d": churn,
    }


# ═══════════════════════════════════════════════════════════════════════════════════
# File selection
# ═══════════════════════════════════════════════════════════════════════════════════

def select_files(state, now_str, count):
    """Weighted random selection of N files. Higher score = higher probability."""
    eligible = scan_eligible_files()
    if not eligible:
        return []

    # Refresh inventory: add new files, update line counts and churn
    for rel in eligible:
        if rel not in state["files"]:
            state["files"][rel] = {
                "last_tested": None,
                "last_result": None,
                "lines": 0,
                "error_count": 0,
                "recent_commits": 0,
            }
        # Always refresh line count and churn (cheap, changes over time)
        state["files"][rel]["lines"] = count_lines(rel)

    # Batch churn lookup (single git call for all files)
    churn_map = build_churn_map()
    for rel in eligible:
        state["files"][rel]["recent_commits"] = churn_map.get(rel, 0)

    # Remove files that no longer exist
    for stale in list(state["files"]):
        if stale not in eligible:
            del state["files"][stale]

    # Compute scores
    scored = []
    for rel in eligible:
        info = state["files"][rel]
        scoring = compute_score(info, now_str)
        scored.append((rel, scoring["score"], scoring))

    # Sort by score descending (for logging), but select via weighted random
    scored.sort(key=lambda x: x[1], reverse=True)

    # Weighted random: use scores as weights
    total_weight = sum(s[1] for s in scored)
    if total_weight <= 0:
        # All zero scores — pure random
        selected = random.sample(scored, min(count, len(scored)))
        return [s[0] for s in selected]

    # Pick N distinct files without replacement
    selected = []
    remaining = list(scored)
    for _ in range(min(count, len(remaining))):
        weights = [s[1] for s in remaining]
        total_w = sum(weights)
        if total_w <= 0:
            pick = random.choice(remaining)
        else:
            r = random.uniform(0, total_w)
            cum = 0.0
            pick = remaining[-1]
            for item in remaining:
                cum += item[1]
                if r <= cum:
                    pick = item
                    break
        selected.append(pick[0])
        remaining = [s for s in remaining if s[0] != pick[0]]

    return selected


# ═══════════════════════════════════════════════════════════════════════════════════
# Mutation execution
# ═══════════════════════════════════════════════════════════════════════════════════

def run_mutation(file_path):
    """Run cargo-mutants on a single file. Returns result dict."""
    log = []

    # Clean previous outcomes
    if MUTANTS_OUT_DIR.exists():
        import shutil
        shutil.rmtree(MUTANTS_OUT_DIR)

    # Per-mutant timeout is TIMEOUT seconds. Whole-file timeout accounts for
    # initial build (~120s) + many mutants each taking up to TIMEOUT seconds.
    # 30 mutants * 45s = 1350s, so: TIMEOUT * 60 + 300 (max ~50 min per file).
    wrapper_timeout = max(int(TIMEOUT) * 60 + 300, 1200)

    try:
        result = subprocess.run(
            ["cargo", "mutants", "--file", file_path, "--in-place",
             "--timeout", str(TIMEOUT), "--cap-lints=true", "--no-shuffle", "-vV"],
            capture_output=True, text=True,
            cwd=str(PROJECT_ROOT),
            timeout=wrapper_timeout,
            encoding=SUBPROCESS_ENCODING, errors='replace'
        )
        stdout = result.stdout
        stderr = result.stderr
        exit_code = result.returncode
    except subprocess.TimeoutExpired:
        return {
            "total": 0, "caught": 0, "missed": 0, "timeout": 0, "unviable": 0,
            "status": "error",
            "error": f"cargo-mutants timed out after {wrapper_timeout}s (per-mutant timeout was {TIMEOUT}s)",
        }
    except Exception as e:
        return {
            "total": 0, "caught": 0, "missed": 0, "timeout": 0, "unviable": 0,
            "status": "error",
            "error": f"subprocess failed: {e}"[:500],
        }

    # Detect "0 mutants" case
    if "Found 0 mutants to test" in stdout or "Found 0 mutants to test" in stderr:
        return {
            "total": 0, "caught": 0, "missed": 0, "timeout": 0, "unviable": 0,
            "status": "no_mutants",
            "error": None,
        }

    # Parse outcomes.json
    outcomes_json = MUTANTS_OUT_DIR / "outcomes.json"
    if outcomes_json.exists():
        try:
            with open(outcomes_json, encoding=FILE_ENCODING) as f:
                data = json.load(f)
            outcomes = data.get("outcomes", [])
            caught = sum(1 for o in outcomes if o.get("summary") == "CaughtMutant")
            missed = sum(1 for o in outcomes if o.get("summary") == "MissedMutant")
            timeout = sum(1 for o in outcomes if o.get("summary") == "Timeout")
            unviable = sum(1 for o in outcomes if o.get("summary") == "Unviable")
            total = caught + missed + timeout + unviable
            return {
                "total": total, "caught": caught, "missed": missed,
                "timeout": timeout, "unviable": unviable,
                "status": "ok",
                "error": None,
            }
        except Exception as e:
            return {
                "total": 0, "caught": 0, "missed": 0, "timeout": 0, "unviable": 0,
                "status": "parse_error",
                "error": f"Failed to parse outcomes.json: {e}",
            }

    # No outcomes.json and not 0-mutants → likely a build error
    error_msg = ""
    # Extract last meaningful error lines from stderr
    err_lines = [l for l in (stderr + stdout).split('\n') if l.strip()]
    if err_lines:
        error_msg = '; '.join(err_lines[-5:])  # last 5 lines
    else:
        error_msg = f"exit_code={exit_code}, no outcomes.json, no stdout/stderr"

    return {
        "total": 0, "caught": 0, "missed": 0, "timeout": 0, "unviable": 0,
        "status": "error",
        "error": error_msg[:500],
    }


# ═══════════════════════════════════════════════════════════════════════════════════
# Markdown generation
# ═══════════════════════════════════════════════════════════════════════════════════

def generate_markdown(state):
    c = state["cumulative"]
    total_tested = c["caught"] + c["missed"]
    catch_rate = f"{c['caught'] / total_tested:.1%}" if total_tested > 0 else "N/A"

    lines = []
    lines.append("# Micro-Mutation Tracking")
    lines.append("")
    lines.append("Stochastic mutation testing — one random source file per daily CI run.")
    lines.append("Selection weighted by **staleness** (50%), **file size** (30%), and **git churn** (20%).")
    lines.append("Over weeks, covers the codebase without combinatorial explosion.")
    lines.append("")
    lines.append(f"**Cumulative (all runs)**: {c['total_mutants']} mutants | "
                 f"{c['caught']} caught | {c['missed']} missed | "
                 f"{c['timeout']} timeout | {c['unviable']} unviable | "
                 f"{c['zero_mutant']} zero-mutant | {c['error']} errors | "
                 f"**catch rate: {catch_rate}** | {c['runs']} runs | "
                 f"{c['files_tested']} files tested")
    lines.append("")

    # Last run info
    if state["runs"]:
        last = state["runs"][-1]
        lines.append(f"**Last run**: {last['date']} (`{last['commit']}`)")
    lines.append("")
    lines.append("---")
    lines.append("")

    # ── Full File Inventory ──
    lines.append("## File Inventory")
    lines.append("")
    lines.append(f"All {len(state['files'])} eligible source files. "
                 "Sorted by selection priority (staleness × size × churn).")
    lines.append("")

    # Compute scores for all files to sort by priority
    now_str = datetime.now(timezone.utc).isoformat()
    file_scores = []
    for rel, info in state["files"].items():
        scoring = compute_score(info, now_str)
        file_scores.append((rel, info, scoring))

    file_scores.sort(key=lambda x: x[2]["score"], reverse=True)

    # Table header
    lines.append("| Priority | File | Lines | Churn | Stale | Last Tested | Result | Status |")
    lines.append("|----------|------|-------|-------|-------|-------------|--------|--------|")

    for i, (rel, info, scoring) in enumerate(file_scores, 1):
        # Status badge
        last_result = info.get("last_result") or {}
        status_str = last_result.get("status", "pending") if last_result else "pending"

        if status_str == "ok":
            t = last_result.get("total", 0)
            c_ = last_result.get("caught", 0)
            if t > 0:
                rate = f"{c_ / t:.0%}" if t > 0 else "—"
                result_str = f"{c_}/{t} ({rate})"
            else:
                result_str = "0 mutants"
            status_icon = "✅" if t == 0 or (c_ > 0 and last_result.get("missed", 0) == 0) else "⚠️"
        elif status_str == "no_mutants":
            result_str = "0 mutants"
            status_icon = "➖"
        elif status_str == "error":
            result_str = "error"
            status_icon = "❌"
        else:  # pending
            result_str = "—"
            status_icon = "⬜"

        last_tested = info.get("last_tested", "never") or "never"
        if last_tested != "never":
            try:
                dt = datetime.fromisoformat(last_tested)
                last_tested = dt.strftime("%Y-%m-%d")
            except Exception:
                pass

        stale_days = scoring["staleness_days"]
        stale_str = f"{stale_days}d" if stale_days > 0 else "today"

        score_pct = f"{scoring['score']*100:.0f}%"

        lines.append(
            f"| {score_pct} | `{rel}` | {scoring['lines']} | {scoring['churn_90d']} | "
            f"{stale_str} | {last_tested} | {result_str} | {status_icon} |"
        )

    lines.append("")

    # ── Recent Runs ──
    lines.append("---")
    lines.append("")
    lines.append("## Recent Runs")
    lines.append("")
    if state["runs"]:
        lines.append("| Date | Commit | File | Total | Caught | Missed | Timeout | Unviable | Status |")
        lines.append("|------|--------|------|-------|--------|--------|---------|----------|--------|")
        for run in reversed(state["runs"][-30:]):  # last 30 runs
            result = run.get("result", {})
            status = result.get("status", "?")
            if status == "no_mutants":
                status_str = "0 mutants"
            elif status == "error":
                status_str = f"❌ {result.get('error', 'unknown')[:60]}"
            elif status == "ok":
                status_str = "✅"
            else:
                status_str = status
            lines.append(
                f"| {run['date']} | `{run['commit']}` | `{run['file']}` | "
                f"{result.get('total', 0)} | {result.get('caught', 0)} | "
                f"{result.get('missed', 0)} | {result.get('timeout', 0)} | "
                f"{result.get('unviable', 0)} | {status_str} |"
            )
    else:
        lines.append("_No runs yet._")
    lines.append("")

    # ── Exclusions reference ──
    lines.append("---")
    lines.append("")
    lines.append("## Excluded Paths")
    lines.append("")
    for pat in EXCLUDE_PATTERNS:
        lines.append(f"- `{pat}`")
    lines.append("")
    lines.append("<!-- Generated by scripts/micro-mutations.sh -->")
    lines.append("")

    return "\n".join(lines)


# ═══════════════════════════════════════════════════════════════════════════════════
# State initialization
# ═══════════════════════════════════════════════════════════════════════════════════

def init_state():
    """Seed the state file from scratch by scanning all eligible files.
    Resets cumulative stats and run log — only preserves file inventory."""
    log("Initializing state file from scratch...")
    # Start fresh: only carry forward file test history, reset cumulative + runs
    old_state = load_state()

    eligible = scan_eligible_files()
    log(f"  Found {len(eligible)} eligible files")

    state = {
        "files": {},
        "cumulative": {
            "total_mutants": 0, "caught": 0, "missed": 0,
            "timeout": 0, "unviable": 0, "zero_mutant": 0, "error": 0,
            "runs": 0, "files_tested": 0,
        },
        "runs": [],
    }

    # Preserve per-file test history from old state
    for rel in eligible:
        if rel in old_state.get("files", {}):
            state["files"][rel] = dict(old_state["files"][rel])
        else:
            state["files"][rel] = {
                "last_tested": None,
                "last_result": None,
                "lines": 0,
                "error_count": 0,
                "recent_commits": 0,
            }

    now_str = datetime.now(timezone.utc).isoformat()

    # Refresh line counts and churn for all files
    for rel in eligible:
        state["files"][rel]["lines"] = count_lines(rel)

    # Batch churn lookup
    churn_map = build_churn_map()
    for rel in eligible:
        state["files"][rel]["recent_commits"] = churn_map.get(rel, 0)

    # Remove stale entries
    for stale in list(state["files"]):
        if stale not in eligible:
            del state["files"][stale]

    save_state(state)
    md = generate_markdown(state)
    with open(TRACKING_DOC, 'w', encoding=FILE_ENCODING) as f:
        f.write(md)

    log(f"State initialized: {len(state['files'])} files")
    log(f"Markdown written: {TRACKING_DOC}")
    return state


# ═══════════════════════════════════════════════════════════════════════════════════
# Main run
# ═══════════════════════════════════════════════════════════════════════════════════

def main_run():
    try:
        _main_run_impl()
    except Exception as e:
        warn(f"Unexpected crash: {e}")
        import traceback
        traceback.print_exc(file=sys.stderr)
        # Try to save whatever state we had before crash
        try:
            save_state(_current_state)
            md = generate_markdown(_current_state)
            with open(TRACKING_DOC, 'w', encoding=FILE_ENCODING) as f:
                f.write(md)
        except Exception:
            pass
        sys.exit(1)

# Module-level mutable for crash recovery
_current_state = None

def _main_run_impl():
    global _current_state
    log("Micro-mutation run starting")
    log(f"  sample_size={SAMPLE_SIZE}, timeout={TIMEOUT}s")

    state = load_state()
    _current_state = state
    now_str = datetime.now(timezone.utc).isoformat()
    now_date = datetime.now(timezone.utc).strftime("%Y-%m-%d")

    # Get commit SHA
    try:
        commit_sha = subprocess.run(
            ["git", "rev-parse", "--short", "HEAD"],
            capture_output=True, text=True, cwd=str(PROJECT_ROOT), timeout=5,
            encoding=SUBPROCESS_ENCODING, errors='replace'
        ).stdout.strip()
    except Exception:
        commit_sha = "unknown"

    # Select files
    selected = select_files(state, now_str, SAMPLE_SIZE)
    if not selected:
        warn("No eligible files found")
        return

    log(f"Selected {len(selected)} file(s) for mutation:")
    for f in selected:
        info = state["files"].get(f, {})
        scoring = compute_score(info, now_str)
        log(f"  {f}  (score={scoring['score']:.3f}, stale={scoring['staleness_days']}d, "
            f"lines={scoring['lines']}, churn={scoring['churn_90d']})")

    # Run mutations
    for file_path in selected:
        log("")
        log(f"=== Mutating: {file_path} ===")

        result = run_mutation(file_path)

        if result["status"] == "no_mutants":
            log(f"  No mutants found in this file (0 mutants generated)")
        elif result["status"] == "error":
            warn(f"  Mutation failed: {result['error']}")
            log(f"  Mutants: 0 total (error)")
        else:
            total = result["total"]
            caught = result["caught"]
            rate = f"{caught / total:.1%}" if total > 0 else "N/A"
            log(f"  Mutants: {total} total | {caught} caught | "
                f"{result['missed']} missed | {result['timeout']} timeout | "
                f"{result['unviable']} unviable")
            log(f"  Catch rate: {rate}")

        # Update per-file state
        info = state["files"].setdefault(file_path, {
            "lines": count_lines(file_path),
            "error_count": 0,
            "recent_commits": 0,
        })
        info["last_tested"] = now_date
        info["last_result"] = result

        if result["status"] == "error":
            info["error_count"] = info.get("error_count", 0) + 1

        # Update cumulative
        c = state["cumulative"]
        c["total_mutants"] += result["total"]
        c["caught"] += result["caught"]
        c["missed"] += result["missed"]
        c["timeout"] += result["timeout"]
        c["unviable"] += result["unviable"]
        if result["status"] == "no_mutants":
            c["zero_mutant"] += 1
        elif result["status"] == "error":
            c["error"] += 1
        c["runs"] += 1
        if info["last_tested"] == now_date or info.get("last_result", {}).get("status") == "pending":
            # Count unique files tested (once per file, not per run)
            all_tested = sum(1 for v in state["files"].values() if v.get("last_tested"))
            c["files_tested"] = all_tested

        # Append to run log
        state["runs"].append({
            "date": now_date,
            "commit": commit_sha,
            "file": file_path,
            "result": result,
        })

    # Save state and regenerate markdown
    save_state(state)
    log("")
    log("Regenerating tracking doc...")
    md = generate_markdown(state)
    with open(TRACKING_DOC, 'w', encoding=FILE_ENCODING) as f:
        f.write(md)

    log(f"Updated: {TRACKING_DOC}")
    log("")
    log("Micro-mutation run complete.")
    c = state["cumulative"]
    total_tested = c["caught"] + c["missed"]
    catch_rate = f"{c['caught'] / total_tested:.1%}" if total_tested > 0 else "N/A"
    log(f"  Cumulative: {c['total_mutants']} mutants | {c['caught']} caught ({catch_rate}) | "
        f"{c['missed']} missed | {c['zero_mutant']} zero-mutant | {c['error']} errors | "
        f"{c['runs']} runs | {c['files_tested']} files tested")


# ═══════════════════════════════════════════════════════════════════════════════════
# Dispatch
# ═══════════════════════════════════════════════════════════════════════════════════

def log(msg):
    print(f":: {msg}", file=sys.stderr)

def warn(msg):
    print(f"::warning::{msg}", file=sys.stderr)

if MODE == "--init-state":
    init_state()
elif MODE == "run":
    main_run()
else:
    print(f"Unknown mode: {MODE}", file=sys.stderr)
    sys.exit(1)
PYEOF
}


# ─── Main ────────────────────────────────────────────────────────────────────────

cd "$PROJECT_ROOT"

export PROJECT_ROOT STATE_FILE TRACKING_DOC MUTANTS_OUT_DIR SAMPLE_SIZE TIMEOUT

case "${1:-run}" in
    --init-state)
        log "Initializing state file and regenerating markdown..."
        run_python --init-state
        ;;
    --print-exclusions)
        echo "Excluded path patterns:"
        for pat in "${EXCLUDE_PATTERNS[@]}"; do
            echo "  $pat"
        done
        ;;
    *)
        log "Micro-mutation run starting (sample_size=$SAMPLE_SIZE, timeout=${TIMEOUT}s)"
        run_python run
        ;;
esac
