# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Axiom2d is an LLM-optimized 2D game engine written in Rust. The project is in early development (scaffolding stage). The full architectural vision is documented in `Doc/Axiom_Blueprint.md`.

## Development Environment

This project is developed with **RustRover on Windows** while **Claude Code runs in WSL2 (Ubuntu 24.04)**. The repo lives on the Windows filesystem at `/mnt/c/Users/t.henzen/RustroverProjects/Axiom2d`.

**Rust is installed on the Windows side only** (stable-x86_64-pc-windows-msvc). There is no WSL-native Rust toolchain. To run cargo commands from WSL:

```bash
cargo.exe build              # Build the library
cargo.exe test               # Run all tests
cargo.exe test <test_name>   # Run a single test by name
cargo.exe clippy             # Lint (when configured)
```

Always use `cargo.exe` (not `cargo`) since the Rust toolchain is Windows-only. The same applies to `rustc.exe`, `rustfmt.exe`, etc.

The project uses Rust edition 2024. Dependencies are declared at the workspace level: `glam` (math), `thiserror` (errors).

### WSL/Windows Gotchas

- **Use `cargo.exe` not `cargo`**: The Rust toolchain lives at `/mnt/c/Users/t.henzen/.cargo/bin/` and is a Windows installation. WSL has no native Rust.
- **Filesystem performance**: The project is on a `/mnt/c` drvfs mount. File I/O from WSL is slower than native. Build artifacts in `target/` are written through this mount.
- **Line endings**: Git and editors should be configured for consistent line endings. RustRover on Windows may default to CRLF; ensure `.gitattributes` or git config handles this.
- **Path formats**: WSL sees `/mnt/c/...` paths; Windows/RustRover sees `C:\...` paths. Cargo and rustc invoked via `.exe` will interpret paths as Windows paths. When passing paths to `cargo.exe`, use Windows-style paths or rely on the working directory.
- **File locking**: Both RustRover (Windows) and Claude Code (WSL) access the same files. Avoid concurrent builds — RustRover's build and `cargo.exe` from WSL share the same `target/` directory and lock file.
- **Target triple**: The Windows toolchain compiles for `x86_64-pc-windows-msvc` by default. Any platform-specific code or dependencies should target Windows, not Linux.

## Architecture (Planned)

The engine follows a **Bevy-inspired archetypal ECS** pattern optimized for LLM code generation. Key design principles:

- **Archetypal ECS**: Entities with identical component sets stored together. Systems are plain functions with typed parameters (e.g., `Query<(&mut Position, &Velocity)>`). The plan is to use `bevy_ecs` as a standalone crate or wrap `hecs`.
- **Code-defined assets**: All assets (sprites, audio, shaders, tilemaps) are expressed as Rust code or RON data — no binary asset files. Uses `lyon` for vector graphics, `fundsp` for audio synthesis, WGSL for shaders.
- **Trait-abstracted hardware**: Every hardware-dependent subsystem (renderer, audio, input) hides behind a trait with null/mock implementations for testing. Canonical test pattern: `World` + `Schedule` without touching hardware.
- **Flat workspace of crates**: Planned layout under `crates/` (e.g., `engine_core`, `engine_ecs`, `engine_render`, `engine_audio`, `engine_input`, `engine_physics`, `engine_scene`, `engine_assets`, `engine_ui`) with a virtual manifest at root.

### Scheduling Phases

`Input → PreUpdate → Update → PostUpdate → Render`

### Render Pipeline

`Clear → Sprite Pass → Particle Pass → Post-Process → UI Pass → Present`

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
