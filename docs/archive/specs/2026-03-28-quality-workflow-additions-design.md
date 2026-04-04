# Quality Workflow Additions

Add four new jobs to `.github/workflows/quality.yml` to replace the removed `mutants` job.

## Jobs

### 1. `udeps` — Unused dependency detection

**Tool:** `cargo-udeps` (requires nightly toolchain)
**Duration:** ~3-5 min (full rebuild under nightly)

Detects workspace dependencies declared in `Cargo.toml` that are not actually used by any source file.

**Steps:**
1. Install nightly toolchain (udeps requires compiler internals)
2. Install `cargo-udeps` via `cargo install cargo-udeps`
3. Run `cargo +nightly udeps --workspace --all-targets`
4. Fails if any unused deps found

### 2. `shaders` — WGSL shader validation

**Tool:** `naga-cli` (already depends on `naga` v24 in workspace)
**Duration:** ~10s

Validates all 10 `.wgsl` files against the naga WGSL frontend without requiring a full build or GPU.

**Files to validate:**
- `crates/engine_render/src/wgpu_renderer/*.wgsl` (5 shaders)
- `crates/card_game/src/shaders/*.wgsl` (5 shaders)

**Steps:**
1. Install `naga-cli` via cargo
2. Find all `.wgsl` files with `find crates -name '*.wgsl'`
3. Run `naga <file> --validate` on each
4. Fails on first invalid shader

### 3. `dead-code` — Unused code detection

**Tool:** `rustc` with `RUSTFLAGS="-D dead_code"`
**Duration:** ~3 min (full build)

Promotes the `dead_code` lint from warning to error. Catches unused functions, types, methods, and constants that survive across the workspace.

**Steps:**
1. Run `cargo build --workspace --all-targets` with `RUSTFLAGS="-D dead_code"`
2. Fails if any dead code exists

**Note:** This is a separate build from clippy (which runs its own lints). The `dead_code` lint is a rustc lint, not covered by clippy's pedantic group.

### 4. `duplicates` — Copy-paste detection

**Tool:** `jscpd` (Node.js, token-based Rabin-Karp)
**Duration:** ~10s
**Threshold:** 50 tokens minimum

Scans all `.rs` files in `crates/` for duplicate code blocks. Reports all duplicates before failing, mutant-hunt style.

**Steps:**
1. Install Node.js and `jscpd`
2. Run `jscpd crates/ --pattern "**/*.rs" --min-tokens 50 --reporters consoleFull --output jscpd-report`
3. Show full duplicate listing regardless of result
4. Upload `jscpd-report/` as artifact
5. Check exit code — fail if duplicates found

**Behavior:** jscpd lists all duplicate pairs with file paths, line ranges, and token counts. The job always shows the full report (using `if: always()` on the report step), then fails at the end if duplicates were found. This matches the mutants pattern — collect everything, show the list, then fail.

## Integration

All four jobs run in parallel alongside existing jobs (clippy, audit, doc, coverage). No dependencies between them.

## CLAUDE.md update

Update the CI Workflows section to mention the new jobs:

```
- **`quality.yml`** (daily at 06:00 UTC + manual `workflow_dispatch`): clippy, audit, docs, coverage, udeps, shader validation, dead code, duplicate detection.
```
