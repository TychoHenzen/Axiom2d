use std::collections::BTreeMap;

type Vertex = (i32, i32);
type EdgeKey = (Vertex, Vertex);

/// 4 axis-aligned directions in clockwise order (image-space, Y-down):
/// Right(1,0), Down(0,1), Left(-1,0), Up(0,-1)
const CW: [Vertex; 4] = [(1, 0), (0, 1), (-1, 0), (0, -1)];

fn opaque(mask: &[bool], x: i32, y: i32, w: i32, h: i32) -> bool {
    x >= 0 && y >= 0 && x < w && y < h && mask[y as usize * w as usize + x as usize]
}

/// Collect directed boundary edges from all opaque pixels.
///
/// For each opaque pixel at (px, py), emit a directed edge along any side
/// where the adjacent pixel is transparent (or out-of-bounds). Edge direction
/// is CCW around the opaque region in image space (Y-down):
/// - top edge:    left → right   (x0,y0) → (x1,y0)
/// - right edge:  top → bottom   (x1,y0) → (x1,y1)
/// - bottom edge: right → left   (x1,y1) → (x0,y1)
/// - left edge:   bottom → top   (x0,y1) → (x0,y0)
///
/// Returns a multi-map: vertex → sorted list of outgoing destination vertices.
fn collect_edges(mask: &[bool], w: i32, h: i32) -> BTreeMap<Vertex, Vec<Vertex>> {
    let mut outgoing: BTreeMap<Vertex, Vec<Vertex>> = BTreeMap::new();

    for py in 0..h {
        for px in 0..w {
            if !opaque(mask, px, py, w, h) {
                continue;
            }
            let (x0, y0, x1, y1) = (px, py, px + 1, py + 1);

            // Top edge: neighbor above is transparent
            if !opaque(mask, px, py - 1, w, h) {
                outgoing.entry((x0, y0)).or_default().push((x1, y0));
            }
            // Right edge: neighbor to right is transparent
            if !opaque(mask, px + 1, py, w, h) {
                outgoing.entry((x1, y0)).or_default().push((x1, y1));
            }
            // Bottom edge: neighbor below is transparent
            if !opaque(mask, px, py + 1, w, h) {
                outgoing.entry((x1, y1)).or_default().push((x0, y1));
            }
            // Left edge: neighbor to left is transparent
            if !opaque(mask, px - 1, py, w, h) {
                outgoing.entry((x0, y1)).or_default().push((x0, y0));
            }
        }
    }

    outgoing
}

/// Pick the outgoing edge that makes the tightest right turn from the
/// incoming direction. This resolves saddle-point vertices where two
/// boundary chains cross the same grid intersection.
///
/// Incoming direction = `cur - prev`. We rotate 90° clockwise (right turn)
/// and try each candidate; the one closest to that preferred direction wins.
fn pick_right_turn(prev: Vertex, cur: Vertex, candidates: &[Vertex]) -> Vertex {
    debug_assert!(!candidates.is_empty());
    if candidates.len() == 1 {
        return candidates[0];
    }

    let (idx, idy) = (cur.0 - prev.0, cur.1 - prev.1);

    let incoming_slot = CW
        .iter()
        .position(|&d| d == (idx, idy))
        .expect("edge direction must be axis-aligned");

    // Try right turn first (CW+1), then straight (CW+2), then left (CW+3).
    // Skip U-turn (CW+0) — only used as last resort.
    for offset in 1..=4 {
        let preferred = CW[(incoming_slot + offset) % 4];
        let target = (cur.0 + preferred.0, cur.1 + preferred.1);
        if candidates.contains(&target) {
            return target;
        }
    }

    // Fallback (should not happen with valid boundary edges).
    candidates[0]
}

/// Build a next-edge map that resolves saddle points via right-turn rule.
///
/// Returns: `(prev, cur) → next` so the tracer can walk the chain.
fn build_next_map(outgoing: &BTreeMap<Vertex, Vec<Vertex>>) -> BTreeMap<EdgeKey, Vertex> {
    let mut next_map: BTreeMap<EdgeKey, Vertex> = BTreeMap::new();

    // For each edge (a → b), resolve the next vertex c after b.
    for (&a, a_outs) in outgoing {
        for &b in a_outs {
            if let Some(b_outs) = outgoing.get(&b) {
                let c = pick_right_turn(a, b, b_outs);
                next_map.insert((a, b), c);
            }
        }
    }

    next_map
}

/// Trace closed contours along pixel edges from a binary mask.
///
/// Uses directed boundary edges emitted per opaque pixel, chained into closed
/// polygons via right-turn disambiguation at saddle-point vertices. Coordinates
/// are grid intersections in the range `[0, width] × [0, height]` (f32).
///
/// Winding is CCW in image space (Y-down). After the `pixel_to_engine` Y-flip
/// applied downstream, this becomes CW — which is what lyon expects for outer
/// contours with its default fill rule.
pub fn trace_contours(mask: &[bool], width: u32, height: u32) -> Vec<Vec<(f32, f32)>> {
    let w = width as i32;
    let h = height as i32;

    let outgoing = collect_edges(mask, w, h);
    if outgoing.is_empty() {
        return Vec::new();
    }

    let next_map = build_next_map(&outgoing);

    let mut used: std::collections::HashSet<EdgeKey> = std::collections::HashSet::new();
    let mut contours: Vec<Vec<(f32, f32)>> = Vec::new();

    // Iterate edges in deterministic (BTreeMap) order.
    let seeds: Vec<EdgeKey> = outgoing
        .iter()
        .flat_map(|(&from, tos)| tos.iter().map(move |&to| (from, to)))
        .collect();

    for seed_edge in seeds {
        if used.contains(&seed_edge) {
            continue;
        }

        let mut poly: Vec<(f32, f32)> = Vec::new();
        let (mut prev, mut cur) = seed_edge;

        loop {
            if !used.insert((prev, cur)) {
                break;
            }
            poly.push((prev.0 as f32, prev.1 as f32));

            match next_map.get(&(prev, cur)) {
                Some(&next) => {
                    prev = cur;
                    cur = next;
                }
                None => break,
            }
        }

        if poly.len() >= 3 {
            contours.push(poly);
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
    fn when_single_opaque_pixel_then_one_square_contour() {
        // Arrange — center pixel of 3x3 mask
        let mut mask = vec![false; 9];
        mask[4] = true; // (1,1)

        // Act
        let contours = trace_contours(&mask, 3, 3);

        // Assert — grid-edge polygon has exactly 4 corners
        assert_eq!(contours.len(), 1);
        assert_eq!(contours[0].len(), 4);
    }

    #[test]
    fn when_2x2_opaque_square_then_one_contour_with_4_corners() {
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
        // 2x2 pixel square → 8 grid-edge vertices (collinear ones on straight
        // edges are expected; RDP simplification removes them downstream).
        assert!(
            contours[0].len() >= 4,
            "2x2 square should have at least 4 vertices, got {}",
            contours[0].len()
        );
    }

    #[test]
    fn when_l_shaped_region_then_one_contour_with_concavity() {
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

        // Assert — L-shape has 6 corners (a rectangle with one corner notched)
        assert_eq!(contours.len(), 1);
        assert!(
            contours[0].len() >= 6,
            "L-shape has at least 6 corner vertices, got {}",
            contours[0].len()
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
        // All vertices should be on the image border (x=0 or 4, y=0 or 4)
        for &(x, y) in &contours[0] {
            assert!(
                x == 0.0 || x == 4.0 || y == 0.0 || y == 4.0,
                "vertex ({x}, {y}) is not on the image border"
            );
        }
    }

    #[test]
    fn when_diagonal_touching_pixels_then_two_separate_contours() {
        // Arrange — saddle case: NW and SE pixels opaque, touching at (1,1)
        #[rustfmt::skip]
        let mask = vec![
            true,  false,
            false, true,
        ];

        // Act
        let contours = trace_contours(&mask, 2, 2);

        // Assert — must produce two separate contours, not one broken chain
        assert_eq!(
            contours.len(),
            2,
            "diagonal pixels must form two separate contours"
        );
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
    fn when_diamond_region_traced_then_single_contour_with_concavities() {
        // Arrange — diamond in 5x5
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
        assert_eq!(contours.len(), 1);
        // Diamond has staircase edges, more than 4 vertices
        assert!(
            contours[0].len() > 4,
            "diamond should have more than 4 vertices, got {}",
            contours[0].len()
        );
    }

    #[test]
    fn when_contour_traced_then_polygon_is_closed() {
        // Arrange — 3x3 solid block
        let mask = vec![true; 9];

        // Act
        let contours = trace_contours(&mask, 3, 3);

        // Assert — first and last point should connect back (polygon is closed)
        assert_eq!(contours.len(), 1);
        let poly = &contours[0];
        assert!(poly.len() >= 3);
        // The polygon should form a closed loop — verify by checking it's a
        // valid rectangle: 4 corners at (0,0), (3,0), (3,3), (0,3)
        // Grid-edge tracing produces vertices at every pixel boundary
        // intersection. A 3x3 square has 12 edge vertices (4 corners + 2
        // intermediates per side). Check corner vertices are present.
        let corners: Vec<(f32, f32)> = vec![(0.0, 0.0), (3.0, 0.0), (3.0, 3.0), (0.0, 3.0)];
        for &pt in &corners {
            assert!(
                poly.contains(&pt),
                "missing corner vertex {pt:?} in {poly:?}"
            );
        }
    }
}
