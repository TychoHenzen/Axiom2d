#![allow(clippy::unwrap_used)] // viewer binary, not library

use std::fmt::Write as _;

use axiom2d::prelude::*;
use bytemuck::bytes_of;
use engine_ecs::schedule::Phase;
use engine_input::mouse::{MouseButton, MouseState};
use engine_render::camera::Camera2D;
use engine_render::material::Material2d;
use engine_render::shader::ShaderRegistry;
use engine_render::shape::{ColorMesh, ColorVertex, TessellatedColorMesh};
use glam::{Affine2, Vec2};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use terrain::dual_grid::DualGrid;
use terrain::material::{TerrainId, TerrainMaterial, default_materials};
use terrain::shader::register_terrain_shader;
use terrain::wfc::{ConstraintTable, Grid as WfcGrid, collapse};
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
    WfcGrid,
}

/// WFC generation state (seed for deterministic replay).
#[derive(Resource, Debug)]
struct WfcState {
    seed: u64,
}

/// Camera drag state for right-mouse-button panning.
#[derive(Resource, Debug, Default)]
struct CameraDragState {
    anchor: Option<Vec2>,
}

/// Flag requesting a WFC grid generation on next frame.
#[derive(Resource, Debug, Default)]
struct WfcGenerateRequested(bool);

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
        world.insert_resource(WfcState { seed: 42 });
        world.insert_resource(CameraDragState::default());
        world.insert_resource(WfcGenerateRequested::default());
    }

    app.world_mut()
        .resource_mut::<PostSplashSetup>()
        .add_systems(spawn_viewer_scene);

    app.add_systems(
        Phase::Update,
        (terrain_input_system, hud_update_system).chain(),
    );
    app.add_systems(Phase::Update, camera_zoom_system);
    app.add_systems(Phase::Update, camera_drag_system);
    app.add_systems(
        Phase::Update,
        mode_switch_system.after(terrain_input_system),
    );
}

fn build_quad_mesh(half: f32) -> TessellatedColorMesh {
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

fn spawn_viewer_scene(world: &mut World) {
    // Camera
    world.spawn(Camera2D::default());

    let shader = world.resource::<TerrainShader>().0;
    let materials = world.resource::<TerrainMaterials>().clone();
    let selected = world.resource::<SelectedTerrain>().0;

    // Build terrain quad -- large quad centered at origin
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

fn format_hud(mat: &TerrainMaterial, mode: ViewerMode, seed: Option<u64>) -> String {
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

/// Pack a single `TerrainMaterial` into the uniform buffer.
/// Layout: `material[0]` = (packed corners, world x, world y, seed),
/// `material[1..4]` = `MaterialParams`.
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
/// Layout: `material[0]` = (packed corners, world x, world y, seed),
/// `material[1..4]` = primary `MaterialParams`,
/// `material[5..8]` = secondary `MaterialParams`.
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

fn spawn_grid_tiles(
    world: &mut World,
    grid: &DualGrid,
    shader: engine_render::shader::ShaderHandle,
    materials: &[TerrainMaterial],
    tile_size: f32,
) -> Vec<Entity> {
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

fn spawn_dual_grid(
    world: &mut World,
    shader: engine_render::shader::ShaderHandle,
    materials: &[TerrainMaterial],
) -> Vec<Entity> {
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

    spawn_grid_tiles(world, &grid, shader, materials, 120.0)
}

fn default_constraints(materials: &[TerrainMaterial]) -> ConstraintTable {
    let types: Vec<TerrainId> = materials.iter().map(|m| m.id).collect();
    let mut table = ConstraintTable::new(types.clone());
    // Allow all pairs for now
    for &a in &types {
        for &b in &types {
            table.allow(a, b);
        }
    }
    table
}

fn generate_wfc_grid(
    materials: &[TerrainMaterial],
    width: usize,
    height: usize,
    seed: u64,
) -> DualGrid {
    let constraints = default_constraints(materials);
    let mut wfc_grid = WfcGrid::new(width, height);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    match collapse(&mut wfc_grid, &constraints, &mut rng) {
        Ok(()) => {
            let mut dual = DualGrid::new(width, height, TerrainId(0));
            for y in 0..height {
                for x in 0..width {
                    if let Some(id) = wfc_grid.get(x, y) {
                        dual.set(x, y, id);
                    }
                }
            }
            dual
        }
        Err(_) => DualGrid::new(width, height, TerrainId(0)),
    }
}

fn terrain_input_system(
    input: Res<InputState>,
    mut selected: ResMut<SelectedTerrain>,
    mut materials: ResMut<TerrainMaterials>,
    quad: Option<Res<TerrainQuadEntity>>,
    mut mode_switch: ResMut<ModeSwitchRequested>,
    mode: Res<ViewerMode>,
    mut wfc_state: ResMut<WfcState>,
    mut wfc_generate: ResMut<WfcGenerateRequested>,
    mut query: Query<&mut Material2d>,
) {
    // Mode toggle
    if input.just_pressed(KeyCode::Tab) {
        mode_switch.0 = true;
    }

    // G: generate WFC grid (current seed)
    if input.just_pressed(KeyCode::KeyG) {
        wfc_generate.0 = true;
    }
    // N: re-roll seed and generate
    if input.just_pressed(KeyCode::KeyN) {
        wfc_state.seed += 1;
        wfc_generate.0 = true;
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
    if changed
        && *mode == ViewerMode::SingleMaterial
        && let Some(ref quad) = quad
        && let Ok(mut mat2d) = query.get_mut(quad.0)
    {
        mat2d.uniforms = build_single_material_uniform(&materials.0[selected.0]);
    }
}

fn mode_switch_system(world: &mut World) {
    let tab_pressed = world.resource::<ModeSwitchRequested>().0;
    let gen_requested = world.resource::<WfcGenerateRequested>().0;

    if !tab_pressed && !gen_requested {
        return;
    }

    world.resource_mut::<ModeSwitchRequested>().0 = false;
    world.resource_mut::<WfcGenerateRequested>().0 = false;

    let current_mode = *world.resource::<ViewerMode>();
    let shader = world.resource::<TerrainShader>().0;
    let materials = world.resource::<TerrainMaterials>().clone();
    let selected = world.resource::<SelectedTerrain>().0;

    // Determine target mode
    let target = if gen_requested {
        ViewerMode::WfcGrid
    } else {
        // Tab cycles
        match current_mode {
            ViewerMode::SingleMaterial => ViewerMode::DualGrid,
            ViewerMode::DualGrid => ViewerMode::WfcGrid,
            ViewerMode::WfcGrid => ViewerMode::SingleMaterial,
        }
    };

    // Clean up current mode
    match current_mode {
        ViewerMode::SingleMaterial => {
            let quad_entity = world.resource::<TerrainQuadEntity>().0;
            world.despawn(quad_entity);
        }
        ViewerMode::DualGrid | ViewerMode::WfcGrid => {
            let tile_entities = std::mem::take(&mut world.resource_mut::<GridTileEntities>().0);
            for entity in tile_entities {
                world.despawn(entity);
            }
        }
    }

    // Set up target mode
    match target {
        ViewerMode::SingleMaterial => {
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
        }
        ViewerMode::DualGrid => {
            let entities = spawn_dual_grid(world, shader, &materials.0);
            world.insert_resource(GridTileEntities(entities));
        }
        ViewerMode::WfcGrid => {
            let seed = world.resource::<WfcState>().seed;
            let grid = generate_wfc_grid(&materials.0, 20, 15, seed);
            let entities = spawn_grid_tiles(world, &grid, shader, &materials.0, 60.0);
            world.insert_resource(GridTileEntities(entities));
        }
    }

    world.insert_resource(target);
}

fn hud_update_system(
    selected: Res<SelectedTerrain>,
    materials: Res<TerrainMaterials>,
    mode: Res<ViewerMode>,
    wfc_state: Res<WfcState>,
    hud: Option<Res<HudTextEntity>>,
    mut query: Query<&mut engine_ui::widget::Text>,
) {
    let Some(hud) = hud else { return };
    let seed = if *mode == ViewerMode::WfcGrid {
        Some(wfc_state.seed)
    } else {
        None
    };
    if let Ok(mut text) = query.get_mut(hud.0) {
        let new_content = format_hud(&materials.0[selected.0], *mode, seed);
        if text.content != new_content {
            text.content = new_content;
        }
    }
}

fn camera_drag_system(
    mouse: Res<MouseState>,
    mut drag_state: ResMut<CameraDragState>,
    mut query: Query<&mut Camera2D>,
) {
    if mouse.just_released(MouseButton::Right) {
        drag_state.anchor = None;
        return;
    }
    if mouse.just_pressed(MouseButton::Right) {
        drag_state.anchor = Some(mouse.screen_pos());
        return;
    }
    if mouse.pressed(MouseButton::Right)
        && let Some(anchor) = drag_state.anchor
    {
        let delta = mouse.screen_pos() - anchor;
        if let Ok(mut camera) = query.single_mut() {
            let zoom = camera.zoom;
            camera.position -= delta / zoom;
        }
        drag_state.anchor = Some(mouse.screen_pos());
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
