use super::residual::{ModifierType, ResidualStats};

const EFFECT_SCALE: f32 = 20.0;
const MIN_DISPLAY_VALUE: i32 = 1;

fn stat_entries(stats: &ResidualStats) -> Vec<(ModifierType, f32)> {
    vec![
        (ModifierType::Power, stats.power),
        (ModifierType::Healing, stats.healing),
        (ModifierType::Defense, stats.defense),
        (ModifierType::Speed, stats.speed),
        (ModifierType::Cost, stats.cost),
        (ModifierType::Duration, stats.duration),
        (ModifierType::Range, stats.range),
        (ModifierType::Special, stats.special),
    ]
}

fn format_effect(modifier_type: ModifierType, raw_value: f32) -> Option<String> {
    let scaled = (raw_value * EFFECT_SCALE).round() as i32;
    if scaled.abs() < MIN_DISPLAY_VALUE {
        return None;
    }
    let value = scaled.unsigned_abs();
    match modifier_type {
        ModifierType::Power => Some(format!("Deal {value} damage")),
        ModifierType::Healing => Some(format!("Restore {value} health")),
        ModifierType::Defense => Some(format!("Block {value} damage")),
        ModifierType::Speed => Some(format!("+{value} initiative")),
        ModifierType::Cost => Some(format!("Cost {value}")),
        ModifierType::Duration => Some(format!("{value} turns")),
        ModifierType::Range => Some(format!("Range {value}")),
        ModifierType::Special => Some(format!("Special {value}")),
    }
}

pub fn generate_card_description(stats: &ResidualStats) -> String {
    let mut entries = stat_entries(stats);
    entries.sort_by(|a, b| b.1.abs().partial_cmp(&a.1.abs()).expect("NaN in stats"));
    entries
        .into_iter()
        .take(3)
        .filter_map(|(mt, val)| format_effect(mt, val))
        .collect::<Vec<_>>()
        .join("\n")
}
