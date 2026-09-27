use std::fmt::Write as _;

use axiom2d::prelude::{Color, GlobalTransform2D, RenderLayer, SortOrder, Transform2D, World};
use bytemuck::bytes_of;
use engine_render::camera::Camera2D;
use engine_render::material::Material2d;
use engine_render::shader::ShaderHandle;
use engine_render::shape::{ColorMesh, ColorVertex, TessellatedColorMesh};
use glam::{Affine2, Vec2};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use terrain::dual_grid::{DualGrid, VisualTile};
use terrain::material::{TerrainId, TerrainMaterial};
use terrain::wfc::{ConstraintTable, Grid as WfcGrid, collapse};
use terrain::{TerrainMaterials, TerrainShader};
use tracing::warn;

use crate::state::{HudTextEntity, TerrainQuadEntity};
use crate::state::{SelectedTerrain, ViewerMode};

pub(crate) fn build_quad_mesh(half: f32) -> TessellatedColorMesh {
    TessellatedColorMesh {
        vertices: vec![
            ColorVertex {
                position: [-half, -half],
                color: [1.0; 4],
                uv: [0.0, 0.0],
            },
            ColorVertex {
                position: [half, -half],
                color: [1.0; 4],
                uv: [1.0, 0.0],
            },
            ColorVertex {
                position: [half, half],
                color: [1.0; 4],
                uv: [1.0, 1.0],
            },
            ColorVertex {
                position: [-half, half],
                color: [1.0; 4],
                uv: [0.0, 1.0],
            },
        ],
        indices: vec![0, 1, 2, 0, 2, 3],
    }
}

pub(crate) fn spawn_viewer_scene(world: &mut World) {
    world.spawn(Camera2D::default());

    let shader = world.resource::<TerrainShader>().0;
    let materials = world.resource::<TerrainMaterials>().clone();
    let selected = world.resource::<SelectedTerrain>().0;

    let mesh = build_quad_mesh(350.0);
    let uniforms = build_single_material_uniform(&materials.0[selected]);

    let entity = world
        .spawn((
            Transform2D {
                position: Vec2::ZERO,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
            GlobalTransform2D(Affine2::IDENTITY),
            ColorMesh(mesh),
            Material2d {
                shader,
                uniforms,
                ..Material2d::default()
            },
            RenderLayer::Background,
            SortOrder::default(),
        ))
        .id();

    world.insert_resource(TerrainQuadEntity(entity));

    let hud = world
        .spawn((
            engine_ui::widget::Text {
                content: format_hud(&materials.0[selected], ViewerMode::SingleMaterial, None),
                font_size: 16.0,
                color: Color::WHITE,
                max_width: None,
            },
            GlobalTransform2D(Affine2::from_translation(Vec2::new(-480.0, 350.0))),
            RenderLayer::UI,
            SortOrder::new(100),
        ))
        .id();
    world.insert_resource(HudTextEntity(hud));
}

pub(crate) fn format_hud(mat: &TerrainMaterial, mode: ViewerMode, seed: Option<u64>) -> String {
    let mode_str = match mode {
        ViewerMode::SingleMaterial => "Single",
        ViewerMode::DualGrid => "DualGrid",
        ViewerMode::WfcGrid => "WFC",
    };
    let mut s = format!(
        "[{mode_str}] {} | freq:{:.1} amp:{:.2} warp:{:.2} scale:{:.1}\n\
         color_a: ({:.2},{:.2},{:.2})  color_b: ({:.2},{:.2},{:.2})\n\
         [1-6] type  [Q/W] freq  [E/R] amp  [T/Y] warp  [Tab] mode  [Shift] fast",
        mat.kind.name(),
        mat.params[0],
        mat.params[1],
        mat.params[2],
        mat.params[3],
        mat.color_a[0],
        mat.color_a[1],
        mat.color_a[2],
        mat.color_b[0],
        mat.color_b[1],
        mat.color_b[2],
    );
    if let Some(seed) = seed {
        let _ = write!(s, "\nseed: {seed}  [G] generate  [N] re-roll  [RMB] pan");
    }
    s
}

pub(crate) fn build_single_material_uniform(mat: &TerrainMaterial) -> Vec<u8> {
    let gpu = mat.to_gpu_params();
    let kind = u32::from(mat.kind as u8);
    let packed_corners = kind | (kind << 8) | (kind << 16) | (kind << 24);
    let header: [f32; 4] = [f32::from_bits(packed_corners), 0.0, 0.0, 42.0];

    let mut buf = Vec::with_capacity(256);
    buf.extend_from_slice(bytes_of(&header));
    buf.extend_from_slice(bytes_of(&gpu));
    buf.resize(256, 0);
    buf
}

pub(crate) fn build_tile_uniform(tile: &VisualTile, materials: &[TerrainMaterial]) -> Vec<u8> {
    let kind_of = |id: TerrainId| -> u32 {
        materials
            .iter()
            .find(|m| m.id == id)
            .map_or(0, |m| u32::from(m.kind as u8))
    };

    let c = tile.corners;
    let packed =
        kind_of(c[0]) | (kind_of(c[1]) << 8) | (kind_of(c[2]) << 16) | (kind_of(c[3]) << 24);

    let header: [f32; 4] = [
        f32::from_bits(packed),
        tile.x,
        tile.y,
        (tile.seed % 1000) as f32,
    ];

    let gpu_primary = materials
        .iter()
        .find(|m| m.id == c[0])
        .map(TerrainMaterial::to_gpu_params)
        .unwrap_or_default();
    let other_id = c.iter().find(|&&id| id != c[0]).copied().unwrap_or(c[0]);
    let gpu_secondary = materials
        .iter()
        .find(|m| m.id == other_id)
        .map(TerrainMaterial::to_gpu_params)
        .unwrap_or_default();

    let mut buf = Vec::with_capacity(256);
    buf.extend_from_slice(bytes_of(&header));
    buf.extend_from_slice(bytes_of(&gpu_primary));
    buf.extend_from_slice(bytes_of(&gpu_secondary));
    buf.resize(256, 0);
    buf
}

pub(crate) fn spawn_grid_tiles(
    world: &mut World,
    grid: &DualGrid,
    shader: ShaderHandle,
    materials: &[TerrainMaterial],
    tile_size: f32,
) -> Vec<bevy_ecs::entity::Entity> {
    let tiles = grid.visual_tiles();
    let half = tile_size / 2.0;
    let grid_offset = Vec2::new(
        -(grid.width() as f32) * tile_size / 2.0,
        -(grid.height() as f32) * tile_size / 2.0,
    );

    let mut entities = Vec::new();
    for tile in &tiles {
        let pos = grid_offset + Vec2::new(tile.x * tile_size, tile.y * tile_size);
        let mesh = build_quad_mesh(half);
        let uniforms = build_tile_uniform(tile, materials);

        let entity = world
            .spawn((
                Transform2D {
                    position: pos,
                    rotation: 0.0,
                    scale: Vec2::ONE,
                },
                GlobalTransform2D(Affine2::IDENTITY),
                ColorMesh(mesh),
                Material2d {
                    shader,
                    uniforms,
                    ..Material2d::default()
                },
                RenderLayer::Background,
                SortOrder::default(),
            ))
            .id();
        entities.push(entity);
    }
    entities
}

pub(crate) fn spawn_dual_grid(
    world: &mut World,
    shader: ShaderHandle,
    materials: &[TerrainMaterial],
) -> Vec<bevy_ecs::entity::Entity> {
    let mut grid = DualGrid::new(4, 4, TerrainId(0));
    for x in 0..4 {
        for y in 2..4 {
            grid.set(x, y, TerrainId(1));
        }
    }
    grid.set(1, 1, TerrainId(3));
    grid.set(2, 2, TerrainId(3));

    spawn_grid_tiles(world, &grid, shader, materials, 120.0)
}

fn default_constraints(materials: &[TerrainMaterial]) -> ConstraintTable {
    let types: Vec<TerrainId> = materials.iter().map(|m| m.id).collect();
    let mut table = ConstraintTable::new(types.clone());
    for &a in &types {
        for &b in &types {
            table.allow(a, b);
        }
    }
    table
}

pub(crate) fn generate_wfc_grid(
    materials: &[TerrainMaterial],
    width: usize,
    height: usize,
    seed: u64,
) -> DualGrid {
    let constraints = default_constraints(materials);
    let mut wfc_grid = WfcGrid::new(width, height);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    if let Ok(()) = collapse(&mut wfc_grid, &constraints, &mut rng) {
        let mut dual = DualGrid::new(width, height, TerrainId(0));
        for y in 0..height {
            for x in 0..width {
                if let Some(id) = wfc_grid.get(x, y) {
                    dual.set(x, y, id);
                }
            }
        }
        dual
    } else {
        warn!("WFC grid collapse failed: seed={seed}");
        DualGrid::new(width, height, TerrainId(0))
    }
}
