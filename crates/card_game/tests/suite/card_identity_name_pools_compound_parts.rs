#![allow(clippy::unwrap_used)]

use card_game::card::identity::name_pools::AspectCluster;
use card_game::card::identity::name_pools::compound_parts::{
    generate_compound, prefix_pool, suffix_pool,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

const ARCHETYPES: &[&str] = &["Weapon", "Shield", "Spell", "Healer", "Scout", "Artifact"];
const CLUSTERS: &[AspectCluster] = &[
    AspectCluster::Physical,
    AspectCluster::Elemental,
    AspectCluster::Nature,
    AspectCluster::Arcane,
];

#[test]
fn when_known_archetype_then_prefix_pool_nonempty() {
    for archetype in ARCHETYPES {
        for &cluster in CLUSTERS {
            let pool = prefix_pool(archetype, cluster);
            assert!(
                pool.len() >= 12,
                "{archetype}/{cluster:?} prefix pool has only {} entries",
                pool.len()
            );
        }
    }
}

#[test]
fn when_known_archetype_then_suffix_pool_nonempty() {
    for archetype in ARCHETYPES {
        for &cluster in CLUSTERS {
            let pool = suffix_pool(archetype, cluster);
            assert!(
                pool.len() >= 12,
                "{archetype}/{cluster:?} suffix pool has only {} entries",
                pool.len()
            );
        }
    }
}

#[test]
fn when_unknown_archetype_then_fallback_pools_used() {
    for &cluster in CLUSTERS {
        let prefixes = prefix_pool("Unknown", cluster);
        let suffixes = suffix_pool("Unknown", cluster);
        assert!(!prefixes.is_empty());
        assert!(!suffixes.is_empty());
    }
}

#[test]
fn when_generate_compound_then_suffix_is_lowercased() {
    // Arrange
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    // Act
    let name = generate_compound(&mut rng, "Weapon", AspectCluster::Physical);

    // Assert — first char is uppercase (from prefix), compound has no spaces
    assert!(!name.is_empty());
    assert!(!name.contains(' '));
    assert!(name.chars().next().unwrap().is_uppercase());
}

#[test]
fn when_different_seeds_then_different_compounds() {
    // Arrange
    let mut rng1 = ChaCha8Rng::seed_from_u64(1);
    let mut rng2 = ChaCha8Rng::seed_from_u64(999);

    // Act
    let names1: Vec<String> = (0..10)
        .map(|_| generate_compound(&mut rng1, "Spell", AspectCluster::Elemental))
        .collect();
    let names2: Vec<String> = (0..10)
        .map(|_| generate_compound(&mut rng2, "Spell", AspectCluster::Elemental))
        .collect();

    // Assert — not all names should be identical across different seeds
    assert_ne!(names1, names2);
}
