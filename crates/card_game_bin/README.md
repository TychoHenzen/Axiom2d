# Card game binary

Run the native UI drag smoke from an interactive Windows desktop session:

```powershell
pwsh -NoProfile -File .\crates\card_game_bin\tests\ui-smoke.ps1
```

The script builds the test-only observer, launches the real game, sends native Windows pointer events to its Winit window, and checks the seeded face-down card's drag state and rendered position. It requires an interactive Windows desktop. If Windows refuses to foreground the game, the script fails before moving the cursor or sending a click. Successful runs capture the client frame and state under `target/ui-smoke-*`, then restore the previous focus and cursor position; failures retain logs and available state/frame evidence. Normal builds do not enable the observer.
