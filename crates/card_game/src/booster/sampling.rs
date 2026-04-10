// EVOLVE-BLOCK-START
// Card signature sampling from SignatureSpace regions

use rand::Rng;
use rand_chacha::ChaCha8Rng;

use crate::card::identity::signature::CardSignature;
use crate::card::reader::SignatureSpace;

/// Sample `count` random `CardSignature` values from within a `SignatureSpace` region.
///
/// Each sample picks a random base point along the space's polyline (weighted by
/// segment length), adds a random 8D offset within the tube radius, and clamps
/// all axes to [-1.0, 1.0].
pub fn sample_signatures_from_space(
    space: &SignatureSpace,
    count: usize,
    rng: &mut ChaCha8Rng,
) -> Vec<CardSignature> {
    (0..count)
        .map(|_| {
            let base = pick_base_point(space, rng);
            let offset = random_8d_offset(space.radius, rng);
            let mut axes = base.axes();
            for (i, v) in axes.iter_mut().enumerate() {
                *v += offset[i];
            }
            CardSignature::new(axes)
        })
        .collect()
}

/// Pick a random point along the space's polyline, weighted by segment length.
///
/// Single-point spaces return the single control point.
/// Multi-point spaces interpolate along segments weighted by their length.
fn pick_base_point(space: &SignatureSpace, rng: &mut ChaCha8Rng) -> CardSignature {
    match space.control_points.len() {
        0 => CardSignature::default(),
        1 => space.control_points[0],
        _ => {
            // Compute segment lengths
            let n = space.control_points.len();
            let segment_count = if n == 2 { 1 } else { n };
            let mut lengths = Vec::with_capacity(segment_count);
            let mut total = 0.0_f32;
            for i in 0..segment_count {
                let j = (i + 1) % n;
                let len = space.control_points[i].distance_to(&space.control_points[j]);
                lengths.push(len);
                total += len;
            }

            if total < f32::EPSILON {
                return space.control_points[0];
            }

            // Pick a random distance along the total polyline
            let target = rng.random_range(0.0..total);
            let mut accumulated = 0.0_f32;
            for (i, &seg_len) in lengths.iter().enumerate() {
                accumulated += seg_len;
                if target <= accumulated {
                    let overshoot = accumulated - target;
                    let t = if seg_len > f32::EPSILON {
                        1.0 - overshoot / seg_len
                    } else {
                        0.0
                    };
                    let j = (i + 1) % n;
                    return lerp_signature(&space.control_points[i], &space.control_points[j], t);
                }
            }

            // Fallback (shouldn't happen due to float precision)
            let last_seg = segment_count - 1;
            let j = (last_seg + 1) % n;
            lerp_signature(
                &space.control_points[last_seg],
                &space.control_points[j],
                1.0,
            )
        }
    }
}

/// Linear interpolation between two `CardSignature` values.
fn lerp_signature(a: &CardSignature, b: &CardSignature, t: f32) -> CardSignature {
    let aa = a.axes();
    let ba = b.axes();
    let mut result = [0.0_f32; 8];
    for (i, v) in result.iter_mut().enumerate() {
        *v = aa[i] + t * (ba[i] - aa[i]);
    }
    // Don't clamp here — the caller will clamp via CardSignature::new if needed
    CardSignature::new(result)
}

/// Generate a random offset vector uniformly distributed within an 8D sphere
/// of the given radius.
///
/// Uses the rejection-free method: pick a random direction on the 8D unit sphere
/// (via normalized Gaussian vector), then scale by a random distance with the
/// appropriate power-law distribution for uniform volume sampling.
fn random_8d_offset(radius: f32, rng: &mut ChaCha8Rng) -> [f32; 8] {
    // Random direction: sample 8 independent standard normal values, then normalize
    let mut direction = [0.0_f32; 8];
    let mut mag_sq = 0.0_f32;
    for v in &mut direction {
        // Box-Muller is overkill; use the built-in normal distribution
        *v = sample_standard_normal(rng);
        mag_sq += *v * *v;
    }

    let mag = mag_sq.sqrt();
    if mag < f32::EPSILON {
        return [0.0; 8];
    }

    for v in &mut direction {
        *v /= mag;
    }

    // Random distance: for uniform sampling in 8D, r = radius * U^(1/8)
    let u: f32 = rng.random_range(0.0_f32..1.0_f32);
    let r = radius * u.powf(1.0 / 8.0);

    for v in &mut direction {
        *v *= r;
    }

    direction
}

/// Sample from a standard normal distribution using the Box-Muller transform.
fn sample_standard_normal(rng: &mut ChaCha8Rng) -> f32 {
    let u1: f32 = rng.random_range(f32::EPSILON..1.0_f32);
    let u2: f32 = rng.random_range(0.0_f32..std::f32::consts::TAU);
    (-2.0 * u1.ln()).sqrt() * u2.cos()
}
// EVOLVE-BLOCK-END
