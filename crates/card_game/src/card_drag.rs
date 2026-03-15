use bevy_ecs::prelude::{Res, ResMut};
use engine_input::prelude::{MouseButton, MouseState};
use engine_physics::prelude::PhysicsRes;
use glam::Vec2;

use crate::drag_state::DragState;

pub const DRAG_GAIN: f32 = 20.0;
pub const ANGULAR_GAIN: f32 = 0.1;

pub fn card_drag_system(
    mouse: Res<MouseState>,
    drag_state: Res<DragState>,
    mut physics: ResMut<PhysicsRes>,
) {
    let Some(info) = &drag_state.dragging else {
        return;
    };
    if !mouse.pressed(MouseButton::Left) {
        return;
    }

    let Some(grab_world) = physics.body_point_to_world(info.entity, info.local_grab_offset) else {
        return;
    };
    let Some(body_pos) = physics.body_position(info.entity) else {
        return;
    };
    let cursor = mouse.world_pos();
    let displacement = cursor - grab_world;

    // Velocity proportional to displacement — card tracks cursor directly
    physics.set_linear_velocity(info.entity, DRAG_GAIN * displacement);

    // Torque from off-center grab creates natural trailing rotation
    let arm = grab_world - body_pos;
    let torque = arm.perp_dot(displacement);
    physics.set_angular_velocity(info.entity, ANGULAR_GAIN * torque);
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use bevy_ecs::prelude::*;
    use engine_input::prelude::{MouseButton, MouseState};
    use engine_physics::prelude::{PhysicsBackend, PhysicsRes};
    use glam::Vec2;

    use super::card_drag_system;
    use crate::card_zone::CardZone;
    use crate::drag_state::{DragInfo, DragState};

    type ForceLog = Arc<Mutex<Vec<(Entity, Vec2, Vec2)>>>;

    struct SpyPhysicsBackend {
        positions: HashMap<Entity, Vec2>,
        rotations: HashMap<Entity, f32>,
        force_log: ForceLog,
    }

    impl SpyPhysicsBackend {
        fn new(force_log: ForceLog) -> Self {
            Self {
                positions: HashMap::new(),
                rotations: HashMap::new(),
                force_log,
            }
        }

        fn with_body(mut self, entity: Entity, position: Vec2, rotation: f32) -> Self {
            self.positions.insert(entity, position);
            self.rotations.insert(entity, rotation);
            self
        }
    }

    impl PhysicsBackend for SpyPhysicsBackend {
        fn step(&mut self, _dt: engine_core::prelude::Seconds) {}
        fn add_body(&mut self, _: Entity, _: &engine_physics::prelude::RigidBody, _: Vec2) -> bool {
            false
        }
        fn add_collider(&mut self, _: Entity, _: &engine_physics::prelude::Collider) -> bool {
            false
        }
        fn remove_body(&mut self, _: Entity) {}
        fn body_position(&self, entity: Entity) -> Option<Vec2> {
            self.positions.get(&entity).copied()
        }
        fn body_rotation(&self, entity: Entity) -> Option<f32> {
            self.rotations.get(&entity).copied()
        }
        fn drain_collision_events(&mut self) -> Vec<engine_physics::prelude::CollisionEvent> {
            Vec::new()
        }
        fn body_linear_velocity(&self, _: Entity) -> Option<Vec2> {
            Some(Vec2::ZERO)
        }
        fn set_linear_velocity(&mut self, _: Entity, _: Vec2) {}
        fn set_angular_velocity(&mut self, _: Entity, _: f32) {}
        fn add_force_at_point(&mut self, entity: Entity, force: Vec2, world_point: Vec2) {
            self.force_log
                .lock()
                .unwrap()
                .push((entity, force, world_point));
        }
        fn set_damping(&mut self, _: Entity, _: f32, _: f32) {}
    }

    fn run_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(card_drag_system);
        schedule.run(world);
    }

    fn setup_drag_world(
        entity: Entity,
        local_grab_offset: Vec2,
        body_pos: Vec2,
        body_rot: f32,
        cursor_pos: Vec2,
        mouse_pressed: bool,
    ) -> (World, ForceLog) {
        let mut world = World::new();
        let force_log: ForceLog = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyPhysicsBackend::new(force_log.clone()).with_body(entity, body_pos, body_rot);
        world.insert_resource(PhysicsRes::new(Box::new(spy)));

        let mut mouse = MouseState::default();
        if mouse_pressed {
            mouse.press(MouseButton::Left);
        }
        mouse.set_world_pos(cursor_pos);
        world.insert_resource(mouse);

        world.insert_resource(DragState {
            dragging: Some(DragInfo {
                entity,
                local_grab_offset,
                origin_zone: CardZone::Table,
            }),
        });

        (world, force_log)
    }

    fn spawn_entity() -> Entity {
        World::new().spawn_empty().id()
    }

    #[test]
    fn when_dragging_and_cursor_offset_then_force_applied_toward_cursor() {
        // Arrange
        let entity = spawn_entity();
        let (mut world, force_log) = setup_drag_world(
            entity,
            Vec2::ZERO,
            Vec2::ZERO,
            0.0,
            Vec2::new(10.0, 0.0),
            true,
        );

        // Act
        run_system(&mut world);

        // Assert
        let calls = force_log.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert!(calls[0].1.x > 0.0, "force should point toward cursor (+X)");
    }

    #[test]
    fn when_cursor_equals_grab_point_then_zero_force() {
        // Arrange
        let entity = spawn_entity();
        let (mut world, force_log) = setup_drag_world(
            entity,
            Vec2::ZERO,
            Vec2::new(5.0, 5.0),
            0.0,
            Vec2::new(5.0, 5.0),
            true,
        );

        // Act
        run_system(&mut world);

        // Assert
        let calls = force_log.lock().unwrap();
        if !calls.is_empty() {
            assert!(calls[0].1.length() < 1e-6, "force should be ~zero");
        }
    }

    #[test]
    fn when_cursor_twice_as_far_then_force_magnitude_doubles() {
        // Arrange
        let entity_a = spawn_entity();
        let (mut world_a, log_a) = setup_drag_world(
            entity_a,
            Vec2::ZERO,
            Vec2::ZERO,
            0.0,
            Vec2::new(10.0, 0.0),
            true,
        );
        run_system(&mut world_a);

        let entity_b = spawn_entity();
        let (mut world_b, log_b) = setup_drag_world(
            entity_b,
            Vec2::ZERO,
            Vec2::ZERO,
            0.0,
            Vec2::new(20.0, 0.0),
            true,
        );
        run_system(&mut world_b);

        // Assert
        let calls_a = log_a.lock().unwrap();
        let calls_b = log_b.lock().unwrap();
        assert_eq!(calls_a.len(), 1);
        assert_eq!(calls_b.len(), 1);
        let ratio = calls_b[0].1.length() / calls_a[0].1.length();
        assert!(
            (ratio - 2.0).abs() < 1e-4,
            "force ratio should be 2.0, got {ratio}"
        );
    }

    #[test]
    fn when_drag_has_offset_then_force_applied_at_grab_world_point() {
        // Arrange
        let entity = spawn_entity();
        let (mut world, force_log) = setup_drag_world(
            entity,
            Vec2::new(5.0, 0.0),
            Vec2::ZERO,
            0.0,
            Vec2::new(10.0, 0.0),
            true,
        );

        // Act
        run_system(&mut world);

        // Assert
        let calls = force_log.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert!(
            (calls[0].2.x - 5.0).abs() < 1e-4,
            "force world_point.x should be 5.0, got {}",
            calls[0].2.x
        );
    }

    #[test]
    fn when_not_dragging_then_no_force_applied() {
        // Arrange
        let mut world = World::new();
        let force_log: ForceLog = Arc::new(Mutex::new(Vec::new()));
        let entity = spawn_entity();
        let spy = SpyPhysicsBackend::new(force_log.clone()).with_body(entity, Vec2::ZERO, 0.0);
        world.insert_resource(PhysicsRes::new(Box::new(spy)));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::new(10.0, 0.0));
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());

        // Act
        run_system(&mut world);

        // Assert
        let calls = force_log.lock().unwrap();
        assert!(calls.is_empty());
    }

    #[test]
    fn when_dragging_but_mouse_not_pressed_then_no_force_applied() {
        // Arrange
        let entity = spawn_entity();
        let (mut world, force_log) = setup_drag_world(
            entity,
            Vec2::ZERO,
            Vec2::ZERO,
            0.0,
            Vec2::new(10.0, 0.0),
            false,
        );

        // Act
        run_system(&mut world);

        // Assert
        let calls = force_log.lock().unwrap();
        assert!(calls.is_empty());
    }
}
