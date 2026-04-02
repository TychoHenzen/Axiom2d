#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::*;
use engine_core::color::Color;

use engine_render::shape::{CachedMesh, Shape, ShapeVariant, mesh_cache_system, tessellate};

fn run_system(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(mesh_cache_system);
    schedule.run(world);
}

/// @doc: `mesh_cache_system` is the single point of truth for fill-mesh geometry on Shape
/// entities. Every downstream consumer (render, culling, physics) reads `CachedMesh` rather
/// than re-tessellating independently, keeping tessellation cost to once per dirty frame.
/// Without this system, render systems each tessellate the same shape every frame, and there
/// is no shared mesh that other systems can inspect without GPU access.
#[test]
fn when_shape_added_then_cached_mesh_component_present_on_entity() {
    // Arrange
    let mut world = World::new();
    let entity = world
        .spawn(Shape {
            variant: ShapeVariant::Circle { radius: 50.0 },
            color: Color::new(1.0, 0.0, 0.0, 1.0),
        })
        .id();

    // Act
    run_system(&mut world);

    // Assert
    assert!(
        world.entity(entity).contains::<CachedMesh>(),
        "CachedMesh must be inserted by mesh_cache_system after Shape is spawned"
    );
}

/// @doc: `CachedMesh` must contain real geometry after tessellation — an empty mesh would
/// cause shapes to render as invisible even though the component exists. Verifying vertex
/// count > 0 ensures `mesh_cache_system` called `tessellate` and stored the result rather
/// than inserting a zero-filled placeholder. Without this contract, a stub implementation
/// could satisfy the existence check while producing nothing drawable.
#[test]
fn when_shape_added_then_cached_mesh_contains_nonempty_vertices() {
    // Arrange
    let mut world = World::new();
    let entity = world
        .spawn(Shape {
            variant: ShapeVariant::Circle { radius: 30.0 },
            color: Color::new(0.0, 1.0, 0.0, 1.0),
        })
        .id();

    // Act
    run_system(&mut world);

    // Assert
    let cached = world
        .entity(entity)
        .get::<CachedMesh>()
        .expect("CachedMesh should be present");
    assert!(
        !cached.0.vertices.is_empty(),
        "CachedMesh must contain tessellated vertices, got empty mesh"
    );
    assert!(
        !cached.0.indices.is_empty(),
        "CachedMesh must contain tessellated indices, got empty mesh"
    );
}

/// @doc: When a `Shape` component is mutated (e.g., radius changed), the cached mesh must be
/// re-tessellated to match. Without re-tessellation on `Changed<Shape>`, the cached geometry
/// goes stale and the rendered shape silently diverges from the logical shape — a circle
/// resized to radius 100 would still render with radius 30 vertices.
#[test]
fn when_shape_mutated_then_cached_mesh_updated_to_new_variant() {
    // Arrange — spawn with small circle, run system to populate cache.
    let mut world = World::new();
    let entity = world
        .spawn(Shape {
            variant: ShapeVariant::Circle { radius: 30.0 },
            color: Color::new(1.0, 1.0, 1.0, 1.0),
        })
        .id();
    run_system(&mut world);

    let old_vertex_count = world
        .entity(entity)
        .get::<CachedMesh>()
        .unwrap()
        .0
        .vertices
        .len();

    // Act — mutate shape to a larger circle, run system again.
    world.entity_mut(entity).get_mut::<Shape>().unwrap().variant =
        ShapeVariant::Circle { radius: 100.0 };
    run_system(&mut world);

    // Assert — cached mesh should now match the new variant.
    let expected =
        tessellate(&ShapeVariant::Circle { radius: 100.0 }).expect("tessellate must succeed");
    let cached = world.entity(entity).get::<CachedMesh>().unwrap();
    assert_eq!(
        cached.0.vertices, expected.vertices,
        "CachedMesh must be updated when Shape changes (old had {old_vertex_count} verts)"
    );
}

/// @doc: `CachedMesh` must store geometry that matches what `tessellate` returns for the same
/// variant — any divergence means render and cache are operating on different meshes, causing
/// visual inconsistencies between cached and live draw paths. This test pins the contract that
/// the cache is a true mirror of the tessellation output, not an approximation. Without it,
/// an implementation could cache a bounding-box quad instead of the real mesh and pass the
/// existence and non-empty checks above.
#[test]
fn when_shape_added_then_cached_mesh_matches_direct_tessellation_output() {
    // Arrange
    let variant = ShapeVariant::Circle { radius: 40.0 };
    let mut world = World::new();
    let entity = world
        .spawn(Shape {
            variant: variant.clone(),
            color: Color::new(0.0, 0.0, 1.0, 1.0),
        })
        .id();

    let expected = tessellate(&variant).expect("tessellate must succeed for a valid circle");

    // Act
    run_system(&mut world);

    // Assert
    let cached = world
        .entity(entity)
        .get::<CachedMesh>()
        .expect("CachedMesh should be present");
    assert_eq!(
        cached.0.vertices, expected.vertices,
        "CachedMesh vertices must match tessellate() output"
    );
    assert_eq!(
        cached.0.indices, expected.indices,
        "CachedMesh indices must match tessellate() output"
    );
}
