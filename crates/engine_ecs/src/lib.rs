pub mod prelude;
pub mod schedule;

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::prelude::Component;
    use crate::prelude::Resource;
    use crate::prelude::World;

    #[derive(Component)]
    struct Health(u32);

    #[derive(Resource)]
    struct Score(u32);

    #[test]
    fn when_component_derive_used_then_struct_can_be_spawned_into_world() {
        // Arrange
        let mut world = World::new();

        // Act
        let entity = world.spawn(Health(100)).id();

        // Assert
        let health = world
            .get::<Health>(entity)
            .expect("Health component missing");
        assert_eq!(health.0, 100);
    }

    #[test]
    fn when_resource_derive_used_then_resource_can_be_inserted_and_read_from_world() {
        // Arrange
        let mut world = World::new();

        // Act
        world.insert_resource(Score(42));

        // Assert
        assert_eq!(world.resource::<Score>().0, 42);
    }

    #[test]
    fn when_system_runs_in_schedule_then_it_can_mutate_a_component_via_query() {
        use crate::prelude::{Phase, Query, Schedule};

        #[derive(Component)]
        struct Counter(u32);

        fn increment(mut q: Query<&mut Counter>) {
            for mut c in &mut q {
                c.0 += 1;
            }
        }

        // Arrange
        let mut world = World::new();
        let entity = world.spawn(Counter(0)).id();
        let mut schedule = Schedule::new(Phase::Update);
        schedule.add_systems(increment);

        // Act
        schedule.run(&mut world);

        // Assert
        assert_eq!(world.get::<Counter>(entity).unwrap().0, 1);
    }

    #[test]
    fn when_system_runs_in_schedule_then_it_can_read_a_resource_via_res() {
        use crate::prelude::{Phase, Res, Schedule};

        fn check_score(score: Res<Score>) {
            assert_eq!(score.0, 99);
        }

        // Arrange
        let mut world = World::new();
        world.insert_resource(Score(99));
        let mut schedule = Schedule::new(Phase::Update);
        schedule.add_systems(check_score);

        // Act
        schedule.run(&mut world);
    }

    #[test]
    fn when_system_runs_in_schedule_then_it_can_mutate_a_resource_via_resmut() {
        use crate::prelude::{Phase, ResMut, Schedule};

        #[derive(Resource)]
        struct Ticks(u32);

        fn tick(mut ticks: ResMut<Ticks>) {
            ticks.0 += 1;
        }

        // Arrange
        let mut world = World::new();
        world.insert_resource(Ticks(0));
        let mut schedule = Schedule::new(Phase::Update);
        schedule.add_systems(tick);

        // Act
        schedule.run(&mut world);

        // Assert
        assert_eq!(world.resource::<Ticks>().0, 1);
    }
}
