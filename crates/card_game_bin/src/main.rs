use axiom2d::prelude::*;
use card_game::prelude::*;
use glam::Vec2;

const TABLE_COLOR: Color = Color {
    r: 0.15,
    g: 0.45,
    b: 0.2,
    a: 1.0,
};

#[allow(clippy::too_many_lines)]
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
    for (i, &pos) in card_positions.iter().enumerate() {
        let card = if i == 2 {
            Card {
                face_texture: TextureId(0),
                back_texture: TextureId(0),
                face_up: true,
            }
        } else {
            Card::face_down(TextureId(0), TextureId(0))
        };
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

    register_game_resources(app);
    spawn_scene(app.world_mut());
    register_preload_hook(app);
    register_game_systems(app, config);
}

fn register_game_resources(app: &mut App) {
    let world = app.world_mut();
    world.insert_resource(PhysicsRes::new(Box::new(RapierBackend::new(Vec2::ZERO))));
    world.insert_resource(CollisionEventBuffer::default());
    world.insert_resource(DragState::default());
    world.insert_resource(CameraDragState::default());
    world.insert_resource(StashVisible::default());
    world.insert_resource(StashGrid::new(10, 10, 3));
    world.insert_resource(Hand::new(10));
    world.insert_resource(StashHoverPreview::default());
    world.insert_resource(ClearColor(Color {
        r: 0.1,
        g: 0.1,
        b: 0.1,
        a: 1.0,
    }));

    let art_shader = register_card_art_shader(&mut world.resource_mut::<ShaderRegistry>());
    world.insert_resource(art_shader);
}

fn register_preload_hook(app: &mut App) {
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
}

#[allow(clippy::too_many_lines)]
fn register_game_systems(app: &mut App, config: WindowConfig) {
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
                stash_boundary_system,
                stash_drag_hover_system,
                card_release_system,
                card_flip_system,
                flip_animation_system,
            )
                .chain(),
        )
        .add_systems(Phase::Update, (camera_drag_system, camera_zoom_system))
        .add_systems(Phase::Update, (stash_toggle_system, stash_tab_click_system))
        .add_systems(Phase::Update, stash_hover_preview_system)
        .add_systems(
            Phase::PostUpdate,
            (
                card_item_form_visibility_system,
                stash_layout_system,
                sort_propagation_system,
                hand_layout_system,
            ),
        )
        .add_systems(
            Phase::PostUpdate,
            (sync_scale_spring_lock_x, scale_spring_system).chain(),
        )
        .add_systems(
            Phase::Render,
            stash_render_system.after(shape_render_system),
        )
        .add_systems(
            Phase::Render,
            (
                stash_tab_render_system,
                stash_hover_preview_render_system,
            )
                .after(stash_render_system),
        );
}

fn main() {
    let mut app = App::new();
    setup(&mut app);
    app.run();
}
