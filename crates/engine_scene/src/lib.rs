pub mod hierarchy;
pub mod prelude;
pub mod spawn_child;
pub mod transform_propagation;

#[cfg(test)]
pub(crate) mod test_helpers {
    use bevy_ecs::prelude::*;

    pub(crate) fn run_hierarchy_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(crate::hierarchy::hierarchy_maintenance_system);
        schedule.run(world);
    }

    pub(crate) fn run_transform_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(crate::transform_propagation::transform_propagation_system);
        schedule.run(world);
    }
}
