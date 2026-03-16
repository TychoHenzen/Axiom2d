use axiom2d::prelude::*;
use card_game::prelude::*;
use engine_physics::prelude::*;
use glam::Vec2;

const TABLE_COLOR: Color = Color {
    r: 0.15,
    g: 0.45,
    b: 0.2,
    a: 1.0,
};
const CARD_COLLISION_GROUP: u32 = 0b0001;
const CARD_COLLISION_FILTER: u32 = 0b0010;

fn spawn_scene(world: &mut bevy_ecs::world::World) {
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

    let card_size = Vec2::new(CARD_WIDTH, CARD_HEIGHT);
    let mut card_entities = Vec::new();
    for &pos in &card_positions {
        let card = Card::face_down(TextureId(0), TextureId(0));
        let entity = spawn_visual_card(world, card, pos, card_size);
        card_entities.push(entity);
    }

    // Set collision groups and give initial impulses
    let mut physics = world.resource_mut::<PhysicsRes>();
    for &entity in &card_entities {
        physics.set_collision_group(entity, CARD_COLLISION_GROUP, CARD_COLLISION_FILTER);
    }
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
}

fn setup(app: &mut App) {
    app.add_plugin(DefaultPlugins);

    let config = WindowConfig {
        title: "Card Game",
        width: 1024,
        height: 768,
        ..Default::default()
    };

    let world = app.world_mut();
    world.insert_resource(PhysicsRes::new(Box::new(RapierBackend::new(Vec2::ZERO))));
    world.insert_resource(CollisionEventBuffer::default());
    world.insert_resource(DragState::default());
    world.insert_resource(CameraDragState::default());
    world.insert_resource(StashVisible::default());
    world.insert_resource(StashGrid::new(10, 10, 1));
    world.insert_resource(ClearColor(Color {
        r: 0.1,
        g: 0.1,
        b: 0.1,
        a: 1.0,
    }));

    spawn_scene(world);

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

    app.set_window_config(config)
        .add_systems(
            Phase::PreUpdate,
            (
                physics_step_system,
                physics_sync_system,
                card_damping_system,
            )
                .chain(),
        )
        .add_systems(
            Phase::Update,
            (
                card_pick_system,
                card_drag_system,
                card_release_system,
                card_flip_system,
            )
                .chain(),
        )
        .add_systems(Phase::Update, (camera_drag_system, camera_zoom_system))
        .add_systems(Phase::Update, stash_toggle_system)
        .add_systems(
            Phase::PostUpdate,
            (card_face_visibility_sync_system, sort_propagation_system),
        )
        .add_systems(
            Phase::Render,
            stash_render_system.after(shape_render_system),
        );
}

fn main() {
    let mut app = App::new();
    setup(&mut app);
    app.run();
}
