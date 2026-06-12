# GPS Companion App — Design Spec

## Concept

Mobile app collects **signature grains** (tiny 8-float `CardSignature` fragments) by walking outdoors. 100 grains auto-forge into a booster pack. At home, scan a QR code on the desktop game to transfer packs over LAN. Desktop game generates cards from the collected signature data using the existing `CardSignature` pipeline.

Mobile knows nothing about cards. Desktop is the sole card factory.

## Grain Data Model

A grain is a miniature `CardSignature` — 8 floats in `[-1, 1]`, one per Element:

| Axis | Element | Aspect Pair | Grain Type | Biome Source |
|------|---------|-------------|------------|--------------|
| 0 | Solidum | Solid/Fragile | Earth | Mountains, rock, desert |
| 1 | Febris | Heat/Cold | *(leyline)* | Procedural overlay only |
| 2 | Ordinem | Order/Chaos | Urban | Buildings, roads, industrial |
| 3 | Lumines | Light/Dark | *(leyline)* | Procedural overlay only |
| 4 | Varias | Change/Stasis | Water | Rivers, lakes, coast |
| 5 | Inertiae | Force/Calm | *(leyline)* | Procedural overlay only |
| 6 | Subsidium | Growth/Decay | Nature | Forests, parks, grassland |
| 7 | Spatium | Expansion/Contraction | Arcane | Universities, churches, historic |

5 terrestrial grain types + 3 leyline types (wild, overlay-only).

### Grain Structure

```rust
struct Grain {
    axes: [f32; 8],       // dominant axis ~0.01–0.15, others ~±0.005
    grain_type: GrainType, // Nature | Urban | Water | Earth | Arcane | Febris | Lumines | Inertiae
    rarity: GrainRarity,   // Common | Uncommon | Rare | Epic | Legendary
}
```

### Grain Rarity

Same `geometric_level` algorithm as `CardSignature::rarity_with_config()`. Dominant axis magnitude determines rarity:

| Rarity | Dominant axis magnitude | Spawn rate |
|--------|------------------------|------------|
| Common | ~0.01 | ~60% |
| Uncommon | ~0.03 | ~25% |
| Rare | ~0.06 | ~10% |
| Epic | ~0.10 | ~4% |
| Legendary | ~0.15 | ~1% |

Grain rarity is per-grain. 100 legendary grains → large aggregated signature → `CardSignature::rarity()` returns Legendary. No special rules needed.

## Biome System

### Layer 1 — Base Biome (static)

Embedded OSM land-use polygons (~5MB simplified GeoJSON). Point-in-polygon lookup at GPS coordinate returns grain type distribution:

| Biome | Nature | Urban | Water | Earth | Arcane | Leyline |
|-------|--------|-------|-------|-------|--------|---------|
| Forest/Park | 65% | 5% | 10% | 10% | 10% | 0% |
| Urban/Residential | 10% | 60% | 5% | 10% | 15% | 0% |
| Water/Coast | 15% | 5% | 55% | 10% | 15% | 0% |
| Mountain/Desert | 5% | 5% | 5% | 70% | 15% | 0% |
| Historic/Cultural | 10% | 15% | 5% | 5% | 65% | 0% |
| No data (fallback) | 25% | 25% | 15% | 20% | 15% | 0% |

Leyline types never appear in base biome — they come from the overlay.

### Layer 2 — Procedural Leyline Overlay (weekly seed)

3D simplex noise over `(lat, lon, week_number)`. Outputs 8-element modifier vector applied as multiplier to base distribution. Each week has a thematic "alignment":

- Febris-dominant week → Heat/Cold leylines visible as red/blue wisps on map
- Lumines-dominant week → Light/Dark leylines as white/purple wisps
- Inertiae-dominant week → Force/Calm leylines as orange/teal wisps
- Mixed weeks → multiple leyline colors visible

**Effect:** noise at a given GPS coordinate amplifies specific elements' spawn chances. A forest (65% Nature) might get +25% Ordinem this week → Urban grains appear among trees. Strong leyline nodes become "hotspots" players learn to revisit.

**Visualization:** colored leyline wisps rendered on mobile map as semi-transparent overlays. Player sees where energy flows but not exact grain positions. Partial information — encourages exploration without removing discovery.

### Weekly Leyline Theme

Displayed in app header. Examples:
- "Week 23: Febris Surge" — strong Heat/Cold leylines
- "Week 24: Arcane Alignment" — Spatium amplified globally
- "Week 25: Chaos Confluence" — all 3 leyline elements active, weak everywhere

Two parameters per week: which leyline element(s) dominate, and global intensity (0.5× to 2.0× multiplier on overlay magnitude).

## Spawn Algorithm

```
for each S2 cell (level ~15, ~300m²) within viewport:
    cell_seed = hash(cell_id, date, week_seed)
    grain_count = poisson_sample(cell_seed, biome_density)
    
    for each grain in grain_count:
        position = deterministic_jitter(cell_seed, grain_index)
        axes = generate_grain_axes(base_biome, overlay_noise, position, cell_seed)
        grain_type = argmax(|axes[i]|)  // dominant element
        rarity = geometric_level(hash(axes), config)
        
        spawn grain at position
```

Density per biome (grains per km² per day):
- Urban: 200 (highest — player is always in urban areas)
- Forest/Park: 300 (reward for going to parks)
- Water/Coast: 150 (linear features, follow coastline)
- Mountain/Desert: 100 (sparse but high Epic+ rate)
- Historic/Cultural: 80 (scarcest, highest Legendary rate)

Grains expire at midnight UTC. Uncollected grains replaced by new spawns.

## Collection & Forging

- **Collect:** tap grain on map when within ~20m radius. Grain adds to inventory (in-memory array, JSON-serialized to localStorage for persistence).
- **Auto-forge:** when grain count hits 100, forge into booster pack. No manual trigger. Grain counter resets to 0. Remaining grains spill over into next pack.
- **Booster pack storage:** array of booster packs, each = `Vec<Grain; 100>`. No limit. Persisted to localStorage.
- **Booster pack theming (mobile UI):** each pack shows wrapper art based on grain mix:
  - ≥60% single type → that type's color theme (green/Nature, gray/Urban, blue/Water, brown/Earth, purple/Arcane)
  - Mixed (no type ≥50%) → iridescent blend
  - ≥90% single type → intense glow + "Pure" badge → guarantee Rare+ card
  - ≥5 Legendary grains → gold shimmer effect

## LAN Transfer

### Pairing Flow

1. Desktop: player clicks "Pair Phone" → spawns WebSocket server on random port
2. Desktop: renders QR code containing `{"ip":"192.168.1.42","port":9876,"token":"a1b2c3"}`
3. Phone: scans QR → connects WebSocket → validates token
4. Phone: sends all booster packs as JSON array
5. Desktop: receives → sums grain axes per pack → `CardSignature` → cards spawn
6. Desktop: shows "X booster packs imported" confirmation
7. Connection closes. QR invalidated.

QR token rotates every 60s. Server dies after successful transfer or 120s timeout.

### Payload Format

```json
{
  "version": 1,
  "device_id": "abc123",
  "boosters": [
    {
      "forged_at": "2026-06-05T14:30:00Z",
      "location": {"lat": 51.5074, "lon": -0.1278},
      "location_name": "Hyde Park",
      "grains": [
        {"axes": [0.02,-0.01,0.0,0.0,0.0,0.0,0.0,0.0], "grain_type": "Nature", "rarity": "Common"},
        ...
      ]
    }
  ]
}
```

### Desktop Card Generation

```rust
fn grain_batch_to_cards(grains: &[Grain; 100]) -> Vec<CardDefinition> {
    let mut combined = [0.0f32; 8];
    for grain in grains {
        for i in 0..8 {
            combined[i] += grain.axes[i];
        }
    }
    // Clamp to [-1, 1]
    for v in &mut combined {
        *v = v.clamp(-1.0, 1.0);
    }
    
    let signature = CardSignature::new(combined);
    let rarity = signature.rarity();
    // Feed into existing procedural card generation pipeline
    // Card count per booster determined by Rust-side unpack logic
}
```

Card count, rarity distribution, and type assignment from booster pack are controlled entirely by desktop-side unpack logic. Grain mix influences but does not dictate — the existing `CardSignature → CardDefinition` pipeline owns all generation.

## Desktop Integration

New `card_game::import` module:

```
crates/card_game/src/import/
    mod.rs              — ImportPlugin, ImportState resource
    lan_server.rs       — WebSocket server (tungstenite + std::thread)
    qr_display.rs       — QR code → engine texture (qrcode crate)
    grain_unpack.rs     — GrainBatch → CardSignature → spawn cards
```

### New Dependencies (Rust workspace)

- `tungstenite` — lightweight WebSocket, no async runtime needed (use std::thread)
- `qrcode` — QR bitmap generation

### Systems

| System | Phase | Purpose |
|--------|-------|---------|
| `import_pairing_system` | Update | Monitors ImportState, spawns LAN server on request |
| `import_qr_display_system` | Render | Renders QR code as fullscreen overlay when pairing |
| `import_receive_system` | Update | Polls WebSocket for incoming data, deserializes, spawns |
| `import_timeout_system` | Update | Kills server after 120s idle or on success |

## Mobile App

### Tech Stack

**Prototype:** PWA (HTML/JS, MapLibre GL, simplex-noise.js, jsQR)
- Zero app store, iterate in hours
- GPS via `navigator.geolocation.watchPosition()`
- Map with biome tiles + leyline overlay
- localStorage for grain/booster persistence
- WebSocket for LAN transfer
- PWA manifest for home screen install

**Production:** Flutter (if app store distribution needed)

### UI Screens

1. **Map** — primary screen. GPS dot, grain nodes (colored by type, sized by rarity), leyline overlay, grain counter (87/100), pack count badge
2. **Booster Gallery** — scrollable grid of forged packs, each with wrapper art based on grain mix. Tap to see details (location, date, grain breakdown)
3. **Transfer** — QR scanner viewfinder + "Ready to send X packs" button

### Offline

All data local. No backend. No auth. No internet requirement beyond initial PWA load.

## Development Phases

### Phase 1: Desktop Import Pipeline (1-2 days, Rust)
- `GrainBatch` JSON format finalized with serde
- Deserialize → sum axes → `CardSignature` → spawn cards
- QR display system (show QR, no scanner/sender yet)
- Test with hand-crafted JSON payloads

### Phase 2: LAN Server (1 day, Rust)
- WebSocket server on separate thread
- Accept connection, receive booster packs JSON
- Wire into Phase 1 pipeline
- Test with browser WebSocket console

### Phase 3: Mobile PWA (3-5 days, JS)
- Map view with GPS + grain spawning
- OSM biome data (pre-processed GeoJSON)
- Procedural leyline overlay (simplex-noise)
- Collect → auto-forge → booster gallery
- QR scanner + WebSocket send
- PWA manifest + service worker

### Phase 4: Tuning (ongoing)
- Grain density feel
- Leyline seed cadence (weekly confirmed)
- Booster theming visuals
- Card generation balance from real grain data
