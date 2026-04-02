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
