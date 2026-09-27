use std::collections::VecDeque;

/// Convert a binary occupancy grid (negative = wall, positive = free)
/// into a signed distance field with distances in texel units.
/// Uses BFS from boundary cells for approximate EDT.
/// Positive = free space distance to wall, negative = depth inside wall.
pub fn compute_distance_field(grid: &mut [f32], res: u32) {
    let n = (res * res) as usize;
    let mut dist = vec![f32::MAX; n];
    let mut queue = VecDeque::with_capacity(n / 4);
    let res_i = res as i32;

    // Seed boundary cells: any cell adjacent to a cell of opposite sign.
    for y in 0..res_i {
        for x in 0..res_i {
            let idx = (y as u32 * res + x as u32) as usize;
            let inside = grid[idx] < 0.0;
            let mut on_boundary = false;
            for &(dx, dy) in &[(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
                let nx = x + dx;
                let ny = y + dy;
                if nx >= 0 && nx < res_i && ny >= 0 && ny < res_i {
                    let ni = (ny as u32 * res + nx as u32) as usize;
                    if (grid[ni] < 0.0) != inside {
                        on_boundary = true;
                        break;
                    }
                }
            }
            if on_boundary {
                dist[idx] = 0.5;
                queue.push_back((x as u32, y as u32));
            }
        }
    }

    // BFS propagation with 8-connectivity.
    while let Some((x, y)) = queue.pop_front() {
        let idx = (y * res + x) as usize;
        let d = dist[idx];
        for &(dx, dy) in &[
            (1i32, 0i32),
            (-1, 0),
            (0, 1),
            (0, -1),
            (1, 1),
            (1, -1),
            (-1, 1),
            (-1, -1),
        ] {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx < 0 || nx >= res_i || ny < 0 || ny >= res_i {
                continue;
            }
            let ni = (ny as u32 * res + nx as u32) as usize;
            let step = if dx.abs() + dy.abs() > 1 {
                std::f32::consts::SQRT_2
            } else {
                1.0
            };
            let nd = d + step;
            if nd < dist[ni] {
                dist[ni] = nd;
                queue.push_back((nx as u32, ny as u32));
            }
        }
    }

    // Apply sign: negative inside wall, positive in free space.
    for i in 0..n {
        let sign = if grid[i] < 0.0 { -1.0 } else { 1.0 };
        grid[i] = sign * dist[i];
    }
}

/// Compute distance field on a copy of the binary grid, returning the
/// SDF-ready data for GPU upload. Leaves the original grid unchanged
/// so further painting works on binary values.
pub fn sdf_for_upload(grid: &[f32], res: u32) -> Vec<f32> {
    let mut copy = grid.to_vec();
    compute_distance_field(&mut copy, res);
    copy
}
