use std::collections::{HashMap, HashSet};

use bevy_ecs::prelude::Component;
use engine_core::types::TextureId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BlendMode {
    Alpha,
    Additive,
    Multiply,
}

impl BlendMode {
    pub const ALL: [Self; 3] = [Self::Alpha, Self::Additive, Self::Multiply];

    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShaderHandle(pub u32);

#[derive(Default)]
pub struct ShaderRegistry {
    sources: HashMap<ShaderHandle, String>,
    next_id: u32,
}

impl ShaderRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, source: &str) -> ShaderHandle {
        let handle = ShaderHandle(self.next_id);
        self.next_id += 1;
        self.sources.insert(handle, source.to_owned());
        handle
    }

    pub fn lookup(&self, handle: ShaderHandle) -> Option<&str> {
        self.sources.get(&handle).map(String::as_str)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextureBinding {
    pub texture: TextureId,
    pub binding: u32,
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct Material2d {
    pub blend_mode: BlendMode,
    pub shader: ShaderHandle,
    pub textures: Vec<TextureBinding>,
    pub uniforms: Vec<u8>,
}

#[must_use]
pub fn effective_blend_mode(material: Option<&Material2d>) -> BlendMode {
    material.map_or(BlendMode::Alpha, |m| m.blend_mode)
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

impl Default for Material2d {
    fn default() -> Self {
        Self {
            blend_mode: BlendMode::Alpha,
            shader: ShaderHandle(0),
            textures: Vec::new(),
            uniforms: Vec::new(),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn when_comparing_blend_modes_then_alpha_less_than_additive_less_than_multiply() {
        // Arrange
        let alpha = BlendMode::Alpha;
        let additive = BlendMode::Additive;
        let multiply = BlendMode::Multiply;

        // Act / Assert
        assert!(alpha < additive);
        assert!(additive < multiply);
    }

    #[test]
    fn when_registering_shader_then_lookup_returns_same_source() {
        // Arrange
        let mut registry = ShaderRegistry::new();
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
        let mut registry = ShaderRegistry::new();

        // Act
        let h1 = registry.register("shader_a");
        let h2 = registry.register("shader_b");

        // Assert
        assert_ne!(h1, h2);
    }

    #[test]
    fn when_looking_up_unregistered_handle_then_returns_none() {
        // Arrange
        let registry = ShaderRegistry::new();

        // Act
        let result = registry.lookup(ShaderHandle(99));

        // Assert
        assert_eq!(result, None);
    }

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
}
