# Building a 2D Particle-Simulation Idle/Factory Game for Steam: Complete Technical Landscape

## TL;DR
- **Build a custom Rust engine on wgpu + bevy_ecs** (the developer's existing "Axiom" stack), running the particle simulation as a **GPU compute-shader Discrete Element Method (DEM) / Position-Based Dynamics (PBD) solver with spatial-hash neighbor search**, and use **Rapier2D as the rigid-body engine for machines** (conveyors, pistons, molds). This is the only stack that simultaneously hits 100k+ particles, gives full compute control, and leverages the developer's strongest skills. Godot 4 is the recommended fallback if tooling speed matters more than raw control.
- **Phase the build strictly: particle sim → machine interaction → factory/recipe layer → economy → logic/automation → LLM discovery.** The particle engine is the technical keystone and highest risk; prove 100k particles at 60 FPS with pressure and inter-particle reactions before building anything else.
- **The LLM "novel product discovery" feature is proven and shippable** — Neal Agarwal's *Infinite Craft* does exactly this with Llama on Together AI plus aggressive database caching for determinism. For an offline Steam game, ship a small local model (Phi/Qwen/Llama-3.2-1B–3B class) via **llama.cpp with GBNF grammar-constrained JSON output**, or avoid the LLM entirely with **embedding + nearest-neighbor** recipe generation. Cache every result to guarantee determinism.

## Key Findings

**1. Particles, not cellular automata, is the right call — and it changes the whole architecture.** Lafikobra's videos are Blender offline renders using 100,000+ particles per scene; they are not real-time and not a shippable technique, but they define the *visual target*. Real-time equivalents (Sandspiel, Noita, Powder Toy) are cellular automata on a grid, which the brief explicitly rejects. The correct real-time approach for true position/velocity particles at this scale is a **GPU compute-shader particle solver** using PBD or DEM with a spatial-hash grid for neighbor finding. Published implementations sustain 500k particles above 30 FPS and 1M particles interactively.

**2. Pressure/incompressibility in containers is a solved problem via Position-Based Fluids (PBF).** Macklin & Müller's "Position Based Fluids" (SIGGRAPH 2013) adds a density constraint to PBD that produces incompressible, stable behavior at large timesteps — exactly what you need for particles piling in hoppers and pushing through pipes. NVIDIA PhysX 5 ships a production PBD/PBF particle solver.

**3. Machines should be rigid bodies, not particles.** Every reference factory game (Factorio, Satisfactory, Mindustry) and Noita itself treats machines/structures as separate rigid or kinematic entities that particles collide against. Rapier2D (Rust) is faster than Box2D and far more stable on joints, making it the natural choice for conveyor surfaces, pistons, grinders, and molds as kinematic/dynamic bodies the particle field reads as collision geometry.

**4. Factory logistics have well-documented, cheap architectures.** Factorio compresses long belt runs into single "transport line" entities with amortized O(1) updates; Satisfactory-style belts are simpler one-in/one-out segments. Neither uses per-item physics. Developer "theor" (theor.xyz), reimplementing Satisfactory-style belts with Unity DOTS, reports his "implementation is highly parallel and works at 60fps for a million items"; a separate enzisoft Factorio-in-Unity writeup reports reaching "1 million items at 8 ms… The new 60 fps record is 1.7 million." Recipes are data-driven state machines.

**5. Economy: a mean-reverting "spot price + base price" model is the proven pattern.** Offworld Trading Company uses pure supply/demand where buying/selling moves price (a community strategy guide states "every 10 units of something bought will raise the price by $1") with slow baseline drift, plus an AI colony as a market stabilizer. This directly satisfies "flood the market → price drops."

**6. Player logic: copy Factorio's combinator/circuit-network model.** Signals-on-wires with arithmetic and decider combinators giving `if X > Y then Z`, latches, and clocks — a Turing-complete but approachable node system. Mindustry ships a more imperative "logic processor" block language.

**7. LLM product discovery is shipped and viable, but caching is mandatory.** Infinite Craft proves the concept; the critical engineering insight is that the LLM is called *only on cache-miss* and every pairing is stored so results are globally deterministic.

## Details

### 1. Particle Physics Engine

**Recommended core algorithm: GPU-based Position-Based Dynamics (PBD/XPBD) with an optional Position-Based Fluids density constraint, or a DEM contact solver for granular feel.**

- **Ten Minute Physics** by Matthias Müller (matthias-research.github.io/pages/tenMinutePhysics/) — the definitive practical PBD tutorial series covering position-based dynamics, XPBD, rigid bodies, and fluid simulation in short video lectures with code. The PoC solver follows this reference.
- **PBD/XPBD** (Müller et al. 2007; XPBD adds a compliance parameter decoupling stiffness from timestep/iteration count) predicts positions via explicit Euler under gravity, then iteratively projects geometric constraints (collision, distance, density). It derives velocity from position change, avoiding the instability of force-based explicit integration and permitting large timesteps — ideal for real-time. NVIDIA PhysX 5 and Isaac Sim both use PBD for particles, and cap neighbor counts (PhysX uses a max of 96 neighbors per particle) to bound cost.
- **Position-Based Fluids (PBF)** — Macklin & Müller, SIGGRAPH 2013 — is the pressure/incompressibility layer. It enforces a constant-density constraint per particle, adds an artificial pressure term for surface tension and even particle spacing, and applies vorticity confinement to restore lost energy. This is what makes particles behave plausibly under pressure in containers and pipes without full Navier-Stokes SPH cost.
- **DEM (Discrete Element Method)** is the alternative if you want a granular/aggregate "grinding powder" feel matching Lafikobra. Each grain is a particle with contact forces (Hertzian normal, Mindlin friction). Project Chrono's `Chrono::GPU` and `DEM-Engine` are the reference open-source GPU DEM implementations, but they are research-grade dual-GPU C++ tools — you would borrow the *contact model math*, not the library. Note DEM often needs small timesteps for stiff contacts, so a PBD-style solver is generally the better real-time compromise.
- A pragmatic hybrid used in industry: PhysX-style physics engines simulate "realistic particles" much faster than academic DEM because they use simplified contact models and GPU parallelism — the research literature confirms PhysX produces DEM-comparable results while running dramatically faster.

**Neighbor search (the performance-critical inner loop): spatial hashing / uniform grid on the GPU.** The standard pipeline: impose a uniform grid with cell size ≥ interaction radius; hash each particle to a cell; sort particles by cell (counting sort + parallel prefix scan is the fast GPU method); then each particle only checks its 9 neighboring cells (2D). A 2025 study (MDPI *Applied Sciences*, "Moving Towards Large-Scale Particle Based Fluid Simulation in Unity 3D") reports this Count-Sort + Parallel-Prefix-Scan spatial hashing turns the O(n²) search into O(n), hits throughput up to 168,600 particles/ms, maintains 5.7–6.0 ms frame times from 10k–1M particles, holds >30 FPS at 500k particles, and stays responsive at 1M — using a Structure-of-Arrays layout that beat Array-of-Structures by 30–45%. Use SoA layout for GPU cache efficiency.

**Data layout and GPU pipeline pattern (from Wicked Engine and others):**
- Store particles in GPU storage buffers (SoA: separate position, velocity, type/species buffers). A particle can be as small as 5 floats (x, y, vx, vy, species).
- Use an "alive list" with append/consume buffers or atomic counters so you only simulate/draw live particles.
- Use `DispatchIndirect`/`DrawIndirect` so particle counts never round-trip to the CPU.
- Render via instanced quads expanded in the vertex shader (avoid geometry shaders — they serialize badly). For very high counts, compute-shader "software rasterization" with atomic adds to a buffer can beat hardware rasterization (Mike Turitzin measured 37–432% speedups with 64-bit atomics).

**Inter-particle reactions (mixing type A + type B → type C):** handled naturally in the neighbor-search compute pass — when two particles of reacting species are within radius, probabilistically transmute one/both by rewriting the species field (and optionally spawning/removing particles via the alive-list). This is the particle analogue of Sandspiel's element rules, but on true particles.

**Particle↔machine interaction:** Machines are rigid/kinematic colliders. The particle solver adds collision constraints against machine geometry (SDF or polygon colliders). Noita's trick is instructive: when a rigid body moves through the pixel field, affected pixels are lifted into a velocity-based particle simulation — the inverse of what you want, but the same principle of coupling a rigid-body engine to the particle field. Feed Rapier2D collider positions into the compute shader each frame as an array of shapes/SDFs.

### 2. Engine / Tech Stack Selection

| Option | Particle-sim capability | Compute shaders | 2D tooling | Steam integration | Ecosystem/maturity | Verdict |
|---|---|---|---|---|---|---|
| **Custom Rust (wgpu + bevy_ecs) — "Axiom"** | Full control; write your own DEM/PBD compute pipeline in WGSL; ceiling = your skill | First-class via wgpu; WGSL (also SPIR-V) | You build it; bevy_ecs is excellent for machines/factory ECS | `steamworks` crate (Noxime) + `bevy_steamworks`; bundles SDK redistributables | You own it all; bevy_hanabi proves GPU particles work in this stack | **Primary recommendation** — matches developer's strongest skill and gives the control this game needs |
| **Godot 4** | Good; compute via `RenderingDevice` + GLSL compute; a Godot 4.1 compute-shader boids demo (niceeffort) simulated "32,000 boids at 30 FPS" vs ~300 on CPU | Yes, but API is new/verbose; GDScript particle *shaders* are noted as ~10× slower than needed | Best-in-class 2D editor, tilemaps, scene system | GodotSteam extension, mature | Large, friendly; fast iteration | **Strong fallback** — pick if editor/iteration speed beats raw control |
| **Unity (DOTS/compute)** | Proven (1M+ belt items via DOTS/Burst; many GPU particle projects) | Yes, mature | Excellent | Steamworks.NET / Facepunch.Steamworks | Huge | Viable given developer's C#/.NET skills, but licensing/runtime-fee history and less control than Rust |
| **Unreal** | Overkill for 2D | Yes (Niagara) | Weak 2D | Native | Huge | Not recommended |
| **MonoGame/FNA + compute** | Possible but you build the compute layer; compute support is limited/immature | Limited | Minimal | Via Steamworks.NET | Mature 2D but low-level | Only if committed to C# and custom engine |
| **SDL2/SDL3 + custom** | Full control, most work | Via Vulkan/GL yourself | None | Manual Steamworks | Minimal | High effort, no advantage over wgpu |
| **Raylib** | Basic; not built for 100k GPU particles | Limited | Basic | Manual | Small | Not recommended at this scale |

**Integratable libraries:**
- **Rapier2D** (Dimforge, pure Rust) — rigid-body engine for machines. Benchmarks: 5–8× faster than nphysics, slightly faster than Box2D, far more joint-stable, close to CPU-PhysX. Deterministic and snapshot-serializable (useful for save games).
- **bevy_hanabi** — GPU particle VFX plugin (compute-shader based, millions of particles) — useful reference/foundation, though it's tuned for VFX not gameplay physics, so you'll likely write a custom solver.
- **salva2d** (Dimforge) — SPH/PBF fluid in Rust, integrates with Rapier; a candidate if you want a ready fluid solver.
- **bevy_app_compute** — ergonomic wgpu compute dispatch inside Bevy.

**Steam integration is a solved, low-risk problem in Rust:** the `steamworks` crate bundles the redistributable SDK binaries (currently v158a via `bevy-steamworks`); achievements, stats, friends, and networking sockets are all exposed. Just ship the redistributable DLLs next to the binary.

### 3. Machine / Factory System Architecture

- **Conveyors:** Do not give each item physics. Factorio's model (FFF #176) merges contiguous belt segments into one "transport line" storing items as gap-offsets from the start; because a compressed belt stays compressed, the last-gap index only ever decreases, giving amortized O(1) updates — they update two integers instead of moving 200 items. Satisfactory's model is simpler: each belt is one entry → one exit, single lane, and items teleport across belt boundaries to avoid reservation logic. A Unity DOTS/Burst reimplementation of the Satisfactory approach ran a million items at 60 FPS by processing segments in reverse order.
- **ECS for machines:** Use bevy_ecs. Components: `Machine{kind}`, `Recipe`, `InputBuffer`, `OutputBuffer`, `ProcessingTimer`, `Collider(Rapier handle)`. Systems tick recipes, pull from input buffers, and emit to output/particle spawners. This cleanly separates the rigid machine layer from the GPU particle layer, which only needs collider transforms each frame.
- **Recipe/crafting as data-driven state machines:** Represent each operation (grind → heat → mold → mix) as a node consuming input particle-types/quantities and time, producing outputs. Sequences of operations form a directed production graph — the Factorio/Shapez pattern. Shapez.io (open-source, GPL) and Mindustry (open-source Java/libGDX) are directly readable references for recipe and block/production-chain code.
- **Particle-stream coupling:** machines read particle counts crossing an input trigger volume (a sensor collider), consume them from the alive-list, and spawn output particles of the new type at an output port.

### 4. Economic Simulation / Stock Market

**Recommended model: spot price with mean-reversion to a slowly drifting base price (the "rubber band" model), driven by player buy/sell volume.**

- Offworld Trading Company mechanics (documented on its wiki and community guides): prices move purely on supply/demand. A community strategy guide states "every 10 units of something bought will raise the price by $1 (Also, each click of the buy/sell buttons will purchase/sell 10 units)," and the official OTC Wiki adds a ceiling — "it will never require more than 100 units bought to increase the price by $1." Base demand drifts prices slowly upward over time; the official OTC Wiki gives the exact formula: "The base demand increases the price of every resource by 0.001 * # of Players * (seconds + 100). For example, 150 seconds into the game, prices will increase by $1 in a 4-player game." An AI "colony" consumes/produces to act as a **market stabilizer**, and random shortage/surplus events inject temporary demand/supply. Per the same strategy guide, "Prices of base materials (water, iron, etc.) will tend to be more volatile (faster changing) than prices of the more advanced goods… it takes a lot of factories to make a big change in prices of advanced resources, but the simple base resources can change in price very quickly."
- A community-proposed refinement (the "rubber band" model) that maps well to an idle game: keep a **stable base price** per commodity that drifts on long-term trends, plus a **spot price** pulled toward it by a restoring force; buying/selling stretches the band, with resistance increasing nonlinearly as spot diverges from base. This prevents the unrealistic 100→1 collapse from a few trades while preserving responsiveness.
- Implementation: `price += k_impact × net_trade_volume; price += k_revert × (base_price − price); base_price += slow_trend`. This is a discrete Ornstein-Uhlenbeck (mean-reverting) process plus an impact term — standard, cheap, and tunable. Add optional agent-based consumers (NPC buyers with demand curves) for emergent dynamics.
- This directly delivers the brief's requirement: flood the market with product X → `net_trade_volume` spikes negative → X's price drops.

### 5. Logic / Automation System

**Recommended: a Factorio-style circuit network (signals on wires + combinators), which is the genre's proven, approachable-yet-powerful model.**

- **Signals:** every wire carries a set of (item-type → integer) pairs that sum across connected sources.
- **Combinators:**
    - *Constant* — outputs fixed signals (virtual chest).
    - *Arithmetic* — `+ − × ÷`, modulo, bit-shift on signals.
    - *Decider* — `if input {>,<,=} threshold then output signal` — this is your `if price of X > Y then switch production` primitive.
    - Combinators 2.0 (FFF #384) added multiple AND/OR conditions and multiple outputs per decider, plus red/green wire input selection.
- **Latches, clocks, memory** are all buildable from feedback loops (SR/RS latch = decider feeding its own input), enabling stable state without flicker — critical so recipes don't oscillate.
- Mindustry offers an alternative model: an imperative "logic processor" block with a small instruction language (loops, conditionals, sensor reads, unit commands) — closer to Zachtronics-style programming. If your audience skews programmer, offer this; if broader, the visual combinator graph is more discoverable.
- For your specific "if market price of X > Y, switch production" use case, the decider combinator reading a market-price signal (exposed as a virtual signal) and enabling/disabling machines is the direct implementation.

### 6. LLM Integration for Novel Product Discovery

**This is proven. Neal Agarwal's *Infinite Craft* (Jan 2024) generates the result of combining any two elements via an LLM, and is the definitive reference.**

- **Model & infra:** Agarwal stated to PC Gamer: *"I'm using the latest Llama 2 LLM from Facebook on the backend. Every time someone tries to craft something novel, I ask Llama 2 with a prompt what the result should be."* Dot Esports confirms the hosting: "The language model LLaMA is responsible for making new element, while Together AI manages the servers… according to an interview with PC Gamer." Wikipedia notes it later also uses Llama 3.1 and caps each element at 20 tokens.
- **Determinism via caching (the essential engineering lesson):** Wikipedia — *"the game checks from its database if these two elements have already been combined before—if they have not, the generative AI creates a new element which is then saved to the database… to ensure that the same pair of elements always outputs the same result for all players."* The backend calls the LLM **only on cache-miss**; developer Arthur O'Dwyer infers a memcached-like store, and clones use SQLite/KV stores. The `isNew` flag (cache-miss) powers "First Discovery."
- **Scale:** Debarghya Das's technical writeup reports Infinite Craft handles "a mind-boggling 1000 crafts / second"; Wikipedia states that per Agarwal "over 300 million recipes are created each day."

**For an offline Steam game, three concrete options:**

1. **Ship a small local LLM via llama.cpp with GBNF grammar-constrained output.** llama.cpp's GBNF (GGML BNF) grammars constrain generation to valid JSON/schema at the sampling stage — invalid tokens are masked, guaranteeing structurally valid recipe output (e.g., `{"result": "...", "emoji": "...", "properties": {...}}`). It supports auto-converting a JSON Schema (Draft 7 subset) to GBNF. Small models in the Phi / Qwen / Llama-3.2-1B–3B class run on consumer CPUs/GPUs. Precedent: an iOS chess app fine-tuned and embedded LiquidAI's **LFM2-350M** (<1B params) locally; a 2026 arXiv proof-of-concept shows small language models generating dynamic game content on consumer hardware with a retry-until-success strategy. **Caveat:** grammar guarantees structure, not semantic validity or that generation completes before the token limit — you still need a validation/retry layer and a cache.
2. **Skip the LLM: embedding + nearest-neighbor recipe generation.** Convert item names to vectors (word2vec/GloVe or a small embedding model like gpt4all), sum/average the two input vectors, and return the nearest dictionary word via cosine similarity — word2vec's "additive compositionality" (e.g., `Russian + river ≈ Volga River`; `human + robot ≈ cyborg`). Omar Shehata's writeup explicitly shows using gpt4all embeddings + the Vectra vector DB + cosine KNN to "basically recreate Neal's Infinite Craft." This is deterministic by construction, tiny, and fully offline. Gensim's `most_similar()` with pretrained vectors is the standard tooling.
3. **Pure procedural: a hash-based deterministic recipe table.** Hash the sorted pair of inputs (FNV-1a) to seed a deterministic pseudo-random selection from a curated output pool, or hand-author a recipe dictionary (the classic Little Alchemy / Doodle God model). Cheapest and most controllable, but least "novel."

**Recommendation:** Ship option 2 or 3 as the guaranteed-offline baseline (deterministic, tiny, no model download), and optionally layer option 1 (local llama.cpp + GBNF) as an opt-in "infinite discovery" mode. In all cases, **cache every combination to a local SQLite DB** so results are deterministic and repeat combinations are instant — this is the one non-negotiable pattern from Infinite Craft.

### 7. Development Roadmap / Scope

**MVP definition:** a single screen where particles pour from a hopper, fall under gravity with plausible pressure/piling, get carried by one conveyor, processed by one machine (e.g., a grinder that transmutes particle type), and the output sells into a price-reactive market. If that loop is fun at 50k+ particles and 60 FPS, the game works.

**Recommended phasing (strictly gated on the prior phase):**
1. **Particle sim core (highest risk — do first):** GPU compute PBD/DEM + spatial hash; prove 100k particles at 60 FPS with gravity, container pressure, and one inter-particle reaction. Benchmark on a mid-range GPU (GTX 1060 / RX 580 class), not just your dev machine — Sandspiel's author (Max Bittker) regretted spending his entire perf headroom, locking out low-end devices.
2. **Machine coupling:** Rapier2D bodies + collision constraints; conveyors, one grinder/heater. Machines read/consume/spawn particles.
3. **Factory/recipe layer:** data-driven recipe graph; multiple machine types; buffers; the grind→heat→mold→mix chain.
4. **Economy:** mean-reverting spot/base price market reacting to output.
5. **Logic/automation:** combinator network; `if price > Y switch production`.
6. **LLM/procedural discovery:** embedding or local-LLM recipe generation with SQLite caching.

**Performance budget:** target 16.6 ms/frame. The MDPI benchmark shows spatial-hash PBF fluid can hold ~6 ms for the sim across 10k–1M particles on capable GPUs; budget the rest for machines (Rapier, cheap), rendering (instanced/compute), and game logic. Keep everything in SoA GPU buffers; minimize CPU↔GPU round-trips (the classic bottleneck — early GPU particle work was limited to ~10k purely by CPU→GPU transfer). Use dirty-region/chunk updates for spatially localized work (Noita's 64×64 dirty-rect chunks are the model).

**Steam Early Access strategy:** The genre thrives in EA — factory/idle/automation players expect and reward iterative development, and many current idle/automation titles (e.g., *Idle Research*, *Revolution Idle*, *The Farmer Was Replaced*) launched and iterated in Early Access explicitly to refine balance and pacing with player feedback. Ship the MVP loop (phases 1–4) as EA with a clear roadmap, gather balance/pacing feedback, then layer logic and LLM discovery. Set up the Steam page early to collect wishlists during the particle-sim development phase.

## Recommendations

1. **Start immediately on the GPU particle solver as a standalone wgpu prototype**, decoupled from any game logic. Implement XPBD with a PBF density constraint and Count-Sort + Prefix-Scan spatial hashing, SoA buffers, indirect draw. **Go/no-go benchmark: 100k particles, 60 FPS, stable container pressure, one reaction, on a GTX 1060-class GPU.** If you can't hit this, drop the target to 50k or reconsider Godot for faster iteration.
2. **Use the developer's Axiom (wgpu + bevy_ecs) stack as the primary engine, Rapier2D for machines.** This maximizes the developer's Rust strength and gives the control the particle sim demands. Only switch to Godot 4 if, after ~2 weeks, engine plumbing (not the sim) is the bottleneck.
3. **Build the machine layer as pure ECS + Rapier kinematic bodies** that feed collider transforms to the compute shader; never make machines out of particles.
4. **Implement the economy as a mean-reverting spot/base-price model** with a per-commodity impact coefficient; tune so raw materials are more volatile than refined goods.
5. **Ship recipe discovery as embedding-NN or a hashed table first (offline, deterministic, tiny), with local-LLM-via-llama.cpp+GBNF as an opt-in stretch feature.** Cache everything to SQLite. Do not make a cloud LLM a hard dependency for an offline Steam game.
6. **Set up the Steam page during phase 1** and plan an Early Access launch after phase 4.

**Thresholds that change the plan:**
- If the 100k/60fps benchmark fails on mid-range GPUs → cut target particle count, adopt Godot for faster iteration, or fall back to a chunked hybrid.
- If Rapier2D can't keep up with machine counts → batch machines as static colliders / SDFs in the compute shader instead of individual rigid bodies.
- If local-LLM latency or model size hurts the player experience → drop to embedding-NN or a hand-authored recipe table.

## Caveats

- **Lafikobra's 100k-particle scenes are offline Blender renders, not real-time** — they set the aesthetic bar, not an achievable real-time spec. Match the *look*, not the fidelity.
- **The MDPI 168,600 particles/ms and 1M-particle figures are from academic GPU SPH benchmarks on capable hardware**; real-game frame budgets (with rendering, machines, and logic competing for the GPU) will be materially lower. The DOTS 1M-item belt figures are likewise from focused benchmarks, not shipping games with everything running at once. Treat all as ceilings, not guarantees, and always benchmark on mid/low-range hardware.
- **DEM/Chrono figures come from scientific dual-GPU simulators**, not games; borrow the contact-force math, not the performance numbers.
- **The "Llama 3.1" upgrade for Infinite Craft is Wikipedia-sourced, not a verbatim developer quote**; the confirmed on-record statement covers Llama 2. The "memcached" backing store is an informed inference by a third-party developer (Arthur O'Dwyer), not an official disclosure — only the deterministic caching *behavior* is confirmed.
- **GBNF grammar constraints guarantee structural (syntactic) validity, not semantic sense or completion** — a model can still emit a valid-JSON-but-nonsensical recipe, or hit the token limit mid-output. A validation + retry + cache layer is required.
- **No shipped game is confirmed to embed a local generative LLM for core crafting** at the time of writing; Infinite Craft is server-side, and local-LLM-in-game examples (chess, NPC dialogue) are proofs of concept. Shipping a bundled local model for recipe generation would be relatively novel and carries integration/QA risk — hence the recommendation to keep a deterministic non-LLM path as the shippable baseline.
- **Bevy is pre-1.0 and evolves fast**; expect breaking changes across versions. This is the main maturity risk of the Rust path versus Godot/Unity.