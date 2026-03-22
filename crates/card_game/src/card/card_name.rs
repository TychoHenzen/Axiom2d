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

    match profile.rarity {
        Rarity::Common | Rarity::Uncommon => format!("{adj} {noun}"),
        Rarity::Rare | Rarity::Epic => {
            let compound_nouns = compound_noun_pool(dominant_cluster, archetype);
            let compound_noun = compound_nouns.choose(rng).copied().unwrap_or("Relic");
            let secondary_adj = profile.secondary_axis.map(|el| {
                let pool = adjective_pool(profile.aspects[el as usize]);
                pool.choose(rng).copied().unwrap_or("Ancient")
            });
            match secondary_adj {
                Some(sec) => format!("{adj} {compound_noun} of {sec}"),
                None => format!("{compound_noun} of {adj}"),
            }
        }
        Rarity::Legendary => {
            let legendary_pool =
                legendary_name_pool(dominant_cluster, profile.archetype.as_deref());
            let proper_noun = legendary_pool.choose(rng).copied().unwrap_or("Relic");
            let epithet_adj = adjectives
                .iter()
                .filter(|&&a| a != adj)
                .copied()
                .collect::<Vec<_>>();
            let epithet = epithet_adj.choose(rng).copied().unwrap_or(adj);
            format!("{proper_noun}, the {epithet}")
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

fn noun_pool(archetype: &str, cluster: AspectCluster) -> &'static [&'static str] {
    match (archetype, cluster) {
        ("Weapon", AspectCluster::Physical) => {
            &["Blade", "Maul", "Cleaver", "Hammer", "Edge", "Bludgeon"]
        }
        ("Weapon", AspectCluster::Elemental) => &[
            "Brand",
            "Frostfang",
            "Sunblade",
            "Ashbringer",
            "Ember",
            "Shard",
        ],
        ("Weapon", AspectCluster::Nature) => {
            &["Thorn", "Briar", "Rootcleaver", "Fang", "Stinger", "Husk"]
        }
        ("Weapon", AspectCluster::Arcane) => &[
            "Voidedge",
            "Riftblade",
            "Phasecutter",
            "Flux",
            "Shiftshard",
            "Warp",
        ],

        ("Shield", AspectCluster::Physical) => {
            &["Bulwark", "Plate", "Rampart", "Guard", "Wall", "Bastion"]
        }
        ("Shield", AspectCluster::Elemental) => &[
            "Aegis",
            "Flameward",
            "Frostguard",
            "Dawnshield",
            "Veil",
            "Mantle",
        ],
        ("Shield", AspectCluster::Nature) => &[
            "Barkwall",
            "Thornguard",
            "Rootbulwark",
            "Huskshield",
            "Shell",
            "Carapace",
        ],
        ("Shield", AspectCluster::Arcane) => &[
            "Voidwall",
            "Phaseguard",
            "Riftbarrier",
            "Warpshield",
            "Nexus",
            "Ward",
        ],

        ("Spell", AspectCluster::Physical) => {
            &["Shatter", "Crush", "Tremor", "Quake", "Impact", "Shockwave"]
        }
        ("Spell", AspectCluster::Elemental) => {
            &["Bolt", "Flare", "Frostbolt", "Pyre", "Sunburst", "Eclipse"]
        }
        ("Spell", AspectCluster::Nature) => &[
            "Bloom",
            "Blight",
            "Overgrowth",
            "Wilt",
            "Tangle",
            "Rootgrasp",
        ],
        ("Spell", AspectCluster::Arcane) => &[
            "Rift",
            "Voidbolt",
            "Warpweave",
            "Phaseshift",
            "Paradox",
            "Flux",
        ],

        ("Healer", AspectCluster::Physical) => {
            &["Salve", "Balm", "Poultice", "Compress", "Splint", "Bandage"]
        }
        ("Healer", AspectCluster::Elemental) => &[
            "Elixir",
            "Sunwater",
            "Frostbalm",
            "Embertonic",
            "Radiance",
            "Glow",
        ],
        ("Healer", AspectCluster::Nature) => {
            &["Bloom", "Remedy", "Verdance", "Sprout", "Renewal", "Sap"]
        }
        ("Healer", AspectCluster::Arcane) => &[
            "Mend",
            "Restoration",
            "Phaseweave",
            "Paradox",
            "Flux",
            "Reversal",
        ],

        ("Scout", AspectCluster::Physical) => {
            &["Stride", "Footing", "Bearing", "Passage", "Track", "March"]
        }
        ("Scout", AspectCluster::Elemental) => &[
            "Flicker",
            "Glint",
            "Frostpath",
            "Embertrace",
            "Signal",
            "Gleam",
        ],
        ("Scout", AspectCluster::Nature) => &["Trail", "Scent", "Trace", "Omen", "Burrow", "Path"],
        ("Scout", AspectCluster::Arcane) => &["Vantage", "Rift", "Warp", "Phase", "Blink", "Vista"],

        ("Artifact", AspectCluster::Physical) => {
            &["Relic", "Totem", "Idol", "Slab", "Monolith", "Anvil"]
        }
        ("Artifact", AspectCluster::Elemental) => {
            &["Orb", "Prism", "Lantern", "Censer", "Brazier", "Crystal"]
        }
        ("Artifact", AspectCluster::Nature) => {
            &["Seedstone", "Root", "Fossil", "Husk", "Acorn", "Heartwood"]
        }
        ("Artifact", AspectCluster::Arcane) => {
            &["Focus", "Catalyst", "Nexus", "Gate", "Lens", "Conduit"]
        }

        (_, AspectCluster::Physical) => {
            &["Fragment", "Shard", "Remnant", "Chunk", "Scrap", "Sliver"]
        }
        (_, AspectCluster::Elemental) => &["Spark", "Ember", "Wisp", "Mote", "Gleam", "Flicker"],
        (_, AspectCluster::Nature) => &["Spore", "Root", "Seed", "Husk", "Twig", "Petal"],
        (_, AspectCluster::Arcane) => &["Echo", "Vestige", "Trace", "Rift", "Ripple", "Anomaly"],
    }
}

fn compound_noun_pool(cluster: AspectCluster, archetype: &str) -> &'static [&'static str] {
    match (archetype, cluster) {
        ("Weapon", AspectCluster::Physical) => &[
            "Ironbrand",
            "Steelcleave",
            "Warhammer",
            "Stonebreaker",
            "Maulfist",
            "Grindedge",
        ],
        ("Weapon", AspectCluster::Elemental) => &[
            "Flamebrand",
            "Frostfang",
            "Sunblade",
            "Ashbringer",
            "Emberstrike",
            "Duskedge",
        ],
        ("Weapon", AspectCluster::Nature) => &[
            "Thornblade",
            "Briarfang",
            "Rootcleaver",
            "Vinecutter",
            "Petaledge",
            "Blightsteel",
        ],
        ("Weapon", AspectCluster::Arcane) => &[
            "Voidedge",
            "Riftblade",
            "Phasecutter",
            "Warpfang",
            "Fluxbrand",
            "Nullcleave",
        ],

        ("Shield", AspectCluster::Physical) => &[
            "Ironwall",
            "Steelguard",
            "Stonebastion",
            "Plateward",
            "Grindshield",
            "Hammerguard",
        ],
        ("Shield", AspectCluster::Elemental) => &[
            "Flameward",
            "Frostguard",
            "Dawnshield",
            "Ashmantle",
            "Embershield",
            "Duskveil",
        ],
        ("Shield", AspectCluster::Nature) => &[
            "Barkwall",
            "Thornguard",
            "Rootbulwark",
            "Vineward",
            "Petalshield",
            "Blightmantle",
        ],
        ("Shield", AspectCluster::Arcane) => &[
            "Voidwall",
            "Riftguard",
            "Phaseward",
            "Warpshield",
            "Fluxbarrier",
            "Nullmantle",
        ],

        ("Spell", AspectCluster::Physical) => &[
            "Ironpulse",
            "Steelbolt",
            "Stonecrush",
            "Grindwave",
            "Shatterblast",
            "Tremorsurge",
        ],
        ("Spell", AspectCluster::Elemental) => &[
            "Firebolt",
            "Frostpulse",
            "Sunburst",
            "Ashblast",
            "Emberflare",
            "Duskwave",
        ],
        ("Spell", AspectCluster::Nature) => &[
            "Bloomsurge",
            "Blightbolt",
            "Rootgrasp",
            "Vinewhip",
            "Petalstorm",
            "Thornpulse",
        ],
        ("Spell", AspectCluster::Arcane) => &[
            "Voidbolt",
            "Riftpulse",
            "Phaseblast",
            "Warpweave",
            "Fluxsurge",
            "Nullburst",
        ],

        ("Healer", AspectCluster::Physical) => &[
            "Ironbalm",
            "Steelsalve",
            "Stonemend",
            "Platebind",
            "Grindpatch",
            "Hammersplint",
        ],
        ("Healer", AspectCluster::Elemental) => &[
            "Firesalve",
            "Frostbalm",
            "Sunmend",
            "Embertonic",
            "Glowdraught",
            "Dawnwater",
        ],
        ("Healer", AspectCluster::Nature) => &[
            "Bloomsalve",
            "Rootmend",
            "Vinebalm",
            "Petalcure",
            "Verdantbloom",
            "Sproutelixir",
        ],
        ("Healer", AspectCluster::Arcane) => &[
            "Voidmend",
            "Riftcure",
            "Phaseweave",
            "Warpsalve",
            "Fluxdraught",
            "Nullbloom",
        ],

        ("Scout", AspectCluster::Physical) => &[
            "Ironstride",
            "Steeltrack",
            "Stonepath",
            "Grindmarch",
            "Platestep",
            "Hammergait",
        ],
        ("Scout", AspectCluster::Elemental) => &[
            "Flamestep",
            "Frostpath",
            "Sunstride",
            "Embertrace",
            "Glowtrail",
            "Dusktrack",
        ],
        ("Scout", AspectCluster::Nature) => &[
            "Bloompath",
            "Roottrail",
            "Vinestride",
            "Petaltrace",
            "Verdantpath",
            "Sproutstep",
        ],
        ("Scout", AspectCluster::Arcane) => &[
            "Voidstride",
            "Riftpath",
            "Phasestep",
            "Warptrace",
            "Fluxtrail",
            "Nullstep",
        ],

        ("Artifact", AspectCluster::Physical) => &[
            "Ironcore",
            "Steelrelic",
            "Stoneshard",
            "Monolith",
            "Platetotem",
            "Grindfocus",
        ],
        ("Artifact", AspectCluster::Elemental) => &[
            "Fireshard",
            "Frostspark",
            "Sunshard",
            "Emberfocus",
            "Glowstone",
            "Duskrelic",
        ],
        ("Artifact", AspectCluster::Nature) => &[
            "Bloomseed",
            "Rootshard",
            "Vinecore",
            "Petalfocus",
            "Verdantstone",
            "Sproutfocus",
        ],
        ("Artifact", AspectCluster::Arcane) => &[
            "Voidcore",
            "Riftshard",
            "Phasefocus",
            "Warpstone",
            "Fluxshard",
            "Nullrelic",
        ],

        (_, AspectCluster::Physical) => &[
            "Ironchunk",
            "Steelfragment",
            "Stoneshard",
            "Grindscrap",
            "Platesliver",
            "Hammerchip",
        ],
        (_, AspectCluster::Elemental) => &[
            "Firemote",
            "Frostspark",
            "Sunwisp",
            "Emberfleck",
            "Glowmote",
            "Duskflicker",
        ],
        (_, AspectCluster::Nature) => &[
            "Bloomspore",
            "Rootseed",
            "Vinehusk",
            "Petaltwig",
            "Verdantmote",
            "Sproutseed",
        ],
        (_, AspectCluster::Arcane) => &[
            "Voidecho",
            "Riftwisp",
            "Phasemote",
            "Warpripple",
            "Fluxmote",
            "Nulltrace",
        ],
    }
}

fn legendary_name_pool(cluster: AspectCluster, archetype: Option<&str>) -> &'static [&'static str] {
    match (archetype, cluster) {
        (Some("Weapon"), AspectCluster::Physical) => &[
            "Grimholt", "Valkyr", "Dominus", "Ferrus", "Thane", "Arcturus",
        ],
        (Some("Weapon"), AspectCluster::Elemental) => &[
            "Solaris",
            "Pyraxis",
            "Glacius",
            "Ashenveil",
            "Luminos",
            "Duskrender",
        ],
        (Some("Weapon"), AspectCluster::Nature) => &[
            "Thornvale",
            "Verdaxis",
            "Blightcrown",
            "Briarwraith",
            "Rootcaller",
            "Sylvanis",
        ],
        (Some("Weapon"), AspectCluster::Arcane) => &[
            "Nullaris",
            "Voidheart",
            "Riftkeeper",
            "Phaselord",
            "Fluxion",
            "Warpmind",
        ],

        (Some("Shield"), AspectCluster::Physical) => &[
            "Basaltus",
            "Aegion",
            "Stoneward",
            "Ironheart",
            "Plateborn",
            "Grindwall",
        ],
        (Some("Shield"), AspectCluster::Elemental) => &[
            "Dawnguard",
            "Frostheim",
            "Pyreshield",
            "Ashward",
            "Luminarch",
            "Duskhaven",
        ],
        (Some("Shield"), AspectCluster::Nature) => &[
            "Oakheart",
            "Thornwall",
            "Rootshield",
            "Petalguard",
            "Verdance",
            "Blightward",
        ],
        (Some("Shield"), AspectCluster::Arcane) => &[
            "Nullguard",
            "Voidshield",
            "Riftmantle",
            "Phaseveil",
            "Fluxward",
            "Warpwall",
        ],

        (Some("Spell"), AspectCluster::Physical) => &[
            "Ironwrath",
            "Stonecall",
            "Shatterlord",
            "Gravimance",
            "Tremorsoul",
            "Crushbinder",
        ],
        (Some("Spell"), AspectCluster::Elemental) => &[
            "Pyrocant",
            "Frostweaver",
            "Solarion",
            "Ashsinger",
            "Luminance",
            "Duskmancer",
        ],
        (Some("Spell"), AspectCluster::Nature) => &[
            "Bloomcaller",
            "Blightwarden",
            "Rootsinger",
            "Vinecaster",
            "Verdancall",
            "Thornmancer",
        ],
        (Some("Spell"), AspectCluster::Arcane) => &[
            "Nullcaster",
            "Voidcaller",
            "Riftmancer",
            "Phasebinder",
            "Fluxweaver",
            "Warpsinger",
        ],

        (Some("Healer"), AspectCluster::Physical) => &[
            "Ironmender",
            "Stonebinder",
            "Platecure",
            "Grindhealer",
            "Hammermend",
            "Steelsuture",
        ],
        (Some("Healer"), AspectCluster::Elemental) => &[
            "Dawncaller",
            "Frostmender",
            "Pyrehealer",
            "Ashbinder",
            "Luminare",
            "Duskmender",
        ],
        (Some("Healer"), AspectCluster::Nature) => &[
            "Bloomhealer",
            "Rootbinder",
            "Verdantcure",
            "Petalweaver",
            "Sproutcaller",
            "Saplord",
        ],
        (Some("Healer"), AspectCluster::Arcane) => &[
            "Nullhealer",
            "Voidmender",
            "Riftbinder",
            "Phasecure",
            "Fluxmender",
            "Warpcaller",
        ],

        (Some("Scout"), AspectCluster::Physical) => &[
            "Ironseeker",
            "Stonewatcher",
            "Platescout",
            "Grindwalker",
            "Hammerpath",
            "Steelstrider",
        ],
        (Some("Scout"), AspectCluster::Elemental) => &[
            "Dawnseeker",
            "Frostwatcher",
            "Pyrepath",
            "Ashstrider",
            "Glowfinder",
            "Duskwalker",
        ],
        (Some("Scout"), AspectCluster::Nature) => &[
            "Bloomseeker",
            "Rootwatcher",
            "Verdantscout",
            "Petalstrider",
            "Sproutfinder",
            "Trailbloom",
        ],
        (Some("Scout"), AspectCluster::Arcane) => &[
            "Nullseeker",
            "Voidwatcher",
            "Riftscout",
            "Phasewalker",
            "Fluxfinder",
            "Warpstrider",
        ],

        (Some("Artifact"), AspectCluster::Physical) => &[
            "Ironheart",
            "Stonecore",
            "Platecrown",
            "Grindsoul",
            "Hammerstone",
            "Steelshard",
        ],
        (Some("Artifact"), AspectCluster::Elemental) => &[
            "Dawnstone",
            "Frostcore",
            "Pyreheart",
            "Ashcrown",
            "Luminore",
            "Duskshard",
        ],
        (Some("Artifact"), AspectCluster::Nature) => &[
            "Bloomheart",
            "Rootcore",
            "Verdantcrown",
            "Petalstone",
            "Sproutsoul",
            "Seedheart",
        ],
        (Some("Artifact"), AspectCluster::Arcane) => &[
            "Nullheart",
            "Voidcore",
            "Riftstone",
            "Phasecrown",
            "Fluxsoul",
            "Warpshard",
        ],

        (_, AspectCluster::Physical) => &[
            "Grimhollow",
            "Stonewright",
            "Ironbound",
            "Steelwarden",
            "Platecrown",
            "Grindwraith",
        ],
        (_, AspectCluster::Elemental) => &[
            "Ashenveil",
            "Frostborn",
            "Solarian",
            "Luminari",
            "Pyrecrown",
            "Duskwarden",
        ],
        (_, AspectCluster::Nature) => &[
            "Thornhollow",
            "Bloomwright",
            "Verdantborn",
            "Rootwarden",
            "Blightcrown",
            "Petalwraith",
        ],
        (_, AspectCluster::Arcane) => &[
            "Voidborn",
            "Rifthollow",
            "Phasewright",
            "Nullwarden",
            "Fluxcrown",
            "Warpwraith",
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AspectCluster {
    Physical,
    Elemental,
    Nature,
    Arcane,
}

pub fn aspect_cluster(aspect: Aspect) -> AspectCluster {
    match aspect {
        Aspect::Solid | Aspect::Fragile | Aspect::Force | Aspect::Calm => AspectCluster::Physical,
        Aspect::Heat | Aspect::Cold | Aspect::Light | Aspect::Dark => AspectCluster::Elemental,
        Aspect::Growth | Aspect::Decay | Aspect::Order | Aspect::Chaos => AspectCluster::Nature,
        Aspect::Change | Aspect::Stasis | Aspect::Expansion | Aspect::Contraction => {
            AspectCluster::Arcane
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::card::base_type::BaseCardTypeRegistry;
    use crate::card::signature::CardSignature;
    use crate::card::signature_profile::SignatureProfile;

    fn make_registry() -> BaseCardTypeRegistry {
        let mut registry = BaseCardTypeRegistry::default();
        crate::card::base_type::populate_default_types(&mut registry);
        registry
    }

    fn dominant_cluster_for(profile: &SignatureProfile) -> AspectCluster {
        profile.dominant_axis.map_or(AspectCluster::Physical, |el| {
            aspect_cluster(profile.aspects[el as usize])
        })
    }

    fn weapon_fixture() -> (SignatureProfile, CardSignature) {
        let registry = make_registry();
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
        // Arrange — Solidum at 0.8 → Intense tier, Solid → Physical cluster, archetype Weapon
        let (profile, sig) = weapon_fixture();

        // Act
        let name = generate_card_name(&profile, &sig);

        // Assert — subtitle is a lore phrase from subtitle_phrase(Intense, Physical)
        let expected = subtitle_phrase(Tier::Intense, AspectCluster::Physical);
        assert_eq!(name.subtitle, expected);
    }

    #[test]
    fn when_profile_has_no_archetype_then_subtitle_is_lore_phrase() {
        // Arrange — Lumines at 0.5 → Active tier, Light → Elemental cluster, no archetype
        let sig = CardSignature::new([0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.0]);
        let profile = SignatureProfile::without_archetype(&sig);

        // Act
        let name = generate_card_name(&profile, &sig);

        // Assert — subtitle is same format regardless of archetype
        let expected = subtitle_phrase(Tier::Active, AspectCluster::Elemental);
        assert_eq!(name.subtitle, expected);
    }

    #[test]
    fn when_all_axes_zero_then_subtitle_uses_dormant_phrase() {
        // Arrange — all zeros → dominant=Solidum, tier=Dormant, Fragile → Physical cluster
        let sig = CardSignature::new([0.0; 8]);
        let profile = SignatureProfile::without_archetype(&sig);

        // Act
        let name = generate_card_name(&profile, &sig);

        // Assert
        let expected = subtitle_phrase(Tier::Dormant, AspectCluster::Physical);
        assert_eq!(name.subtitle, expected);
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
            let pool = noun_pool(archetype, AspectCluster::Physical);
            assert!(
                !pool.is_empty(),
                "noun pool for '{archetype}' must not be empty"
            );
        }
        // Also check the fallback (None/unknown archetype)
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
    fn when_weapon_with_solid_aspect_then_title_noun_from_physical_pool() {
        // Arrange — Solidum +0.8 → Solid aspect → Physical cluster, matches Weapon archetype
        let (profile, sig) = weapon_fixture();
        assert_eq!(profile.rarity, Rarity::Uncommon);
        let physical_nouns = noun_pool("Weapon", AspectCluster::Physical);

        // Act
        let name = generate_card_name(&profile, &sig);
        let noun = name
            .title
            .split_whitespace()
            .nth(1)
            .expect("title has noun");

        // Assert
        assert!(
            physical_nouns.contains(&noun),
            "noun '{noun}' should be in Weapon+Physical pool {physical_nouns:?}"
        );
    }

    #[test]
    fn when_weapon_with_heat_aspect_then_title_noun_from_elemental_pool() {
        // Arrange — Febris dominant but still within Weapon match radius
        let registry = make_registry();
        let sig = CardSignature::new([0.7, 0.9, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let profile = SignatureProfile::new(&sig, &registry);
        assert_eq!(profile.archetype.as_deref(), Some("Weapon"));
        let elemental_nouns = noun_pool("Weapon", AspectCluster::Elemental);

        // Act
        let name = generate_card_name(&profile, &sig);
        let noun = name
            .title
            .split_whitespace()
            .nth(1)
            .expect("title has noun");

        // Assert
        assert!(
            elemental_nouns.contains(&noun),
            "noun '{noun}' should be in Weapon+Elemental pool {elemental_nouns:?}"
        );
    }

    #[test]
    fn when_compound_noun_pool_queried_for_elemental_and_weapon_then_returns_compound_words() {
        // Arrange
        let cluster = AspectCluster::Elemental;
        let archetype = "Weapon";

        // Act
        let pool = compound_noun_pool(cluster, archetype);

        // Assert
        assert!(
            !pool.is_empty(),
            "compound pool for ({cluster:?}, '{archetype}') must not be empty"
        );
        let known_compounds = [
            "Flamebrand",
            "Frostfang",
            "Sunblade",
            "Emberstrike",
            "Coldedge",
        ];
        let has_compound = pool.iter().any(|w| known_compounds.contains(w));
        assert!(
            has_compound,
            "compound pool for ({cluster:?}, '{archetype}') should contain at least one known compound; got {pool:?}"
        );
    }

    #[test]
    fn when_compound_noun_pool_queried_for_all_cluster_and_archetype_combinations_then_non_empty() {
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
                let pool = compound_noun_pool(cluster, archetype);
                assert!(
                    !pool.is_empty(),
                    "compound pool for ('{archetype}', {cluster:?}) must not be empty"
                );
            }
        }
    }

    #[test]
    fn when_rarity_is_rare_then_title_noun_is_from_compound_pool() {
        // Arrange — four axes at 0.6 → Rare; Solidum dominant → Physical cluster
        let sig = CardSignature::new([0.6, 0.6, 0.6, 0.6, 0.0, 0.0, 0.0, 0.0]);
        let profile = SignatureProfile::without_archetype(&sig);
        assert_eq!(profile.rarity, Rarity::Rare);

        let compound_pool = compound_noun_pool(dominant_cluster_for(&profile), "Unknown");

        // Act
        let name = generate_card_name(&profile, &sig);

        // Assert — extract noun: 4-word = "[Adj] [Noun] of [SecAdj]", 3-word = "[Noun] of [Adj]"
        let words: Vec<&str> = name.title.split_whitespace().collect();
        let noun = match words.len() {
            4 => words[1],
            3 => words[0],
            n => panic!("Rare title '{}' has unexpected word count {n}", name.title),
        };
        assert!(
            compound_pool.contains(&noun),
            "Rare title '{}': noun '{noun}' should be from compound pool {compound_pool:?}",
            name.title
        );
    }

    #[test]
    fn when_rarity_is_common_then_title_noun_is_from_simple_pool() {
        // Arrange — all zeros → Common; dominant → Physical cluster
        let sig = CardSignature::new([0.0; 8]);
        let profile = SignatureProfile::without_archetype(&sig);
        assert_eq!(profile.rarity, Rarity::Common);

        let simple_pool = noun_pool("Unknown", dominant_cluster_for(&profile));

        // Act
        let name = generate_card_name(&profile, &sig);

        // Assert — Common title is "[Adj] [Noun]"
        let noun = name
            .title
            .split_whitespace()
            .nth(1)
            .expect("Common title should have 2 words");
        assert!(
            simple_pool.contains(&noun),
            "Common title '{}': noun '{noun}' should be from simple pool {simple_pool:?}",
            name.title
        );
    }

    #[test]
    fn when_legendary_name_pool_queried_for_all_cluster_and_archetype_combinations_then_non_empty()
    {
        // Arrange
        let archetypes: &[Option<&str>] = &[
            Some("Weapon"),
            Some("Shield"),
            Some("Spell"),
            Some("Healer"),
            Some("Scout"),
            Some("Artifact"),
            None,
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
                let pool = legendary_name_pool(cluster, *archetype);
                assert!(
                    !pool.is_empty(),
                    "legendary pool for ({archetype:?}, {cluster:?}) must not be empty"
                );
            }
        }
    }

    #[test]
    fn when_legendary_name_pool_queried_for_weapon_vs_spell_then_pools_differ() {
        // Arrange
        let cluster = AspectCluster::Physical;

        // Act
        let weapon_pool = legendary_name_pool(cluster, Some("Weapon"));
        let spell_pool = legendary_name_pool(cluster, Some("Spell"));

        // Assert
        assert_ne!(
            weapon_pool, spell_pool,
            "Weapon and Spell legendary pools must differ for the same cluster"
        );
    }

    #[test]
    fn when_rarity_is_legendary_then_proper_noun_is_from_legendary_pool() {
        // Arrange — all axes 1.0 → Legendary
        let sig = CardSignature::new([1.0; 8]);
        let profile = SignatureProfile::without_archetype(&sig);
        assert_eq!(profile.rarity, Rarity::Legendary);

        let legendary_pool = legendary_name_pool(dominant_cluster_for(&profile), None);

        // Act
        let name = generate_card_name(&profile, &sig);

        // Assert — Legendary title is "[ProperNoun], the [Epithet]"
        let proper_noun = name
            .title
            .split(',')
            .next()
            .expect("Legendary title should have comma");
        assert!(
            legendary_pool.contains(&proper_noun),
            "Legendary title '{}': proper noun '{proper_noun}' should be from legendary pool {legendary_pool:?}",
            name.title
        );
    }

    #[test]
    fn when_no_archetype_and_arcane_aspect_then_title_noun_from_arcane_fallback() {
        // Arrange — Varias +0.8 → Change → Arcane cluster, no archetype match
        let sig = CardSignature::new([0.0, 0.0, 0.0, 0.0, 0.8, 0.0, 0.0, 0.0]);
        let profile = SignatureProfile::without_archetype(&sig);
        assert!(profile.archetype.is_none());
        let arcane_nouns = noun_pool("Unknown", AspectCluster::Arcane);

        // Act
        let name = generate_card_name(&profile, &sig);
        let noun = name
            .title
            .split_whitespace()
            .nth(1)
            .expect("title has noun");

        // Assert
        assert!(
            arcane_nouns.contains(&noun),
            "noun '{noun}' should be in fallback+Arcane pool {arcane_nouns:?}"
        );
    }
}
