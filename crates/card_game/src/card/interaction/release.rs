use bevy_ecs::prelude::{Commands, Entity, Query, Res};
use engine_core::prelude::Transform2D;
use engine_input::prelude::{MouseButton, MouseState};
use engine_physics::prelude::{Collider, PhysicsRes, RigidBody};
use engine_render::prelude::RendererRes;
use engine_scene::prelude::RenderLayer;
use glam::Vec2;

use crate::card::component::Card;
use crate::card::component::CardItemForm;
use crate::card::component::CardZone;
use crate::card::interaction::flip_animation::FlipAnimation;
use crate::card::interaction::game_state_param::CardGameState;
use crate::card::interaction::physics_helpers::activate_physics_body;
use crate::card::interaction::pick::{CARD_COLLISION_FILTER, CARD_COLLISION_GROUP};
use crate::card::rendering::drop_zone_glow::HAND_DROP_ZONE_HEIGHT;
use crate::hand::cards::Hand;
use crate::hand::layout::HandSpring;
use crate::stash::grid::StashGrid;
use crate::stash::grid::find_stash_slot_at;
use engine_core::scale_spring::ScaleSpring;

fn is_hand_drop_zone(screen_y: f32, viewport_height: f32) -> bool {
    screen_y >= viewport_height - HAND_DROP_ZONE_HEIGHT
}

enum DropTarget {
    Stash { page: u8, col: u8, row: u8 },
    Hand,
    Table,
    TableSnapBack,
}

fn resolve_drop_target(
    screen_pos: Vec2,
    viewport_height: f32,
    stash_visible: bool,
    grid: &StashGrid,
    origin_zone: &CardZone,
) -> DropTarget {
    if stash_visible
        && let Some((col, row)) = find_stash_slot_at(screen_pos, grid.width(), grid.height())
    {
        let page = grid.current_page();
        if grid.get(page, col, row).is_none() {
            return DropTarget::Stash { page, col, row };
        }
        if let CardZone::Stash {
            page: op,
            col: oc,
            row: orow,
        } = *origin_zone
            && grid.get(op, oc, orow).is_none()
        {
            return DropTarget::Stash {
                page: op,
                col: oc,
                row: orow,
            };
        }
        return DropTarget::TableSnapBack;
    }

    if viewport_height > 0.0 && is_hand_drop_zone(screen_pos.y, viewport_height) {
        DropTarget::Hand
    } else {
        DropTarget::Table
    }
}

pub fn card_release_system(
    mouse: Res<MouseState>,
    mut state: CardGameState,
    renderer: Res<RendererRes>,
    mut commands: Commands,
    transform_query: Query<(&Transform2D, &Collider)>,
    card_query: Query<&Card>,
) {
    let Some(info) = state.drag_state.dragging else {
        return;
    };
    if !mouse.just_released(MouseButton::Left) {
        return;
    }

    let screen_pos = mouse.screen_pos();
    let (_, vh) = renderer.viewport_size();
    let vh = vh as f32;

    let target = resolve_drop_target(
        screen_pos,
        vh,
        state.stash_visible.0,
        &state.grid,
        &info.origin_zone,
    );

    match target {
        DropTarget::Stash { page, col, row } => {
            let current_pos = transform_query
                .get(info.entity)
                .ok()
                .map(|(t, _)| t.position);
            drop_on_stash(
                info.entity,
                page,
                col,
                row,
                current_pos,
                &mut state.grid,
                &mut state.physics,
                &mut commands,
            );
        }
        DropTarget::Hand => {
            let face_up = card_query.get(info.entity).is_ok_and(|c| c.face_up);
            drop_on_hand(
                info.entity,
                face_up,
                info.origin_position,
                &mut state.hand,
                &mut state.physics,
                &mut commands,
            );
        }
        DropTarget::Table => {
            drop_on_table(
                info.entity,
                None,
                &mut state.physics,
                &mut commands,
                &transform_query,
            );
        }
        DropTarget::TableSnapBack => {
            drop_on_table(
                info.entity,
                Some(info.origin_position),
                &mut state.physics,
                &mut commands,
                &transform_query,
            );
        }
    }

    state.drag_state.dragging = None;
}

fn drop_on_hand(
    entity: Entity,
    face_up: bool,
    origin_position: Vec2,
    hand: &mut Hand,
    physics: &mut PhysicsRes,
    commands: &mut Commands,
) {
    let _ = physics.remove_body(entity); // ok to fail: stash/hand cards have no physics body
    let zone = if let Ok(index) = hand.add(entity) {
        CardZone::Hand(index)
    } else {
        commands.entity(entity).insert(Transform2D {
            position: origin_position,
            rotation: 0.0,
            scale: Vec2::ONE,
        });
        CardZone::Table
    };
    let mut ec = commands.entity(entity);
    ec.remove::<RigidBody>()
        .insert(zone)
        .insert(RenderLayer::UI)
        .remove::<CardItemForm>()
        .insert(HandSpring::new());
    if !face_up {
        ec.insert(FlipAnimation::start(true));
    }
}

fn drop_on_stash(
    entity: Entity,
    page: u8,
    col: u8,
    row: u8,
    current_pos: Option<Vec2>,
    grid: &mut StashGrid,
    physics: &mut PhysicsRes,
    commands: &mut Commands,
) {
    let _ = physics.remove_body(entity); // ok to fail: stash/hand cards have no physics body
    grid.place(page, col, row, entity)
        .expect("slot should be empty: guarded by is_none check above");
    let mut ec = commands.entity(entity);
    ec.remove::<RigidBody>()
        .insert(CardZone::Stash { page, col, row })
        .insert(RenderLayer::UI)
        .insert(CardItemForm);
    if let Some(pos) = current_pos {
        ec.insert(Transform2D {
            position: pos,
            rotation: 0.0,
            scale: Vec2::ONE,
        });
    }
}

fn drop_on_table(
    entity: Entity,
    snap_back: Option<Vec2>,
    physics: &mut PhysicsRes,
    commands: &mut Commands,
    transform_query: &Query<(&Transform2D, &Collider)>,
) {
    let position = if let Some(origin) = snap_back {
        origin
    } else {
        transform_query
            .get(entity)
            .map(|(t, _)| t.position)
            .unwrap_or(Vec2::ZERO)
    };

    if let Ok((_, collider)) = transform_query.get(entity) {
        activate_physics_body(
            entity,
            position,
            collider,
            physics,
            CARD_COLLISION_GROUP,
            CARD_COLLISION_FILTER,
        );
    }
    let mut ec = commands.entity(entity);
    ec.insert(RigidBody::Dynamic)
        .insert(CardZone::Table)
        .insert(RenderLayer::World)
        .remove::<CardItemForm>()
        .insert(ScaleSpring::new(1.0));
    if snap_back.is_some() {
        ec.insert(Transform2D {
            position,
            rotation: 0.0,
            scale: Vec2::ONE,
        });
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::struct_excessive_bools)]
mod tests {
    use std::sync::{Arc, Mutex};

    use bevy_ecs::prelude::*;
    use engine_core::prelude::Transform2D;
    use engine_input::prelude::{MouseButton, MouseState};
    use engine_physics::prelude::{Collider, PhysicsRes, RigidBody};
    use engine_render::prelude::RendererRes;
    use engine_render::testing::SpyRenderer;
    use engine_scene::prelude::RenderLayer;
    use glam::Vec2;

    use super::card_release_system;
    use crate::card::component::CardItemForm;
    use crate::card::component::CardZone;
    use crate::card::interaction::drag_state::{DragInfo, DragState};
    use crate::card::interaction::flip_animation::FlipAnimation;
    use crate::hand::cards::Hand;
    use crate::hand::layout::HandSpring;
    use crate::stash::grid::StashGrid;
    use crate::test_helpers::{AddBodyLog, RemoveBodyLog, SpyPhysicsBackend};

    fn run_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(card_release_system);
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

        fn build(self) -> (World, Entity, RemoveBodyLog, AddBodyLog) {
            let remove_log: RemoveBodyLog = Arc::new(Mutex::new(Vec::new()));
            let add_log: AddBodyLog = Arc::new(Mutex::new(Vec::new()));
            let mut world = World::new();
            world.insert_resource(PhysicsRes::new(Box::new(
                SpyPhysicsBackend::new()
                    .with_remove_body_log(remove_log.clone())
                    .with_add_body_log(add_log.clone()),
            )));

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
            world.insert_resource(crate::stash::toggle::StashVisible(self.stash_visible));

            let mut card = crate::test_helpers::make_test_card();
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

            (world, entity, remove_log, add_log)
        }
    }

    // ── Tests ────────────────────────────────────────────────────────

    #[test]
    fn when_mouse_released_while_dragging_then_drag_state_cleared() {
        // Arrange
        let (mut world, _, _, _) = ReleaseTestBuilder::card_on_table().build();

        // Act
        run_system(&mut world);

        // Assert
        assert!(world.resource::<DragState>().dragging.is_none());
    }

    #[test]
    fn when_mouse_released_while_not_dragging_then_no_panic_and_stays_none() {
        // Arrange
        let (mut world, _, _, _) = ReleaseTestBuilder::card_on_table().build();
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
        let log = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log).with_viewport(800, 600);
        world.insert_resource(RendererRes::new(Box::new(spy)));
        world.insert_resource(StashGrid::new(10, 10, 1));
        world.insert_resource(crate::stash::toggle::StashVisible(false));

        // Act
        run_system(&mut world);

        // Assert
        assert!(world.resource::<DragState>().dragging.is_some());
    }

    /// @doc: Table-to-table release preserves zone—player can drop and re-arrange cards on play surface
    #[test]
    fn when_card_released_on_table_then_zone_unchanged() {
        // Arrange
        let (mut world, entity, _, _) = ReleaseTestBuilder::card_on_table().build();

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(*world.get::<CardZone>(entity).unwrap(), CardZone::Table);
    }

    /// @doc: Hand transition removes physics body—hand cards never collide with table cards
    #[test]
    fn when_card_released_into_hand_from_table_then_full_zone_transition() {
        // Arrange
        let (mut world, entity, remove_log, _) = ReleaseTestBuilder::card_on_table()
            .screen_pos(400.0, 550.0)
            .build();

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(world.resource::<Hand>().cards(), &[entity]);
        assert_eq!(*world.get::<CardZone>(entity).unwrap(), CardZone::Hand(0));
        assert_eq!(*world.get::<RenderLayer>(entity).unwrap(), RenderLayer::UI);
        assert_eq!(remove_log.lock().unwrap().len(), 1);
        assert_eq!(remove_log.lock().unwrap()[0], entity);
    }

    /// @doc: Hand-to-table release transitions zone—card leaves hand inventory to play surface
    #[test]
    fn when_release_on_table_from_hand_then_zone_becomes_table() {
        // Arrange
        let (mut world, entity, _, _) = ReleaseTestBuilder::card_in_hand(0)
            .card_position(Vec2::new(50.0, 50.0))
            .build();

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(*world.get::<CardZone>(entity).unwrap(), CardZone::Table);
    }

    /// @doc: Hand-to-table transition adds physics body—card becomes subject to collisions and gravity
    #[test]
    fn when_release_on_table_from_hand_then_physics_body_added() {
        // Arrange
        let (mut world, entity, _, add_log) = ReleaseTestBuilder::card_in_hand(0)
            .card_position(Vec2::new(50.0, 50.0))
            .build();

        // Act
        run_system(&mut world);

        // Assert
        let calls = add_log.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, entity);
        assert_eq!(calls[0].1, Vec2::new(50.0, 50.0));
    }

    /// @doc: Face-down cards auto-flip to face-up on hand entry—reveal card on pickup
    /// @doc: Face-down cards auto-flip to face-up on hand entry—reveal card on pickup
    #[test]
    fn when_face_down_card_released_into_hand_then_flip_animation_inserted() {
        // Arrange
        let (mut world, entity, _, _) = ReleaseTestBuilder::card_on_table()
            .screen_pos(400.0, 550.0)
            .build();

        // Act
        run_system(&mut world);

        // Assert
        let flip = world.get::<FlipAnimation>(entity);
        assert!(flip.is_some(), "expected FlipAnimation to be inserted");
        assert!(flip.unwrap().target_face_up, "expected target_face_up=true");
    }

    #[test]
    fn when_face_up_card_released_into_hand_then_no_flip_animation() {
        // Arrange
        let (mut world, entity, _, _) = ReleaseTestBuilder::card_on_table()
            .screen_pos(400.0, 550.0)
            .face_up()
            .build();

        // Act
        run_system(&mut world);

        // Assert
        assert!(
            world.get::<FlipAnimation>(entity).is_none(),
            "expected no FlipAnimation for face-up card"
        );
    }

    #[test]
    fn when_face_down_card_released_on_table_then_no_flip_animation() {
        // Arrange
        let (mut world, entity, _, _) = ReleaseTestBuilder::card_on_table().build();

        // Act
        run_system(&mut world);

        // Assert
        assert!(
            world.get::<FlipAnimation>(entity).is_none(),
            "expected no FlipAnimation for table drop"
        );
    }

    #[test]
    fn when_face_down_card_released_into_hand_then_also_added_to_hand() {
        // Arrange
        let (mut world, entity, _, _) = ReleaseTestBuilder::card_on_table()
            .screen_pos(400.0, 550.0)
            .build();

        // Act
        run_system(&mut world);

        // Assert
        assert!(
            world.resource::<Hand>().cards().contains(&entity),
            "expected card in hand"
        );
        assert!(
            world.get::<FlipAnimation>(entity).is_some(),
            "expected FlipAnimation also present"
        );
    }

    /// @doc: Full hand prevents pickup—card drops back to table if hand is at capacity
    #[test]
    fn when_hand_full_and_release_in_hand_area_then_card_stays_on_table() {
        // Arrange
        let (mut world, entity, _, _) = ReleaseTestBuilder::card_on_table()
            .screen_pos(400.0, 550.0)
            .hand_capacity(1)
            .pre_fill_hand(1)
            .build();

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(*world.get::<CardZone>(entity).unwrap(), CardZone::Table);
    }

    /// @doc: Zero viewport height disables hand drop zone—guards against division by zero or absent UI
    #[test]
    fn when_viewport_height_zero_then_card_dropped_on_table_not_hand() {
        // Arrange
        let (mut world, entity, _, add_log) = ReleaseTestBuilder::card_on_table()
            .viewport_height(0)
            .card_position(Vec2::new(50.0, 50.0))
            .build();

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(*world.get::<CardZone>(entity).unwrap(), CardZone::Table);
        assert!(
            world.resource::<Hand>().cards().is_empty(),
            "card should not be added to hand when viewport height is 0"
        );
        assert_eq!(
            add_log.lock().unwrap().len(),
            1,
            "physics body should be re-added for table drop"
        );
    }

    /// @doc: `HandSpring` attached on hand pickup—card animates to its layout position
    #[test]
    fn when_release_to_hand_then_handspring_inserted() {
        // Arrange
        let (mut world, entity, _, _) = ReleaseTestBuilder::card_on_table()
            .screen_pos(400.0, 550.0)
            .build();

        // Act
        run_system(&mut world);

        // Assert
        assert!(
            world.get::<HandSpring>(entity).is_some(),
            "HandSpring should be inserted on release to hand"
        );
    }

    #[test]
    fn when_stash_card_released_over_empty_stash_slot_then_full_stash_transition() {
        // Arrange
        // slot (0,1,0) center: x = 20 + 1*54 + 25 = 99.0, y = 20 + 0*54 + 25 = 45.0
        let (mut world, entity, remove_log, _) = ReleaseTestBuilder::card_in_stash(0, 0, 0)
            .screen_pos(99.0, 45.0)
            .build();

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(
            world.resource::<StashGrid>().get(0, 1, 0),
            Some(&entity),
            "card should be placed at slot (0,1,0)"
        );
        assert_eq!(
            *world.get::<CardZone>(entity).unwrap(),
            CardZone::Stash {
                page: 0,
                col: 1,
                row: 0
            }
        );
        assert_eq!(*world.get::<RenderLayer>(entity).unwrap(), RenderLayer::UI);
        assert_eq!(
            remove_log.lock().unwrap().len(),
            1,
            "remove_body called once"
        );
        assert!(world.resource::<DragState>().dragging.is_none());
    }

    /// @doc: Occupied stash slot forces snap-back to origin—collision avoidance for stash placement
    #[test]
    fn when_released_over_occupied_stash_slot_then_card_returned_to_origin() {
        // Arrange — slot (0,0,0) occupied by blocker, origin slot (0,1,0)
        let (mut world, entity, _, _) = ReleaseTestBuilder::card_in_stash(0, 1, 0)
            .screen_pos(45.0, 45.0)
            .build();
        // Override: place blocker in slot (0,0,0) so drop fails
        let blocker = world.spawn_empty().id();
        let mut grid = StashGrid::new(10, 10, 1);
        grid.place(0, 0, 0, blocker).unwrap();
        world.insert_resource(grid);

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(
            world.resource::<StashGrid>().get(0, 1, 0),
            Some(&entity),
            "card should be returned to origin slot (0,1,0)"
        );
        assert_eq!(
            *world.get::<CardZone>(entity).unwrap(),
            CardZone::Stash {
                page: 0,
                col: 1,
                row: 0
            }
        );
    }

    #[test]
    fn when_stash_card_released_outside_zones_then_drops_on_table() {
        // Arrange — x=600 past stash grid; y=200 above hand zone
        let (mut world, entity, _, add_log) = ReleaseTestBuilder::card_in_stash(0, 0, 0)
            .screen_pos(600.0, 200.0)
            .card_position(Vec2::new(10.0, 20.0))
            .build();

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(*world.get::<CardZone>(entity).unwrap(), CardZone::Table);
        assert_eq!(
            add_log.lock().unwrap().len(),
            1,
            "physics body should be re-added for table drop"
        );
    }

    #[test]
    fn when_stash_card_released_in_hand_zone_then_full_hand_transition() {
        // Arrange — x=600 past stash grid; y=550 in hand zone
        let (mut world, entity, _, _) = ReleaseTestBuilder::card_in_stash(0, 0, 0)
            .screen_pos(600.0, 550.0)
            .build();

        // Act
        run_system(&mut world);

        // Assert
        assert!(
            world.resource::<Hand>().cards().contains(&entity),
            "stash-origin card should be in Hand"
        );
        assert_eq!(*world.get::<CardZone>(entity).unwrap(), CardZone::Hand(0));
        assert!(
            world.resource::<StashGrid>().get(0, 0, 0).is_none(),
            "stash slot should not be repopulated after drop-on-hand"
        );
    }

    #[test]
    fn when_table_card_released_over_empty_stash_slot_then_full_stash_transition() {
        // Arrange — slot (0,0,0) center: x=45.0, y=45.0
        let (mut world, entity, remove_log, _) = ReleaseTestBuilder::card_on_table()
            .screen_pos(45.0, 45.0)
            .stash_visible()
            .build();

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(
            world.resource::<StashGrid>().get(0, 0, 0),
            Some(&entity),
            "card should occupy slot (0,0,0)"
        );
        assert_eq!(
            *world.get::<CardZone>(entity).unwrap(),
            CardZone::Stash {
                page: 0,
                col: 0,
                row: 0
            }
        );
        let calls = remove_log.lock().unwrap();
        assert_eq!(calls.len(), 1, "remove_body should be called once");
        assert_eq!(calls[0], entity);
    }

    #[test]
    fn when_hand_card_released_over_empty_stash_slot_then_not_in_hand_resource() {
        // Arrange — slot (0,0,0) center: x=45.0, y=45.0
        let (mut world, entity, _, _) = ReleaseTestBuilder::card_in_hand(0)
            .screen_pos(45.0, 45.0)
            .stash_visible()
            .build();

        // Act
        run_system(&mut world);

        // Assert
        assert!(
            !world.resource::<Hand>().cards().contains(&entity),
            "hand-origin card dropped on stash must not be in Hand resource"
        );
        assert_eq!(
            world.resource::<StashGrid>().get(0, 0, 0),
            Some(&entity),
            "card should be in stash grid at slot (0,0,0)"
        );
    }

    #[test]
    fn when_released_at_stash_slot_then_stash_drop_takes_priority_over_hand_zone() {
        // Arrange — slot (0,0,0) center at screen (45, 45); stash check runs BEFORE hand zone
        let (mut world, entity, _, _) = ReleaseTestBuilder::card_on_table()
            .screen_pos(45.0, 45.0)
            .stash_visible()
            .build();

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(
            world.resource::<StashGrid>().get(0, 0, 0),
            Some(&entity),
            "card should be in stash slot"
        );
        assert!(!world.resource::<Hand>().cards().contains(&entity));
    }

    #[test]
    fn when_table_card_dropped_on_stash_slot_then_card_item_form_inserted() {
        // Arrange — slot (0,0,0) center: x=45.0, y=45.0
        let (mut world, entity, _, _) = ReleaseTestBuilder::card_on_table()
            .screen_pos(45.0, 45.0)
            .stash_visible()
            .build();

        // Act
        run_system(&mut world);

        // Assert
        assert!(
            world.get::<CardItemForm>(entity).is_some(),
            "CardItemForm should be inserted when card is dropped on stash slot"
        );
    }

    #[test]
    fn when_stash_card_dropped_on_table_area_then_card_item_form_removed() {
        // Arrange — x=600 past stash grid; y=200 above hand zone
        let (mut world, entity, _, _) = ReleaseTestBuilder::card_in_stash(0, 0, 0)
            .screen_pos(600.0, 200.0)
            .with_item_form()
            .build();

        // Act
        run_system(&mut world);

        // Assert
        assert!(
            world.get::<CardItemForm>(entity).is_none(),
            "CardItemForm should be removed when card is dropped on table area"
        );
    }

    #[test]
    fn when_stash_card_dropped_on_hand_zone_then_card_item_form_removed() {
        // Arrange — x=600 past stash grid; y=550 in hand zone
        let (mut world, entity, _, _) = ReleaseTestBuilder::card_in_stash(0, 0, 0)
            .screen_pos(600.0, 550.0)
            .with_item_form()
            .build();

        // Act
        run_system(&mut world);

        // Assert
        assert!(
            world.get::<CardItemForm>(entity).is_none(),
            "CardItemForm should be removed when card is dropped on hand zone"
        );
    }

    #[test]
    fn when_table_card_dropped_on_stash_slot_then_only_dragged_entity_gains_card_item_form() {
        // Arrange — slot (0,0,0) center: x=45.0, y=45.0
        let (mut world, dragged, _, _) = ReleaseTestBuilder::card_on_table()
            .screen_pos(45.0, 45.0)
            .stash_visible()
            .build();
        let bystander = world
            .spawn((
                crate::test_helpers::make_test_card(),
                CardZone::Table,
                Transform2D {
                    position: Vec2::new(200.0, 200.0),
                    rotation: 0.0,
                    scale: Vec2::ONE,
                },
            ))
            .id();

        // Act
        run_system(&mut world);

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
        let (mut world, entity, _, _) = ReleaseTestBuilder::card_on_table()
            .screen_pos(45.0, 57.5)
            .stash_visible()
            .card_scale(Vec2::splat(0.833))
            .build();

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(world.get::<Transform2D>(entity).unwrap().scale, Vec2::ONE);
    }

    #[test]
    fn when_card_released_on_table_then_scale_spring_target_is_one() {
        use engine_core::scale_spring::ScaleSpring;

        // Arrange
        let (mut world, entity, _, _) = ReleaseTestBuilder::card_on_table().build();

        // Act
        run_system(&mut world);

        // Assert
        let spring = world
            .get::<ScaleSpring>(entity)
            .expect("ScaleSpring should be inserted on table release");
        assert_eq!(spring.target, 1.0);
    }

    #[test]
    fn when_card_dropped_on_stash_then_rotation_reset_to_zero() {
        // Arrange — slot (0,0,0) center: x=45.0, y=57.5
        let (mut world, entity, _, _) = ReleaseTestBuilder::card_on_table()
            .screen_pos(45.0, 57.5)
            .stash_visible()
            .card_rotation(0.8)
            .build();

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(world.get::<Transform2D>(entity).unwrap().rotation, 0.0);
    }

    #[test]
    fn when_table_card_dropped_on_occupied_stash_then_position_restored_to_origin() {
        // Arrange
        let (mut world, entity, _, _) = ReleaseTestBuilder::card_on_table()
            .screen_pos(45.0, 45.0)
            .stash_visible()
            .card_position(Vec2::new(300.0, 400.0))
            .origin_position(Vec2::new(50.0, 75.0))
            .build();
        // Override: place blocker in slot (0,0,0) so drop fails => snap back
        let blocker = world.spawn_empty().id();
        let mut grid = StashGrid::new(10, 10, 1);
        grid.place(0, 0, 0, blocker).expect("blocker placed");
        world.insert_resource(grid);

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(
            world.get::<Transform2D>(entity).unwrap().position,
            Vec2::new(50.0, 75.0),
            "card should snap back to origin_position"
        );
    }

    #[test]
    fn when_table_card_dropped_on_empty_stash_then_position_not_forced_to_origin() {
        // Arrange — slot (0,0,0) is empty, drop should succeed
        let (mut world, entity, _, _) = ReleaseTestBuilder::card_on_table()
            .screen_pos(45.0, 57.5)
            .stash_visible()
            .card_position(Vec2::new(300.0, 400.0))
            .origin_position(Vec2::new(50.0, 75.0))
            .build();

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(
            *world.get::<CardZone>(entity).unwrap(),
            CardZone::Stash {
                page: 0,
                col: 0,
                row: 0
            }
        );
        assert_ne!(
            world.get::<Transform2D>(entity).unwrap().position,
            Vec2::new(50.0, 75.0),
            "valid drop should not snap back to origin"
        );
    }

    #[test]
    fn when_table_card_dropped_on_full_hand_then_position_restored_to_origin() {
        // Arrange
        let (mut world, entity, _, _) = ReleaseTestBuilder::card_on_table()
            .screen_pos(400.0, 550.0)
            .hand_capacity(1)
            .pre_fill_hand(1)
            .card_position(Vec2::new(300.0, 400.0))
            .origin_position(Vec2::new(30.0, 40.0))
            .build();

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(*world.get::<CardZone>(entity).unwrap(), CardZone::Table);
        assert_eq!(
            world.get::<Transform2D>(entity).unwrap().position,
            Vec2::new(30.0, 40.0),
            "card should snap back to origin_position when hand is full"
        );
    }

    #[test]
    fn when_hand_card_released_over_empty_stash_slot_then_placed_in_stash_not_in_hand() {
        // Arrange — slot (0,0,0) center: x=45.0, y=45.0
        let (mut world, entity, _, _) = ReleaseTestBuilder::card_in_hand(0)
            .screen_pos(45.0, 45.0)
            .stash_visible()
            .build();

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(
            world.resource::<StashGrid>().get(0, 0, 0),
            Some(&entity),
            "card should be in stash grid"
        );
        assert!(
            !world.resource::<Hand>().cards().contains(&entity),
            "hand resource should not contain the card after stash drop"
        );
    }
}
