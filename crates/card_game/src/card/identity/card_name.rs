use rand::SeedableRng;
use rand::seq::SliceRandom;
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::card::identity::base_type::BaseCardTypeRegistry;
    use crate::card::identity::signature::CardSignature;
    use crate::card::identity::signature_profile::SignatureProfile;

    fn make_registry() -> BaseCardTypeRegistry {
        let mut registry = BaseCardTypeRegistry::default();
        crate::card::identity::base_type::populate_default_types(&mut registry);
        registry
    }

    fn weapon_fixture() -> (SignatureProfile, CardSignature) {
        let registry = make_registry();
        let sig = CardSignature::new([0.8, 0.3, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let profile = SignatureProfile::new(&sig, &registry);
        (profile, sig)
    }

    /// Returns true if the title matches one of the Common/Uncommon templates.
    fn matches_common_template(title: &str) -> bool {
        let has_possessive = title.contains("'s ");
        let words: Vec<&str> = title.split_whitespace().collect();
        match words.len() {
            // {adj} {noun} or {adj} {compound}
            2 => true,
            // {noun} of {adj}, The {adj} {noun}
            3 if words[0] == "The" || words[1] == "of" => true,
            // {name}'s {adj} {noun}
            3 if has_possessive => true,
            // The {adj} {noun} of {name}
            5 if words[0] == "The" && words[3] == "of" => true,
            _ => false,
        }
    }

    /// Returns true if the title matches one of the Rare/Epic templates.
    fn matches_rare_template(title: &str) -> bool {
        let has_possessive = title.contains("'s ");
        let words: Vec<&str> = title.split_whitespace().collect();
        // {compound}, {adj} and {adj2}
        if title.contains(',') && title.contains(" and ") {
            return true;
        }
        #[allow(clippy::match_same_arms)]
        match words.len() {
            // The {adj} {compound}
            3 if words[0] == "The" => true,
            // {name}'s {adj} {compound}
            3 if has_possessive => true,
            // {adj} {compound} of {adj2}, {adj2} {compound} of {adj},
            // {compound} of the {adj}
            4 if words[1] == "of" || words[2] == "of" => true,
            // {name}'s {adj2} {adj} {noun}
            4 if has_possessive => true,
            // The {adj} {compound} of {name}
            5 if words[0] == "The" && words[3] == "of" => true,
            _ => false,
        }
    }

    /// Returns true if the title matches one of the Legendary templates.
    fn matches_legendary_template(title: &str) -> bool {
        let has_possessive = title.contains("'s ");
        // {name}, the {epithet}
        if title.contains(", the ") {
            return true;
        }
        // The {epithet} {name}
        if title.starts_with("The ") && !title.contains(',') && !title.contains(" of ") {
            return true;
        }
        // {name}'s {adj} {compound} or {name}'s {epithet} {noun}
        if has_possessive && !title.contains(" of ") {
            return true;
        }
        // The {adj} {compound} of {name}
        if title.starts_with("The ") && title.contains(" of ") {
            return true;
        }
        false
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
        let registry = make_registry();
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
    fn when_profile_has_dominant_axis_and_archetype_then_subtitle_is_lore_phrase() {
        // Arrange
        let (profile, sig) = weapon_fixture();

        // Act
        let name = generate_card_name(&profile, &sig);

        // Assert
        let expected = subtitle_phrase(Tier::Intense, AspectCluster::Physical);
        assert_eq!(name.subtitle, expected);
    }

    #[test]
    fn when_profile_has_no_archetype_then_subtitle_is_lore_phrase() {
        // Arrange
        let sig = CardSignature::new([0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.0]);
        let profile = SignatureProfile::without_archetype(&sig);

        // Act
        let name = generate_card_name(&profile, &sig);

        // Assert
        let expected = subtitle_phrase(Tier::Active, AspectCluster::Elemental);
        assert_eq!(name.subtitle, expected);
    }

    #[test]
    fn when_all_axes_zero_then_subtitle_uses_dormant_phrase() {
        // Arrange
        let sig = CardSignature::new([0.0; 8]);
        let profile = SignatureProfile::without_archetype(&sig);

        // Act
        let name = generate_card_name(&profile, &sig);

        // Assert
        let expected = subtitle_phrase(Tier::Dormant, AspectCluster::Physical);
        assert_eq!(name.subtitle, expected);
    }

    #[test]
    fn when_rarity_is_common_then_title_matches_one_of_four_templates() {
        // Arrange
        let sig = CardSignature::new([0.0; 8]);
        let profile = SignatureProfile::without_archetype(&sig);
        assert_eq!(profile.rarity, Rarity::Common);

        // Act
        let name = generate_card_name(&profile, &sig);

        // Assert
        assert!(
            matches_common_template(&name.title),
            "Common title '{}' must match a common template pattern",
            name.title
        );
    }

    #[test]
    fn when_rarity_is_uncommon_then_title_matches_one_of_four_templates() {
        // Arrange
        let sig = CardSignature::new([0.5, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let profile = SignatureProfile::without_archetype(&sig);
        assert_eq!(profile.rarity, Rarity::Uncommon);

        // Act
        let name = generate_card_name(&profile, &sig);

        // Assert
        assert!(
            matches_common_template(&name.title),
            "Uncommon title '{}' must match a common template pattern",
            name.title
        );
    }

    #[test]
    fn when_rarity_is_rare_then_title_matches_one_of_five_templates() {
        // Arrange
        let sig = CardSignature::new([0.6, 0.6, 0.6, 0.6, 0.0, 0.0, 0.0, 0.0]);
        let profile = SignatureProfile::without_archetype(&sig);
        assert_eq!(profile.rarity, Rarity::Rare);

        // Act
        let name = generate_card_name(&profile, &sig);

        // Assert
        assert!(
            matches_rare_template(&name.title),
            "Rare title '{}' must match a rare template pattern",
            name.title
        );
    }

    #[test]
    fn when_rarity_is_epic_then_title_matches_one_of_five_templates() {
        // Arrange
        let sig = CardSignature::new([0.8, 0.8, 0.8, 0.8, 0.8, 0.8, 0.0, 0.0]);
        let profile = SignatureProfile::without_archetype(&sig);
        assert_eq!(profile.rarity, Rarity::Epic);

        // Act
        let name = generate_card_name(&profile, &sig);

        // Assert
        assert!(
            matches_rare_template(&name.title),
            "Epic title '{}' must match a rare template pattern",
            name.title
        );
    }

    #[test]
    fn when_rarity_is_legendary_then_title_matches_one_of_two_templates() {
        // Arrange
        let sig = CardSignature::new([1.0; 8]);
        let profile = SignatureProfile::without_archetype(&sig);
        assert_eq!(profile.rarity, Rarity::Legendary);

        // Act
        let name = generate_card_name(&profile, &sig);

        // Assert
        assert!(
            matches_legendary_template(&name.title),
            "Legendary title '{}' must match a legendary template pattern",
            name.title
        );
    }

    #[test]
    fn when_archetype_is_weapon_then_title_noun_differs_from_spell() {
        // Arrange
        let registry = make_registry();
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
        // Arrange
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
            let pool = noun_pool(archetype, AspectCluster::Physical);
            assert!(
                !pool.is_empty(),
                "noun pool for '{archetype}' must not be empty"
            );
        }
        let fallback = noun_pool("UnknownType", AspectCluster::Physical);
        assert!(!fallback.is_empty(), "fallback noun pool must not be empty");
    }

    #[test]
    fn when_noun_pool_queried_for_weapon_physical_vs_elemental_then_pools_differ() {
        // Arrange
        let archetype = "Weapon";

        // Act
        let physical_pool = noun_pool(archetype, AspectCluster::Physical);
        let elemental_pool = noun_pool(archetype, AspectCluster::Elemental);

        // Assert
        assert_ne!(
            physical_pool, elemental_pool,
            "Physical and Elemental clusters must return different noun pools for '{archetype}'"
        );
    }

    #[test]
    fn when_noun_pool_queried_for_every_archetype_and_cluster_then_non_empty() {
        // Arrange
        let archetypes = [
            "Weapon",
            "Shield",
            "Spell",
            "Healer",
            "Scout",
            "Artifact",
            "UnknownType",
        ];
        let clusters = [
            AspectCluster::Physical,
            AspectCluster::Elemental,
            AspectCluster::Nature,
            AspectCluster::Arcane,
        ];

        // Act & Assert
        for archetype in archetypes {
            for cluster in clusters {
                let pool = noun_pool(archetype, cluster);
                assert!(
                    !pool.is_empty(),
                    "noun pool for ('{archetype}', {cluster:?}) must not be empty"
                );
            }
        }
    }

    #[test]
    fn when_multiple_rare_signatures_then_all_titles_match_rare_templates() {
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
            assert!(
                matches_rare_template(&name.title),
                "Rare title '{}' (from {axes:?}) must match a rare template",
                name.title
            );
        }
    }

    #[test]
    fn when_multiple_legendary_signatures_then_all_titles_match_legendary_templates() {
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
                matches_legendary_template(&name.title),
                "Legendary title '{}' (from {axes:?}) must match a legendary template",
                name.title
            );
        }
    }

    #[test]
    fn when_subtitle_phrase_queried_for_all_tier_and_cluster_combinations_then_all_non_empty() {
        // Arrange
        let tiers = [Tier::Dormant, Tier::Active, Tier::Intense];
        let clusters = [
            AspectCluster::Physical,
            AspectCluster::Elemental,
            AspectCluster::Nature,
            AspectCluster::Arcane,
        ];

        // Act & Assert
        for tier in tiers {
            for cluster in clusters {
                let phrase = subtitle_phrase(tier, cluster);
                assert!(
                    !phrase.is_empty(),
                    "subtitle phrase for ({tier:?}, {cluster:?}) must not be empty"
                );
                assert!(
                    phrase.len() >= 10,
                    "subtitle phrase '{phrase}' for ({tier:?}, {cluster:?}) should be a real lore phrase"
                );
            }
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

    #[test]
    fn when_aspect_cluster_called_for_all_aspects_then_each_maps_to_correct_cluster() {
        // Arrange
        let cases: &[(Aspect, AspectCluster)] = &[
            (Aspect::Solid, AspectCluster::Physical),
            (Aspect::Fragile, AspectCluster::Physical),
            (Aspect::Force, AspectCluster::Physical),
            (Aspect::Calm, AspectCluster::Physical),
            (Aspect::Heat, AspectCluster::Elemental),
            (Aspect::Cold, AspectCluster::Elemental),
            (Aspect::Light, AspectCluster::Elemental),
            (Aspect::Dark, AspectCluster::Elemental),
            (Aspect::Growth, AspectCluster::Nature),
            (Aspect::Decay, AspectCluster::Nature),
            (Aspect::Order, AspectCluster::Nature),
            (Aspect::Chaos, AspectCluster::Nature),
            (Aspect::Change, AspectCluster::Arcane),
            (Aspect::Stasis, AspectCluster::Arcane),
            (Aspect::Expansion, AspectCluster::Arcane),
            (Aspect::Contraction, AspectCluster::Arcane),
        ];

        // Act & Assert
        for &(aspect, expected) in cases {
            let cluster = aspect_cluster(aspect);
            assert_eq!(
                cluster, expected,
                "{aspect:?} should map to {expected:?}, got {cluster:?}"
            );
        }
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
    }
}
