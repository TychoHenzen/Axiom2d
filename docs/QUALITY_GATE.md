# Quality Gate

Ratchet-based quality enforcement. Quality can only improve over time — any regression fails CI.

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│                    QUALITY GATE                           │
│                                                           │
│  ┌──────────┐  ┌──────────┐  ┌───────────────────┐       │
│  │ Hard     │  │ Soft     │  │ Trend Advisories  │       │
│  │ Gates    │  │ Ratchets │  │                   │       │
│  │ (fail=✗) │  │ (fail=✗) │  │ (warn only)      │       │
│  └────┬─────┘  └────┬─────┘  └────────┬──────────┘       │
│       │             │                │                    │
│       ▼             ▼                ▼                    │
│  ┌──────────────────────────────────────────────────┐    │
│  │        BASELINE COMPARISON ENGINE                 │    │
│  │  current_metrics vs QUALITY_BASELINE.ron          │    │
│  └──────────────────────┬───────────────────────────┘    │
│                         │                                │
│                         ▼                                │
│                ┌─────────────┐                           │
│                │ PASS / FAIL │                           │
│                └─────────────┘                           │
└──────────────────────────────────────────────────────────┘
```

## Dimensions

### Tier 1: Hard Gates (must be zero — failure blocks merge)

| Dimension | Metric | How Measured | Current |
|-----------|--------|-------------|---------|
| `clippy_warnings` | Warning count | `cargo clippy --workspace --all-targets -- -D warnings` count | 0 |
| `audit_vulnerabilities` | Known CVEs | `cargo audit` (built into quality.yml) | 0 |
| `unused_dependencies` | Unused crate deps | `cargo udeps --workspace --all-targets` | TBD |
| `doc_warnings` | Rustdoc warnings | `cargo doc --workspace --no-deps` with `RUSTDOCFLAGS=-D warnings` | 0 |
| `dead_code` | Dead code items | `cargo build --workspace --all-targets` with `RUSTFLAGS=-D dead_code` | 0 |
| `shader_validity` | Invalid WGSL | `naga` validation of all `.wgsl` files | 0 |

Rationale: These are always bugs or near-bugs. Zero tolerance.

### Tier 2: Soft Ratchets (cannot regress — failure blocks merge)

| Dimension | Metric | Scope | Current |
|-----------|--------|-------|---------|
| `test_count` | `#[test]` functions | per crate | ~1544 total |
| `smell_markers` | TODO/FIXME/HACK in production code | per crate | 0 |
| `unsafe_blocks` | `unsafe { }` blocks in production code | per crate | 2 total |
| `unwrap_in_prod` | `.unwrap()` in non-test code | per crate | 11 |
| `duplicate_blocks` | jscpd clone detection | workspace | per quality.yml |

Rationale: These are quality indicators that should never get worse. Improvements (lower counts) auto-ratchet the baseline down.

### Tier 3: Trend Advisories (warn only — does not block)

| Dimension | Metric | Scope |
|-----------|--------|-------|
| `magic_literals` | Unnamed numeric constants | top-10 files |
| `max_function_length` | Longest function per file | top-10 files |
| `max_nesting_depth` | Deepest nest per file | top-10 files |
| `file_length` | Lines per file | top-10 files |
| `arch_gaps` | Isolated nodes, thin communities | workspace |

Rationale: These have false positives (data tables, trait definitions, vertex coordinates). Tracking for awareness; blocks via human judgment.

## Baseline File

`docs/QUALITY_BASELINE.ron` stores the ratchet thresholds. Format:

```ron
{
    "hard": {
        "clippy_warnings": 0,
        "audit_vulnerabilities": 0,
        "unused_dependencies": 0,
        "doc_warnings": 0,
        "dead_code": 0,
        "shader_validity": true,
    },
    "soft": {
        "test_count_total": 1544,
        "smell_markers_total": 0,
        "unsafe_blocks_total": 2,
        "unwrap_in_prod_total": 11,
        "duplicate_blocks_max": None, // set after jscpd run
    },
    "advisory_thresholds": {
        "magic_literals_max_per_file": 320,
        "max_function_length": 861,
        "max_nesting_depth": 12,
        "max_file_length": 861,
    },
    "meta": {
        "last_updated": "2026-07-04",
        "updated_by": "quality-gate-init",
    },
}
```

## How It Works

### CI Check (daily, `quality.yml`)

1. Each quality job outputs metrics as a JSON artifact
2. `quality-gate` job downloads all artifacts, aggregates into current metrics
3. Compares current vs baseline:
   - Hard gates: current > 0 → **FAIL**
   - Soft ratchets: current > baseline → **FAIL**
   - Trend advisories: current > baseline → **WARN** (annotation on job)
4. If PASS: optionally auto-updates baseline (commits lower thresholds back to repo)

### Local Check

```bash
# Show current vs baseline diff
./scripts/quality-gate-check.sh --diff

# Soft ratchets only (fast, <1s)
./scripts/quality-gate-check.sh --soft

# Full gate (hard + soft — slow, runs cargo builds)
./scripts/quality-gate-check.sh

# Auto-update baseline after improvement
./scripts/quality-gate-check.sh --update

# Install git pre-commit hook
./scripts/quality-gate-check.sh --install-hooks
```

### Auto-Baseline Updates

When quality improves:
1. CI detects current < baseline for any soft ratchet dimension
2. Posts a PR updating `QUALITY_BASELINE.ron` with new lower values
3. PR title: `chore: ratchet quality baseline down (X improvements)`

Manual trigger: `cargo run --bin quality-gate -- update-baseline`

## Ratcheting Rules

1. **Hard gates can't regress.** If clippy-warnings goes 0→1, gate fails. Period.
2. **Soft ratchets can't regress.** If unsafe_blocks goes 2→3, gate fails. If it goes 2→1, baseline auto-updates to 1.
3. **Trend advisories don't block.** If max_function_length goes 861→900, job annotates a warning but doesn't fail.
4. **New crates start at zero.** When a new crate is added, its metrics initialize at current values (not blocked by workspace totals).
5. **Intentional regressions need explicit approval.** If you must increase a soft ratchet (e.g., adding an `unsafe` block for performance), update the baseline in the same PR with a justification comment in the RON file.

## Emergency Override

If the gate blocks legitimate work:

```ron
// In QUALITY_BASELINE.ron:
"overrides": {
    "unsafe_blocks_total": {
        "new_value": 3,
        "reason": "GPU buffer mapping requires unsafe for zero-copy access. Reviewed in PR #NNN.",
        "expires": "2026-08-01",  // optional: auto-revert after date
    },
}
```

Overrides are reviewed during PR and must include a reason.

## Integration with Existing Infrastructure

| Existing Check | Gate Tier | Notes |
|---------------|-----------|-------|
| CI autofix (clippy --fix + fmt) | Hard | Auto-fixes pushed back to branch |
| CI build-and-test | Hard | Already blocks merge |
| quality.yml clippy | Hard | `-D warnings` fails on any warning |
| quality.yml audit | Hard | `cargo audit` fails on any vulnerability |
| quality.yml docs | Hard | `RUSTDOCFLAGS=-D warnings` |
| quality.yml udeps | Hard | Fails on unused deps |
| quality.yml dead-code | Hard | `RUSTFLAGS=-D dead_code` |
| quality.yml shaders | Hard | naga validation |
| quality.yml duplicates | Soft | jscpd clone count — ratchet prevents regression |
| TECH_DEBT_LEDGER.md | Advisory | Manual/scripted structural scoring — feeds trend advisories |
| quality/MICRO_MUTATIONS.md | Advisory | Stochastic mutation testing (1 random file/day in CI) — see `scripts/micro-mutations.sh` |
| quality.yml complexity | Soft | Functions with cyclomatic complexity >10 (arborist-cli) — ratchet prevents regression |
| quality.yml coverage | Soft | Line coverage % (cargo-llvm-cov) — ratchet prevents regression |

## Future Dimensions

Dimensions to add as tooling matures:

- **Mutation score** (`cargo mutants`): % mutants killed. Full runs too slow for CI; stochastic micro-mutation runs daily (1 random file, `quality.yml` → `quality/MICRO_MUTATIONS.md`). Full hunt via `/mutant-hunt` skill.
- **Architectural coupling** (code-review-graph): cross-community edge count. Graph needs full build first.
- **Comment quality**: ratio of `///` doc comments to `pub` items. Measure documentation coverage.
- **Deprecation debt**: count of deprecated API uses. Future Rust editions may add this.
