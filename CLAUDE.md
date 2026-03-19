# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Axiom2d is an LLM-optimized 2D game engine written in Rust. The engine scaffolding is complete (12 engine crates + `axiom2d` facade + `demo`, 1100+ tests). A card game is being built on top (`card_game` + `card_game_bin` crates). The full architectural vision is documented in `Doc/Axiom_Blueprint.md`. The engine implementation roadmap lives in `Doc/Implementation_Roadmap.md`. Known technical debt is tracked in `Doc/Technical_Debt_Audit.md`.

## Current Focus: Card Game

We are now building the first real game on the engine — a card game with physics-based card manipulation. The implementation plan lives in `Doc/Card_Game_Roadmap.md`.

**Before starting work**, read `Doc/Card_Game_Roadmap.md` to understand what's been done, what's next, and the dependency order between phases.

### Working on a step

1. Pick the next `[NOT STARTED]` step that has all dependencies satisfied (check the dependency graph at the bottom of the roadmap).
2. Mark the step `[IN PROGRESS]` in `Doc/Card_Game_Roadmap.md` before writing code.
3. Implement with tests (follow the Testing Strategy below). Each checkbox in the step is a deliverable — check it off as you complete it.
4. Run `cargo.exe test` and `cargo.exe build` to verify everything passes.
5. Mark the step `[DONE]` when all checkboxes are checked and tests pass.

### After completing a step

1. Update `Doc/Card_Game_Roadmap.md`: mark `[DONE]`, check all boxes.
2. **Wire new systems into `card_game_bin/src/main.rs`**. Every new ECS system must be registered in the `setup()` function with the correct `Phase` and ordering constraints (`.after()`, `.chain()`). A system that only exists in the library crate with tests but is never added to a schedule in the binary **does not exist in the game**. Always verify with `cargo.exe build -p card_game_bin`.
3. Update the memory file if new public types/traits/systems were added.
4. Run `cargo.exe fmt --all`.
5. If new workspace dependencies were added, mention them in the Development Environment section.

### Before committing

Run `cargo.exe clean` before committing. Incremental compilation artifacts bloat `target/` (~100MB per build) and accumulate across debug/flycheck directories.

### Engine changes

Card game work may require engine extensions (Phase A in the roadmap). When modifying engine crates (e.g. `engine_physics`), also update `Doc/Implementation_Roadmap.md` if relevant and keep the engine's test suite passing.

## Development Environment

This project is developed with **RustRover on Windows** while **Claude Code runs in WSL2 (Ubuntu 24.04)**. The repo lives on the Windows filesystem at `/mnt/c/Users/t.henzen/RustroverProjects/Axiom2d`.

**Rust is installed on the Windows side only** (stable-x86_64-pc-windows-msvc). There is no WSL-native Rust toolchain. To run cargo commands from WSL:

```bash
cargo.exe build              # Build the library
cargo.exe test               # Run all tests
cargo.exe test <test_name>   # Run a single test by name
cargo.exe clippy             # Lint (pedantic, workspace-configured)
cargo.exe fmt --all          # Format all crates
```

Always use `cargo.exe` (not `cargo`) since the Rust toolchain is Windows-only. The same applies to `rustc.exe`, `rustfmt.exe`, etc.

The project uses Rust edition 2024. Dependencies are declared at the workspace level: `glam` (math), `thiserror` (errors), `winit` (windowing), `wgpu` (GPU rendering), `pollster` (async blocking), `bytemuck` (safe type casting for GPU buffers), `bevy_ecs` (standalone ECS), `guillotiere` (2D texture atlas rect packing), `image` (PNG/JPEG decoding for embedded assets), `lyon` (2D vector path tessellation), `fundsp` (audio DSP graph synthesis), `rapier2d` (2D physics engine), `proptest` (property-based testing, dev-dep across 7 crates), `insta` (snapshot testing with RON feature, dev-dep across 3 crates), `image-compare` (SSIM visual regression comparison, optional dep in engine_render behind `testing` feature + dev-dep), `ttf-parser` (TTF font outline parsing for vector text rendering).

### WSL/Windows Gotchas

- **Use `cargo.exe` not `cargo`**: The Rust toolchain lives at `/mnt/c/Users/t.henzen/.cargo/bin/` and is a Windows installation. WSL has no native Rust.
- **Filesystem performance**: The project is on a `/mnt/c` drvfs mount. File I/O from WSL is slower than native. Build artifacts in `target/` are written through this mount.
- **Line endings**: Git and editors should be configured for consistent line endings. RustRover on Windows may default to CRLF; ensure `.gitattributes` or git config handles this.
- **Path formats**: WSL sees `/mnt/c/...` paths; Windows/RustRover sees `C:\...` paths. Cargo and rustc invoked via `.exe` will interpret paths as Windows paths. When passing paths to `cargo.exe`, use Windows-style paths or rely on the working directory.
- **File locking**: Both RustRover (Windows) and Claude Code (WSL) access the same files. Avoid concurrent builds — RustRover's build and `cargo.exe` from WSL share the same `target/` directory and lock file.
- **Target triple**: The Windows toolchain compiles for `x86_64-pc-windows-msvc` by default. Any platform-specific code or dependencies should target Windows, not Linux.
- **Snapshot testing (insta)**: `INSTA_UPDATE=always` does not propagate from WSL env vars to Windows `cargo.exe`. To accept new snapshots, rename `.snap.new` → `.snap` manually (e.g., `for f in $(find crates -name "*.snap.new"); do mv "$f" "${f%.new}"; done`).

### Clippy Configuration

Workspace-level clippy lints are configured in the root `Cargo.toml` under `[workspace.lints.clippy]`. All crates inherit via `[lints] workspace = true`. The `pedantic` group is enabled as warnings with selective allows for noisy lints (cast lints, `module_name_repetitions`, `must_use_candidate`, etc.). `unwrap_used` is promoted to warn — use `.expect("reason")` in production code. Test modules use `#[allow(clippy::unwrap_used)]` at the module level.

### CI Workflows

Two GitHub Actions workflows in `.github/workflows/`:
- **`ci.yml`** (every push/PR to master): autofix (clippy --fix + fmt, pushes fixes back) → build + test — fast gate for every commit.
- **`quality.yml`** (daily at 06:00 UTC + manual `workflow_dispatch`): clippy, audit, docs, coverage, mutants — expensive checks run on a schedule to conserve Actions minutes.

## Architecture

The engine follows a **Bevy-inspired archetypal ECS** pattern optimized for LLM code generation. Key design principles:

- **Archetypal ECS**: Entities with identical component sets stored together. Systems are plain functions with typed parameters (e.g., `Query<(&mut Position, &Velocity)>`). Uses `bevy_ecs` as a standalone crate, wrapped by `engine_ecs`.
- **Code-defined assets**: All assets (sprites, audio, shaders, tilemaps) are expressed as Rust code or RON data — no binary asset files. Uses `lyon` for vector graphics, `fundsp` for audio synthesis, WGSL for shaders.
- **Trait-abstracted hardware**: Every hardware-dependent subsystem (renderer, audio, input) hides behind a trait with null/mock implementations for testing. Canonical test pattern: `World` + `Schedule` without touching hardware.
- **Flat workspace of crates**: Layout under `crates/` — `engine_core`, `engine_render`, `engine_app`, `engine_ecs`, `engine_input`, `engine_scene`, `engine_audio`, `engine_physics`, `engine_assets`, `engine_ui`, `axiom2d` (facade + DefaultPlugins), `card_game` (game logic library), `card_game_bin` (binary entry point), and `demo` (solar system smoke test). Virtual manifest at root.

### Scheduling Phases

`Input → PreUpdate → Update → PostUpdate → Render` — implemented as `engine_ecs::schedule::Phase` enum with `ScheduleLabel` derive.

### Render Pipeline

`Clear → Atlas Upload → Camera Prepare → Splash Camera → Sprite Pass → Shape Pass → Post-Process → Present`

## API Design Conventions

- Components use `#[derive(Component)]` — no manual registration
- Configs use plain structs with `Default` — LLMs handle `..Default::default()` reliably
- NewType wrappers for type safety (e.g., `Pixels(f32)`, `Seconds(f32)`, `TextureId(u32)`)
- Enums over magic numbers for constrained states (e.g., `BlendMode`, `RenderLayer`)
- Limit trait indirection to 2 levels max
- Each crate exports a `prelude` module

## Testing Strategy

- Test naming: `when_action_then_outcome` hybrid style (skip `given_` when precondition is obvious)
- Test body structure: `// Arrange` / `// Act` / `// Assert` section markers
- Inline `#[cfg(test)] mod tests` in each source file — no separate test files
- Deterministic game loop: fixed timestep, injectable mock time, seeded RNG (`ChaCha8Rng`)
- Use `BTreeMap` or fixed-seed `ahash` instead of `HashMap` where iteration order matters
- Property-based testing with `proptest` for physics invariants, serialization roundtrips, ECS invariants
- Visual regression via wgpu headless mode with SSIM comparison (0.99 threshold)
- Snapshot testing with `insta`
- Target: `cargo-nextest` as test runner for parallel execution

### What NOT to test

Only test **behavior you wrote**, not language or framework guarantees. The following test categories are banned:

- **Prelude/re-export tests**: Don't test that `use crate::prelude::*` makes types available. Real usage catches missing re-exports at compile time.
- **Derive tests**: Don't test that `Clone`, `Copy`, `PartialEq`, `Debug`, `Hash` derives work. Rust's derive macros are not broken.
- **Struct construction tests**: Don't test that `Foo { x: 1 }.x == 1`. Rust structs store their fields.
- **Resource insertion tests**: Don't test that `world.insert_resource(X); world.get_resource::<X>().is_some()`. The `#[derive(Resource)]` macro from bevy_ecs works.
- **Component spawn tests**: Don't test that `world.spawn(C)` makes `C` queryable. Same reasoning as Resource insertion.
- **Trivial default tests**: Don't test that `Default` returns the value written in `impl Default`. If the default matters for correctness, test it through the system that depends on it.
- **Boxing/trait-object tests**: Don't test that `Box::new(X) as Box<dyn Trait>` compiles. Compilation is the test.

**Do test**: Custom logic (arithmetic operators, conversion functions like `from_u8`), system behavior (clear_system, time_system, input_system), non-trivial algorithms (FixedTimestep.tick accumulator math), and design constraints (no-clamping on Rect values).
