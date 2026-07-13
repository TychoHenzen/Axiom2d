# Quality Metrics Tooling

How complexity, coverage, and duplication metrics are integrated into Axiom2d's CI and quality gates.

## Cyclomatic Complexity

**Tool:** [`arborist-cli`](https://crates.io/crates/arborist-cli) — Rust-native, reports cognitive + cyclomatic complexity per function.

```bash
# Install
cargo install arborist-cli

# Count functions with CC > 10
arborist crates/ --languages rust --format json | python3 -c "
import json, sys
data = json.load(sys.stdin)
print(sum(1 for f in data for func in f.get('functions',[])
    if func.get('cyclomatic',0) > 10))
"
```

**Note:** `cargo-cyclomatic` (audunhalland) doesn't exist on crates.io. `arborist-cli` is the practical alternative — it provides both cognitive and cyclomatic complexity via JSON output.

**Threshold:** ≤10 McCabe cyclomatic complexity per function, per `docs/architecture_bible.md` section 7 (NIST Special Publication 500-235).

**CI integration:**
1. `quality.yml` `complexity` job installs `arborist-cli`, runs it, outputs count
2. `soft-gate` job reads `needs.complexity.outputs.cyclo_over_10` and ratchets against `cyclomatic_over_10` in baseline

**Proof format for DoDs:**
```json
{
  "command": "arborist crates/ --languages rust --format json | python3 -c \"import json,sys;data=json.load(sys.stdin);print(sum(1 for f in data for func in f.get('functions',[]) if func.get('cyclomatic',0)>10))\"",
  "predicate": {"type": "regression", "extract": "^(\\d+)$", "lower_is_better": true},
  "description": "functions with cyclomatic complexity >10 does not increase",
  "category": "complexity"
}
```

## Code Coverage

**Tool:** [`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov) — faster than tarpaulin, uses LLVM coverage instrumentation.

```bash
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov

# Run (summary only — fastest)
cargo llvm-cov --workspace --summary-only

# Extract coverage %
cargo llvm-cov --workspace --summary-only 2>/dev/null | grep "^TOTAL" | awk '{print $4}' | tr -d '%'
```

**Why not tarpaulin:** `cargo-llvm-cov` is significantly faster (~50% on this workspace). Tarpaulin uses source-based instrumentation which is slower but works without nightly; llvm-cov needs `llvm-tools-preview` component but is the recommended approach for CI.

**CI integration:**
1. `quality.yml` `coverage` job generates coverage, extracts percentage from `TOTAL` line, uploads `lcov.info`
2. `soft-gate` job reads `needs.coverage.outputs.coverage_pct` and ratchets against `line_coverage_pct` in baseline
3. Uses `bc` for float comparison since coverage is a percentage

**Proof format for DoDs:**
```json
{
  "command": "cargo llvm-cov --workspace --summary-only 2>/dev/null | grep \"^TOTAL\" | awk '{print $4}' | tr -d '%'",
  "predicate": {"type": "regression", "extract": "^(\\d+)$", "lower_is_better": false},
  "description": "overall line coverage does not regress",
  "category": "coverage"
}
```

## Code Duplication

**Tool:** [`jscpd`](https://github.com/kucherenko/jscpd) (npm) — works on any text including Rust.

```bash
npm install -g jscpd
# or via npx:
npx jscpd crates/ --pattern "**/*.rs" --min-tokens 50 --min-lines 5 --mode strict

# Count clones
npx jscpd crates/ --pattern "**/*.rs" --min-tokens 50 --min-lines 5 --mode strict 2>&1 | grep -c "Clone found"
```

**Note:** `--path` flag was renamed to positional argument in jscpd v5.x. Use `npx jscpd crates/ ...` not `npx jscpd --path crates/`.

**CI integration:**
1. `quality.yml` `duplicates` job runs jscpd, counts "Clone found" lines, outputs `clone_count`
2. `soft-gate` job reads `needs.duplicates.outputs.clone_count` and ratchets against `jscpd_clone_count` in baseline
3. Formerly a hard-fail gate — now a soft ratchet with auto-ratchet on improvement

**Proof format for DoDs:**
```json
{
  "command": "npx jscpd crates/ --pattern \"**/*.rs\" --min-tokens 50 --min-lines 5 --mode strict 2>&1 | grep -c \"Clone found\" || echo \"0\"",
  "predicate": {"type": "regression", "extract": "^(\\d+)$", "lower_is_better": true},
  "description": "code duplication clone count does not increase",
  "category": "duplication"
}
```

## Quality Output Directory

All quality CI output artifacts live under `quality/` (not `docs/`):
- `quality/coverage/lcov.info` — coverage report
- `quality/duplicates/` — jscpd HTML/JSON reports
- `quality/MICRO_MUTATIONS.md` — micro-mutation tracking (human-readable)
- `quality/.micro_mutation_state.json` — micro-mutation state (machine-readable)

The baseline RON stays in `docs/QUALITY_BASELINE.ron` — it's documentation of thresholds, not CI output.

## Baseline Thresholds

Current baselines in `docs/QUALITY_BASELINE.ron`:

```ron
"soft": {
    // ... existing ...
    "cyclomatic_over_10": 29,       // functions with CC > 10
    "line_coverage_pct": 78.67,     // workspace line coverage %
    "jscpd_clone_count": 1197,      // jscpd clone count
}
```

All three are soft ratchets — they allow improvement (auto-ratchet down) but block regression.

## Local Check

`scripts/quality-gate-check.sh` supports all three dimensions:
- Cyclomatic complexity: if `arborist` is installed, checked; otherwise skipped
- Coverage: if `cargo-llvm-cov` is installed, checked; otherwise skipped
- Duplication: if `npx`/`jscpd` is available, checked; otherwise skipped

Use `--diff` to see current vs baseline, `--update` to ratchet down.
