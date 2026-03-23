use rand_chacha::ChaCha8Rng;

/// All word ingredients needed to assemble a card title.
pub struct TitleParts<'a> {
    pub adj: &'a str,
    pub noun: &'a str,
    pub compound: &'a str,
    pub name: &'a str,
    pub adj2: &'a str,
}

/// Picks an index from `weights` proportionally.
pub fn weighted_choose(rng: &mut ChaCha8Rng, weights: &[u32]) -> usize {
    use rand::Rng;
    let total: u32 = weights.iter().sum();
    let roll = rng.gen_range(0..total);
    let mut cumulative = 0;
    for (i, &w) in weights.iter().enumerate() {
        cumulative += w;
        if roll < cumulative {
            return i;
        }
    }
    weights.len() - 1
}

/// Weighted template selection for Common/Uncommon rarity.
/// Short templates only — no possessives or proper nouns.
///
/// 35% {adj} {noun}, 25% {noun} of {adj}, 20% The {adj} {noun}, 20% {adj} {compound}
pub fn common_title(rng: &mut ChaCha8Rng, parts: &TitleParts<'_>) -> String {
    let TitleParts {
        adj,
        noun,
        compound,
        ..
    } = parts;
    match weighted_choose(rng, &[35, 25, 20, 20]) {
        0 => format!("{adj} {noun}"),
        1 => format!("{noun} of {adj}"),
        2 => format!("The {adj} {noun}"),
        _ => format!("{adj} {compound}"),
    }
}

/// Weighted template selection for Rare/Epic rarity.
///
/// 20% {adj} {compound} of {adj2}, 15% {compound} of the {adj},
/// 15% The {adj} {compound}, 15% {name}'s {adj} {compound},
/// 10% {adj2} {compound} of {adj}, 10% the {adj} {compound} of {name},
/// 10% {compound}, {adj} and {adj2}, 5% {name}'s {adj2} {adj} {noun}
pub fn rare_title(rng: &mut ChaCha8Rng, parts: &TitleParts<'_>) -> String {
    let TitleParts {
        adj,
        noun,
        compound,
        name,
        adj2,
    } = parts;
    match weighted_choose(rng, &[20, 15, 15, 15, 10, 10, 10, 5]) {
        0 => format!("{adj} {compound} of {adj2}"),
        1 => format!("{compound} of the {adj}"),
        2 => format!("The {adj} {compound}"),
        3 => format!("{name}'s {adj} {compound}"),
        4 => format!("{adj2} {compound} of {adj}"),
        5 => format!("The {adj} {compound} of {name}"),
        6 => format!("{compound}, {adj} and {adj2}"),
        _ => format!("{name}'s {adj2} {adj} {noun}"),
    }
}

/// Weighted template selection for Legendary rarity.
///
/// 30% {name}, the {adj2}, 25% The {adj2} {name},
/// 20% {name}'s {adj} {compound}, 15% the {adj} {compound} of {name},
/// 10% {name}'s {adj2} {noun}
pub fn legendary_title(rng: &mut ChaCha8Rng, parts: &TitleParts<'_>) -> String {
    let TitleParts {
        adj,
        noun,
        compound,
        name,
        adj2,
    } = parts;
    match weighted_choose(rng, &[30, 25, 20, 15, 10]) {
        0 => format!("{name}, the {adj2}"),
        1 => format!("The {adj2} {name}"),
        2 => format!("{name}'s {adj} {compound}"),
        3 => format!("The {adj} {compound} of {name}"),
        _ => format!("{name}'s {adj2} {noun}"),
    }
}
