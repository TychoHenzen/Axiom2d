use bevy_ecs::schedule::ScheduleLabel;

pub const PHASE_COUNT: usize = 5;

#[derive(ScheduleLabel, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Phase {
    Input,
    PreUpdate,
    Update,
    PostUpdate,
    Render,
}

impl Phase {
    pub const ALL: [Self; PHASE_COUNT] = [
        Self::Input,
        Self::PreUpdate,
        Self::Update,
        Self::PostUpdate,
        Self::Render,
    ];

    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// @doc: Phase indices must match enum declaration order because the app
    /// loop iterates `Phase::ALL` by index to run schedules in the canonical
    /// Input→PreUpdate→Update→PostUpdate→Render sequence. If a variant were
    /// reordered without updating ALL, systems would execute in the wrong phase.
    #[test]
    fn when_index_then_matches_declaration_order() {
        for (expected, phase) in Phase::ALL.iter().enumerate() {
            assert_eq!(
                phase.index(),
                expected,
                "{phase:?} should have index {expected}"
            );
        }
    }
}
