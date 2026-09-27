mod card_data;

#[cfg(feature = "ui-test")]
use bevy_ecs::system::SystemParam;
#[cfg(feature = "ui-test")]
use card_game::booster::opening::BoosterOpenPhase;
#[cfg(feature = "ui-test")]
use std::path::PathBuf;
#[cfg(feature = "ui-test")]
use std::sync::OnceLock;

use axiom2d::prelude::*;
use card_game::card::art::ShapeRepository;
#[cfg(feature = "ui-test")]
use card_game::card::art_selection::select_art_for_signature;
#[cfg(feature = "ui-test")]
use card_game::card::combiner_device::CombinerDevice;
use card_game::card::combiner_device::spawn_combiner_device;
#[cfg(feature = "ui-test")]
use card_game::card::component::{Card, CardLabel, CardZone};
#[cfg(feature = "ui-test")]
use card_game::card::interaction::drag_state::DragState;
#[cfg(feature = "ui-test")]
use card_game::card::jack_cable::{Cable, Jack, WireEndpoints, WrapWire};
#[cfg(feature = "ui-test")]
use card_game::card::jack_socket::{JackSocket, PendingCable};
#[cfg(feature = "ui-test")]
use card_game::card::reader::{CardReader, SignatureSpace};
use card_game::card::reader::{
    READER_COLLISION_FILTER, READER_COLLISION_GROUP, READER_HALF_EXTENTS, spawn_reader,
};
#[cfg(feature = "ui-test")]
use card_game::card::rendering::art_shader::{
    ConditionEffect, DORMANT_WGSL, FOIL_WGSL, INTENSE_WGSL, TierShaders, VariantShaders,
};
#[cfg(feature = "ui-test")]
use card_game::card::screen_device::ScreenSignalShape;
use card_game::card::screen_device::spawn_screen_device;
#[cfg(feature = "ui-test")]
use card_game::card::zone_config::ZoneConfig;
use card_game::prelude::*;
#[cfg(feature = "ui-test")]
use card_game::stash::grid::StashGrid;
#[cfg(feature = "ui-test")]
use card_game::stash::hover::StashHoverPreview;
#[cfg(feature = "ui-test")]
use card_game::stash::toggle::StashVisible;
#[cfg(feature = "ui-test")]
use card_game::terrain::{DualGrid, TerrainId, TerrainMaterial, default_materials};
#[cfg(feature = "ui-test")]
use engine_render::shape::MeshOverlays;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

const TABLE_COLOR: Color = Color {
    r: 0.15,
    g: 0.45,
    b: 0.2,
    a: 1.0,
};

#[cfg(feature = "ui-test")]
const UI_TEST_TERRAIN_SCENARIO: &str = "seeded-card-terrain";
#[cfg(feature = "ui-test")]
const UI_TEST_PLUGIN_WIRING_SCENARIO: &str = "seeded-card-plugin-wiring";
#[cfg(feature = "ui-test")]
const UI_TEST_TERRAIN_TILE_SIZE: f32 = 40.0;
#[cfg(feature = "ui-test")]
const UI_TEST_TERRAIN_CENTER: Vec2 = Vec2::new(-300.0, -250.0);

#[cfg(feature = "ui-test")]
static UI_TEST_STATE_FILE: OnceLock<PathBuf> = OnceLock::new();
#[cfg(feature = "ui-test")]
static UI_TEST_CARD_ENTITY: OnceLock<Entity> = OnceLock::new();
#[cfg(feature = "ui-test")]
static UI_TEST_SECOND_CARD_ENTITY: OnceLock<Entity> = OnceLock::new();
#[cfg(feature = "ui-test")]
static UI_TEST_COMBINER_ENTITY: OnceLock<Entity> = OnceLock::new();
#[cfg(feature = "ui-test")]
static UI_TEST_BOOSTER_ENTITY: OnceLock<Entity> = OnceLock::new();
#[cfg(feature = "ui-test")]
static UI_TEST_BOOSTER_SIGNATURE: OnceLock<CardSignature> = OnceLock::new();
#[cfg(feature = "ui-test")]
static UI_TEST_SCENARIO: OnceLock<String> = OnceLock::new();
#[cfg(feature = "ui-test")]
static UI_TEST_SCREEN_ENTITY: OnceLock<Entity> = OnceLock::new();
#[cfg(feature = "ui-test")]
static UI_TEST_SCREEN_JACK_ENTITY: OnceLock<Entity> = OnceLock::new();

#[cfg(feature = "ui-test")]
#[derive(Component)]
struct UiTestTerrainTile {
    index: usize,
}

#[cfg(feature = "ui-test")]
#[derive(Resource)]
struct UiTestTerrain {
    grid: DualGrid,
    materials: Vec<TerrainMaterial>,
    click_count: u32,
    last_clicked_tile: Option<usize>,
}

#[cfg(feature = "ui-test")]
#[derive(SystemParam)]
struct UiTestCableParams<'w, 's> {
    pending_cable: Res<'w, PendingCable>,
    cables: Query<
        'w,
        's,
        (
            Entity,
            Option<&'static Cable>,
            &'static WireEndpoints,
            &'static WrapWire,
            &'static Shape,
        ),
    >,
    transforms: Query<'w, 's, &'static Transform2D>,
    screen_shapes: Query<
        'w,
        's,
        (
            &'static ScreenSignalShape,
            &'static ChildOf,
            &'static Shape,
            &'static Visible,
        ),
    >,
}

#[cfg(feature = "ui-test")]
#[derive(SystemParam)]
struct UiTestRenderingParams<'w, 's> {
    cards: Query<
        'w,
        's,
        (
            Entity,
            &'static Card,
            &'static CardZone,
            &'static Transform2D,
            Option<&'static MeshOverlays>,
        ),
    >,
    variant_shaders: Res<'w, VariantShaders>,
    tier_shaders: Res<'w, TierShaders>,
    shader_registry: Res<'w, ShaderRegistry>,
    terrain: Option<Res<'w, UiTestTerrain>>,
    tiles: Query<'w, 's, (&'static UiTestTerrainTile, &'static Shape)>,
}

#[cfg(feature = "ui-test")]
fn rendered_centerline(points: &[Vec2]) -> Vec<Vec2> {
    if points.len() < 4 || !points.len().is_multiple_of(2) {
        return Vec::new();
    }

    let half = points.len() / 2;
    (0..half)
        .map(|index| (points[index] + points[points.len() - 1 - index]) * 0.5)
        .collect()
}

#[cfg(feature = "ui-test")]
fn nearest_rendered_point(points: &[Vec2], target: Vec2) -> Vec2 {
    points
        .iter()
        .copied()
        .min_by(|a, b| {
            ((*a - target).length_squared()).total_cmp(&((*b - target).length_squared()))
        })
        .unwrap_or(Vec2::ZERO)
}

#[cfg(feature = "ui-test")]
fn rendered_max_deviation(points: &[Vec2], source: Vec2, dest: Vec2) -> f32 {
    let span = dest - source;
    let span_length = span.length();
    if span_length <= f32::EPSILON {
        return 0.0;
    }

    points
        .iter()
        .map(|point| span.perp_dot(*point - source).abs() / span_length)
        .fold(0.0, f32::max)
}

#[cfg(feature = "ui-test")]
fn format_geometry(points: &[Vec2]) -> String {
    points
        .iter()
        .map(|point| format!("{:.4},{:.4}", point.x, point.y))
        .collect::<Vec<_>>()
        .join(";")
}

#[cfg(feature = "ui-test")]
const SCREEN_SPLINE_GOLDEN_SIGNATURE: &str =
    "0.309394,0.418151,0.459740,-0.068157,0.014729,0.398286,0.121934,-0.879658";

#[cfg(feature = "ui-test")]
const SCREEN_SPLINE_GOLDEN_GEOMETRY: [&str; 4] = [
    "24.6385,20.9075;24.4623,22.6963;23.9406,24.4163;23.0933,26.0014;21.9530,27.3908;20.5636,28.5311;18.9784,29.3784;17.2584,29.9001;15.4697,30.0763;13.6810,29.9001;11.9610,29.3784;10.3758,28.5311;8.9864,27.3908;7.8461,26.0014;6.9989,24.4163;6.4771,22.6963;6.3009,20.9075;6.4771,19.1188;6.9989,17.3988;7.8461,15.8136;8.9864,14.4242;10.3758,13.2840;11.9610,12.4367;13.6810,11.9149;15.4697,11.7388;17.2584,11.9149;18.9784,12.4367;20.5636,13.2840;21.9530,14.4242;23.0933,15.8136;23.9406,17.3988;24.4623,19.1188",
    "32.1558,-3.4078;31.9796,-1.6191;31.4578,0.1009;30.6105,1.6861;29.4703,3.0755;28.0809,4.2157;26.4957,5.0630;24.7757,5.5848;22.9870,5.7609;21.1982,5.5848;19.4782,5.0630;17.8931,4.2157;16.5037,3.0755;15.3634,1.6861;14.5161,0.1009;13.9944,-1.6191;13.8182,-3.4078;13.9944,-5.1966;14.5161,-6.9166;15.3634,-8.5017;16.5037,-9.8911;17.8931,-11.0314;19.4782,-11.8787;21.1982,-12.4004;22.9870,-12.5766;24.7757,-12.4004;26.4957,-11.8787;28.0809,-11.0314;29.4703,-9.8911;30.6105,-8.5017;31.4578,-6.9166;31.9796,-5.1966",
    "9.9052,19.9143;9.7291,21.7031;9.2073,23.4231;8.3600,25.0082;7.2198,26.3976;5.8304,27.5379;4.2452,28.3852;2.5252,28.9069;0.7365,29.0831;-1.0523,28.9069;-2.7723,28.3852;-4.3574,27.5379;-5.7468,26.3976;-6.8871,25.0082;-7.7344,23.4231;-8.2561,21.7031;-8.4323,19.9143;-8.2561,18.1256;-7.7344,16.4056;-6.8871,14.8204;-5.7468,13.4310;-4.3574,12.2908;-2.7723,11.4435;-1.0523,10.9217;0.7365,10.7455;2.5252,10.9217;4.2452,11.4435;5.8304,12.2908;7.2198,13.4310;8.3600,14.8204;9.2073,16.4056;9.7291,18.1256",
    "15.2655,-43.9829;15.0893,-42.1941;14.5675,-40.4741;13.7203,-38.8890;12.5800,-37.4996;11.1906,-36.3593;9.6054,-35.5120;7.8854,-34.9903;6.0967,-34.8141;4.3080,-34.9903;2.5880,-35.5120;1.0028,-36.3593;-0.3866,-37.4996;-1.5269,-38.8890;-2.3742,-40.4741;-2.8959,-42.1941;-3.0721,-43.9829;-2.8959,-45.7716;-2.3742,-47.4916;-1.5269,-49.0768;-0.7692,-50.0000;12.9626,-50.0000;13.7203,-49.0768;14.5675,-47.4916;15.0893,-45.7716",
];

#[cfg(feature = "ui-test")]
static SCREEN_SPLINE_GOLDEN_POINTS: OnceLock<[Vec<Vec2>; 4]> = OnceLock::new();

#[cfg(feature = "ui-test")]
fn screen_spline_golden_points() -> &'static [Vec<Vec2>; 4] {
    SCREEN_SPLINE_GOLDEN_POINTS.get_or_init(|| {
        SCREEN_SPLINE_GOLDEN_GEOMETRY.map(|panel| {
            panel
                .split(';')
                .map(|point| {
                    let (x, y) = point
                        .split_once(',')
                        .expect("screen spline golden point must contain x,y");
                    Vec2::new(
                        x.parse::<f32>()
                            .expect("screen spline golden x must be a float"),
                        y.parse::<f32>()
                            .expect("screen spline golden y must be a float"),
                    )
                })
                .collect()
        })
    })
}

fn hydrate_shape_repository_system(world: &mut World) {
    let mut repo = ShapeRepository::new();
    repo.hydrate_all();
    world.insert_resource(repo);
}

fn warm_up_physics_system(world: &mut World) {
    const WARM_UP_STEPS: u32 = 10;
    const WARM_UP_DT: f32 = 1.0 / 60.0;

    let Some(mut physics) = world.remove_resource::<PhysicsRes>() else {
        return;
    };
    for _ in 0..WARM_UP_STEPS {
        physics.step(Seconds(WARM_UP_DT));
    }
    world.insert_resource(physics);
}

#[cfg(feature = "ui-test")]
fn stabilize_shader_variant_pointer(mut mouse: ResMut<MouseState>) {
    if UI_TEST_SCENARIO
        .get()
        .is_some_and(|scenario| scenario == "seeded-card-shader-variant")
    {
        mouse.set_screen_pos(Vec2::ZERO);
        mouse.set_world_pos(Vec2::ZERO);
    }
}

#[cfg(feature = "ui-test")]
fn terrain_test_color(corners: [TerrainId; 4], materials: &[TerrainMaterial]) -> Color {
    let mut color = [0.0; 3];
    for id in corners {
        let material = materials
            .iter()
            .find(|material| material.id == id)
            .expect("terrain fixture uses a known material");
        for (channel, value) in material.color_a.iter().enumerate() {
            color[channel] += (*value + material.color_b[channel]) * 0.5;
        }
    }
    let scale = 0.25;
    Color::new(color[0] * scale, color[1] * scale, color[2] * scale, 1.0)
}

#[cfg(feature = "ui-test")]
fn terrain_test_grid() -> DualGrid {
    let mut grid = DualGrid::new(4, 3, TerrainId(0));
    grid.set(1, 1, TerrainId(1));
    grid.set(2, 1, TerrainId(2));
    grid.set(1, 2, TerrainId(3));
    grid
}

#[cfg(feature = "ui-test")]
fn spawn_terrain_test_fixture(world: &mut World) {
    if UI_TEST_SCENARIO.get().is_none_or(|scenario| {
        scenario != UI_TEST_TERRAIN_SCENARIO && scenario != UI_TEST_PLUGIN_WIRING_SCENARIO
    }) {
        return;
    }

    let grid = terrain_test_grid();
    let materials = default_materials();
    let visual_tiles = grid.visual_tiles();
    let visual_center = Vec2::new(
        (grid.width() as f32 - 1.0) * 0.5,
        (grid.height() as f32 - 1.0) * 0.5,
    );
    let half_tile = UI_TEST_TERRAIN_TILE_SIZE * 0.5 - 1.0;

    for (index, tile) in visual_tiles.iter().enumerate() {
        let position = UI_TEST_TERRAIN_CENTER
            + Vec2::new(
                (tile.x - visual_center.x) * UI_TEST_TERRAIN_TILE_SIZE,
                (tile.y - visual_center.y) * UI_TEST_TERRAIN_TILE_SIZE,
            );
        world.spawn((
            UiTestTerrainTile { index },
            Transform2D {
                position,
                ..Default::default()
            },
            Shape {
                variant: ShapeVariant::Polygon {
                    points: vec![
                        Vec2::new(-half_tile, -half_tile),
                        Vec2::new(half_tile, -half_tile),
                        Vec2::new(half_tile, half_tile),
                        Vec2::new(-half_tile, half_tile),
                    ],
                },
                color: terrain_test_color(tile.corners, &materials),
            },
            Stroke {
                color: Color::new(0.08, 0.08, 0.08, 1.0),
                width: 1.0,
            },
            RenderLayer::World,
            LocalSortOrder(-100),
        ));
    }

    world.insert_resource(UiTestTerrain {
        grid,
        materials,
        click_count: 0,
        last_clicked_tile: None,
    });
}

#[cfg(feature = "ui-test")]
fn terrain_test_interaction_system(
    mouse: Res<MouseState>,
    terrain: Option<ResMut<UiTestTerrain>>,
    mut tiles: Query<(&UiTestTerrainTile, &Transform2D, &mut Shape)>,
) {
    let Some(mut terrain) = terrain else {
        return;
    };
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let clicked_tile = tiles.iter().find_map(|(tile, transform, _)| {
        let local = mouse.world_pos() - transform.position;
        (local.x.abs() <= UI_TEST_TERRAIN_TILE_SIZE * 0.5
            && local.y.abs() <= UI_TEST_TERRAIN_TILE_SIZE * 0.5)
            .then_some(tile.index)
    });
    let Some(clicked_tile) = clicked_tile else {
        return;
    };

    let visual_width = terrain.grid.width() + 1;
    let visual_x = clicked_tile % visual_width;
    let visual_y = clicked_tile / visual_width;
    let cell_x = visual_x.saturating_sub(1).min(terrain.grid.width() - 1);
    let cell_y = visual_y.min(terrain.grid.height() - 1);
    let next_id = if terrain.grid.get(cell_x, cell_y) == Some(TerrainId(1)) {
        TerrainId(2)
    } else {
        TerrainId(1)
    };
    terrain.grid.set(cell_x, cell_y, next_id);
    terrain.click_count += 1;
    terrain.last_clicked_tile = Some(clicked_tile);

    let visual_tiles = terrain.grid.visual_tiles();
    for (tile, _, mut shape) in &mut tiles {
        shape.color = terrain_test_color(visual_tiles[tile.index].corners, &terrain.materials);
    }
}

#[cfg(feature = "ui-test")]
fn format_terrain_corners(corners: [TerrainId; 4]) -> String {
    corners
        .iter()
        .map(|id| id.0.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn spawn_scene(world: &mut World) {
    world.spawn((
        Transform2D {
            position: Vec2::ZERO,
            ..Default::default()
        },
        Shape {
            variant: ShapeVariant::Polygon {
                points: vec![
                    Vec2::new(-500.0, -375.0),
                    Vec2::new(500.0, -375.0),
                    Vec2::new(500.0, 375.0),
                    Vec2::new(-500.0, 375.0),
                ],
            },
            color: TABLE_COLOR,
        },
        RenderLayer::Background,
        SortOrder::default(),
    ));

    #[cfg(feature = "ui-test")]
    spawn_terrain_test_fixture(world);

    world.spawn(Camera2D {
        position: Vec2::ZERO,
        zoom: 1.0,
    });

    let card_size = Vec2::new(TABLE_CARD_WIDTH, TABLE_CARD_HEIGHT);
    let mut rng = ChaCha8Rng::seed_from_u64(0);
    let deck = card_data::starter_deck(&mut rng);
    let mut card_entities = Vec::new();
    for card in &deck {
        let entity = spawn_visual_card(
            world,
            &card.definition,
            card.position,
            card_size,
            card.face_up,
            card.signature,
        );
        card_entities.push(entity);
    }
    #[cfg(feature = "ui-test")]
    {
        let ui_test_card_index = if UI_TEST_SCENARIO
            .get()
            .is_some_and(|scenario| scenario == "seeded-card-shader-variant")
        {
            4
        } else {
            usize::from(UI_TEST_SCENARIO.get().is_some_and(|scenario| {
                scenario == "seeded-card-identity" || scenario == "seeded-card-art-face"
            }))
        };
        UI_TEST_CARD_ENTITY
            .set(card_entities[ui_test_card_index])
            .expect("UI test card can only be configured once");
        if UI_TEST_SCENARIO.get().is_some_and(|scenario| {
            scenario == "seeded-card-combiner" || scenario == UI_TEST_PLUGIN_WIRING_SCENARIO
        }) {
            UI_TEST_SECOND_CARD_ENTITY
                .set(card_entities[1])
                .expect("UI test second card can only be configured once");
        }
    }

    #[cfg(feature = "ui-test")]
    if UI_TEST_SCENARIO.get().is_some_and(|scenario| {
        scenario == "seeded-booster-opening" || scenario == UI_TEST_PLUGIN_WIRING_SCENARIO
    }) {
        let booster_signature =
            CardSignature::new([0.125, -0.25, 0.375, -0.5, 0.625, -0.75, 0.875, -1.0]);
        let booster_entity =
            spawn_booster_pack(world, Vec2::new(-300.0, -150.0), vec![booster_signature]);
        UI_TEST_BOOSTER_ENTITY
            .set(booster_entity)
            .expect("UI test booster can only be configured once");
        UI_TEST_BOOSTER_SIGNATURE
            .set(booster_signature)
            .expect("UI test booster signature can only be configured once");
    }

    // Spawn a card reader altar with child visual entities.
    let reader_pos = Vec2::new(300.0, 0.0);
    let (reader_entity, _reader_jack) = spawn_reader(world, reader_pos);

    // Spawn a screen device — connect to the reader by dragging a cable interactively.
    let screen_pos = Vec2::new(300.0, 150.0);
    #[cfg(feature = "ui-test")]
    let (screen_entity, screen_jack) = spawn_screen_device(world, screen_pos);
    #[cfg(not(feature = "ui-test"))]
    let (_screen_entity, _screen_jack) = spawn_screen_device(world, screen_pos);
    #[cfg(feature = "ui-test")]
    {
        UI_TEST_SCREEN_ENTITY
            .set(screen_entity)
            .expect("UI test screen can only be configured once");
        UI_TEST_SCREEN_JACK_ENTITY
            .set(screen_jack)
            .expect("UI test screen jack can only be configured once");
    }

    // Spawn a combiner device — wire interactively.
    let combiner_pos = Vec2::new(300.0, -150.0);
    let (combiner_entity, _comb_in_a, _comb_in_b, _comb_out) =
        spawn_combiner_device(world, combiner_pos);

    #[cfg(feature = "ui-test")]
    let second_reader = if UI_TEST_SCENARIO.get().is_some_and(|scenario| {
        scenario == "seeded-card-combiner" || scenario == UI_TEST_PLUGIN_WIRING_SCENARIO
    }) {
        UI_TEST_COMBINER_ENTITY
            .set(combiner_entity)
            .expect("UI test combiner can only be configured once");
        Some((
            spawn_reader(world, Vec2::new(100.0, -150.0)),
            Vec2::new(100.0, -150.0),
        ))
    } else {
        None
    };
    #[cfg(not(feature = "ui-test"))]
    let _ = combiner_entity;

    let mut bus = world.resource_mut::<EventBus<PhysicsCommand>>();
    bus.push(PhysicsCommand::AddBody {
        entity: reader_entity,
        body_type: RigidBody::Kinematic,
        position: reader_pos,
    });
    bus.push(PhysicsCommand::AddCollider {
        entity: reader_entity,
        collider: Collider::Aabb(READER_HALF_EXTENTS),
    });
    bus.push(PhysicsCommand::SetCollisionGroup {
        entity: reader_entity,
        membership: READER_COLLISION_GROUP,
        filter: READER_COLLISION_FILTER,
    });
    #[cfg(feature = "ui-test")]
    if let Some(((second_reader_entity, _), second_reader_pos)) = second_reader {
        bus.push(PhysicsCommand::AddBody {
            entity: second_reader_entity,
            body_type: RigidBody::Kinematic,
            position: second_reader_pos,
        });
        bus.push(PhysicsCommand::AddCollider {
            entity: second_reader_entity,
            collider: Collider::Aabb(READER_HALF_EXTENTS),
        });
        bus.push(PhysicsCommand::SetCollisionGroup {
            entity: second_reader_entity,
            membership: READER_COLLISION_GROUP,
            filter: READER_COLLISION_FILTER,
        });
    }
    for &entity in &card_entities {
        bus.push(PhysicsCommand::SetCollisionGroup {
            entity,
            membership: CARD_COLLISION_GROUP,
            filter: CARD_COLLISION_FILTER,
        });
    }
}

fn setup(app: &mut App) {
    // PhysicsRes must be inserted before DefaultPlugins (which checks for it)
    app.world_mut()
        .insert_resource(PhysicsRes::new(Box::new(RapierBackend::new(Vec2::ZERO))));

    app.add_plugin(DefaultPlugins);
    app.add_plugin(CardGamePlugin);

    #[cfg(feature = "ui-test")]
    {
        let state_file = std::env::var_os("AXIOM_UI_TEST_STATE_FILE").map_or_else(
            || panic!("AXIOM_UI_TEST_STATE_FILE must be set with ui-test"),
            PathBuf::from,
        );
        UI_TEST_STATE_FILE
            .set(state_file)
            .expect("UI test state file can only be configured once");
        UI_TEST_SCENARIO
            .set(
                std::env::var("AXIOM_UI_TEST_SCENARIO")
                    .unwrap_or_else(|_| "seeded-card-drag".to_owned()),
            )
            .expect("UI test scenario can only be configured once");
        app.add_systems(
            Phase::LateUpdate,
            stabilize_shader_variant_pointer
                .after(mouse_world_pos_system)
                .before(shader_pointer_system),
        );
        app.add_systems(Phase::Update, terrain_test_interaction_system);
        app.add_systems(Phase::PostRender, record_ui_test_state);
    }

    let window_config = WindowConfig {
        title: "Card Game",
        width: 1024,
        height: 768,
        ..Default::default()
    };
    #[cfg(feature = "ui-test")]
    let window_config = {
        let mut window_config = window_config;
        if UI_TEST_SCENARIO
            .get()
            .is_some_and(|scenario| scenario == "seeded-card-stash-roundtrip")
        {
            window_config.height = 900;
        }
        window_config
    };
    app.set_window_config(window_config);

    app.world_mut()
        .resource_mut::<PostSplashSetup>()
        .add_systems(spawn_scene);

    {
        let hooks = &mut *app.world_mut().resource_mut::<PreloadHooks>();
        hooks.add_systems((hydrate_shape_repository_system, warm_up_physics_system).chain());
    }
}

#[cfg(feature = "ui-test")]
mod ui_test_state;

#[cfg(feature = "ui-test")]
fn record_ui_test_state(
    drag_state: Res<DragState>,
    hand: Res<Hand>,
    mouse: Res<MouseState>,
    cable_params: UiTestCableParams,
    stash_grid: Res<StashGrid>,
    stash_hover: Res<StashHoverPreview>,
    stash_visible: Res<StashVisible>,
    art_repository: Res<ShapeRepository>,
    booster_opening: Option<Res<BoosterOpening>>,
    card_labels: Query<&CardLabel>,
    booster_packs: Query<(Entity, &BoosterPack, &Transform2D)>,
    readers: Query<&CardReader>,
    reader_jacks: Query<&Jack<SignatureSpace>>,
    combiners: Query<&CombinerDevice>,
    sockets: Query<&JackSocket>,
    rendering: UiTestRenderingParams,
) {
    ui_test_state::collect_and_write(
        drag_state,
        hand,
        mouse,
        cable_params,
        stash_grid,
        stash_hover,
        stash_visible,
        art_repository,
        booster_opening,
        card_labels,
        booster_packs,
        readers,
        reader_jacks,
        combiners,
        sockets,
        rendering,
    );
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing_subscriber::filter::LevelFilter::WARN)
        .init();

    let mut app = App::new();
    setup(&mut app);
    app.run();
}
