use bevy_ecs::prelude::Entity;
use engine_physics::prelude::{Collider, PhysicsError, PhysicsRes, RigidBody};
use glam::Vec2;

use crate::card::interaction::damping::{BASE_ANGULAR_DRAG, BASE_LINEAR_DRAG};

/// Report a physics operation that returned `Err` so the failure is visible in logs.
pub(crate) fn warn_on_physics_result(
    operation: &'static str,
    entity: Entity,
    result: Result<(), PhysicsError>,
) {
    if let Err(error) = result {
        tracing::warn!(?entity, operation, error = %error, "physics operation failed");
    }
}

/// Report a physics operation that returned `false` so the failure is visible in logs.
pub(crate) fn warn_on_physics_bool(operation: &'static str, entity: Entity, success: bool) {
    if !success {
        tracing::warn!(?entity, operation, "physics operation failed");
    }
}

/// Register a dynamic rigid body in the physics backend with base damping
/// and the given collision group. No-ops if the entity already has a body.
pub(crate) fn activate_physics_body(
    entity: Entity,
    position: Vec2,
    collider: &Collider,
    physics: &mut PhysicsRes,
    membership: u32,
    filter: u32,
) {
    if physics.body_position(entity).is_some() {
        return;
    }
    warn_on_physics_bool(
        "add_body",
        entity,
        physics.add_body(entity, &RigidBody::Dynamic, position),
    );
    warn_on_physics_bool(
        "add_collider",
        entity,
        physics.add_collider(entity, collider),
    );
    physics
        .set_damping(entity, BASE_LINEAR_DRAG, BASE_ANGULAR_DRAG)
        .expect("activate_physics_body: entity should have been just added");
    physics
        .set_collision_group(entity, membership, filter)
        .expect("activate_physics_body: entity should have been just added");
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::io::{self, Write};
    use std::sync::{Arc, Mutex};

    use bevy_ecs::prelude::World;
    use tracing_subscriber::fmt::MakeWriter;

    use super::{warn_on_physics_bool, warn_on_physics_result};
    use engine_physics::prelude::PhysicsError;

    struct BufferWriter(Arc<Mutex<Vec<u8>>>);

    struct BufferGuard(Arc<Mutex<Vec<u8>>>);

    impl<'a> MakeWriter<'a> for BufferWriter {
        type Writer = BufferGuard;

        fn make_writer(&'a self) -> Self::Writer {
            BufferGuard(self.0.clone())
        }
    }

    impl Write for BufferGuard {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    fn capture_log(output: impl FnOnce()) -> String {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let subscriber = tracing_subscriber::fmt()
            .with_ansi(false)
            .without_time()
            .with_writer(BufferWriter(buffer.clone()))
            .finish();
        let dispatch = tracing::Dispatch::new(subscriber);
        tracing::dispatcher::with_default(&dispatch, output);
        String::from_utf8(buffer.lock().unwrap().clone()).expect("log output should be UTF-8")
    }

    #[test]
    fn when_physics_result_is_err_then_warning_is_emitted() {
        // Arrange
        let entity = World::new().spawn_empty().id();

        // Act
        let output = capture_log(|| {
            warn_on_physics_result(
                "remove_body",
                entity,
                Err(PhysicsError::EntityNotFound(entity)),
            );
        });

        // Assert
        assert!(output.contains("remove_body"), "operation should be logged");
        assert!(
            output.contains("not found in physics world"),
            "error should be logged"
        );
        assert!(
            output.contains("physics operation failed"),
            "warning text should be logged"
        );
    }

    #[test]
    fn when_physics_bool_is_false_then_warning_is_emitted() {
        // Arrange
        let entity = World::new().spawn_empty().id();

        // Act
        let output = capture_log(|| {
            warn_on_physics_bool("add_body", entity, false);
        });

        // Assert
        assert!(output.contains("add_body"), "operation should be logged");
        assert!(
            output.contains("physics operation failed"),
            "warning text should be logged"
        );
    }
}
