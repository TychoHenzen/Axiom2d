#![allow(clippy::unwrap_used)]

use card_game::card::identity::name_pools::{AspectCluster, noun_pool};

const ARCHETYPES: &[&str] = &["Weapon", "Shield", "Spell", "Healer", "Scout", "Artifact"];
const CLUSTERS: &[AspectCluster] = &[
    AspectCluster::Physical,
    AspectCluster::Elemental,
    AspectCluster::Nature,
    AspectCluster::Arcane,
];

/// @doc: Every (archetype, cluster) pair returns a non-empty noun pool.
#[test]
fn all_archetype_cluster_pools_non_empty() {
    for &archetype in ARCHETYPES {
        for &cluster in CLUSTERS {
            let pool = noun_pool(archetype, cluster);
            assert!(
                !pool.is_empty(),
                "Pool for archetype '{archetype}', cluster {cluster:?} should be non-empty"
            );
        }
    }
}

/// @doc: Every (archetype, cluster) pool contains at least 12 entries.
#[test]
fn all_pools_have_minimum_entries() {
    for &archetype in ARCHETYPES {
        for &cluster in CLUSTERS {
            let pool = noun_pool(archetype, cluster);
            assert!(
                pool.len() >= 12,
                "Pool for archetype '{archetype}', cluster {cluster:?} should have >=12 entries, got {}",
                pool.len()
            );
        }
    }
}

/// @doc: Every entry across all pools is a non-empty string.
#[test]
fn all_entries_non_empty() {
    for &archetype in ARCHETYPES {
        for &cluster in CLUSTERS {
            for &entry in noun_pool(archetype, cluster) {
                assert!(
                    !entry.is_empty(),
                    "Empty entry in archetype '{archetype}', cluster {cluster:?}"
                );
            }
        }
    }
}

/// @doc: Every entry across all pools is at least 2 characters long.
#[test]
fn all_entries_minimum_length() {
    for &archetype in ARCHETYPES {
        for &cluster in CLUSTERS {
            for &entry in noun_pool(archetype, cluster) {
                assert!(
                    entry.len() >= 2,
                    "Entry too short ({len}) in archetype '{archetype}', cluster {cluster:?}: '{entry}'",
                    len = entry.len()
                );
            }
        }
    }
}

/// @doc: Every entry across all pools starts with an uppercase ASCII letter.
#[test]
fn all_entries_start_uppercase() {
    for &archetype in ARCHETYPES {
        for &cluster in CLUSTERS {
            for &entry in noun_pool(archetype, cluster) {
                assert!(
                    entry.starts_with(|c: char| c.is_ascii_uppercase()),
                    "Entry does not start uppercase in archetype '{archetype}', cluster {cluster:?}: '{entry}'"
                );
            }
        }
    }
}

/// @doc: Every entry across all pools is pure ASCII.
#[test]
fn all_entries_ascii() {
    for &archetype in ARCHETYPES {
        for &cluster in CLUSTERS {
            for &entry in noun_pool(archetype, cluster) {
                assert!(
                    entry.is_ascii(),
                    "Entry not ASCII in archetype '{archetype}', cluster {cluster:?}: '{entry}'"
                );
            }
        }
    }
}

/// @doc: Calling noun_pool with the same arguments returns the same static slice pointer.
#[test]
fn same_archetype_cluster_deterministic_pointer() {
    for &archetype in ARCHETYPES {
        for &cluster in CLUSTERS {
            let pool1 = noun_pool(archetype, cluster) as *const _;
            let pool2 = noun_pool(archetype, cluster) as *const _;
            assert_eq!(
                pool1, pool2,
                "Two calls for archetype '{archetype}', cluster {cluster:?} returned different pointers"
            );
        }
    }
}

/// @doc: Different archetype-cluster pairs return different pools (different pointer).
#[test]
fn different_pools_have_different_pointers() {
    let mut seen = std::collections::HashSet::new();
    for &archetype in ARCHETYPES {
        for &cluster in CLUSTERS {
            let ptr = std::ptr::from_ref(noun_pool(archetype, cluster)) as *const () as usize;
            assert!(
                seen.insert(ptr),
                "Duplicate pool pointer for archetype '{archetype}', cluster {cluster:?}"
            );
        }
    }
}

/// @doc: Unknown archetypes fall back to a non-empty general pool per cluster.
#[test]
fn unknown_archetype_returns_fallback() {
    for &cluster in CLUSTERS {
        let pool = noun_pool("NonExistentArchetype", cluster);
        assert!(
            !pool.is_empty(),
            "Fallback pool for cluster {cluster:?} should be non-empty"
        );
        for &entry in pool {
            assert!(!entry.is_empty(), "Empty entry in fallback for cluster {cluster:?}");
        }
    }
}

/// @doc: Unknown archetype fallback pools are distinct across clusters.
#[test]
fn fallback_pools_differ_by_cluster() {
    let pools: Vec<_> = CLUSTERS
        .iter()
        .map(|&c| noun_pool("Unknown", c) as *const _)
        .collect();
    let mut unique = pools.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(
        unique.len(),
        pools.len(),
        "Fallback pools should differ across all clusters"
    );
}
