#![allow(clippy::unwrap_used)] // viewer binary, not library

mod render;
mod state;
mod systems;

use axiom2d::prelude::*;
use engine_ecs::schedule::Phase;
use engine_render::shader::ShaderRegistry;
use terrain::material::default_materials;
use terrain::shader::register_terrain_shader;
use terrain::{TerrainMaterials, TerrainShader};

use render::spawn_viewer_scene;
use state::{
    CameraDragState, GridTileEntities, HudTextEntity, ModeSwitchRequested, SelectedTerrain,
    TerrainQuadEntity, ViewerMode, WfcGenerateRequested, WfcState,
};
use systems::{
    camera_drag_system, camera_zoom_system, hud_update_system, mode_switch_system,
    terrain_input_system,
};

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

#[cfg(test)]
mod tests {
    use super::*;
    use engine_ecs::prelude::Schedule;
    use engine_input::mouse::{MouseButton, MouseState};
    use engine_render::camera::Camera2D;
    use engine_render::shader::ShaderHandle;
    use glam::Vec2;
    use terrain::dual_grid::DualGrid;
    use terrain::material::{TerrainId, default_materials};

    use crate::render::{
        build_quad_mesh, build_single_material_uniform, build_tile_uniform, format_hud,
        generate_wfc_grid,
    };

    fn viewer_world() -> World {
        let mut world = World::new();
        world.insert_resource(TerrainShader(ShaderHandle(1)));
        world.insert_resource(ViewerMode::SingleMaterial);
        world.insert_resource(ModeSwitchRequested::default());
        world.insert_resource(TerrainMaterials(default_materials()));
        world.insert_resource(SelectedTerrain(0));
        world.insert_resource(GridTileEntities::default());
        world.insert_resource(WfcState { seed: 42 });
        world.insert_resource(WfcGenerateRequested::default());
        world
    }

    #[test]
    fn setup_registers_viewer_resources() {
        let mut app = App::new();
        setup(&mut app);

        assert_eq!(app.plugin_count(), 2);
        assert_eq!(app.window_config().width, 1024);
        assert!(app.world().get_resource::<TerrainMaterials>().is_some());
        assert!(app.world().get_resource::<TerrainShader>().is_some());
        assert!(app.world().get_resource::<SelectedTerrain>().is_some());
        assert!(app.world().get_resource::<PostSplashSetup>().is_some());
    }

    #[test]
    fn viewer_helpers_pack_and_generate_expected_data() {
        let materials = default_materials();
        let mesh = build_quad_mesh(2.0);
        assert_eq!(mesh.vertices.len(), 4);
        assert_eq!(mesh.indices, vec![0, 1, 2, 0, 2, 3]);

        let single_uniform = build_single_material_uniform(&materials[0]);
        assert_eq!(single_uniform.len(), 256);
        assert_ne!(&single_uniform[..16], &[0; 16]);

        let grid = DualGrid::new(2, 2, TerrainId(0));
        let tile = grid
            .visual_tiles()
            .into_iter()
            .next()
            .expect("dual grid has a visual tile");
        let tile_uniform = build_tile_uniform(&tile, &materials);
        assert_eq!(tile_uniform.len(), 256);

        let generated = generate_wfc_grid(&materials, 3, 2, 7);
        assert_eq!(generated.width(), 3);
        assert_eq!(generated.height(), 2);
        assert_eq!(generated.visual_tiles().len(), 12);

        let hud = format_hud(&materials[0], ViewerMode::WfcGrid, Some(7));
        assert!(hud.contains("[WFC] Grass"));
        assert!(hud.contains("seed: 7"));
    }

    #[test]
    fn mode_switches_replace_scene_entities() {
        let mut world = viewer_world();
        spawn_viewer_scene(&mut world);
        let initial_quad = world.resource::<TerrainQuadEntity>().0;
        assert!(
            world
                .get::<Camera2D>(
                    world
                        .iter_entities()
                        .next()
                        .expect("scene starts with a camera")
                        .id(),
                )
                .is_some()
        );

        world.resource_mut::<ModeSwitchRequested>().0 = true;
        mode_switch_system(&mut world);
        assert_eq!(*world.resource::<ViewerMode>(), ViewerMode::DualGrid);
        assert_eq!(world.resource::<GridTileEntities>().0.len(), 25);
        assert!(world.get_entity(initial_quad).is_err());

        world.resource_mut::<ModeSwitchRequested>().0 = true;
        mode_switch_system(&mut world);
        assert_eq!(*world.resource::<ViewerMode>(), ViewerMode::WfcGrid);
        assert_eq!(world.resource::<GridTileEntities>().0.len(), 336);

        world.resource_mut::<ModeSwitchRequested>().0 = true;
        mode_switch_system(&mut world);
        assert_eq!(*world.resource::<ViewerMode>(), ViewerMode::SingleMaterial);
        assert!(world.get_resource::<TerrainQuadEntity>().is_some());

        world.resource_mut::<WfcGenerateRequested>().0 = true;
        mode_switch_system(&mut world);
        assert_eq!(*world.resource::<ViewerMode>(), ViewerMode::WfcGrid);
    }

    #[test]
    fn input_hud_and_camera_systems_update_state() {
        let mut world = viewer_world();
        spawn_viewer_scene(&mut world);
        world.insert_resource(InputState::default());
        world.insert_resource(MouseState::default());
        world.insert_resource(CameraDragState::default());

        for key in [
            KeyCode::Tab,
            KeyCode::KeyG,
            KeyCode::KeyN,
            KeyCode::ArrowRight,
            KeyCode::Digit2,
            KeyCode::ShiftLeft,
            KeyCode::KeyQ,
            KeyCode::KeyW,
            KeyCode::KeyE,
            KeyCode::KeyR,
            KeyCode::KeyT,
            KeyCode::KeyY,
        ] {
            world.resource_mut::<InputState>().press(key);
        }

        let mut input_schedule = Schedule::default();
        input_schedule.add_systems(terrain_input_system);
        input_schedule.run(&mut world);
        assert_eq!(world.resource::<SelectedTerrain>().0, 1);
        assert!(world.resource::<ModeSwitchRequested>().0);
        assert!(world.resource::<WfcGenerateRequested>().0);
        assert_eq!(world.resource::<WfcState>().seed, 43);

        *world.resource_mut::<ViewerMode>() = ViewerMode::WfcGrid;
        let mut hud_schedule = Schedule::default();
        hud_schedule.add_systems(hud_update_system);
        hud_schedule.run(&mut world);
        let hud_entity = world.resource::<HudTextEntity>().0;
        assert!(
            world
                .get::<engine_ui::widget::Text>(hud_entity)
                .expect("HUD entity exists")
                .content
                .contains("seed: 43")
        );

        let camera = world
            .iter_entities()
            .find(|entity| entity.get::<Camera2D>().is_some())
            .expect("scene camera exists")
            .id();
        let mut camera_schedule = Schedule::default();
        camera_schedule.add_systems((camera_drag_system, camera_zoom_system));

        world
            .resource_mut::<MouseState>()
            .set_screen_pos(Vec2::new(10.0, 10.0));
        world.resource_mut::<MouseState>().press(MouseButton::Right);
        camera_schedule.run(&mut world);
        assert_eq!(
            world.resource::<CameraDragState>().anchor,
            Some(Vec2::new(10.0, 10.0))
        );

        world.resource_mut::<MouseState>().clear_frame_state();
        world
            .resource_mut::<MouseState>()
            .set_screen_pos(Vec2::new(20.0, 30.0));
        camera_schedule.run(&mut world);
        assert_eq!(
            world
                .get::<Camera2D>(camera)
                .expect("scene camera exists")
                .position,
            Vec2::new(-10.0, -20.0)
        );

        world
            .resource_mut::<MouseState>()
            .release(MouseButton::Right);
        camera_schedule.run(&mut world);
        assert_eq!(world.resource::<CameraDragState>().anchor, None);

        world
            .resource_mut::<MouseState>()
            .add_scroll_delta(Vec2::new(0.0, 2.0));
        camera_schedule.run(&mut world);
        assert!(
            (world
                .get::<Camera2D>(camera)
                .expect("scene camera exists")
                .zoom
                - 1.2)
                .abs()
                < f32::EPSILON
        );
    }
}
