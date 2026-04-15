use crate::card::identity::signature::{CardSignature, Element, compute_seed};
use crate::card::identity::signature_profile::{SignatureProfile, Tier};
use crate::card::rendering::art_shader::ShaderVariant;
use engine_core::color::Color;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

#[derive(Debug, PartialEq)]
pub struct CardVisualParams {
    pub art_color: Color,
    pub element_tint: Color,
    pub tier_detail: Tier,
    pub pattern_index: u8,
    pub shader_variant: ShaderVariant,
}

pub const PATTERN_COUNT: u8 = 4;

const COLOR_NOISE: f32 = 0.05;

pub fn element_base_color(element: Element) -> Color {
    match element {
        Element::Solidum => Color::new(0.55, 0.38, 0.18, 1.0), // brown/amber — earth, stone
        Element::Febris => Color::new(0.85, 0.25, 0.10, 1.0),  // red-orange — heat, cold
        Element::Ordinem => Color::new(0.20, 0.40, 0.80, 1.0), // blue — order, chaos
        Element::Lumines => Color::new(0.90, 0.78, 0.15, 1.0), // gold/yellow — light, dark
        Element::Varias => Color::new(0.22, 0.70, 0.28, 1.0),  // green — change, stasis
        Element::Inertiae => Color::new(0.50, 0.58, 0.68, 1.0), // steel gray-blue — force, stillness
        Element::Subsidium => Color::new(0.10, 0.65, 0.52, 1.0), // emerald/teal — growth, decay
        Element::Spatium => Color::new(0.58, 0.20, 0.80, 1.0), // violet/purple — expansion, contraction
    }
}

pub fn generate_card_visuals(
    signature: &CardSignature,
    profile: &SignatureProfile,
) -> CardVisualParams {
    let seed = compute_seed(signature);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    let element_tint = profile
        .dominant_axis
        .map_or(Color::new(0.5, 0.5, 0.5, 1.0), element_base_color);

    let art_color = Color::new(
        (element_tint.r + rng.random_range(-COLOR_NOISE..=COLOR_NOISE)).clamp(0.0, 1.0),
        (element_tint.g + rng.random_range(-COLOR_NOISE..=COLOR_NOISE)).clamp(0.0, 1.0),
        (element_tint.b + rng.random_range(-COLOR_NOISE..=COLOR_NOISE)).clamp(0.0, 1.0),
        1.0,
    );
    let pattern_index = rng.random_range(0..PATTERN_COUNT);
    let shader_variant = ShaderVariant::from_rarity(profile.rarity);
    let tier_detail = profile.tier;

    CardVisualParams {
        art_color,
        element_tint,
        tier_detail,
        pattern_index,
        shader_variant,
    }
}
