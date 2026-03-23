mod adjectives;
mod compound_parts;
mod nouns;
mod syllables;
pub mod templates;

use crate::card::signature::Aspect;

pub use adjectives::adjective_pool;
pub use compound_parts::generate_compound;
pub use nouns::noun_pool;
pub use syllables::generate_proper_noun;
pub use templates::{TitleParts, common_title, legendary_title, rare_title};

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
