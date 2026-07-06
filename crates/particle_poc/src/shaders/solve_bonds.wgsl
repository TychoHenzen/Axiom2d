// Bond constraint solver: tension-only distance constraints for green bonds.
// Runs each substep after PBD project, before apply.
// Accumulates bond corrections into the corrections buffer (binding 3) rather
// than modifying positions directly. The apply pass adds corrections[i] to
// positions[i] and recomputes velocities — keeping the PBD pipeline consistent.
//
// Both sides independently process their bond declarations. Each writes
// half the correction (0.5 each) → equal opposite → momentum conserved when mutual.
// Tension-only: corrects only when dist > rest (stretched). Never fights PBD.
// Bonds clear when species changes or break when dist > BOND_BREAK_MULTIPLIER * rest.

struct Params {
    particle_count: u32,
    dt: f32,
    gravity: f32,
    particle_radius: f32,
    wall_min_x: f32,
    wall_min_y: f32,
    wall_max_x: f32,
    wall_max_y: f32,
    friction_mu: f32,
    grid_cell_size: f32,
    grid_width: u32,
    grid_height: u32,
    _pad0: u32,
    _pad1: u32,
}

struct BondSlot {
    partner: u32,
    rest: f32,
}

const GREEN_SPECIES: u32 = 2u;
const INVALID_BOND: u32 = 0xFFFFFFFFu;
// Fraction of stretch corrected per substep per side. With both sides
// processing a mutual bond, net correction = BOND_COMPLIANCE * stretch.
// 0.15 at 16 substeps compounds to ~93% correction per frame.
const BOND_COMPLIANCE: f32 = 0.08;
const BOND_BREAK_MULTIPLIER: f32 = 3.0;

@group(0) @binding(0) var<storage, read_write> positions: array<vec2<f32>>;
@group(0) @binding(2) var<storage, read_write> species: array<u32>;
@group(0) @binding(3) var<storage, read_write> corrections: array<vec2<f32>>;
@group(0) @binding(7) var<uniform> params: Params;
@group(0) @binding(10) var<storage, read_write> bond_slot_a: array<BondSlot>;
@group(0) @binding(11) var<storage, read_write> bond_slot_b: array<BondSlot>;

@group(1) @binding(0) var<storage, read_write> _cell_idx: array<u32>;
@group(1) @binding(1) var<storage, read_write> _cell_cnt: array<u32>;
@group(1) @binding(2) var<storage, read_write> _cell_off: array<u32>;
@group(1) @binding(3) var<storage, read_write> _sorted: array<u32>;
@group(1) @binding(4) var<storage, read_write> _morton: array<u32>;

fn solve_slot_a(i: u32) {
    let partner = bond_slot_a[i].partner;
    if partner == INVALID_BOND { return; }
    let rest = bond_slot_a[i].rest;

    if species[i] != GREEN_SPECIES || species[partner] != GREEN_SPECIES {
        bond_slot_a[i].partner = INVALID_BOND;
        bond_slot_a[i].rest = 0.0;
        return;
    }

    let delta = positions[i] - positions[partner];
    let dist = length(delta);

    if dist > BOND_BREAK_MULTIPLIER * rest {
        bond_slot_a[i].partner = INVALID_BOND;
        bond_slot_a[i].rest = 0.0;
        return;
    }

    // Tension only.
    if dist <= rest { return; }

    let error = dist - rest;
    let n = delta / dist;
    // Half correction — partner's thread also applies 0.5 if mutual,
    // giving correct total. If unilateral, corrects 50% which is harmless.
    var bond_correction = 0.5 * error * n * BOND_COMPLIANCE;
    // Cap per-substep correction to particle_radius to prevent energy spikes.
    let max_corr = params.particle_radius;
    let mag = length(bond_correction);
    if mag > max_corr {
        bond_correction *= max_corr / mag;
    }
    // Accumulate into corrections buffer — apply pass adds this to position
    // and recomputes velocity correctly.
    corrections[i] -= bond_correction; // -n: move i toward partner
}

fn solve_slot_b(i: u32) {
    let partner = bond_slot_b[i].partner;
    if partner == INVALID_BOND { return; }
    let rest = bond_slot_b[i].rest;

    if species[i] != GREEN_SPECIES || species[partner] != GREEN_SPECIES {
        bond_slot_b[i].partner = INVALID_BOND;
        bond_slot_b[i].rest = 0.0;
        return;
    }

    let delta = positions[i] - positions[partner];
    let dist = length(delta);

    if dist > BOND_BREAK_MULTIPLIER * rest {
        bond_slot_b[i].partner = INVALID_BOND;
        bond_slot_b[i].rest = 0.0;
        return;
    }

    if dist <= rest { return; }

    let error = dist - rest;
    let n = delta / dist;
    var bond_correction = 0.5 * error * n * BOND_COMPLIANCE;
    let max_corr = params.particle_radius;
    let mag = length(bond_correction);
    if mag > max_corr {
        bond_correction *= max_corr / mag;
    }
    corrections[i] -= bond_correction; // -n: move i toward partner
}

@compute @workgroup_size(256)
fn solve_bonds(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if i >= params.particle_count { return; }
    if species[i] != GREEN_SPECIES { return; }

    solve_slot_a(i);
    solve_slot_b(i);
}
