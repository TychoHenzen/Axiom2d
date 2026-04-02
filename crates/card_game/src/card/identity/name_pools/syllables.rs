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
