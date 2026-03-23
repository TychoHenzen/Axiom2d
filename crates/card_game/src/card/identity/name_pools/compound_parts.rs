use super::AspectCluster;
use rand::seq::SliceRandom;
use rand_chacha::ChaCha8Rng;

/// Returns thematic prefixes for compound word generation.
/// Prefixes are short, evocative word-beginnings themed by archetype and cluster.
#[allow(clippy::too_many_lines)]
pub fn prefix_pool(archetype: &str, cluster: AspectCluster) -> &'static [&'static str] {
    match (archetype, cluster) {
        // === Weapon ===
        ("Weapon", AspectCluster::Physical) => &[
            "Iron", "Steel", "War", "Stone", "Bone", "Grim", "Dread", "Slag", "Anvil", "Maul",
            "Bane", "Flint", "Wrath", "Gore",
        ],
        ("Weapon", AspectCluster::Elemental) => &[
            "Flame", "Frost", "Sun", "Ash", "Ember", "Storm", "Thunder", "Cinder", "Dusk", "Dawn",
            "Blaze", "Hail", "Spark", "Scorch",
        ],
        ("Weapon", AspectCluster::Nature) => &[
            "Thorn", "Root", "Vine", "Briar", "Fang", "Claw", "Bark", "Moss", "Wild", "Wolf",
            "Venom", "Sting", "Rot", "Bloom",
        ],
        ("Weapon", AspectCluster::Arcane) => &[
            "Void", "Rift", "Phase", "Gloom", "Hex", "Rune", "Warp", "Null", "Shade", "Blight",
            "Doom", "Nether", "Dread", "Flux",
        ],

        // === Shield ===
        ("Shield", AspectCluster::Physical) => &[
            "Iron", "Stone", "Plate", "Grind", "Hammer", "Bolt", "Bulwark", "Basalt", "Forge",
            "Brace", "Granite", "Cobalt", "Rampart", "Slab",
        ],
        ("Shield", AspectCluster::Elemental) => &[
            "Frost", "Flame", "Storm", "Sun", "Ice", "Ash", "Thunder", "Ember", "Gale", "Pyre",
            "Sleet", "Cinder", "Corona", "Flash",
        ],
        ("Shield", AspectCluster::Nature) => &[
            "Oak", "Thorn", "Root", "Bark", "Shell", "Hide", "Scale", "Hedge", "Moss", "Vine",
            "Carapace", "Horn", "Chitin", "Reed",
        ],
        ("Shield", AspectCluster::Arcane) => &[
            "Void", "Gloom", "Phase", "Warp", "Null", "Shade", "Rune", "Sigil", "Rift", "Hex",
            "Veil", "Nether", "Shroud", "Glyph",
        ],

        // === Spell ===
        ("Spell", AspectCluster::Physical) => &[
            "Stone", "Iron", "Slag", "Grit", "Dust", "Quake", "Crush", "Gravel", "Flint",
            "Shatter", "Hammer", "Maul", "Anvil", "Boulder",
        ],
        ("Spell", AspectCluster::Elemental) => &[
            "Flame",
            "Frost",
            "Storm",
            "Sun",
            "Ember",
            "Thunder",
            "Lightning",
            "Blaze",
            "Hail",
            "Scorch",
            "Spark",
            "Cinder",
            "Inferno",
            "Tempest",
        ],
        ("Spell", AspectCluster::Nature) => &[
            "Thorn", "Spore", "Bloom", "Vine", "Root", "Swarm", "Petal", "Seed", "Marsh", "Fungal",
            "Blight", "Pollen", "Canopy", "Moss",
        ],
        ("Spell", AspectCluster::Arcane) => &[
            "Void", "Rift", "Phase", "Warp", "Flux", "Null", "Astral", "Nether", "Chrono",
            "Aether", "Prism", "Echo", "Rune", "Enigma",
        ],

        // === Healer ===
        ("Healer", AspectCluster::Physical) => &[
            "Bone", "Flesh", "Sinew", "Marrow", "Blood", "Suture", "Splint", "Mend", "Pulse",
            "Vigor", "Salve", "Balm", "Poultice", "Graft",
        ],
        ("Healer", AspectCluster::Elemental) => &[
            "Sun", "Dawn", "Ember", "Warm", "Light", "Radiant", "Bright", "Glow", "Flicker",
            "Gleam", "Shimmer", "Haze", "Aurora", "Dew",
        ],
        ("Healer", AspectCluster::Nature) => &[
            "Bloom", "Petal", "Leaf", "Sap", "Herb", "Root", "Moss", "Dew", "Spring", "Nectar",
            "Seed", "Sprout", "Verdant", "Willow",
        ],
        ("Healer", AspectCluster::Arcane) => &[
            "Soul", "Spirit", "Aether", "Phase", "Dream", "Astral", "Echo", "Wisp", "Reverie",
            "Mist", "Hallow", "Vesper", "Wraith", "Lumen",
        ],

        // === Scout ===
        ("Scout", AspectCluster::Physical) => &[
            "Stone", "Dust", "Grit", "Flint", "Sand", "Gravel", "Iron", "Steel", "Slate", "Cobble",
            "Crag", "Ridge", "Cliff", "Shard",
        ],
        ("Scout", AspectCluster::Elemental) => &[
            "Wind", "Gale", "Storm", "Breeze", "Flash", "Spark", "Bolt", "Drift", "Gust", "Squall",
            "Zephyr", "Thunder", "Streak", "Swift",
        ],
        ("Scout", AspectCluster::Nature) => &[
            "Fox", "Hawk", "Wolf", "Stag", "Hare", "Owl", "Lynx", "Crow", "Viper", "Wren", "Moth",
            "Fern", "Briar", "Thorn",
        ],
        ("Scout", AspectCluster::Arcane) => &[
            "Shade", "Phase", "Gloom", "Veil", "Wisp", "Wraith", "Phantom", "Ghost", "Blur",
            "Shimmer", "Flicker", "Haze", "Void", "Rift",
        ],

        // === Artifact ===
        ("Artifact", AspectCluster::Physical) => &[
            "Iron", "Stone", "Ore", "Anvil", "Forge", "Basalt", "Obsidian", "Granite", "Cobalt",
            "Titanium", "Adamant", "Bronze", "Chrome", "Quartz",
        ],
        ("Artifact", AspectCluster::Elemental) => &[
            "Sun", "Star", "Moon", "Ember", "Flame", "Storm", "Dawn", "Dusk", "Eclipse",
            "Solstice", "Equinox", "Corona", "Zenith", "Nadir",
        ],
        ("Artifact", AspectCluster::Nature) => &[
            "Amber", "Jade", "Root", "Seed", "Fossil", "Bone", "Shell", "Pearl", "Coral", "Resin",
            "Tusk", "Horn", "Ivory", "Sap",
        ],
        ("Artifact", AspectCluster::Arcane) => &[
            "Void", "Aether", "Rune", "Sigil", "Glyph", "Prism", "Astral", "Nether", "Rift",
            "Chrono", "Enigma", "Oracle", "Cipher", "Flux",
        ],

        // === Wildcard fallback ===
        (_, AspectCluster::Physical) => &[
            "Iron", "Stone", "Grim", "Slag", "Bone", "Flint", "Dust", "Grit", "Bane", "Dread",
            "Wrath", "Steel", "Anvil", "Slab",
        ],
        (_, AspectCluster::Elemental) => &[
            "Flame", "Frost", "Storm", "Ember", "Ash", "Sun", "Thunder", "Spark", "Blaze", "Hail",
            "Cinder", "Dawn", "Dusk", "Flash",
        ],
        (_, AspectCluster::Nature) => &[
            "Thorn", "Root", "Vine", "Bloom", "Fang", "Moss", "Bark", "Wild", "Briar", "Seed",
            "Leaf", "Claw", "Rot", "Spore",
        ],
        (_, AspectCluster::Arcane) => &[
            "Void", "Rift", "Phase", "Gloom", "Hex", "Rune", "Warp", "Null", "Shade", "Flux",
            "Nether", "Aether", "Sigil", "Doom",
        ],
    }
}

/// Returns thematic suffixes for compound word generation.
/// Suffixes are word-endings that form the second half of a compound noun, themed by archetype.
#[allow(clippy::too_many_lines)]
pub fn suffix_pool(archetype: &str, cluster: AspectCluster) -> &'static [&'static str] {
    match (archetype, cluster) {
        // === Weapon ===
        ("Weapon", AspectCluster::Physical) => &[
            "brand", "cleave", "fang", "edge", "bite", "strike", "rend", "slash", "cut", "pierce",
            "break", "crush", "maim", "crack",
        ],
        ("Weapon", AspectCluster::Elemental) => &[
            "brand", "strike", "edge", "fury", "fang", "flare", "blast", "scorch", "sear", "burn",
            "rend", "lance", "bolt", "gash",
        ],
        ("Weapon", AspectCluster::Nature) => &[
            "fang", "bite", "sting", "thorn", "claw", "rend", "lash", "barb", "gore", "strike",
            "slash", "snap", "gnaw", "rip",
        ],
        ("Weapon", AspectCluster::Arcane) => &[
            "brand", "edge", "rend", "bane", "strike", "curse", "hex", "pierce", "cut", "sever",
            "reave", "cleave", "scar", "wilt",
        ],

        // === Shield ===
        ("Shield", AspectCluster::Physical) => &[
            "wall", "guard", "ward", "mantle", "bulwark", "shield", "gate", "bastion", "plate",
            "veil", "shell", "helm", "keep", "brace",
        ],
        ("Shield", AspectCluster::Elemental) => &[
            "ward", "guard", "mantle", "veil", "shell", "shroud", "shield", "cloak", "aegis",
            "barrier", "screen", "halo", "dome", "cowl",
        ],
        ("Shield", AspectCluster::Nature) => &[
            "wall", "guard", "bark", "shell", "hide", "mantle", "weave", "wrap", "coat", "shroud",
            "veil", "thatch", "husk", "rind",
        ],
        ("Shield", AspectCluster::Arcane) => &[
            "ward", "veil", "shroud", "mantle", "guard", "barrier", "aegis", "seal", "lock",
            "gate", "cloak", "screen", "weave", "shell",
        ],

        // === Spell ===
        ("Spell", AspectCluster::Physical) => &[
            "bolt", "pulse", "blast", "wave", "burst", "surge", "slam", "crash", "quake", "tremor",
            "shock", "split", "shatter", "crack",
        ],
        ("Spell", AspectCluster::Elemental) => &[
            "bolt", "pulse", "blast", "wave", "burst", "surge", "flare", "storm", "strike", "call",
            "nova", "torrent", "cascade", "barrage",
        ],
        ("Spell", AspectCluster::Nature) => &[
            "burst", "bloom", "wave", "pulse", "swarm", "cloud", "rain", "spore", "blight",
            "surge", "shower", "flood", "tide", "gust",
        ],
        ("Spell", AspectCluster::Arcane) => &[
            "bolt", "pulse", "blast", "wave", "burst", "surge", "rift", "tear", "warp", "call",
            "snap", "shift", "fracture", "bend",
        ],

        // === Healer ===
        ("Healer", AspectCluster::Physical) => &[
            "balm", "salve", "mend", "cure", "bloom", "draught", "tonic", "bind", "weave", "seal",
            "stitch", "knit", "patch", "graft",
        ],
        ("Healer", AspectCluster::Elemental) => &[
            "balm", "salve", "mend", "glow", "bloom", "warmth", "light", "grace", "kiss", "touch",
            "embrace", "cradle", "gift", "boon",
        ],
        ("Healer", AspectCluster::Nature) => &[
            "balm", "salve", "bloom", "cure", "draught", "tonic", "dew", "sap", "nectar",
            "poultice", "extract", "essence", "elixir", "infusion",
        ],
        ("Healer", AspectCluster::Arcane) => &[
            "mend", "weave", "seal", "bind", "call", "song", "chant", "prayer", "whisper", "dream",
            "vision", "trance", "echo", "pulse",
        ],

        // === Scout ===
        ("Scout", AspectCluster::Physical) => &[
            "stride", "path", "step", "trail", "track", "gait", "march", "dash", "sprint", "trace",
            "climb", "vault", "leap", "bound",
        ],
        ("Scout", AspectCluster::Elemental) => &[
            "stride", "dash", "sprint", "bolt", "flash", "streak", "rush", "burst", "gust",
            "surge", "drift", "slip", "glide", "snap",
        ],
        ("Scout", AspectCluster::Nature) => &[
            "stride", "path", "trail", "track", "stalk", "prowl", "hunt", "roam", "creep", "skulk",
            "dart", "pounce", "weave", "slip",
        ],
        ("Scout", AspectCluster::Arcane) => &[
            "step", "shift", "blink", "fade", "slip", "drift", "phase", "warp", "stride", "vanish",
            "flicker", "blur", "glide", "haunt",
        ],

        // === Artifact ===
        ("Artifact", AspectCluster::Physical) => &[
            "core", "shard", "stone", "relic", "focus", "heart", "soul", "crown", "eye", "seed",
            "anchor", "pillar", "monolith", "obelisk",
        ],
        ("Artifact", AspectCluster::Elemental) => &[
            "core", "shard", "stone", "focus", "heart", "crown", "eye", "flame", "spark", "star",
            "orb", "prism", "beacon", "lantern",
        ],
        ("Artifact", AspectCluster::Nature) => &[
            "core", "seed", "heart", "stone", "relic", "root", "bloom", "fruit", "husk", "shell",
            "fossil", "bone", "tooth", "thorn",
        ],
        ("Artifact", AspectCluster::Arcane) => &[
            "core", "shard", "relic", "focus", "soul", "crown", "eye", "seed", "nexus", "matrix",
            "lattice", "conduit", "lens", "codex",
        ],

        // === Wildcard fallback ===
        (_, AspectCluster::Physical) => &[
            "fragment", "sliver", "shard", "mote", "wisp", "scrap", "chip", "fleck", "spark",
            "trace", "husk", "splint", "grain", "dust",
        ],
        (_, AspectCluster::Elemental) => &[
            "fragment", "sliver", "shard", "mote", "wisp", "scrap", "chip", "fleck", "spark",
            "trace", "ember", "cinder", "flash", "gleam",
        ],
        (_, AspectCluster::Nature) => &[
            "fragment", "sliver", "shard", "mote", "wisp", "scrap", "chip", "fleck", "spark",
            "trace", "spore", "leaf", "twig", "burr",
        ],
        (_, AspectCluster::Arcane) => &[
            "fragment", "sliver", "shard", "mote", "wisp", "scrap", "chip", "fleck", "spark",
            "trace", "echo", "rift", "glyph", "rune",
        ],
    }
}

/// Generates a compound word by combining a random prefix with a random suffix.
/// The suffix is lowercased so "Iron" + "Brand" becomes "Ironbrand".
pub fn generate_compound(rng: &mut ChaCha8Rng, archetype: &str, cluster: AspectCluster) -> String {
    let prefixes = prefix_pool(archetype, cluster);
    let suffixes = suffix_pool(archetype, cluster);

    let prefix = prefixes.choose(rng).unwrap_or(&"Null");
    let suffix = suffixes.choose(rng).unwrap_or(&"shard");

    let suffix_lower = suffix.to_lowercase();
    format!("{prefix}{suffix_lower}")
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rand::SeedableRng;

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
}
