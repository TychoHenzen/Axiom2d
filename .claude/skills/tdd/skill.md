---
name: tdd
description: Test-Driven Development workflow with pair programming and aggressive quality improvement at every step. Project-local override that adds living documentation requirements.
argument-hint: "[problem statement]"
allowed-tools: [Task, Read, Write, Edit, AskUserQuestion, Glob, Grep, Bash(git *:ls*:mkdir*:rm*:dotnet *:npm *:cargo *:go *:python *:pytest*:make*)]
---

<objective>
Apply Test-Driven Development to: $ARGUMENTS

This is a pair programming TDD session. You drive, the user navigates. Build software through disciplined red-green-refactor cycles with aggressive quality improvement at every step.
</objective>

<context>
Project structure: !`ls -d *.csproj *.fsproj pyproject.toml package.json Cargo.toml go.mod pom.xml build.gradle **/*.csproj **/*.fsproj 2>/dev/null | head -20 || echo "(no project files found)"`
Git status: !`git status --short 2>/dev/null || echo "(not a git repository)"`
Test infrastructure: !`ls -d **/*[Tt]est*.*proj **/*test*.py **/*.test.js **/*.test.ts **/*_test.go **/*Test.java 2>/dev/null | head -10 || echo "(no test projects found)"`
</context>

<fresh_code_principles>
**CRITICAL - Foundational mindset for ALL code in this workflow:**

1. **All code is fresh** - We are writing new software, not maintaining deployed production systems. No existing user base to protect.
2. **No backwards compatibility** - Never create adapters, shims, wrappers, or compatibility layers. Just write it right.
3. **No legacy accommodations** - Don't preserve old interfaces "just in case." Change it and update all callers.
4. **"About what you'd expect"** - Every function, class, and module does exactly what its name suggests. A reader should think "yeah, that's about what I'd expect."
5. **Delete fearlessly** - Unused code is deleted. Simplifiable code is simplified. Tests catch mistakes.
6. **No speculation** - Don't add features, parameters, or abstractions for hypothetical future needs.
</fresh_code_principles>

<process>
**Phase 1: Understand the Problem (Pair Programming Intake)**
1. Read the problem statement
2. Use AskUserQuestion to clarify:
   - What specific behavior needs to exist?
   - What are the inputs and outputs?
   - Edge cases to handle?
   - Constraints or preferences?
3. Do NOT proceed until there is a clear, shared understanding

**CRITICAL: Question Quality Standards**
When asking questions (in ANY phase), follow these rules:
- **Assume expert developer, unfamiliar codebase**: The user is a skilled developer who does NOT know the internal names, module responsibilities, or architecture of this specific codebase. Every question must be self-contained.
- **Define every project-specific term**: If a question references a module, type, trait, struct, or concept from this codebase (e.g. `engine_app`, `EcsPlugin`, `prelude`), include a 1-sentence explanation of what it is and what it does. Example: "engine_app (the crate that defines the main App struct and plugin registration system)"
- **Explain the "why" behind each option**: Don't just list what an option includes — explain what it enables, what it defers, and what trade-offs it creates. Example: instead of "No app integration yet", say "This defers plugin registration, meaning the ECS schedule won't run automatically when the app starts — you'd wire that up manually or in a later step."
- **Make options evaluable in isolation**: A reader should be able to pick an option without needing to look up any referenced types, modules, or concepts. Each option description should stand on its own.
- **Avoid bare jargon**: Terms like "prelude", "bridging", "change detection helpers" need brief definitions when first used. A "prelude" is a convenience module that re-exports commonly used types so users can `use crate::prelude::*` instead of importing individually.

**Phase 2: Research**
4. Launch tdd-research agent to discover project language, framework, test infrastructure, existing patterns, and related code
5. Share key findings with user (informational, no checkpoint needed)

**Phase 3: Decompose into Test Cases**
6. Launch tdd-decomposer agent with problem statement + research findings
7. Present the test case plan to user via AskUserQuestion:
   - Show each test case with its Given/When/Then description
   - Ask: "Does this breakdown capture the right behaviors?"
8. Revise based on feedback until user approves

**Phase 4: TDD Cycles**

FOR EACH test case in execution order:

**4a. RED - Write Failing Test**
- Launch tdd-red-agent with test case + research context
- **Include the `/// @doc:` annotation requirements from <living_documentation> in the agent prompt**
- Agent returns 2-3 candidate tests with different approaches
- Launch tdd-voting-judge to auto-select the best candidate
- Write the selected test to the codebase (show it to the user as informational output, no question)
- Run the test and verify it FAILS
- If test passes: inform user, ask whether to skip or adjust

**4b. GREEN - Make Test Pass**
- Launch tdd-green-implementation with failing test + context
- Agent returns 2-3 candidate implementations
- Launch tdd-voting-judge to auto-select the best candidate
- Apply the selected implementation (show it to the user as informational output, no question)
- Run ALL tests - verify they pass with no regressions
- If tests fail: report to user, ask how to proceed

**4c. REFACTOR - Aggressive Quality Improvement**
- Launch tdd-refactor-reviewer with:
  - Files changed in this cycle
  - Adjacent files (same directory + files in the import chain of changed files)
  - scope: "cycle"
- Agent actively refactors: removes dead code, deduplicates, simplifies, applies clean code principles
- **Agent also reviews `/// @doc:` annotations on new tests** — improves thin ones, removes redundant ones
- Safe changes applied automatically; risky changes reported back
- Present any risky proposed changes to user via AskUserQuestion
- Run ALL tests after refactoring to verify no regressions
- If tests break: revert the breaking change, report to user

**4d. Cycle Checkpoint**
- Summarize what was accomplished
- Use AskUserQuestion: "Continue to next test case? / Adjust the plan? / Done for now?"

NEXT test case...

**Phase 5: Session-Wide Cleanup**
After all test cases are complete (or user says "done for now"):
- Launch tdd-refactor-reviewer with ALL files changed during this session, scope: "session"
- Look for cross-cutting improvements: deduplication across test cases, shared patterns to extract, naming consistency
- **Review all `/// @doc:` annotations written during the session for consistency, redundancy, and quality**
- Present proposed changes to user via AskUserQuestion
- Apply approved changes and verify tests pass

**Phase 6: Completion**
- List all test cases with pass/fail status
- List all files created/modified
- Summarize quality improvements from refactoring
- Verify clean test suite
</process>

<living_documentation>
**CRITICAL: Every new test MUST include a `/// @doc:` annotation.**

This project uses a living documentation system (`cargo.exe run -p living-docs -- --llm`) that extracts `/// @doc:` annotations from tests into `Doc/Living_Documentation_LLM.md`. Annotations are user-facing documentation — they explain *why* a test matters, not just *what* it checks.

### Format

Multi-line `/// @doc:` block placed directly above `#[test]`. Use `///` continuation lines:
```rust
/// @doc: Cards entering the hand lose their physics body so they can't be
/// knocked around by table collisions. Without this, a card you've already
/// picked up could get launched off-screen by another card sliding into it,
/// which would be confusing and break the hand inventory's spatial layout.
#[test]
fn when_card_enters_hand_then_physics_body_removed() {
```

### What makes a good annotation

An annotation connects the test to a **design decision, invariant, or user-visible behavior** — explaining not just *what* the test checks but *why* it matters and *what would go wrong* without it.

**Good annotations** (2-4 sentences, ~40-80 words, explain design intent + consequences):
```rust
/// @doc: The fixed timestep accumulator carries fractional frame time across
/// frames so that physics always runs at a consistent tick rate regardless of
/// render FPS. If the remainder were discarded, fast machines would simulate
/// slightly less total time than slow ones, causing drift in deterministic
/// replays and making physics-dependent gameplay subtly frame-rate-dependent.

/// @doc: When the emitter and listener occupy the exact same position, the
/// angle between them is undefined (atan2(0,0)). The panning system handles
/// this by defaulting to centered stereo rather than producing NaN, which
/// would propagate through the mix and silence the entire audio output.
```

**Bad annotations** (too terse, just restate the test name — do NOT write these):
```rust
/// @doc: Tests that color converts from u8  // just restates the test name
/// @doc: Checks the hit test function       // says nothing about why
/// @doc: Verifies the system works           // completely vacuous
```

### Style rules

- **Design-intent first**: Lead with the design decision or invariant, then explain what would break without it.
- **Domain language**: Use the game/engine vocabulary — "card", "hand", "stash", "flip", "drag", "camera", "atlas", "render layer", "sort order".
- **Edge cases get context**: If the test exists because of a specific edge case, explain the failure mode it prevents.
- **No redundancy with test name**: The test name says *what*. The annotation says *why it matters* and *what would go wrong*.
- **Skip when trivial**: If a test is truly self-explanatory from its name and there's no deeper "why", skip the annotation rather than writing a thin one. Not every test needs one — only add them when they add genuine information.

### When to skip annotations

These tests typically don't need annotations:
- Simple arithmetic operator tests where the test name fully describes the behavior
- Trivial getter/setter validation (though these should rarely exist per the banned test patterns)

### Integration with TDD phases

- **RED phase (tdd-red-agent)**: Write the `/// @doc:` annotation as part of the test. The annotation is written at the same time as the test, not retroactively. Think of it as part of the test's specification — if you can't articulate why this test matters in 2-4 sentences, reconsider whether the test case is well-scoped.
- **REFACTOR phase (tdd-refactor-reviewer)**: Review annotations for quality. Improve thin ones, remove redundant ones that just restate the test name.
- **SESSION CLEANUP phase**: Check annotation consistency across all tests written in this session.
</living_documentation>

<candidate_selection>
**How Candidate Selection Works (Red and Green Phases):**

Candidate selection is AUTOMATIC — the user should NOT be asked to pick between implementation variations.
Choosing between mocking strategies, assertion styles, complexity levels, or structural approaches is an
implementation detail, not a design decision. The tdd-voting-judge agent makes these calls.

**Flow:**
1. Agent generates 2-3 candidates (for quality through diversity)
2. Present the candidates to the tdd-voting-judge agent using the structured format below
3. The voting judge selects the best candidate
4. Show the selected test/implementation to the user as informational output (not a question)
5. Proceed immediately

For RED phase (test selection), send to tdd-voting-judge:
```
Problem: "{behavior_description}"
Criteria: "Which test most precisely validates this single behavior? Annotation quality is a factor — the /// @doc: annotation should explain design intent, not restate the test name."

A: {approach_name}
   Tradeoff: {tradeoff}
   Code:
   {full test code including /// @doc: annotation}

B: {approach_name}
   Tradeoff: {tradeoff}
   Code:
   {full test code including /// @doc: annotation}

C: {approach_name}
   Tradeoff: {tradeoff}
   Code:
   {full test code including /// @doc: annotation}
```

For GREEN phase (implementation selection), send to tdd-voting-judge:
```
Problem: "Make test '{test_name}' pass"
Criteria: "Which implementation is simplest while following project patterns?"

A: {approach_name}
   Tradeoff: {tradeoff}
   Code:
   {implementation code}

B: {approach_name}
   Tradeoff: {tradeoff}
   Code:
   {implementation code}

C: {approach_name}
   Tradeoff: {tradeoff}
   Code:
   {implementation code}
```

**What IS a design decision (ask the user):**
- Problem clarification (Phase 1)
- Which behaviors to test and in what order (Phase 3)
- Whether to skip a test that already passes
- Whether to adjust a test that can't be satisfied
- Risky refactoring that might change behavior
- Whether to continue, adjust the plan, or stop (cycle checkpoints)

**What is NOT a design decision (don't ask):**
- Mocking strategy (full isolation vs integration-style vs balanced)
- Assertion precision (exact value vs shape vs behavioral)
- Setup complexity (inline vs builder vs realistic)
- Implementation complexity (minimal vs practical vs robust)
- Structural approach (inline vs extract method vs extract class)
- Pattern application (direct vs strategy vs factory)
- `/// @doc:` annotation wording (quality is judged by the voting judge)
</candidate_selection>

<refactoring_scope>
**Per-Cycle Refactoring (scope: "cycle"):**
1. **Changed files**: All files created or modified in this red-green cycle
2. **Adjacent files**: Files in the same directory as changed files AND files that import/reference or are imported by changed files
3. Focus: Dead code, LLM-isms, naming, dedup with adjacent code, simplification
4. **`/// @doc:` quality**: Review annotations on new tests — improve thin ones, remove ones that just restate the test name

**Session-Wide Cleanup (scope: "session"):**
1. **All session files**: Every file created or modified during any cycle
2. **Cross-cutting**: Patterns repeated across multiple test cases
3. **Naming consistency**: Consistent naming across all new code
4. **Shared abstractions**: Extract common patterns ONLY if they genuinely reduce duplication (not speculative)
5. **Annotation consistency**: Ensure `/// @doc:` annotations across all new tests use consistent style, domain vocabulary, and depth
</refactoring_scope>

<quality_standards>
**The "About What You'd Expect" Standard:**

Every piece of code passes this test: a competent developer reads the function name, then reads the body, and thinks "yeah, that's about what I'd expect."

**Specific Standards:**
- Self-documenting names that reveal intent
- Functions do one thing well
- No dead code (unused imports, methods, variables, parameters)
- No duplicated logic - extract shared behavior
- No magic numbers or strings - use named constants
- No backwards compatibility layers
- No speculative generality
- No LLM-isms (narrative comments, TODOs, "let me..." comments)

**Test Standards:**
- Test NAMES use Given/When/Then format: `Given_Precondition_When_Action_Then_Outcome`
- Test BODIES use Arrange/Act/Assert with `// Arrange`, `// Act`, `// Assert` section markers
- Every new test includes a `/// @doc:` annotation (unless truly self-explanatory — see <living_documentation>)
- Reading all test names top-to-bottom should describe the complete feature
- Tests verify behavior, not implementation details
- Tests survive internal refactors without breaking
</quality_standards>

<error_recovery>
**Test Won't Fail (Red phase)**
-> Ask user: "Test passes immediately - behavior already exists. Skip this test case, or test a more specific edge case?"

**Implementation Won't Pass (Green phase)**
-> Ask user: "Can't satisfy the test with minimal changes. Adjust the test, try a different approach, or decompose into smaller steps?"

**Refactoring Breaks Tests**
-> Automatically revert the breaking change
-> Ask user: "This refactoring changed behavior. Skip it, try a smaller refactoring, or adjust manually?"

**Regeneration Requested**
-> Ask user what they want different
-> Re-invoke agent with specific guidance
</error_recovery>

<implementation_note>
**YOU (main Claude instance) are the orchestrator.** You will:

1. **Invoke specialized agents** using the Task tool:
   - `subagent_type='tdd-research'` - Discover codebase patterns
   - `subagent_type='tdd-decomposer'` - Break problem into test cases
   - `subagent_type='tdd-red-agent'` - Generate candidate failing tests
   - `subagent_type='tdd-green-implementation'` - Generate candidate implementations
   - `subagent_type='tdd-voting-judge'` - Auto-select best candidate from red/green agents
   - `subagent_type='tdd-refactor-reviewer'` - Actively refactor and clean code

2. **Auto-select candidates** via tdd-voting-judge (NOT by asking the user)

3. **Execute mechanical steps** (running tests, applying code) without asking

4. **Ask user ONLY at design decision points:**
   - Problem clarification (intake)
   - Test case plan approval (decomposition)
   - Risky refactoring approval (refactor phase)
   - Cycle continuation (checkpoint)
   - Session cleanup approval (final cleanup)

   Do NOT ask the user to pick between test or implementation variations — that's an implementation detail.

5. **Track progress informally** - summarize at checkpoints rather than maintaining state files

6. **Pass `/// @doc:` requirements to tdd-red-agent**: When launching the red agent, include the full <living_documentation> section in the prompt so candidates include annotations.

Start by understanding the problem statement, then launch research.
</implementation_note>
