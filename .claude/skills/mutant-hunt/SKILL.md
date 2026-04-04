---
name: mutant-hunt
description: Run cargo-mutants locally to find surviving mutants, then fix them by adding or improving tests. Use when you want to hunt mutants, improve test coverage via mutation testing, or find weak spots in the test suite.
---

# Mutant Hunt

Run mutation testing locally with `cargo-mutants` and fix any surviving mutants by writing better tests.

## Prerequisites

If `cargo-mutants` is not installed:

```bash
cargo.exe install cargo-mutants --locked
```

Verify: `cargo.exe mutants --version`

## Workflow

### 1. Run mutation testing

Run cargo-mutants via WSL using the Windows toolchain:

```bash
cargo.exe mutants --no-shuffle -vV --in-place --timeout 30 --cap-lints=true
```

**Optional scoping** — if the user specifies a crate or file, pass `--file` or `-p` flags:
- Single crate: `cargo.exe mutants -p card_game --no-shuffle -vV --in-place --timeout 30 --cap-lits=true`
- Single file: `cargo.exe mutants --file crates/card_game/src/foo.rs --no-shuffle -vV --in-place --timeout 30 --cap-lints=true`

This will take a while. Run it and wait for it to complete.

### 2. Parse aggregate results

After the run completes, read `mutants.out/outcomes.json` and extract missed mutants:

```bash
jq -r '[.outcomes[] | select(.summary == "MissedMutant")] | .[] | "\(.scenario.Mutant.file):\(.scenario.Mutant.line) — \(.scenario.Mutant.name)"' mutants.out/outcomes.json
```

Also show the summary:

```bash
jq '{missed: [.outcomes[] | select(.summary == "MissedMutant")] | length, caught: [.outcomes[] | select(.summary == "CaughtMutant")] | length, timeout: [.outcomes[] | select(.summary == "Timeout")] | length, unviable: [.outcomes[] | select(.summary == "Unviable")] | length}' mutants.out/outcomes.json
```

### 3. Per-test attribution (find trash and bloated tests)

Parse the log files for every caught mutant to build a reverse index: which tests caught which mutants. This surfaces two categories of problem tests:

- **Zero-kill tests** — never caught any mutant; likely banned test patterns from CLAUDE.md
- **Zero-unique-kill tests** — caught mutants, but every one was also caught by another test; 100% redundant
- **Mutants caught by many tests** — a single mutation breaking many tests means those tests lack isolation; they're testing the same behavior and any one of them is redundant

**Get all test names in the workspace:**

```bash
cargo.exe test --workspace -- --list 2>&1 | grep ': test' | sed 's/: test//' | sort -u > /tmp/all_tests.txt
echo "Total tests: $(wc -l < /tmp/all_tests.txt)"
```

**Build attribution table and flag problems:**

```python
# Save as /tmp/mutant_attribution.py and run: python3 /tmp/mutant_attribution.py
import json, re, os
from collections import defaultdict

def to_wsl(path):
    """Convert Windows absolute path to WSL path."""
    p = path.replace('\\', '/')
    if len(p) >= 2 and p[1] == ':':
        drive = p[0].lower()
        p = f"/mnt/{drive}{p[2:]}"
    return p

with open('mutants.out/outcomes.json') as f:
    data = json.load(f)

# mutant_id → set of tests that caught it
mutant_catchers = {}

for i, outcome in enumerate(data['outcomes']):
    if outcome['summary'] != 'CaughtMutant':
        continue
    mutant_id = f"{outcome['scenario']['Mutant'].get('file', '?')}:{outcome['scenario']['Mutant'].get('line', '?')} {outcome['scenario']['Mutant'].get('name', '?')}"
    catchers = set()
    for phase in outcome.get('phase_results', []):
        if phase['phase'] != 'Test':
            continue
        log_file = to_wsl(phase.get('log_file', ''))
        if not log_file or not os.path.exists(log_file):
            continue
        with open(log_file) as lf:
            for line in lf:
                m = re.search(r'^test (.+) \.\.\. FAILED', line)
                if m:
                    catchers.add(m.group(1).strip())
                    continue
                m = re.search(r'FAILED \[.*?\] \S+ (.+)', line)
                if m:
                    catchers.add(m.group(1).strip())
    if catchers:
        mutant_catchers[mutant_id] = catchers

# Per-test counts
total_kills = defaultdict(int)
unique_kills = defaultdict(int)

for mutant_id, catchers in mutant_catchers.items():
    for t in catchers:
        total_kills[t] += 1
    if len(catchers) == 1:
        unique_kills[list(catchers)[0]] += 1

# Load all known tests
all_tests = set()
try:
    with open('/tmp/all_tests.txt') as f:
        all_tests = {l.strip() for l in f if l.strip()}
except FileNotFoundError:
    pass

zero_kill  = sorted(all_tests - set(total_kills.keys()))
zero_unique = sorted(t for t in total_kills if unique_kills[t] == 0)

# Mutants caught by suspiciously many tests (threshold: >5)
BLOAT_THRESHOLD = 5
bloated_mutants = {m: c for m, c in mutant_catchers.items() if len(c) > BLOAT_THRESHOLD}

print(f"\n=== Per-test attribution ===")
print(f"{'Test':<60} {'Total':>6} {'Unique':>7}")
print("-" * 75)
for test in sorted(total_kills, key=lambda t: -unique_kills[t]):
    print(f"{test:<60} {total_kills[test]:>6} {unique_kills[test]:>7}")

print(f"\n=== TRASH: zero-kill tests ({len(zero_kill)}) ===")
print("These tests never caught any mutant. Check against banned patterns in CLAUDE.md.")
for t in zero_kill:
    print(f"  {t}")

print(f"\n=== REDUNDANT: zero-unique-kill tests ({len(zero_unique)}) ===")
print("Every mutant these tests catch is also caught by at least one other test.")
for t in zero_unique:
    print(f"  {t}  (total kills: {total_kills[t]})")

print(f"\n=== BLOATED MUTANTS: caught by >{BLOAT_THRESHOLD} tests ({len(bloated_mutants)}) ===")
print("A single change breaking many tests signals lack of test isolation.")
for mutant_id, catchers in sorted(bloated_mutants.items(), key=lambda x: -len(x[1])):
    print(f"\n  {mutant_id}")
    print(f"    Caught by {len(catchers)} tests:")
    for t in sorted(catchers):
        print(f"      {t}")
```

**Interpreting the three output sections:**

| Section | What it means | Action |
|---|---|---|
| Zero-kill | Test catches no mutations | Cross-check CLAUDE.md banned list; delete if it's a banned pattern |
| Zero-unique-kill | Fully redundant — another test covers everything it covers | Delete unless it tests a different layer (integration vs unit) |
| Bloated mutants | One code change breaks many tests | Identify which of those tests is the right one; consider whether the others are over-broad |

### 4. Triage missed mutants

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

### 5. Fix real gaps

For each real gap:
1. Write a test that would **catch** the mutant — the test must fail if the mutation were applied
2. The test should be **specific**: it should fail for exactly this reason, not because it's broadly asserting everything
3. Follow the project's test conventions (see CLAUDE.md: `when_action_then_outcome`, Arrange/Act/Assert, `tests/suite/` layout)
4. Run `cargo.exe test` to verify the new test passes on unmutated code
5. Optionally re-run mutants on just that file to confirm the mutant is now caught

### 6. Report

After fixing, present a summary:
- Total mutants / caught / missed / timeout / unviable
- Which missed mutants were fixed (with test names)
- Which missed mutants were triaged as skip (with brief reason)
- Trash/redundant tests flagged, and which were deleted
- New catch rate

## Rules

- **Don't write tests banned by CLAUDE.md** (derive tests, constructor-echo tests, etc.)
- **Do write behavioral tests** — assert on game state, not implementation details
- **One behavior per test** — a new test should catch exactly the mutants it's designed for, not broadly assert everything
- **Use `cargo.exe`** not `cargo` (Windows toolchain from WSL)
- **Run `cargo.exe fmt --all`** after adding tests
