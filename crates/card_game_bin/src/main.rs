mod card_data;

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
use card_game::card::jack_cable::{Cable, Jack};
#[cfg(feature = "ui-test")]
use card_game::card::jack_socket::JackSocket;
#[cfg(feature = "ui-test")]
use card_game::card::reader::{CardReader, SignatureSpace};
use card_game::card::reader::{
    READER_COLLISION_FILTER, READER_COLLISION_GROUP, READER_HALF_EXTENTS, spawn_reader,
};
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
    {
        let ui_test_card_index = usize::from(UI_TEST_SCENARIO.get().is_some_and(|scenario| {
            scenario == "seeded-card-identity" || scenario == "seeded-card-art-face"
        }));
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
    let (_screen_entity, _screen_jack) = spawn_screen_device(world, screen_pos);

    // Spawn a combiner device — wire interactively.
    let combiner_pos = Vec2::new(300.0, -150.0);
    let (_combiner_entity, _comb_in_a, _comb_in_b, _comb_out) =
        spawn_combiner_device(world, combiner_pos);

    #[cfg(feature = "ui-test")]
    let second_reader = if UI_TEST_SCENARIO
        .get()
        .is_some_and(|scenario| scenario == "seeded-card-combiner")
    {
        UI_TEST_COMBINER_ENTITY
            .set(_combiner_entity)
            .expect("UI test combiner can only be configured once");
        Some((
            spawn_reader(world, Vec2::new(100.0, -150.0)),
            Vec2::new(100.0, -150.0),
        ))
    } else {
        None
    };

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
    art_repository: Res<ShapeRepository>,
    booster_opening: Option<Res<BoosterOpening>>,
    cards: Query<(Entity, &Card, &CardZone, &Transform2D)>,
    card_labels: Query<&CardLabel>,
    booster_packs: Query<(Entity, &BoosterPack, &Transform2D)>,
    readers: Query<&CardReader>,
    reader_jacks: Query<&Jack<SignatureSpace>>,
    combiners: Query<&CombinerDevice>,
    sockets: Query<&JackSocket>,
    cables: Query<&Cable>,
) {
    let Some(card_entity) = UI_TEST_CARD_ENTITY.get() else {
        return;
    };
    let Ok((entity, card, zone, transform)) = cards.get(*card_entity) else {
        return;
    };

    let second_card = UI_TEST_SECOND_CARD_ENTITY
        .get()
        .and_then(|second_entity| cards.get(*second_entity).ok());
    let second_dragging = second_card
        .as_ref()
        .is_some_and(|(second_entity, _, _, _)| {
            drag_state
                .dragging
                .as_ref()
                .is_some_and(|drag| drag.entity == *second_entity)
        });
    let second_zone = second_card
        .as_ref()
        .map_or_else(|| "none".to_owned(), |(_, _, zone, _)| format!("{zone:?}"));
    let (second_rendered_x, second_rendered_y) = second_card
        .as_ref()
        .map_or((0.0, 0.0), |(_, _, _, transform)| {
            (transform.position.x, transform.position.y)
        });
    let second_reader_loaded = second_card
        .as_ref()
        .is_some_and(|(second_entity, _, _, _)| {
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
    let combiner = UI_TEST_COMBINER_ENTITY
        .get()
        .and_then(|combiner_entity| combiners.get(*combiner_entity).ok());
    let combiner_input_a_connected = combiner.is_some_and(|device| {
        sockets.get(device.input_a).is_ok_and(|socket| {
            socket
                .connected_cable
                .is_some_and(|cable_entity| cables.get(cable_entity).is_ok())
        })
    });
    let combiner_input_b_connected = combiner.is_some_and(|device| {
        sockets.get(device.input_b).is_ok_and(|socket| {
            socket
                .connected_cable
                .is_some_and(|cable_entity| cables.get(cable_entity).is_ok())
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
        second_card.as_ref().is_some_and(|(_, second_card, _, _)| {
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
            .map_or_else(String::new, |(_, card, _, _)| {
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
        cards.iter().find(|(candidate, candidate_card, _, _)| {
            *candidate != entity && candidate_card.signature == signature
        })
    });
    let opened_card_present = opened_card.is_some();
    let opened_card_zone = opened_card
        .as_ref()
        .map_or_else(|| "none".to_owned(), |(_, _, zone, _)| format!("{zone:?}"));
    let opened_card_seed = opened_card.as_ref().map_or(0, |(_, candidate_card, _, _)| {
        compute_seed(&candidate_card.signature)
    });
    let opened_card_face_up = opened_card
        .as_ref()
        .is_some_and(|(_, candidate_card, _, _)| candidate_card.face_up);
    let (opened_card_x, opened_card_y) = opened_card
        .as_ref()
        .map_or((0.0, 0.0), |(_, _, _, transform)| {
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
    let mouse_position = mouse.screen_pos();
    let scenario = UI_TEST_SCENARIO
        .get()
        .expect("UI test scenario must be configured before the app runs");
    let snapshot = format!(
        "scenario={scenario}\ndragging={dragging}\nsecond_dragging={second_dragging}\nsecond_zone={second_zone}\nsecond_rendered_x={second_rendered_x:.1}\nsecond_rendered_y={second_rendered_y:.1}\nsecond_reader_loaded={second_reader_loaded}\nbooster_dragging={booster_dragging}\nzone={zone:?}\nhand_contains={hand_contains}\nhand_count={}\nstash_visible={}\nstash_page={}\nstash_storage_page={}\nstash_slot={stash_slot}\nstash_slot_present={stash_slot_present}\nstash_origin={stash_origin}\nstash_cursor_follow={stash_cursor_follow}\nstash_hovered={stash_hovered}\nreader_loaded={reader_loaded}\nreader_signature={reader_signature}\nreader_space_contains={reader_space_contains}\nreader_space_source_count={reader_space_source_count}\nreader_space_radius={reader_space_radius:.4}\nreader_feedback={reader_feedback}\ncombiner_input_a_connected={combiner_input_a_connected}\ncombiner_input_b_connected={combiner_input_b_connected}\ncombiner_input_a_source_count={combiner_input_a_source_count}\ncombiner_input_b_source_count={combiner_input_b_source_count}\ncombiner_output_source_count={combiner_output_source_count}\ncombiner_output_control_points={combiner_output_control_points}\ncombiner_output_radius={combiner_output_radius:.4}\ncombiner_output_contains_primary={combiner_output_contains_primary}\ncombiner_output_contains_secondary={combiner_output_contains_secondary}\nidentity_signature={identity_signature}\nsecond_identity_signature={second_identity_signature}\nidentity_seed={identity_seed}\nidentity_rarity={identity_rarity}\nidentity_tier={identity_tier}\nidentity_name={identity_name}\nart_signature={art_signature}\nart_element={art_element}\nart_aspect={art_aspect}\nart_shape_count={art_shape_count}\nbooster_pack_present={booster_pack_present}\nbooster_phase={booster_phase}\nbooster_card_count={booster_card_count}\nbooster_rendered_x={booster_rendered_x:.1}\nbooster_rendered_y={booster_rendered_y:.1}\nexpected_card_seed={expected_card_seed}\nopened_card_present={opened_card_present}\nopened_card_zone={opened_card_zone}\nopened_card_seed={opened_card_seed}\nopened_card_face_up={opened_card_face_up}\nopened_card_x={opened_card_x:.1}\nopened_card_y={opened_card_y:.1}\nrendered_x={:.1}\nrendered_y={:.1}\nrotation={:.4}\nface_up={}\nmouse_x={:.1}\nmouse_y={:.1}\nleft_pressed={}\nright_pressed={}\nholder={holder}\nholder_occupied={holder_occupied}\nzone_has_physics={}\nzone_render_layer={:?}\nzone_has_item_form={}\n",
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
