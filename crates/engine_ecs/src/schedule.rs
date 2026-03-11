use bevy_ecs::schedule::ScheduleLabel;

#[derive(ScheduleLabel, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Phase {
    Input,
    PreUpdate,
    Update,
    PostUpdate,
    Render,
}
