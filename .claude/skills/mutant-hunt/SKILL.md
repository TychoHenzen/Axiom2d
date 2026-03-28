---
name: mutant-hunt
description: Run cargo-mutants locally to find surviving mutants, then fix them by adding or improving tests. Use when you want to hunt mutants, improve test coverage via mutation testing, or find weak spots in the test suite.
---

# Mutant Hunt

Run mutation testing locally with `cargo-mutants` and fix any surviving mutants by writing better tests.

## Workflow

### 1. Run mutation testing

Run cargo-mutants via WSL using the Windows toolchain:

```bash
cargo.exe mutants --no-shuffle -vV --in-place --timeout 30 --cap-lints=true
```

**Optional scoping** — if the user specifies a crate or file, pass `--file` or `-p` flags:
- Single crate: `cargo.exe mutants -p card_game --no-shuffle -vV --in-place --timeout 30 --cap-lints=true`
- Single file: `cargo.exe mutants --file crates/card_game/src/foo.rs --no-shuffle -vV --in-place --timeout 30 --cap-lints=true`

This will take a while. Run it and wait for it to complete.

### 2. Parse results

After the run completes, read `mutants.out/outcomes.json` and extract missed mutants:

```bash
jq -r '[.outcomes[] | select(.summary == "MissedMutant")] | .[] | "\(.scenario.Mutant.file):\(.scenario.Mutant.line) — \(.scenario.Mutant.name)"' mutants.out/outcomes.json
```

Also show the summary:

```bash
jq '{missed: [.outcomes[] | select(.summary == "MissedMutant")] | length, caught: [.outcomes[] | select(.summary == "CaughtMutant")] | length, timeout: [.outcomes[] | select(.summary == "Timeout")] | length, unviable: [.outcomes[] | select(.summary == "Unviable")] | length}' mutants.out/outcomes.json
```

### 3. Triage missed mutants

For each missed mutant:
1. Read the source file at the mutant's location
2. Understand what the mutation changes (the mutant name describes the transformation)
3. Decide: is this a **real gap** in test coverage, or a **trivial/uninteresting** mutation?

**Real gaps** — fix these:
- Mutants in business logic, algorithms, or state transitions
- Mutants that change observable behavior (return values, side effects)
- Mutants in boundary conditions or error paths

**Skip these** (mention them but don't fix):
- Mutants in pure display/formatting code with no behavioral impact
- Mutants where the mutation produces equivalent behavior (e.g., `>=` vs `>` when the boundary value is never hit)
- Mutants in generated code or trivial getters

### 4. Fix real gaps

For each real gap:
1. Write a test that would **catch** the mutant — the test must fail if the mutation were applied
2. Follow the project's test conventions (see CLAUDE.md: `when_action_then_outcome`, Arrange/Act/Assert, inline `#[cfg(test)] mod tests`)
3. Run `cargo.exe test` to verify the new test passes on unmutated code
4. Optionally re-run mutants on just that file to confirm the mutant is now caught

### 5. Report

After fixing, present a summary:
- Total mutants / caught / missed / timeout / unviable
- Which missed mutants were fixed (with test names)
- Which missed mutants were triaged as skip (with brief reason)
- New catch rate

## Rules

- **Don't write tests banned by CLAUDE.md** (derive tests, constructor-echo tests, etc.)
- **Do write behavioral tests** — assert on game state, not implementation details
- **Use `cargo.exe`** not `cargo` (Windows toolchain from WSL)
- **Run `cargo.exe fmt --all`** after adding tests
