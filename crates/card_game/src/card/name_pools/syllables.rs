use rand::Rng as _;
use rand::seq::SliceRandom;
use rand_chacha::ChaCha8Rng;

static FIRST_SYLLABLES: &[&str] = &[
    "Bry", "Ju", "Ben", "Mal", "Vor", "Kal", "Tho", "Grim", "Ash", "Dun", "Fen", "Har", "Kel",
    "Mor", "Nis", "Oth", "Rath", "Sel", "Tur", "Ul", "Val", "Wyn", "Xan", "Zar", "Cor", "Dra",
    "Eld", "Fal", "Gil", "Hev", "Ith", "Jes", "Kyr", "Lor", "Mav", "Nor", "Pax", "Ren", "Siv",
    "Tel", "Bran", "Cyn", "Dol", "Gar", "Hul", "Krath", "Leth", "Morn", "Sar", "Thal",
];

static MIDDLE_SYLLABLES: &[&str] = &[
    "da", "the", "lo", "ri", "na", "ke", "mu", "ve", "la", "so", "en", "ar", "il", "os", "un",
    "ew", "al", "is", "or", "an",
];

static FINAL_SYLLABLES: &[&str] = &[
    "dam", "thew", "frey", "cuth", "then", "dris", "mund", "wick", "ley", "ton", "rick", "bert",
    "holm", "gar", "wen", "tus", "vex", "lix", "nox", "ram", "dor", "wyn", "kas", "mir", "von",
    "zek", "gon", "rath", "lund", "bane",
];

/// Generates a fantasy proper noun from syllable parts.
///
/// Produces 2-syllable names (60%) like "Brydam" or "Juthew",
/// or 3-syllable names (40%) like "Kaldathew" or "Benfrey".
pub fn generate_proper_noun(rng: &mut ChaCha8Rng) -> String {
    let first = FIRST_SYLLABLES.choose(rng).expect("non-empty pool");
    let final_syl = FINAL_SYLLABLES.choose(rng).expect("non-empty pool");

    let use_middle = rng.gen_ratio(2, 5); // 40% chance of 3 syllables

    if use_middle {
        let mid = MIDDLE_SYLLABLES.choose(rng).expect("non-empty pool");
        format!("{first}{mid}{final_syl}")
    } else {
        format!("{first}{final_syl}")
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rand::SeedableRng;

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
}
