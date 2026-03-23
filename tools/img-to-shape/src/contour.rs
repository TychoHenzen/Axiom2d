use std::collections::HashSet;

// Cardinal directions: E, S, W, N (clockwise order).
const DIRS: [(i32, i32); 4] = [(1, 0), (0, 1), (-1, 0), (0, -1)];

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

/// Trace the boundary starting at `(sx, sy)` using the left-hand rule with
/// 4-directional movement. Returns the ordered list of boundary pixels.
#[allow(clippy::too_many_arguments)]
fn trace_from(
    mask: &[bool],
    sx: u32,
    sy: u32,
    width: u32,
    height: u32,
    visited: &mut HashSet<(u32, u32)>,
) -> Vec<(u32, u32)> {
    let mut contour = vec![(sx, sy)];
    visited.insert((sx, sy));
    let mut x = sx as i32;
    let mut y = sy as i32;
    // Start heading east (we scan left-to-right, so east is the natural direction).
    let mut dir = 0usize;

    loop {
        // Try: left turn, straight, right turn, reverse.
        let attempts = [
            (dir + 3) % 4, // left
            dir,           // straight
            (dir + 1) % 4, // right
            (dir + 2) % 4, // reverse
        ];

        let mut found = false;
        for &try_dir in &attempts {
            let (dx, dy) = DIRS[try_dir];
            let nx = x + dx;
            let ny = y + dy;
            if !is_opaque(mask, nx, ny, width, height) {
                continue;
            }
            if !is_boundary(mask, nx as u32, ny as u32, width, height) {
                continue;
            }
            // Closing the loop — return to start.
            if nx as u32 == sx && ny as u32 == sy && contour.len() > 1 {
                return contour;
            }
            if visited.contains(&(nx as u32, ny as u32)) {
                continue;
            }
            visited.insert((nx as u32, ny as u32));
            contour.push((nx as u32, ny as u32));
            x = nx;
            y = ny;
            dir = try_dir;
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
/// Scans left-to-right, top-to-bottom for unvisited boundary pixels and traces
/// each connected boundary using the left-hand rule (4-directional, clockwise).
/// Returns one `Vec<(x, y)>` per connected boundary region.
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
            let contour = trace_from(mask, x, y, width, height, &mut visited);
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
}
