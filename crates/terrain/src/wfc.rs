use std::collections::BTreeSet;

use rand::Rng;
use rand_chacha::ChaCha8Rng;

use crate::material::TerrainId;

#[derive(Debug, thiserror::Error)]
pub enum WfcError {
    #[error("WFC contradiction: no valid assignment exists")]
    Contradiction,
}

/// Which terrain types may be placed next to each other.
#[derive(Clone, Debug)]
pub struct ConstraintTable {
    types: Vec<TerrainId>,
    /// Set of allowed `(from, to)` adjacency pairs.
    allowed: BTreeSet<(TerrainId, TerrainId)>,
}

impl ConstraintTable {
    #[must_use]
    pub fn new(types: Vec<TerrainId>) -> Self {
        Self {
            types,
            allowed: BTreeSet::new(),
        }
    }

    pub fn allow(&mut self, from: TerrainId, to: TerrainId) {
        self.allowed.insert((from, to));
    }

    fn is_allowed(&self, from: TerrainId, to: TerrainId) -> bool {
        self.allowed.contains(&(from, to))
    }

    fn types(&self) -> &[TerrainId] {
        &self.types
    }
}

/// A 2D grid of optionally-collapsed cells.
#[derive(Clone, Debug)]
pub struct Grid {
    width: usize,
    height: usize,
    cells: Vec<Option<TerrainId>>,
}

impl Grid {
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![None; width * height],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    #[must_use]
    pub fn get(&self, x: usize, y: usize) -> Option<TerrainId> {
        self.cells[y * self.width + x]
    }

    pub fn set(&mut self, x: usize, y: usize, value: Option<TerrainId>) {
        self.cells[y * self.width + x] = value;
    }

    fn neighbors(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let mut result = Vec::with_capacity(4);
        if x > 0 {
            result.push((x - 1, y));
        }
        if x + 1 < self.width {
            result.push((x + 1, y));
        }
        if y > 0 {
            result.push((x, y - 1));
        }
        if y + 1 < self.height {
            result.push((x, y + 1));
        }
        result
    }
}

/// Collapse the grid using WFC with constraint propagation and backtracking.
pub fn collapse(
    grid: &mut Grid,
    constraints: &ConstraintTable,
    rng: &mut ChaCha8Rng,
) -> Result<(), WfcError> {
    let w = grid.width;
    let h = grid.height;

    // Build possibility sets for each cell
    let all_types: BTreeSet<TerrainId> = constraints.types().iter().copied().collect();
    let mut possible: Vec<BTreeSet<TerrainId>> = vec![all_types.clone(); w * h];

    // Constrain from pinned cells
    for y in 0..h {
        for x in 0..w {
            if let Some(id) = grid.get(x, y) {
                possible[y * w + x] = BTreeSet::from([id]);
                propagate(grid, &mut possible, constraints, x, y);
            }
        }
    }

    // Main loop: pick lowest-entropy uncollapsed cell, collapse it
    let max_backtracks = w * h * 4;
    let mut backtrack_count = 0;
    let mut history: Vec<(usize, usize, BTreeSet<TerrainId>, Vec<BTreeSet<TerrainId>>)> =
        Vec::new();

    loop {
        // Find uncollapsed cell with fewest possibilities
        let mut best: Option<(usize, usize, usize)> = None;
        let mut found_contradiction = false;
        for y in 0..h {
            for x in 0..w {
                if grid.get(x, y).is_some() {
                    continue;
                }
                let count = possible[y * w + x].len();
                if count == 0 {
                    found_contradiction = true;
                    break;
                }
                if best.is_none() || count < best.expect("checked").2 {
                    best = Some((x, y, count));
                }
            }
            if found_contradiction {
                break;
            }
        }

        if found_contradiction {
            if let Some((bx, by, remaining, snapshot)) = history.pop() {
                backtrack_count += 1;
                if backtrack_count > max_backtracks {
                    return Err(WfcError::Contradiction);
                }
                grid.set(bx, by, None);
                possible = snapshot;
                possible[by * w + bx] = remaining;
                continue;
            }
            return Err(WfcError::Contradiction);
        }

        let Some((cx, cy, _)) = best else {
            break; // All cells collapsed
        };

        let options: Vec<TerrainId> = possible[cy * w + cx].iter().copied().collect();
        let chosen_idx = rng.random_range(0..options.len());
        let chosen = options[chosen_idx];

        // Save state for backtracking
        let mut remaining = possible[cy * w + cx].clone();
        remaining.remove(&chosen);
        history.push((cx, cy, remaining, possible.clone()));

        // Collapse this cell
        grid.set(cx, cy, Some(chosen));
        possible[cy * w + cx] = BTreeSet::from([chosen]);
        propagate(grid, &mut possible, constraints, cx, cy);
    }

    Ok(())
}

fn propagate(
    grid: &Grid,
    possible: &mut [BTreeSet<TerrainId>],
    constraints: &ConstraintTable,
    start_x: usize,
    start_y: usize,
) {
    let w = grid.width;
    let mut queue = vec![(start_x, start_y)];

    while let Some((x, y)) = queue.pop() {
        let current_set = possible[y * w + x].clone();
        for (nx, ny) in grid.neighbors(x, y) {
            if grid.get(nx, ny).is_some() {
                continue;
            }
            let idx = ny * w + nx;
            let before = possible[idx].len();
            possible[idx].retain(|&candidate| {
                current_set
                    .iter()
                    .any(|&src| constraints.is_allowed(src, candidate))
            });
            if possible[idx].len() < before {
                queue.push((nx, ny));
            }
        }
    }
}
