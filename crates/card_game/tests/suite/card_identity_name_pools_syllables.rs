#![allow(clippy::unwrap_used)]

use card_game::card::identity::name_pools::syllables::generate_proper_noun;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

#[test]
fn generate_proper_noun_deterministic_with_seed() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let name1 = generate_proper_noun(&mut rng);

    let mut rng2 = ChaCha8Rng::seed_from_u64(42);
    let name2 = generate_proper_noun(&mut rng2);

    assert_eq!(name1, name2);
}

#[test]
fn generate_proper_noun_starts_uppercase() {
    let mut rng = ChaCha8Rng::seed_from_u64(0);
    for _ in 0..100 {
        let name = generate_proper_noun(&mut rng);
        assert!(
            name.starts_with(|c: char| c.is_ascii_uppercase()),
            "Name should start uppercase: {name}"
        );
    }
}

#[test]
fn generate_proper_noun_reasonable_length() {
    let mut rng = ChaCha8Rng::seed_from_u64(99);
    for _ in 0..200 {
        let name = generate_proper_noun(&mut rng);
        let len = name.len();
        assert!(
            (3..=15).contains(&len),
            "Name length {len} out of range: {name}"
        );
    }
}

#[test]
fn generate_proper_noun_produces_both_lengths() {
    let mut rng = ChaCha8Rng::seed_from_u64(7);
    let names: Vec<String> = (0..100).map(|_| generate_proper_noun(&mut rng)).collect();

    let short_count = names
        .iter()
        .filter(|n| {
            // 2-syllable names tend to be shorter
            n.len() <= 7
        })
        .count();

    let long_count = names
        .iter()
        .filter(|n| {
            // 3-syllable names tend to be longer
            n.len() > 7
        })
        .count();

    assert!(
        short_count > 0,
        "Should produce some short (2-syllable) names"
    );
    assert!(
        long_count > 0,
        "Should produce some long (3-syllable) names"
    );
}

#[test]
fn generate_proper_noun_all_ascii() {
    let mut rng = ChaCha8Rng::seed_from_u64(123);
    for _ in 0..100 {
        let name = generate_proper_noun(&mut rng);
        assert!(name.is_ascii(), "Name should be ASCII: {name}");
    }
}
