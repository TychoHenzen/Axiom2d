use std::collections::{HashSet, VecDeque};

// 8 directions clockwise: E, SE, S, SW, W, NW, N, NE.
const DIRS: [(i32, i32); 8] = [
    (1, 0),   // 0 E
    (1, 1),   // 1 SE
    (0, 1),   // 2 S
    (-1, 1),  // 3 SW
    (-1, 0),  // 4 W
    (-1, -1), // 5 NW
    (0, -1),  // 6 N
    (1, -1),  // 7 NE
];

const NEIGHBORS8: [(i32, i32); 8] = [
    (-1, 0),
    (-1, -1),
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
];

fn is_opaque(mask: &[bool], x: i32, y: i32, width: u32, height: u32) -> bool {
    if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
        return false;
    }
    mask[y as usize * width as usize + x as usize]
}

fn is_boundary(mask: &[bool], x: u32, y: u32, width: u32, height: u32) -> bool {
    if !mask[y as usize * width as usize + x as usize] {
        return false;
    }
    NEIGHBORS8
        .iter()
        .any(|(dx, dy)| !is_opaque(mask, x as i32 + dx, y as i32 + dy, width, height))
}

/// Map a neighbor offset to a DIRS index.
fn dir_index(dx: i32, dy: i32) -> usize {
    match (dx, dy) {
        (1, 0) => 0,
        (1, 1) => 1,
        (0, 1) => 2,
        (-1, 1) => 3,
        (-1, 0) => 4,
        (-1, -1) => 5,
        (0, -1) => 6,
        (1, -1) => 7,
        _ => unreachable!(),
    }
}

/// Mark all boundary pixels 8-connected to `(sx, sy)` as visited via BFS.
#[allow(clippy::too_many_arguments)]
fn mark_component_visited(
    mask: &[bool],
    sx: u32,
    sy: u32,
    width: u32,
    height: u32,
    visited: &mut HashSet<(u32, u32)>,
) {
    let mut queue: VecDeque<(u32, u32)> = VecDeque::new();
    visited.insert((sx, sy));
    queue.push_back((sx, sy));

    while let Some((x, y)) = queue.pop_front() {
        for &(dx, dy) in &DIRS {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
                continue;
            }
            let p = (nx as u32, ny as u32);
            if visited.contains(&p) {
                continue;
            }
            if !is_boundary(mask, p.0, p.1, width, height) {
                continue;
            }
            visited.insert(p);
            queue.push_back(p);
        }
    }
}

/// Trace the outer perimeter of a boundary component using Moore neighborhood
/// tracing. Returns boundary pixels in clockwise perimeter order.
///
/// Uses Jacob's stopping criterion: stop when we return to the start pixel
/// from the same direction we initially entered it.
#[allow(clippy::too_many_lines)]
fn trace_perimeter(mask: &[bool], sx: u32, sy: u32, width: u32, height: u32) -> Vec<(u32, u32)> {
    // Check if start pixel has any boundary neighbor
    let has_neighbor = DIRS.iter().any(|&(dx, dy)| {
        let nx = sx as i32 + dx;
        let ny = sy as i32 + dy;
        nx >= 0
            && ny >= 0
            && nx < width as i32
            && ny < height as i32
            && is_boundary(mask, nx as u32, ny as u32, width, height)
    });
    if !has_neighbor {
        return vec![(sx, sy)];
    }

    let mut contour: Vec<(u32, u32)> = vec![(sx, sy)];
    let mut seen: HashSet<(u32, u32)> = HashSet::new();
    seen.insert((sx, sy));

    let mut px = sx as i32;
    let mut py = sy as i32;
    // Initial backtrack: west of start (we scan left-to-right).
    let start_back = (sx as i32 - 1, sy as i32);
    let mut bx = start_back.0;
    let mut by = start_back.1;

    let max_steps = (width as usize) * (height as usize) * 2;
    for _ in 0..max_steps {
        let back_dir = dir_index(bx - px, by - py);

        let mut found = false;
        for i in 0..8 {
            let check_dir = (back_dir + i) % 8;
            let (dx, dy) = DIRS[check_dir];
            let nx = px + dx;
            let ny = py + dy;

            if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
                continue;
            }
            if !is_boundary(mask, nx as u32, ny as u32, width, height) {
                continue;
            }

            // t = pixel just before c in clockwise rotation around p
            let t_dir = (check_dir + 7) % 8;
            let (tdx, tdy) = DIRS[t_dir];
            let tx = px + tdx;
            let ty = py + tdy;

            // Jacob's stopping criterion
            if nx as u32 == sx
                && ny as u32 == sy
                && tx == start_back.0
                && ty == start_back.1
                && contour.len() > 1
            {
                return contour;
            }

            if seen.insert((nx as u32, ny as u32)) {
                contour.push((nx as u32, ny as u32));
            }

            px = nx;
            py = ny;
            bx = tx;
            by = ty;
            found = true;
            break;
        }

        if !found {
            break;
        }
    }

    contour
}

/// Extract closed boundary contours from a binary mask.
///
/// Scans left-to-right, top-to-bottom for unvisited boundary pixels. For each
/// new component, traces the outer perimeter using Moore neighborhood tracing
/// (8-directional, clockwise), then marks all 8-connected boundary pixels in
/// the component as visited via BFS.
pub fn trace_contours(mask: &[bool], width: u32, height: u32) -> Vec<Vec<(u32, u32)>> {
    let mut contours = Vec::new();
    let mut visited: HashSet<(u32, u32)> = HashSet::new();

    for y in 0..height {
        for x in 0..width {
            if visited.contains(&(x, y)) {
                continue;
            }
            if !is_boundary(mask, x, y, width, height) {
                continue;
            }
            let contour = trace_perimeter(mask, x, y, width, height);
            mark_component_visited(mask, x, y, width, height, &mut visited);
            if !contour.is_empty() {
                contours.push(contour);
            }
        }
    }

    contours
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::trace_contours;

    #[test]
    fn when_fully_transparent_mask_then_returns_empty() {
        // Arrange
        let mask = vec![false; 9];

        // Act
        let contours = trace_contours(&mask, 3, 3);

        // Assert
        assert!(contours.is_empty());
    }

    #[test]
    fn when_single_opaque_pixel_then_one_contour_returned() {
        // Arrange — center pixel of 3x3 mask
        let mut mask = vec![false; 9];
        mask[4] = true; // (1,1)

        // Act
        let contours = trace_contours(&mask, 3, 3);

        // Assert
        assert_eq!(contours.len(), 1);
    }

    #[test]
    fn when_2x2_opaque_square_then_one_contour_with_boundary_points() {
        // Arrange — top-left 2x2 block in a 4x4 grid
        #[rustfmt::skip]
        let mask = vec![
            true,  true,  false, false,
            true,  true,  false, false,
            false, false, false, false,
            false, false, false, false,
        ];

        // Act
        let contours = trace_contours(&mask, 4, 4);

        // Assert
        assert_eq!(contours.len(), 1);
        assert!(
            contours[0].len() >= 4,
            "boundary of 2x2 should have at least 4 points"
        );
    }

    #[test]
    fn when_l_shaped_region_then_one_contour_traces_concavity() {
        // Arrange — L-shape in a 4x4 grid
        #[rustfmt::skip]
        let mask = vec![
            true,  false, false, false,
            true,  false, false, false,
            true,  true,  true,  false,
            false, false, false, false,
        ];

        // Act
        let contours = trace_contours(&mask, 4, 4);

        // Assert
        assert_eq!(contours.len(), 1);
        assert!(
            contours[0].len() >= 5,
            "L-shape boundary should have at least 5 points"
        );
    }

    #[test]
    fn when_two_disconnected_regions_then_two_contours_returned() {
        // Arrange — two isolated pixels in a 5x1 strip
        #[rustfmt::skip]
        let mask = vec![true, false, false, false, true];

        // Act
        let contours = trace_contours(&mask, 5, 1);

        // Assert
        assert_eq!(contours.len(), 2);
    }

    #[test]
    fn when_fully_opaque_image_then_one_contour_on_border() {
        // Arrange — 4x4 all-true
        let mask = vec![true; 16];

        // Act
        let contours = trace_contours(&mask, 4, 4);

        // Assert
        assert_eq!(contours.len(), 1);
        // All boundary points should be on the image border (row 0/3 or col 0/3)
        for &(x, y) in &contours[0] {
            assert!(
                x == 0 || x == 3 || y == 0 || y == 3,
                "boundary point ({x}, {y}) is not on the image border"
            );
        }
    }

    #[test]
    fn when_triangle_region_traced_then_single_contour_returned() {
        // Arrange — triangle: tip at (2,0), base spans full row 2
        #[rustfmt::skip]
        let mask = vec![
            false, false, true,  false, false,
            false, true,  true,  true,  false,
            true,  true,  true,  true,  true,
        ];

        // Act
        let contours = trace_contours(&mask, 5, 3);

        // Assert
        assert_eq!(
            contours.len(),
            1,
            "triangle boundary should be one contour, got {}",
            contours.len()
        );
    }

    #[test]
    fn when_triangle_region_traced_then_contour_contains_all_corner_pixels() {
        // Arrange — same triangle mask
        #[rustfmt::skip]
        let mask = vec![
            false, false, true,  false, false,
            false, true,  true,  true,  false,
            true,  true,  true,  true,  true,
        ];

        // Act
        let contours = trace_contours(&mask, 5, 3);

        // Assert — tip and both base corners must be in the single contour
        let contour = &contours[0];
        assert!(contour.contains(&(2, 0)), "missing tip pixel (2,0)");
        assert!(contour.contains(&(0, 2)), "missing base corner (0,2)");
        assert!(contour.contains(&(4, 2)), "missing base corner (4,2)");
    }

    #[test]
    fn when_diamond_region_traced_then_single_contour_returned() {
        // Arrange — diamond in 5x5: all boundary pixels are diagonal-only connected
        #[rustfmt::skip]
        let mask = vec![
            false, false, true,  false, false,
            false, true,  true,  true,  false,
            true,  true,  true,  true,  true,
            false, true,  true,  true,  false,
            false, false, true,  false, false,
        ];

        // Act
        let contours = trace_contours(&mask, 5, 5);

        // Assert
        assert_eq!(
            contours.len(),
            1,
            "diamond boundary should be one contour, got {}",
            contours.len()
        );
        let contour = &contours[0];
        assert!(contour.contains(&(2, 0)), "missing top corner (2,0)");
        assert!(contour.contains(&(0, 2)), "missing left corner (0,2)");
        assert!(contour.contains(&(4, 2)), "missing right corner (4,2)");
        assert!(contour.contains(&(2, 4)), "missing bottom corner (2,4)");
    }
}
