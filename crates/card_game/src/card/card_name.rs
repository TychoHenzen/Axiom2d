use rand::SeedableRng;
use rand::seq::SliceRandom;
use rand_chacha::ChaCha8Rng;

use crate::card::signature::{Aspect, CardSignature, Rarity};
use crate::card::signature_profile::{SignatureProfile, Tier};

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
        let bits = v.to_bits() as u64;
        acc ^ bits
            .wrapping_mul(0x9e37_79b9_7f4a_7c15)
            .wrapping_add(i as u64)
    });
    ChaCha8Rng::seed_from_u64(seed)
}

fn build_title(profile: &SignatureProfile, rng: &mut ChaCha8Rng) -> String {
    let archetype = profile.archetype.as_deref().unwrap_or("Unknown");
    let nouns = noun_pool(archetype);
    let noun = nouns.choose(rng).copied().unwrap_or("Relic");

    let dominant_aspect = profile
        .dominant_axis
        .map(|el| profile.aspects[el as usize])
        .unwrap_or(Aspect::Solid);
    let adjectives = adjective_pool(dominant_aspect);
    let adj = adjectives.choose(rng).copied().unwrap_or("Ancient");

    match profile.rarity {
        Rarity::Common | Rarity::Uncommon => format!("{adj} {noun}"),
        Rarity::Rare | Rarity::Epic => {
            let secondary_adj = profile.secondary_axis.map(|el| {
                let pool = adjective_pool(profile.aspects[el as usize]);
                pool.choose(rng).copied().unwrap_or("Ancient")
            });
            match secondary_adj {
                Some(sec) => format!("{adj} {noun} of {sec}"),
                None => format!("{noun} of {adj}"),
            }
        }
        Rarity::Legendary => {
            let epithet_adj = adjectives
                .iter()
                .filter(|&&a| a != adj)
                .copied()
                .collect::<Vec<_>>();
            let epithet = epithet_adj.choose(rng).copied().unwrap_or(adj);
            format!("{noun}, the {epithet}")
        }
    }
}

fn build_subtitle(profile: &SignatureProfile) -> String {
    let tier = profile
        .dominant_axis
        .map(|el| tier_label(profile.tiers[el as usize]))
        .unwrap_or("Dormant");

    let aspect = profile
        .dominant_axis
        .map(|el| aspect_label(profile.aspects[el as usize]))
        .unwrap_or("Unknown");

    match &profile.archetype {
        Some(archetype) => format!("{tier} {aspect} {archetype}"),
        None => format!("{tier} {aspect}"),
    }
}

fn tier_label(tier: Tier) -> &'static str {
    match tier {
        Tier::Dormant => "Dormant",
        Tier::Active => "Active",
        Tier::Intense => "Intense",
    }
}

fn aspect_label(aspect: Aspect) -> &'static str {
    match aspect {
        Aspect::Solid => "Solid",
        Aspect::Fragile => "Fragile",
        Aspect::Heat => "Heat",
        Aspect::Cold => "Cold",
        Aspect::Order => "Order",
        Aspect::Chaos => "Chaos",
        Aspect::Light => "Light",
        Aspect::Dark => "Dark",
        Aspect::Change => "Change",
        Aspect::Stasis => "Stasis",
        Aspect::Force => "Force",
        Aspect::Calm => "Calm",
        Aspect::Growth => "Growth",
        Aspect::Decay => "Decay",
        Aspect::Expansion => "Expansion",
        Aspect::Contraction => "Contraction",
    }
}

fn noun_pool(archetype: &str) -> &'static [&'static str] {
    match archetype {
        "Weapon" => &[
            "Blade", "Edge", "Fang", "Shard", "Talon", "Spike", "Cleaver", "Barb", "Skewer",
            "Sliver", "Reaver", "Thorn",
        ],
        "Shield" => &[
            "Bulwark", "Barrier", "Aegis", "Ward", "Plate", "Rampart", "Veil", "Mantle", "Bastion",
            "Crest", "Guard", "Shell",
        ],
        "Spell" => &[
            "Rune",
            "Glyph",
            "Hex",
            "Sigil",
            "Surge",
            "Pulse",
            "Bolt",
            "Rift",
            "Flare",
            "Weave",
            "Mark",
            "Invocation",
        ],
        "Healer" => &[
            "Salve", "Balm", "Tincture", "Poultice", "Remedy", "Elixir", "Draught", "Infusion",
            "Compress", "Liniment", "Boon", "Mend",
        ],
        "Scout" => &[
            "Vantage", "Quarry", "Trail", "Bearing", "Passage", "Sign", "Trace", "Mark", "Path",
            "Stride", "Omen", "Footing",
        ],
        "Artifact" => &[
            "Relic", "Shard", "Totem", "Orb", "Focus", "Effigy", "Token", "Idol", "Charm",
            "Trinket", "Amulet", "Catalyst",
        ],
        _ => &[
            "Relic", "Fragment", "Essence", "Spark", "Echo", "Remnant", "Token", "Shard", "Trace",
            "Wisp", "Vestige", "Mote",
        ],
    }
}

fn adjective_pool(aspect: Aspect) -> &'static [&'static str] {
    match aspect {
        Aspect::Solid => &[
            "Iron",
            "Stone",
            "Unyielding",
            "Forged",
            "Tempered",
            "Rigid",
            "Steadfast",
            "Unbroken",
            "Solid",
            "Heavy",
            "Reinforced",
            "Stubborn",
        ],
        Aspect::Fragile => &[
            "Brittle",
            "Cracked",
            "Hollow",
            "Frail",
            "Splintered",
            "Shattered",
            "Thin",
            "Worn",
            "Faded",
            "Delicate",
            "Fractured",
            "Withered",
        ],
        Aspect::Heat => &[
            "Blazing",
            "Scorched",
            "Ember",
            "Molten",
            "Searing",
            "Scalding",
            "Smoldering",
            "Incandescent",
            "Fervent",
            "Glowing",
            "Burning",
            "Ashen",
        ],
        Aspect::Cold => &[
            "Frozen",
            "Glacial",
            "Frost",
            "Icy",
            "Bitter",
            "Pale",
            "Numbing",
            "Crisp",
            "Winter",
            "Still",
            "Chill",
            "Crystalline",
        ],
        Aspect::Order => &[
            "Lawful",
            "Carved",
            "True",
            "Exact",
            "Measured",
            "Precise",
            "Aligned",
            "Structured",
            "Ordained",
            "Balanced",
            "Steady",
            "Proper",
        ],
        Aspect::Chaos => &[
            "Wild",
            "Frenzied",
            "Jagged",
            "Warped",
            "Volatile",
            "Errant",
            "Twisted",
            "Manic",
            "Riotous",
            "Fractious",
            "Unbound",
            "Scattered",
        ],
        Aspect::Light => &[
            "Radiant",
            "Gleaming",
            "Gilded",
            "Luminous",
            "Brilliant",
            "Shining",
            "Aureate",
            "Blessed",
            "Dawnlit",
            "Pristine",
            "Clear",
            "Celestial",
        ],
        Aspect::Dark => &[
            "Shadow", "Dread", "Ashen", "Murky", "Cursed", "Blighted", "Umbral", "Hollow", "Sable",
            "Tainted", "Veiled", "Grim",
        ],
        Aspect::Change => &[
            "Shifting",
            "Fluid",
            "Mercurial",
            "Mutable",
            "Drifting",
            "Restless",
            "Turning",
            "Adaptive",
            "Flowing",
            "Transient",
            "Evolving",
            "Flux",
        ],
        Aspect::Stasis => &[
            "Sealed",
            "Resting",
            "Preserved",
            "Dormant",
            "Locked",
            "Suspended",
            "Inert",
            "Crystallized",
            "Timeless",
            "Still",
            "Bound",
            "Petrified",
        ],
        Aspect::Force => &[
            "Crushing",
            "Driving",
            "Kinetic",
            "Forceful",
            "Surging",
            "Impact",
            "Powerful",
            "Crashing",
            "Thundering",
            "Relentless",
            "Brutal",
            "Fierce",
        ],
        Aspect::Calm => &[
            "Serene", "Tranquil", "Quiet", "Hushed", "Gentle", "Soft", "Peaceful", "Drifting",
            "Smooth", "Measured", "Restful", "Subtle",
        ],
        Aspect::Growth => &[
            "Verdant",
            "Thriving",
            "Rising",
            "Blooming",
            "Burgeoning",
            "Living",
            "Sprouting",
            "Vital",
            "Lush",
            "Fertile",
            "Abundant",
            "Flourishing",
        ],
        Aspect::Decay => &[
            "Rotting",
            "Rusted",
            "Corroded",
            "Crumbling",
            "Festering",
            "Withered",
            "Putrid",
            "Gnawed",
            "Blighted",
            "Moldering",
            "Ancient",
            "Ravaged",
        ],
        Aspect::Expansion => &[
            "Vast",
            "Reaching",
            "Sweeping",
            "Boundless",
            "Spreading",
            "Wide",
            "Open",
            "Soaring",
            "Distant",
            "Expanding",
            "Unfolding",
            "Broad",
        ],
        Aspect::Contraction => &[
            "Compressed",
            "Dense",
            "Tight",
            "Focused",
            "Collapsed",
            "Compact",
            "Condensed",
            "Drawn",
            "Closed",
            "Narrow",
            "Coiled",
            "Concentrated",
        ],
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::card::base_type::BaseCardTypeRegistry;
    use crate::card::signature::CardSignature;
    use crate::card::signature_profile::SignatureProfile;

    fn weapon_fixture() -> (SignatureProfile, CardSignature) {
        let mut registry = BaseCardTypeRegistry::default();
        crate::card::base_type::populate_default_types(&mut registry);
        let sig = CardSignature::new([0.8, 0.3, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let profile = SignatureProfile::new(&sig, &registry);
        (profile, sig)
    }

    #[test]
    fn when_same_signature_used_twice_then_title_is_identical() {
        // Arrange
        let (profile, sig) = weapon_fixture();

        // Act
        let name1 = generate_card_name(&profile, &sig);
        let name2 = generate_card_name(&profile, &sig);

        // Assert
        assert_eq!(
            name1.title, name2.title,
            "same signature must produce same title"
        );
    }

    #[test]
    fn when_two_distinct_signatures_used_then_titles_are_different() {
        // Arrange
        let mut registry = BaseCardTypeRegistry::default();
        crate::card::base_type::populate_default_types(&mut registry);
        let sig_a = CardSignature::new([0.0; 8]);
        let sig_b = CardSignature::new([1.0; 8]);
        let profile_a = SignatureProfile::new(&sig_a, &registry);
        let profile_b = SignatureProfile::new(&sig_b, &registry);

        // Act
        let name_a = generate_card_name(&profile_a, &sig_a);
        let name_b = generate_card_name(&profile_b, &sig_b);

        // Assert
        assert_ne!(
            name_a.title, name_b.title,
            "distinct signatures should produce different titles"
        );
    }

    #[test]
    fn when_profile_has_dominant_axis_and_archetype_then_subtitle_is_tier_aspect_archetype() {
        // Arrange — Solidum at 0.8 → Intense tier, positive → Solid aspect, archetype Weapon
        let (profile, sig) = weapon_fixture();

        // Act
        let name = generate_card_name(&profile, &sig);

        // Assert
        assert_eq!(name.subtitle, "Intense Solid Weapon");
    }

    #[test]
    fn when_profile_has_no_archetype_then_subtitle_is_tier_aspect_only() {
        // Arrange — Lumines at 0.5 → Active tier, positive → Light aspect, no archetype
        let sig = CardSignature::new([0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.0]);
        let profile = SignatureProfile::without_archetype(&sig);

        // Act
        let name = generate_card_name(&profile, &sig);

        // Assert
        assert_eq!(name.subtitle, "Active Light");
    }

    #[test]
    fn when_all_axes_zero_then_subtitle_uses_dormant_tier() {
        // Arrange — all zeros → dominant=Solidum, tier=Dormant, aspect=Fragile (0.0 ≤ 0)
        let sig = CardSignature::new([0.0; 8]);
        let profile = SignatureProfile::without_archetype(&sig);

        // Act
        let name = generate_card_name(&profile, &sig);

        // Assert
        assert!(
            name.subtitle.starts_with("Dormant"),
            "zero signature should have Dormant tier"
        );
    }

    #[test]
    fn when_rarity_is_common_then_title_uses_two_word_template() {
        // Arrange — all zeros → raw_score=0 → Common
        let sig = CardSignature::new([0.0; 8]);
        let profile = SignatureProfile::without_archetype(&sig);
        assert_eq!(profile.rarity, Rarity::Common);

        // Act
        let name = generate_card_name(&profile, &sig);

        // Assert
        let word_count = name.title.split_whitespace().count();
        assert_eq!(
            word_count, 2,
            "Common title '{}' should be exactly 2 words",
            name.title
        );
    }

    #[test]
    fn when_rarity_is_uncommon_then_title_uses_two_word_template() {
        // Arrange — two axes at 0.5 → raw=1.0 → normalized≈0.315 → Uncommon
        let sig = CardSignature::new([0.5, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let profile = SignatureProfile::without_archetype(&sig);
        assert_eq!(profile.rarity, Rarity::Uncommon);

        // Act
        let name = generate_card_name(&profile, &sig);

        // Assert
        let word_count = name.title.split_whitespace().count();
        assert_eq!(
            word_count, 2,
            "Uncommon title '{}' should be exactly 2 words",
            name.title
        );
    }

    #[test]
    fn when_rarity_is_rare_then_title_uses_three_or_four_word_template() {
        // Arrange — four axes at 0.6 → raw=2.4 → normalized≈0.557 → Rare
        let sig = CardSignature::new([0.6, 0.6, 0.6, 0.6, 0.0, 0.0, 0.0, 0.0]);
        let profile = SignatureProfile::without_archetype(&sig);
        assert_eq!(profile.rarity, Rarity::Rare);

        // Act
        let name = generate_card_name(&profile, &sig);

        // Assert
        let word_count = name.title.split_whitespace().count();
        assert!(
            (3..=4).contains(&word_count),
            "Rare title '{}' should be 3-4 words, got {word_count}",
            name.title
        );
    }

    #[test]
    fn when_rarity_is_epic_then_title_uses_three_or_four_word_template() {
        // Arrange — six axes at 0.8 → raw=4.8 → normalized≈0.800 → Epic
        let sig = CardSignature::new([0.8, 0.8, 0.8, 0.8, 0.8, 0.8, 0.0, 0.0]);
        let profile = SignatureProfile::without_archetype(&sig);
        assert_eq!(profile.rarity, Rarity::Epic);

        // Act
        let name = generate_card_name(&profile, &sig);

        // Assert
        let word_count = name.title.split_whitespace().count();
        assert!(
            (3..=4).contains(&word_count),
            "Epic title '{}' should be 3-4 words, got {word_count}",
            name.title
        );
    }

    #[test]
    fn when_rarity_is_legendary_then_title_uses_curated_format_with_comma() {
        // Arrange — all axes at 1.0 → Legendary
        let sig = CardSignature::new([1.0; 8]);
        let profile = SignatureProfile::without_archetype(&sig);
        assert_eq!(profile.rarity, Rarity::Legendary);

        // Act
        let name = generate_card_name(&profile, &sig);

        // Assert
        assert!(
            name.title.contains(','),
            "Legendary title '{}' should contain a comma",
            name.title
        );
        assert!(
            name.title.contains("the"),
            "Legendary title '{}' should contain 'the'",
            name.title
        );
    }

    #[test]
    fn when_archetype_is_weapon_then_title_noun_differs_from_spell() {
        // Arrange
        let mut registry = BaseCardTypeRegistry::default();
        crate::card::base_type::populate_default_types(&mut registry);
        let sig = CardSignature::new([0.5, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let mut weapon_profile = SignatureProfile::new(&sig, &registry);
        weapon_profile.archetype = Some("Weapon".to_string());
        let mut spell_profile = SignatureProfile::new(&sig, &registry);
        spell_profile.archetype = Some("Spell".to_string());

        // Act
        let weapon_name = generate_card_name(&weapon_profile, &sig);
        let spell_name = generate_card_name(&spell_profile, &sig);

        // Assert
        assert_ne!(
            weapon_name.title, spell_name.title,
            "different archetypes should produce different titles"
        );
    }

    #[test]
    fn when_archetype_is_none_then_title_uses_generic_noun_pool() {
        // Arrange
        let sig = CardSignature::new([0.5, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let profile = SignatureProfile::without_archetype(&sig);

        // Act
        let name = generate_card_name(&profile, &sig);

        // Assert
        assert!(
            !name.title.is_empty(),
            "no-archetype card should still get a title"
        );
    }

    #[test]
    fn when_dominant_aspect_is_heat_vs_cold_then_titles_differ() {
        // Arrange — Febris +0.8 → Heat, Febris -0.8 → Cold
        let sig_heat = CardSignature::new([0.0, 0.8, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let sig_cold = CardSignature::new([0.0, -0.8, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let profile_heat = SignatureProfile::without_archetype(&sig_heat);
        let profile_cold = SignatureProfile::without_archetype(&sig_cold);

        // Act
        let name_heat = generate_card_name(&profile_heat, &sig_heat);
        let name_cold = generate_card_name(&profile_cold, &sig_cold);

        // Assert
        assert_ne!(
            name_heat.title, name_cold.title,
            "Heat vs Cold aspect should produce different titles"
        );
    }

    #[test]
    fn when_adjective_pool_queried_for_every_aspect_then_non_empty() {
        // Arrange
        let all_aspects = [
            Aspect::Solid,
            Aspect::Fragile,
            Aspect::Heat,
            Aspect::Cold,
            Aspect::Order,
            Aspect::Chaos,
            Aspect::Light,
            Aspect::Dark,
            Aspect::Change,
            Aspect::Stasis,
            Aspect::Force,
            Aspect::Calm,
            Aspect::Growth,
            Aspect::Decay,
            Aspect::Expansion,
            Aspect::Contraction,
        ];

        // Act & Assert
        for aspect in all_aspects {
            let pool = adjective_pool(aspect);
            assert!(
                !pool.is_empty(),
                "adjective pool for {aspect:?} must not be empty"
            );
        }
    }

    #[test]
    fn when_noun_pool_queried_for_every_archetype_then_non_empty() {
        // Arrange
        let archetypes = ["Weapon", "Spell", "Shield", "Healer", "Scout"];

        // Act & Assert
        for archetype in archetypes {
            let pool = noun_pool(archetype);
            assert!(
                !pool.is_empty(),
                "noun pool for '{archetype}' must not be empty"
            );
        }
        // Also check the fallback (None/unknown archetype)
        let fallback = noun_pool("UnknownType");
        assert!(!fallback.is_empty(), "fallback noun pool must not be empty");
    }

    #[test]
    fn when_secondary_axis_present_and_rare_then_title_has_more_tokens_than_without() {
        // Arrange — Rare with secondary (ratio < 1.5): [0.6, 0.5, 0.6, 0.5, ...]
        let sig_with_sec = CardSignature::new([0.6, 0.5, 0.6, 0.5, 0.0, 0.0, 0.0, 0.0]);
        let profile_with_sec = SignatureProfile::without_archetype(&sig_with_sec);
        assert!(
            profile_with_sec.secondary_axis.is_some(),
            "fixture must have secondary axis"
        );
        assert_eq!(profile_with_sec.rarity, Rarity::Rare);

        // Rare without secondary (ratio > 1.5): dominant=0.9, all others low
        let sig_no_sec = CardSignature::new([0.9, 0.2, 0.2, 0.2, 0.2, 0.2, 0.2, 0.2]);
        let profile_no_sec = SignatureProfile::without_archetype(&sig_no_sec);
        assert!(
            profile_no_sec.secondary_axis.is_none(),
            "fixture must not have secondary axis"
        );

        // Act
        let name_with = generate_card_name(&profile_with_sec, &sig_with_sec);
        let name_without = generate_card_name(&profile_no_sec, &sig_no_sec);

        // Assert
        let words_with = name_with.title.split_whitespace().count();
        let words_without = name_without.title.split_whitespace().count();
        assert!(
            words_with > words_without,
            "with secondary ({words_with} words: '{}') should have more tokens than without ({words_without} words: '{}')",
            name_with.title,
            name_without.title
        );
    }

    #[test]
    fn when_secondary_axis_absent_and_rare_then_title_still_generated() {
        // Arrange — dominant=0.9, others at 0.2 → ratio > 1.5 → no secondary
        let sig = CardSignature::new([0.9, 0.2, 0.2, 0.2, 0.2, 0.2, 0.2, 0.2]);
        let profile = SignatureProfile::without_archetype(&sig);
        assert!(profile.secondary_axis.is_none());

        // Act
        let name = generate_card_name(&profile, &sig);

        // Assert
        assert!(!name.title.is_empty());
        let word_count = name.title.split_whitespace().count();
        assert!(
            word_count >= 2,
            "title should have at least 2 words, got {word_count}"
        );
    }

    #[test]
    fn when_multiple_rare_signatures_then_all_titles_are_three_or_four_words() {
        // Arrange — 10 distinct Rare signatures
        let rare_sigs = [
            [0.6, 0.6, 0.6, 0.6, 0.0, 0.0, 0.0, 0.0],
            [0.0, 0.6, 0.6, 0.6, 0.6, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.6, 0.6, 0.6, 0.6, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.6, 0.6, 0.6, 0.6, 0.0],
            [0.0, 0.0, 0.0, 0.0, 0.6, 0.6, 0.6, 0.6],
            [0.7, 0.5, 0.5, 0.5, 0.0, 0.0, 0.0, 0.0],
            [0.5, 0.7, 0.5, 0.5, 0.0, 0.0, 0.0, 0.0],
            [0.5, 0.5, 0.7, 0.5, 0.0, 0.0, 0.0, 0.0],
            [-0.6, -0.6, -0.6, -0.6, 0.0, 0.0, 0.0, 0.0],
            [0.8, 0.4, 0.4, 0.4, 0.0, 0.0, 0.0, 0.0],
        ];

        for axes in &rare_sigs {
            let sig = CardSignature::new(*axes);
            let profile = SignatureProfile::without_archetype(&sig);
            assert_eq!(
                profile.rarity,
                Rarity::Rare,
                "fixture {axes:?} must be Rare"
            );

            // Act
            let name = generate_card_name(&profile, &sig);

            // Assert
            let word_count = name.title.split_whitespace().count();
            assert!(
                (3..=4).contains(&word_count),
                "Rare title '{}' (from {axes:?}) should be 3-4 words, got {word_count}",
                name.title
            );
        }
    }

    #[test]
    fn when_multiple_legendary_signatures_then_all_titles_have_comma_and_the() {
        // Arrange — 5 distinct Legendary signatures
        let legendary_sigs = [
            [1.0; 8],
            [0.95, 0.95, 0.95, 0.95, 0.95, 0.95, 0.95, 0.95],
            [-1.0; 8],
            [1.0, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0, -1.0],
            [0.9, 1.0, 0.9, 1.0, 0.9, 1.0, 0.9, 1.0],
        ];

        for axes in &legendary_sigs {
            let sig = CardSignature::new(*axes);
            let profile = SignatureProfile::without_archetype(&sig);
            assert_eq!(
                profile.rarity,
                Rarity::Legendary,
                "fixture {axes:?} must be Legendary"
            );

            // Act
            let name = generate_card_name(&profile, &sig);

            // Assert
            assert!(
                name.title.contains(','),
                "Legendary title '{}' (from {axes:?}) should contain a comma",
                name.title
            );
            assert!(
                name.title.contains("the"),
                "Legendary title '{}' (from {axes:?}) should contain 'the'",
                name.title
            );
        }
    }

    #[test]
    fn when_generating_card_name_then_result_has_non_empty_title_and_subtitle() {
        // Arrange
        let (profile, sig) = weapon_fixture();

        // Act
        let name = generate_card_name(&profile, &sig);

        // Assert
        assert!(!name.title.is_empty());
        assert!(!name.subtitle.is_empty());
    }
}
