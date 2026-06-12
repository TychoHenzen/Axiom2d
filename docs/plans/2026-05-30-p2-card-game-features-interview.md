# Card Game Features (Priority 2) — Requirements Spec (Stub)

> **For Claude (/goal):** Work through each incomplete step below.
> 1. Mark a step `[>]` when you begin working on it.
> 2. Verify each proof by running the stated command/process and confirming the expected outcome.
> 3. Mark each proof `[x]` only when the claim has been tested and matches the expected value.
> 4. A step may only be marked `[x]` once ALL its proofs are `[x]` or `[~]`.
> 5. If a proof cannot be met because requirements changed or the original condition is unreasonable:
>    - Mark it `[~]` with the original condition struck through.
>    - Add a bullet underneath: `  - Met instead: [what was actually achieved]`
>    - The step can still be `[x]` once all proofs are resolved (either `[x]` or `[~]`).
> 6. Continue until every step is `[x]` — then stop and report done.
>
> **Self-contained.** No external context needed. Run the commands listed in proofs directly.
>
> **Stub spec.** Requirements are derived from backlog one-liners. Run `/interview` on individual items before implementing to fill in behavioral details, edge cases, and error handling.

## Context

**Backlog IDs:** I11, I14, I15, I16, I17, I18, I22, I23, I24

**Goal:** Implement remaining card game features needed for playable state.

**Test convention:** Tests live in `crates/card_game/tests/suite/` with `when_action_then_outcome` naming. Systems must be wired into `crates/card_game_bin/src/main.rs`.

---

## Steps

### Step 1: Game session state machine (I11)

- [ ] Implement a state machine governing game session flow (e.g., setup -> playing -> paused -> game over). Define states as an enum, transitions as a system.

**Proofs:**
- [ ] `rtk cargo build -p card_game` exits 0
- [ ] `rtk grep "pub enum GameState\|pub enum SessionState" crates/card_game/src/` shows state enum exists
- [ ] `rtk cargo test -p card_game -- session\|game_state` exits 0, at least 3 tests run
- [ ] `rtk grep "session\|game_state" crates/card_game_bin/src/main.rs` shows system registered

---

### Step 2: Signature-only serialization (I14)

- [ ] Serialize card identity using only the signature (seed-based). Full card data is regenerated from the signature on load. Keeps save files compact.

**Proofs:**
- [ ] `rtk cargo build -p card_game` exits 0
- [ ] `rtk cargo test -p card_game -- serialize\|signature.*serial\|serial.*signature` exits 0, at least 2 tests run
- [ ] Roundtrip test exists: serialize a card's signature, deserialize, regenerate card, verify identical to original

---

### Step 3: Card physics sleep behavior (I15)

- [ ] Enforce that cards at rest enter physics sleep state. Cards on the table that haven't been interacted with should not consume physics simulation resources.

**Proofs:**
- [ ] `rtk cargo build -p card_game` exits 0
- [ ] `rtk cargo test -p card_game -- sleep\|physics_sleep` exits 0, at least 2 tests run
- [ ] `rtk grep "sleep\|Sleeping" crates/card_game/src/` shows sleep-related logic exists

---

### Step 4: Drop preview indicators (I16)

- [ ] Show visual indicators for valid landing targets when dragging a card. Highlight valid drop zones (hand, stash, table, reader) based on the card being dragged.

**Proofs:**
- [ ] `rtk cargo build -p card_game` exits 0
- [ ] `rtk cargo test -p card_game -- drop_preview\|landing.*indicator` exits 0, at least 2 tests run
- [ ] `rtk grep "DropPreview\|LandingIndicator" crates/card_game/src/` shows component/system exists

---

### Step 5: Card highlight system (I17)

- [ ] Visual highlight effect on cards (e.g., on hover, on selection, on valid drop target). Should use existing shape/material system.

**Proofs:**
- [ ] `rtk cargo build -p card_game` exits 0
- [ ] `rtk cargo test -p card_game -- highlight` exits 0, at least 2 tests run
- [ ] `rtk grep "Highlight\|CardHighlight" crates/card_game/src/` shows component exists

---

### Step 6: Batched card spawning (I18)

- [ ] Spawn multiple cards in a single frame without frame hitches. Spread tessellation/baking work across multiple frames if needed.

**Proofs:**
- [ ] `rtk cargo build -p card_game` exits 0
- [ ] `rtk cargo test -p card_game -- batch.*spawn\|spawn.*batch` exits 0, at least 2 tests run
- [ ] `rtk grep "batch\|BatchSpawn\|SpawnQueue" crates/card_game/src/` shows batching mechanism exists

---

### Step 7: Auto-save (I22)

- [ ] Automatically save game state at regular intervals or on significant events. Uses signature-only serialization (Step 2 dependency).

**Proofs:**
- [ ] `rtk cargo build -p card_game` exits 0
- [ ] `rtk cargo test -p card_game -- auto_save\|autosave` exits 0, at least 2 tests run
- [ ] `rtk grep "AutoSave\|auto_save" crates/card_game/src/` shows system exists
- [ ] `rtk grep "auto_save\|autosave" crates/card_game_bin/src/main.rs` shows system registered

---

### Step 8: Generation progress UI (I23)

- [ ] Display progress indicator during card generation (when many cards are being generated/baked). Uses `engine_ui` widget system.

**Proofs:**
- [ ] `rtk cargo build -p card_game` exits 0
- [ ] `rtk cargo test -p card_game -- generation.*progress\|progress.*generation` exits 0, at least 1 test runs
- [ ] `rtk grep "GenerationProgress\|ProgressBar" crates/card_game/src/` shows component/widget exists

---

### Step 9: Pause system support (I24)

- [ ] Implement pause functionality that freezes game simulation but allows UI interaction. Integrate with game session state machine (Step 1).

**Proofs:**
- [ ] `rtk cargo build -p card_game` exits 0
- [ ] `rtk cargo test -p card_game -- pause` exits 0, at least 2 tests run
- [ ] `rtk grep "Paused\|PauseState\|pause_system" crates/card_game/src/` shows pause mechanism exists
- [ ] `rtk grep "pause" crates/card_game_bin/src/main.rs` shows system registered

---

## Dependencies

```
Step 2 (serialization) ← Step 7 (auto-save)
Step 1 (state machine) ← Step 9 (pause)
Step 6 (batch spawning) ← Step 8 (progress UI)
```

## Open Questions

> These MUST be resolved via `/interview` before implementation begins.

- **I11 (State Machine):** What are the exact states and valid transitions? How does the state machine interact with the ECS schedule (run conditions, phase gating)?
- **I14 (Serialization):** What serialization format (RON, bincode, JSON)? Where are save files stored? What version/migration strategy?
- **I15 (Sleep):** What thresholds determine "at rest"? Should sleep be automatic via Rapier config or manually managed?
- **I16 (Drop Preview):** What visual style for indicators (outline, glow, color tint)? How are valid zones determined per card type?
- **I17 (Highlight):** What highlight styles are needed (hover vs. selected vs. valid target)? Additive color, outline, scale pulse?
- **I18 (Batch Spawning):** What batch size per frame? Fixed count or time-budget-based? Priority ordering for which cards spawn first?
- **I22 (Auto-save):** Save interval? What events trigger immediate save? How many save slots? Overwrite or rotate?
- **I23 (Progress UI):** Where is the progress bar positioned? What information is shown (count, percentage, ETA)?
- **I24 (Pause):** Does pause affect physics only, or also animations? Can cards still be inspected (hover/zoom) while paused?
