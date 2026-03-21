use std::collections::{HashMap, HashSet};

use bevy_ecs::prelude::Resource;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ShaderHandle(pub u32);

#[derive(Resource)]
pub struct ShaderRegistry {
    sources: HashMap<ShaderHandle, String>,
    next_id: u32,
}

impl Default for ShaderRegistry {
    fn default() -> Self {
        Self {
            sources: HashMap::new(),
            next_id: 1, // 0 is reserved for the built-in default shader
        }
    }
}

impl ShaderRegistry {
    pub fn register(&mut self, source: &str) -> ShaderHandle {
        let handle = ShaderHandle(self.next_id);
        self.next_id += 1;
        self.sources.insert(handle, source.to_owned());
        handle
    }

    pub fn lookup(&self, handle: ShaderHandle) -> Option<&str> {
        self.sources.get(&handle).map(String::as_str)
    }

    pub fn iter(&self) -> impl Iterator<Item = (ShaderHandle, &str)> {
        self.sources.iter().map(|(&h, s)| (h, s.as_str()))
    }
}

#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn preprocess(source: &str, defines: &HashSet<&str>) -> String {
    let mut output = String::new();
    let mut skip_depth = 0u32;

    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("#ifdef ") {
            let name = rest.trim();
            if skip_depth > 0 || !defines.contains(name) {
                skip_depth += 1;
            }
        } else if trimmed == "#endif" {
            skip_depth = skip_depth.saturating_sub(1);
        } else if skip_depth == 0 {
            if !output.is_empty() {
                output.push('\n');
            }
            output.push_str(line);
        }
    }

    output
}

pub fn shader_prepare_system(
    registry: Option<bevy_ecs::prelude::Res<ShaderRegistry>>,
    mut renderer: bevy_ecs::prelude::ResMut<crate::renderer::RendererRes>,
) {
    let Some(registry) = registry else { return };
    for (handle, source) in registry.iter() {
        renderer
            .compile_shader(handle, source)
            .expect("shader compilation should succeed");
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

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
        let source = "before\n#ifdef OUTER\nmiddle\n#ifdef INNER\nskipped\n#endif\nafter_inner\n#endif\nfooter";
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
        use crate::testing::insert_spy_with_compile_shader_capture;
        use bevy_ecs::prelude::{Schedule, World};

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
        use crate::testing::insert_spy_with_compile_shader_capture;
        use bevy_ecs::prelude::{Schedule, World};

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
        use crate::testing::insert_spy_with_compile_shader_capture;
        use bevy_ecs::prelude::{Schedule, World};

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
        use crate::testing::insert_spy;
        use bevy_ecs::prelude::{Schedule, World};

        // Arrange
        let mut world = World::new();
        let _log = insert_spy(&mut world);
        let mut schedule = Schedule::default();
        schedule.add_systems(shader_prepare_system);

        // Act — should not panic
        schedule.run(&mut world);
    }

    #[test]
    fn when_shader_prepare_runs_before_shape_render_then_compile_precedes_draw_in_log() {
        use crate::shape::Shape;
        use crate::testing::insert_spy;
        use bevy_ecs::prelude::{Schedule, World};
        use bevy_ecs::schedule::IntoScheduleConfigs;
        use engine_scene::prelude::GlobalTransform2D;
        use glam::Affine2;

        // Arrange
        let mut world = World::new();
        let log = insert_spy(&mut world);
        let mut registry = ShaderRegistry::default();
        registry.register("test_shader");
        world.insert_resource(registry);
        world.spawn((
            Shape {
                variant: crate::shape::ShapeVariant::Circle { radius: 10.0 },
                color: engine_core::color::Color::WHITE,
            },
            GlobalTransform2D(Affine2::IDENTITY),
        ));
        let mut schedule = Schedule::default();
        schedule.add_systems((shader_prepare_system, crate::shape::shape_render_system).chain());

        // Act
        schedule.run(&mut world);

        // Assert
        let log = log.lock().unwrap();
        let compile_pos = log.iter().position(|s| s == "compile_shader");
        let draw_pos = log.iter().position(|s| s == "draw_shape");
        assert!(compile_pos.is_some(), "compile_shader should appear in log");
        assert!(draw_pos.is_some(), "draw_shape should appear in log");
        assert!(
            compile_pos.unwrap() < draw_pos.unwrap(),
            "compile_shader must run before draw_shape"
        );
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
}
