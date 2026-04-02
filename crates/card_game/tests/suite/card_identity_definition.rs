#![allow(clippy::unwrap_used)]

use card_game::card::identity::definition::{
    CardAbilities, CardDefinition, CardStats, CardType, Keyword, art_descriptor_default,
    description_from_abilities, rarity_border_color,
};
use card_game::card::identity::signature::{CardSignature, Rarity};

#[test]
fn when_description_from_abilities_called_with_no_keywords_then_returns_freeform_text() {
    // Arrange
    let abilities = CardAbilities {
        keywords: vec![],
        text: "Draw two cards.".to_owned(),
    };

    // Act
    let desc = description_from_abilities(&abilities);

    // Assert
    assert_eq!(desc, "Draw two cards.");
}

#[test]
fn when_description_from_abilities_called_with_keywords_only_then_returns_keyword_names() {
    // Arrange
    let abilities = CardAbilities {
        keywords: vec![Keyword::Taunt, Keyword::Rush],
        text: String::new(),
    };

    // Act
    let desc = description_from_abilities(&abilities);

    // Assert
    assert_eq!(desc, "Taunt. Rush.");
}

#[test]
fn when_description_from_abilities_called_with_keywords_and_text_then_keywords_precede_freeform() {
    // Arrange
    let abilities = CardAbilities {
        keywords: vec![Keyword::Lifesteal],
        text: "Restore health equal to damage dealt.".to_owned(),
    };

    // Act
    let desc = description_from_abilities(&abilities);

    // Assert
    let keyword_pos = desc
        .find("Lifesteal")
        .expect("Lifesteal not found in output");
    let text_pos = desc
        .find("Restore health")
        .expect("freeform text not found in output");
    assert!(
        keyword_pos < text_pos,
        "expected 'Lifesteal' before 'Restore health', got: {desc:?}"
    );
    assert!(
        desc.contains('\n') || desc.contains(". ") || desc.contains(' '),
        "expected a separator between keyword and freeform text, got: {desc:?}"
    );
}

#[test]
fn when_description_from_abilities_called_with_single_keyword_then_keyword_name_appears() {
    // Arrange
    let abilities = CardAbilities {
        keywords: vec![Keyword::Taunt],
        text: String::new(),
    };

    // Act
    let desc = description_from_abilities(&abilities);

    // Assert
    assert_eq!(desc, "Taunt.");
}

#[test]
fn when_rarity_border_color_called_with_same_sig_then_different_rarities_produce_different_colors()
{
    // Arrange
    let sig = CardSignature::new([0.5; 8]);
    let rarities = [
        Rarity::Common,
        Rarity::Uncommon,
        Rarity::Rare,
        Rarity::Epic,
        Rarity::Legendary,
    ];

    // Act
    let colors: Vec<_> = rarities
        .iter()
        .map(|r| rarity_border_color(*r, &sig))
        .collect();

    // Assert
    for i in 0..colors.len() {
        for j in (i + 1)..colors.len() {
            assert_ne!(colors[i], colors[j], "rarities {i} and {j} share a color");
        }
    }
}

#[test]
fn when_rarity_border_color_called_with_different_sigs_then_same_rarity_varies() {
    // Arrange
    let sig_a = CardSignature::new([0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8]);
    let sig_b = CardSignature::new([0.8, 0.7, 0.6, 0.5, 0.4, 0.3, 0.2, 0.1]);

    // Act
    let color_a = rarity_border_color(Rarity::Rare, &sig_a);
    let color_b = rarity_border_color(Rarity::Rare, &sig_b);

    // Assert — different signatures should produce different shades
    assert_ne!(
        color_a, color_b,
        "same rarity with different signatures should produce different colors"
    );
}

#[test]
fn when_rarity_border_color_called_then_same_inputs_produce_same_output() {
    // Arrange
    let sig = CardSignature::new([0.3, 0.6, 0.1, 0.9, 0.2, 0.5, 0.4, 0.7]);

    // Act
    let color1 = rarity_border_color(Rarity::Epic, &sig);
    let color2 = rarity_border_color(Rarity::Epic, &sig);

    // Assert
    assert_eq!(
        color1, color2,
        "same inputs must produce deterministic output"
    );
}

#[test]
fn when_art_descriptor_default_called_for_creature_then_gradient_is_not_flat() {
    // Act
    let art = art_descriptor_default(CardType::Creature);

    // Assert
    assert_ne!(art.background.top, art.background.bottom);
}

#[test]
fn when_art_descriptor_default_called_for_spell_then_gradient_differs_from_creature() {
    // Act
    let creature = art_descriptor_default(CardType::Creature);
    let spell = art_descriptor_default(CardType::Spell);

    // Assert
    assert!(
        creature.background.top != spell.background.top
            || creature.background.bottom != spell.background.bottom,
        "spell and creature should have distinct default gradients"
    );
}

#[test]
fn when_art_descriptor_default_called_for_artifact_then_gradient_differs_from_creature() {
    // Act
    let creature = art_descriptor_default(CardType::Creature);
    let artifact = art_descriptor_default(CardType::Artifact);

    // Assert
    assert!(
        creature.background.top != artifact.background.top
            || creature.background.bottom != artifact.background.bottom,
        "artifact and creature should have distinct default gradients"
    );
}

#[test]
fn when_card_definition_is_creature_with_none_stats_then_stats_is_none() {
    // Arrange
    let def = CardDefinition {
        card_type: CardType::Creature,
        name: "Token".to_owned(),
        stats: None,
        abilities: CardAbilities {
            keywords: vec![],
            text: String::new(),
        },
        art: art_descriptor_default(CardType::Creature),
    };

    // Assert
    assert!(def.stats.is_none());
}

#[test]
fn when_card_definition_is_spell_with_some_stats_then_stats_is_some() {
    // Arrange
    let def = CardDefinition {
        card_type: CardType::Spell,
        name: "Fireball".to_owned(),
        stats: Some(CardStats {
            cost: 4,
            attack: 0,
            health: 0,
        }),
        abilities: CardAbilities {
            keywords: vec![],
            text: "Deal 6 damage.".to_owned(),
        },
        art: art_descriptor_default(CardType::Spell),
    };

    // Assert
    assert!(def.stats.is_some());
}
