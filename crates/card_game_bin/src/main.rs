mod card_data;

use axiom2d::prelude::*;
use card_game::card::art::ShapeRepository;
use card_game::card::reader::{
    READER_COLLISION_FILTER, READER_COLLISION_GROUP, READER_HALF_EXTENTS, spawn_reader,
};
use card_game::card::screen_device::spawn_screen_device;
use card_game::prelude::*;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

const TABLE_COLOR: Color = Color {
    r: 0.15,
    g: 0.45,
    b: 0.2,
    a: 1.0,
};

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

    // Spawn a card reader altar with child visual entities.
    let reader_pos = Vec2::new(300.0, 0.0);
    let (reader_entity, _reader_jack) = spawn_reader(world, reader_pos);

    // Spawn a screen device — connect to the reader by dragging a cable interactively.
    let screen_pos = Vec2::new(300.0, 150.0);
    let (_screen_entity, _screen_jack) = spawn_screen_device(world, screen_pos);

    let mut physics = world.resource_mut::<PhysicsRes>();
    physics.add_body(reader_entity, &RigidBody::Kinematic, reader_pos);
    physics.add_collider(reader_entity, &Collider::Aabb(READER_HALF_EXTENTS));
    physics
        .set_collision_group(
            reader_entity,
            READER_COLLISION_GROUP,
            READER_COLLISION_FILTER,
        )
        .expect("reader entity should have physics body");
    for &entity in &card_entities {
        physics
            .set_collision_group(entity, CARD_COLLISION_GROUP, CARD_COLLISION_FILTER)
            .expect("card entity should have physics body");
    }
}

fn setup(app: &mut App) {
    // PhysicsRes must be inserted before DefaultPlugins (which checks for it)
    app.world_mut()
        .insert_resource(PhysicsRes::new(Box::new(RapierBackend::new(Vec2::ZERO))));

    app.add_plugin(DefaultPlugins);
    app.add_plugin(CardGamePlugin);

    app.set_window_config(WindowConfig {
        title: "Card Game",
        width: 1024,
        height: 768,
        ..Default::default()
    });

    app.world_mut()
        .resource_mut::<PostSplashSetup>()
        .add(spawn_scene);

    {
        let hooks = &mut *app.world_mut().resource_mut::<PreloadHooks>();
        hooks.add(|world: &mut World| {
            let mut repo = ShapeRepository::new();
            repo.hydrate_all();
            world.insert_resource(repo);
        });
        hooks.add(|world: &mut World| {
            const WARM_UP_STEPS: u32 = 10;
            const WARM_UP_DT: f32 = 1.0 / 60.0;
            let Some(mut physics) = world.remove_resource::<PhysicsRes>() else {
                return;
            };
            for _ in 0..WARM_UP_STEPS {
                physics.step(axiom2d::prelude::Seconds(WARM_UP_DT));
            }
            world.insert_resource(physics);
        });
    }
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing_subscriber::filter::LevelFilter::WARN)
        .init();

    let mut app = App::new();
    setup(&mut app);
    app.run();
}
