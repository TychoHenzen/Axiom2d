mod card_data;

#[cfg(feature = "ui-test")]
use std::path::PathBuf;
#[cfg(feature = "ui-test")]
use std::sync::OnceLock;

use axiom2d::prelude::*;
use card_game::card::art::ShapeRepository;
use card_game::card::combiner_device::spawn_combiner_device;
#[cfg(feature = "ui-test")]
use card_game::card::component::{Card, CardZone};
#[cfg(feature = "ui-test")]
use card_game::card::interaction::drag_state::DragState;
use card_game::card::reader::{
    READER_COLLISION_FILTER, READER_COLLISION_GROUP, READER_HALF_EXTENTS, spawn_reader,
};
use card_game::card::screen_device::spawn_screen_device;
use card_game::prelude::*;
#[cfg(feature = "ui-test")]
use card_game::stash::grid::StashGrid;
#[cfg(feature = "ui-test")]
use card_game::stash::hover::StashHoverPreview;
#[cfg(feature = "ui-test")]
use card_game::stash::toggle::StashVisible;
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
static UI_TEST_SCENARIO: OnceLock<String> = OnceLock::new();

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
    UI_TEST_CARD_ENTITY
        .set(card_entities[0])
        .expect("UI test card can only be configured once");

    // Spawn a card reader altar with child visual entities.
    let reader_pos = Vec2::new(300.0, 0.0);
    let (reader_entity, _reader_jack) = spawn_reader(world, reader_pos);

    // Spawn a screen device — connect to the reader by dragging a cable interactively.
    let screen_pos = Vec2::new(300.0, 150.0);
    let (_screen_entity, _screen_jack) = spawn_screen_device(world, screen_pos);

    // Spawn a combiner device — wire interactively.
    let combiner_pos = Vec2::new(300.0, -150.0);
    let (_combiner_entity, _comb_in_a, _comb_in_b, _comb_out) =
        spawn_combiner_device(world, combiner_pos);

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
    stash_grid: Res<StashGrid>,
    stash_hover: Res<StashHoverPreview>,
    stash_visible: Res<StashVisible>,
    cards: Query<(Entity, &Card, &CardZone, &Transform2D)>,
) {
    let Some(card_entity) = UI_TEST_CARD_ENTITY.get() else {
        return;
    };
    let Ok((entity, card, zone, transform)) = cards.get(*card_entity) else {
        return;
    };

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
    let mouse_position = mouse.screen_pos();
    let scenario = UI_TEST_SCENARIO
        .get()
        .expect("UI test scenario must be configured before the app runs");
    let snapshot = format!(
        "scenario={scenario}\ndragging={dragging}\nzone={zone:?}\nhand_contains={hand_contains}\nhand_count={}\nstash_visible={}\nstash_page={}\nstash_storage_page={}\nstash_slot={stash_slot}\nstash_slot_present={stash_slot_present}\nstash_origin={stash_origin}\nstash_cursor_follow={stash_cursor_follow}\nstash_hovered={stash_hovered}\nrendered_x={:.1}\nrendered_y={:.1}\nrotation={:.4}\nface_up={}\nmouse_x={:.1}\nmouse_y={:.1}\nleft_pressed={}\nright_pressed={}\n",
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
        mouse.pressed(MouseButton::Right)
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
