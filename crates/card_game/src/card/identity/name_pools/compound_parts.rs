use super::AspectCluster;
use rand::seq::IndexedRandom;
use rand_chacha::ChaCha8Rng;

/// Returns thematic prefixes for compound word generation.
/// Prefixes are short, evocative word-beginnings themed by archetype and cluster.
#[allow(clippy::too_many_lines)]
pub fn prefix_pool(archetype: &str, cluster: AspectCluster) -> &'static [&'static str] {
    match (archetype, cluster) {
        // === Weapon ===
        ("Weapon", AspectCluster::Physical) => &[
            "Iron", "Steel", "War", "Stone", "Bone", "Grim", "Dread", "Slag", "Anvil", "Maul",
            "Bane", "Flint", "Wrath", "Gore", "Hard", "Crude", "Rust", "Bare", "Blunt", "Hack",
            "Gash", "Grit", "Hew", "Scar", "Keen", "Fell", "Raze", "Shard", "Chip", "Brute",
        ],
        ("Weapon", AspectCluster::Elemental) => &[
            "Flame", "Frost", "Sun", "Ash", "Ember", "Storm", "Thunder", "Cinder", "Dusk", "Dawn",
            "Blaze", "Hail", "Spark", "Scorch", "Coal", "Snow", "Sear", "Burn", "Char", "Arc",
            "Bolt", "Glow", "Heat", "Ice", "Sleet", "Melt", "Smelt", "Vent", "Rage", "Flash",
        ],
        ("Weapon", AspectCluster::Nature) => &[
            "Thorn", "Root", "Vine", "Briar", "Fang", "Claw", "Bark", "Moss", "Wild", "Wolf",
            "Venom", "Sting", "Rot", "Bloom", "Hide", "Leaf", "Sap", "Weed", "Reed", "Bog", "Mire",
            "Mud", "Burr", "Wasp", "Adder", "Peat", "Gall", "Hornet", "Asp", "Nettle",
        ],
        ("Weapon", AspectCluster::Arcane) => &[
            "Void", "Rift", "Phase", "Gloom", "Hex", "Rune", "Warp", "Null", "Shade", "Blight",
            "Doom", "Nether", "Dread", "Flux", "Dark", "Curse", "Fell", "Murk", "Soul", "Scar",
            "Bind", "Ghost", "Pall", "Wraith", "Mist", "Shroud", "Dusk", "Blot", "Bane", "Grim",
        ],

        // === Shield ===
        ("Shield", AspectCluster::Physical) => &[
            "Iron", "Stone", "Plate", "Grind", "Hammer", "Bolt", "Bulwark", "Basalt", "Forge",
            "Brace", "Granite", "Cobalt", "Rampart", "Slab", "Hard", "Thick", "Broad", "Flat",
            "Blunt", "Rough", "Dense", "Stout", "Tough", "Firm", "Bold", "Solid", "Dull", "Crude",
        ],
        ("Shield", AspectCluster::Elemental) => &[
            "Frost", "Flame", "Storm", "Sun", "Ice", "Ash", "Thunder", "Ember", "Gale", "Pyre",
            "Sleet", "Cinder", "Corona", "Flash", "Snow", "Rain", "Wind", "Hail", "Coal", "Heat",
            "Warm", "Glow", "Bolt", "Arc", "Fog", "Mist", "Blaze", "Damp", "Char", "Glare",
        ],
        ("Shield", AspectCluster::Nature) => &[
            "Oak", "Thorn", "Root", "Bark", "Shell", "Hide", "Scale", "Hedge", "Moss", "Vine",
            "Carapace", "Horn", "Chitin", "Reed", "Pine", "Elm", "Ash", "Fir", "Briar", "Burr",
            "Hull", "Husk", "Rind", "Broad", "Dense", "Hard", "Thick", "Leaf", "Felt", "Pelt",
        ],
        ("Shield", AspectCluster::Arcane) => &[
            "Void", "Gloom", "Phase", "Warp", "Null", "Shade", "Rune", "Sigil", "Rift", "Hex",
            "Veil", "Nether", "Shroud", "Glyph", "Mist", "Haze", "Blur", "Dim", "Dark", "Murk",
            "Pall", "Ghost", "Bind", "Seal", "Ward", "Lock", "Curse", "Soul", "Pale", "Dusk",
        ],

        // === Spell ===
        ("Spell", AspectCluster::Physical) => &[
            "Stone", "Iron", "Slag", "Grit", "Dust", "Quake", "Crush", "Gravel", "Flint",
            "Shatter", "Hammer", "Maul", "Anvil", "Boulder", "Smash", "Crack", "Break", "Split",
            "Rend", "Hard", "Dense", "Heavy", "Blunt", "Chip", "Chunk", "Block", "Mass", "Brace",
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
            "Arc",
            "Bolt",
            "Snow",
            "Rain",
            "Heat",
            "Coal",
            "Char",
            "Sear",
            "Burn",
            "Ice",
            "Wind",
            "Gale",
            "Ash",
            "Flash",
            "Sleet",
        ],
        ("Spell", AspectCluster::Nature) => &[
            "Thorn", "Spore", "Bloom", "Vine", "Root", "Swarm", "Petal", "Seed", "Marsh", "Fungal",
            "Blight", "Pollen", "Canopy", "Moss", "Sap", "Weed", "Briar", "Fern", "Reed", "Rot",
            "Mold", "Mud", "Bog", "Burr", "Wasp", "Slug", "Gall", "Leaf", "Bark", "Fen",
        ],
        ("Spell", AspectCluster::Arcane) => &[
            "Void", "Rift", "Phase", "Warp", "Flux", "Null", "Astral", "Nether", "Chrono",
            "Aether", "Prism", "Echo", "Rune", "Enigma", "Dark", "Soul", "Bind", "Hex", "Curse",
            "Glyph", "Sigil", "Fade", "Blur", "Shift", "Fold", "Veil", "Weft", "Lace", "Fell",
        ],

        // === Healer ===
        ("Healer", AspectCluster::Physical) => &[
            "Bone", "Flesh", "Sinew", "Marrow", "Blood", "Suture", "Splint", "Mend", "Pulse",
            "Vigor", "Salve", "Balm", "Poultice", "Graft", "Vein", "Core", "Knit", "Bind", "Sap",
            "Gut", "Skin", "Seal", "Wrap", "Tend", "Cure", "Brace", "Fuse", "Weld", "Close",
        ],
        ("Healer", AspectCluster::Elemental) => &[
            "Sun", "Dawn", "Ember", "Warm", "Light", "Radiant", "Bright", "Glow", "Flicker",
            "Gleam", "Shimmer", "Haze", "Aurora", "Dew", "Soft", "Mild", "Dim", "Hush", "Calm",
            "Lull", "Pale", "Gold", "Rose", "Blush", "Clear", "Pure", "Still", "Faint", "Beam",
        ],
        ("Healer", AspectCluster::Nature) => &[
            "Bloom", "Petal", "Leaf", "Sap", "Herb", "Root", "Moss", "Dew", "Spring", "Nectar",
            "Seed", "Sprout", "Verdant", "Willow", "Bud", "Stem", "Fern", "Mint", "Sage", "Balm",
            "Reed", "Birch", "Elm", "Oak", "Clover", "Sprig", "Frond", "Briar", "Thyme", "Rue",
        ],
        ("Healer", AspectCluster::Arcane) => &[
            "Soul", "Spirit", "Aether", "Phase", "Dream", "Astral", "Echo", "Wisp", "Reverie",
            "Mist", "Hallow", "Vesper", "Wraith", "Lumen", "Soft", "Pale", "Still", "Hush", "Calm",
            "Veil", "Fold", "Haze", "Faint", "Pure", "Ward", "Keen", "Dusk", "Wane",
        ],

        // === Scout ===
        ("Scout", AspectCluster::Physical) => &[
            "Stone", "Dust", "Grit", "Flint", "Sand", "Gravel", "Iron", "Steel", "Slate", "Cobble",
            "Crag", "Ridge", "Cliff", "Shard", "Dun", "Dry", "Bare", "Slim", "Lean", "Lithe",
            "Swift", "Keen", "Sharp", "Taut", "Trim", "Wiry", "Pale", "Plain", "Thin", "Spare",
        ],
        ("Scout", AspectCluster::Elemental) => &[
            "Wind", "Gale", "Storm", "Breeze", "Flash", "Spark", "Bolt", "Drift", "Gust", "Squall",
            "Zephyr", "Thunder", "Streak", "Swift", "Arc", "Blur", "Rush", "Dart", "Zip", "Snap",
            "Hiss", "Haze", "Fog", "Mist", "Rain", "Cloud", "Smoke", "Slip", "Whip", "Zing",
        ],
        ("Scout", AspectCluster::Nature) => &[
            "Fox", "Hawk", "Wolf", "Stag", "Hare", "Owl", "Lynx", "Crow", "Viper", "Wren", "Moth",
            "Fern", "Briar", "Thorn", "Rat", "Bat", "Cat", "Kite", "Mink", "Newt", "Pike", "Rook",
            "Shrew", "Toad", "Vole", "Wasp", "Adder", "Deer", "Stoat", "Finch",
        ],
        ("Scout", AspectCluster::Arcane) => &[
            "Shade", "Phase", "Gloom", "Veil", "Wisp", "Wraith", "Phantom", "Ghost", "Blur",
            "Shimmer", "Flicker", "Haze", "Void", "Rift", "Dark", "Dim", "Murk", "Fade", "Mist",
            "Pale", "Thin", "Soft", "Slip", "Hush", "Still", "Dusk", "Null", "Pall", "Wane",
        ],

        // === Artifact ===
        ("Artifact", AspectCluster::Physical) => &[
            "Iron", "Stone", "Ore", "Anvil", "Forge", "Basalt", "Obsidian", "Granite", "Cobalt",
            "Titanium", "Adamant", "Bronze", "Chrome", "Quartz", "Rock", "Flint", "Slag", "Steel",
            "Tin", "Lead", "Zinc", "Slab", "Chip", "Lode", "Vein", "Seam", "Crude", "Raw", "Dull",
        ],
        ("Artifact", AspectCluster::Elemental) => &[
            "Sun", "Star", "Moon", "Ember", "Flame", "Storm", "Dawn", "Dusk", "Eclipse",
            "Solstice", "Equinox", "Corona", "Zenith", "Nadir", "Ash", "Arc", "Bolt", "Glow",
            "Heat", "Ice", "Rain", "Snow", "Wind", "Mist", "Fog", "Cloud", "Haze", "Flash", "Char",
        ],
        ("Artifact", AspectCluster::Nature) => &[
            "Amber", "Jade", "Root", "Seed", "Fossil", "Bone", "Shell", "Pearl", "Coral", "Resin",
            "Tusk", "Horn", "Ivory", "Sap", "Jet", "Flint", "Reed", "Hide", "Scale", "Bark",
            "Peat", "Tar", "Pitch", "Gum", "Claw", "Tooth", "Quill", "Spine", "Knot", "Burl",
        ],
        ("Artifact", AspectCluster::Arcane) => &[
            "Void", "Aether", "Rune", "Sigil", "Glyph", "Prism", "Astral", "Nether", "Rift",
            "Chrono", "Enigma", "Oracle", "Cipher", "Flux", "Soul", "Hex", "Bind", "Ward", "Lock",
            "Seal", "Mark", "Brand", "Lore", "Tome", "Script", "Ink", "Grim", "Pale", "Fell",
        ],

        // === Wildcard fallback ===
        (_, AspectCluster::Physical) => &[
            "Iron", "Stone", "Grim", "Slag", "Bone", "Flint", "Dust", "Grit", "Bane", "Dread",
            "Wrath", "Steel", "Anvil", "Slab", "Hard", "Rough", "Crude", "Bare", "Plain", "Blunt",
            "Dense", "Dull", "Stout", "Firm", "Bold", "Lean", "Raw", "Keen", "Fell", "Worn",
        ],
        (_, AspectCluster::Elemental) => &[
            "Flame", "Frost", "Storm", "Ember", "Ash", "Sun", "Thunder", "Spark", "Blaze", "Hail",
            "Cinder", "Dawn", "Dusk", "Flash", "Arc", "Bolt", "Snow", "Rain", "Heat", "Ice",
            "Wind", "Gale", "Char", "Sear", "Burn", "Coal", "Mist", "Fog", "Sleet", "Smoke",
        ],
        (_, AspectCluster::Nature) => &[
            "Thorn", "Root", "Vine", "Bloom", "Fang", "Moss", "Bark", "Wild", "Briar", "Seed",
            "Leaf", "Claw", "Rot", "Spore", "Weed", "Reed", "Mud", "Bog", "Sap", "Bud", "Stem",
            "Hide", "Scale", "Burr", "Gall", "Mold", "Fern", "Fen", "Mire", "Peat",
        ],
        (_, AspectCluster::Arcane) => &[
            "Void", "Rift", "Phase", "Gloom", "Hex", "Rune", "Warp", "Null", "Shade", "Flux",
            "Nether", "Aether", "Sigil", "Doom", "Dark", "Murk", "Fade", "Blur", "Mist", "Haze",
            "Pall", "Pale", "Dusk", "Dim", "Soul", "Bind", "Curse", "Mark", "Fell", "Fold",
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
            "break", "crush", "maim", "crack", "chip", "hack", "hew", "gash", "gouge", "nick",
            "notch", "score", "slit", "snap", "tear", "cleft", "split", "lash", "chop", "shear",
        ],
        ("Weapon", AspectCluster::Elemental) => &[
            "brand", "strike", "edge", "fury", "fang", "flare", "blast", "scorch", "sear", "burn",
            "rend", "lance", "bolt", "gash", "arc", "char", "heat", "lash", "wave", "burst",
            "pulse", "coil", "curl", "lick", "flash", "spark", "blaze", "singe", "smelt", "roar",
        ],
        ("Weapon", AspectCluster::Nature) => &[
            "fang", "bite", "sting", "thorn", "claw", "rend", "lash", "barb", "gore", "strike",
            "slash", "snap", "gnaw", "rip", "rake", "tear", "gouge", "hook", "jab", "lunge",
            "peck", "prick", "scratch", "graze", "nick", "prod", "thrust", "slash", "pierce",
            "spit",
        ],
        ("Weapon", AspectCluster::Arcane) => &[
            "brand", "edge", "rend", "bane", "strike", "curse", "hex", "pierce", "cut", "sever",
            "reave", "cleave", "scar", "wilt", "blight", "fade", "rot", "wane", "drain", "leech",
            "void", "null", "break", "dim", "dull", "numb", "blot", "blunt", "sap", "mark",
        ],

        // === Shield ===
        ("Shield", AspectCluster::Physical) => &[
            "wall", "guard", "ward", "mantle", "bulwark", "shield", "gate", "bastion", "plate",
            "veil", "shell", "helm", "keep", "brace", "lock", "bolt", "bar", "block", "stop",
            "hold", "fix", "bind", "clamp", "clasp", "latch", "pin", "stay", "set", "balk", "stay",
        ],
        ("Shield", AspectCluster::Elemental) => &[
            "ward", "guard", "mantle", "veil", "shell", "shroud", "shield", "cloak", "aegis",
            "barrier", "screen", "halo", "dome", "cowl", "wrap", "coat", "film", "glaze", "sheen",
            "mist", "haze", "blur", "dim", "veil", "mask", "hood", "husk", "cap", "skin", "gloss",
        ],
        ("Shield", AspectCluster::Nature) => &[
            "wall", "guard", "bark", "shell", "hide", "mantle", "weave", "wrap", "coat", "shroud",
            "veil", "thatch", "husk", "rind", "scale", "skin", "hull", "peel", "crust", "felt",
            "fur", "pelt", "fleece", "mat", "pad", "layer", "sheath", "clad", "helm", "leaf",
        ],
        ("Shield", AspectCluster::Arcane) => &[
            "ward", "veil", "shroud", "mantle", "guard", "barrier", "aegis", "seal", "lock",
            "gate", "cloak", "screen", "weave", "shell", "bind", "hold", "fix", "set", "pin",
            "mark", "brand", "rune", "glyph", "sigil", "hex", "curse", "knot", "link", "lace",
        ],

        // === Spell ===
        ("Spell", AspectCluster::Physical) => &[
            "bolt", "pulse", "blast", "wave", "burst", "surge", "slam", "crash", "quake", "tremor",
            "shock", "split", "shatter", "crack", "smash", "bash", "bang", "clap", "thud", "thump",
            "knock", "jolt", "lurch", "jerk", "jar", "rap", "pound", "bump", "tap", "chip",
        ],
        ("Spell", AspectCluster::Elemental) => &[
            "bolt", "pulse", "blast", "wave", "burst", "surge", "flare", "storm", "strike", "call",
            "nova", "torrent", "cascade", "barrage", "arc", "flash", "lash", "rain", "hail",
            "sleet", "flood", "rush", "flow", "roll", "swell", "tide", "wash", "sweep", "spout",
            "roar",
        ],
        ("Spell", AspectCluster::Nature) => &[
            "burst", "bloom", "wave", "pulse", "swarm", "cloud", "rain", "spore", "blight",
            "surge", "shower", "flood", "tide", "gust", "drift", "puff", "waft", "blow", "breath",
            "rush", "stream", "flow", "seep", "spread", "creep", "crawl", "trail", "reek", "mist",
        ],
        ("Spell", AspectCluster::Arcane) => &[
            "bolt", "pulse", "blast", "wave", "burst", "surge", "rift", "tear", "warp", "call",
            "snap", "shift", "fracture", "bend", "fold", "twist", "loop", "curl", "coil", "arc",
            "skip", "flip", "spin", "lurch", "drift", "slip", "jump", "fade", "shred", "void",
        ],

        // === Healer ===
        ("Healer", AspectCluster::Physical) => &[
            "balm", "salve", "mend", "cure", "bloom", "draught", "tonic", "bind", "weave", "seal",
            "stitch", "knit", "patch", "graft", "wrap", "set", "fix", "fuse", "weld", "close",
            "fill", "plug", "stop", "pack", "pad", "join", "bond", "clot", "knit", "brace",
        ],
        ("Healer", AspectCluster::Elemental) => &[
            "balm", "salve", "mend", "glow", "bloom", "warmth", "light", "grace", "kiss", "touch",
            "embrace", "cradle", "gift", "boon", "beam", "ray", "shine", "gleam", "heat", "warm",
            "soft", "mild", "calm", "still", "hush", "lull", "ease", "soothe", "gild", "blush",
        ],
        ("Healer", AspectCluster::Nature) => &[
            "balm", "salve", "bloom", "cure", "draught", "tonic", "dew", "sap", "nectar",
            "poultice", "extract", "essence", "elixir", "infusion", "juice", "drip", "drop",
            "seep", "flow", "stem", "bind", "brew", "mix", "blend", "steep", "draw", "press",
            "weal", "soak",
        ],
        ("Healer", AspectCluster::Arcane) => &[
            "mend", "weave", "seal", "bind", "call", "song", "chant", "prayer", "whisper", "dream",
            "vision", "trance", "echo", "pulse", "hum", "sigh", "breath", "voice", "word", "spell",
            "rite", "sign", "touch", "sense", "feel", "wake", "stir", "know", "lull", "calm",
        ],

        // === Scout ===
        ("Scout", AspectCluster::Physical) => &[
            "stride", "path", "step", "trail", "track", "gait", "march", "dash", "sprint", "trace",
            "climb", "vault", "leap", "bound", "run", "walk", "trot", "jog", "skip", "hop", "jump",
            "dart", "slip", "slide", "creep", "crawl", "stalk", "slink", "lope", "bolt",
        ],
        ("Scout", AspectCluster::Elemental) => &[
            "stride", "dash", "sprint", "bolt", "flash", "streak", "rush", "burst", "gust",
            "surge", "drift", "slip", "glide", "snap", "zip", "zap", "whip", "whiz", "hiss",
            "buzz", "hum", "ping", "zing", "blur", "arc", "flick", "flare", "lash", "gale", "keen",
        ],
        ("Scout", AspectCluster::Nature) => &[
            "stride", "path", "trail", "track", "stalk", "prowl", "hunt", "roam", "creep", "skulk",
            "dart", "pounce", "weave", "slip", "run", "leap", "dash", "sprint", "dodge", "duck",
            "dip", "dive", "roll", "spin", "twist", "swerve", "cut", "dart", "bolt", "lope",
        ],
        ("Scout", AspectCluster::Arcane) => &[
            "step", "shift", "blink", "fade", "slip", "drift", "phase", "warp", "stride", "vanish",
            "flicker", "blur", "glide", "haunt", "fold", "skip", "hop", "lurch", "flash", "snap",
            "flick", "pop", "drop", "rise", "sink", "melt", "flow", "pass", "flit", "dart",
        ],

        // === Artifact ===
        ("Artifact", AspectCluster::Physical) => &[
            "core", "shard", "stone", "relic", "focus", "heart", "soul", "crown", "eye", "seed",
            "anchor", "pillar", "monolith", "obelisk", "orb", "gem", "disk", "ring", "link",
            "band", "plate", "slab", "block", "cube", "lump", "mass", "chunk", "hunk", "knot",
            "pin",
        ],
        ("Artifact", AspectCluster::Elemental) => &[
            "core", "shard", "stone", "focus", "heart", "crown", "eye", "flame", "spark", "star",
            "orb", "prism", "beacon", "lantern", "gem", "disk", "ring", "lens", "dome", "globe",
            "sphere", "crystal", "bead", "vial", "flask", "lamp", "torch", "glow", "coal", "cell",
        ],
        ("Artifact", AspectCluster::Nature) => &[
            "core", "seed", "heart", "stone", "relic", "root", "bloom", "fruit", "husk", "shell",
            "fossil", "bone", "tooth", "thorn", "spore", "pod", "knot", "burl", "cone", "nut",
            "pit", "pip", "nub", "stub", "cob", "hull", "rind", "peel", "burr", "gall",
        ],
        ("Artifact", AspectCluster::Arcane) => &[
            "core", "shard", "relic", "focus", "soul", "crown", "eye", "seed", "nexus", "matrix",
            "lattice", "conduit", "lens", "codex", "tome", "scroll", "mark", "seal", "bind",
            "knot", "node", "hub", "link", "chain", "lace", "weave", "mesh", "web", "glyph",
            "rune",
        ],

        // === Wildcard fallback ===
        (_, AspectCluster::Physical) => &[
            "fragment", "sliver", "shard", "mote", "wisp", "scrap", "chip", "fleck", "spark",
            "trace", "husk", "splint", "grain", "dust", "bit", "piece", "chunk", "lump", "slice",
            "strip", "shred", "flake", "scale", "speck", "dot", "nub", "stub", "knot", "block",
            "pin",
        ],
        (_, AspectCluster::Elemental) => &[
            "fragment", "sliver", "shard", "mote", "wisp", "scrap", "chip", "fleck", "spark",
            "trace", "ember", "cinder", "flash", "gleam", "glow", "glint", "glare", "flare",
            "flicker", "arc", "bolt", "beam", "ray", "shaft", "streak", "band", "vein", "pulse",
            "surge",
        ],
        (_, AspectCluster::Nature) => &[
            "fragment", "sliver", "shard", "mote", "wisp", "scrap", "chip", "fleck", "spark",
            "trace", "spore", "leaf", "twig", "burr", "sprig", "stem", "blade", "frond", "shoot",
            "bud", "knot", "node", "tip", "nub", "stub", "bit", "scrap", "pith", "rind", "hull",
        ],
        (_, AspectCluster::Arcane) => &[
            "fragment", "sliver", "shard", "mote", "wisp", "scrap", "chip", "fleck", "spark",
            "trace", "echo", "rift", "glyph", "rune", "sign", "mark", "seal", "bind", "hex",
            "curse", "brand", "word", "lore", "arc", "void", "null", "fold", "dim", "haze", "pall",
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
