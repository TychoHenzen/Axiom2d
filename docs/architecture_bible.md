# 10 design decisions that keep a game engine maintainable

**Architecture choices — not code cleanliness — are the single greatest predictor of long-term software maintainability.** A Carnegie Mellon SEI study of 536 practitioners found architectural decisions are the #1 source of consequential technical debt, and a six-country survey of 653 developers showed that roughly **25% of all development effort** gets wasted on debt-caused rework. The principles below are ordered by impact, drawn from empirical research and the collected wisdom of Ousterhout, Martin, Fowler, Gregory, and the Rust/Bevy communities. Each can be mechanically checked against a real codebase.

---

## 1. Enforce an acyclic, layered dependency graph between crates

**The principle.** Source code dependencies must form a directed acyclic graph (DAG) where lower-level crates never import higher-level ones. For a 2D engine: `engine_core` (math, time) → `engine_ecs` (bevy_ecs integration) → `engine_render` (wgpu + lyon) → `engine_audio` (fundsp) → `game` (game-specific logic). The game crate depends on engine crates; engine crates never depend on the game crate. No cycles anywhere.

**Why it matters.** Robert Martin's Acyclic Dependencies Principle (ADP) and Stable Dependencies Principle (SDP) are the structural backbone of Clean Architecture. Jason Gregory's *Game Engine Architecture* prescribes the same layering for Naughty Dog's engines. Cycles mean any change can propagate back to where it started, creating unbounded ripple effects. Lehman's Laws (validated across Apache, Eclipse, and the Linux kernel) confirm that **complexity increases monotonically unless explicit work prevents it** — dependency cycles are the primary mechanism.

**How to evaluate.**
- Run `cargo tree` and visually confirm the dependency DAG. No crate should appear as both a dependency and a reverse-dependency of another.
- Calculate Martin's **Instability metric** per crate: `I = Ce / (Ce + Ca)` where Ce = outgoing dependencies, Ca = incoming. Verify that I *decreases* in the direction of dependency arrows — stable crates (low I) should be depended upon, not depend on volatile ones.
- Use `cargo-modules` or `cargo-depgraph` to generate and inspect the graph. Flag any cycle.
- **Threshold**: Distance from Main Sequence `D = |A + I - 1|` should be below **0.3** for every crate.

**Common violations and warning signs.** Game-specific entity types imported in `engine_render`. A rendering crate that imports audio types. Circular `use` statements between sibling modules. A single monolithic crate containing everything. If your `Cargo.toml` dependency lists look like a tangled web rather than a clean tree, you have a structural problem.

**Rust/game engine specifics.** Cargo workspaces enforce crate boundaries at compile time — a crate literally cannot access another crate's `pub(crate)` items. This is stronger than any linter-based architecture check in other languages. Use traits defined in lower-level crates and implemented in higher-level crates to invert dependencies when needed (e.g., define a `Renderable` trait in `engine_core`, implement it for game types in the `game` crate).

---

## 2. Build deep modules that hide design decisions

**The principle.** Every module should provide powerful functionality behind a simple interface. John Ousterhout calls these "deep modules" — a small interface relative to a large, complex implementation. Each module encapsulates one or more design decisions that do not leak into its interface. If knowledge about an internal format, ordering, or protocol appears in more than one module, that's **information leakage**, the root mechanism of change amplification.

**Why it matters.** Ousterhout's empirical observations across dozens of Stanford student projects show that "shallow modules" (many small classes with little logic) increase total system complexity because each interface carries a cost. The classic deep module is Unix file I/O: five system calls hiding filesystems, disk scheduling, caching, and permissions. Shallow modules force callers to understand implementation details, which multiplies cognitive load across every consumer.

**How to evaluate.**
- **Interface-to-implementation ratio**: For each public module, count public functions/types vs. lines of internal logic. A module with 3 public functions and 500 lines of implementation is deep. A module with 20 public functions and 50 lines of logic is shallow.
- **Pass-through method detection**: Flag methods whose body is a single delegation call with the same or similar parameters. These are strong indicators of shallow abstraction.
- **Grep test for information leakage**: Search for knowledge about internal data formats appearing in multiple files. If two modules both parse the same wire format, one of them is leaking.
- **Change amplification metric**: Track the number of files touched per feature change in version control. If a single logical change routinely touches **>5 files**, you have either information leakage or misaligned module boundaries.

**Common violations.** Wrapper types that add no value beyond indirection. Configuration structs with dozens of fields pushed to callers when reasonable defaults would suffice (Ousterhout: "configuration parameters are a common anti-pattern of pushing complexity upward"). Multiple modules that must be called in a specific sequence not enforced by the API.

**Rust/game engine specifics.** Rust's trait system enables deep modules naturally: a `RenderPipeline` trait with two methods (`configure` and `draw`) can hide thousands of lines of wgpu shader compilation, buffer management, and tessellation via lyon. For ECS, the `bevy_ecs` system parameter API is itself a deep module — users write `Query<(&Position, &Velocity)>` and the framework hides archetype storage, change detection, and parallel scheduling.

---

## 3. Make illegal states unrepresentable through the type system

**The principle.** Design data types so that invalid combinations of values **cannot be constructed**. Use Rust's enums, newtypes, the typestate pattern, and private fields with validated constructors to enforce domain invariants at compile time rather than runtime.

**Why it matters.** This principle, coined by Yaron Minsky and central to Rust's design philosophy, eliminates entire categories of bugs *and* the defensive validation code that tries to catch them. When illegal states are representable, every function handling that data must include validation logic — this duplicates checks across the codebase and creates bugs when a check is missed. Ousterhout's "define errors out of existence" principle is the same idea: redesign interfaces so error conditions cannot arise.

**How to evaluate.**
- **Primitive obsession audit**: Grep for public functions taking multiple parameters of the same primitive type (e.g., `fn spawn(x: f32, y: f32, speed: f32)`). Each should use a newtype: `WorldPosition`, `Speed`.
- **Boolean parameter audit**: Any `bool` parameter in a public API should be an enum. `draw(true, false)` is opaque; `draw(Visibility::Visible, BlendMode::Opaque)` is self-documenting and type-checked.
- **Sum type check**: Are mutually exclusive states modeled as separate enum variants rather than combinations of optional fields? An `AnimationState` should be `enum { Idle, Running(f32), Jumping { height: f32, time: f32 } }`, not a struct with `is_running: bool, is_jumping: bool, jump_height: Option<f32>`.
- **"Parse, don't validate" test**: Is validation done at system boundaries and encoded into types, or repeated by every consumer?
- **Typestate audit**: For resources with lifecycle phases (audio graph: building → playing; render pipeline: configuring → active), can you call phase-inappropriate methods? If yes, the typestate pattern is missing.

**Common violations.** Multiple boolean flags where certain combinations are invalid. Optional fields that are required in certain states. Comments saying "this should never happen" or "this case is impossible." Using `String` where an enum would suffice.

**Rust/game engine specifics.** This is Rust's killer feature for maintainability and uniquely valuable for LLM-assisted development. The compiler becomes an automatic reviewer of LLM-generated code — if an AI produces code that misuses types, `cargo check` catches it in seconds. For a 2D engine: `WorldCoord(Vec2)` vs. `ScreenCoord(Vec2)` prevents coordinate-system bugs; `FrameCount(u64)` vs. `SampleRate(u32)` prevents unit confusion in audio; and typestate on `AudioGraph<Building>` vs. `AudioGraph<Playing>` prevents calling `get_sample()` on an incomplete graph.

---

## 4. Keep ECS components small and systems focused

**The principle.** Components should contain only data (no behavior methods), represent a single concept, and have **≤5 fields**. Systems should be stateless functions that query **≤5-6 distinct component types** and fit within **~100 lines**. New behavior emerges from composing small components on entities, not from enlarging existing components or systems.

**Why it matters.** The Overwatch team at Blizzard (GDC 2017, Timothy Ford) codified strict rules: components have no functions, systems have no state, behavior dependencies are controlled by tick order. This discipline enabled monthly content updates with new heroes and game modes on a live-service game without proportional complexity growth. Conversely, monolithic components waste memory (bevy_ecs uses archetype storage, so a fat component bloats every entity in that archetype), and god systems create the same maintenance problems as god classes — they become change magnets where every feature request requires modifying the same code.

**How to evaluate.**
- **Component field count**: Every `#[derive(Component)]` struct should have ≤5 fields. Components like `Health { current: f32, max: f32 }` and `Armor { defense: f32 }` are good. `CombatStats { health, armor, strength, speed, ... }` is bad.
- **System parameter count**: Count distinct component types in each system's signature. More than 6 is a warning sign. More than 10 means the system is doing too much.
- **System line count**: Systems exceeding 100 lines should be split. Track this with a simple line-count tool.
- **Entity archetype audit**: If entities routinely carry components where most fields are unused, the components should be split.
- **Cyclomatic complexity per system**: Apply the standard threshold of **≤10** per function (NIST Special Publication 500-235, validated over decades).

**Common violations.** A `PlayerSystem` that handles input, animation, physics, and scoring. Components with methods that mutate other components. Systems that know about specific entity "types" rather than component combinations. Excessive use of `Local<T>` state in systems (systems should be stateless; use Resources for shared state).

**Rust/game engine specifics.** bevy_ecs enforces the data-only component pattern naturally — components are Rust structs, and systems are plain functions with typed parameters. Rust's borrow checker prevents two systems from holding `&mut` references to the same component simultaneously, but doesn't prevent *logical* coupling between systems. Use **events** (`EventWriter<T>`, `EventReader<T>`) for inter-system communication rather than one system writing a component that another reads in a specific order.

---

## 5. Isolate subsystems behind plugin boundaries with event-driven communication

**The principle.** Structure the engine as a collection of plugins where each plugin owns its systems, components, events, and resources. Plugins communicate through **events and shared component types defined in a core crate**, never through direct system-to-system coupling. Disabling any single plugin should not break the rest of the application.

**Why it matters.** The "glue code" anti-pattern is the #1 killer of custom game engines. Leafwing Studios observed that GitHub is "littered with failed projects" that tried to glue libraries together. Glue code "breaks with alarming regularity whenever dependencies change, is soul-sucking to write, painful to test, and makes switching dependencies incredibly high." Unreal Engine 5's Modular Game Features system demonstrates the gold standard: "new content can be injected via a plugin architecture — the core game is completely unaware of its existence." This pattern enabled Fortnite's seasonal content cycling.

**How to evaluate.**
- **Plugin isolation test**: For each plugin, can you comment out its registration and still compile and run the rest of the engine? If not, what breaks?
- **Event-driven communication check**: Do systems in different plugins communicate through `Events<T>` rather than directly reading/writing each other's components?
- **Dependency direction**: Plugin dependencies should form a DAG. No circular dependencies between plugins.
- **Configuration exposure**: Plugin behavior should be configurable through Resources or plugin struct fields, not by requiring users to modify plugin internals.
- **Schedule coupling check**: Are plugins tightly bound to specific schedules (Update, FixedUpdate)? Bevy issue #14412 identifies this as a real maintainability problem — plugins should expose system configurations that users can place.

**Common violations.** An FPS camera plugin that directly writes to `Transform`, bypassing physics (real Bevy issue #9732). Audio systems that import rendering types. Plugin initialization order that matters but isn't documented or enforced. A "core" plugin that contains half the codebase.

**Rust/game engine specifics.** For a 2D engine using bevy_ecs + wgpu + lyon + fundsp, define: `RenderPlugin` (wgpu + lyon integration), `AudioPlugin` (fundsp integration), `InputPlugin`, `PhysicsPlugin`, and game-specific plugins. Each plugin's components and events should be defined in the plugin's module or crate. Use Bevy's `run_if(on_event::<T>())` pattern so event-handling systems only execute when relevant events exist, avoiding wasted computation.

---

## 6. Wrap external dependencies behind internal abstractions

**The principle.** Never let external crate types (wgpu types, lyon paths, fundsp nodes) leak into your engine's public API. Define internal types and traits that wrap or abstract over third-party functionality. External dependencies should be directly imported by **at most one crate** in your workspace.

**Why it matters.** Each dependency is a permanent maintenance commitment. Industry data shows **80% of dependencies go un-upgraded for over a year**, and each upgrade takes ~2 hours. Google's engineering book warns that "upstream projects not explicitly prioritizing stability are a risk" for codebases with expected lifespans of years. The XZ Utils backdoor (2024) and Log4Shell (2021) demonstrate that dependency security is a real threat. Wrapping creates an isolation layer so that when wgpu changes its API (which it does regularly), you absorb that change in one wrapper crate rather than across your entire codebase.

**How to evaluate.**
- **Leakage grep**: Search for external crate types in your public API signatures. `pub fn create_mesh(device: &wgpu::Device)` leaks wgpu. `pub fn create_mesh(renderer: &Renderer)` does not.
- **Single-point-of-contact rule**: For each external dependency, how many of your crates directly import it? The answer should be **1** (or at most 2).
- **Replaceability test**: Could you swap wgpu for another graphics backend by changing only the `engine_render` crate? Could you swap fundsp for another audio library by changing only `engine_audio`?
- **`cargo tree` audit**: Run `cargo tree` periodically. Flag transitive dependency count growth. Run `cargo audit` for known vulnerabilities.

**Common violations.** Passing `wgpu::RenderPass` through three layers of your engine. Game logic that directly constructs `lyon::path::Path` objects. Returning `fundsp::AudioUnit` from your public audio API. Test code that depends on the specific behavior of third-party types.

**Rust/game engine specifics.** Rust's orphan rules mean you can't implement foreign traits on foreign types — but you *can* create newtypes (`struct EnginePath(lyon::path::Path)`) and implement your own traits on them. This is idiomatic Rust and provides the wrapping layer naturally. For wgpu specifically, keep all GPU interaction in `engine_render` and expose a high-level `Renderer` trait. For fundsp, wrap the audio graph behind an `AudioEngine` abstraction that game code interacts with.

---

## 7. Maintain function complexity below empirically validated thresholds

**The principle.** Keep cyclomatic complexity **≤10** per function (McCabe/NIST threshold, validated over decades). Keep cognitive complexity **≤15** per function (SonarSource, validated by a University of Stuttgart meta-analysis of ~24,000 human comprehensibility evaluations). Every function above these thresholds is a refactoring target.

**Why it matters.** McCabe's cyclomatic complexity score directly equals the **minimum number of test cases needed for branch coverage**. A function with CC=25 needs at least 25 tests for full coverage — if you're doing TDD, this means something went wrong structurally. The University of Stuttgart study confirmed that high cognitive complexity correlates with longer comprehension time and worse developer ratings. For LLM-assisted development, high-complexity functions are precisely the ones where AI assistants produce the most errors — they exceed the model's effective reasoning window.

**How to evaluate.**
- Use `cargo-clippy` with `cognitive_complexity` lint (configurable threshold).
- Use `rust-code-analysis` crate for cyclomatic complexity metrics on Rust code.
- **File length**: Keep files under **300 lines**. This ensures each file fits within an LLM's effective context window and represents a single coherent concept.
- **Function length**: Soft limit of **30 lines** per function (McConnell, Martin). Hard flag at 50.
- Track complexity metrics per release. Lehman's Laws predict monotonic increase unless you actively intervene.

**Common violations.** Match statements with 15+ arms that each contain logic (extract to separate functions). Deeply nested `if`/`match` blocks. Systems that handle multiple game states in a single function. Event handlers with long chains of conditional logic.

**Rust/game engine specifics.** Rust's pattern matching and `?` operator naturally reduce cyclomatic complexity compared to languages with exceptions. However, complex system functions in ECS code can still grow unwieldy. Split by extracting helper functions, using `SystemParam` derives for complex parameter bundles, and breaking large systems into smaller ones connected by events. For a TDD workflow, if writing tests for a function feels painful, the function is too complex.

---

## 8. Design per-crate error types with `thiserror` at boundaries

**The principle.** Each workspace crate defines its own error enum using `thiserror`. Error messages are lowercase, without trailing punctuation, and describe only themselves (not their source). Use `#[source]`/`#[from]` to preserve error chains. Application-level orchestration code (main loop, top-level systems) uses `anyhow::Result` for ergonomic propagation.

**Why it matters.** Ousterhout identifies exception handling as "one of the worst sources of complexity in software systems." A clean error type per crate documents exactly what can go wrong at each boundary, enables callers to handle specific failures programmatically, and prevents error type leakage across architectural layers. Luca Palmieri's error handling guide and the RisingWave project's error conventions confirm this as Rust community best practice.

**How to evaluate.**
- **Error type inventory**: Each crate should have exactly one primary error enum (or a small family). If a crate has no custom error type, it's probably using `anyhow` where it shouldn't, or worse, panicking.
- **Error variant audit**: If an error enum has >15 variants and callers always handle them identically, the variants are over-engineered. Group related errors.
- **Panic audit**: Search for `unwrap()`, `expect()`, and `panic!()` outside of tests. Each use in production code should be justified with a comment explaining why the condition is truly unreachable.
- **Error chain test**: Trigger an error at the lowest level and verify the full chain prints cleanly: "failed to render frame: shader compilation failed: syntax error at line 42."

**Common violations.** Using `String` as the error type. Panicking on recoverable errors. Duplicating source error text in wrapper error messages. A single `EngineError` enum shared across all crates (leaks internal boundaries). Returning `anyhow::Error` from library crate public APIs.

**Rust/game engine specifics.** For a game engine: `RenderError` (shader failures, device lost, texture not found), `AudioError` (device unavailable, graph construction failure), `AssetError` (file not found, parse failure). The game loop catches these at the top level with `anyhow`. This pattern means that when fundsp changes its error types in a new version, only `engine_audio`'s error mapping changes — downstream code is insulated.

---

## 9. Maintain living architecture documentation and AI context files

**The principle.** Keep an `ARCHITECTURE.md` that describes the system's crate structure, module responsibilities, dependency rules, and key invariants. Maintain a `CLAUDE.md` (or equivalent AI rules file) that encodes project conventions: naming patterns, preferred crates, forbidden patterns, module boundaries, and the newtype/component vocabulary. Update both whenever the architecture changes.

**Why it matters.** Li et al.'s systematic study of architecture erosion identified **knowledge vaporization** (from turnover + poor documentation) and **disconnect between architects and developers** as two of the 12 primary causes of architectural decay. For LLM-assisted development, Addy Osmani's workflow research emphasizes that "planning first forces you and the AI onto the same page" — a spec prevents the LLM from making architectural decisions that conflict with the project's design. LLMs are literalists: explicitly encoding "always use `WorldCoord` not raw `Vec2`" or "error types use `thiserror`" prevents stylistic drift across AI-generated code.

**How to evaluate.**
- **Existence check**: Does `ARCHITECTURE.md` exist? Does it describe every crate and their dependency relationships?
- **Freshness check**: Does the documented architecture match the actual `Cargo.toml` dependency graph? Run `cargo-depgraph` and compare.
- **AI rules check**: Does a rules file exist? Does it cover: naming conventions, component design rules, error handling patterns, forbidden patterns, and test expectations?
- **Onboarding test**: Can a new developer (or a fresh LLM session) understand the project's architecture from documentation alone, without reading source code?

**Common violations.** No architecture documentation at all (the most common case). Documentation that was written once and never updated. AI rules files that are vague ("write clean code") rather than specific ("all public functions in engine crates use newtype parameters, never bare primitives"). Architecture diagrams that don't match reality.

**Rust/game engine specifics.** For LLM-assisted TDD development, the AI rules file is arguably *more* important than traditional documentation. Include: the ECS component vocabulary (what components exist and what they mean), system naming conventions, the plugin structure, which crate owns which responsibility, and the test patterns to follow (create minimal `App`, spawn entities, add system, call `update()`, assert). This file is the single highest-leverage artifact for maintaining consistency across hundreds of AI-assisted coding sessions.

---

## 10. Test behavior through ECS queries and events, not implementation details

**The principle.** Every ECS system should be testable by creating a minimal `App`, spawning entities with components, adding the system under test, calling `app.update()`, and asserting on component state or emitted events. Tests should never require initializing a window, GPU, or audio device. Use property-based testing (via `proptest`) for mathematical operations — coordinate transforms, geometry, audio DSP.

**Why it matters.** Inozemtseva and Holmes (ICSE 2014) showed that test suite **size matters more than coverage percentage** for fault detection — but the key insight is that test *design quality* matters most of all. Tests that assert on behavior (observable state changes) survive refactoring; tests that assert on implementation details break with every internal change and create a maintenance burden that discourages refactoring. Capers Jones's data shows no single detection technique exceeds 75% effectiveness — you need multiple techniques (types + tests + review). Property-based testing is especially valuable for game engines because it finds edge cases in mathematical operations that humans systematically miss.

**How to evaluate.**
- **GPU/window independence**: Can your full test suite run in CI without a GPU? If not, separate integration tests from unit tests.
- **Test isolation**: Does each test create its own `App` with only the components and systems needed? Tests sharing mutable state between runs is a major red flag.
- **Property test coverage**: Do coordinate transforms, geometry operations, and audio processing have `proptest` roundtrip tests?
- **Behavior focus**: Do tests assert on component values after `app.update()`, or do they reach into system internals?
- **Test-to-system ratio**: Every system should have at least one test. Track this metric.

**Common violations.** Tests that call system functions directly with hand-constructed parameters rather than going through the ECS scheduler. Integration tests that require a live GPU. Tests that break when internal helper functions are renamed or reorganized. No property-based tests for math-heavy code. Test files exceeding 500 lines with duplicated setup logic (extract test builders).

**Rust/game engine specifics.** bevy_ecs makes this pattern natural — systems are plain functions, so testing them through a minimal `App` is straightforward. The combination of Rust's type system (catching type errors), `cargo test` (catching logic errors), `proptest` (catching edge cases), and `cargo clippy` (catching style issues) creates a layered defect detection system that approaches the "all-four-categories combined" **99% detection rate** from Capers Jones's research. This is the foundational argument for Rust + TDD in game engine development.

---

## Conclusion: the meta-principle is architectural discipline

The thread connecting all 10 principles is that **architecture degrades by default and must be actively maintained**. Lehman's Laws confirm complexity increases monotonically. Ernst et al. confirm architecture is the #1 debt category. The InsighTD survey confirms time pressure is the #1 cause. These are not opinions — they are empirically validated findings.

For a Rust 2D engine with bevy_ecs, you have structural advantages that most projects lack: Cargo workspaces enforce crate boundaries at compile time, the borrow checker prevents data races, the type system can make illegal states unrepresentable, and ECS naturally separates data from logic. But these advantages only hold if you actively maintain the architecture: enforce the dependency DAG, keep components small, wrap dependencies, and document invariants.

The most leveraged investment is **principle #1 (dependency structure) combined with principle #9 (architecture documentation)**. Get the crate boundaries right, write them down, encode them in AI rules files, and every subsequent decision becomes easier — because the compiler itself enforces your architectural intent.