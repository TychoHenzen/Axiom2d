---
name: quality-check
description: Trigger the Quality CI workflow, monitor all jobs, and fix any failures locally. Use when the user wants to run quality checks, fix CI failures, or ensure the codebase passes all quality gates.
---

# Quality Check

Trigger the GitHub Actions Quality workflow, monitor it to completion, and fix any failures locally.

## Workflow

### 1. Trigger the workflow

```bash
gh workflow run quality.yml
```

Wait a few seconds for the run to appear, then capture the run ID:

```bash
sleep 5
RUN_ID=$(gh run list --workflow=quality.yml --limit 1 --json databaseId --jq '.[0].databaseId')
```

### 2. Monitor to completion

Watch the run, but don't exit on failure — we want to inspect all results:

```bash
gh run watch "$RUN_ID" || true
```

### 3. Check results

Get the status of each job:

```bash
gh run view "$RUN_ID" --json jobs --jq '.jobs[] | "\(.name): \(.conclusion)"'
```

If all jobs passed, report success and stop.

### 4. Fix failures

For each failed job, fetch its logs and diagnose:

```bash
gh run view "$RUN_ID" --log-failed
```

Then fix the issues locally based on the job type:

#### clippy failures
- Read the specific warnings/errors from the log
- Fix the code issues locally
- Verify with `cargo.exe clippy --workspace --all-targets --keep-going -- -D warnings`

#### audit failures
- Read the advisory details from the log
- Check if the vulnerable dependency can be updated in `Cargo.toml`
- If it's a transitive dep, check if a parent dep has a newer version that pulls a patched version
- If no fix is available, document the advisory and consider `cargo audit --ignore RUSTSEC-XXXX-XXXX` with justification

#### doc failures
- Read the doc warnings from the log (broken intra-doc links, missing docs on public items)
- Fix the doc comments locally
- Verify with `cargo.exe doc --workspace --no-deps` (check for warnings manually since `RUSTDOCFLAGS` env var doesn't propagate through cargo.exe from WSL)

#### coverage failures
- Coverage job doesn't have a pass/fail threshold, so failures here are build/test failures
- Diagnose as you would a regular test failure

### 5. Verify fixes locally

After fixing all issues, run the relevant checks locally:

```bash
cargo.exe clippy --workspace --all-targets --keep-going -- -D warnings
cargo.exe test --workspace
cargo.exe doc --workspace --no-deps
```

### 6. Commit and re-trigger

If fixes were made:
1. Stage and commit the fixes
2. Push to remote
3. The push triggers CI (ci.yml). Wait for it to pass.
4. Optionally re-trigger the quality workflow to confirm all jobs pass:
   ```bash
   gh workflow run quality.yml
   ```

## Rules

- **Use `cargo.exe`** not `cargo` (Windows toolchain from WSL)
- **Fix issues in the code**, don't suppress warnings unless there's a genuine false positive
- **Report unfixable issues** (e.g., upstream vulnerability with no patch) clearly to the user
- **Don't loop** — if a fix doesn't work after one attempt, report the issue and ask for guidance
