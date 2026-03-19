use bevy_ecs::prelude::{Commands, Query, Res, ResMut};
use engine_input::prelude::MouseState;
use engine_physics::prelude::{Collider, PhysicsRes, RigidBody};
use engine_scene::prelude::{GlobalTransform2D, RenderLayer};

use crate::card_damping::{BASE_ANGULAR_DRAG, BASE_LINEAR_DRAG};
use crate::card_item_form::CardItemForm;
use crate::card_pick::{DRAGGED_COLLISION_FILTER, DRAGGED_COLLISION_GROUP};
use crate::drag_state::DragState;
use crate::spawn_table_card::CARD_WIDTH;
use crate::stash_grid::StashGrid;
use crate::stash_grid::find_stash_slot_at;
use crate::stash_render::SLOT_WIDTH;
use crate::stash_toggle::StashVisible;
use engine_core::scale_spring::ScaleSpring;

pub fn stash_boundary_system(
    mouse: Res<MouseState>,
    mut drag_state: ResMut<DragState>,
    mut physics: ResMut<PhysicsRes>,
    stash_visible: Res<StashVisible>,
    grid: Res<StashGrid>,
    mut commands: Commands,
    transform_query: Query<(&GlobalTransform2D, &Collider)>,
) {
    let Some(info) = drag_state.dragging else {
        return;
    };

    let over_stash = stash_visible.0
        && find_stash_slot_at(mouse.screen_pos(), grid.width(), grid.height()).is_some();

    if info.stash_cursor_follow && !over_stash {
        // Exit stash: add physics body, switch to spring drag
        if let Ok((transform, collider)) = transform_query.get(info.entity) {
            let position = transform.0.translation;
            physics.add_body(info.entity, &RigidBody::Dynamic, position);
            physics.add_collider(info.entity, collider);
            physics.set_damping(info.entity, BASE_LINEAR_DRAG, BASE_ANGULAR_DRAG);
            physics.set_collision_group(
                info.entity,
                DRAGGED_COLLISION_GROUP,
                DRAGGED_COLLISION_FILTER,
            );
        }
        commands.entity(info.entity).insert(RigidBody::Dynamic);
        commands.entity(info.entity).insert(RenderLayer::World);
        commands.entity(info.entity).remove::<CardItemForm>();
        commands.entity(info.entity).insert(ScaleSpring::new(1.0));
        drag_state.dragging = Some(crate::drag_state::DragInfo {
            stash_cursor_follow: false,
            ..info
        });
    } else if !info.stash_cursor_follow && over_stash {
        // Enter stash: remove physics body, switch to cursor-follow
        physics.remove_body(info.entity);
        commands.entity(info.entity).remove::<RigidBody>();
        commands.entity(info.entity).insert(RenderLayer::UI);
        commands.entity(info.entity).insert(CardItemForm);
        commands
            .entity(info.entity)
            .insert(ScaleSpring::new(SLOT_WIDTH / CARD_WIDTH));
        drag_state.dragging = Some(crate::drag_state::DragInfo {
            stash_cursor_follow: true,
            ..info
        });
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::{Arc, Mutex};

    use bevy_ecs::prelude::*;
    use engine_input::prelude::MouseState;
    use engine_physics::prelude::{
        Collider, CollisionEvent, PhysicsBackend, PhysicsRes, RigidBody,
    };
    use engine_scene::prelude::GlobalTransform2D;
    use glam::{Affine2, Vec2};

    use super::stash_boundary_system;
    use crate::card_zone::CardZone;
    use crate::drag_state::{DragInfo, DragState};
    use crate::spawn_table_card::CARD_WIDTH;
    use crate::stash_grid::StashGrid;
    use crate::stash_render::SLOT_WIDTH;
    use crate::stash_toggle::StashVisible;
    use engine_core::scale_spring::ScaleSpring;

    type AddBodyLog = Arc<Mutex<Vec<(Entity, Vec2)>>>;
    type RemoveBodyLog = Arc<Mutex<Vec<Entity>>>;

    struct SpyPhysicsBackend {
        add_body_log: AddBodyLog,
        remove_body_log: RemoveBodyLog,
    }

    impl SpyPhysicsBackend {
        fn new(add_body_log: AddBodyLog, remove_body_log: RemoveBodyLog) -> Self {
            Self {
                add_body_log,
                remove_body_log,
            }
        }
    }

    impl PhysicsBackend for SpyPhysicsBackend {
        fn step(&mut self, _: engine_core::prelude::Seconds) {}
        fn add_body(&mut self, entity: Entity, _: &RigidBody, position: Vec2) -> bool {
            self.add_body_log.lock().unwrap().push((entity, position));
            true
        }
        fn add_collider(&mut self, _: Entity, _: &Collider) -> bool {
            true
        }
        fn remove_body(&mut self, entity: Entity) {
            self.remove_body_log.lock().unwrap().push(entity);
        }
        fn body_position(&self, _: Entity) -> Option<Vec2> {
            None
        }
        fn body_rotation(&self, _: Entity) -> Option<f32> {
            None
        }
        fn drain_collision_events(&mut self) -> Vec<CollisionEvent> {
            Vec::new()
        }
        fn body_linear_velocity(&self, _: Entity) -> Option<Vec2> {
            None
        }
        fn set_linear_velocity(&mut self, _: Entity, _: Vec2) {}
        fn set_angular_velocity(&mut self, _: Entity, _: f32) {}
        fn add_force_at_point(&mut self, _: Entity, _: Vec2, _: Vec2) {}
        fn body_angular_velocity(&self, _: Entity) -> Option<f32> {
            None
        }
        fn set_damping(&mut self, _: Entity, _: f32, _: f32) {}
        fn set_collision_group(&mut self, _: Entity, _: u32, _: u32) {}
    }

    fn run_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(stash_boundary_system);
        schedule.run(world);
    }

    fn make_spy_physics() -> (
        Box<dyn PhysicsBackend + Send + Sync>,
        AddBodyLog,
        RemoveBodyLog,
    ) {
        let add_log: AddBodyLog = Arc::new(Mutex::new(Vec::new()));
        let remove_log: RemoveBodyLog = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyPhysicsBackend::new(add_log.clone(), remove_log.clone());
        (Box::new(spy), add_log, remove_log)
    }

    fn make_drag_info(entity: Entity, stash_cursor_follow: bool) -> DragState {
        DragState {
            dragging: Some(DragInfo {
                entity,
                local_grab_offset: Vec2::ZERO,
                origin_zone: CardZone::Stash {
                    page: 0,
                    col: 0,
                    row: 0,
                },
                stash_cursor_follow,
            }),
        }
    }

    fn mouse_at_screen(pos: Vec2) -> MouseState {
        let mut mouse = MouseState::default();
        mouse.set_screen_pos(pos);
        mouse
    }

    // -----------------------------------------------------------------------
    // No drag → no-op
    // -----------------------------------------------------------------------

    #[test]
    fn when_no_drag_then_no_physics_calls() {
        // Arrange
        let mut world = World::new();
        let (spy, add_log, remove_log) = make_spy_physics();
        world.insert_resource(PhysicsRes::new(spy));
        world.insert_resource(DragState::default());
        world.insert_resource(mouse_at_screen(Vec2::new(45.0, 57.5)));
        world.insert_resource(StashVisible(true));
        world.insert_resource(StashGrid::new(10, 10, 1));

        // Act
        run_system(&mut world);

        // Assert
        assert!(add_log.lock().unwrap().is_empty());
        assert!(remove_log.lock().unwrap().is_empty());
    }

    // -----------------------------------------------------------------------
    // Cursor in stash + follow=true → no-op (steady state)
    // -----------------------------------------------------------------------

    #[test]
    fn when_stash_follow_and_cursor_in_stash_then_no_physics_calls() {
        // Arrange
        let mut world = World::new();
        let entity = world
            .spawn((
                GlobalTransform2D(Affine2::IDENTITY),
                Collider::Aabb(Vec2::new(30.0, 45.0)),
            ))
            .id();
        let (spy, add_log, remove_log) = make_spy_physics();
        world.insert_resource(PhysicsRes::new(spy));
        world.insert_resource(make_drag_info(entity, true));
        world.insert_resource(mouse_at_screen(Vec2::new(45.0, 57.5))); // inside slot (0,0)
        world.insert_resource(StashVisible(true));
        world.insert_resource(StashGrid::new(10, 10, 1));

        // Act
        run_system(&mut world);

        // Assert
        assert!(add_log.lock().unwrap().is_empty());
        assert!(remove_log.lock().unwrap().is_empty());
        assert!(
            world
                .resource::<DragState>()
                .dragging
                .unwrap()
                .stash_cursor_follow,
            "stash_cursor_follow should remain true"
        );
    }

    // -----------------------------------------------------------------------
    // Exit stash: follow=true + cursor outside → add physics, follow=false
    // -----------------------------------------------------------------------

    #[test]
    fn when_stash_follow_and_cursor_exits_stash_then_physics_body_added() {
        // Arrange
        let mut world = World::new();
        let entity = world
            .spawn((
                GlobalTransform2D(Affine2::from_translation(Vec2::new(100.0, 200.0))),
                Collider::Aabb(Vec2::new(30.0, 45.0)),
            ))
            .id();
        let (spy, add_log, _) = make_spy_physics();
        world.insert_resource(PhysicsRes::new(spy));
        world.insert_resource(make_drag_info(entity, true));
        world.insert_resource(mouse_at_screen(Vec2::new(800.0, 400.0))); // outside stash
        world.insert_resource(StashVisible(true));
        world.insert_resource(StashGrid::new(10, 10, 1));

        // Act
        run_system(&mut world);

        // Assert
        let calls = add_log.lock().unwrap();
        assert_eq!(calls.len(), 1, "add_body should be called once");
        assert_eq!(calls[0].0, entity);
    }

    #[test]
    fn when_stash_follow_and_cursor_exits_stash_then_follow_set_false() {
        // Arrange
        let mut world = World::new();
        let entity = world
            .spawn((
                GlobalTransform2D(Affine2::IDENTITY),
                Collider::Aabb(Vec2::new(30.0, 45.0)),
            ))
            .id();
        let (spy, _, _) = make_spy_physics();
        world.insert_resource(PhysicsRes::new(spy));
        world.insert_resource(make_drag_info(entity, true));
        world.insert_resource(mouse_at_screen(Vec2::new(800.0, 400.0)));
        world.insert_resource(StashVisible(true));
        world.insert_resource(StashGrid::new(10, 10, 1));

        // Act
        run_system(&mut world);

        // Assert
        let drag = world.resource::<DragState>().dragging.unwrap();
        assert!(
            !drag.stash_cursor_follow,
            "stash_cursor_follow should be false after exit"
        );
    }

    #[test]
    fn when_stash_follow_and_cursor_exits_stash_then_scale_spring_targets_one() {
        // Arrange
        let mut world = World::new();
        let entity = world
            .spawn((
                GlobalTransform2D(Affine2::IDENTITY),
                Collider::Aabb(Vec2::new(30.0, 45.0)),
            ))
            .id();
        let (spy, _, _) = make_spy_physics();
        world.insert_resource(PhysicsRes::new(spy));
        world.insert_resource(make_drag_info(entity, true));
        world.insert_resource(mouse_at_screen(Vec2::new(800.0, 400.0)));
        world.insert_resource(StashVisible(true));
        world.insert_resource(StashGrid::new(10, 10, 1));

        // Act
        run_system(&mut world);

        // Assert
        let spring = world
            .get::<ScaleSpring>(entity)
            .expect("ScaleSpring should be inserted");
        assert!(
            (spring.target - 1.0).abs() < 1e-4,
            "ScaleSpring target should be 1.0, got {}",
            spring.target
        );
    }

    // -----------------------------------------------------------------------
    // Enter stash: follow=false + cursor inside → remove physics, follow=true
    // -----------------------------------------------------------------------

    #[test]
    fn when_physics_drag_and_cursor_enters_stash_then_physics_body_removed() {
        // Arrange
        let mut world = World::new();
        let entity = world
            .spawn((
                GlobalTransform2D(Affine2::IDENTITY),
                Collider::Aabb(Vec2::new(30.0, 45.0)),
            ))
            .id();
        let (spy, _, remove_log) = make_spy_physics();
        world.insert_resource(PhysicsRes::new(spy));
        world.insert_resource(make_drag_info(entity, false));
        world.insert_resource(mouse_at_screen(Vec2::new(45.0, 57.5))); // inside slot (0,0)
        world.insert_resource(StashVisible(true));
        world.insert_resource(StashGrid::new(10, 10, 1));

        // Act
        run_system(&mut world);

        // Assert
        let calls = remove_log.lock().unwrap();
        assert_eq!(calls.len(), 1, "remove_body should be called once");
        assert_eq!(calls[0], entity);
    }

    #[test]
    fn when_physics_drag_and_cursor_enters_stash_then_follow_set_true() {
        // Arrange
        let mut world = World::new();
        let entity = world
            .spawn((
                GlobalTransform2D(Affine2::IDENTITY),
                Collider::Aabb(Vec2::new(30.0, 45.0)),
            ))
            .id();
        let (spy, _, _) = make_spy_physics();
        world.insert_resource(PhysicsRes::new(spy));
        world.insert_resource(make_drag_info(entity, false));
        world.insert_resource(mouse_at_screen(Vec2::new(45.0, 57.5)));
        world.insert_resource(StashVisible(true));
        world.insert_resource(StashGrid::new(10, 10, 1));

        // Act
        run_system(&mut world);

        // Assert
        let drag = world.resource::<DragState>().dragging.unwrap();
        assert!(
            drag.stash_cursor_follow,
            "stash_cursor_follow should be true after entry"
        );
    }

    #[test]
    fn when_physics_drag_and_cursor_enters_stash_then_scale_spring_targets_slot_scale() {
        // Arrange
        let mut world = World::new();
        let entity = world
            .spawn((
                GlobalTransform2D(Affine2::IDENTITY),
                Collider::Aabb(Vec2::new(30.0, 45.0)),
            ))
            .id();
        let (spy, _, _) = make_spy_physics();
        world.insert_resource(PhysicsRes::new(spy));
        world.insert_resource(make_drag_info(entity, false));
        world.insert_resource(mouse_at_screen(Vec2::new(45.0, 57.5)));
        world.insert_resource(StashVisible(true));
        world.insert_resource(StashGrid::new(10, 10, 1));

        // Act
        run_system(&mut world);

        // Assert
        let expected = SLOT_WIDTH / CARD_WIDTH;
        let spring = world
            .get::<ScaleSpring>(entity)
            .expect("ScaleSpring should be inserted");
        assert!(
            (spring.target - expected).abs() < 1e-4,
            "ScaleSpring target should be {expected}, got {}",
            spring.target
        );
    }

    // -----------------------------------------------------------------------
    // Stash hidden + follow=true → treated as exit
    // -----------------------------------------------------------------------

    #[test]
    fn when_stash_hidden_and_follow_true_then_physics_body_added() {
        // Arrange
        let mut world = World::new();
        let entity = world
            .spawn((
                GlobalTransform2D(Affine2::IDENTITY),
                Collider::Aabb(Vec2::new(30.0, 45.0)),
            ))
            .id();
        let (spy, add_log, _) = make_spy_physics();
        world.insert_resource(PhysicsRes::new(spy));
        world.insert_resource(make_drag_info(entity, true));
        world.insert_resource(mouse_at_screen(Vec2::new(45.0, 57.5))); // would be in stash, but stash hidden
        world.insert_resource(StashVisible(false));
        world.insert_resource(StashGrid::new(10, 10, 1));

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(
            add_log.lock().unwrap().len(),
            1,
            "stash hidden → over_stash=false → exit transition should fire"
        );
    }
}
