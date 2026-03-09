pub mod hierarchy;
pub mod prelude;
pub mod render_order;
pub mod spawn_child;
pub mod transform_propagation;
pub mod visibility;

#[cfg(test)]
pub(crate) mod test_helpers {
    use bevy_ecs::prelude::*;
    use bevy_ecs::schedule::IntoScheduleConfigs;
    use bevy_ecs::system::ScheduleSystem;

    pub(crate) fn run_system<M>(
        world: &mut World,
        system: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) {
        let mut schedule = Schedule::default();
        schedule.add_systems(system);
        schedule.run(world);
    }

    pub(crate) fn run_hierarchy_system(world: &mut World) {
        run_system(world, crate::hierarchy::hierarchy_maintenance_system);
    }

    pub(crate) fn run_transform_system(world: &mut World) {
        run_system(world, crate::transform_propagation::transform_propagation_system);
    }

    pub(crate) fn run_visibility_system(world: &mut World) {
        run_system(world, crate::visibility::visibility_system);
    }
}
