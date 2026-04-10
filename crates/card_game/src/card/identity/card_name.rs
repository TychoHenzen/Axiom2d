// EVOLVE-BLOCK-START
use rand::SeedableRng;
use rand::seq::IndexedRandom;
use rand_chacha::ChaCha8Rng;

use crate::card::identity::name_pools::{
    AspectCluster, TitleParts, adjective_pool, aspect_cluster, common_title, generate_compound,
    generate_proper_noun, legendary_title, noun_pool, rare_title,
};
use crate::card::identity::signature::{Aspect, CardSignature, Rarity};
use crate::card::identity::signature_profile::{SignatureProfile, Tier};

#[derive(Debug, Clone, PartialEq)]
pub struct CardName {
    pub title: String,
    pub subtitle: String,
}

pub fn generate_card_name(profile: &SignatureProfile, signature: &CardSignature) -> CardName {
    let mut rng = rng_from_signature(signature);
    let title = build_title(profile, &mut rng);
    let subtitle = build_subtitle(profile);
    CardName { title, subtitle }
}

fn rng_from_signature(signature: &CardSignature) -> ChaCha8Rng {
    let axes = signature.axes();
    let seed = axes.iter().enumerate().fold(0u64, |acc, (i, &v)| {
        let bits = u64::from(v.to_bits());
        acc ^ bits
            .wrapping_mul(0x9e37_79b9_7f4a_7c15)
            .wrapping_add(i as u64)
    });
    ChaCha8Rng::seed_from_u64(seed)
}

fn build_title(profile: &SignatureProfile, rng: &mut ChaCha8Rng) -> String {
    let archetype = profile.archetype.as_deref().unwrap_or("Unknown");
    let dominant_cluster = profile.dominant_axis.map_or(AspectCluster::Physical, |el| {
        aspect_cluster(profile.aspects[el as usize])
    });
    let nouns = noun_pool(archetype, dominant_cluster);
    let noun = nouns.choose(rng).copied().unwrap_or("Relic");

    let dominant_aspect = profile
        .dominant_axis
        .map_or(Aspect::Solid, |el| profile.aspects[el as usize]);
    let adjectives = adjective_pool(dominant_aspect);
    let adj = adjectives.choose(rng).copied().unwrap_or("Ancient");

    let compound = generate_compound(rng, archetype, dominant_cluster);
    let name = generate_proper_noun(rng);

    match profile.rarity {
        Rarity::Common | Rarity::Uncommon => {
            let parts = TitleParts {
                adj,
                noun,
                compound: &compound,
                name: &name,
                adj2: adj,
            };
            common_title(rng, &parts)
        }
        Rarity::Rare | Rarity::Epic => {
            let secondary_adj = profile.secondary_axis.map(|el| {
                let pool = adjective_pool(profile.aspects[el as usize]);
                pool.choose(rng).copied().unwrap_or("Ancient")
            });
            let adj2 = secondary_adj.unwrap_or(adj);
            let parts = TitleParts {
                adj,
                noun,
                compound: &compound,
                name: &name,
                adj2,
            };
            rare_title(rng, &parts)
        }
        Rarity::Legendary => {
            let adj2_pool = adjectives
                .iter()
                .filter(|&&a| a != adj)
                .copied()
                .collect::<Vec<_>>();
            let epithet = adj2_pool.choose(rng).copied().unwrap_or(adj);
            let parts = TitleParts {
                adj,
                noun,
                compound: &compound,
                name: &name,
                adj2: epithet,
            };
            legendary_title(rng, &parts)
        }
    }
}

fn build_subtitle(profile: &SignatureProfile) -> String {
    let tier = profile
        .dominant_axis
        .map_or(Tier::Dormant, |el| profile.tiers[el as usize]);

    let cluster = profile.dominant_axis.map_or(AspectCluster::Physical, |el| {
        aspect_cluster(profile.aspects[el as usize])
    });

    subtitle_phrase(tier, cluster).to_string()
}

pub fn subtitle_phrase(tier: Tier, cluster: AspectCluster) -> &'static str {
    match (tier, cluster) {
        (Tier::Dormant, AspectCluster::Physical) => "Resting beneath cold iron",
        (Tier::Dormant, AspectCluster::Elemental) => "A faint ember, barely lit",
        (Tier::Dormant, AspectCluster::Nature) => "Seeds sleeping under frost",
        (Tier::Dormant, AspectCluster::Arcane) => "A whisper between worlds",
        (Tier::Active, AspectCluster::Physical) => "Tempered by steady hands",
        (Tier::Active, AspectCluster::Elemental) => "Touched by waking light",
        (Tier::Active, AspectCluster::Nature) => "Roots stirring in warm soil",
        (Tier::Active, AspectCluster::Arcane) => "Drawn through a thinning veil",
        (Tier::Intense, AspectCluster::Physical) => "Forged in unyielding stone",
        (Tier::Intense, AspectCluster::Elemental) => "Wreathed in endless flame",
        (Tier::Intense, AspectCluster::Nature) => "Consumed by wild overgrowth",
        (Tier::Intense, AspectCluster::Arcane) => "Torn from the fabric of space",
    }
}
// EVOLVE-BLOCK-END
