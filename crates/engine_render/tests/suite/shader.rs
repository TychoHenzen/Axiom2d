#![allow(clippy::unwrap_used)]

use std::collections::HashSet;

use engine_render::shader::{ShaderHandle, ShaderRegistry, preprocess, shader_prepare_system};

#[test]
fn when_registering_shader_then_lookup_returns_same_source() {
    // Arrange
    let mut registry = ShaderRegistry::default();
    let source = "@vertex fn vs_main() {}";

    // Act
    let handle = registry.register(source);
    let result = registry.lookup(handle);

    // Assert
    assert_eq!(result, Some(source));
}

#[test]
fn when_registering_multiple_shaders_then_handles_are_unique() {
    // Arrange
    let mut registry = ShaderRegistry::default();

    // Act
    let h1 = registry.register("shader_a");
    let h2 = registry.register("shader_b");

    // Assert
    assert_ne!(h1, h2);
}

#[test]
fn when_looking_up_unregistered_handle_then_returns_none() {
    // Arrange
    let registry = ShaderRegistry::default();

    // Act
    let result = registry.lookup(ShaderHandle(99));

    // Assert
    assert_eq!(result, None);
}

#[test]
fn when_comparing_shader_handles_then_ordered_by_inner_u32() {
    // Arrange
    let a = ShaderHandle(0);
    let b = ShaderHandle(1);
    let c = ShaderHandle(99);

    // Act / Assert
    assert!(a < b);
    assert!(b < c);
}

#[test]
fn when_shader_registry_used_as_resource_in_system_then_lookup_works() {
    use bevy_ecs::prelude::{Res, Schedule, World};

    // Arrange
    let mut registry = ShaderRegistry::default();
    let _handle = registry.register("@vertex fn vs_main() {}");
    let mut world = World::new();
    world.insert_resource(registry);
    let mut schedule = Schedule::default();
    schedule.add_systems(|registry: Res<ShaderRegistry>| {
        assert_eq!(
            registry.lookup(ShaderHandle(1)),
            Some("@vertex fn vs_main() {}")
        );
    });

    // Act / Assert
    schedule.run(&mut world);
}

/// @doc: #ifdef preprocessor conditionally includes shader blocks — enables feature-based shader variants
#[test]
fn when_preprocessing_with_define_present_then_ifdef_block_included() {
    // Arrange
    let source = "header\n#ifdef MY_FEATURE\nfeature_line\n#endif\nfooter";
    let mut defines = HashSet::new();
    defines.insert("MY_FEATURE");

    // Act
    let result = preprocess(source, &defines);

    // Assert
    assert!(result.contains("feature_line"));
    assert!(result.contains("header"));
    assert!(result.contains("footer"));
}

#[test]
fn when_preprocessing_nested_ifdef_with_outer_defined_inner_not_then_inner_excluded() {
    // Arrange
    let source =
        "before\n#ifdef OUTER\nmiddle\n#ifdef INNER\nskipped\n#endif\nafter_inner\n#endif\nfooter";
    let mut defines = HashSet::new();
    defines.insert("OUTER");

    // Act
    let result = preprocess(source, &defines);

    // Assert
    assert!(result.contains("before"));
    assert!(result.contains("middle"));
    assert!(!result.contains("skipped"));
    assert!(result.contains("after_inner"));
    assert!(result.contains("footer"));
}

#[test]
fn when_preprocessing_without_define_then_ifdef_block_excluded() {
    // Arrange
    let source = "header\n#ifdef MY_FEATURE\nfeature_line\n#endif\nfooter";
    let defines = HashSet::new();

    // Act
    let result = preprocess(source, &defines);

    // Assert
    assert!(!result.contains("feature_line"));
    assert!(result.contains("header"));
    assert!(result.contains("footer"));
}

#[test]
fn when_preprocessing_outer_undefined_inner_defined_then_entire_block_excluded() {
    // Arrange — OUTER undefined, INNER defined; everything inside OUTER must be skipped
    let source = "before\n#ifdef OUTER\nouter_only\n#ifdef INNER\ninner_only\n#endif\nafter_inner\n#endif\nfooter";
    let mut defines = HashSet::new();
    defines.insert("INNER");

    // Act
    let result = preprocess(source, &defines);

    // Assert
    assert!(result.contains("before"));
    assert!(result.contains("footer"));
    assert!(!result.contains("outer_only"));
    assert!(!result.contains("inner_only"));
    assert!(!result.contains("after_inner"));
}

#[test]
fn when_shader_registry_has_one_entry_and_system_runs_then_compile_shader_called_once() {
    use bevy_ecs::prelude::{Schedule, World};
    use engine_render::testing::insert_spy_with_compile_shader_capture;

    // Arrange
    let mut world = World::new();
    let capture = insert_spy_with_compile_shader_capture(&mut world);
    let mut registry = ShaderRegistry::default();
    let handle = registry.register("my_shader_source");
    world.insert_resource(registry);
    let mut schedule = Schedule::default();
    schedule.add_systems(shader_prepare_system);

    // Act
    schedule.run(&mut world);

    // Assert
    let calls = capture.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, handle);
    assert_eq!(calls[0].1, "my_shader_source");
}

#[test]
fn when_shader_registry_has_two_entries_and_system_runs_then_compile_shader_called_for_each() {
    use bevy_ecs::prelude::{Schedule, World};
    use engine_render::testing::insert_spy_with_compile_shader_capture;

    // Arrange
    let mut world = World::new();
    let capture = insert_spy_with_compile_shader_capture(&mut world);
    let mut registry = ShaderRegistry::default();
    let h1 = registry.register("shader_a");
    let h2 = registry.register("shader_b");
    world.insert_resource(registry);
    let mut schedule = Schedule::default();
    schedule.add_systems(shader_prepare_system);

    // Act
    schedule.run(&mut world);

    // Assert
    let calls = capture.lock().unwrap();
    assert_eq!(calls.len(), 2);
    let mut handles: Vec<_> = calls.iter().map(|(h, _)| *h).collect();
    handles.sort();
    assert!(handles.contains(&h1));
    assert!(handles.contains(&h2));
}

#[test]
fn when_shader_registry_is_empty_and_system_runs_then_compile_shader_not_called() {
    use bevy_ecs::prelude::{Schedule, World};
    use engine_render::testing::insert_spy_with_compile_shader_capture;

    // Arrange
    let mut world = World::new();
    let capture = insert_spy_with_compile_shader_capture(&mut world);
    world.insert_resource(ShaderRegistry::default());
    let mut schedule = Schedule::default();
    schedule.add_systems(shader_prepare_system);

    // Act
    schedule.run(&mut world);

    // Assert
    let calls = capture.lock().unwrap();
    assert!(calls.is_empty());
}

#[test]
fn when_shader_registry_absent_from_world_and_system_runs_then_no_panic() {
    use bevy_ecs::prelude::{Schedule, World};
    use engine_render::testing::insert_spy;

    // Arrange
    let mut world = World::new();
    let _log = insert_spy(&mut world);
    let mut schedule = Schedule::default();
    schedule.add_systems(shader_prepare_system);

    // Act — should not panic
    schedule.run(&mut world);
}

#[test]
fn when_preprocessing_multiple_lines_then_separated_by_newlines() {
    // Arrange
    let source = "line_one\nline_two\nline_three";
    let defines = HashSet::new();

    // Act
    let result = preprocess(source, &defines);

    // Assert
    assert_eq!(result, "line_one\nline_two\nline_three");
}
