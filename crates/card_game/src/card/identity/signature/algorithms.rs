// EVOLVE-BLOCK-START
use super::types::CardSignature;

/// Hash a `CardSignature` into a stable 64-bit seed.
///
/// Used by rarity/tier assignment (different bit ranges for independence) and by visual parameter
/// generation. Placing this here avoids a circular dependency between `signature` and
/// `visual_params`.
pub fn compute_seed(signature: &CardSignature) -> u64 {
    signature
        .axes()
        .iter()
        .enumerate()
        .fold(0u64, |acc, (i, &v)| {
            let bits = u64::from(v.to_bits());
            let mixed = bits
                .wrapping_add(0x9e37_79b9_7f4a_7c15)
                .wrapping_mul(i as u64 + 1);
            acc ^ mixed.rotate_left(17).wrapping_mul(0x94d0_49bb_1331_11eb)
        })
}

pub fn geometric_level(value: f32, advance_rate: f32, max_levels: usize) -> usize {
    let mut remaining = value;
    for level in 0..max_levels - 1 {
        if remaining >= advance_rate {
            return level;
        }
        remaining /= advance_rate;
    }
    max_levels - 1
}
// EVOLVE-BLOCK-END
