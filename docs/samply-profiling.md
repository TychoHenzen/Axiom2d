# Samply Profiling

Use Samply as the primary profiling workflow for `card_game_bin`.

The old CSV frame profiler is still available in engine code for ad hoc instrumentation, but the game binary no longer enables it by default. For actual slowdown investigation, prefer a sampling profiler that can show full call stacks and time spent outside the ECS phase buckets.

## Why this workflow

- `perf_log.csv` only captured schedule execution plus a few manual spans.
- It did not measure `renderer.present()` or other uninstrumented work.
- Samply records end-to-end CPU samples for the real Windows process and opens the profile in the Firefox Profiler web UI.

## Prerequisites

- Install `samply` on Windows and make sure `samply.exe` is on `PATH`.
- Build and run the game with the repo-local Cargo `profiling` profile instead of the size-tuned `release` profile.

## Recommended command

From PowerShell in the repo root:

```powershell
.\scripts\profile-card-game.ps1
```

This script:

1. Builds `card_game_bin` with `cargo.exe build --profile profiling -p card_game_bin`
2. Runs `samply record` against `target\profiling\card_game_bin.exe`
3. Enables the Microsoft symbol server so Windows stacks resolve cleanly

Any extra arguments passed to the script are forwarded to `card_game_bin.exe`.

## Running from WSL

From WSL, invoke the Windows PowerShell script:

```bash
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$(wslpath -w "$PWD/scripts/profile-card-game.ps1")"
```

## Brave

Samply opens the Firefox Profiler UI in your default browser when the capture is ready. If Brave is your default browser, it should open there directly.

The UI itself is a web app, so Brave should work fine in practice. This is an inference from Firefox Profiler's browser-based design rather than an explicit Brave-specific guarantee from Samply.
