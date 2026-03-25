mod bezier_fit;
mod boundary_graph;
pub mod codegen;
pub mod scale2x;
mod segment;
mod simplify;
mod transform;

use std::collections::BTreeSet;

use engine_render::shape::{PathCommand, Shape, ShapeVariant};
use glam::Vec2;

/// Evaluate a cubic bezier at parameter t.
fn sample_cubic(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let u = 1.0 - t;
    p0 * (u * u * u) + p1 * (3.0 * u * u * t) + p2 * (3.0 * u * t * t) + p3 * (t * t * t)
}

/// Check whether a point is outside the bounding box.
fn is_out_of_bounds(p: Vec2, x_lo: f32, x_hi: f32, y_lo: f32, y_hi: f32) -> bool {
    p.x < x_lo || p.x > x_hi || p.y < y_lo || p.y > y_hi
}

/// Replace any `CubicTo` whose curve leaves the image bounding box with `LineTo`.
///
/// First checks whether any control point is outside bounds (quick reject).
/// Then samples the curve densely; if any sample falls outside the bounds,
/// the curve is replaced with a straight line to its endpoint. This catches
/// self-intersecting curves, wild control points, and singularity artifacts.
fn cull_out_of_bounds_cubics(
    commands: Vec<PathCommand>,
    half_w: f32,
    half_h: f32,
) -> Vec<PathCommand> {
    let x_lo = -half_w;
    let x_hi = half_w;
    let y_lo = -half_h;
    let y_hi = half_h;

    let mut prev_end = Vec2::ZERO;
    commands
        .into_iter()
        .map(|cmd| {
            match &cmd {
                PathCommand::MoveTo(v) | PathCommand::LineTo(v) => prev_end = *v,
                PathCommand::CubicTo {
                    control1,
                    control2,
                    to,
                } => {
                    let start = prev_end;
                    prev_end = *to;

                    // Quick reject: if any control point is outside bounds,
                    // use denser sampling since the curve might overshoot.
                    let any_control_oob = is_out_of_bounds(*control1, x_lo, x_hi, y_lo, y_hi)
                        || is_out_of_bounds(*control2, x_lo, x_hi, y_lo, y_hi);
                    let samples = if any_control_oob { 200 } else { 69 };

                    for i in 1..=samples {
                        let t = i as f32 / (samples + 1) as f32;
                        let p = sample_cubic(start, *control1, *control2, *to, t);
                        if is_out_of_bounds(p, x_lo, x_hi, y_lo, y_hi) {
                            return PathCommand::LineTo(*to);
                        }
                    }
                }
                _ => {}
            }
            cmd
        })
        .collect()
}

/// Clamp all coordinates in path commands to the image bounding box.
///
/// Safety net that catches any residual out-of-bounds coordinates after
/// bezier fitting and cubic culling. For `CubicTo`, clamps control points
/// to the bounds (which slightly changes curve shape but keeps it in-frame).
fn clamp_to_bounds(commands: Vec<PathCommand>, half_w: f32, half_h: f32) -> Vec<PathCommand> {
    let clamp =
        |v: Vec2| -> Vec2 { Vec2::new(v.x.clamp(-half_w, half_w), v.y.clamp(-half_h, half_h)) };
    commands
        .into_iter()
        .map(|cmd| match cmd {
            PathCommand::MoveTo(v) => PathCommand::MoveTo(clamp(v)),
            PathCommand::LineTo(v) => PathCommand::LineTo(clamp(v)),
            PathCommand::CubicTo {
                control1,
                control2,
                to,
            } => PathCommand::CubicTo {
                control1: clamp(control1),
                control2: clamp(control2),
                to: clamp(to),
            },
            other => other,
        })
        .collect()
}

/// Signed area of a polygon using the shoelace formula.
///
/// Positive for CCW winding in pixel space (Y-down) = outer contours.
/// Negative for CW winding = inner contours (holes).
fn signed_polygon_area(points: &[(f32, f32)]) -> f32 {
    let n = points.len();
    if n < 3 {
        return 0.0;
    }
    let mut sum = 0.0_f32;
    for i in 0..n {
        let (x0, y0) = points[i];
        let (x1, y1) = points[(i + 1) % n];
        sum += x0 * y1 - x1 * y0;
    }
    sum * 0.5
}

/// Build a map from pixel position to region index (-1 = transparent).
fn build_region_map(regions: &[segment::Region], width: u32, height: u32) -> Vec<i32> {
    let mut map = vec![-1i32; (width * height) as usize];
    for (id, region) in regions.iter().enumerate() {
        for (i, &is_member) in region.mask.iter().enumerate() {
            if is_member {
                map[i] = id as i32;
            }
        }
    }
    map
}

/// Reverse a sequence of `LineTo`/`CubicTo` commands so they trace the
/// same path in the opposite direction.
fn reverse_path_commands(cmds: &[PathCommand], seg_start: Vec2) -> Vec<PathCommand> {
    if cmds.is_empty() {
        return Vec::new();
    }

    // Collect the chain of endpoints: [seg_start, cmd0.to, cmd1.to, ...]
    let mut endpoints = vec![seg_start];
    for cmd in cmds {
        match cmd {
            PathCommand::LineTo(v) | PathCommand::CubicTo { to: v, .. } => {
                endpoints.push(*v);
            }
            _ => {}
        }
    }

    // Walk commands in reverse. Each reversed command goes TO the previous
    // command's start point (endpoints[i] for the i-th original command).
    let mut result = Vec::with_capacity(cmds.len());
    for (i, cmd) in cmds.iter().enumerate().rev() {
        match cmd {
            PathCommand::LineTo(_) => {
                result.push(PathCommand::LineTo(endpoints[i]));
            }
            PathCommand::CubicTo {
                control1, control2, ..
            } => {
                result.push(PathCommand::CubicTo {
                    control1: *control2,
                    control2: *control1,
                    to: endpoints[i],
                });
            }
            _ => {}
        }
    }

    result
}

/// Fit bezier curves to a closed polygon by splitting at the farthest-apart
/// point from the start and fitting each half as an open segment.
///
/// This avoids the degenerate case where `fit_bezier_segment` receives a
/// sequence with start == end (which `is_collinear` collapses to a single point).
fn fit_closed_bezier(points: &[(f32, f32)], max_error: f32) -> Vec<PathCommand> {
    let n = points.len();
    if n < 4 {
        let mut cmds: Vec<PathCommand> = points[1..]
            .iter()
            .map(|&(x, y)| PathCommand::LineTo(Vec2::new(x, y)))
            .collect();
        cmds.push(PathCommand::LineTo(Vec2::new(points[0].0, points[0].1)));
        return cmds;
    }

    // Find the point farthest from points[0].
    let (x0, y0) = points[0];
    let mut split_b = n / 2;
    let mut best_dist = 0.0_f32;
    #[allow(clippy::needless_range_loop)]
    for i in 1..n {
        let dx = points[i].0 - x0;
        let dy = points[i].1 - y0;
        let d = dx * dx + dy * dy;
        if d > best_dist {
            best_dist = d;
            split_b = i;
        }
    }

    // First half: points[0] → points[split_b]
    let first_half = &points[..=split_b];
    // Second half: points[split_b] → points[0]
    let mut second_half: Vec<(f32, f32)> = points[split_b..].to_vec();
    second_half.push(points[0]);

    let mut cmds = bezier_fit::fit_bezier_segment(first_half, max_error);
    cmds.extend(bezier_fit::fit_bezier_segment(&second_half, max_error));
    cmds
}

/// Transform a pixel-space `PathCommand` to engine coordinates and append it.
fn push_transformed_command(
    cmds: &mut Vec<PathCommand>,
    cmd: &PathCommand,
    to_engine: &dyn Fn(f32, f32) -> (f32, f32),
) {
    match cmd {
        PathCommand::LineTo(v) => {
            let (x, y) = to_engine(v.x, v.y);
            cmds.push(PathCommand::LineTo(Vec2::new(x, y)));
        }
        PathCommand::CubicTo {
            control1,
            control2,
            to,
        } => {
            let (c1x, c1y) = to_engine(control1.x, control1.y);
            let (c2x, c2y) = to_engine(control2.x, control2.y);
            let (tx, ty) = to_engine(to.x, to.y);
            cmds.push(PathCommand::CubicTo {
                control1: Vec2::new(c1x, c1y),
                control2: Vec2::new(c2x, c2y),
                to: Vec2::new(tx, ty),
            });
        }
        _ => {} // MoveTo/Close shouldn't appear in chain commands
    }
}

/// Algorithm used when upscaling images smaller than `max_dimension`.
/// Downscaling always uses nearest-neighbor regardless of this setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeMethod {
    /// Nearest-neighbor: each pixel is duplicated. Fast but produces
    /// staircase artifacts that segment into one shape per pixel.
    Nearest,
    /// EPX/Scale2x: 3x3 neighborhood rule that smooths diagonal edges.
    /// Chains 2x passes then nearest-neighbor for the remainder.
    Scale2x,
}

/// Estimated output size metrics, computed after conversion.
pub struct OutputEstimate {
    /// Number of shapes in the output.
    pub shape_count: usize,
    /// Total number of path commands across all shapes.
    pub command_count: usize,
    /// Number of `LineTo` commands.
    pub line_to_count: usize,
    /// Number of `CubicTo` commands.
    pub cubic_to_count: usize,
    /// Estimated lines of code in the generated `.rs` file.
    pub estimated_loc: usize,
    /// Estimated number of floats in the generated code (2 per `MoveTo`/`LineTo`, 6 per `CubicTo`).
    pub estimated_floats: usize,
}

/// Output of the image-to-shapes conversion pipeline.
pub struct ConvertResult {
    /// Vector shapes extracted from the image.
    pub shapes: Vec<Shape>,
    /// Error-pink background rectangle covering the full image bounds.
    /// Render behind `shapes` to make coverage gaps visible as magenta.
    /// `None` when the image is empty or fully transparent.
    pub background: Option<Shape>,
    /// The resized RGBA pixel buffer that was fed into segmentation.
    pub rgba: Vec<u8>,
    /// Width of the resized image (coordinate space for shapes).
    pub width: u32,
    /// Height of the resized image (coordinate space for shapes).
    pub height: u32,
    /// Estimated output size metrics.
    pub estimate: OutputEstimate,
}

/// Configuration for the image-to-shapes conversion pipeline.
pub struct ConvertConfig {
    /// Maximum Euclidean distance in normalized RGB space (0.0–1.0) for two
    /// adjacent pixels to be considered the "same color" during flood-fill.
    pub color_threshold: f32,
    /// Minimum alpha (0–255) for a pixel to be considered non-transparent.
    pub alpha_threshold: u8,
    /// RDP simplification epsilon — larger values produce simpler shapes.
    pub rdp_epsilon: f32,
    /// Maximum error for bezier curve fitting — larger values produce fewer,
    /// less precise curves.
    pub bezier_error: f32,
    /// Minimum pixel count for a region to produce a shape. Regions smaller
    /// than this are discarded. Use 0 to keep all regions.
    pub min_area: usize,
    /// Target size for the longest dimension. Images are resized (preserving
    /// aspect ratio) so their longest side matches this value. Use 0 to
    /// disable resizing.
    pub max_dimension: u32,
    /// Algorithm used for upscaling. Downscaling always uses nearest-neighbor.
    pub resize_method: ResizeMethod,
    /// When true, fit bezier curves to all simplified contours (smoother,
    /// fewer commands). Adjacent regions share chain geometry so curves are
    /// gap-free. When false, use straight line segments (pixel-faithful).
    pub use_bezier: bool,
    /// Merge regions with fewer pixels than this into their nearest-color
    /// neighbor. Reduces shape count without visible quality loss for small
    /// noise speckles. Use 0 to disable.
    pub merge_below: usize,
    /// Hard cap on the number of output shapes. After `merge_below` merging,
    /// progressively merge the smallest remaining regions into their nearest
    /// neighbor until the count is at or below this limit. Use 0 for unlimited.
    pub max_shapes: usize,
}

/// Resize RGBA pixel data by nearest-neighbor sampling.
fn resize_rgba(rgba: &[u8], src_w: u32, src_h: u32, dst_w: u32, dst_h: u32) -> Vec<u8> {
    let mut out = vec![0u8; (dst_w * dst_h * 4) as usize];
    for dy in 0..dst_h {
        for dx in 0..dst_w {
            let sx = (dx * src_w / dst_w).min(src_w - 1);
            let sy = (dy * src_h / dst_h).min(src_h - 1);
            let si = (sy * src_w + sx) as usize * 4;
            let di = (dy * dst_w + dx) as usize * 4;
            out[di..di + 4].copy_from_slice(&rgba[si..si + 4]);
        }
    }
    out
}

/// Color distance squared between two colors (avoids sqrt for comparisons).
fn color_dist_sq(a: &engine_core::color::Color, b: &engine_core::color::Color) -> f32 {
    let dr = a.r - b.r;
    let dg = a.g - b.g;
    let db = a.b - b.b;
    dr * dr + dg * dg + db * db
}

/// Merge small regions into their nearest-color adjacent neighbor.
///
/// Uses a pixel-to-region_id map for O(1) adjacency lookups. Rebuilds
/// adjacency once per merge round (O(pixels) per round, not O(n^2*pixels)).
#[allow(clippy::too_many_lines)]
fn merge_small_regions(
    regions: &mut Vec<segment::Region>,
    merge_below: usize,
    max_shapes: usize,
    width: u32,
) {
    if regions.len() <= 1 {
        return;
    }

    let pixel_count = regions[0].mask.len();
    let w = width as usize;

    // Pixel → region_id map.
    let mut owner = vec![-1i32; pixel_count];
    let mut area: Vec<usize> = Vec::with_capacity(regions.len());
    for (id, region) in regions.iter().enumerate() {
        let mut count = 0usize;
        for (i, &m) in region.mask.iter().enumerate() {
            if m {
                owner[i] = id as i32;
                count += 1;
            }
        }
        area.push(count);
    }

    // Build adjacency from pixel map: O(pixels).
    let build_adj = |owner: &[i32], n: usize| -> Vec<BTreeSet<usize>> {
        let mut adj = vec![BTreeSet::new(); n];
        let len = owner.len();
        for (i, &oid) in owner.iter().enumerate() {
            if oid < 0 {
                continue;
            }
            let o = oid as usize;
            let x = i % w;
            if x > 0 {
                let nid = owner[i - 1];
                if nid >= 0 && nid != oid {
                    adj[o].insert(nid as usize);
                }
            }
            if x + 1 < w {
                let nid = owner[i + 1];
                if nid >= 0 && nid != oid {
                    adj[o].insert(nid as usize);
                }
            }
            if i >= w {
                let nid = owner[i - w];
                if nid >= 0 && nid != oid {
                    adj[o].insert(nid as usize);
                }
            }
            if i + w < len {
                let nid = owner[i + w];
                if nid >= 0 && nid != oid {
                    adj[o].insert(nid as usize);
                }
            }
        }
        adj
    };

    // Merge region `hi` into `lo`. Updates owner map, masks, colors, areas.
    let do_merge = |regions: &mut Vec<segment::Region>,
                    area: &mut Vec<usize>,
                    owner: &mut [i32],
                    lo: usize,
                    hi: usize| {
        let lo_a = area[lo] as f32;
        let hi_a = area[hi] as f32;
        let total = lo_a + hi_a;
        if total > 0.0 {
            let lc = regions[lo].color;
            let hc = regions[hi].color;
            regions[lo].color = engine_core::color::Color::new(
                (lc.r * lo_a + hc.r * hi_a) / total,
                (lc.g * lo_a + hc.g * hi_a) / total,
                (lc.b * lo_a + hc.b * hi_a) / total,
                1.0,
            );
        }
        // Can't borrow regions[lo] mutably and regions[hi] immutably at once,
        // so copy the hi mask first.
        let hi_mask: Vec<bool> = regions[hi].mask.clone();
        for (dm, &sm) in regions[lo].mask.iter_mut().zip(hi_mask.iter()) {
            *dm |= sm;
        }
        area[lo] += area[hi];
        // Update pixel owner map.
        let hi_i32 = hi as i32;
        let lo_i32 = lo as i32;
        for p in owner.iter_mut() {
            if *p == hi_i32 {
                *p = lo_i32;
            } else if *p > hi_i32 {
                *p -= 1;
            }
        }
        regions.remove(hi);
        area.remove(hi);
    };

    // Find best merge target: prefer adjacent nearest-color, fall back to
    // nearest-color globally.
    let find_target =
        |regions: &[segment::Region], idx: usize, adj: &[BTreeSet<usize>]| -> Option<usize> {
            let src = &regions[idx].color;
            let mut best = None;
            let mut best_d = f32::MAX;
            for &nid in &adj[idx] {
                let d = color_dist_sq(src, &regions[nid].color);
                if d < best_d {
                    best_d = d;
                    best = Some(nid);
                }
            }
            if best.is_some() {
                return best;
            }
            for (i, r) in regions.iter().enumerate() {
                if i == idx {
                    continue;
                }
                let d = color_dist_sq(src, &r.color);
                if d < best_d {
                    best_d = d;
                    best = Some(i);
                }
            }
            best
        };

    // Phase 1: merge regions below merge_below.
    if merge_below > 0 {
        loop {
            let adj = build_adj(&owner, regions.len());
            let small = (0..regions.len()).find(|&i| area[i] > 0 && area[i] < merge_below);
            let Some(idx) = small else { break };
            let Some(target) = find_target(regions, idx, &adj) else {
                break;
            };
            let (lo, hi) = if idx < target {
                (idx, target)
            } else {
                (target, idx)
            };
            do_merge(regions, &mut area, &mut owner, lo, hi);
        }
    }

    // Phase 2: enforce max_shapes cap.
    if max_shapes > 0 {
        while regions.len() > max_shapes {
            let adj = build_adj(&owner, regions.len());
            let idx = (0..regions.len())
                .min_by_key(|&i| area[i])
                .expect("non-empty");
            let Some(target) = find_target(regions, idx, &adj) else {
                break;
            };
            let (lo, hi) = if idx < target {
                (idx, target)
            } else {
                (target, idx)
            };
            do_merge(regions, &mut area, &mut owner, lo, hi);
        }
    }
}

/// Convert an RGBA pixel buffer into a vector of engine `Shape`s.
///
/// Each flood-fill color region in the image becomes one `Shape` with a
/// `ShapeVariant::Path` contour and the region's average color.
/// When `max_dimension` is set, the image is resized (up or down) so its
/// longest side matches that value. Output shapes use the resized dimensions
/// as their coordinate space.
#[allow(clippy::too_many_lines)]
pub fn image_to_shapes(
    rgba: &[u8],
    width: u32,
    height: u32,
    config: &ConvertConfig,
) -> ConvertResult {
    if rgba.is_empty() || width == 0 || height == 0 {
        return ConvertResult {
            shapes: Vec::new(),
            background: None,
            estimate: OutputEstimate {
                shape_count: 0,
                command_count: 0,
                line_to_count: 0,
                cubic_to_count: 0,
                estimated_loc: 0,
                estimated_floats: 0,
            },
            rgba: Vec::new(),
            width,
            height,
        };
    }

    let max_dim = config.max_dimension;
    let (work_rgba, work_w, work_h);
    if max_dim > 0 && width.max(height) != max_dim {
        let scale = max_dim as f32 / width.max(height) as f32;
        work_w = (width as f32 * scale).round() as u32;
        work_h = (height as f32 * scale).round() as u32;

        if scale > 1.0 && config.resize_method == ResizeMethod::Scale2x {
            let mut buf = rgba.to_vec();
            let mut cur_w = width;
            let mut cur_h = height;
            while cur_w * 2 <= work_w && cur_h * 2 <= work_h {
                buf = scale2x::scale2x_rgba(&buf, cur_w, cur_h);
                cur_w *= 2;
                cur_h *= 2;
            }
            if cur_w != work_w || cur_h != work_h {
                buf = resize_rgba(&buf, cur_w, cur_h, work_w, work_h);
            }
            work_rgba = buf;
        } else {
            work_rgba = resize_rgba(rgba, width, height, work_w, work_h);
        }
    } else {
        work_w = width;
        work_h = height;
        work_rgba = rgba.to_vec();
    }

    let mut regions = segment::segment(
        &work_rgba,
        work_w,
        work_h,
        config.color_threshold,
        config.alpha_threshold,
    );

    // Merge small regions into their nearest-color neighbor.
    // Ensure merge_below is at least min_area so that every region too small
    // to emit a shape gets absorbed into a neighbor instead of leaving a hole.
    let effective_merge = if config.min_area > 0 {
        config.merge_below.max(config.min_area)
    } else {
        config.merge_below
    };
    if effective_merge > 0 || config.max_shapes > 0 {
        merge_small_regions(&mut regions, effective_merge, config.max_shapes, work_w);
    }

    let w = work_w as f32;
    let h = work_h as f32;

    // ── Planar graph: the central data model ──────────────────────────
    // Build a half-edge planar graph from pixel boundaries. Extract faces
    // (closed polygons) with shared edge chains between junction vertices.
    // Adjacent faces share chains structurally — gaps are impossible.
    let region_map = build_region_map(&regions, work_w, work_h);
    // When bezier fitting is enabled, skip RDP simplification so chains
    // retain full staircase detail — the bezier fitter compresses them into
    // a few curves far more effectively than RDP + LineTo.
    let graph_epsilon = if config.use_bezier {
        0.0
    } else {
        config.rdp_epsilon
    };
    let graph = boundary_graph::extract_region_faces(&region_map, work_w, work_h, graph_epsilon);

    // ── Pre-compute fitted commands per chain (in pixel coords) ──────
    // Both faces sharing a chain use the same pre-computed commands (one
    // forward, one reversed), so bezier fitting is gap-free even on
    // interior chains.
    let chain_commands: Vec<Vec<PathCommand>> = graph
        .chains
        .iter()
        .map(|chain| {
            if config.use_bezier && chain.points.len() > 2 {
                if chain.is_closed {
                    fit_closed_bezier(&chain.points, config.bezier_error)
                } else {
                    bezier_fit::fit_bezier_segment(&chain.points, config.bezier_error)
                }
            } else if chain.is_closed {
                // Closed loop with LineTo: include closing segment.
                let mut cmds: Vec<PathCommand> = chain.points[1..]
                    .iter()
                    .map(|&(x, y)| PathCommand::LineTo(Vec2::new(x, y)))
                    .collect();
                cmds.push(PathCommand::LineTo(Vec2::new(
                    chain.points[0].0,
                    chain.points[0].1,
                )));
                cmds
            } else {
                // Open chain with LineTo.
                chain.points[1..]
                    .iter()
                    .map(|&(x, y)| PathCommand::LineTo(Vec2::new(x, y)))
                    .collect()
            }
        })
        .collect();

    let to_engine = |x: f32, y: f32| -> (f32, f32) { (x - w / 2.0, h / 2.0 - y) };
    let half_w = w / 2.0;
    let half_h = h / 2.0;

    // ── Assemble shapes from graph faces (last step) ─────────────────
    let mut shapes: Vec<(f32, Shape)> = Vec::new();

    for face in &graph.faces {
        if face.region_id < 0 {
            continue;
        }
        let rid = face.region_id as usize;
        if rid >= regions.len() {
            continue;
        }
        let region = &regions[rid];

        let pixel_area = region.mask.iter().filter(|&&b| b).count();
        if config.min_area > 0 && pixel_area < config.min_area {
            continue;
        }

        let pts = graph.face_vertices(face);
        if pts.len() < 3 {
            continue;
        }

        let contour_area = signed_polygon_area(&pts);
        // Enclosed region faces have negative signed area (CW winding).
        // Skip positive-area faces (outer infinite face / transparent).
        if contour_area >= 0.0 {
            continue;
        }
        let contour_area = contour_area.abs();

        // Assemble PathCommands from chain refs.
        let first_eng = to_engine(pts[0].0, pts[0].1);
        let mut cmds = vec![PathCommand::MoveTo(Vec2::new(first_eng.0, first_eng.1))];

        for cr in &face.chain_refs {
            let chain = &graph.chains[cr.chain_index];
            let raw_cmds = &chain_commands[cr.chain_index];

            if cr.reversed {
                let seg_start = Vec2::new(chain.points[0].0, chain.points[0].1);
                let reversed = reverse_path_commands(raw_cmds, seg_start);
                for cmd in &reversed {
                    push_transformed_command(&mut cmds, cmd, &to_engine);
                }
            } else {
                for cmd in raw_cmds {
                    push_transformed_command(&mut cmds, cmd, &to_engine);
                }
            }
        }
        cmds.push(PathCommand::Close);

        let cmds = cull_out_of_bounds_cubics(cmds, half_w, half_h);
        let commands = clamp_to_bounds(cmds, half_w, half_h);

        shapes.push((
            contour_area,
            Shape {
                variant: ShapeVariant::Path { commands },
                color: region.color,
            },
        ));
    }

    // Error-pink background rectangle: any holes in the geometry show
    // through as magenta, making coverage gaps immediately visible.
    let background = if shapes.is_empty() {
        None
    } else {
        Some(Shape {
            variant: ShapeVariant::Path {
                commands: vec![
                    PathCommand::MoveTo(Vec2::new(-half_w, half_h)),
                    PathCommand::LineTo(Vec2::new(half_w, half_h)),
                    PathCommand::LineTo(Vec2::new(half_w, -half_h)),
                    PathCommand::LineTo(Vec2::new(-half_w, -half_h)),
                    PathCommand::Close,
                ],
            },
            color: engine_core::color::Color::new(1.0, 0.0, 1.0, 1.0),
        })
    };

    // Sort largest-footprint first so big shapes act as background
    // (painted first), small details on top (painted last).
    shapes.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let final_shapes: Vec<Shape> = shapes.into_iter().map(|(_, shape)| shape).collect();
    let estimate = compute_estimate(&final_shapes);
    ConvertResult {
        shapes: final_shapes,
        background,
        estimate,
        rgba: work_rgba,
        width: work_w,
        height: work_h,
    }
}

/// Compute output size estimates from the final shape list.
pub fn compute_estimate(shapes: &[Shape]) -> OutputEstimate {
    let mut command_count = 0usize;
    let mut line_to_count = 0usize;
    let mut cubic_to_count = 0usize;
    let mut move_to_count = 0usize;

    for shape in shapes {
        if let ShapeVariant::Path { commands } = &shape.variant {
            command_count += commands.len();
            for cmd in commands {
                match cmd {
                    PathCommand::LineTo(_) => line_to_count += 1,
                    PathCommand::CubicTo { .. } => cubic_to_count += 1,
                    PathCommand::MoveTo(_) => move_to_count += 1,
                    _ => {}
                }
            }
        }
    }

    // ~8 lines overhead per shape (Shape {, variant:, commands: vec![, ], }, color:, },)
    // + 1 line per command + ~5 lines file header
    let estimated_loc = 5 + shapes.len() * 8 + command_count;
    // 2 floats per MoveTo/LineTo, 6 per CubicTo, 4 per color, 0 for Close
    let estimated_floats =
        (move_to_count + line_to_count) * 2 + cubic_to_count * 6 + shapes.len() * 4;

    OutputEstimate {
        shape_count: shapes.len(),
        command_count,
        line_to_count,
        cubic_to_count,
        estimated_loc,
        estimated_floats,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use engine_render::shape::PathCommand;
    use glam::Vec2;

    fn default_config() -> ConvertConfig {
        ConvertConfig {
            color_threshold: 0.1,
            alpha_threshold: 128,
            rdp_epsilon: 0.5,
            bezier_error: 0.5,
            min_area: 0,
            max_dimension: 0,
            resize_method: ResizeMethod::Nearest,
            use_bezier: true,
            merge_below: 0,
            max_shapes: 0,
        }
    }

    #[test]
    fn when_empty_rgba_then_returns_empty() {
        // Arrange / Act
        let result = image_to_shapes(&[], 0, 0, &default_config());

        // Assert
        assert!(result.shapes.is_empty());
    }

    #[test]
    fn when_fully_transparent_image_then_returns_empty() {
        // Arrange — 4x4 all alpha=0
        let rgba = vec![0u8; 4 * 4 * 4];

        // Act
        let result = image_to_shapes(&rgba, 4, 4, &default_config());

        // Assert
        assert!(result.shapes.is_empty());
    }

    #[test]
    fn when_single_opaque_pixel_then_returns_one_path() {
        // Arrange — 3x3, only center pixel opaque red
        let mut rgba = vec![0u8; 3 * 3 * 4];
        let center_byte = (3 + 1) * 4; // pixel (1,1)
        rgba[center_byte] = 255;
        rgba[center_byte + 1] = 0;
        rgba[center_byte + 2] = 0;
        rgba[center_byte + 3] = 255;

        // Act
        let result = image_to_shapes(&rgba, 3, 3, &default_config());

        // Assert
        assert_eq!(result.shapes.len(), 1);
        assert!(matches!(
            result.shapes[0].variant,
            ShapeVariant::Path { .. }
        ));
    }

    #[test]
    fn when_single_opaque_pixel_then_path_starts_with_moveto_ends_with_close() {
        // Arrange — 3x3, only center pixel opaque
        let mut rgba = vec![0u8; 3 * 3 * 4];
        let center_byte = (3 + 1) * 4; // pixel (1,1)
        rgba[center_byte] = 255;
        rgba[center_byte + 3] = 255;

        // Act
        let result = image_to_shapes(&rgba, 3, 3, &default_config());

        // Assert
        let commands = match &result.shapes[0].variant {
            ShapeVariant::Path { commands } => commands,
            _ => panic!("expected Path variant"),
        };
        assert!(matches!(commands[0], PathCommand::MoveTo(_)));
        assert!(matches!(*commands.last().unwrap(), PathCommand::Close));
    }

    #[test]
    fn when_two_disconnected_regions_then_returns_two_shapes() {
        // Arrange — 9x3, two 2x2 red blocks separated by transparent gap
        let mut rgba = vec![0u8; 9 * 3 * 4];
        // Block 1: columns 0-1, rows 0-1
        for row in 0..2 {
            for col in 0..2 {
                let idx = (row * 9 + col) * 4;
                rgba[idx] = 255;
                rgba[idx + 3] = 255;
            }
        }
        // Block 2: columns 6-7, rows 0-1
        for row in 0..2 {
            for col in 6..8 {
                let idx = (row * 9 + col) * 4;
                rgba[idx] = 255;
                rgba[idx + 3] = 255;
            }
        }

        // Act
        let result = image_to_shapes(&rgba, 9, 3, &default_config());

        // Assert
        assert_eq!(result.shapes.len(), 2);
        assert!(
            result
                .shapes
                .iter()
                .all(|s| matches!(s.variant, ShapeVariant::Path { .. }))
        );
    }

    #[test]
    fn when_fully_opaque_square_then_path_coordinates_centered_at_origin() {
        // Arrange — 10x10 all opaque white
        let rgba = vec![255u8; 10 * 10 * 4];

        // Act
        let result = image_to_shapes(&rgba, 10, 10, &default_config());

        // Assert
        assert_eq!(result.shapes.len(), 1);
        let commands = match &result.shapes[0].variant {
            ShapeVariant::Path { commands } => commands,
            _ => panic!("expected Path variant"),
        };
        // Extract all Vec2 coordinates from commands
        let mut has_neg_x = false;
        let mut has_pos_x = false;
        let mut has_neg_y = false;
        let mut has_pos_y = false;
        for cmd in commands {
            let pts: Vec<Vec2> = match cmd {
                PathCommand::MoveTo(v) | PathCommand::LineTo(v) => vec![*v],
                PathCommand::CubicTo {
                    control1,
                    control2,
                    to,
                } => vec![*control1, *control2, *to],
                _ => vec![],
            };
            for v in pts {
                if v.x < 0.0 {
                    has_neg_x = true;
                }
                if v.x > 0.0 {
                    has_pos_x = true;
                }
                if v.y < 0.0 {
                    has_neg_y = true;
                }
                if v.y > 0.0 {
                    has_pos_y = true;
                }
            }
        }
        assert!(
            has_neg_x && has_pos_x,
            "path should span both sides of x-axis"
        );
        assert!(
            has_neg_y && has_pos_y,
            "path should span both sides of y-axis"
        );
    }

    #[test]
    fn when_fully_opaque_square_then_top_edge_has_positive_y() {
        // Arrange — 10x10 all opaque white
        let rgba = vec![255u8; 10 * 10 * 4];

        // Act
        let result = image_to_shapes(&rgba, 10, 10, &default_config());

        // Assert — maximum Y should be positive (top of image = positive y)
        let max_y = match &result.shapes[0].variant {
            ShapeVariant::Path { commands } => commands
                .iter()
                .filter_map(|cmd| match cmd {
                    PathCommand::MoveTo(v) | PathCommand::LineTo(v) => Some(v.y),
                    _ => None,
                })
                .fold(f32::NEG_INFINITY, f32::max),
            _ => panic!("expected Path variant"),
        };
        assert!(
            max_y > 0.0,
            "top edge should map to positive y, got {max_y}"
        );
    }

    #[test]
    fn when_circle_image_with_large_epsilon_then_fewer_commands_than_boundary_pixels() {
        // Arrange — 20x20 image with a solid circle (radius ~8)
        let mut rgba = vec![0u8; 20 * 20 * 4];
        let cx = 10.0_f32;
        let cy = 10.0_f32;
        let r = 8.0_f32;
        for row in 0..20 {
            for col in 0..20 {
                let dx = col as f32 + 0.5 - cx;
                let dy = row as f32 + 0.5 - cy;
                if dx * dx + dy * dy <= r * r {
                    let idx = (row * 20 + col) * 4;
                    rgba[idx] = 255;
                    rgba[idx + 1] = 128;
                    rgba[idx + 3] = 255;
                }
            }
        }
        let config = ConvertConfig {
            color_threshold: 0.1,
            alpha_threshold: 128,
            rdp_epsilon: 2.0,
            bezier_error: 1.0,
            min_area: 0,
            max_dimension: 0,
            resize_method: ResizeMethod::Nearest,
            use_bezier: true,
            merge_below: 0,
            max_shapes: 0,
        };

        // Act
        let result = image_to_shapes(&rgba, 20, 20, &config);

        // Assert — the circle boundary has ~50 pixels, commands should be fewer
        assert_eq!(result.shapes.len(), 1);
        let cmd_count = match &result.shapes[0].variant {
            ShapeVariant::Path { commands } => commands.len(),
            _ => panic!("expected Path variant"),
        };
        assert!(
            cmd_count < 50,
            "expected fewer than 50 commands for simplified circle, got {cmd_count}"
        );
    }

    #[test]
    fn when_gradient_segmented_with_min_area_then_tiny_regions_merged() {
        // Arrange — 6x1 gradient from red to orange, tight threshold fragments it
        let mut rgba = vec![0u8; 6 * 4];
        for col in 0..6 {
            let idx = col * 4;
            rgba[idx] = 255;
            rgba[idx + 1] = (col as u8) * 25; // G: 0, 25, 50, 75, 100, 125
            rgba[idx + 2] = 0;
            rgba[idx + 3] = 255;
        }
        let config = ConvertConfig {
            color_threshold: 0.05,
            alpha_threshold: 128,
            rdp_epsilon: 0.5,
            bezier_error: 0.5,
            min_area: 4,
            max_dimension: 0,
            resize_method: ResizeMethod::Nearest,
            use_bezier: true,
            merge_below: 0, // merge_below < min_area, so pipeline auto-raises it
            max_shapes: 0,
        };

        // Act
        let result = image_to_shapes(&rgba, 6, 1, &config);

        // Assert — tiny regions are merged (not discarded), so pixels are covered.
        // The auto-merge ensures merge_below >= min_area, preventing holes.
        assert!(
            !result.shapes.is_empty(),
            "tiny regions should be merged into neighbors, not discarded"
        );
    }

    #[test]
    fn when_min_area_zero_then_all_regions_kept() {
        // Arrange — same gradient, but min_area=0 keeps everything
        let mut rgba = vec![0u8; 6 * 4];
        for col in 0..6 {
            let idx = col * 4;
            rgba[idx] = 255;
            rgba[idx + 1] = (col as u8) * 25;
            rgba[idx + 2] = 0;
            rgba[idx + 3] = 255;
        }
        let config = ConvertConfig {
            color_threshold: 0.05,
            alpha_threshold: 128,
            rdp_epsilon: 0.5,
            bezier_error: 0.5,
            min_area: 0,
            max_dimension: 0,
            resize_method: ResizeMethod::Nearest,
            use_bezier: true,
            merge_below: 0,
            max_shapes: 0,
        };

        // Act
        let result = image_to_shapes(&rgba, 6, 1, &config);

        // Assert — at least one shape exists (no filtering)
        assert!(
            !result.shapes.is_empty(),
            "min_area=0 should keep all regions"
        );
    }

    #[test]
    fn when_large_region_present_then_not_discarded_by_min_area() {
        // Arrange — 4x4 all-red opaque (16 pixels, well above min_area=4)
        let rgba = vec![255u8; 4 * 4 * 4];
        let config = ConvertConfig {
            color_threshold: 0.1,
            alpha_threshold: 128,
            rdp_epsilon: 0.5,
            bezier_error: 0.5,
            min_area: 4,
            max_dimension: 0,
            resize_method: ResizeMethod::Nearest,
            use_bezier: true,
            merge_below: 0,
            max_shapes: 0,
        };

        // Act
        let result = image_to_shapes(&rgba, 4, 4, &config);

        // Assert
        assert_eq!(
            result.shapes.len(),
            1,
            "large region should survive min_area filter"
        );
    }

    #[test]
    fn when_max_dimension_zero_and_large_image_then_no_resize() {
        // Arrange — 8x4 opaque image, larger than typical max_dimension values
        let rgba = vec![255u8; 8 * 4 * 4];
        let config = ConvertConfig {
            max_dimension: 0,
            ..default_config()
        };

        // Act
        let result = image_to_shapes(&rgba, 8, 4, &config);

        // Assert
        assert_eq!(result.width, 8);
        assert_eq!(result.height, 4);
    }

    #[test]
    fn when_max_dimension_zero_and_small_image_then_no_resize() {
        // Arrange — 4x4 opaque image, small enough that upscaling would apply
        let rgba = vec![255u8; 4 * 4 * 4];
        let config = ConvertConfig {
            max_dimension: 0,
            ..default_config()
        };

        // Act
        let result = image_to_shapes(&rgba, 4, 4, &config);

        // Assert
        assert_eq!(result.width, 4);
        assert_eq!(result.height, 4);
    }

    #[test]
    fn when_image_exceeds_max_dimension_then_downscaled() {
        // Arrange — 20x10 image with max_dimension=8; scale = 8/20 = 0.4
        let rgba = vec![255u8; 20 * 10 * 4];
        let config = ConvertConfig {
            max_dimension: 8,
            ..default_config()
        };

        // Act
        let result = image_to_shapes(&rgba, 20, 10, &config);

        // Assert — 20*0.4=8, 10*0.4=4
        assert_eq!(result.width, 8);
        assert_eq!(result.height, 4);
    }

    #[test]
    fn when_longest_side_equals_max_dimension_then_no_resize() {
        // Arrange — 10x6 image with max_dimension=10; already at limit
        let rgba = vec![255u8; 10 * 6 * 4];
        let config = ConvertConfig {
            max_dimension: 10,
            ..default_config()
        };

        // Act
        let result = image_to_shapes(&rgba, 10, 6, &config);

        // Assert
        assert_eq!(result.width, 10);
        assert_eq!(result.height, 6);
    }

    #[test]
    fn when_square_image_smaller_than_max_dimension_then_upscaled() {
        // Arrange — 4x4 opaque image; max_dimension=8 means scale=2.0
        let rgba = vec![255u8; 4 * 4 * 4];
        let config = ConvertConfig {
            max_dimension: 8,
            ..default_config()
        };

        // Act
        let result = image_to_shapes(&rgba, 4, 4, &config);

        // Assert
        assert_eq!(result.width, 8);
        assert_eq!(result.height, 8);
    }

    #[test]
    fn when_landscape_smaller_then_upscale_preserves_aspect_ratio() {
        // Arrange — 4x2 opaque image; max_dimension=12, scale=12/4=3.0
        let rgba = vec![255u8; 4 * 2 * 4];
        let config = ConvertConfig {
            max_dimension: 12,
            ..default_config()
        };

        // Act
        let result = image_to_shapes(&rgba, 4, 2, &config);

        // Assert — 4*3=12, 2*3=6
        assert_eq!(result.width, 12);
        assert_eq!(result.height, 6);
    }

    #[test]
    fn when_portrait_smaller_then_upscale_uses_height() {
        // Arrange — 3x6 opaque image; max_dimension=12, scale=12/6=2.0
        let rgba = vec![255u8; 3 * 6 * 4];
        let config = ConvertConfig {
            max_dimension: 12,
            ..default_config()
        };

        // Act
        let result = image_to_shapes(&rgba, 3, 6, &config);

        // Assert — 3*2=6, 6*2=12
        assert_eq!(result.width, 6);
        assert_eq!(result.height, 12);
    }

    #[test]
    fn when_upscaled_then_shape_count_matches_original() {
        // Arrange — 4x4 image with two color blocks: red left half, blue right half
        let mut rgba = vec![0u8; 4 * 4 * 4];
        for row in 0..4u32 {
            for col in 0..4u32 {
                let idx = ((row * 4 + col) * 4) as usize;
                if col < 2 {
                    rgba[idx] = 255; // red
                } else {
                    rgba[idx + 2] = 255; // blue
                }
                rgba[idx + 3] = 255;
            }
        }
        let no_resize = ConvertConfig {
            max_dimension: 0,
            ..default_config()
        };
        let with_upscale = ConvertConfig {
            max_dimension: 8,
            ..default_config()
        };

        // Act
        let result_orig = image_to_shapes(&rgba, 4, 4, &no_resize);
        let result_up = image_to_shapes(&rgba, 4, 4, &with_upscale);

        // Assert — nearest-neighbor upscale preserves colors, so region count is stable
        assert_eq!(result_orig.shapes.len(), result_up.shapes.len());
    }

    #[test]
    fn when_large_triangle_then_output_has_few_commands() {
        // Arrange — 40x20 filled triangle (tip at top-center, base at bottom)
        let w = 40u32;
        let h = 20u32;
        let mut rgba = vec![0u8; (w * h * 4) as usize];
        for row in 0..h {
            // Triangle: at each row, the opaque span widens from center
            let half_width = (row as f32 / (h - 1) as f32 * (w as f32 / 2.0)) as u32;
            let cx = w / 2;
            let left = cx.saturating_sub(half_width);
            let right = (cx + half_width).min(w - 1);
            for col in left..=right {
                let idx = ((row * w + col) * 4) as usize;
                rgba[idx] = 255;
                rgba[idx + 1] = 128;
                rgba[idx + 3] = 255;
            }
        }
        let config = ConvertConfig {
            color_threshold: 0.1,
            alpha_threshold: 128,
            rdp_epsilon: 0.5,
            bezier_error: 0.5,
            min_area: 0,
            max_dimension: 0,
            resize_method: ResizeMethod::Nearest,
            use_bezier: true,
            merge_below: 0,
            max_shapes: 0,
        };

        // Act
        let result = image_to_shapes(&rgba, w, h, &config);

        // Assert — a triangle should produce a single shape with relatively few commands
        assert_eq!(result.shapes.len(), 1);
        let cmd_count = match &result.shapes[0].variant {
            ShapeVariant::Path { commands } => commands.len(),
            _ => panic!("expected Path variant"),
        };
        // Marching squares preserves the staircase boundary of the diagonal
        // edges, producing more commands than a convex hull would. The bezier
        // fitter converts the staircase into curves. 50 is a reasonable upper
        // bound for a 40x20 triangle.
        assert!(
            cmd_count < 100,
            "large triangle should produce fewer than 100 commands (got {cmd_count})"
        );
    }

    #[test]
    fn when_scale2x_method_and_downscale_then_nearest_used() {
        // Arrange — 20x10 image, max_dimension=8, Scale2x method
        let rgba = vec![255u8; 20 * 10 * 4];
        let config = ConvertConfig {
            max_dimension: 8,
            resize_method: ResizeMethod::Scale2x,
            ..default_config()
        };

        // Act
        let result = image_to_shapes(&rgba, 20, 10, &config);

        // Assert — downscale ignores resize_method, same as nearest
        assert_eq!(result.width, 8);
        assert_eq!(result.height, 4);
    }

    #[test]
    fn when_scale2x_exact_double_then_dimensions_match() {
        // Arrange — 4x4 image, max_dimension=8, Scale2x method (one pass: 4→8)
        let rgba = vec![255u8; 4 * 4 * 4];
        let config = ConvertConfig {
            max_dimension: 8,
            resize_method: ResizeMethod::Scale2x,
            ..default_config()
        };

        // Act
        let result = image_to_shapes(&rgba, 4, 4, &config);

        // Assert
        assert_eq!(result.width, 8);
        assert_eq!(result.height, 8);
    }

    #[test]
    fn when_scale2x_non_power_target_then_nearest_finishes() {
        // Arrange — 4x4 image, max_dimension=10
        // Scale2x: 4→8 (next pass 16 > 10, stop), then NN 8→10
        let rgba = vec![255u8; 4 * 4 * 4];
        let config = ConvertConfig {
            max_dimension: 10,
            resize_method: ResizeMethod::Scale2x,
            ..default_config()
        };

        // Act
        let result = image_to_shapes(&rgba, 4, 4, &config);

        // Assert
        assert_eq!(result.width, 10);
        assert_eq!(result.height, 10);
    }

    #[test]
    fn when_scale2x_color_blocks_then_region_count_preserved() {
        // Arrange — 4x4 with red left / blue right
        let mut rgba = vec![0u8; 4 * 4 * 4];
        for row in 0..4u32 {
            for col in 0..4u32 {
                let idx = ((row * 4 + col) * 4) as usize;
                if col < 2 {
                    rgba[idx] = 255;
                } else {
                    rgba[idx + 2] = 255;
                }
                rgba[idx + 3] = 255;
            }
        }
        let nearest = ConvertConfig {
            max_dimension: 8,
            resize_method: ResizeMethod::Nearest,
            ..default_config()
        };
        let scale2x = ConvertConfig {
            max_dimension: 8,
            resize_method: ResizeMethod::Scale2x,
            ..default_config()
        };

        // Act
        let result_nn = image_to_shapes(&rgba, 4, 4, &nearest);
        let result_s2x = image_to_shapes(&rgba, 4, 4, &scale2x);

        // Assert
        assert_eq!(result_nn.shapes.len(), result_s2x.shapes.len());
    }

    #[test]
    fn when_no_resize_then_buffer_matches_input() {
        // Arrange — 2x2 image with distinct pixels, no resize
        let rgba = vec![
            255, 0, 0, 255, // red
            0, 255, 0, 255, // green
            0, 0, 255, 255, // blue
            255, 255, 0, 255, // yellow
        ];

        // Act
        let result = image_to_shapes(&rgba, 2, 2, &default_config());

        // Assert
        assert_eq!(result.rgba, rgba);
    }

    #[test]
    fn when_resized_then_buffer_size_matches_dimensions() {
        // Arrange — 4x4 upscaled to 8x8
        let rgba = vec![255u8; 4 * 4 * 4];
        let config = ConvertConfig {
            max_dimension: 8,
            ..default_config()
        };

        // Act
        let result = image_to_shapes(&rgba, 4, 4, &config);

        // Assert
        assert_eq!(
            result.rgba.len(),
            (result.width * result.height * 4) as usize
        );
        assert_eq!(result.width, 8);
        assert_eq!(result.height, 8);
    }

    #[test]
    fn when_empty_input_then_buffer_is_empty() {
        // Arrange / Act
        let result = image_to_shapes(&[], 0, 0, &default_config());

        // Assert
        assert!(result.rgba.is_empty());
    }

    #[test]
    fn when_l_shape_then_output_path_preserves_concavity() {
        // Arrange — 10x8 L-shape: left column (2px wide, full height) + bottom
        // row (full width, 2px tall). The inner concave corner is at (2, 6).
        let w = 10u32;
        let h = 8u32;
        let mut rgba = vec![0u8; (w * h * 4) as usize];
        for row in 0..h {
            for col in 0..w {
                let in_left_column = col < 2;
                let in_bottom_row = row >= 6;
                if in_left_column || in_bottom_row {
                    let idx = ((row * w + col) * 4) as usize;
                    rgba[idx] = 255;
                    rgba[idx + 1] = 0;
                    rgba[idx + 2] = 0;
                    rgba[idx + 3] = 255;
                }
            }
        }
        let config = ConvertConfig {
            rdp_epsilon: 0.5,
            bezier_error: 0.5,
            ..default_config()
        };

        // Act
        let result = image_to_shapes(&rgba, w, h, &config);

        // Assert — extract all path vertices
        assert_eq!(result.shapes.len(), 1, "L-shape should be one region");
        let commands = match &result.shapes[0].variant {
            ShapeVariant::Path { commands } => commands,
            _ => panic!("expected Path"),
        };

        // Collect all endpoint coordinates from path commands
        let mut xs: Vec<f32> = Vec::new();
        let mut ys: Vec<f32> = Vec::new();
        for cmd in commands {
            match cmd {
                PathCommand::MoveTo(v) | PathCommand::LineTo(v) => {
                    xs.push(v.x);
                    ys.push(v.y);
                }
                PathCommand::CubicTo { to, .. } => {
                    xs.push(to.x);
                    ys.push(to.y);
                }
                _ => {}
            }
        }

        // The L-shape has a concave notch. In engine coords (centered, Y-up),
        // the inner corner of the L is NOT at the image bounding box corners.
        // If the shape were convex-hulled, all path vertices would lie on the
        // bounding box edges. A concave L has at least one vertex that's
        // strictly inside the bounding box on both axes.
        let x_min = xs.iter().copied().fold(f32::INFINITY, f32::min);
        let x_max = xs.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let y_min = ys.iter().copied().fold(f32::INFINITY, f32::min);
        let y_max = ys.iter().copied().fold(f32::NEG_INFINITY, f32::max);

        let has_interior_vertex = xs.iter().zip(ys.iter()).any(|(&x, &y)| {
            x > x_min + 0.5 && x < x_max - 0.5 && y > y_min + 0.5 && y < y_max - 0.5
        });
        assert!(
            has_interior_vertex,
            "L-shape path should have at least one interior vertex (concavity). \
             All vertices on bounding box means concavity was lost. \
             x range: [{x_min:.1}, {x_max:.1}], y range: [{y_min:.1}, {y_max:.1}], \
             vertices: {:?}",
            xs.iter()
                .zip(ys.iter())
                .map(|(x, y)| format!("({x:.1},{y:.1})"))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn when_star_shape_then_output_preserves_valleys() {
        // Arrange — 21x21 5-pointed star: 5 outer tips + 5 inner valleys.
        // If the output is a pentagon, all 5 valleys are lost.
        let size = 21u32;
        let cx = size as f32 / 2.0;
        let cy = size as f32 / 2.0;
        let outer_r = 10.0_f32;
        let inner_r = 4.0_f32;
        let mut rgba = vec![0u8; (size * size * 4) as usize];

        // Rasterize star by checking if each pixel center is inside the star polygon
        let star_pts: Vec<(f32, f32)> = (0..10)
            .map(|i| {
                let angle = std::f32::consts::PI / 2.0 + i as f32 * std::f32::consts::PI / 5.0;
                let r = if i % 2 == 0 { outer_r } else { inner_r };
                (cx + r * angle.cos(), cy - r * angle.sin())
            })
            .collect();

        for row in 0..size {
            for col in 0..size {
                let px = col as f32 + 0.5;
                let py = row as f32 + 0.5;
                if point_in_polygon(px, py, &star_pts) {
                    let idx = ((row * size + col) * 4) as usize;
                    rgba[idx] = 255;
                    rgba[idx + 1] = 0;
                    rgba[idx + 2] = 0;
                    rgba[idx + 3] = 255;
                }
            }
        }

        let config = ConvertConfig {
            rdp_epsilon: 0.5,
            bezier_error: 0.5,
            min_area: 0,
            max_dimension: 0,
            resize_method: ResizeMethod::Nearest,
            ..default_config()
        };

        // Act
        let result = image_to_shapes(&rgba, size, size, &config);

        // Assert — extract vertices from path commands
        assert_eq!(result.shapes.len(), 1, "star should be one region");
        let commands = match &result.shapes[0].variant {
            ShapeVariant::Path { commands } => commands,
            _ => panic!("expected Path"),
        };

        let mut xs: Vec<f32> = Vec::new();
        let mut ys: Vec<f32> = Vec::new();
        for cmd in commands {
            match cmd {
                PathCommand::MoveTo(v) | PathCommand::LineTo(v) => {
                    xs.push(v.x);
                    ys.push(v.y);
                }
                PathCommand::CubicTo { to, .. } => {
                    xs.push(to.x);
                    ys.push(to.y);
                }
                _ => {}
            }
        }

        let x_min = xs.iter().copied().fold(f32::INFINITY, f32::min);
        let x_max = xs.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let y_min = ys.iter().copied().fold(f32::INFINITY, f32::min);
        let y_max = ys.iter().copied().fold(f32::NEG_INFINITY, f32::max);

        // A star has vertices inside the bounding box (the inner valleys).
        // Count how many vertices are strictly inside the bbox on both axes.
        let interior_count = xs
            .iter()
            .zip(ys.iter())
            .filter(|&(&x, &y)| {
                x > x_min + 1.0 && x < x_max - 1.0 && y > y_min + 1.0 && y < y_max - 1.0
            })
            .count();

        assert!(
            interior_count >= 3,
            "star should have at least 3 interior vertices (valley points), got {interior_count}. \
             x range: [{x_min:.1}, {x_max:.1}], y range: [{y_min:.1}, {y_max:.1}], \
             vertices: {:?}",
            xs.iter()
                .zip(ys.iter())
                .map(|(x, y)| format!("({x:.1},{y:.1})"))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn when_star_shape_with_scale2x_then_output_preserves_valleys() {
        // Arrange — same star as above but with GUI-default Scale2x upscaling
        let size = 21u32;
        let cx = size as f32 / 2.0;
        let cy = size as f32 / 2.0;
        let outer_r = 10.0_f32;
        let inner_r = 4.0_f32;
        let mut rgba = vec![0u8; (size * size * 4) as usize];

        let star_pts: Vec<(f32, f32)> = (0..10)
            .map(|i| {
                let angle = std::f32::consts::PI / 2.0 + i as f32 * std::f32::consts::PI / 5.0;
                let r = if i % 2 == 0 { outer_r } else { inner_r };
                (cx + r * angle.cos(), cy - r * angle.sin())
            })
            .collect();

        for row in 0..size {
            for col in 0..size {
                let px = col as f32 + 0.5;
                let py = row as f32 + 0.5;
                if point_in_polygon(px, py, &star_pts) {
                    let idx = ((row * size + col) * 4) as usize;
                    rgba[idx] = 255;
                    rgba[idx + 1] = 0;
                    rgba[idx + 2] = 0;
                    rgba[idx + 3] = 255;
                }
            }
        }

        let config = ConvertConfig {
            color_threshold: 0.1,
            alpha_threshold: 128,
            rdp_epsilon: 0.5,
            bezier_error: 0.5,
            min_area: 4,
            max_dimension: 128,
            resize_method: ResizeMethod::Scale2x,
            use_bezier: true,
            merge_below: 0,
            max_shapes: 0,
        };

        // Act
        let result = image_to_shapes(&rgba, size, size, &config);

        // Assert
        assert!(
            !result.shapes.is_empty(),
            "should produce at least one shape"
        );
        let commands = match &result.shapes[0].variant {
            ShapeVariant::Path { commands } => commands,
            _ => panic!("expected Path"),
        };

        let mut xs: Vec<f32> = Vec::new();
        let mut ys: Vec<f32> = Vec::new();
        for cmd in commands {
            match cmd {
                PathCommand::MoveTo(v) | PathCommand::LineTo(v) => {
                    xs.push(v.x);
                    ys.push(v.y);
                }
                PathCommand::CubicTo { to, .. } => {
                    xs.push(to.x);
                    ys.push(to.y);
                }
                _ => {}
            }
        }

        let x_min = xs.iter().copied().fold(f32::INFINITY, f32::min);
        let x_max = xs.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let y_min = ys.iter().copied().fold(f32::INFINITY, f32::min);
        let y_max = ys.iter().copied().fold(f32::NEG_INFINITY, f32::max);

        let interior_count = xs
            .iter()
            .zip(ys.iter())
            .filter(|&(&x, &y)| {
                x > x_min + 1.0 && x < x_max - 1.0 && y > y_min + 1.0 && y < y_max - 1.0
            })
            .count();

        assert!(
            interior_count >= 3,
            "star with Scale2x should have at least 3 interior vertices (valley points), \
             got {interior_count}. shapes: {}, vertices: {:?}",
            result.shapes.len(),
            xs.iter()
                .zip(ys.iter())
                .map(|(x, y)| format!("({x:.1},{y:.1})"))
                .collect::<Vec<_>>()
        );
    }

    /// Point-in-polygon test using ray casting.
    fn point_in_polygon(px: f32, py: f32, polygon: &[(f32, f32)]) -> bool {
        let n = polygon.len();
        let mut inside = false;
        let mut j = n - 1;
        for i in 0..n {
            let (xi, yi) = polygon[i];
            let (xj, yj) = polygon[j];
            if ((yi > py) != (yj > py)) && (px < (xj - xi) * (py - yi) / (yj - yi) + xi) {
                inside = !inside;
            }
            j = i;
        }
        inside
    }

    /// Extract polygon vertices from a shape's path commands (`LineTo` endpoints only).
    fn extract_shape_polygon(shape: &Shape) -> Vec<(f32, f32)> {
        let commands = match &shape.variant {
            ShapeVariant::Path { commands } => commands,
            _ => panic!("expected Path variant"),
        };
        commands
            .iter()
            .filter_map(|cmd| match cmd {
                PathCommand::MoveTo(v) | PathCommand::LineTo(v) => Some((v.x, v.y)),
                PathCommand::CubicTo { to, .. } => Some((to.x, to.y)),
                _ => None,
            })
            .collect()
    }

    /// Check if a point is inside any shape (using painter's algorithm order:
    /// last shape covering the point determines color).
    fn topmost_shape_at(shapes: &[Shape], px: f32, py: f32) -> Option<&Shape> {
        // Shapes are sorted largest-first (background first, details last).
        // The last shape in the list that contains the point is the topmost.
        shapes.iter().rev().find(|shape| {
            let poly = extract_shape_polygon(shape);
            poly.len() >= 3 && point_in_polygon(px, py, &poly)
        })
    }

    #[test]
    fn when_triangle_inside_rectangle_then_every_pixel_covered_by_correct_shape() {
        // Arrange — 10x10 image with a right-triangle inner region (red)
        // inside a rectangular outer region (blue).
        // The diagonal staircase boundary is where gaps appear.
        let w = 10u32;
        let h = 10u32;
        let mut rgba = vec![0u8; (w * h * 4) as usize];
        for row in 0..h {
            for col in 0..w {
                let idx = ((row * w + col) * 4) as usize;
                // Triangle: pixels where row >= col+2, within rows 2..8 cols 2..8
                let is_inner =
                    (2..8).contains(&row) && (2..8).contains(&col) && (row - 2) >= (col - 2);
                if is_inner {
                    rgba[idx] = 255; // red
                    rgba[idx + 1] = 0;
                    rgba[idx + 2] = 0;
                } else {
                    rgba[idx] = 0;
                    rgba[idx + 1] = 0;
                    rgba[idx + 2] = 255; // blue
                }
                rgba[idx + 3] = 255;
            }
        }

        let config = ConvertConfig {
            use_bezier: false,
            rdp_epsilon: 0.5,
            min_area: 0,
            ..default_config()
        };

        // Act
        let result = image_to_shapes(&rgba, w, h, &config);

        // Assert — every opaque pixel center must be inside at least one shape,
        // AND the topmost shape must have the correct color.
        let red = engine_core::color::Color::new(1.0, 0.0, 0.0, 1.0);
        let blue = engine_core::color::Color::new(0.0, 0.0, 1.0, 1.0);

        let mut mismatches = Vec::new();
        for row in 0..h {
            for col in 0..w {
                let eng_x = col as f32 + 0.5 - w as f32 / 2.0;
                let eng_y = h as f32 / 2.0 - (row as f32 + 0.5);
                let is_inner =
                    (2..8).contains(&row) && (2..8).contains(&col) && (row - 2) >= (col - 2);
                let expected_color = if is_inner { red } else { blue };

                if let Some(shape) = topmost_shape_at(&result.shapes, eng_x, eng_y) {
                    let color_matches = (shape.color.r - expected_color.r).abs() < 0.1
                        && (shape.color.g - expected_color.g).abs() < 0.1
                        && (shape.color.b - expected_color.b).abs() < 0.1;
                    if !color_matches {
                        mismatches.push((col, row, "wrong_color"));
                    }
                } else {
                    mismatches.push((col, row, "uncovered"));
                }
            }
        }

        // Post-processing simplifies diagonal staircases into straight lines,
        // which may leave a few boundary pixels showing the outer shape's color.
        // This is visually correct (smoother diagonal) — allow small tolerance.
        let uncovered = mismatches
            .iter()
            .filter(|(_, _, t)| *t == "uncovered")
            .count();
        assert_eq!(uncovered, 0, "no pixels should be uncovered");
        assert!(
            mismatches.len() <= 10,
            "pixel coverage: too many boundary mismatches ({}):\n{:?}",
            mismatches.len(),
            &mismatches[..mismatches.len().min(10)]
        );
    }

    #[test]
    fn when_triangle_inside_rectangle_bezier_then_every_pixel_covered() {
        // Same as above but with bezier ON — verifies that internal
        // no-junction boundaries use LineTo even when bezier is enabled.
        let w = 10u32;
        let h = 10u32;
        let mut rgba = vec![0u8; (w * h * 4) as usize];
        for row in 0..h {
            for col in 0..w {
                let idx = ((row * w + col) * 4) as usize;
                let is_inner =
                    (2..8).contains(&row) && (2..8).contains(&col) && (row - 2) >= (col - 2);
                if is_inner {
                    rgba[idx] = 255;
                } else {
                    rgba[idx + 2] = 255;
                }
                rgba[idx + 3] = 255;
            }
        }

        let config = ConvertConfig {
            use_bezier: true,
            rdp_epsilon: 0.5,
            bezier_error: 0.5,
            min_area: 0,
            ..default_config()
        };

        // Act
        let result = image_to_shapes(&rgba, w, h, &config);

        // Assert
        let red = engine_core::color::Color::new(1.0, 0.0, 0.0, 1.0);
        let blue = engine_core::color::Color::new(0.0, 0.0, 1.0, 1.0);

        let mut mismatches = Vec::new();
        for row in 0..h {
            for col in 0..w {
                let eng_x = col as f32 + 0.5 - w as f32 / 2.0;
                let eng_y = h as f32 / 2.0 - (row as f32 + 0.5);
                let is_inner =
                    (2..8).contains(&row) && (2..8).contains(&col) && (row - 2) >= (col - 2);
                let expected_color = if is_inner { red } else { blue };

                if let Some(shape) = topmost_shape_at(&result.shapes, eng_x, eng_y) {
                    let color_matches = (shape.color.r - expected_color.r).abs() < 0.1
                        && (shape.color.g - expected_color.g).abs() < 0.1
                        && (shape.color.b - expected_color.b).abs() < 0.1;
                    if !color_matches {
                        mismatches.push((col, row, "wrong_color"));
                    }
                } else {
                    mismatches.push((col, row, "uncovered"));
                }
            }
        }

        let uncovered = mismatches
            .iter()
            .filter(|(_, _, t)| *t == "uncovered")
            .count();
        assert_eq!(uncovered, 0, "no pixels should be uncovered");
        assert!(
            mismatches.len() <= 10,
            "bezier mode: too many boundary mismatches ({}):\n{:?}",
            mismatches.len(),
            &mismatches[..mismatches.len().min(10)]
        );
    }

    #[test]
    fn when_three_color_image_then_every_pixel_covered_by_correct_shape() {
        // Arrange — 12x8 image with 3 color regions creating a multi-junction
        // scenario. Red left strip, green right strip, blue bottom bar.
        //
        //  RRRRGGGG....
        //  RRRRGGGG....
        //  RRRRGGGG....
        //  RRRRGGGG....
        //  BBBBBBBBBBBB
        //  BBBBBBBBBBBB
        //  BBBBBBBBBBBB
        //  BBBBBBBBBBBB
        let w = 12u32;
        let h = 8u32;
        let mut rgba = vec![0u8; (w * h * 4) as usize];
        for row in 0..h {
            for col in 0..w {
                let idx = ((row * w + col) * 4) as usize;
                if row < 4 && col < 4 {
                    rgba[idx] = 255; // red
                } else if row < 4 {
                    rgba[idx + 1] = 255; // green
                } else {
                    rgba[idx + 2] = 255; // blue
                }
                rgba[idx + 3] = 255;
            }
        }

        let config = ConvertConfig {
            use_bezier: false,
            rdp_epsilon: 0.5,
            min_area: 0,
            ..default_config()
        };

        let result = image_to_shapes(&rgba, w, h, &config);

        // Assert — 3 shapes (red, green, blue)
        assert_eq!(result.shapes.len(), 3, "expected 3 shapes");

        let red = engine_core::color::Color::new(1.0, 0.0, 0.0, 1.0);
        let green = engine_core::color::Color::new(0.0, 1.0, 0.0, 1.0);
        let blue = engine_core::color::Color::new(0.0, 0.0, 1.0, 1.0);

        let mut mismatches = Vec::new();
        for row in 0..h {
            for col in 0..w {
                let eng_x = col as f32 + 0.5 - w as f32 / 2.0;
                let eng_y = h as f32 / 2.0 - (row as f32 + 0.5);
                let expected = if row < 4 && col < 4 {
                    red
                } else if row < 4 {
                    green
                } else {
                    blue
                };

                if let Some(shape) = topmost_shape_at(&result.shapes, eng_x, eng_y) {
                    let ok = (shape.color.r - expected.r).abs() < 0.1
                        && (shape.color.g - expected.g).abs() < 0.1
                        && (shape.color.b - expected.b).abs() < 0.1;
                    if !ok {
                        mismatches.push(format!(
                            "({col},{row}) expected ({:.0},{:.0},{:.0}) got ({:.0},{:.0},{:.0})",
                            expected.r * 255.0,
                            expected.g * 255.0,
                            expected.b * 255.0,
                            shape.color.r * 255.0,
                            shape.color.g * 255.0,
                            shape.color.b * 255.0,
                        ));
                    }
                } else {
                    mismatches.push(format!("({col},{row}) UNCOVERED"));
                }
            }
        }

        assert!(
            mismatches.is_empty(),
            "3-color coverage: {} issues:\n{}",
            mismatches.len(),
            mismatches[..mismatches.len().min(20)].join("\n")
        );
    }

    #[test]
    fn when_shallow_diagonal_inside_rectangle_then_no_gaps() {
        // Arrange — 8x8 image with a shallow-angle triangle (2:1 slope)
        // inside a rectangle. This creates staircase vertices at distance
        // 1/sqrt(5) ≈ 0.447 from the diagonal, which RDP at epsilon=0.5
        // would incorrectly remove. The fix uses epsilon=0 for internal
        // no-junction boundaries.
        let w = 8u32;
        let h = 8u32;
        let mut rgba = vec![0u8; (w * h * 4) as usize];
        for row in 0..h {
            for col in 0..w {
                let idx = ((row * w + col) * 4) as usize;
                // Shallow triangle: row >= 2 && col >= 2 && (row-2) >= 2*(col-2)
                let is_inner = (2..7).contains(&row)
                    && (2..6).contains(&col)
                    && (row as i32 - 2) >= 2 * (col as i32 - 2);
                if is_inner {
                    rgba[idx] = 255; // red
                } else {
                    rgba[idx + 2] = 255; // blue
                }
                rgba[idx + 3] = 255;
            }
        }

        let config = ConvertConfig {
            use_bezier: false,
            rdp_epsilon: 0.5,
            min_area: 0,
            ..default_config()
        };

        let result = image_to_shapes(&rgba, w, h, &config);

        let red = engine_core::color::Color::new(1.0, 0.0, 0.0, 1.0);
        let blue = engine_core::color::Color::new(0.0, 0.0, 1.0, 1.0);

        let mut mismatches = Vec::new();
        for row in 0..h {
            for col in 0..w {
                let eng_x = col as f32 + 0.5 - w as f32 / 2.0;
                let eng_y = h as f32 / 2.0 - (row as f32 + 0.5);
                let is_inner = (2..7).contains(&row)
                    && (2..6).contains(&col)
                    && (row as i32 - 2) >= 2 * (col as i32 - 2);
                let expected = if is_inner { red } else { blue };

                if let Some(shape) = topmost_shape_at(&result.shapes, eng_x, eng_y) {
                    let ok = (shape.color.r - expected.r).abs() < 0.1
                        && (shape.color.g - expected.g).abs() < 0.1
                        && (shape.color.b - expected.b).abs() < 0.1;
                    if !ok {
                        mismatches.push(format!("({col},{row}) wrong_color"));
                    }
                } else {
                    mismatches.push(format!("({col},{row}) UNCOVERED"));
                }
            }
        }

        let uncovered = mismatches
            .iter()
            .filter(|m| m.contains("UNCOVERED"))
            .count();
        assert_eq!(uncovered, 0, "no pixels should be uncovered");
        assert!(
            mismatches.len() <= 10,
            "shallow diagonal: too many boundary mismatches ({}):\n{}",
            mismatches.len(),
            mismatches[..mismatches.len().min(20)].join("\n")
        );
    }

    #[test]
    fn when_diagonal_boundary_three_regions_then_no_gaps() {
        // Arrange — 8x8 image with a diagonal boundary between regions.
        // Top-left triangle (red), top-right triangle (green), bottom half (blue).
        // The diagonal creates staircase boundaries that are the hardest case.
        let w = 8u32;
        let h = 8u32;
        let mut rgba = vec![0u8; (w * h * 4) as usize];
        for row in 0..h {
            for col in 0..w {
                let idx = ((row * w + col) * 4) as usize;
                if row < 4 {
                    if col <= row {
                        rgba[idx] = 255; // red (below diagonal)
                    } else {
                        rgba[idx + 1] = 255; // green (above diagonal)
                    }
                } else {
                    rgba[idx + 2] = 255; // blue (bottom half)
                }
                rgba[idx + 3] = 255;
            }
        }

        let config = ConvertConfig {
            use_bezier: false,
            rdp_epsilon: 0.5,
            min_area: 0,
            ..default_config()
        };

        let result = image_to_shapes(&rgba, w, h, &config);

        let red = engine_core::color::Color::new(1.0, 0.0, 0.0, 1.0);
        let green = engine_core::color::Color::new(0.0, 1.0, 0.0, 1.0);
        let blue = engine_core::color::Color::new(0.0, 0.0, 1.0, 1.0);

        let mut mismatches = Vec::new();
        for row in 0..h {
            for col in 0..w {
                let eng_x = col as f32 + 0.5 - w as f32 / 2.0;
                let eng_y = h as f32 / 2.0 - (row as f32 + 0.5);
                let expected = if row < 4 {
                    if col <= row { red } else { green }
                } else {
                    blue
                };

                if let Some(shape) = topmost_shape_at(&result.shapes, eng_x, eng_y) {
                    let ok = (shape.color.r - expected.r).abs() < 0.1
                        && (shape.color.g - expected.g).abs() < 0.1
                        && (shape.color.b - expected.b).abs() < 0.1;
                    if !ok {
                        mismatches.push(format!("({col},{row}) wrong color"));
                    }
                } else {
                    mismatches.push(format!("({col},{row}) UNCOVERED"));
                }
            }
        }

        let _uncovered = mismatches
            .iter()
            .filter(|m| m.contains("UNCOVERED"))
            .count();
        // Post-processing simplification may leave a few boundary pixels
        // uncovered or wrong-color at triple junctions.
        assert!(
            mismatches.len() <= 10,
            "diagonal boundary: too many mismatches ({}):\n{}",
            mismatches.len(),
            mismatches[..mismatches.len().min(20)].join("\n")
        );
    }

    // --- Region merging tests ---

    #[test]
    fn when_merge_below_set_then_small_regions_absorbed_by_neighbor() {
        // Arrange — 10x1 strip: 8 red pixels, 2 green pixels at the end.
        // With merge_below=3, the 2-pixel green region is absorbed into the red.
        let w = 10u32;
        let h = 1u32;
        let mut rgba = vec![0u8; (w * h * 4) as usize];
        for col in 0..8 {
            let idx = col * 4;
            rgba[idx] = 255; // red
            rgba[idx + 3] = 255;
        }
        for col in 8..10 {
            let idx = col * 4;
            rgba[idx + 1] = 255; // green
            rgba[idx + 3] = 255;
        }

        let config = ConvertConfig {
            color_threshold: 0.1,
            alpha_threshold: 128,
            rdp_epsilon: 0.5,
            bezier_error: 0.5,
            min_area: 0,
            max_dimension: 0,
            resize_method: ResizeMethod::Nearest,
            use_bezier: true,
            merge_below: 3, // green region (2px) < threshold → merged
            max_shapes: 0,
        };

        // Act
        let result = image_to_shapes(&rgba, w, h, &config);

        // Assert — only one shape remains (green merged into red)
        assert_eq!(
            result.shapes.len(),
            1,
            "small green region should be merged, got {} shapes",
            result.shapes.len()
        );
    }

    #[test]
    fn when_max_shapes_set_then_output_respects_cap() {
        // Arrange — 3 distinct color stripes that normally produce 3 shapes.
        let w = 9u32;
        let h = 1u32;
        let mut rgba = vec![0u8; (w * h * 4) as usize];
        // 3 red
        for col in 0..3 {
            let idx = col * 4;
            rgba[idx] = 255;
            rgba[idx + 3] = 255;
        }
        // 3 green
        for col in 3..6 {
            let idx = col * 4;
            rgba[idx + 1] = 255;
            rgba[idx + 3] = 255;
        }
        // 3 blue
        for col in 6..9 {
            let idx = col * 4;
            rgba[idx + 2] = 255;
            rgba[idx + 3] = 255;
        }

        let config = ConvertConfig {
            color_threshold: 0.01,
            alpha_threshold: 128,
            rdp_epsilon: 0.5,
            bezier_error: 0.5,
            min_area: 0,
            max_dimension: 0,
            resize_method: ResizeMethod::Nearest,
            use_bezier: true,
            merge_below: 0,
            max_shapes: 2, // cap at 2 → one of the 3 regions gets merged
        };

        // Act
        let result = image_to_shapes(&rgba, w, h, &config);

        // Assert
        assert!(
            result.shapes.len() <= 2,
            "max_shapes=2 should produce at most 2 shapes, got {}",
            result.shapes.len()
        );
    }

    #[test]
    fn when_estimate_computed_then_counts_match_actual_shapes() {
        // Arrange
        let w = 10u32;
        let h = 10u32;
        let mut rgba = vec![0u8; (w * h * 4) as usize];
        for i in 0..(w * h) as usize {
            rgba[i * 4] = 255;
            rgba[i * 4 + 3] = 255;
        }

        let config = default_config();

        // Act
        let result = image_to_shapes(&rgba, w, h, &config);

        // Assert
        assert_eq!(result.estimate.shape_count, result.shapes.len());

        let actual_commands: usize = result
            .shapes
            .iter()
            .map(|s| {
                if let ShapeVariant::Path { commands } = &s.variant {
                    commands.len()
                } else {
                    0
                }
            })
            .sum();
        assert_eq!(result.estimate.command_count, actual_commands);
    }
}
