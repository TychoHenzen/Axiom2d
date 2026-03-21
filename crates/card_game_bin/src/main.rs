mod card_data;

use axiom2d::prelude::*;
use card_game::prelude::*;

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
        SortOrder(0),
    ));

    world.spawn(Camera2D {
        position: Vec2::ZERO,
        zoom: 1.0,
    });

    let card_size = Vec2::new(TABLE_CARD_WIDTH, TABLE_CARD_HEIGHT);
    let deck = card_data::starter_deck();
    let mut card_entities = Vec::new();
    for card in &deck {
        let entity = spawn_visual_card(
            world,
            &card.definition,
            card.position,
            card_size,
            card.face_up,
        );
        card_entities.push(entity);
    }

    let mut physics = world.resource_mut::<PhysicsRes>();
    for &entity in &card_entities {
        physics
            .set_collision_group(entity, CARD_COLLISION_GROUP, CARD_COLLISION_FILTER)
            .expect("card entity should have physics body");
    }
    physics
        .add_force_at_point(
            card_entities[0],
            Vec2::new(5000.0, 2000.0),
            Vec2::new(-120.0, 0.0),
        )
        .expect("card entity should have physics body");
    physics
        .add_force_at_point(
            card_entities[3],
            Vec2::new(-3000.0, 4000.0),
            Vec2::new(60.0, -20.0),
        )
        .expect("card entity should have physics body");
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

    app.world_mut()
        .resource_mut::<PreloadHooks>()
        .add(|world: &mut World| {
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

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();

    let mut app = App::new();
    setup(&mut app);
    app.run();
}
