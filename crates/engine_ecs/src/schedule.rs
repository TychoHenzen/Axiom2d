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
