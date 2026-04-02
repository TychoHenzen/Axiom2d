use bevy_ecs::prelude::Component;
use engine_render::renderer::GpuMeshHandle;

/// Persistent GPU vertex/index buffers for a card's front and back faces.
/// Uploaded once at spawn via [`Renderer::upload_persistent_colored_mesh`].
/// The render system reads the correct handle based on [`Card::face_up`].
#[derive(Component, Debug, Clone, Copy)]
pub struct GpuCardMesh {
    pub front: GpuMeshHandle,
    pub back: GpuMeshHandle,
}
