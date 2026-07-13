#!/usr/bin/env bash
# Quality Gate — local ratchet check.
# Run before pushing to verify no quality regressions.
#
# Usage:
#   ./scripts/quality-gate-check.sh          # Full check (hard + soft gates)
#   ./scripts/quality-gate-check.sh --soft   # Soft ratchets only
#   ./scripts/quality-gate-check.sh --hard   # Hard gates only
#   ./scripts/quality-gate-check.sh --diff   # Show current vs baseline diff
#   ./scripts/quality-gate-check.sh --update        # Update baseline to current metrics
#   ./scripts/quality-gate-check.sh --install-hooks  # Install git hooks via core.hooksPath

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BASELINE="$PROJECT_ROOT/docs/QUALITY_BASELINE.ron"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

PASS=0
FAIL=0
WARN=0

# grep returns 1 on no-match. Count safely.
safe_count() {
    # Usage: safe_count "pattern" "glob" [exclude1] [exclude2] ...
    local pattern="$1" glob="$2"
    shift 2
    local result
    set +e
    result=$(grep -rn "$pattern" "$PROJECT_ROOT/crates/" --include="$glob" 2>/dev/null)
    for excl in "$@"; do
        if [ -n "$excl" ] && [ -n "$result" ]; then
            result=$(echo "$result" | grep -v "$excl" || true)
        fi
    done
    if [ -z "$result" ]; then
        echo "0"
    else
        echo "$result" | wc -l
    fi
    set -e
}

# Parse RON value: grep for key, extract number after colon (avoids capturing digits in key name)
ron_value() {
    set +e
    local val
    # sed: match the key, then capture the number (int or float) after the colon
    val=$(sed -n "s/.*\"$1\"[[:space:]]*:[[:space:]]*\([0-9.]\+\).*/\\1/p" "$BASELINE" 2>/dev/null | head -1)
    set -e
    echo "${val:-0}"
}

# ─── Hard Gates ───────────────────────────────────────────────────────────────

check_hard_gates() {
    echo ""
    echo -e "${CYAN}═══ Hard Gates (must be zero) ═══${NC}"
    echo ""

    # Clippy
    printf "  %-35s " "clippy warnings:"
    if cargo clippy --workspace --all-targets -- -D warnings > /dev/null 2>&1; then
        echo -e "${GREEN}PASS${NC} (0)"
        PASS=$((PASS + 1))
    else
        echo -e "${RED}FAIL${NC}"
        FAIL=$((FAIL + 1))
    fi

    # Doc warnings
    printf "  %-35s " "doc warnings:"
    local doc_out doc_warnings
    set +e
    doc_out=$(cargo doc --workspace --no-deps 2>&1)
    doc_warnings=$(echo "$doc_out" | grep -c "warning:" || true)
    set -e
    if [ "${doc_warnings:-0}" -eq 0 ]; then
        echo -e "${GREEN}PASS${NC} (0)"
        PASS=$((PASS + 1))
    else
        echo -e "${RED}FAIL${NC} ($doc_warnings warnings)"
        FAIL=$((FAIL + 1))
    fi

    # Dead code
    printf "  %-35s " "dead code:"
    if RUSTFLAGS="-D dead_code" cargo build --workspace --all-targets > /dev/null 2>&1; then
        echo -e "${GREEN}PASS${NC} (0)"
        PASS=$((PASS + 1))
    else
        echo -e "${RED}FAIL${NC}"
        FAIL=$((FAIL + 1))
    fi

    echo ""
    echo -e "${CYAN}Hard gates: ${GREEN}$PASS passed${NC}, ${RED}$FAIL failed${NC}${NC}"
}

# ─── Soft Ratchets ────────────────────────────────────────────────────────────

check_soft_ratchets() {
    echo ""
    echo -e "${CYAN}═══ Soft Ratchets (cannot regress) ═══${NC}"
    echo ""

    # Extract baselines
    local base_test base_smell base_unsafe base_unwrap base_cyclo base_cov base_clones
    base_test=$(ron_value "test_count_total")
    base_smell=$(ron_value "smell_markers_total")
    base_unsafe=$(ron_value "unsafe_blocks_total")
    base_unwrap=$(ron_value "unwrap_in_prod_total")
    base_cyclo=$(ron_value "cyclomatic_over_10")
    base_cov=$(ron_value "line_coverage_pct")
    base_clones=$(ron_value "jscpd_clone_count")

    # Test count
    local cur_test
    cur_test=$(safe_count '#\[test\]\|#\[tokio::test\]' "*.rs")
    check_dimension "test count" "$cur_test" "$base_test" "higher"

    # Smell markers
    local cur_smell
    cur_smell=$(safe_count 'TODO\|FIXME\|HACK' "*.rs" "tests/")
    check_dimension "smell markers (TODO/FIXME/HACK)" "$cur_smell" "$base_smell" "lower"

    # Unsafe blocks
    local cur_unsafe
    cur_unsafe=$(safe_count '\bunsafe\b' "*.rs" "tests/")
    check_dimension "unsafe blocks" "$cur_unsafe" "$base_unsafe" "lower"

    # Unwrap in production
    local cur_unwrap
    cur_unwrap=$(safe_count '\.unwrap()' "*.rs" "tests/" "benches/" "particle_poc/")
    check_dimension "unwrap() in prod" "$cur_unwrap" "$base_unwrap" "lower"

    # Cyclomatic complexity (optional — needs arborist-cli)
    local cur_cyclo
    if command -v arborist &>/dev/null; then
        set +e
        cur_cyclo=$(arborist crates/ --languages rust --format json 2>/dev/null | python3 -c "import json,sys;data=json.load(sys.stdin);print(sum(1 for f in data for func in f.get('functions',[]) if func.get('cyclomatic',0)>10))" 2>/dev/null)
        set -e
        cur_cyclo="${cur_cyclo:-0}"
        check_dimension "cyclomatic CC > 10" "$cur_cyclo" "$base_cyclo" "lower"
    else
        printf "  %-35s ${YELLOW}SKIP${NC} (arborist-cli not installed)\n" "cyclomatic CC > 10:"
    fi

    # Code coverage (optional — needs cargo-llvm-cov)
    local cur_cov
    if command -v cargo-llvm-cov &>/dev/null && [ "${base_cov:-0}" != "0" ]; then
        set +e
        cur_cov=$(cargo llvm-cov --workspace --summary-only 2>/dev/null | grep "^TOTAL" | awk '{print $4}' | tr -d '%')
        set -e
        if [ -n "$cur_cov" ] && [ "$cur_cov" != "0" ]; then
            check_dimension_float "line coverage %" "$cur_cov" "$base_cov" "higher"
        else
            printf "  %-35s ${YELLOW}SKIP${NC} (coverage data unavailable)\n" "line coverage %:"
        fi
    else
        printf "  %-35s ${YELLOW}SKIP${NC} (cargo-llvm-cov not installed)\n" "line coverage %:"
    fi

    # Code duplication (optional — needs npx/jscpd)
    local cur_clones
    if command -v npx &>/dev/null && [ "${base_clones:-0}" != "0" ]; then
        set +e
        cur_clones=$(npx jscpd crates/ --pattern "**/*.rs" --min-tokens 50 --min-lines 5 --mode strict 2>&1 | grep -c "Clone found" || echo "0")
        set -e
        cur_clones="${cur_clones:-0}"
        check_dimension "jscpd clone count" "$cur_clones" "$base_clones" "lower"
    else
        printf "  %-35s ${YELLOW}SKIP${NC} (npx/jscpd not available)\n" "jscpd clone count:"
    fi

    echo ""
    echo -e "${CYAN}Soft ratchets: ${GREEN}$PASS passed${NC}, ${RED}$FAIL failed${NC}, ${YELLOW}$WARN improved${NC}"
}

check_dimension() {
    local name="$1" current="$2" baseline="$3" direction="$4"

    printf "  %-35s " "$name:"
    if [ "$direction" = "lower" ]; then
        if [ "$current" -gt "$baseline" ]; then
            echo -e "${RED}REGRESSED${NC}  ($current > baseline $baseline)"
            FAIL=$((FAIL + 1))
        elif [ "$current" -lt "$baseline" ]; then
            echo -e "${YELLOW}IMPROVED${NC}  ($current < baseline $baseline — ratchet me!)"
            WARN=$((WARN + 1))
        else
            echo -e "${GREEN}OK${NC}        ($current = baseline)"
            PASS=$((PASS + 1))
        fi
    else
        # higher is better (test count)
        if [ "$current" -lt "$baseline" ]; then
            echo -e "${RED}REGRESSED${NC}  ($current < baseline $baseline)"
            FAIL=$((FAIL + 1))
        elif [ "$current" -gt "$baseline" ]; then
            echo -e "${YELLOW}IMPROVED${NC}  ($current > baseline $baseline — ratchet me!)"
            WARN=$((WARN + 1))
        else
            echo -e "${GREEN}OK${NC}        ($current = baseline)"
            PASS=$((PASS + 1))
        fi
    fi
}

check_dimension_float() {
    local name="$1" current="$2" baseline="$3" direction="$4"

    printf "  %-35s " "$name:"
    if [ "$direction" = "higher" ]; then
        result=$(echo "$current < $baseline" | bc -l 2>/dev/null)
        if [ "$result" = "1" ]; then
            echo -e "${RED}REGRESSED${NC}  ($current < baseline $baseline)"
            FAIL=$((FAIL + 1))
        else
            result=$(echo "$current > $baseline" | bc -l 2>/dev/null)
            if [ "$result" = "1" ]; then
                echo -e "${YELLOW}IMPROVED${NC}  ($current > baseline $baseline — ratchet me!)"
                WARN=$((WARN + 1))
            else
                echo -e "${GREEN}OK${NC}        ($current = baseline)"
                PASS=$((PASS + 1))
            fi
        fi
    else
        result=$(echo "$current > $baseline" | bc -l 2>/dev/null)
        if [ "$result" = "1" ]; then
            echo -e "${RED}REGRESSED${NC}  ($current > baseline $baseline)"
            FAIL=$((FAIL + 1))
        else
            result=$(echo "$current < $baseline" | bc -l 2>/dev/null)
            if [ "$result" = "1" ]; then
                echo -e "${YELLOW}IMPROVED${NC}  ($current < baseline $baseline — ratchet me!)"
                WARN=$((WARN + 1))
            else
                echo -e "${GREEN}OK${NC}        ($current = baseline)"
                PASS=$((PASS + 1))
            fi
        fi
    fi
}

# ─── Diff Mode ────────────────────────────────────────────────────────────────

show_diff() {
    echo ""
    echo -e "${CYAN}═══ Current vs Baseline Diff ═══${NC}"
    echo ""

    local cur_test cur_smell cur_unsafe cur_unwrap cur_cyclo cur_cov cur_clones
    local base_test base_smell base_unsafe base_unwrap base_cyclo base_cov base_clones

    cur_test=$(safe_count '#\[test\]\|#\[tokio::test\]' "*.rs")
    cur_smell=$(safe_count 'TODO\|FIXME\|HACK' "*.rs" "tests/")
    cur_unsafe=$(safe_count '\bunsafe\b' "*.rs" "tests/")
    cur_unwrap=$(safe_count '\.unwrap()' "*.rs" "tests/" "benches/" "particle_poc/")

    base_test=$(ron_value "test_count_total")
    base_smell=$(ron_value "smell_markers_total")
    base_unsafe=$(ron_value "unsafe_blocks_total")
    base_unwrap=$(ron_value "unwrap_in_prod_total")
    base_cyclo=$(ron_value "cyclomatic_over_10")
    base_cov=$(ron_value "line_coverage_pct")
    base_clones=$(ron_value "jscpd_clone_count")

    # New dimensions (optional tools)
    if command -v arborist &>/dev/null; then
        cur_cyclo=$(arborist crates/ --languages rust --format json 2>/dev/null | python3 -c "import json,sys;data=json.load(sys.stdin);print(sum(1 for f in data for func in f.get('functions',[]) if func.get('cyclomatic',0)>10))" 2>/dev/null || echo "N/A")
    else
        cur_cyclo="N/A"
    fi
    if command -v cargo-llvm-cov &>/dev/null; then
        cur_cov=$(cargo llvm-cov --workspace --summary-only 2>/dev/null | grep "^TOTAL" | awk '{print $4}' | tr -d '%' || echo "N/A")
    else
        cur_cov="N/A"
    fi
    if command -v npx &>/dev/null; then
        cur_clones=$(npx jscpd crates/ --pattern "**/*.rs" --min-tokens 50 --min-lines 5 --mode strict 2>&1 | grep -c "Clone found" 2>/dev/null || echo "0")
    else
        cur_clones="N/A"
    fi

    printf "  %-30s %12s %12s %12s\n" "Dimension" "Current" "Baseline" "Delta"
    printf "  %-30s %12s %12s %12s\n" "---------" "-------" "--------" "-----"
    printf "  %-30s %12s %12s %+12s\n" "test_count" "$cur_test" "$base_test" "$((cur_test - base_test))"
    printf "  %-30s %12s %12s %+12s\n" "smell_markers" "$cur_smell" "$base_smell" "$((cur_smell - base_smell))"
    printf "  %-30s %12s %12s %+12s\n" "unsafe_blocks" "$cur_unsafe" "$base_unsafe" "$((cur_unsafe - base_unsafe))"
    printf "  %-30s %12s %12s %+12s\n" "unwrap_in_prod" "$cur_unwrap" "$base_unwrap" "$((cur_unwrap - base_unwrap))"
    printf "  %-30s %12s %12s %12s\n" "cyclomatic_CC>10" "$cur_cyclo" "$base_cyclo" "--"
    printf "  %-30s %12s %12s %12s\n" "line_coverage_pct" "$cur_cov" "$base_cov" "--"
    printf "  %-30s %12s %12s %12s\n" "jscpd_clone_count" "$cur_clones" "$base_clones" "--"

    echo ""
}

# ─── Update Baseline ──────────────────────────────────────────────────────────

update_baseline() {
    echo ""
    echo -e "${YELLOW}Updating baseline to current metrics...${NC}"

    local cur_test cur_smell cur_unsafe cur_unwrap cur_cyclo cur_cov cur_clones today
    cur_test=$(safe_count '#\[test\]\|#\[tokio::test\]' "*.rs")
    cur_smell=$(safe_count 'TODO\|FIXME\|HACK' "*.rs" "tests/")
    cur_unsafe=$(safe_count '\bunsafe\b' "*.rs" "tests/")
    cur_unwrap=$(safe_count '\.unwrap()' "*.rs" "tests/" "benches/" "particle_poc/")
    today=$(date -I 2>/dev/null || date +%Y-%m-%d)

    # New dimensions — try current value, fall back to existing baseline
    if command -v arborist &>/dev/null; then
        cur_cyclo=$(arborist crates/ --languages rust --format json 2>/dev/null | python3 -c "import json,sys;data=json.load(sys.stdin);print(sum(1 for f in data for func in f.get('functions',[]) if func.get('cyclomatic',0)>10))" 2>/dev/null || ron_value "cyclomatic_over_10")
    else
        cur_cyclo=$(ron_value "cyclomatic_over_10")
    fi
    if command -v cargo-llvm-cov &>/dev/null; then
        cur_cov=$(cargo llvm-cov --workspace --summary-only 2>/dev/null | grep "^TOTAL" | awk '{print $4}' | tr -d '%' || ron_value "line_coverage_pct")
    else
        cur_cov=$(ron_value "line_coverage_pct")
    fi
    if command -v npx &>/dev/null; then
        cur_clones=$(npx jscpd crates/ --pattern "**/*.rs" --min-tokens 50 --min-lines 5 --mode strict 2>&1 | grep -c "Clone found" 2>/dev/null || ron_value "jscpd_clone_count")
    else
        cur_clones=$(ron_value "jscpd_clone_count")
    fi

    cat > "$BASELINE" << RONEOF
// Quality Gate Baseline — ratchet thresholds for CI enforcement.
// Auto-updated: $today
//
// RULES:
//   Hard gates: current value must be 0. Any increase → CI FAIL.
//   Soft ratchets: current value must be <= baseline. Any increase → CI FAIL.
//     Decrease auto-updates baseline (quality improved).
//   Trend advisories: warn only. Do not block CI.
//
// To override a soft ratchet (intentional regression):
//   Add an entry to overrides{} with reason, then update the baseline value.

{
    "hard": {
        "clippy_warnings": 0,
        "audit_vulnerabilities": 0,
        "unused_dependencies": 0,
        "doc_warnings": 0,
        "dead_code_items": 0,
        "shader_errors": 0,
    },
    "soft": {
        "test_count_total": $cur_test,
        "smell_markers_total": $cur_smell,
        "unsafe_blocks_total": $cur_unsafe,
        "unwrap_in_prod_total": $cur_unwrap,
        "expect_in_prod_total": 92,
        "cyclomatic_over_10": $cur_cyclo,
        "line_coverage_pct": $cur_cov,
        "jscpd_clone_count": $cur_clones,
    },
    "advisory_maxima": {
        "magic_literals_per_file": 320,
        "function_length_lines": 861,
        "nesting_depth": 12,
        "file_length_lines": 861,
    },
    "overrides": {
        // "unsafe_blocks_total": {
        //     "value": 3,
        //     "reason": "GPU buffer mapping requires unsafe for zero-copy",
        //     "pr": "#NNN",
        // },
    },
    "meta": {
        "last_updated": "$today",
        "schema_version": 2,
        "notes": [
            "unsafe_blocks_total=$cur_unsafe: Send+Sync impls for cpal StreamHandle (FFI handle wrapper, soundness verified)",
            "expect_in_prod_total=92: counted across all crates, excludes tests/benches/particle_poc",
            "test_count_total=$cur_test: all #[test] and #[tokio::test] across workspace (including tools/)",
            "smell_markers_total=$cur_smell: no TODO/FIXME/HACK in production code",
            "cyclomatic_over_10=$cur_cyclo: functions with McCabe cyclomatic complexity >10 (arborist-cli)",
            "line_coverage_pct=$cur_cov: workspace line coverage from cargo-llvm-cov",
            "jscpd_clone_count=$cur_clones: duplicate code clones detected by jscpd (min-tokens=50)",
        ],
    },
}
RONEOF

    echo -e "${GREEN}Baseline updated.${NC} Review the diff and commit:"
    echo ""
    echo "  git diff docs/QUALITY_BASELINE.ron"
    echo "  git add docs/QUALITY_BASELINE.ron && git commit -m 'chore: ratchet quality baseline down'"
    echo ""
}

# ─── Install Hooks ────────────────────────────────────────────────────────────

install_hooks() {
    echo ""
    echo "Installing git hooks..."

    local hooks_path
    hooks_path=$(git config core.hooksPath 2>/dev/null || echo "")

    if [ "$hooks_path" = ".githooks" ]; then
        echo -e "  ${GREEN}Already configured${NC} (core.hooksPath = .githooks)"
    else
        git config core.hooksPath .githooks
        echo -e "  ${GREEN}Set${NC} core.hooksPath = .githooks"
    fi

    # Ensure hook scripts are executable
    chmod +x "$PROJECT_ROOT/.githooks/pre-commit" 2>/dev/null || true

    echo ""
    echo "Hooks installed:"
    echo "  pre-commit  → quality gate --soft (fast, <1s grep-based)"
    echo ""
    echo "Hard gates (clippy, docs, dead-code) run in CI via quality.yml — not in hooks."
    echo ""
    echo "To bypass hooks in an emergency:"
    echo "  git commit --no-verify ..."
    echo "  git push --no-verify ..."
    echo ""
}

# ─── Main ─────────────────────────────────────────────────────────────────────

MODE="${1:-full}"

cd "$PROJECT_ROOT"

case "$MODE" in
    --hard)
        check_hard_gates
        ;;
    --soft)
        check_soft_ratchets
        ;;
    --diff)
        show_diff
        ;;
    --update)
        update_baseline
        ;;
    --install-hooks)
        install_hooks
        exit 0
        ;;
    full)
        check_hard_gates
        check_soft_ratchets
        ;;
    *)
        echo "Usage: $0 [--hard|--soft|--diff|--update|--install-hooks]"
        echo "  (no flag)        Full check (hard + soft gates)"
        echo "  --hard           Hard gates only (clippy, docs, dead code)"
        echo "  --soft           Soft ratchets only (tests, smells, unsafe, unwrap)"
        echo "  --diff           Show current vs baseline comparison"
        echo "  --update         Update baseline to current metrics"
        echo "  --install-hooks  Install git pre-commit and pre-push hooks"
        exit 1
        ;;
esac

# Summary
echo ""
if [ "$FAIL" -gt 0 ]; then
    echo -e "${RED}═══ GATE FAILED: $FAIL regression(s) detected ═══${NC}"
    echo ""
    echo "Fix the regressions, or if intentional:"
    echo "  1. Edit docs/QUALITY_BASELINE.ron overrides section"
    echo "  2. Add a reason for the regression"
    echo "  3. Commit both the fix and the baseline update"
    exit 1
elif [ "$WARN" -gt 0 ]; then
    echo -e "${YELLOW}═══ GATE PASSED: $WARN dimension(s) improved — ratchet them down! ═══${NC}"
    echo ""
    echo "Run to lock in improvements:"
    echo "  ./scripts/quality-gate-check.sh --update"
    echo "  git add docs/QUALITY_BASELINE.ron && git commit -m 'chore: ratchet quality baseline down'"
else
    echo -e "${GREEN}═══ GATE PASSED: All dimensions at baseline ═══${NC}"
fi
