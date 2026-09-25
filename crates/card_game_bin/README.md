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

Run the stash round-trip scenario, storing the seeded card, switching pages, previewing the stash slot during a drag, and retrieving it:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\crates\card_game_bin\tests\ui-smoke.ps1 -StashRoundTrip
```

This mode retains stash, page, retrieval, and rendered-difference evidence under `target/ui-smoke-*`.
