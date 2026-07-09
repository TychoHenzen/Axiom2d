#![allow(clippy::unwrap_used)]

use card_game::card::identity::name_pools::templates::{
    TitleParts, common_title, legendary_title, rare_title, weighted_choose,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

const PARTS: TitleParts<'_> = TitleParts {
    adj: "Burning",
    noun: "Sword",
    compound: "Fireblade",
    name: "Ignis",
    adj2: "Infernal",
};

/// @doc: `weighted_choose` returns an index within bounds.
#[test]
fn when_weighted_choose_then_index_in_bounds() {
    // Arrange
    let weights = &[10u32, 20, 30, 40];

    // Act / Assert
    for seed in 0..50 {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let idx = weighted_choose(&mut rng, weights);
        assert!(
            idx < weights.len(),
            "seed {seed}: index {idx} out of bounds [0, {})",
            weights.len()
        );
    }
}

/// @doc: `weighted_choose` is deterministic for the same seed.
#[test]
fn when_same_seed_then_weighted_choose_matches() {
    // Arrange
    let weights = &[10u32, 20, 30, 40];
    let mut rng1 = ChaCha8Rng::seed_from_u64(42);
    let mut rng2 = ChaCha8Rng::seed_from_u64(42);

    // Act
    let results: Vec<usize> = (0..20)
        .map(|_| weighted_choose(&mut rng1, weights))
        .collect();
    let expected: Vec<usize> = (0..20)
        .map(|_| weighted_choose(&mut rng2, weights))
        .collect();

    // Assert
    assert_eq!(
        results, expected,
        "deterministic weighted_choose must produce identical sequences"
    );
}

/// @doc: `weighted_choose` always picks 0 when there is a single weight.
#[test]
fn when_single_weight_then_always_zero() {
    // Arrange
    let mut rng = ChaCha8Rng::seed_from_u64(7);

    // Act / Assert
    for _ in 0..100 {
        assert_eq!(
            weighted_choose(&mut rng, &[100]),
            0,
            "single-weight choose must always return index 0"
        );
    }
}

/// @doc: `weighted_choose` produces every index over many trials with uniform weights.
#[test]
fn when_uniform_weights_then_every_index_appears() {
    // Arrange
    let mut rng = ChaCha8Rng::seed_from_u64(12345);
    let weights = &[1u32, 1, 1, 1, 1];
    let mut seen = [false; 5];

    // Act
    for _ in 0..500 {
        seen[weighted_choose(&mut rng, weights)] = true;
    }

    // Assert
    for (i, &visited) in seen.iter().enumerate() {
        assert!(visited, "index {i} was never selected in 500 trials");
    }
}

/// @doc: `weighted_choose` favours heavier weights over lighter ones statistically.
#[test]
fn when_skewed_weights_then_heavier_selected_more_often() {
    // Arrange
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let weights = &[1u32, 0, 0, 99];
    let mut counts = [0u32; 4];

    // Act
    for _ in 0..1000 {
        counts[weighted_choose(&mut rng, weights)] += 1;
    }

    // Assert — index 3 (weight 99) dominates; index 1/2 (weight 0) never picked
    assert_eq!(counts[1], 0, "zero-weight slot 1 was selected");
    assert_eq!(counts[2], 0, "zero-weight slot 2 was selected");
    assert!(
        counts[3] > counts[0] * 10,
        "heavy index 3 ({}) should drastically outweigh light index 0 ({})",
        counts[3],
        counts[0]
    );
}

/// @doc: `common_title` is deterministic with a fixed seed.
#[test]
fn when_common_title_same_seed_then_matches() {
    // Arrange
    let mut rng1 = ChaCha8Rng::seed_from_u64(42);
    let mut rng2 = ChaCha8Rng::seed_from_u64(42);

    // Act
    let a = common_title(&mut rng1, &PARTS);
    let b = common_title(&mut rng2, &PARTS);

    // Assert
    assert_eq!(a, b, "common_title must be deterministic for the same seed");
}

/// @doc: `common_title` includes the adjective in the output.
#[test]
fn when_common_title_then_contains_adj() {
    // Arrange
    let mut rng = ChaCha8Rng::seed_from_u64(1);

    // Act / Assert
    for _ in 0..200 {
        let title = common_title(&mut rng, &PARTS);
        assert!(
            title.contains("Burning"),
            "common_title output '{title}' should contain the adjective 'Burning'"
        );
    }
}

/// @doc: `common_title` output is non-empty and contains no raw format braces.
#[test]
fn when_common_title_then_no_raw_format_braces() {
    // Arrange
    let mut rng = ChaCha8Rng::seed_from_u64(99);

    // Act / Assert
    for _ in 0..200 {
        let title = common_title(&mut rng, &PARTS);
        assert!(!title.is_empty(), "common_title produced empty string");
        assert!(
            !title.contains('{'),
            "common_title output '{title}' contains raw '{{'"
        );
        assert!(
            !title.contains('}'),
            "common_title output '{title}' contains raw '}}'"
        );
    }
}

/// @doc: `common_title` produces every template variant over enough iterations.
#[test]
fn when_common_title_many_iterations_then_all_templates_used() {
    // Arrange
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut observed = [false; 4];

    // Act
    for _ in 0..500 {
        // Re-seed per batch to avoid one long sequence
        let title = common_title(&mut rng, &PARTS);
        // Identify template by structural patterns
        if title.starts_with("The ") && !title.contains(" of ") {
            observed[2] = true; // "The {adj} {noun}"
        } else if title.contains(" of ") {
            observed[1] = true; // "{noun} of {adj}"
        } else if title.contains("Fireblade") {
            observed[3] = true; // "{adj} {compound}"
        } else {
            observed[0] = true; // "{adj} {noun}"
        }
    }

    // Assert
    for (i, &visited) in observed.iter().enumerate() {
        assert!(
            visited,
            "common_title template variant {i} was never produced in 500 calls"
        );
    }
}

/// @doc: `rare_title` is deterministic with a fixed seed.
#[test]
fn when_rare_title_same_seed_then_matches() {
    // Arrange
    let mut rng1 = ChaCha8Rng::seed_from_u64(42);
    let mut rng2 = ChaCha8Rng::seed_from_u64(42);

    // Act
    let a = rare_title(&mut rng1, &PARTS);
    let b = rare_title(&mut rng2, &PARTS);

    // Assert
    assert_eq!(a, b, "rare_title must be deterministic for the same seed");
}

/// @doc: `rare_title` contains at least one of the parts in its output.
#[test]
fn when_rare_title_then_contains_part() {
    // Arrange
    let mut rng = ChaCha8Rng::seed_from_u64(1);
    let part_names = ["Burning", "Sword", "Fireblade", "Ignis", "Infernal"];

    // Act / Assert
    for _ in 0..500 {
        let title = rare_title(&mut rng, &PARTS);
        let has_part = part_names.iter().any(|p| title.contains(p));
        assert!(
            has_part,
            "rare_title output '{title}' should contain at least one part name"
        );
    }
}

/// @doc: `rare_title` output is non-empty and contains no raw format braces.
#[test]
fn when_rare_title_then_no_raw_format_braces() {
    // Arrange
    let mut rng = ChaCha8Rng::seed_from_u64(99);

    // Act / Assert
    for _ in 0..500 {
        let title = rare_title(&mut rng, &PARTS);
        assert!(!title.is_empty(), "rare_title produced empty string");
        assert!(
            !title.contains('{'),
            "rare_title output '{title}' contains raw '{{'"
        );
        assert!(
            !title.contains('}'),
            "rare_title output '{title}' contains raw '}}'"
        );
    }
}

/// @doc: `rare_title` outputs all three word-connection patterns (apostrophe, of, and comma).
#[test]
fn when_rare_title_then_pattern_words_appear() {
    // Arrange
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut has_apostrophe = false;
    let mut has_of = false;
    let mut has_comma = false;

    // Act
    for _ in 0..2000 {
        let title = rare_title(&mut rng, &PARTS);
        if title.contains('\'') {
            has_apostrophe = true;
        }
        if title.contains(" of ") {
            has_of = true;
        }
        if title.contains(", ") {
            has_comma = true;
        }
    }

    // Assert
    assert!(
        has_apostrophe,
        "rare_title should produce apostrophe pattern like \"X's ...\""
    );
    assert!(has_of, "rare_title should produce 'of' pattern");
    assert!(
        has_comma,
        "rare_title should produce comma pattern like \"..., ... and ...\""
    );
}

/// @doc: `legendary_title` is deterministic with a fixed seed.
#[test]
fn when_legendary_title_same_seed_then_matches() {
    // Arrange
    let mut rng1 = ChaCha8Rng::seed_from_u64(42);
    let mut rng2 = ChaCha8Rng::seed_from_u64(42);

    // Act
    let a = legendary_title(&mut rng1, &PARTS);
    let b = legendary_title(&mut rng2, &PARTS);

    // Assert
    assert_eq!(
        a, b,
        "legendary_title must be deterministic for the same seed"
    );
}

/// @doc: `legendary_title` contains the name in its output.
#[test]
fn when_legendary_title_then_contains_name() {
    // Arrange
    let mut rng = ChaCha8Rng::seed_from_u64(1);

    // Act / Assert
    for _ in 0..500 {
        let title = legendary_title(&mut rng, &PARTS);
        assert!(
            title.contains("Ignis"),
            "legendary_title output '{title}' should contain the name 'Ignis'"
        );
    }
}

/// @doc: `legendary_title` output is non-empty and contains no raw format braces.
#[test]
fn when_legendary_title_then_no_raw_format_braces() {
    // Arrange
    let mut rng = ChaCha8Rng::seed_from_u64(99);

    // Act / Assert
    for _ in 0..150 {
        let title = legendary_title(&mut rng, &PARTS);
        assert!(!title.is_empty(), "legendary_title produced empty string");
        assert!(
            !title.contains('{'),
            "legendary_title output '{title}' contains raw '{{'"
        );
        assert!(
            !title.contains('}'),
            "legendary_title output '{title}' contains raw '}}'"
        );
    }
}

/// @doc: `legendary_title` uses both possessive (apostrophe) and prepositional (of, the) patterns.
#[test]
fn when_legendary_title_then_both_possessive_and_prepositional_patterns() {
    // Arrange
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut has_apostrophe = false;
    let mut has_of = false;
    let mut has_the = false;

    // Act
    for _ in 0..500 {
        let title = legendary_title(&mut rng, &PARTS);
        if title.contains('\'') {
            has_apostrophe = true;
        }
        if title.contains(" of ") {
            has_of = true;
        }
        if title.contains(", the ") {
            has_the = true;
        }
    }

    // Assert
    assert!(
        has_apostrophe,
        "legendary_title should produce possessive pattern like \"X's ...\""
    );
    assert!(has_of, "legendary_title should produce 'of' pattern");
    assert!(has_the, "legendary_title should produce ', the' pattern");
}

/// @doc: `TitleParts` values are accessible after construction.
#[test]
fn when_title_parts_constructed_then_fields_match() {
    // Arrange
    let parts = TitleParts {
        adj: "Burning",
        noun: "Sword",
        compound: "Fireblade",
        name: "Ignis",
        adj2: "Infernal",
    };

    // Assert
    assert_eq!(parts.adj, "Burning", "adj field should match");
    assert_eq!(parts.noun, "Sword", "noun field should match");
    assert_eq!(parts.compound, "Fireblade", "compound field should match");
    assert_eq!(parts.name, "Ignis", "name field should match");
    assert_eq!(parts.adj2, "Infernal", "adj2 field should match");
}
