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
    ConditionEffect, DORMANT_WGSL, FOIL_WGSL, INTENSE_WGSL, ShaderVariant, TierShaders,
    VariantShaders,
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
        if UI_TEST_SCENARIO
            .get()
            .is_some_and(|scenario| scenario == "seeded-card-combiner")
        {
            UI_TEST_SECOND_CARD_ENTITY
                .set(card_entities[1])
                .expect("UI test second card can only be configured once");
        }
    }

    #[cfg(feature = "ui-test")]
    if UI_TEST_SCENARIO
        .get()
        .is_some_and(|scenario| scenario == "seeded-booster-opening")
    {
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
    let (screen_entity, screen_jack) = spawn_screen_device(world, screen_pos);
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
    let second_reader = if UI_TEST_SCENARIO
        .get()
        .is_some_and(|scenario| scenario == "seeded-card-combiner")
    {
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
    let Some(card_entity) = UI_TEST_CARD_ENTITY.get() else {
        return;
    };
    let cards = &rendering.cards;
    let variant_shaders = &rendering.variant_shaders;
    let tier_shaders = &rendering.tier_shaders;
    let shader_registry = &rendering.shader_registry;
    let Ok((entity, card, zone, transform, overlays)) = cards.get(*card_entity) else {
        return;
    };

    let second_card = UI_TEST_SECOND_CARD_ENTITY
        .get()
        .and_then(|second_entity| cards.get(*second_entity).ok());
    let second_dragging = second_card
        .as_ref()
        .is_some_and(|(second_entity, _, _, _, _)| {
            drag_state
                .dragging
                .as_ref()
                .is_some_and(|drag| drag.entity == *second_entity)
        });
    let second_zone = second_card.as_ref().map_or_else(
        || "none".to_owned(),
        |(_, _, zone, _, _)| format!("{zone:?}"),
    );
    let (second_rendered_x, second_rendered_y) = second_card
        .as_ref()
        .map_or((0.0, 0.0), |(_, _, _, transform, _)| {
            (transform.position.x, transform.position.y)
        });
    let second_reader_loaded = second_card
        .as_ref()
        .is_some_and(|(second_entity, _, _, _, _)| {
            readers
                .iter()
                .any(|reader| reader.loaded == Some(*second_entity))
        });

    let dragging = drag_state
        .dragging
        .is_some_and(|drag| drag.entity == entity);
    let stash_cursor_follow = drag_state
        .dragging
        .is_some_and(|drag| drag.entity == entity && drag.stash_cursor_follow);
    let stash_origin = drag_state
        .dragging
        .and_then(|drag| match drag.origin_zone {
            CardZone::Stash { page, col, row } => Some(format!("{page}:{col}:{row}")),
            _ => None,
        })
        .unwrap_or_else(|| "none".to_owned());
    let stash_slot = match zone {
        CardZone::Stash { page, col, row } => format!("{page}:{col}:{row}"),
        _ => "none".to_owned(),
    };
    let stash_slot_present = stash_grid
        .current_storage_page()
        .is_some_and(|page| stash_grid.get(page, 0, 0).is_some());
    let stash_hovered = stash_hover.hovered_entity == Some(entity);
    let hand_contains = hand.cards().contains(card_entity);
    let holder = match zone {
        CardZone::Hand(index) => format!("Hand({index})"),
        _ => "none".to_owned(),
    };
    let holder_occupied = matches!(zone, CardZone::Hand(_)) && hand_contains;
    let zone_config = ZoneConfig::for_zone(zone);
    let reader = readers.iter().find(|reader| reader.loaded == Some(entity));
    let reader_space = reader
        .and_then(|reader| reader_jacks.get(reader.jack_entity).ok())
        .and_then(|jack| jack.data.as_ref());
    let reader_loaded = reader.is_some();
    let reader_signature = reader_space
        .and_then(|space| space.control_points.first())
        .map_or_else(String::new, |signature| {
            signature
                .axes()
                .iter()
                .map(|axis| format!("{axis:.6}"))
                .collect::<Vec<_>>()
                .join(",")
        });
    let reader_space_contains = reader_space.is_some_and(|space| space.contains(&card.signature));
    let reader_space_source_count = reader_space.map_or(0, |space| space.source_cards.len());
    let reader_space_radius = reader_space.map_or(0.0, |space| space.radius);
    let reader_feedback = if reader_loaded { "lit" } else { "dim" };
    let screen_space = UI_TEST_SCREEN_JACK_ENTITY
        .get()
        .and_then(|screen_jack| reader_jacks.get(*screen_jack).ok())
        .and_then(|jack| jack.data.as_ref());
    let screen_signature = screen_space
        .and_then(|space| space.control_points.first())
        .map_or_else(String::new, |signature| {
            signature
                .axes()
                .iter()
                .map(|axis| format!("{axis:.6}"))
                .collect::<Vec<_>>()
                .join(",")
        });
    let screen_entity = UI_TEST_SCREEN_ENTITY.get().copied();
    let mut screen_expected_geometry = vec![String::new(); 4];
    let mut screen_observed_geometry = vec![String::new(); 4];
    let mut screen_geometry_max_error = -1.0;
    let mut screen_geometry_match = false;
    let screen_spline_scenario = UI_TEST_SCENARIO
        .get()
        .is_some_and(|scenario| scenario == "seeded-card-screen-spline");
    if screen_spline_scenario {
        let expected_geometry = screen_spline_golden_points();
        for (display_index, expected) in expected_geometry.iter().enumerate() {
            screen_expected_geometry[display_index] = format_geometry(expected);
        }
        if let (Some(_space), Some(screen_entity)) = (screen_space, screen_entity) {
            let mut observed_points = Vec::with_capacity(4);
            for (display_index, observed_geometry) in
                screen_observed_geometry.iter_mut().enumerate()
            {
                let observed = cable_params
                    .screen_shapes
                    .iter()
                    .find(|(signal, parent, _, visible)| {
                        signal.display_index == display_index
                            && parent.0 == screen_entity
                            && visible.0
                    })
                    .and_then(|(_, _, shape, _)| match &shape.variant {
                        ShapeVariant::Polygon { points } => Some(points.clone()),
                        _ => None,
                    })
                    .unwrap_or_default();
                *observed_geometry = format_geometry(&observed);
                observed_points.push(observed);
            }
            if expected_geometry
                .iter()
                .zip(&observed_points)
                .all(|(expected, observed)| expected.len() == observed.len())
            {
                screen_geometry_max_error = expected_geometry
                    .iter()
                    .zip(&observed_points)
                    .flat_map(|(expected, observed)| {
                        expected
                            .iter()
                            .zip(observed)
                            .map(|(expected, observed)| (*expected - *observed).length())
                    })
                    .fold(0.0, f32::max);
                screen_geometry_match = screen_signature == SCREEN_SPLINE_GOLDEN_SIGNATURE
                    && screen_geometry_max_error <= 0.01;
            }
        }
    }
    let screen_expected_geometry_0 = &screen_expected_geometry[0];
    let screen_expected_geometry_1 = &screen_expected_geometry[1];
    let screen_expected_geometry_2 = &screen_expected_geometry[2];
    let screen_expected_geometry_3 = &screen_expected_geometry[3];
    let screen_observed_geometry_0 = &screen_observed_geometry[0];
    let screen_observed_geometry_1 = &screen_observed_geometry[1];
    let screen_observed_geometry_2 = &screen_observed_geometry[2];
    let screen_observed_geometry_3 = &screen_observed_geometry[3];
    let combiner = UI_TEST_COMBINER_ENTITY
        .get()
        .and_then(|combiner_entity| combiners.get(*combiner_entity).ok());
    let combiner_input_a_connected = combiner.is_some_and(|device| {
        sockets.get(device.input_a).is_ok_and(|socket| {
            socket.connected_cable.is_some_and(|cable_entity| {
                cable_params
                    .cables
                    .get(cable_entity)
                    .is_ok_and(|(_, cable, _, _, _)| cable.is_some())
            })
        })
    });
    let combiner_input_b_connected = combiner.is_some_and(|device| {
        sockets.get(device.input_b).is_ok_and(|socket| {
            socket.connected_cable.is_some_and(|cable_entity| {
                cable_params
                    .cables
                    .get(cable_entity)
                    .is_ok_and(|(_, cable, _, _, _)| cable.is_some())
            })
        })
    });
    let combiner_input_a_space = combiner
        .and_then(|device| reader_jacks.get(device.input_a).ok())
        .and_then(|jack| jack.data.as_ref());
    let combiner_input_b_space = combiner
        .and_then(|device| reader_jacks.get(device.input_b).ok())
        .and_then(|jack| jack.data.as_ref());
    let combiner_output_space = combiner
        .and_then(|device| reader_jacks.get(device.output).ok())
        .and_then(|jack| jack.data.as_ref());
    let combiner_output_contains_primary =
        combiner_output_space.is_some_and(|space| space.contains(&card.signature));
    let combiner_output_contains_secondary =
        second_card
            .as_ref()
            .is_some_and(|(_, second_card, _, _, _)| {
                combiner_output_space.is_some_and(|space| space.contains(&second_card.signature))
            });
    let combiner_input_a_source_count =
        combiner_input_a_space.map_or(0, |space| space.source_cards.len());
    let combiner_input_b_source_count =
        combiner_input_b_space.map_or(0, |space| space.source_cards.len());
    let combiner_output_source_count =
        combiner_output_space.map_or(0, |space| space.source_cards.len());
    let combiner_output_control_points =
        combiner_output_space.map_or(0, |space| space.control_points.len());
    let combiner_output_radius = combiner_output_space.map_or(0.0, |space| space.radius);
    let mut cable_present = false;
    let mut cable_connected = false;
    let mut cable_anchor_count = 0;
    let mut cable_anchor_0 = Vec2::ZERO;
    let mut cable_source = Vec2::ZERO;
    let mut cable_dest = Vec2::ZERO;
    let mut cable_rendered_min = Vec2::ZERO;
    let mut cable_rendered_max = Vec2::ZERO;
    let mut cable_rendered_vertex_count = 0;
    let mut cable_rendered_source = Vec2::ZERO;
    let mut cable_rendered_anchor = Vec2::ZERO;
    let mut cable_rendered_dest = Vec2::ZERO;
    let mut cable_rendered_max_deviation = 0.0;
    if let Some((_, cable, endpoints, wrap, shape)) = cable_params.cables.iter().next() {
        cable_present = true;
        cable_connected = cable.is_some_and(|cable| {
            sockets.get(cable.source).is_ok() && sockets.get(cable.dest).is_ok()
        });
        cable_anchor_count = wrap.anchors.len();
        cable_anchor_0 = wrap
            .anchors
            .first()
            .map_or(Vec2::ZERO, |anchor| anchor.position);
        cable_source = cable_params
            .transforms
            .get(endpoints.source)
            .map_or(Vec2::ZERO, |transform| transform.position);
        cable_dest = cable_params
            .transforms
            .get(endpoints.dest)
            .map_or(Vec2::ZERO, |transform| transform.position);
        if let ShapeVariant::Polygon { points } = &shape.variant {
            cable_rendered_vertex_count = points.len();
            if let Some(first) = points.first() {
                cable_rendered_min = *first;
                cable_rendered_max = *first;
                for point in points.iter().skip(1) {
                    cable_rendered_min = Vec2::new(
                        cable_rendered_min.x.min(point.x),
                        cable_rendered_min.y.min(point.y),
                    );
                    cable_rendered_max = Vec2::new(
                        cable_rendered_max.x.max(point.x),
                        cable_rendered_max.y.max(point.y),
                    );
                }
            }
            let centerline = rendered_centerline(points);
            cable_rendered_source = centerline.first().copied().unwrap_or(Vec2::ZERO);
            cable_rendered_dest = centerline.last().copied().unwrap_or(Vec2::ZERO);
            cable_rendered_anchor = wrap.anchors.first().map_or(Vec2::ZERO, |anchor| {
                nearest_rendered_point(&centerline, anchor.position)
            });
            cable_rendered_max_deviation =
                rendered_max_deviation(&centerline, cable_rendered_source, cable_rendered_dest);
        }
    }
    let identity_signature = card
        .signature
        .axes()
        .iter()
        .map(|axis| format!("{axis:.6}"))
        .collect::<Vec<_>>()
        .join(",");
    let second_identity_signature =
        second_card
            .as_ref()
            .map_or_else(String::new, |(_, card, _, _, _)| {
                card.signature
                    .axes()
                    .iter()
                    .map(|axis| format!("{axis:.6}"))
                    .collect::<Vec<_>>()
                    .join(",")
            });
    let identity_seed = compute_seed(&card.signature);
    let identity_rarity = format!("{:?}", card.signature.rarity());
    let identity_tier = format!("{:?}", card.signature.card_tier());
    let identity_name = card_labels
        .get(entity)
        .map_or(String::new(), |label| label.name.clone());
    let selected_art = select_art_for_signature(&card.signature, &art_repository);
    let art_signature = selected_art.map_or_else(String::new, |entry| {
        entry
            .signature()
            .axes()
            .iter()
            .map(|axis| format!("{axis:.6}"))
            .collect::<Vec<_>>()
            .join(",")
    });
    let art_element =
        selected_art.map_or_else(String::new, |entry| format!("{:?}", entry.element()));
    let art_aspect = selected_art.map_or_else(String::new, |entry| format!("{:?}", entry.aspect()));
    let art_shape_count = selected_art.map_or(0, |entry| entry.shapes().len());
    let booster_pack = UI_TEST_BOOSTER_ENTITY.get().and_then(|expected| {
        booster_packs
            .iter()
            .find(|(candidate, _, _)| *candidate == *expected)
    });
    let booster_dragging = drag_state.dragging.is_some_and(|drag| {
        UI_TEST_BOOSTER_ENTITY
            .get()
            .is_some_and(|booster_entity| drag.entity == *booster_entity)
    });
    let booster_pack_present = booster_pack.is_some();
    let (booster_rendered_x, booster_rendered_y) = booster_pack
        .map_or((0.0, 0.0), |(_, _, transform)| {
            (transform.position.x, transform.position.y)
        });
    let expected_signature = UI_TEST_BOOSTER_SIGNATURE.get().copied();
    let opened_card = expected_signature.and_then(|signature| {
        cards.iter().find(|(candidate, candidate_card, _, _, _)| {
            *candidate != entity && candidate_card.signature == signature
        })
    });
    let opened_card_present = opened_card.is_some();
    let opened_card_zone = opened_card.as_ref().map_or_else(
        || "none".to_owned(),
        |(_, _, zone, _, _)| format!("{zone:?}"),
    );
    let opened_card_seed = opened_card
        .as_ref()
        .map_or(0, |(_, candidate_card, _, _, _)| {
            compute_seed(&candidate_card.signature)
        });
    let opened_card_face_up = opened_card
        .as_ref()
        .is_some_and(|(_, candidate_card, _, _, _)| candidate_card.face_up);
    let (opened_card_x, opened_card_y) = opened_card
        .as_ref()
        .map_or((0.0, 0.0), |(_, _, _, transform, _)| {
            (transform.position.x, transform.position.y)
        });
    let opening_phase = booster_opening
        .as_ref()
        .map(|opening| match &opening.phase {
            BoosterOpenPhase::MovingToCenter { .. } => "moving_to_center",
            BoosterOpenPhase::Ripping { .. } => "ripping",
            BoosterOpenPhase::LoweringPack { .. } => "lowering_pack",
            BoosterOpenPhase::RevealingCards { .. } => "revealing_cards",
            BoosterOpenPhase::Completing { .. } => "completing",
            BoosterOpenPhase::Done => "done",
        });
    let booster_phase = opening_phase.unwrap_or(if booster_pack_present {
        "sealed"
    } else if opened_card_present {
        "done"
    } else {
        "none"
    });
    let booster_card_count = booster_opening.as_ref().map_or_else(
        || {
            booster_pack.map_or(usize::from(opened_card_present), |(_, pack, _)| {
                pack.cards.len()
            })
        },
        |opening| opening.cards.len(),
    );
    let expected_card_seed = expected_signature.map_or(0, |signature| compute_seed(&signature));
    let shader_variant = ShaderVariant::from_rarity(card.signature.rarity());
    let condition_effect = ConditionEffect::from_tier(card.signature.card_tier());
    let expected_variant_shader = match shader_variant {
        ShaderVariant::None => None,
        ShaderVariant::Embossed => Some(variant_shaders.embossed),
        ShaderVariant::Glow => Some(variant_shaders.glow),
        ShaderVariant::Glossy => Some(variant_shaders.glossy),
        ShaderVariant::Foil => Some(variant_shaders.foil),
    };
    let expected_condition_shader = match condition_effect {
        ConditionEffect::None => None,
        ConditionEffect::Worn => Some(tier_shaders.dormant),
        ConditionEffect::Shiny => Some(tier_shaders.intense),
    };
    let expected_condition_source = match condition_effect {
        ConditionEffect::None => None,
        ConditionEffect::Worn => Some(DORMANT_WGSL),
        ConditionEffect::Shiny => Some(INTENSE_WGSL),
    };
    let (variant_overlay, condition_overlay) = overlays.map_or((None, None), |mesh_overlays| {
        let mut variant_overlay = None;
        let mut condition_overlay = None;
        for entry in &mesh_overlays.0 {
            if variant_overlay.is_none()
                && expected_variant_shader.is_some_and(|expected| entry.material.shader == expected)
            {
                variant_overlay = Some(entry);
            }
            if condition_overlay.is_none()
                && expected_condition_shader
                    .is_some_and(|expected| entry.material.shader == expected)
            {
                condition_overlay = Some(entry);
            }
        }
        (variant_overlay, condition_overlay)
    });
    let variant_shader_source_length = expected_variant_shader
        .and_then(|shader| shader_registry.lookup(shader))
        .map_or(0, str::len);
    let variant_shader_source_matches = expected_variant_shader
        .is_some_and(|shader| shader_registry.lookup(shader) == Some(FOIL_WGSL));
    let variant_overlay_present = variant_overlay.is_some();
    let variant_overlay_visible = variant_overlay.is_some_and(|entry| entry.visible);
    let variant_overlay_handle = variant_overlay.map_or(0, |entry| entry.material.shader.0);
    let variant_overlay_handle_matches = expected_variant_shader.is_none()
        || variant_overlay
            .is_some_and(|entry| Some(entry.material.shader) == expected_variant_shader);
    let variant_overlay_vertex_count = variant_overlay.map_or(0, |entry| entry.mesh.vertices.len());
    let condition_overlay_tier = match condition_effect {
        ConditionEffect::None => "None",
        ConditionEffect::Worn => "Dormant",
        ConditionEffect::Shiny => "Intense",
    };
    let condition_overlay_source_length = expected_condition_shader
        .and_then(|shader| shader_registry.lookup(shader))
        .map_or(0, str::len);
    let condition_overlay_source_matches = expected_condition_shader
        .zip(expected_condition_source)
        .is_some_and(|(shader, source)| shader_registry.lookup(shader) == Some(source));
    let condition_overlay_present = condition_overlay.is_some();
    let condition_overlay_visible = condition_overlay.is_some_and(|entry| entry.visible);
    let condition_overlay_handle = condition_overlay.map_or(0, |entry| entry.material.shader.0);
    let condition_overlay_handle_matches = expected_condition_shader.is_none()
        || condition_overlay
            .is_some_and(|entry| Some(entry.material.shader) == expected_condition_shader);
    let condition_overlay_front_only = condition_overlay.is_some_and(|entry| entry.front_only);
    let condition_overlay_vertex_count =
        condition_overlay.map_or(0, |entry| entry.mesh.vertices.len());
    let overlay_count = overlays.map_or(0, |mesh_overlays| mesh_overlays.0.len());
    let mouse_position = mouse.screen_pos();
    let scenario = UI_TEST_SCENARIO
        .get()
        .expect("UI test scenario must be configured before the app runs");
    let snapshot = format!(
        "scenario={scenario}\ndragging={dragging}\nsecond_dragging={second_dragging}\nsecond_zone={second_zone}\nsecond_rendered_x={second_rendered_x:.1}\nsecond_rendered_y={second_rendered_y:.1}\nsecond_reader_loaded={second_reader_loaded}\nbooster_dragging={booster_dragging}\nzone={zone:?}\nhand_contains={hand_contains}\nhand_count={}\nstash_visible={}\nstash_page={}\nstash_storage_page={}\nstash_slot={stash_slot}\nstash_slot_present={stash_slot_present}\nstash_origin={stash_origin}\nstash_cursor_follow={stash_cursor_follow}\nstash_hovered={stash_hovered}\nreader_loaded={reader_loaded}\nreader_signature={reader_signature}\nreader_space_contains={reader_space_contains}\nreader_space_source_count={reader_space_source_count}\nreader_space_radius={reader_space_radius:.4}\nreader_feedback={reader_feedback}\nscreen_signature={screen_signature}\nscreen_geometry_tolerance=0.01\nscreen_geometry_match={screen_geometry_match}\nscreen_geometry_max_error={screen_geometry_max_error:.6}\nscreen_geometry_panel_count=4\nscreen_expected_geometry_0={screen_expected_geometry_0}\nscreen_expected_geometry_1={screen_expected_geometry_1}\nscreen_expected_geometry_2={screen_expected_geometry_2}\nscreen_expected_geometry_3={screen_expected_geometry_3}\nscreen_observed_geometry_0={screen_observed_geometry_0}\nscreen_observed_geometry_1={screen_observed_geometry_1}\nscreen_observed_geometry_2={screen_observed_geometry_2}\nscreen_observed_geometry_3={screen_observed_geometry_3}\ncombiner_input_a_connected={combiner_input_a_connected}\ncombiner_input_b_connected={combiner_input_b_connected}\ncombiner_input_a_source_count={combiner_input_a_source_count}\ncombiner_input_b_source_count={combiner_input_b_source_count}\ncombiner_output_source_count={combiner_output_source_count}\ncombiner_output_control_points={combiner_output_control_points}\ncombiner_output_radius={combiner_output_radius:.4}\ncombiner_output_contains_primary={combiner_output_contains_primary}\ncombiner_output_contains_secondary={combiner_output_contains_secondary}\nidentity_signature={identity_signature}\nsecond_identity_signature={second_identity_signature}\nidentity_seed={identity_seed}\nidentity_rarity={identity_rarity}\nidentity_tier={identity_tier}\nidentity_name={identity_name}\nart_signature={art_signature}\nart_element={art_element}\nart_aspect={art_aspect}\nart_shape_count={art_shape_count}\nbooster_pack_present={booster_pack_present}\nbooster_phase={booster_phase}\nbooster_card_count={booster_card_count}\nbooster_rendered_x={booster_rendered_x:.1}\nbooster_rendered_y={booster_rendered_y:.1}\nexpected_card_seed={expected_card_seed}\nopened_card_present={opened_card_present}\nopened_card_zone={opened_card_zone}\nopened_card_seed={opened_card_seed}\nopened_card_face_up={opened_card_face_up}\nopened_card_x={opened_card_x:.1}\nopened_card_y={opened_card_y:.1}\nrendered_x={:.1}\nrendered_y={:.1}\nrotation={:.4}\nface_up={}\nmouse_x={:.1}\nmouse_y={:.1}\nleft_pressed={}\nright_pressed={}\nholder={holder}\nholder_occupied={holder_occupied}\nzone_has_physics={}\nzone_render_layer={:?}\nzone_has_item_form={}\n",
        hand.len(),
        stash_visible.0,
        stash_grid.current_page(),
        stash_grid.current_storage_page().map_or(-1, i32::from),
        transform.position.x,
        transform.position.y,
        transform.rotation,
        card.face_up,
        mouse_position.x,
        mouse_position.y,
        mouse.pressed(MouseButton::Left),
        mouse.pressed(MouseButton::Right),
        zone_config.has_physics,
        zone_config.render_layer,
        zone_config.has_item_form
    );
    let snapshot = format!(
        "{snapshot}shader_variant={shader_variant:?}\ncondition_effect={condition_effect:?}\nvariant_shader_source_length={variant_shader_source_length}\nvariant_shader_source_matches={variant_shader_source_matches}\nvariant_overlay_present={variant_overlay_present}\nvariant_overlay_visible={variant_overlay_visible}\nvariant_overlay_handle={variant_overlay_handle}\nvariant_overlay_handle_matches={variant_overlay_handle_matches}\nvariant_overlay_vertex_count={variant_overlay_vertex_count}\ncondition_overlay_tier={condition_overlay_tier}\ncondition_overlay_source_length={condition_overlay_source_length}\ncondition_overlay_source_matches={condition_overlay_source_matches}\ncondition_overlay_present={condition_overlay_present}\ncondition_overlay_visible={condition_overlay_visible}\ncondition_overlay_handle={condition_overlay_handle}\ncondition_overlay_handle_matches={condition_overlay_handle_matches}\ncondition_overlay_front_only={condition_overlay_front_only}\ncondition_overlay_vertex_count={condition_overlay_vertex_count}\noverlay_count={overlay_count}\n"
    );
    let snapshot = format!(
        "{snapshot}pending_cable_dragging={}\ncable_present={cable_present}\ncable_connected={cable_connected}\ncable_anchor_count={cable_anchor_count}\ncable_anchor_0_x={:.1}\ncable_anchor_0_y={:.1}\ncable_source_x={:.1}\ncable_source_y={:.1}\ncable_dest_x={:.1}\ncable_dest_y={:.1}\ncable_rendered_source_x={:.1}\ncable_rendered_source_y={:.1}\ncable_rendered_anchor_x={:.1}\ncable_rendered_anchor_y={:.1}\ncable_rendered_dest_x={:.1}\ncable_rendered_dest_y={:.1}\ncable_rendered_max_deviation={:.1}\ncable_rendered_min_x={:.1}\ncable_rendered_min_y={:.1}\ncable_rendered_max_x={:.1}\ncable_rendered_max_y={:.1}\ncable_rendered_vertex_count={}\n",
        cable_params.pending_cable.free_end.is_some(),
        cable_anchor_0.x,
        cable_anchor_0.y,
        cable_source.x,
        cable_source.y,
        cable_dest.x,
        cable_dest.y,
        cable_rendered_source.x,
        cable_rendered_source.y,
        cable_rendered_anchor.x,
        cable_rendered_anchor.y,
        cable_rendered_dest.x,
        cable_rendered_dest.y,
        cable_rendered_max_deviation,
        cable_rendered_min.x,
        cable_rendered_min.y,
        cable_rendered_max.x,
        cable_rendered_max.y,
        cable_rendered_vertex_count,
    );
    let state_file = UI_TEST_STATE_FILE
        .get()
        .expect("UI test state file must be configured before the app runs");
    std::fs::write(state_file, &snapshot).expect("failed to write UI test state snapshot");
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing_subscriber::filter::LevelFilter::WARN)
        .init();

    let mut app = App::new();
    setup(&mut app);
    app.run();
}
