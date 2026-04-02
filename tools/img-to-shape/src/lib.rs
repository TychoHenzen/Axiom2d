mod bezier_fit;
mod boundary_graph;
pub mod codegen;
pub mod manifest;
pub mod scale2x;
mod segment;
mod simplify;

use std::collections::BTreeSet;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex};

use engine_render::shape::{PathCommand, Shape, ShapeVariant};
use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Shared progress handle for tracking conversion progress from another thread.
/// Clone the `Arc` and poll `percent()` / `stage()` from the UI thread.
#[derive(Clone)]
pub struct ConvertProgress {
    percent: Arc<AtomicU8>,
    stage: Arc<Mutex<String>>,
}

impl ConvertProgress {
    pub fn new() -> Self {
        Self {
            percent: Arc::new(AtomicU8::new(0)),
            stage: Arc::new(Mutex::new(String::new())),
        }
    }

    /// Current progress percentage (0–100).
    pub fn percent(&self) -> u8 {
        self.percent.load(Ordering::Relaxed)
    }

    /// Current stage description.
    pub fn stage(&self) -> String {
        self.stage
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    fn set(&self, percent: u8, stage: &str) {
        self.percent.store(percent, Ordering::Relaxed);
        *self
            .stage
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = stage.to_string();
    }
}

impl Default for ConvertProgress {
    fn default() -> Self {
        Self::new()
    }
}

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    report: &dyn Fn(u8, &str),
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

    // Track progress by regions remaining (always accurate, no pre-estimate).
    let start_count = regions.len();

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
            let remaining = regions.len();
            let merged = start_count - remaining;
            if merged.is_multiple_of(3) {
                let frac = merged as f32 / start_count as f32;
                let pct = 30 + (frac * 10.0).min(9.0) as u8; // 30–39%
                report(pct, &format!("Merging small regions ({remaining} left)..."));
            }
        }
    }

    // Phase 2: enforce max_shapes cap.
    if max_shapes > 0 {
        let phase2_start = regions.len();
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
            let remaining = regions.len();
            let to_remove = phase2_start.saturating_sub(max_shapes);
            let removed = phase2_start - remaining;
            if removed.is_multiple_of(3) && to_remove > 0 {
                let frac = removed as f32 / to_remove as f32;
                let pct = 35 + (frac * 5.0).min(4.0) as u8; // 35–39%
                report(
                    pct,
                    &format!("Capping shapes ({remaining} → {max_shapes})..."),
                );
            }
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
    image_to_shapes_with_progress(rgba, width, height, config, None)
}

/// Same as `image_to_shapes` but updates a shared progress handle so the
/// caller can display conversion progress from another thread.
#[allow(clippy::too_many_lines)]
pub fn image_to_shapes_with_progress(
    rgba: &[u8],
    width: u32,
    height: u32,
    config: &ConvertConfig,
    progress: Option<&ConvertProgress>,
) -> ConvertResult {
    let report = |pct: u8, stage: &str| {
        if let Some(p) = progress {
            p.set(pct, stage);
        }
    };

    report(0, "Starting...");

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

    report(5, "Resizing...");
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

    report(15, "Segmenting...");
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
    report(30, "Merging regions...");
    if effective_merge > 0 || config.max_shapes > 0 {
        merge_small_regions(
            &mut regions,
            effective_merge,
            config.max_shapes,
            work_w,
            &report,
        );
    }

    let w = work_w as f32;
    let h = work_h as f32;

    report(40, "Building boundary graph...");
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

    report(60, "Fitting curves...");
    // ── Pre-compute fitted commands per chain (in pixel coords) ──────
    // Both faces sharing a chain use the same pre-computed commands (one
    // forward, one reversed), so bezier fitting is gap-free even on
    // interior chains.
    let total_chains = graph.chains.len();
    let mut chain_commands: Vec<Vec<PathCommand>> = Vec::with_capacity(total_chains);
    for (ci, chain) in graph.chains.iter().enumerate() {
        let cmds = if config.use_bezier && chain.points.len() > 2 {
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
        };
        chain_commands.push(cmds);
        if total_chains > 0 && ci.is_multiple_of(20) {
            let frac = ci as f32 / total_chains as f32;
            let pct = 60 + (frac * 20.0) as u8; // 60–80%
            report(pct, &format!("Fitting curves ({ci}/{total_chains})..."));
        }
    }

    let to_engine = |x: f32, y: f32| -> (f32, f32) { (x - w / 2.0, h / 2.0 - y) };
    let half_w = w / 2.0;
    let half_h = h / 2.0;

    report(80, "Assembling shapes...");
    // ── Assemble shapes from graph faces (last step) ─────────────────
    let mut shapes: Vec<(f32, Shape)> = Vec::new();
    let total_faces = graph.faces.len();

    for (fi, face) in graph.faces.iter().enumerate() {
        if total_faces > 0 && fi.is_multiple_of(50) {
            let frac = fi as f32 / total_faces as f32;
            let pct = 80 + (frac * 15.0) as u8; // 80–95%
            report(pct, &format!("Assembling shapes ({fi}/{total_faces})..."));
        }
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

    report(95, "Sorting & finalizing...");
    // Sort largest-footprint first so big shapes act as background
    // (painted first), small details on top (painted last).
    shapes.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let final_shapes: Vec<Shape> = shapes.into_iter().map(|(_, shape)| shape).collect();
    let estimate = compute_estimate(&final_shapes);
    report(100, "Done");
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
