---
name: livingdoc
description: >
  Autonomous test suite hygiene and living documentation maintenance. Runs the living-docs
  generator, reads the full test index, then systematically removes useless tests and adds
  or improves /// @doc: annotations across the codebase. Use this skill whenever the user
  mentions living documentation, test cleanup, test quality audit, doc annotations, useless
  tests, test hygiene, @doc comments, or wants to improve the test suite's documentation
  value. Also triggers on "clean up tests", "find bad tests", "improve test annotations",
  "expand doc coverage", or any mention of the living-docs tool.
---

# Living Documentation & Test Hygiene

You are performing an autonomous sweep of the Axiom2d test suite. Your two goals are:

1. **Remove or rewrite useless tests** that don't validate meaningful behavior
2. **Add and improve `/// @doc:` annotations** so the living documentation output is genuinely useful to someone reading it

This is autonomous work — make changes directly, don't produce reports. Budget your effort to fit within ~100k tokens by focusing on low-hanging fruit first and working breadth-first across crates.

## Phase 0: Generate the Baseline

Run the living-docs generator to get a current snapshot of every test in the project:

```bash
cargo.exe run -p living-docs -- --llm
```

This produces `Doc/Living_Documentation_LLM.md` — a token-efficient index of every test with:
- Pass/fail status
- Human-readable description (auto-converted from test name)
- `file:line` location
- Existing `/// @doc:` annotation (if any)

Read this file. It is your map for the entire session. Use the `file:line` references to navigate directly to tests that need attention.

## Phase 1: Identify and Remove Useless Tests

Scan the living-docs output and the actual test source code for tests that fall into these categories. The project's CLAUDE.md already bans these patterns — your job is to find any that slipped through:

### Banned patterns (delete on sight)

- **Prelude/re-export tests**: Tests that just verify `use crate::prelude::*` makes types available
- **Derive tests**: Tests that verify `Clone`, `Copy`, `PartialEq`, `Debug`, `Hash` derives work
- **Struct construction tests**: Tests that verify `Foo { x: 1 }.x == 1`
- **Resource insertion tests**: Tests that verify `world.insert_resource(X)` followed by `.is_some()`
- **Component spawn tests**: Tests that verify `world.spawn(C)` makes `C` queryable
- **Trivial default tests**: Tests that verify `Default` returns the literal values written in the impl
- **Boxing/trait-object tests**: Tests that verify `Box::new(X) as Box<dyn Trait>` compiles
- **Serde roundtrip on derived impls**: Tests that roundtrip types using only `#[derive(Serialize, Deserialize)]`
- **PartialEq on derived impls**: Tests that verify derived `PartialEq` distinguishes variants/fields
- **Constructor-echo tests**: Tests that verify `Foo::new(a, b).field == a` when the constructor just stores its args

### Additional smells to flag and fix

- **Assert-only-is_some/is_ok**: Tests that call a function and only assert `.is_some()` or `.is_ok()` without checking the actual value — these pass even when the function returns garbage
- **Duplicate tests**: Two tests that exercise the same code path with the same inputs and assert the same thing
- **Tautological assertions**: Tests where the expected value is computed the same way as the actual value (e.g., `assert_eq!(a + b, a + b)`)
- **Dead assertions**: Tests with assertions that can never fail given the setup (e.g., asserting a freshly-constructed empty vec has `.len() == 0`)

### How to handle removals

- Delete the entire test function including any `/// @doc:` annotation above it
- If removing a test leaves a `#[cfg(test)] mod tests` block empty, delete the entire test module
- If removing a test makes an import unused, remove the import too
- Do NOT delete tests that look simple but actually test custom logic (arithmetic operators, `from_u8` conversions, non-trivial constructors that validate or compute)

## Phase 2: Add and Improve `/// @doc:` Annotations

After the cleanup pass, work through the test index and add `/// @doc:` annotations to unannotated tests, and improve thin or unhelpful existing ones.

### What makes a good annotation

An annotation explains **why the behavior matters** to someone reading the living documentation. It is user-facing documentation that connects the test to a design decision, invariant, or user-visible behavior.

**Format**: Single line, placed directly above `#[test]`:
```rust
/// @doc: Explanation of why this behavior matters
#[test]
fn when_action_then_outcome() {
```

**Good annotations** (explain the "why"):
```rust
/// @doc: Constant-power stereo panning — emitter fully right produces 100% right channel gain
/// @doc: Coincident positions must not produce NaN — atan2(0,0) edge case handled by defaulting to centered pan
/// @doc: Cards entering the hand lose their physics body so they can't be knocked around by table collisions
/// @doc: Fixed timestep accumulates fractional frame time — ensures physics runs at consistent rate regardless of render FPS
```

**Bad annotations** (just restate the test name):
```rust
/// @doc: Tests that color converts from u8  ← just restates "when_color_from_u8"
/// @doc: Checks the hit test function       ← says nothing about why hit testing matters
/// @doc: Verifies the system works           ← completely vacuous
```

### Style guidelines

- **Succinct**: One sentence, ideally under 120 characters. If you need two sentences, the test might be doing too much.
- **Design-intent first**: Lead with the design decision or invariant, not the mechanism. "Cards in hand are immune to physics collisions" not "The system removes the RigidBody component".
- **Domain language**: Use the game/engine's vocabulary — "card", "hand", "stash", "flip", "drag", "camera", "atlas", "render layer", "sort order".
- **Edge cases get context**: If the test exists because of a specific edge case or past bug, say so. "Division by zero when velocity is exactly zero — clamp prevents NaN propagation."
- **No redundancy with test name**: The test name already says *what* happens. The annotation says *why it matters* or *what design decision it protects*.

### Prioritization

Work breadth-first across crates. Prioritize:
1. Tests with complex or non-obvious behavior (physics, rendering, ECS system interactions)
2. Tests that protect edge cases or past bugs
3. Tests in the card_game crate (active development area)
4. Simple unit tests last (they're often self-explanatory from the name)

Skip tests where the name is already perfectly self-documenting and there's no deeper "why" to explain. Not every test needs an annotation — only add them when they add genuine information.

## Phase 3: Verify

After making changes:

```bash
cargo.exe test
cargo.exe fmt --all
```

All tests must still pass. If you deleted a test that was the only consumer of some import or helper, clean up the resulting compiler errors.

## Working Efficiently

- Read tests in batches by file (use the `file:line` references from the living-docs output)
- Make all edits to a file in one pass — don't revisit files
- When a file has both useless tests to remove AND tests needing annotations, do both in the same edit
- If you're running low on budget, stop annotation work (Phase 2) before cleanup work (Phase 1) — removing bad tests has higher ROI than adding annotations
