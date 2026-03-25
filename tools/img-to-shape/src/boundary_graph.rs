//! Planar graph face extraction for the img-to-shape pipeline.
//!
//! Builds a half-edge planar graph from pixel boundaries, extracts faces
//! (closed polygons), and simplifies shared edge chains. Adjacent shapes
//! share edges structurally — gaps are impossible.
//!
//! The `BoundaryGraph` is the central data model: chains are the shared
//! boundary segments between regions, and faces reference chains (possibly
//! reversed). Bezier fitting and coordinate transforms operate on the
//! graph's chain data before final conversion to separate shapes.

use std::collections::BTreeMap;

use crate::simplify;

type Vertex = (i32, i32);

fn dir_index(dx: i32, dy: i32) -> u8 {
    match (dx, dy) {
        (1, 0) => 0,  // right
        (0, 1) => 1,  // down
        (-1, 0) => 2, // left
        (0, -1) => 3, // up
        _ => panic!("non-axis-aligned edge ({dx},{dy})"),
    }
}

struct HalfEdge {
    origin: Vertex,
    dest: Vertex,
    twin: usize,
    next: usize,
    /// Region on the left side of this directed edge.
    left_region: i32,
}

/// A simplified chain of edges between two junction vertices, or a closed loop.
///
/// Open chains are stored in canonical direction (smaller endpoint first).
/// Closed loops are stored in face traversal order.
pub struct ChainData {
    /// Simplified point sequence in pixel coordinates.
    pub points: Vec<(f32, f32)>,
    /// Whether this chain borders transparent space (`region_id` == -1).
    /// Used in tests to verify graph correctness.
    #[cfg_attr(not(test), allow(dead_code))]
    pub is_external: bool,
    /// Whether this chain forms a closed loop (no junctions).
    pub is_closed: bool,
}

/// Reference to a chain within a face's boundary.
pub struct ChainRef {
    /// Index into `BoundaryGraph::chains`.
    pub chain_index: usize,
    /// If true, the face traverses this chain in reverse of the stored direction.
    pub reversed: bool,
}

/// A face of the planar graph.
pub struct Face {
    pub region_id: i32,
    /// Ordered chain references forming the face boundary.
    pub chain_refs: Vec<ChainRef>,
}

/// The planar boundary graph: chains + faces.
///
/// Chains are the shared boundary segments between regions. Each face
/// references chains (possibly reversed). This ensures adjacent faces
/// share exact geometry with no gaps.
pub struct BoundaryGraph {
    pub chains: Vec<ChainData>,
    pub faces: Vec<Face>,
}

impl BoundaryGraph {
    /// Reconstruct the simplified vertex polygon for a face from its chain refs.
    pub fn face_vertices(&self, face: &Face) -> Vec<(f32, f32)> {
        let mut pts = Vec::new();
        for cr in &face.chain_refs {
            let chain = &self.chains[cr.chain_index];
            if chain.is_closed {
                // Closed loop: just return points directly (no reversal needed).
                return chain.points.clone();
            }
            let ordered: Vec<(f32, f32)> = if cr.reversed {
                chain.points.iter().copied().rev().collect()
            } else {
                chain.points.clone()
            };
            if pts.is_empty() {
                pts.extend_from_slice(&ordered);
            } else {
                // Skip first point (same as last of previous chain).
                pts.extend_from_slice(&ordered[1..]);
            }
        }
        // Remove trailing duplicate if polygon wraps around.
        if pts.len() > 1 && pts.first() == pts.last() {
            pts.pop();
        }
        pts
    }
}

/// Build the planar graph and extract simplified region faces.
#[allow(clippy::too_many_lines)]
pub fn extract_region_faces(
    region_map: &[i32],
    width: u32,
    height: u32,
    epsilon: f32,
) -> BoundaryGraph {
    let w = width as i32;
    let h = height as i32;

    let region_at = |x: i32, y: i32| -> i32 {
        if x >= 0 && y >= 0 && x < w && y < h {
            region_map[(y * w + x) as usize]
        } else {
            -1
        }
    };

    // ── Step 1: Emit half-edge pairs ─────────────────────────────────
    let mut half_edges: Vec<HalfEdge> = Vec::new();
    // Lookup: (vertex, dir_index) → half_edge_id
    let mut outgoing: BTreeMap<(Vertex, u8), usize> = BTreeMap::new();

    let mut emit_pair = |a: Vertex, b: Vertex, left_ab: i32, left_ba: i32| {
        let id_ab = half_edges.len();
        let id_ba = id_ab + 1;
        let dx = b.0 - a.0;
        let dy = b.1 - a.1;
        let d_ab = dir_index(dx, dy);
        let d_ba = dir_index(-dx, -dy);
        half_edges.push(HalfEdge {
            origin: a,
            dest: b,
            twin: id_ba,
            next: usize::MAX,
            left_region: left_ab,
        });
        half_edges.push(HalfEdge {
            origin: b,
            dest: a,
            twin: id_ab,
            next: usize::MAX,
            left_region: left_ba,
        });
        outgoing.insert((a, d_ab), id_ab);
        outgoing.insert((b, d_ba), id_ba);
    };

    // Horizontal edges
    for gy in 0..=h {
        for gx in 0..w {
            let above = region_at(gx, gy - 1);
            let below = region_at(gx, gy);
            if above != below {
                // Rightward: left = above, Leftward: left = below
                emit_pair((gx, gy), (gx + 1, gy), above, below);
            }
        }
    }
    // Vertical edges
    for gx in 0..=w {
        for gy in 0..h {
            let left_px = region_at(gx - 1, gy);
            let right_px = region_at(gx, gy);
            if left_px != right_px {
                // Downward: left = right_px, Upward: left = left_px
                emit_pair((gx, gy), (gx, gy + 1), right_px, left_px);
            }
        }
    }

    if half_edges.is_empty() {
        return BoundaryGraph {
            chains: Vec::new(),
            faces: Vec::new(),
        };
    }

    // ── Step 2: Link `next` pointers ─────────────────────────────────
    // For half-edge h arriving at vertex v, the next half-edge in the
    // LEFT face is found by scanning CW (in Y-down) from the twin
    // direction.
    #[allow(clippy::needless_range_loop)]
    for i in 0..half_edges.len() {
        let dest = half_edges[i].dest;
        let dx = dest.0 - half_edges[i].origin.0;
        let dy = dest.1 - half_edges[i].origin.1;
        let incoming_dir = dir_index(dx, dy);
        let reverse_dir = (incoming_dir + 2) % 4;

        // Scan CCW in Y-down (= increasing DIR index) from just after twin.
        let mut found = false;
        for offset in 1..=4u8 {
            let try_dir = (reverse_dir + offset) % 4;
            if let Some(&he_id) = outgoing.get(&(dest, try_dir)) {
                half_edges[i].next = he_id;
                found = true;
                break;
            }
        }
        if !found {
            half_edges[i].next = half_edges[i].twin;
        }
    }

    // ── Step 3: Extract raw faces ────────────────────────────────────
    let mut visited = vec![false; half_edges.len()];
    let mut raw_faces: Vec<(i32, Vec<Vertex>, Vec<usize>)> = Vec::new();

    for start in 0..half_edges.len() {
        if visited[start] {
            continue;
        }

        let region = half_edges[start].left_region;
        let mut verts: Vec<Vertex> = Vec::new();
        let mut he_ids: Vec<usize> = Vec::new();
        let mut cur = start;

        loop {
            if visited[cur] {
                break;
            }
            visited[cur] = true;
            verts.push(half_edges[cur].origin);
            he_ids.push(cur);
            cur = half_edges[cur].next;
            if cur == start {
                break;
            }
        }

        if verts.len() >= 3 {
            raw_faces.push((region, verts, he_ids));
        }
    }

    // ── Step 4: Build chains and faces ───────────────────────────────
    // Identify junction vertices (degree != 2 in the undirected graph).
    let mut vertex_degree: BTreeMap<Vertex, u32> = BTreeMap::new();
    for he in &half_edges {
        if he.origin < he.dest {
            *vertex_degree.entry(he.origin).or_insert(0) += 1;
            *vertex_degree.entry(he.dest).or_insert(0) += 1;
        }
    }

    let is_junction = |v: Vertex| -> bool { vertex_degree.get(&v).copied().unwrap_or(0) != 2 };

    #[allow(clippy::items_after_statements)]
    type ChainKey = (Vertex, Vertex, Vertex);
    let mut chain_cache: BTreeMap<ChainKey, usize> = BTreeMap::new();
    let mut chains: Vec<ChainData> = Vec::new();
    let mut faces: Vec<Face> = Vec::new();

    for (region, verts, he_ids) in &raw_faces {
        let n = verts.len();

        // Find junction indices in this face's vertex list.
        let junction_indices: Vec<usize> = (0..n).filter(|&i| is_junction(verts[i])).collect();

        if junction_indices.is_empty() {
            // No junctions — entire face is one closed loop.
            let pts: Vec<(f32, f32)> = verts.iter().map(|&(x, y)| (x as f32, y as f32)).collect();
            let simplified = if pts.len() < 4 {
                pts
            } else {
                simplify::rdp_simplify_closed(&pts, epsilon)
            };

            if simplified.len() < 3 {
                continue;
            }

            // Determine external status from any half-edge's twin.
            let twin_region = half_edges[half_edges[he_ids[0]].twin].left_region;
            let is_external = twin_region < 0 || *region < 0;

            let chain_idx = chains.len();
            chains.push(ChainData {
                points: simplified,
                is_external,
                is_closed: true,
            });
            faces.push(Face {
                region_id: *region,
                chain_refs: vec![ChainRef {
                    chain_index: chain_idx,
                    reversed: false,
                }],
            });
        } else {
            // Split face boundary into chains between consecutive junctions.
            let mut chain_refs: Vec<ChainRef> = Vec::new();
            let jcount = junction_indices.len();
            let mut any_chain_too_short = false;

            for ji in 0..jcount {
                let chain_start_idx = junction_indices[ji];
                let chain_end_idx = junction_indices[(ji + 1) % jcount];

                // Collect chain vertices from chain_start to chain_end (wrapping).
                let mut chain_verts: Vec<Vertex> = Vec::new();
                let mut chain_he_ids: Vec<usize> = Vec::new();
                let mut idx = chain_start_idx;
                loop {
                    chain_verts.push(verts[idx]);
                    chain_he_ids.push(he_ids[idx]);
                    if idx == chain_end_idx && chain_verts.len() > 1 {
                        break;
                    }
                    idx = (idx + 1) % n;
                    if chain_verts.len() > n + 1 {
                        break; // safety
                    }
                }

                let start_v = chain_verts[0];
                let end_v = *chain_verts.last().expect("non-empty");

                // Canonical key: smaller endpoint first, plus the second
                // vertex from that direction to disambiguate parallel chains.
                let second_from_start = if chain_verts.len() >= 2 {
                    chain_verts[1]
                } else {
                    start_v
                };
                let second_from_end = if chain_verts.len() >= 2 {
                    chain_verts[chain_verts.len() - 2]
                } else {
                    end_v
                };
                let key: ChainKey = if start_v <= end_v {
                    (start_v, end_v, second_from_start)
                } else {
                    (end_v, start_v, second_from_end)
                };

                let need_reverse = start_v > end_v;

                let chain_idx = if let Some(&cached_idx) = chain_cache.get(&key) {
                    cached_idx
                } else {
                    // Always store in canonical direction (key.0 → key.1).
                    let canonical_pts: Vec<(f32, f32)> = if start_v <= end_v {
                        chain_verts
                            .iter()
                            .map(|&(x, y)| (x as f32, y as f32))
                            .collect()
                    } else {
                        chain_verts
                            .iter()
                            .rev()
                            .map(|&(x, y)| (x as f32, y as f32))
                            .collect()
                    };
                    let simplified = if canonical_pts.len() <= 2 {
                        canonical_pts
                    } else {
                        simplify::rdp_open(&canonical_pts, epsilon)
                    };

                    // Determine external status from any half-edge's twin.
                    let twin_region = half_edges[half_edges[chain_he_ids[0]].twin].left_region;
                    let is_external = twin_region < 0 || *region < 0;

                    let new_idx = chains.len();
                    chains.push(ChainData {
                        points: simplified,
                        is_external,
                        is_closed: false,
                    });
                    chain_cache.insert(key, new_idx);
                    new_idx
                };

                if chains[chain_idx].points.len() < 2 {
                    any_chain_too_short = true;
                    break;
                }

                chain_refs.push(ChainRef {
                    chain_index: chain_idx,
                    reversed: need_reverse,
                });
            }

            if any_chain_too_short || chain_refs.is_empty() {
                continue;
            }

            faces.push(Face {
                region_id: *region,
                chain_refs,
            });
        }
    }

    BoundaryGraph { chains, faces }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn signed_area(pts: &[(f32, f32)]) -> f32 {
        let n = pts.len();
        let mut sum = 0.0_f32;
        for i in 0..n {
            let (x0, y0) = pts[i];
            let (x1, y1) = pts[(i + 1) % n];
            sum += x0 * y1 - x1 * y0;
        }
        sum * 0.5
    }

    #[test]
    fn when_single_pixel_then_one_face_with_negative_area() {
        // 3x3, center pixel = region 0, rest transparent.
        let mut region_map = vec![-1i32; 9];
        region_map[4] = 0;

        let graph = extract_region_faces(&region_map, 3, 3, 0.5);

        let region0: Vec<_> = graph.faces.iter().filter(|f| f.region_id == 0).collect();
        assert_eq!(
            region0.len(),
            1,
            "single pixel should produce one face for region 0, got {} total faces ({} for region 0)",
            graph.faces.len(),
            region0.len()
        );

        let verts = graph.face_vertices(region0[0]);
        let area = signed_area(&verts);
        assert!(
            area < 0.0,
            "enclosed face should have negative area, got {area}"
        );
        assert_eq!(verts.len(), 4, "unit square has 4 vertices");
    }

    #[test]
    fn when_two_adjacent_regions_then_shared_chain() {
        // 4x1: region 0 left, region 1 right.
        let region_map = vec![0, 0, 1, 1];

        let graph = extract_region_faces(&region_map, 4, 1, 0.5);

        let f0: Vec<_> = graph.faces.iter().filter(|f| f.region_id == 0).collect();
        let f1: Vec<_> = graph.faces.iter().filter(|f| f.region_id == 1).collect();
        assert_eq!(f0.len(), 1);
        assert_eq!(f1.len(), 1);

        // Both faces should reference at least one common chain index.
        let f0_chains: std::collections::BTreeSet<usize> =
            f0[0].chain_refs.iter().map(|cr| cr.chain_index).collect();
        let f1_chains: std::collections::BTreeSet<usize> =
            f1[0].chain_refs.iter().map(|cr| cr.chain_index).collect();
        let shared: Vec<_> = f0_chains.intersection(&f1_chains).collect();
        assert!(
            !shared.is_empty(),
            "adjacent regions should share at least one chain"
        );

        // The shared chain should be traversed in opposite directions.
        for &chain_idx in &shared {
            let f0_ref = f0[0]
                .chain_refs
                .iter()
                .find(|cr| cr.chain_index == *chain_idx)
                .unwrap();
            let f1_ref = f1[0]
                .chain_refs
                .iter()
                .find(|cr| cr.chain_index == *chain_idx)
                .unwrap();
            assert_ne!(
                f0_ref.reversed, f1_ref.reversed,
                "shared chain should be traversed in opposite directions"
            );
        }

        // Verify shared edge vertices match.
        let s0 = graph.face_vertices(f0[0]);
        let s1 = graph.face_vertices(f1[0]);
        let has_edge = |pts: &[(f32, f32)], a: (f32, f32), b: (f32, f32)| -> bool {
            let n = pts.len();
            (0..n).any(|i| pts[i] == a && pts[(i + 1) % n] == b)
        };

        let fwd = has_edge(&s0, (2.0, 0.0), (2.0, 1.0));
        let rev = has_edge(&s0, (2.0, 1.0), (2.0, 0.0));
        let fwd1 = has_edge(&s1, (2.0, 0.0), (2.0, 1.0));
        let rev1 = has_edge(&s1, (2.0, 1.0), (2.0, 0.0));

        assert!(
            (fwd && rev1) || (rev && fwd1),
            "shared edge should appear forward in one face, reversed in the other.\n\
             f0={s0:?}\nf1={s1:?}"
        );
    }

    #[test]
    fn when_four_regions_then_no_gaps_at_center_junction() {
        // 2x2 image: each pixel is a different region (0,1,2,3).
        let region_map = vec![0, 1, 2, 3];
        let graph = extract_region_faces(&region_map, 2, 2, 0.5);

        let region_faces: Vec<Vec<_>> = (0..4)
            .map(|r| graph.faces.iter().filter(|f| f.region_id == r).collect())
            .collect();

        for r in 0..4 {
            assert_eq!(
                region_faces[r].len(),
                1,
                "region {r} should have 1 face, got {}",
                region_faces[r].len()
            );
        }

        // All 4 faces should contain the center point (1.0, 1.0).
        for r in 0..4 {
            let pts = graph.face_vertices(region_faces[r][0]);
            assert!(
                pts.contains(&(1.0, 1.0)),
                "region {r} face should contain center (1,1), got {pts:?}"
            );
        }

        // Each face should be a unit square (4 vertices).
        for r in 0..4 {
            let pts = graph.face_vertices(region_faces[r][0]);
            assert_eq!(
                pts.len(),
                4,
                "region {r} should be a unit square, got {pts:?}",
            );
        }

        // Center (1,1) should appear in all 4 faces.
        let all_verts: Vec<_> = graph
            .faces
            .iter()
            .filter(|f| f.region_id >= 0)
            .flat_map(|f| graph.face_vertices(f))
            .collect();
        let center_count = all_verts.iter().filter(|&&v| v == (1.0, 1.0)).count();
        assert_eq!(center_count, 4, "center should appear in all 4 faces");
    }

    #[test]
    fn when_diagonal_staircase_then_segments_not_fragmented() {
        // 4x4: region 0 = lower-left triangle (row >= col), region 1 = upper-right.
        #[rustfmt::skip]
        let region_map = vec![
            0, 1, 1, 1,
            0, 0, 1, 1,
            0, 0, 0, 1,
            0, 0, 0, 0,
        ];

        let graph = extract_region_faces(&region_map, 4, 4, 0.5);

        let f0: Vec<_> = graph.faces.iter().filter(|f| f.region_id == 0).collect();
        let f1: Vec<_> = graph.faces.iter().filter(|f| f.region_id == 1).collect();
        assert_eq!(f0.len(), 1, "region 0 should have 1 face");
        assert_eq!(f1.len(), 1, "region 1 should have 1 face");

        let s0 = graph.face_vertices(f0[0]);
        let s1 = graph.face_vertices(f1[0]);

        // Staircase midpoints should appear in both faces.
        let has_vertex = |pts: &[(f32, f32)], v: (f32, f32)| pts.contains(&v);

        assert!(
            has_vertex(&s0, (2.0, 1.0)) && has_vertex(&s1, (2.0, 1.0)),
            "staircase vertex (2,1) should appear in both faces"
        );
    }

    #[test]
    fn when_closed_loop_island_then_chain_is_closed() {
        // 5x5: region 0 border, region 1 center island.
        #[rustfmt::skip]
        let region_map = vec![
            0, 0, 0, 0, 0,
            0, 1, 1, 1, 0,
            0, 1, 1, 1, 0,
            0, 1, 1, 1, 0,
            0, 0, 0, 0, 0,
        ];

        let graph = extract_region_faces(&region_map, 5, 5, 0.5);

        let f1: Vec<_> = graph.faces.iter().filter(|f| f.region_id == 1).collect();
        assert_eq!(f1.len(), 1, "island should have 1 face");

        // The island face should have exactly 1 chain ref, and that chain should be closed.
        assert_eq!(
            f1[0].chain_refs.len(),
            1,
            "island face should reference 1 chain"
        );
        let chain = &graph.chains[f1[0].chain_refs[0].chain_index];
        assert!(chain.is_closed, "island chain should be closed");
        // The island borders region 0, not transparent (-1), so is_external = false.
        assert!(
            !chain.is_external,
            "island chain should NOT be external (borders region 0, not transparent)"
        );
    }

    #[test]
    fn when_single_region_then_chain_is_external() {
        // 3x3: all region 0.
        let region_map = vec![0i32; 9];

        let graph = extract_region_faces(&region_map, 3, 3, 0.5);

        let f0: Vec<_> = graph.faces.iter().filter(|f| f.region_id == 0).collect();
        assert_eq!(f0.len(), 1, "single region should have 1 face");

        // The boundary chain borders transparent (-1), so is_external = true.
        let chain = &graph.chains[f0[0].chain_refs[0].chain_index];
        assert!(chain.is_external, "boundary chain should be external");
        assert!(chain.is_closed, "single region boundary should be closed");
    }
}
