#![allow(clippy::unwrap_used)] // viewer binary, not library

use axiom2d::prelude::*;
use bytemuck::bytes_of;
use engine_ecs::schedule::Phase;
use engine_input::mouse::MouseState;
use engine_render::camera::Camera2D;
use engine_render::material::Material2d;
use engine_render::shader::ShaderRegistry;
use engine_render::shape::{ColorMesh, ColorVertex, TessellatedColorMesh};
use glam::{Affine2, Vec2};
use terrain::material::{TerrainId, TerrainMaterial, default_materials};
use terrain::shader::register_terrain_shader;
use terrain::{TerrainMaterials, TerrainShader};

/// Currently selected terrain type index.
#[derive(Resource, Debug)]
struct SelectedTerrain(usize);

/// Entity that displays the terrain quad.
#[derive(Resource, Debug)]
struct TerrainQuadEntity(Entity);

/// Entity for the HUD text overlay.
#[derive(Resource, Debug)]
struct HudTextEntity(Entity);

/// Viewer display mode.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
enum ViewerMode {
    SingleMaterial,
    DualGrid,
}

/// Tracks all tile entities for cleanup when switching modes.
#[derive(Resource, Debug, Default)]
struct GridTileEntities(Vec<Entity>);

/// Flag requesting a mode switch on next frame.
#[derive(Resource, Debug, Default)]
struct ModeSwitchRequested(bool);

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
        world.insert_resource(ViewerMode::SingleMaterial);
        world.insert_resource(GridTileEntities::default());
        world.insert_resource(ModeSwitchRequested::default());
    }

    app.world_mut()
        .resource_mut::<PostSplashSetup>()
        .add_systems(spawn_viewer_scene);

    app.add_systems(
        Phase::Update,
        (terrain_input_system, hud_update_system).chain(),
    );
    app.add_systems(Phase::Update, camera_zoom_system);
    app.add_systems(
        Phase::Update,
        mode_switch_system.after(terrain_input_system),
    );
}

fn spawn_viewer_scene(world: &mut World) {
    // Camera
    world.spawn(Camera2D::default());

    let shader = world.resource::<TerrainShader>().0;
    let materials = world.resource::<TerrainMaterials>().clone();
    let selected = world.resource::<SelectedTerrain>().0;

    // Build terrain quad -- large quad centered at origin
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

    let hud = world
        .spawn((
            engine_ui::widget::Text {
                content: format_hud(&materials.0[selected], ViewerMode::SingleMaterial),
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

fn format_hud(mat: &TerrainMaterial, mode: ViewerMode) -> String {
    let mode_str = match mode {
        ViewerMode::SingleMaterial => "Single",
        ViewerMode::DualGrid => "DualGrid",
    };
    format!(
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
    )
}

/// Pack a single `TerrainMaterial` into the uniform buffer.
/// Layout: material[0] = (packed_corners, world_x, world_y, seed)
///         material[1..4] = MaterialParams
fn build_single_material_uniform(mat: &TerrainMaterial) -> Vec<u8> {
    let gpu = mat.to_gpu_params();
    let kind = u32::from(mat.kind as u8);
    let packed_corners = kind | (kind << 8) | (kind << 16) | (kind << 24);
    let header: [f32; 4] = [f32::from_bits(packed_corners), 0.0, 0.0, 42.0];

    let mut buf = Vec::with_capacity(256);
    buf.extend_from_slice(bytes_of(&header));
    buf.extend_from_slice(bytes_of(&gpu));
    // Pad to 256 bytes (16 vec4s) for the uniform array
    buf.resize(256, 0);
    buf
}

/// Pack a `VisualTile` into the uniform buffer for dual-grid rendering.
/// Layout: material[0] = (packed_corners, world_x, world_y, seed)
///         material[1..4] = primary MaterialParams
///         material[5..8] = secondary MaterialParams
fn build_tile_uniform(
    tile: &terrain::dual_grid::VisualTile,
    materials: &[TerrainMaterial],
) -> Vec<u8> {
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

    // Primary material = first corner's type
    let gpu_primary = materials
        .iter()
        .find(|m| m.id == c[0])
        .map(TerrainMaterial::to_gpu_params)
        .unwrap_or_default();

    // Secondary = first corner that differs from c[0]
    let other_id = c.iter().find(|&&id| id != c[0]).copied().unwrap_or(c[0]);
    let gpu_secondary = materials
        .iter()
        .find(|m| m.id == other_id)
        .map(TerrainMaterial::to_gpu_params)
        .unwrap_or_default();

    let mut buf = Vec::with_capacity(256);
    buf.extend_from_slice(bytes_of(&header));
    buf.extend_from_slice(bytes_of(&gpu_primary)); // slots 1-4
    buf.extend_from_slice(bytes_of(&gpu_secondary)); // slots 5-8
    buf.resize(256, 0);
    buf
}

fn spawn_dual_grid(
    world: &mut World,
    shader: engine_render::shader::ShaderHandle,
    materials: &[TerrainMaterial],
) -> Vec<Entity> {
    use terrain::dual_grid::DualGrid;

    let mut grid = DualGrid::new(4, 4, TerrainId(0));
    // Bottom half: Stone
    for x in 0..4 {
        for y in 2..4 {
            grid.set(x, y, TerrainId(1));
        }
    }
    // Some sand patches
    grid.set(1, 1, TerrainId(3));
    grid.set(2, 2, TerrainId(3));

    let tiles = grid.visual_tiles();
    let tile_size = 120.0_f32;
    let half = tile_size / 2.0;
    let grid_offset = Vec2::new(
        -(grid.width() as f32) * tile_size / 2.0,
        -(grid.height() as f32) * tile_size / 2.0,
    );

    let mut entities = Vec::new();
    for tile in &tiles {
        let pos = grid_offset + Vec2::new(tile.x * tile_size, tile.y * tile_size);
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

fn terrain_input_system(
    input: Res<InputState>,
    mut selected: ResMut<SelectedTerrain>,
    mut materials: ResMut<TerrainMaterials>,
    quad: Res<TerrainQuadEntity>,
    mut mode_switch: ResMut<ModeSwitchRequested>,
    mode: Res<ViewerMode>,
    mut query: Query<&mut Material2d>,
) {
    // Mode toggle
    if input.just_pressed(KeyCode::Tab) {
        mode_switch.0 = true;
    }

    let count = materials.0.len();
    let mut changed = false;

    // --- Terrain type switching ---
    if input.just_pressed(KeyCode::ArrowRight) || input.just_pressed(KeyCode::KeyD) {
        selected.0 = (selected.0 + 1) % count;
        changed = true;
    }
    if input.just_pressed(KeyCode::ArrowLeft) || input.just_pressed(KeyCode::KeyA) {
        selected.0 = (selected.0 + count - 1) % count;
        changed = true;
    }
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

    // --- Parameter adjustment ---
    let step = if input.pressed(KeyCode::ShiftLeft) {
        0.5
    } else {
        0.1
    };
    let mat = &mut materials.0[selected.0];

    // Q/W: frequency
    if input.just_pressed(KeyCode::KeyQ) {
        mat.params[0] = (mat.params[0] - step).max(0.1);
        changed = true;
    }
    if input.just_pressed(KeyCode::KeyW) {
        mat.params[0] += step;
        changed = true;
    }
    // E/R: amplitude
    if input.just_pressed(KeyCode::KeyE) {
        mat.params[1] = (mat.params[1] - step * 0.5).max(0.0);
        changed = true;
    }
    if input.just_pressed(KeyCode::KeyR) {
        mat.params[1] += step * 0.5;
        changed = true;
    }
    // T/Y: warp strength
    if input.just_pressed(KeyCode::KeyT) {
        mat.params[2] = (mat.params[2] - step * 0.5).max(0.0);
        changed = true;
    }
    if input.just_pressed(KeyCode::KeyY) {
        mat.params[2] += step * 0.5;
        changed = true;
    }

    // Only update the single quad in SingleMaterial mode
    if changed && *mode == ViewerMode::SingleMaterial {
        if let Ok(mut mat2d) = query.get_mut(quad.0) {
            mat2d.uniforms = build_single_material_uniform(&materials.0[selected.0]);
        }
    }
}

fn mode_switch_system(world: &mut World) {
    let requested = world.resource::<ModeSwitchRequested>().0;
    if !requested {
        return;
    }
    world.resource_mut::<ModeSwitchRequested>().0 = false;

    let current_mode = *world.resource::<ViewerMode>();
    let shader = world.resource::<TerrainShader>().0;
    let materials = world.resource::<TerrainMaterials>().clone();
    let selected = world.resource::<SelectedTerrain>().0;

    match current_mode {
        ViewerMode::SingleMaterial => {
            // Switch to DualGrid
            let quad_entity = world.resource::<TerrainQuadEntity>().0;
            world.despawn(quad_entity);

            let entities = spawn_dual_grid(world, shader, &materials.0);
            world.insert_resource(GridTileEntities(entities));
            world.insert_resource(ViewerMode::DualGrid);
        }
        ViewerMode::DualGrid => {
            // Switch to SingleMaterial
            let tile_entities = std::mem::take(&mut world.resource_mut::<GridTileEntities>().0);
            for entity in tile_entities {
                world.despawn(entity);
            }

            // Respawn single quad
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
            world.insert_resource(ViewerMode::SingleMaterial);
        }
    }
}

fn hud_update_system(
    selected: Res<SelectedTerrain>,
    materials: Res<TerrainMaterials>,
    mode: Res<ViewerMode>,
    hud: Res<HudTextEntity>,
    mut query: Query<&mut engine_ui::widget::Text>,
) {
    if let Ok(mut text) = query.get_mut(hud.0) {
        let new_content = format_hud(&materials.0[selected.0], *mode);
        if text.content != new_content {
            text.content = new_content;
        }
    }
}

fn camera_zoom_system(mouse: Res<MouseState>, mut query: Query<&mut Camera2D>) {
    let scroll = mouse.scroll_delta().y;
    if scroll == 0.0 {
        return;
    }
    if let Ok(mut camera) = query.single_mut() {
        camera.zoom = (camera.zoom + 0.1 * scroll).max(0.1);
    }
}
