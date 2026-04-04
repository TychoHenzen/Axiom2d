#![allow(clippy::unwrap_used, clippy::struct_excessive_bools, dead_code)]

use std::sync::{Arc, Mutex};

use bevy_ecs::prelude::*;
use engine_core::prelude::{EventBus, Transform2D};
use engine_input::prelude::{MouseButton, MouseState};
use engine_physics::prelude::{Collider, PhysicsCommand, PhysicsRes, RigidBody};
use engine_render::prelude::RendererRes;
use engine_render::testing::SpyRenderer;
use engine_scene::prelude::RenderLayer;
use glam::Vec2;

use card_game::card::component::CardItemForm;
use card_game::card::component::CardZone;
use card_game::card::interaction::apply::interaction_apply_system;
use card_game::card::interaction::drag_state::{DragInfo, DragState};
use card_game::card::interaction::intent::InteractionIntent;
use card_game::card::interaction::release::card_release_system;
use card_game::hand::cards::Hand;
use card_game::stash::grid::StashGrid;
use card_game::test_helpers::SpyPhysicsBackend;

fn run_system(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(card_release_system);
    schedule.run(world);
}

fn run_release_and_apply(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems((card_release_system, interaction_apply_system).chain());
    schedule.run(world);
}

// ── Builder ──────────────────────────────────────────────────────

struct ReleaseTestBuilder {
    viewport_h: u32,
    screen_pos: Vec2,
    stash_visible: bool,
    origin_zone: CardZone,
    card_zone: CardZone,
    card_position: Vec2,
    card_rotation: f32,
    card_scale: Vec2,
    origin_position: Vec2,
    face_up: bool,
    has_rigid_body: bool,
    has_render_layer: bool,
    has_item_form: bool,
    hand_capacity: usize,
    pre_fill_hand: usize,
}

impl ReleaseTestBuilder {
    fn card_on_table() -> Self {
        Self {
            viewport_h: 600,
            screen_pos: Vec2::new(400.0, 100.0),
            stash_visible: false,
            origin_zone: CardZone::Table,
            card_zone: CardZone::Table,
            card_position: Vec2::ZERO,
            card_rotation: 0.0,
            card_scale: Vec2::ONE,
            origin_position: Vec2::ZERO,
            face_up: false,
            has_rigid_body: true,
            has_render_layer: true,
            has_item_form: false,
            hand_capacity: 10,
            pre_fill_hand: 0,
        }
    }

    fn card_in_hand(index: usize) -> Self {
        Self {
            viewport_h: 600,
            screen_pos: Vec2::new(400.0, 100.0),
            stash_visible: false,
            origin_zone: CardZone::Hand(index),
            card_zone: CardZone::Hand(index),
            card_position: Vec2::ZERO,
            card_rotation: 0.0,
            card_scale: Vec2::ONE,
            origin_position: Vec2::ZERO,
            face_up: false,
            has_rigid_body: true,
            has_render_layer: true,
            has_item_form: false,
            hand_capacity: 10,
            pre_fill_hand: 0,
        }
    }

    fn card_in_stash(page: u8, col: u8, row: u8) -> Self {
        Self {
            viewport_h: 600,
            screen_pos: Vec2::new(600.0, 200.0),
            stash_visible: true,
            origin_zone: CardZone::Stash { page, col, row },
            card_zone: CardZone::Stash { page, col, row },
            card_position: Vec2::ZERO,
            card_rotation: 0.0,
            card_scale: Vec2::ONE,
            origin_position: Vec2::ZERO,
            face_up: false,
            has_rigid_body: true,
            has_render_layer: true,
            has_item_form: false,
            hand_capacity: 10,
            pre_fill_hand: 0,
        }
    }

    fn screen_pos(mut self, x: f32, y: f32) -> Self {
        self.screen_pos = Vec2::new(x, y);
        self
    }

    fn viewport_height(mut self, h: u32) -> Self {
        self.viewport_h = h;
        self
    }

    fn stash_visible(mut self) -> Self {
        self.stash_visible = true;
        self
    }

    fn face_up(mut self) -> Self {
        self.face_up = true;
        self
    }

    fn card_position(mut self, pos: Vec2) -> Self {
        self.card_position = pos;
        self
    }

    fn card_rotation(mut self, r: f32) -> Self {
        self.card_rotation = r;
        self
    }

    fn card_scale(mut self, s: Vec2) -> Self {
        self.card_scale = s;
        self
    }

    fn origin_position(mut self, pos: Vec2) -> Self {
        self.origin_position = pos;
        self
    }

    fn with_item_form(mut self) -> Self {
        self.has_item_form = true;
        self
    }

    fn hand_capacity(mut self, cap: usize) -> Self {
        self.hand_capacity = cap;
        self
    }

    fn pre_fill_hand(mut self, n: usize) -> Self {
        self.pre_fill_hand = n;
        self
    }

    fn build(self) -> (World, Entity) {
        let mut world = World::new();
        world.insert_resource(PhysicsRes::new(Box::new(SpyPhysicsBackend::new())));
        world.insert_resource(EventBus::<PhysicsCommand>::default());
        world.insert_resource(EventBus::<InteractionIntent>::default());

        let mut hand = Hand::new(self.hand_capacity);
        for _ in 0..self.pre_fill_hand {
            let filler = world.spawn_empty().id();
            hand.add(filler).unwrap();
        }
        world.insert_resource(hand);

        let log = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log).with_viewport(800, self.viewport_h);
        world.insert_resource(RendererRes::new(Box::new(spy)));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.release(MouseButton::Left);
        mouse.set_screen_pos(self.screen_pos);
        world.insert_resource(mouse);

        world.insert_resource(StashGrid::new(10, 10, 1));
        world.insert_resource(card_game::stash::toggle::StashVisible(self.stash_visible));

        let mut card = card_game::test_helpers::make_test_card();
        card.face_up = self.face_up;

        let transform = Transform2D {
            position: self.card_position,
            rotation: self.card_rotation,
            scale: self.card_scale,
        };
        let collider = Collider::Aabb(Vec2::new(30.0, 45.0));

        let mut entity_commands = world.spawn((card, self.card_zone, transform, collider));
        if self.has_rigid_body {
            entity_commands.insert(RigidBody::Dynamic);
        }
        if self.has_render_layer {
            entity_commands.insert(RenderLayer::World);
        }
        if self.has_item_form {
            entity_commands.insert(CardItemForm);
        }
        let entity = entity_commands.id();

        world.insert_resource(DragState {
            dragging: Some(DragInfo {
                entity,
                local_grab_offset: Vec2::ZERO,
                origin_zone: self.origin_zone,
                stash_cursor_follow: false,
                origin_position: self.origin_position,
            }),
        });

        (world, entity)
    }
}

// ── Tests ────────────────────────────────────────────────────────

#[test]
fn when_mouse_released_while_not_dragging_then_no_panic_and_stays_none() {
    // Arrange
    let (mut world, _) = ReleaseTestBuilder::card_on_table().build();
    world.insert_resource(DragState::default());

    // Act
    run_system(&mut world);

    // Assert
    assert!(world.resource::<DragState>().dragging.is_none());
}

#[test]
fn when_mouse_not_released_then_drag_state_not_cleared() {
    // Arrange — mouse is pressed but NOT released, so system should skip
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    world.insert_resource(DragState {
        dragging: Some(DragInfo {
            entity,
            local_grab_offset: Vec2::ZERO,
            origin_zone: CardZone::Table,
            stash_cursor_follow: false,
            origin_position: Vec2::ZERO,
        }),
    });
    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Left);
    world.insert_resource(mouse);
    world.insert_resource(Hand::new(10));
    world.insert_resource(PhysicsRes::new(Box::new(
        engine_physics::prelude::NullPhysicsBackend::default(),
    )));
    world.insert_resource(EventBus::<PhysicsCommand>::default());
    world.insert_resource(EventBus::<InteractionIntent>::default());
    let log = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(log).with_viewport(800, 600);
    world.insert_resource(RendererRes::new(Box::new(spy)));
    world.insert_resource(StashGrid::new(10, 10, 1));
    world.insert_resource(card_game::stash::toggle::StashVisible(false));

    // Act
    run_system(&mut world);

    // Assert
    assert!(world.resource::<DragState>().dragging.is_some());
}

/// @doc: Table-to-table release preserves zone—player can drop and re-arrange cards on play surface
#[test]
fn when_card_released_on_table_then_zone_unchanged() {
    // Arrange
    let (mut world, entity) = ReleaseTestBuilder::card_on_table().build();

    // Act
    run_release_and_apply(&mut world);

    // Assert
    assert_eq!(*world.get::<CardZone>(entity).unwrap(), CardZone::Table);
}

#[test]
fn when_face_up_card_released_into_hand_then_no_flip_animation() {
    use card_game::card::interaction::flip_animation::FlipAnimation;

    // Arrange
    let (mut world, entity) = ReleaseTestBuilder::card_on_table()
        .screen_pos(400.0, 550.0)
        .face_up()
        .build();

    // Act
    run_release_and_apply(&mut world);

    // Assert
    assert!(
        world.get::<FlipAnimation>(entity).is_none(),
        "expected no FlipAnimation for face-up card"
    );
}

#[test]
fn when_face_down_card_released_on_table_then_no_flip_animation() {
    use card_game::card::interaction::flip_animation::FlipAnimation;

    // Arrange
    let (mut world, entity) = ReleaseTestBuilder::card_on_table().build();

    // Act
    run_release_and_apply(&mut world);

    // Assert
    assert!(
        world.get::<FlipAnimation>(entity).is_none(),
        "expected no FlipAnimation for table drop"
    );
}

/// @doc: Full hand prevents pickup—card drops back to table if hand is at capacity
#[test]
fn when_hand_full_and_release_in_hand_area_then_card_stays_on_table() {
    // Arrange
    let (mut world, entity) = ReleaseTestBuilder::card_on_table()
        .screen_pos(400.0, 550.0)
        .hand_capacity(1)
        .pre_fill_hand(1)
        .build();

    // Act
    run_release_and_apply(&mut world);

    // Assert
    assert_eq!(*world.get::<CardZone>(entity).unwrap(), CardZone::Table);
}

/// @doc: Zero viewport height disables hand drop zone—guards against division by zero or absent UI
#[test]
fn when_viewport_height_zero_then_card_dropped_on_table_not_hand() {
    // Arrange
    let (mut world, _entity) = ReleaseTestBuilder::card_on_table()
        .viewport_height(0)
        .card_position(Vec2::new(50.0, 50.0))
        .build();

    // Act
    run_system(&mut world);

    // Assert
    let mut bus = world.resource_mut::<EventBus<InteractionIntent>>();
    let intents: Vec<_> = bus.drain().collect();
    assert_eq!(intents.len(), 1);
    assert!(
        matches!(&intents[0], InteractionIntent::ReleaseOnTable { .. }),
        "expected ReleaseOnTable intent when viewport height is 0, got {:?}",
        intents[0]
    );
}

#[test]
fn when_released_at_stash_slot_then_stash_drop_takes_priority_over_hand_zone() {
    // Arrange — slot (0,0,0) center at screen (45, 45); stash check runs BEFORE hand zone
    let (mut world, _entity) = ReleaseTestBuilder::card_on_table()
        .screen_pos(45.0, 45.0)
        .stash_visible()
        .build();

    // Act
    run_system(&mut world);

    // Assert
    let mut bus = world.resource_mut::<EventBus<InteractionIntent>>();
    let intents: Vec<_> = bus.drain().collect();
    assert_eq!(intents.len(), 1);
    assert!(
        matches!(&intents[0], InteractionIntent::ReleaseOnStash { .. }),
        "expected ReleaseOnStash intent (stash priority over hand zone), got {:?}",
        intents[0]
    );
}

#[test]
fn when_table_card_dropped_on_stash_slot_then_only_dragged_entity_gains_card_item_form() {
    // Arrange — slot (0,0,0) center: x=45.0, y=45.0
    let (mut world, dragged) = ReleaseTestBuilder::card_on_table()
        .screen_pos(45.0, 45.0)
        .stash_visible()
        .build();
    let bystander = world
        .spawn((
            card_game::test_helpers::make_test_card(),
            CardZone::Table,
            Transform2D {
                position: Vec2::new(200.0, 200.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();

    // Act
    run_release_and_apply(&mut world);

    // Assert
    assert!(
        world.get::<CardItemForm>(dragged).is_some(),
        "dragged entity should gain CardItemForm"
    );
    assert!(
        world.get::<CardItemForm>(bystander).is_none(),
        "bystander entity must not gain CardItemForm"
    );
}

#[test]
fn when_card_dropped_on_stash_then_scale_reset_to_one() {
    // Arrange — slot (0,0,0) center: x=45.0, y=57.5
    let (mut world, entity) = ReleaseTestBuilder::card_on_table()
        .screen_pos(45.0, 57.5)
        .stash_visible()
        .card_scale(Vec2::splat(0.833))
        .build();

    // Act
    run_release_and_apply(&mut world);

    // Assert
    assert_eq!(world.get::<Transform2D>(entity).unwrap().scale, Vec2::ONE);
}

#[test]
fn when_card_dropped_on_stash_then_rotation_reset_to_zero() {
    // Arrange — slot (0,0,0) center: x=45.0, y=57.5
    let (mut world, entity) = ReleaseTestBuilder::card_on_table()
        .screen_pos(45.0, 57.5)
        .stash_visible()
        .card_rotation(0.8)
        .build();

    // Act
    run_release_and_apply(&mut world);

    // Assert
    assert_eq!(world.get::<Transform2D>(entity).unwrap().rotation, 0.0);
}

/// @doc: `card_release_system` must emit a `ReleaseOnTable` intent instead of calling `drop_on_table`
/// directly. The release system decides WHERE the card lands; the applier system performs the
/// zone transition. If the release system both decides and applies, the intent/applier boundary
/// is violated and two systems race to own `DragState` and physics state on the same frame.
#[test]
fn when_mouse_released_over_table_then_release_on_table_intent_emitted() {
    // Arrange — card on table, mouse released well above hand zone
    let (mut world, entity) = ReleaseTestBuilder::card_on_table()
        .screen_pos(400.0, 100.0)
        .build();

    // Act
    run_system(&mut world);

    // Assert
    let mut bus = world.resource_mut::<EventBus<InteractionIntent>>();
    let intents: Vec<_> = bus.drain().collect();
    assert_eq!(intents.len(), 1, "expected exactly one release intent");
    match &intents[0] {
        InteractionIntent::ReleaseOnTable {
            entity: e,
            snap_back,
        } => {
            assert_eq!(*e, entity);
            assert!(!snap_back, "table drop should not snap back");
        }
        other => panic!("expected ReleaseOnTable, got {other:?}"),
    }
    assert!(
        world.resource::<DragState>().dragging.is_some(),
        "release system must NOT clear DragState; that is the applier's responsibility"
    );
}

/// @doc: The hand drop zone occupies the bottom 120px of the viewport. When the cursor
/// is in this zone on release, the system must emit `ReleaseOnHand` with the card's `face_up`
/// state so the applier can decide whether to insert a `FlipAnimation`. If `face_up` were
/// always defaulted, face-down cards would never auto-flip when entering the hand.
#[test]
fn when_mouse_released_over_hand_zone_then_release_on_hand_intent_emitted() {
    // Arrange — card on table, mouse in hand zone (bottom 120px of 600px viewport)
    let (mut world, entity) = ReleaseTestBuilder::card_on_table()
        .screen_pos(400.0, 550.0)
        .viewport_height(600)
        .build();

    // Act
    run_system(&mut world);

    // Assert
    let mut bus = world.resource_mut::<EventBus<InteractionIntent>>();
    let intents: Vec<_> = bus.drain().collect();
    assert_eq!(intents.len(), 1);
    assert!(
        matches!(&intents[0], InteractionIntent::ReleaseOnHand { entity: e, .. } if *e == entity),
        "expected ReleaseOnHand for entity {entity:?}, got {:?}",
        intents[0]
    );
}

/// @doc: When the cursor is over an empty stash slot on release, the system emits
/// `ReleaseOnStash` with the slot address so the applier can place the card in the grid.
/// The slot coordinates must be accurate or the card ends up in the wrong grid cell.
#[test]
fn when_mouse_released_over_empty_stash_slot_then_release_on_stash_intent_emitted() {
    // Arrange — stash visible, slot (0,0,0) is empty, mouse at slot center
    // Slot (0,0) center: x=20+0*54+25=45, y=20+0*79+37=57
    let (mut world, entity) = ReleaseTestBuilder::card_on_table()
        .stash_visible()
        .screen_pos(45.0, 57.0)
        .build();

    // Act
    run_system(&mut world);

    // Assert
    let mut bus = world.resource_mut::<EventBus<InteractionIntent>>();
    let intents: Vec<_> = bus.drain().collect();
    assert_eq!(intents.len(), 1);
    match &intents[0] {
        InteractionIntent::ReleaseOnStash {
            entity: e,
            page,
            col,
            row,
            ..
        } => {
            assert_eq!(*e, entity);
            assert_eq!(*page, 0);
            assert_eq!(*col, 0);
            assert_eq!(*row, 0);
        }
        other => panic!("expected ReleaseOnStash, got {other:?}"),
    }
}

/// @doc: When the cursor is over an occupied stash slot and the origin slot is also
/// occupied, the card has nowhere to go in the stash. The system must emit `ReleaseOnTable`
/// with `snap_back=true` so the applier teleports the card back to its origin position
/// rather than leaving it floating at the cursor location with no home zone.
#[test]
fn when_mouse_released_over_occupied_stash_slot_then_snap_back_intent_emitted() {
    // Arrange — stash visible, slot (0,0,0) is occupied by another card
    let (mut world, entity) = ReleaseTestBuilder::card_in_stash(0, 1, 1)
        .stash_visible()
        .screen_pos(45.0, 57.0)
        .build();
    // Occupy slot (0,0,0) with a different card
    let blocker = world.spawn_empty().id();
    world
        .resource_mut::<StashGrid>()
        .place(0, 0, 0, blocker)
        .unwrap();
    // Also occupy origin slot (0,1,1) so snap-back-to-origin can't happen
    let blocker2 = world.spawn_empty().id();
    world
        .resource_mut::<StashGrid>()
        .place(0, 1, 1, blocker2)
        .unwrap();

    // Act
    run_system(&mut world);

    // Assert
    let mut bus = world.resource_mut::<EventBus<InteractionIntent>>();
    let intents: Vec<_> = bus.drain().collect();
    assert_eq!(intents.len(), 1);
    match &intents[0] {
        InteractionIntent::ReleaseOnTable {
            entity: e,
            snap_back,
        } => {
            assert_eq!(*e, entity);
            assert!(*snap_back, "should snap back when stash target is occupied");
        }
        other => panic!("expected ReleaseOnTable with snap_back, got {other:?}"),
    }
}

/// @doc: When the mouse button is held (not released), the release system must not emit
/// any intent. This prevents phantom release intents during mid-drag frames, which would
/// cause the applier to end the drag prematurely and drop the card at the cursor position.
#[test]
fn when_mouse_not_released_then_no_release_intent_emitted() {
    // Arrange — mouse pressed but NOT released
    let mut world = World::new();
    let entity = world
        .spawn((
            card_game::test_helpers::make_test_card(),
            CardZone::Table,
            Collider::Aabb(Vec2::new(30.0, 45.0)),
            Transform2D {
                position: Vec2::ZERO,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();
    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Left);
    // Do NOT release
    mouse.set_screen_pos(Vec2::new(400.0, 100.0));
    world.insert_resource(mouse);
    world.insert_resource(DragState {
        dragging: Some(DragInfo {
            entity,
            local_grab_offset: Vec2::ZERO,
            origin_zone: CardZone::Table,
            stash_cursor_follow: false,
            origin_position: Vec2::ZERO,
        }),
    });
    world.insert_resource(Hand::new(10));
    world.insert_resource(EventBus::<PhysicsCommand>::default());
    world.insert_resource(EventBus::<InteractionIntent>::default());
    world.insert_resource(StashGrid::new(10, 10, 1));
    world.insert_resource(card_game::stash::toggle::StashVisible(false));
    let spy = engine_render::testing::SpyRenderer::new(Arc::new(Mutex::new(Vec::new())))
        .with_viewport(800, 600);
    world.insert_resource(RendererRes::new(Box::new(spy)));

    // Act
    run_system(&mut world);

    // Assert
    assert!(
        world.resource::<EventBus<InteractionIntent>>().is_empty(),
        "no intent when mouse is not released"
    );
    assert!(
        world.resource::<DragState>().dragging.is_some(),
        "drag must remain active"
    );
}
