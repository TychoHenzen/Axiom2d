Research Report: Stable Wire/Rope Physics with Collision Wrapping

Executive Summary

Your current rope implementation is a custom Verlet particle chain with Jakobsen constraint relaxation — the right family  
of technique, but it has structural issues preventing both stiffness and proper wrapping. The two most viable paths forward
are: (1) Geometric Wrapping Wire for a taut cable that wraps precisely around obstacles, or (2) upgrading to XPBD for     
iteration-independent stiffness while keeping the particle aesthetic. A hybrid of both is the gold standard but the most   
complex.
                                                                                                                          
---                                                                                                           
Current State (Codebase)

Your rope lives in jack_cable.rs with this simulation loop:

verlet_step(ROPE_DAMPING=0.95) → relax_constraints × 8 → resolve_aabb_collisions → pin_endpoints

Key issues identified:

┌────────────────────────────────────────┬──────────────────────────────┬─────────────────────────────────────────────┐    
│                Problem                 │            Cause             │                   Impact                    │    
├────────────────────────────────────────┼──────────────────────────────┼─────────────────────────────────────────────┤    
│ Runs in Phase::Update, not FixedUpdate │ Frame-rate dependent dt      │ Rope behaves differently at 30fps vs 144fps │    
│                                        │                              │  — jitter, instability                      │    
├────────────────────────────────────────┼──────────────────────────────┼─────────────────────────────────────────────┤
│ ROPE_CONSTRAINT_ITERATIONS = 8         │ Too few for stiff cable with │ Segments stretch, creating visible slack    │    
│                                        │  many nodes                  │ and allowing loops                          │    
├────────────────────────────────────────┼──────────────────────────────┼─────────────────────────────────────────────┤    
│ apply_shrinkage exists but is never    │ Dead code in the system      │ The chord-straightening force that would    │    
│ called                                 │                              │ fight slack is unused                       │    
├────────────────────────────────────────┼──────────────────────────────┼─────────────────────────────────────────────┤    
│ ROPE_SLACK = 1.0 (no extra length)     │ Rest length = straight-line  │ Cable has zero natural droop — contradicts  │    
│                                        │ distance                     │ wanting some physical realism               │    
├────────────────────────────────────────┼──────────────────────────────┼─────────────────────────────────────────────┤    
│ AABB collision is single-pass push-out │ Not interleaved deeply       │ Nodes can tunnel through thin obstacles;    │    
│                                        │ enough with constraints      │ wrapping is approximate at best             │    
├────────────────────────────────────────┼──────────────────────────────┼─────────────────────────────────────────────┤    
│ resize_for_endpoints only shrinks at   │ Particles accumulate but     │ Wrapped cable retains excess particles      │    
│ wrap_ratio < 1.4                       │ rarely reduce                │ after unwrapping                            │    
├────────────────────────────────────────┼──────────────────────────────┼─────────────────────────────────────────────┤    
│ Rapier joints exist in engine but are  │ Custom Verlet is             │ Rapier colliders don't automatically        │    
│ unused                                 │ disconnected from rapier     │ participate in rope collision               │    
└────────────────────────────────────────┴──────────────────────────────┴─────────────────────────────────────────────┘
                                                                                                                             
---                                                                                                                        
Approaches

1. Geometric Wrapping Wire (Best for "taut cable")

How it works: The rope is NOT a particle chain — it's a list of anchor points plus one active segment. When the straight
line from the current anchor to the endpoint crosses an obstacle corner, a new anchor is inserted. When the endpoint swings
back past the anchor, it's removed (unwrap).

- No slack by construction — rope is always straight line segments between anchors
- Perfect wrapping — conforms exactly to obstacle vertices
- Pendulum physics on the active segment only (one rapier RopeJoint or distance constraint)
- Complexity: Medium. Core is 2D cross products + line-segment intersection. Edge cases around simultaneous contacts and   
  high angular velocity need care.

Trade-off: No visual sag/drape between anchors. The cable looks like straight line segments. You could add Catmull-Rom     
smoothing (which you already have) for visual polish.

Sources: http://antonior-software.blogspot.com/2016/12/box2d-ninja-rope.html,                                              
https://www.pentadact.com/2013-12-23-the-grappling-hook-game-dev-log-5-wrapping-and-slacking/

2. XPBD Upgrade (Best for "physical cable feel")

How it works: Replace Jakobsen constraint relaxation with Extended Position-Based Dynamics. Each constraint gets a Lagrange
multiplier and compliance parameter α = 1/stiffness. Effective compliance is α̃ = α/dt².                                   ───────────────────────────────────────────────────────────────────────────────────────────────────────────────   ( ◉  ◉ )

- Stiffness independent of iteration count — set α ≈ 0 for a near-rigid cable                                              
  -⏵Fixed timestep required((move+tobFixedUpdate)
- Same collision approach as current, but interleave collision resolution within each solver substep
- Complexity: Low incremental change from current code. Add one f32 per constraint (λ accumulator), change the constraint  
  projection formula.

Trade-off: Still approximate wrapping (particle-based draping, not geometric). Needs enough particles to resolve obstacle  
shapes. Better than current but not exact.

Sources: http://mmacklin.com/xpbd.pdf, https://matthias-research.github.io/pages/tenMinutePhysics/index.html

3. Rapier Joint Chains (Avoid)

Strong consensus in game dev: chaining physics engine joints for stiff rope is fragile. Constraint errors accumulate along
the chain. You'd get automatic collision but poor stiffness control. Not recommended for your use case.

4. Hybrid: Geometric Wrapping + Verlet Visual Segments

How it works: Geometric layer tracks wrap anchors (approach 1). Between anchors, short Verlet/XPBD chains provide visual   
drape. Geometric layer ensures no global slack; particle chains add local realism.

- Best of both worlds — taut overall, visually organic locally
- Most complex — two systems to synchronize

  ---                                                                                                                        
Trade-offs

┌─────────────────────────┬─────────────────────────────────────┬────────────────────────┬─────────────────────┐           
│        Criterion        │           Geometric Wire            │      XPBD Upgrade      │       Hybrid        │           
├─────────────────────────┼─────────────────────────────────────┼────────────────────────┼─────────────────────┤           
│ No unwanted slack/loops │ Perfect (structural)                │ Good (tunable)         │ Very Good           │           
├─────────────────────────┼─────────────────────────────────────┼────────────────────────┼─────────────────────┤        
│ Wrapping precision      │ Exact at vertices                   │ Approximate (draping)  │ Exact + approximate │
├─────────────────────────┼─────────────────────────────────────┼────────────────────────┼─────────────────────┤           
│ Visual quality          │ Straight segments (needs smoothing) │ Natural sag/sway       │ Best                │           
├─────────────────────────┼─────────────────────────────────────┼────────────────────────┼─────────────────────┤           
│ Implementation effort   │ Medium (new system)                 │ Low (modify existing)  │ High                │           
├─────────────────────────┼─────────────────────────────────────┼────────────────────────┼─────────────────────┤           
│ Rapier integration      │ One joint for active segment        │ Query-only (current)   │ Hybrid              │           
├─────────────────────────┼─────────────────────────────────────┼────────────────────────┼─────────────────────┤           
│ Frame-rate sensitivity  │ Low (geometric)                     │ None if in FixedUpdate │ Mixed               │           
└─────────────────────────┴─────────────────────────────────────┴────────────────────────┴─────────────────────┘
                                                                                                                             
---                                                                                                                        
Recommendation

For your specific requirements (no stable loops without obstacles + full wrapping around objects):

Go with the Geometric Wrapping Wire (Approach 1) as the primary mechanism:

1. Replace the Verlet particle chain with a wrap-point list + active segment
2. Detect wrap/unwrap using 2D cross products against CableCollider corners
3. Use your existing catmull_rom_subdivide for visual smoothing between anchor points
4. Move simulation to Phase::FixedUpdate for stability
5. Optionally add short Verlet chains between anchors later (Approach 4) if you want visible drape

If you prefer an incremental improvement over the current system rather than a rewrite:

1. Move rope_physics_system to Phase::FixedUpdate
2. Upgrade constraint solver from Jakobsen to XPBD (small code change)
3. Increase constraint iterations or use substeps
4. Activate apply_shrinkage with tuned strength to fight slack
5. Add per-iteration collision interleaving (collision inside the constraint loop, not just after)

Quick wins you can do right now regardless of approach:
- Move to FixedUpdate — this alone will dramatically improve stability
- Call apply_shrinkage with a moderate strength (0.1–0.3) to fight slack formation

  ---                                                                                                                        
Sources

1. https://toqoz.fyi/game-rope.html
2. https://jonathanwhiting.com/writing/blog/verlet_rope/
3. http://mmacklin.com/xpbd.pdf
4. https://matthias-research.github.io/pages/tenMinutePhysics/index.html
5. http://antonior-software.blogspot.com/2016/12/box2d-ninja-rope.html
6. https://www.pentadact.com/2013-12-23-the-grappling-hook-game-dev-log-5-wrapping-and-slacking/
7. https://www.cs.cmu.edu/afs/cs/academic/class/15462-s13/www/lec_slides/Jakobsen.pdf
8. https://rapier.rs/docs/user_guides/rust/joints/
9. https://www.owlree.blog/posts/simulating-a-rope.html
10. https://dylan-weeks.com/rope-simulation/                                                                               
                                                             