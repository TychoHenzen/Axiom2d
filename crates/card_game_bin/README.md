# Card game binary

Run the native UI drag smoke from an interactive Windows desktop session:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\crates\card_game_bin\tests\ui-smoke.ps1
```

The script builds the test-only observer, launches the real game, sends HWND-targeted native Windows `PostMessageW` pointer messages to its Winit window with `SWP_NOACTIVATE`, and checks the seeded face-down card's drag state and rendered position. It requires an interactive Windows desktop but stays background-only: it never activates the game or moves the desktop cursor. Successful runs capture the client frame and state under `target/ui-smoke-*`; failures retain logs and available state/frame evidence. Normal builds do not enable the observer.

Run the full card interaction scenario, including off-center spin/glide, release, and right-click flip:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\crates\card_game_bin\tests\ui-smoke.ps1 -Interaction
```

This mode retains `spin-state.txt`, `interaction-state.txt`, `released-state.txt`, `flipped-state.txt`, and the corresponding frame/visual-diff artifacts.

Run the hand round-trip scenario, moving the seeded card into the hand and back to the table:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\crates\card_game_bin\tests\ui-smoke.ps1 -HandRoundTrip
```

This mode retains hand and returned state/frame evidence and verifies the hand membership, layout, and table return.

Run the holder/zone transition scenario, which adds explicit `CardZone` and `ZoneConfig` assertions to the same native path:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\crates\card_game_bin\tests\ui-smoke.ps1 -ZoneTransition
```

This mode retains holder/table state and frame evidence, including hand occupancy, per-zone physics/render-layer/item-form configuration, and the stable table return.

Run the reader insertion and ejection scenario through the real card-game window:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\crates\card_game_bin\tests\ui-smoke.ps1 -ReaderRoundTrip
```

This mode verifies reader occupancy, signature-space propagation, lit/dim reader feedback, ejection, and the returned table card with retained state/frame evidence under `target/ui-smoke-*`.

Run the stash round-trip scenario, storing the seeded card, switching pages, previewing the stash slot during a drag, and retrieving it:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\crates\card_game_bin\tests\ui-smoke.ps1 -StashRoundTrip
```

This mode retains stash, page, retrieval, and rendered-difference evidence under `target/ui-smoke-*`.

Run the seeded booster opening scenario through the real card-game window:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\crates\card_game_bin\tests\ui-smoke.ps1 -BoosterOpening
```

This mode retains sealed, opening, opened-card state, identity, and rendered-difference evidence under `target/ui-smoke-*`.

Run the seeded identity/signature scenario through the real card-game window:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\crates\card_game_bin\tests\ui-smoke.ps1 -IdentitySignature
```

This mode compares the deterministic card signature, seed, rarity, tier, and generated name, then retains `identity-state.txt`, `identity.bmp`, and `identity-visual.txt` under `target/ui-smoke-*`. Signature axes are normalized to six decimal places before comparison; seed, rarity, tier, and name remain exact. The visual assertion compares `identity.bmp` with a template captured independently from `baseline.bmp`, not with a template derived from the identity frame itself.

Run the seeded card-art hydration and face-rendering scenario through the real card-game window:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\crates\card_game_bin\tests\ui-smoke.ps1 -ArtFace
```

This mode verifies the deterministic selected art entry and hydrated shape count, then retains `art-face-state.txt`, `art-face.bmp`, and `art-face-visual.txt` with the focused card-art region evidence under `target/ui-smoke-*`.
