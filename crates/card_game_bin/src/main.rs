use axiom2d::prelude::*;
use engine_physics::prelude::*;
use glam::Vec2;

const TABLE_COLOR: Color = Color {
    r: 0.15,
    g: 0.45,
    b: 0.2,
    a: 1.0,
};
const CARD_WIDTH: f32 = 60.0;
const CARD_HEIGHT: f32 = 84.0;
const CARD_LINEAR_DAMPING: f32 = 3.0;
const CARD_ANGULAR_DAMPING: f32 = 2.0;

const CARD_COLORS: [Color; 5] = [
    Color {
        r: 0.9,
        g: 0.2,
        b: 0.2,
        a: 1.0,
    },
    Color {
        r: 0.2,
        g: 0.4,
        b: 0.9,
        a: 1.0,
    },
    Color {
        r: 0.9,
        g: 0.8,
        b: 0.1,
        a: 1.0,
    },
    Color {
        r: 0.1,
        g: 0.8,
        b: 0.3,
        a: 1.0,
    },
    Color {
        r: 0.8,
        g: 0.3,
        b: 0.8,
        a: 1.0,
    },
];

fn spawn_card(
    world: &mut bevy_ecs::world::World,
    physics: &mut dyn PhysicsBackend,
    position: Vec2,
    color: Color,
) -> bevy_ecs::prelude::Entity {
    let collider = Collider::Aabb(Vec2::new(CARD_WIDTH / 2.0, CARD_HEIGHT / 2.0));
    let entity = world
        .spawn((
            Transform2D {
                position,
                ..Default::default()
            },
            Shape {
                variant: ShapeVariant::Polygon {
                    points: vec![
                        Vec2::new(-CARD_WIDTH / 2.0, -CARD_HEIGHT / 2.0),
                        Vec2::new(CARD_WIDTH / 2.0, -CARD_HEIGHT / 2.0),
                        Vec2::new(CARD_WIDTH / 2.0, CARD_HEIGHT / 2.0),
                        Vec2::new(-CARD_WIDTH / 2.0, CARD_HEIGHT / 2.0),
                    ],
                },
                color,
            },
            RigidBody::Dynamic,
            collider.clone(),
            RenderLayer::World,
            SortOrder(1),
        ))
        .id();

    physics.add_body(entity, &RigidBody::Dynamic, position);
    physics.add_collider(entity, &collider);
    physics.set_damping(entity, CARD_LINEAR_DAMPING, CARD_ANGULAR_DAMPING);

    entity
}

fn setup(app: &mut App) {
    app.add_plugin(DefaultPlugins);
    app.add_plugin(SplashPlugin);

    let config = WindowConfig {
        title: "Card Game",
        width: 1024,
        height: 768,
        ..Default::default()
    };

    let mut physics = RapierBackend::new(Vec2::ZERO);

    let world = app.world_mut();

    // Table background
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

    // Camera centered at origin
    world.spawn(Camera2D {
        position: Vec2::ZERO,
        zoom: 1.0,
    });

    // Spawn cards in a fan layout
    let card_positions = [
        Vec2::new(-120.0, 0.0),
        Vec2::new(-60.0, 30.0),
        Vec2::new(0.0, 0.0),
        Vec2::new(60.0, -20.0),
        Vec2::new(120.0, 10.0),
    ];

    let mut card_entities = Vec::new();
    for (i, &pos) in card_positions.iter().enumerate() {
        let entity = spawn_card(world, &mut physics, pos, CARD_COLORS[i]);
        card_entities.push(entity);
    }

    // Give a couple of cards an initial push so they slide
    physics.add_force_at_point(
        card_entities[0],
        Vec2::new(5000.0, 2000.0),
        Vec2::new(-120.0, 0.0),
    );
    physics.add_force_at_point(
        card_entities[3],
        Vec2::new(-3000.0, 4000.0),
        Vec2::new(60.0, -20.0),
    );

    world.insert_resource(PhysicsRes::new(Box::new(physics)));
    world.insert_resource(CollisionEventBuffer::default());
    world.insert_resource(ClearColor(Color {
        r: 0.1,
        g: 0.1,
        b: 0.1,
        a: 1.0,
    }));

    // Register preload hooks to warm up physics during splash
    app.world_mut()
        .resource_mut::<PreloadHooks>()
        .add(|world: &mut bevy_ecs::world::World| {
            const WARM_UP_STEPS: u32 = 10;
            const WARM_UP_DT: f32 = 1.0 / 60.0;
            let mut physics = world.remove_resource::<PhysicsRes>().expect("PhysicsRes");
            for _ in 0..WARM_UP_STEPS {
                physics.step(axiom2d::prelude::Seconds(WARM_UP_DT));
            }
            world.insert_resource(physics);
        });

    app.set_window_config(config).add_systems(
        Phase::PreUpdate,
        (physics_step_system, physics_sync_system).chain(),
    );
}

fn main() {
    let mut app = App::new();
    setup(&mut app);
    app.run();
}
