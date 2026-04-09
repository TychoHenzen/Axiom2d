use bevy_ecs::prelude::Resource;
use engine_core::color::Color;
use engine_render::material::Material2d;
use engine_render::rect::Rect;
use engine_render::renderer::GpuMeshHandle;
use engine_render::shape::{TessellatedColorMesh, TessellatedMesh};
use engine_scene::prelude::{RenderLayer, SortOrder};

pub struct StrokeCommand {
    pub mesh: TessellatedMesh,
    pub color: Color,
}

pub struct OverlayCommand {
    pub mesh: TessellatedColorMesh,
    pub material: Material2d,
}

pub enum DrawCommand {
    Shape {
        mesh: TessellatedMesh,
        color: Color,
        model: [[f32; 4]; 4],
        material: Option<Material2d>,
        stroke: Option<StrokeCommand>,
    },
    Text {
        content: String,
        font_size: f32,
        color: Color,
        max_width: Option<f32>,
        transform: glam::Affine2,
    },
    ColorMesh {
        mesh: TessellatedColorMesh,
        model: [[f32; 4]; 4],
        material: Option<Material2d>,
        overlays: Vec<OverlayCommand>,
    },
    PersistentMesh {
        handle: GpuMeshHandle,
        model: [[f32; 4]; 4],
        material: Option<Material2d>,
        overlays: Vec<OverlayCommand>,
    },
    Sprite {
        rect: Rect,
        uv_rect: [f32; 4],
        material: Option<Material2d>,
    },
}

pub struct SortedDrawCommand {
    pub sort_key: (RenderLayer, SortOrder),
    pub command: DrawCommand,
}

#[derive(Resource, Default)]
pub struct DrawQueue {
    commands: Vec<SortedDrawCommand>,
}

impl DrawQueue {
    /// Push a draw command with explicit sort position.
    /// Systems in Phase::Render push here; unified_render_system drains in Phase::PostRender.
    pub fn push(&mut self, layer: RenderLayer, order: SortOrder, command: DrawCommand) {
        self.commands.push(SortedDrawCommand {
            sort_key: (layer, order),
            command,
        });
    }

    pub(crate) fn drain(&mut self) -> Vec<SortedDrawCommand> {
        std::mem::take(&mut self.commands)
    }
}
