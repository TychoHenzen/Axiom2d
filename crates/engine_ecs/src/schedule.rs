use bevy_ecs::schedule::ScheduleLabel;

#[derive(ScheduleLabel, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Phase {
    Input,
    PreUpdate,
    Update,
    PostUpdate,
    Render,
}

#[cfg(test)]
mod tests {
    use super::Phase;

    #[test]
    fn when_schedule_phase_variants_used_then_all_five_exist_and_are_distinct() {
        use std::collections::HashSet;

        // Arrange
        let phases = [
            Phase::Input,
            Phase::PreUpdate,
            Phase::Update,
            Phase::PostUpdate,
            Phase::Render,
        ];

        // Assert
        assert_eq!(phases.iter().collect::<HashSet<_>>().len(), 5);
    }

    #[test]
    fn when_phase_variants_formatted_as_debug_then_names_are_human_readable() {
        assert_eq!(format!("{:?}", Phase::Input), "Input");
        assert_eq!(format!("{:?}", Phase::PreUpdate), "PreUpdate");
        assert_eq!(format!("{:?}", Phase::Update), "Update");
        assert_eq!(format!("{:?}", Phase::PostUpdate), "PostUpdate");
        assert_eq!(format!("{:?}", Phase::Render), "Render");
    }

    #[test]
    fn when_phase_used_as_bevy_schedule_label_then_schedule_accepts_it_without_panic() {
        use bevy_ecs::prelude::{Schedule, World};

        fn noop() {}

        // Arrange
        let mut world = World::new();
        let mut schedule = Schedule::new(Phase::Update);
        schedule.add_systems(noop);

        // Act
        schedule.run(&mut world);
    }
}
