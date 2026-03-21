---
name: mine-ideas
description: Use when brainstorming new features or systems for Axiom2d's card game by analyzing the CardCleaner reference project. Triggers include wanting new roadmap ideas, exploring what CardCleaner does, or asking what to build next.
---

# Mine Ideas from CardCleaner

Extract game design ideas from the CardCleaner reference project and translate them into an actionable checklist roadmap for Axiom2d.

## Source Project

**Repo:** https://github.com/TychoHenzen/CardCleaner
**Stack:** Godot 4.4 + C# (.NET 8.0), 3D card game
**Browse with:** `gh api repos/TychoHenzen/CardCleaner/contents/{path} --jq .content | base64 -d`

### CardCleaner's Key Systems

| System | Location | What It Does |
|--------|----------|-------------|
| Card Signatures | `Scripts/Features/Card/Models/CardSignature.cs` | 8D elemental vector [-1,1] per axis. Operations: DistanceTo, Subtract (residual energy), GetDominantAspect, GetIntensity. Each Element has positive/negative Aspects (16 total) |
| Base Card Types | `Scripts/Features/Card/Models/BaseCardType.cs` | Template types (e.g. Weapon) with BaseSignature + MatchRadius. Cards match by inverse-distance weighting in 8D space. Categories: Playstyle, Equipment, Skill |
| Residual Energy | `Scripts/Features/Card/Services/ResidualEnergyModifier.cs` | signature - base_signature = residual. Each element axis maps to gameplay stats (Power, Cost, Duration, Range, Healing, Speed, Defense, Special). Edge-of-radius cards are unusual/powerful |
| Rarity | `Scripts/Features/Card/Services/SignatureCardHelper.cs` | Computed from signature extremity: each element's distance from 0.5 scored on threshold scale (0/1/2/4/8), summed, log-scaled, bucketed into 5 tiers |
| Card Visuals | `Scripts/Features/Card/Components/CardShaderRenderer.cs` | ~19 named layers (base, border, corners, art, symbol, banner, gems, energy fills) composited via shader. 8 gem sockets with emission color/strength from signature |
| Card Generation | `Scripts/Features/Card/Services/SignatureCardGenerator.cs` | Deterministic: seed from signature values → seeded RNG selects textures/visuals. Same signature = same card always |
| Flutter Physics | `Scripts/Features/Card/Components/FlutterCard.cs` | Aerodynamic forces: drag (v²), lift (perpendicular to face), twist/pitch torques with sinusoidal oscillation + noise. Per-card random phase offset |
| Blacklight | `Scripts/Features/Card/Controllers/BlacklightController.cs` | UV light reveal: exposure from spotlight distance/angle → shader parameter reveals hidden card properties |
| Card Holder | `Scripts/Features/Card/Components/CardHolder.cs` | Hand management: cards reparented to camera anchor, physics frozen, stacked by thickness offset |
| Card Dropper | `Scripts/Features/Card/Components/CardDropper.cs` | Right-click to prepare, release to drop. Ray-cast preview to landing point. Drop single (X) or all at once |
| Deck Slots | `Scripts/Features/Deckbuilder/Models/DeckSlot.cs` | Physical zones (Area3D) — cards enter, freeze, stack. ConsumeAllCardSignatures() extracts signatures and destroys physical cards |
| Combat | `Scripts/Features/Deckbuilder/Services/SimpleCombatSystem.cs` | Turn-based with Command pattern (undoable). Player abilities from card signatures. Enemy power scales with signature intensity |
| Card→World Bridge | `Scripts/Features/Worldgen/CardBasedGradient.cs` | 1 card: sphere gradient. 2 cards: capsule gradient (interpolate). 3+: Bezier gradient (De Casteljau closed loop). Multi-card bonuses. Bilinear interpolation on 16x16 sample grid |
| Game Session | `Scripts/Features/Deckbuilder/Services/GameSessionService.cs` | State machine: WaitingForCards → GeneratingMap → Exploring → InCombat → GeneratingLoot → SessionComplete. Map size from signature complexity (50-75 tiles) |
| Exploration | `Scripts/Features/Deckbuilder/Services/ExplorationAI.cs` | A* with fog-of-war. FrontierExploration mode + PathToEnemy mode. Tracks seen/visited/visible tiles. Defers enemy pursuit until path completes |
| WFC Solver | `Scripts/Features/Worldgen/Wfc/WfcSolver.cs` | Weighted entropy (not tile count), soft constraints (diminishing returns, novelty, compactness), connectivity constraint, auto-tile gap enforcement, retry on contradiction |
| Biomes | `Scripts/Features/Worldgen/Biomes/BiomeDefinition.cs` | Affinity signature per biome, passable/blocked tile pools with weights, blocked percentage. Closest biome by signature distance at each position |
| Auto-Tiling | `Scripts/Features/Worldgen/AutoTiling/` | Dual-grid bitmask: visual tiles offset half-cell from data grid, 4-corner sampling → 4-bit bitmask. Gap constraint ensures one auto-tile type per 2x2 window |
| Persistence | `addons/saveable/` | JSON save/load. Cards serialize as signature + transform — visuals reconstructed deterministically |

### CardCleaner Docs Worth Reading

- `CLAUDE.md` — full architecture overview, card signature table, WFC details
- `docs/WORLDGEN_ARCHITECTURE.md` — complete worldgen design
- `docs/WFC_ALGORITHM.md` — WFC algorithm explanation
- `docs/TILE_BIOME_REFACTOR.md` — biome system design

## Translation Philosophy

This is **conceptual mining**, not porting. We are not turning Godot nodes into Rust structs.

**Do:**
- Extract the *game design idea* (e.g., "cards have multi-dimensional properties that influence world generation")
- Think about how that idea would work in a 2D ECS context with physics-based card manipulation
- Identify what engine features would need to exist (gap analysis)
- Consider what makes sense for our specific card game (table + hand + stash paradigm)

**Don't:**
- Try to replicate CardCleaner's class hierarchy
- Map Godot concepts 1:1 to Axiom2d equivalents
- Import CardCleaner's DI/service locator patterns (we use ECS)
- Assume 3D mechanics translate directly (conveyor belts, 3D card models)

### Concept Translation Examples

| CardCleaner Concept | Axiom2d Translation Direction |
|---------------------|-------------------------------|
| CardSignature (8D vector) | Card component with elemental properties — what dimensions fit our game? |
| Residual energy modifiers | Signature minus base = stat deltas. Data-driven element→stat mapping. Edge cards = interesting cards |
| Deterministic generation | Same signature = same card always. Serialize just the signature, reconstruct visuals. Reproducible loot |
| Layered card rendering (19 layers) | Multi-shape card face with UV regions per layer. Gem sockets with emission from signature |
| Rarity from extremity | Computed from how "extreme" the signature is — not random, emergent from the card's identity |
| Flutter physics | 2D adaptation: angular drag + rotational oscillation with per-card phase offset |
| Blacklight reveal | Proximity-based shader effect revealing hidden card properties — "inspection" mechanic |
| DeckSlot consumption | Physical zones where cards are placed, frozen, then consumed to trigger generation |
| CardBasedGradient (sphere/capsule/Bezier) | Cards-as-seeds: 1 card = uniform, 2 = interpolation, 3+ = Bezier loop. Multi-card bonuses |
| SimpleCombatSystem (Command pattern) | Turn-based with undo. Card signatures → abilities. Could mix in physics (fling cards?) |
| WFC map generation | Procedural 2D tilemap — needs engine tilemap support |
| Biome affinity | Biomes have signature affinity — card signatures influence which biomes appear where |
| ExplorationAI (frontier + enemy modes) | Player/AI explores with fog-of-war, switches to pursuit when enemy spotted |
| Game session state machine | WaitingForCards → Generate → Explore → Combat → Loot → repeat |
| Persistence (signature-only) | Cards stored as signatures, visuals reconstructed — tiny save files |

## Workflow

1. **Read CardCleaner source** for the system you're mining (use `gh api` to fetch files)
2. **Read existing Axiom2d state** — check `Doc/Card_Game_Roadmap.md` for what's already built
3. **Extract ideas** at the game-design level, not implementation level
4. **Identify engine gaps** — what would Axiom2d need that it doesn't have yet?
5. **Write roadmap entries** in the output format below

## Output Format

Write to `Doc/CardCleaner_Ideas_Roadmap.md` using this structure:

```markdown
# CardCleaner Ideas Roadmap

Ideas mined from [CardCleaner](https://github.com/TychoHenzen/CardCleaner) for Axiom2d's card game.
Not a porting guide — these are conceptual adaptations for our 2D physics-based card engine.

## Category Name

### Idea Title `[NOT STARTED]`
**Inspired by:** CardCleaner file/system reference
**Engine gaps:** What Axiom2d engine crates would need (if any)
**Why:** What this adds to the game experience

- [ ] Deliverable 1
- [ ] Deliverable 2
- [ ] Deliverable 3

---
```

**Rules for entries:**
- Each idea gets a status tag: `[NOT STARTED]`, `[IN PROGRESS]`, `[DONE]`
- Checkboxes are concrete deliverables, not vague goals
- "Engine gaps" explicitly lists new engine features needed (crate + capability)
- Ideas should be ordered by dependency within each category
- Keep ideas granular enough to complete in 1-2 sessions
- Include a dependency graph at the bottom if ideas have ordering constraints
