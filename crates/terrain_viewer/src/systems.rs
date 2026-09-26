use axiom2d::prelude::{
    GlobalTransform2D, InputState, KeyCode, Query, RenderLayer, Res, ResMut, SortOrder,
    Transform2D, World,
};
use engine_input::mouse::{MouseButton, MouseState};
use engine_render::camera::Camera2D;
use engine_render::material::Material2d;
use engine_render::shader::ShaderHandle;
use engine_render::shape::ColorMesh;
use glam::{Affine2, Vec2};
use terrain::material::TerrainMaterial;
use terrain::{TerrainMaterials, TerrainShader};

use crate::render::{
    build_quad_mesh, build_single_material_uniform, format_hud, generate_wfc_grid, spawn_dual_grid,
    spawn_grid_tiles,
};
use crate::state::{
    CameraDragState, GridTileEntities, HudTextEntity, ModeSwitchRequested, SelectedTerrain,
    TerrainQuadEntity, ViewerMode, WfcGenerateRequested, WfcState,
};

pub(crate) fn terrain_input_system(
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
    handle_mode_requests(&input, &mut mode_switch, &mut wfc_generate, &mut wfc_state);
    let changed = handle_terrain_selection(&input, &mut selected, materials.0.len());
    let changed = handle_parameter_adjustment(&input, &mut materials.0[selected.0]) || changed;
    if changed && *mode == ViewerMode::SingleMaterial {
        update_single_material_quad(quad, &mut query, &materials.0[selected.0]);
    }
}

fn handle_mode_requests(
    input: &InputState,
    mode_switch: &mut ModeSwitchRequested,
    wfc_generate: &mut WfcGenerateRequested,
    wfc_state: &mut WfcState,
) {
    if input.just_pressed(KeyCode::Tab) {
        mode_switch.0 = true;
    }
    if input.just_pressed(KeyCode::KeyG) {
        wfc_generate.0 = true;
    }
    if input.just_pressed(KeyCode::KeyN) {
        wfc_state.seed += 1;
        wfc_generate.0 = true;
    }
}

fn handle_terrain_selection(
    input: &InputState,
    selected: &mut SelectedTerrain,
    count: usize,
) -> bool {
    let mut changed = false;
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
    changed
}

#[allow(clippy::neg_multiply)]
fn handle_parameter_adjustment(input: &InputState, mat: &mut TerrainMaterial) -> bool {
    let step = if input.pressed(KeyCode::ShiftLeft) {
        0.5
    } else {
        0.1
    };
    let mut changed = false;

    let mut adjust = |index: usize, delta: f32, min: f32, _max: f32| {
        let new = (mat.params[index] + delta).max(min);
        if new != mat.params[index] {
            mat.params[index] = new;
            changed = true;
        }
    };
    macro_rules! mul {
        ($key:expr, $idx:expr, $scale:expr, $min:expr, $max:expr) => {
            if input.just_pressed($key) {
                adjust($idx, step * $scale, $min, $max);
            }
        };
    }
    mul!(KeyCode::KeyQ, 0, -1.0, 0.1, 100.0);
    mul!(KeyCode::KeyW, 0, 1.0, 0.1, 100.0);
    mul!(KeyCode::KeyE, 1, -0.5, 0.0, 10.0);
    mul!(KeyCode::KeyR, 1, 0.5, 0.0, 10.0);
    mul!(KeyCode::KeyT, 2, -0.5, 0.0, 10.0);
    mul!(KeyCode::KeyY, 2, 0.5, 0.0, 10.0);
    changed
}

fn update_single_material_quad(
    quad: Option<Res<TerrainQuadEntity>>,
    query: &mut Query<&mut Material2d>,
    mat: &TerrainMaterial,
) {
    if let Some(ref quad) = quad
        && let Ok(mut mat2d) = query.get_mut(quad.0)
    {
        mat2d.uniforms = build_single_material_uniform(mat);
    }
}

pub(crate) fn mode_switch_system(world: &mut World) {
    let tab_pressed = world.resource::<ModeSwitchRequested>().0;
    let gen_requested = world.resource::<WfcGenerateRequested>().0;

    if !tab_pressed && !gen_requested {
        return;
    }

    world.resource_mut::<ModeSwitchRequested>().0 = false;
    world.resource_mut::<WfcGenerateRequested>().0 = false;

    let selected = world.resource::<SelectedTerrain>().0;
    let shader = world.resource::<TerrainShader>().0;
    let materials = world.resource::<TerrainMaterials>().clone();
    let current_mode = *world.resource::<ViewerMode>();

    let target = if gen_requested {
        ViewerMode::WfcGrid
    } else {
        next_mode(current_mode)
    };

    cleanup_current_mode(world, current_mode);
    spawn_target_mode(world, target, shader, &materials.0, selected);
    world.insert_resource(target);
}

fn next_mode(current: ViewerMode) -> ViewerMode {
    match current {
        ViewerMode::SingleMaterial => ViewerMode::DualGrid,
        ViewerMode::DualGrid => ViewerMode::WfcGrid,
        ViewerMode::WfcGrid => ViewerMode::SingleMaterial,
    }
}

fn cleanup_current_mode(world: &mut World, mode: ViewerMode) {
    match mode {
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
}

fn spawn_target_mode(
    world: &mut World,
    target: ViewerMode,
    shader: ShaderHandle,
    materials: &[TerrainMaterial],
    selected: usize,
) {
    match target {
        ViewerMode::SingleMaterial => {
            let mesh = build_quad_mesh(350.0);
            let uniforms = build_single_material_uniform(&materials[selected]);
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
            let entities = spawn_dual_grid(world, shader, materials);
            world.insert_resource(GridTileEntities(entities));
        }
        ViewerMode::WfcGrid => {
            let seed = world.resource::<WfcState>().seed;
            let grid = generate_wfc_grid(materials, 20, 15, seed);
            let entities = spawn_grid_tiles(world, &grid, shader, materials, 60.0);
            world.insert_resource(GridTileEntities(entities));
        }
    }
}

pub(crate) fn hud_update_system(
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

pub(crate) fn camera_drag_system(
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

pub(crate) fn camera_zoom_system(mouse: Res<MouseState>, mut query: Query<&mut Camera2D>) {
    let scroll = mouse.scroll_delta().y;
    if scroll == 0.0 {
        return;
    }
    if let Ok(mut camera) = query.single_mut() {
        camera.zoom = (camera.zoom + 0.1 * scroll).max(0.1);
    }
}
