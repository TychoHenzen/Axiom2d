#!/usr/bin/env bash
# Micro-mutation testing — randomized subset of mutations per CI run.
# Trades coverage completeness for speed: over weeks, covers the codebase stochastically.
#
# Usage:
#   ./scripts/micro-mutations.sh            # Run with defaults (1 random file)
#   MICRO_SAMPLE_SIZE=3 ./scripts/micro-mutations.sh  # Run on 3 random files
#   ./scripts/micro-mutations.sh --print-exclusions   # Show excluded paths
#
# CI: called from quality.yml "micro-mutations" job.
# Appends results to docs/MICRO_MUTATIONS.md, then commits the update.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TRACKING_DOC="$PROJECT_ROOT/docs/MICRO_MUTATIONS.md"
MUTANTS_OUT_DIR="$PROJECT_ROOT/mutants.out"
SAMPLE_SIZE="${MICRO_SAMPLE_SIZE:-1}"
TIMEOUT="${MICRO_TIMEOUT:-45}"

# ─── Exclusions (mirrors .cargo/mutants.toml exclude_globs) ─────────────────────
# find -path patterns for files/dirs excluded from mutation testing.
# Keep in sync with .cargo/mutants.toml [exclude_globs].
EXCLUDE_PATTERNS=(
    '*/demo/*'
    '*/card_game_bin/*'
    '*/wgpu_renderer/*'
    '*/art/generated/*'
    '*/card_back.rs'
    '*/repository.rs'
    '*/hydrate.rs'
    '*/tests/*'          # Never mutate test files
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
    # Build find command with multiple exclusion patterns
    local find_args=("crates" "-name" "*.rs" "-type" "f")
    for pat in "${EXCLUDE_PATTERNS[@]}"; do
        find_args+=("-not" "-path" "$pat")
    done
    find "${find_args[@]}" 2>/dev/null || true
}

json_val() {
    # Extract a JSON key value. Input on stdin, key as $1.
    # Only works for simple string/number values at top level.
    python3 -c "
import json, sys
data = json.load(sys.stdin)
print(data.get('$1', 0))
" 2>/dev/null || echo "0"
}

# ─── Main ────────────────────────────────────────────────────────────────────────

cd "$PROJECT_ROOT"

log "Micro-mutation run starting (sample_size=$SAMPLE_SIZE, timeout=${TIMEOUT}s)"

# 1. Find eligible files
log "Finding eligible source files..."
ELIGIBLE_COUNT=$(eligible_files | wc -l)
log "  $ELIGIBLE_COUNT eligible files found"

if [ "$ELIGIBLE_COUNT" -eq 0 ]; then
    warn "No eligible files found — nothing to mutate"
    exit 0
fi

# 2. Randomly select N files
SELECTED=$(eligible_files | shuf -n "$SAMPLE_SIZE")
log "Selected for mutation:"
echo "$SELECTED" | while read -r f; do log "  $f"; done

# 3. Run cargo-mutants on each selected file
TOTAL_MUTANTS=0
TOTAL_CAUGHT=0
TOTAL_MISSED=0
TOTAL_TIMEOUT=0
TOTAL_UNVIABLE=0
MUTATED_FILES=()

while IFS= read -r file; do
    [ -z "$file" ] && continue
    log ""
    log "=== Mutating: $file ==="

    # Clean previous outcomes
    rm -rf "$MUTANTS_OUT_DIR"

    # Run cargo-mutants scoped to this file
    # --in-place avoids copying the whole target/ dir
    # Use cargo.exe? No — CI runs on ubuntu-latest, use cargo.
    set +e
    cargo mutants \
        --file "$file" \
        --in-place \
        --timeout "$TIMEOUT" \
        --cap-lints=true \
        --no-shuffle \
        -vV \
        > /dev/null 2>&1
    MUTANT_EXIT=$?
    set -e

    # Parse outcomes
    OUTCOMES="$MUTANTS_OUT_DIR/outcomes.json"
    if [ -f "$OUTCOMES" ]; then
        # Python for reliable JSON parsing
        read -r caught missed timeout unviable <<< "$(python3 -c "
import json
with open('$OUTCOMES') as f:
    data = json.load(f)
outcomes = data.get('outcomes', [])
caught = sum(1 for o in outcomes if o.get('summary') == 'CaughtMutant')
missed = sum(1 for o in outcomes if o.get('summary') == 'MissedMutant')
timeout = sum(1 for o in outcomes if o.get('summary') == 'Timeout')
unviable = sum(1 for o in outcomes if o.get('summary') == 'Unviable')
print(caught, missed, timeout, unviable)
" 2>/dev/null || echo "0 0 0 0")"

        TOTAL=$((caught + missed + timeout + unviable))
        TOTAL_MUTANTS=$((TOTAL_MUTANTS + TOTAL))
        TOTAL_CAUGHT=$((TOTAL_CAUGHT + caught))
        TOTAL_MISSED=$((TOTAL_MISSED + missed))
        TOTAL_TIMEOUT=$((TOTAL_TIMEOUT + timeout))
        TOTAL_UNVIABLE=$((TOTAL_UNVIABLE + unviable))

        if [ "$TOTAL" -gt 0 ]; then
            CATCH_RATE=$(python3 -c "print(f'{$caught / $TOTAL:.1%}')" 2>/dev/null || echo "?")
        else
            CATCH_RATE="N/A"
        fi

        log "  Mutants: $TOTAL total | $caught caught | $missed missed | $timeout timeout | $unviable unviable"
        log "  Catch rate: $CATCH_RATE"

        MUTATED_FILES+=("$file:$TOTAL:$caught:$missed:$CATCH_RATE")
    else
        log "  No outcomes file — cargo-mutants may have failed (exit=$MUTANT_EXIT)"
        MUTATED_FILES+=("$file:0:0:0:error")
    fi

done <<< "$SELECTED"

# 4. Append to tracking doc
log ""
log "=== Updating tracking doc ==="

COMMIT_SHA=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
COMMIT_DATE=$(date -u +"%Y-%m-%d %H:%M UTC")
TODAY=$(date -u +"%Y-%m-%d")

# Build the new table row
ROW="| $TODAY | \`$COMMIT_SHA\` | $TOTAL_MUTANTS | $TOTAL_CAUGHT | $TOTAL_MISSED | $TOTAL_TIMEOUT | $TOTAL_UNVIABLE |"

# Build file detail string
FILE_DETAIL=""
for entry in "${MUTATED_FILES[@]}"; do
    IFS=':' read -r fname ftotal fcaught fmissed fcrate <<< "$entry"
    FILE_DETAIL="$FILE_DETAIL\n<!-- detail: $fname → $fcaught/$ftotal caught ($fcrate) -->"
done

# Calculate cumulative stats from all existing rows + new row
# (Python: parse all table rows, add current, compute cumulative)
# NOTE: heredoc delimiter quoted ('PYEOF') to prevent bash from
# expanding backticks / regex escapes inside the Python code.
export TRACKING_DOC
export TOTAL_MUTANTS TOTAL_CAUGHT TOTAL_MISSED TOTAL_TIMEOUT TOTAL_UNVIABLE
export ROW FILE_DETAIL COMMIT_SHA TODAY
python3 << 'PYEOF'
import os
import re

doc = os.environ["TRACKING_DOC"]
row_total = int(os.environ["TOTAL_MUTANTS"])
row_caught = int(os.environ["TOTAL_CAUGHT"])
row_missed = int(os.environ["TOTAL_MISSED"])
row_timeout = int(os.environ["TOTAL_TIMEOUT"])
row_unviable = int(os.environ["TOTAL_UNVIABLE"])
new_row = os.environ["ROW"]
file_detail = os.environ["FILE_DETAIL"]
commit_sha = os.environ["COMMIT_SHA"]
run_date = os.environ["TODAY"]

# Read existing doc
if os.path.exists(doc):
    with open(doc) as f:
        content = f.read()
else:
    content = ""

# Parse existing rows to compute cumulative
cum_total = 0
cum_caught = 0
cum_missed = 0
cum_timeout = 0
cum_unviable = 0
run_count = 0

for line in content.split('\n'):
    m = re.match(r'\|\s*(\d{4}-\d{2}-\d{2})\s*\|\s*`[^`]+`\s*\|\s*(\d+)\s*\|\s*(\d+)\s*\|\s*(\d+)\s*\|\s*(\d+)\s*\|\s*(\d+)\s*\|', line)
    if m:
        run_count += 1
        cum_total += int(m.group(2))
        cum_caught += int(m.group(3))
        cum_missed += int(m.group(4))
        cum_timeout += int(m.group(5))
        cum_unviable += int(m.group(6))

cum_total += row_total
cum_caught += row_caught
cum_missed += row_missed
cum_timeout += row_timeout
cum_unviable += row_unviable
run_count += 1

cum_rate = f"{cum_caught / cum_total:.1%}" if cum_total > 0 else "N/A"

# Build the updated doc
if not content or '## Run Log' not in content:
    # Create from scratch
    content = f"""# Micro-Mutation Tracking

Stochastic mutation testing — one random source file per daily CI run.
Over weeks, covers the codebase without combinatorial explosion.

**Cumulative (all runs)**: {cum_total} mutants | {cum_caught} caught | {cum_missed} missed | {cum_timeout} timeout | {cum_unviable} unviable | **catch rate: {cum_rate}** | {run_count} runs

**How to read**: Each row = one CI run. A single random source file is selected
and all mutants generated for it are tested. Over time, this builds a picture
of mutation coverage across the workspace.

**Last run**: {run_date} (`{commit_sha}`)

---

## Run Log

| Date | Commit | Total | Caught | Missed | Timeout | Unviable |
|------|--------|-------|--------|--------|---------|----------|
{new_row}
{file_detail}
"""
else:
    # Append new row and update header stats
    # Update cumulative line
    old_cum = re.search(r'\*\*Cumulative \(all runs\)\*\*:.*', content)
    new_cum = f"**Cumulative (all runs)**: {cum_total} mutants | {cum_caught} caught | {cum_missed} missed | {cum_timeout} timeout | {cum_unviable} unviable | **catch rate: {cum_rate}** | {run_count} runs"
    if old_cum:
        content = content.replace(old_cum.group(0), new_cum)

    # Update last run line
    old_last = re.search(r'\*\*Last run\*\*:.*', content)
    new_last = f"**Last run**: {run_date} (`{commit_sha}`)"
    if old_last:
        content = content.replace(old_last.group(0), new_last)

    # Insert new row after the header separator
    sep = '|------|--------|-------|--------|--------|---------|----------|'
    sep_pos = content.find(sep)
    if sep_pos != -1:
        insert_pos = content.find('\n', sep_pos) + 1
        content = content[:insert_pos] + new_row + '\n' + file_detail + '\n' + content[insert_pos:]

# Write back
with open(doc, 'w') as f:
    f.write(content)

print(f"Updated {doc}")
print(f"  Cumulative: {cum_total} mutants, {cum_caught} caught ({cum_rate}), {cum_missed} missed")
PYEOF

log ""
log "Micro-mutation run complete."
log "  Total this run: $TOTAL_MUTANTS mutants across ${#MUTATED_FILES[@]} file(s)"
log "  Tracking doc updated: $TRACKING_DOC"
