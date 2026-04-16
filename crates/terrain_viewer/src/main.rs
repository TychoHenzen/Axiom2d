#![allow(clippy::unwrap_used)] // viewer binary, not library

use axiom2d::prelude::*;
use bevy_ecs::prelude::*;
use bytemuck::bytes_of;
use engine_ecs::schedule::Phase;
use engine_render::camera::Camera2D;
use engine_render::material::Material2d;
use engine_render::shader::ShaderRegistry;
use engine_render::shape::{ColorMesh, ColorVertex, TessellatedColorMesh};
use glam::{Affine2, Vec2};
use terrain::material::{TerrainMaterial, default_materials};
use terrain::shader::register_terrain_shader;
use terrain::{TerrainMaterials, TerrainShader};

/// Currently selected terrain type index.
#[derive(Resource, Debug)]
struct SelectedTerrain(usize);

/// Entity that displays the terrain quad.
#[derive(Resource, Debug)]
struct TerrainQuadEntity(Entity);

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing_subscriber::filter::LevelFilter::WARN)
        .init();

    let mut app = App::new();
    setup(&mut app);
    app.run();
}

fn setup(app: &mut App) {
    app.add_plugin(DefaultPlugins);

    app.set_window_config(WindowConfig {
        title: "Terrain Viewer",
        width: 1024,
        height: 768,
        ..Default::default()
    });

    // Register shader and insert resources before post-splash setup.
    {
        let world = app.world_mut();
        let shader_handle = {
            let mut reg = world.resource_mut::<ShaderRegistry>();
            register_terrain_shader(&mut reg)
        };
        world.insert_resource(TerrainShader(shader_handle));

        let materials = default_materials();
        world.insert_resource(TerrainMaterials(materials));
        world.insert_resource(SelectedTerrain(0));
    }

    app.world_mut()
        .resource_mut::<PostSplashSetup>()
        .add_systems(spawn_viewer_scene);

    app.add_systems(Phase::Update, terrain_input_system);
}

fn spawn_viewer_scene(world: &mut World) {
    // Camera
    world.spawn(Camera2D::default());

    let shader = world.resource::<TerrainShader>().0;
    let materials = world.resource::<TerrainMaterials>().clone();
    let selected = world.resource::<SelectedTerrain>().0;

    // Build terrain quad — large quad centered at origin
    let half = 350.0_f32;
    let mesh = TessellatedColorMesh {
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
    };

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
}

/// Pack a single `TerrainMaterial` into the uniform buffer for Phase 1.
/// Layout: material[0] = (type_id_bits, world_x, world_y, seed)
///         material[1..4] = MaterialParams
fn build_single_material_uniform(mat: &TerrainMaterial) -> Vec<u8> {
    let gpu = mat.to_gpu_params();
    let type_id_f32 = f32::from_bits(u32::from(mat.kind as u8));
    let header: [f32; 4] = [type_id_f32, 0.0, 0.0, 42.0]; // world_pos=(0,0), seed=42

    let mut buf = Vec::with_capacity(256);
    buf.extend_from_slice(bytes_of(&header));
    buf.extend_from_slice(bytes_of(&gpu));
    // Pad to 256 bytes (16 vec4s) for the uniform array
    buf.resize(256, 0);
    buf
}

fn terrain_input_system(
    input: Res<InputState>,
    mut selected: ResMut<SelectedTerrain>,
    materials: Res<TerrainMaterials>,
    quad: Res<TerrainQuadEntity>,
    mut query: Query<&mut Material2d>,
) {
    let count = materials.0.len();
    let mut changed = false;

    if input.just_pressed(KeyCode::ArrowRight) || input.just_pressed(KeyCode::KeyD) {
        selected.0 = (selected.0 + 1) % count;
        changed = true;
    }
    if input.just_pressed(KeyCode::ArrowLeft) || input.just_pressed(KeyCode::KeyA) {
        selected.0 = (selected.0 + count - 1) % count;
        changed = true;
    }

    // Number keys 1-6 for direct selection
    let key_map = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
    ];
    for (i, key) in key_map.iter().enumerate() {
        if i < count && input.just_pressed(*key) {
            selected.0 = i;
            changed = true;
        }
    }

    if changed {
        if let Ok(mut mat) = query.get_mut(quad.0) {
            mat.uniforms = build_single_material_uniform(&materials.0[selected.0]);
        }
    }
}
