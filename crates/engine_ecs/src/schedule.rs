// EVOLVE-BLOCK-START
use bevy_ecs::schedule::ScheduleLabel;

pub const PHASE_COUNT: usize = 18;

#[derive(ScheduleLabel, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Phase {
    Startup,
    OnEnable,
    FixedUpdate,
    AsyncFixedUpdate,
    OnCollision,
    Input,
    Update,
    Async,
    Animate,
    LateUpdate,
    OnBecameVisible,
    Render,
    PostRender,
    AsyncEndOfFrame,
    OnPause,
    OnDisable,
    OnDestroy,
    WaitForVBlank,
}

impl Phase {
    pub const ALL: [Self; PHASE_COUNT] = [
        Self::Startup,
        Self::OnEnable,
        Self::FixedUpdate,
        Self::AsyncFixedUpdate,
        Self::OnCollision,
        Self::Input,
        Self::Update,
        Self::Async,
        Self::Animate,
        Self::LateUpdate,
        Self::OnBecameVisible,
        Self::Render,
        Self::PostRender,
        Self::AsyncEndOfFrame,
        Self::OnPause,
        Self::OnDisable,
        Self::OnDestroy,
        Self::WaitForVBlank,
    ];

    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Startup => "Startup",
            Self::OnEnable => "OnEnable",
            Self::FixedUpdate => "FixedUpdate",
            Self::AsyncFixedUpdate => "AsyncFixedUpdate",
            Self::OnCollision => "OnCollision",
            Self::Input => "Input",
            Self::Update => "Update",
            Self::Async => "Async",
            Self::Animate => "Animate",
            Self::LateUpdate => "LateUpdate",
            Self::OnBecameVisible => "OnBecameVisible",
            Self::Render => "Render",
            Self::PostRender => "PostRender",
            Self::AsyncEndOfFrame => "AsyncEndOfFrame",
            Self::OnPause => "OnPause",
            Self::OnDisable => "OnDisable",
            Self::OnDestroy => "OnDestroy",
            Self::WaitForVBlank => "WaitForVBlank",
        }
    }
}
// EVOLVE-BLOCK-END
