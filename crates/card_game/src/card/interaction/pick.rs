use bevy_ecs::prelude::{Commands, Entity, Query, Res};
use engine_input::prelude::{MouseButton, MouseState};
use engine_physics::prelude::{Collider, PhysicsRes, RigidBody};
use engine_scene::prelude::{GlobalTransform2D, LocalSortOrder, RenderLayer, SortOrder};
use glam::Vec2;

use crate::card::component::Card;
use crate::card::component::CardItemForm;
use crate::card::component::CardZone;
use crate::card::interaction::drag_state::{DragInfo, DragState};
use crate::card::interaction::game_state_param::CardGameState;
use crate::card::interaction::physics_helpers::activate_physics_body;
use crate::card::reader::ReaderDragState;
use crate::hand::cards::Hand;
use crate::hand::layout::HandSpring;
use engine_core::scale_spring::ScaleSpring;

mod hit_test;
mod source;

pub const CARD_COLLISION_GROUP: u32 = 0b0001;
pub const CARD_COLLISION_FILTER: u32 = 0b0010;
pub(crate) const DRAGGED_COLLISION_GROUP: u32 = 0;
pub(crate) const DRAGGED_COLLISION_FILTER: u32 = 0;
pub(crate) const DRAG_SCALE: f32 = 1.05;

pub(crate) use source::PickSource;

pub fn card_pick_system(
    mouse: Res<MouseState>,
    mut state: CardGameState,
    reader_drag: Res<ReaderDragState>,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &Card,
        &CardZone,
        &GlobalTransform2D,
        &Collider,
        &SortOrder,
    )>,
) {
    if state.drag_state.dragging.is_some() || reader_drag.dragging.is_some() {
        return;
    }
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(source) =
        source::identify_pick_source(&mouse, &state.stash_visible, &mut state.grid, &query)
    else {
        return;
    };

    match source {
        PickSource::Stash {
            entity,
            page,
            col,
            row,
        } => {
            state.grid.take(page, col, row);
            pick_from_stash(entity, page, col, row, &mut state.drag_state, &mut commands);
        }
        PickSource::Card {
            entity,
            zone,
            collider,
            grab_offset,
        } => {
            pick_from_card(
                entity,
                zone,
                collider,
                grab_offset,
                &mut state,
                &mut commands,
                &mut query,
            );
        }
    }
}

fn pick_from_stash(
    entity: Entity,
    page: u8,
    col: u8,
    row: u8,
    drag_state: &mut DragState,
    commands: &mut Commands,
) {
    commands.entity(entity).insert(CardZone::Table);
    commands.entity(entity).remove::<CardItemForm>();
    commands.entity(entity).insert(ScaleSpring::new(DRAG_SCALE));
    drag_state.dragging = Some(DragInfo {
        entity,
        local_grab_offset: Vec2::ZERO,
        origin_zone: CardZone::Stash { page, col, row },
        stash_cursor_follow: true,
        origin_position: Vec2::ZERO,
    });
}

fn pick_from_card(
    entity: Entity,
    zone: CardZone,
    collider: Collider,
    grab_offset: Vec2,
    state: &mut CardGameState,
    commands: &mut Commands,
    query: &mut Query<(
        Entity,
        &Card,
        &CardZone,
        &GlobalTransform2D,
        &Collider,
        &SortOrder,
    )>,
) {
    let max_sort = max_table_sort_order(query);

    if let CardZone::Hand(_) = zone {
        transition_hand_to_table(
            entity,
            &mut state.hand,
            &mut state.physics,
            commands,
            query,
            &collider,
        );
    }

    if let CardZone::Stash { page, col, row } = zone {
        state.grid.take(page, col, row);
        commands.entity(entity).insert(CardZone::Table);
        commands.entity(entity).remove::<CardItemForm>();
    }

    if matches!(zone, CardZone::Table) {
        state
            .physics
            .set_collision_group(entity, DRAGGED_COLLISION_GROUP, DRAGGED_COLLISION_FILTER)
            .expect("picked entity should have physics body");
    }

    let origin_position = query
        .get(entity)
        .map(|(_, _, _, t, _, _)| t.0.translation)
        .unwrap_or(Vec2::ZERO);
    state.drag_state.dragging = Some(DragInfo {
        entity,
        local_grab_offset: grab_offset,
        origin_zone: zone,
        stash_cursor_follow: false,
        origin_position,
    });
    commands.entity(entity).insert(LocalSortOrder(max_sort + 1));
    commands.entity(entity).insert(ScaleSpring::new(DRAG_SCALE));
}

fn max_table_sort_order(
    query: &Query<(
        Entity,
        &Card,
        &CardZone,
        &GlobalTransform2D,
        &Collider,
        &SortOrder,
    )>,
) -> i32 {
    query
        .iter()
        .filter(|(_, _, zone, _, _, _)| **zone == CardZone::Table)
        .map(|(_, _, _, _, _, sort)| sort.value())
        .max()
        .unwrap_or(0)
}

fn transition_hand_to_table(
    entity: Entity,
    hand: &mut Hand,
    physics: &mut PhysicsRes,
    commands: &mut Commands,
    query: &Query<(
        Entity,
        &Card,
        &CardZone,
        &GlobalTransform2D,
        &Collider,
        &SortOrder,
    )>,
    collider: &Collider,
) {
    hand.remove(entity);
    let position = query
        .get(entity)
        .map(|(_, _, _, t, _, _)| t.0.translation)
        .unwrap_or(Vec2::ZERO);
    activate_physics_body(
        entity,
        position,
        collider,
        physics,
        DRAGGED_COLLISION_GROUP,
        DRAGGED_COLLISION_FILTER,
    );
    commands.entity(entity).insert(RigidBody::Dynamic);
    commands.entity(entity).insert(RenderLayer::World);
    commands.entity(entity).remove::<HandSpring>();
    commands.entity(entity).insert(ScaleSpring::new(1.0));
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::float_cmp)]
mod tests {
    use std::sync::{Arc, Mutex};

    use bevy_ecs::prelude::*;
    use engine_input::prelude::{MouseButton, MouseState};
    use engine_physics::prelude::{Collider, NullPhysicsBackend, PhysicsBackend, PhysicsRes};
    use engine_scene::prelude::{GlobalTransform2D, LocalSortOrder, RenderLayer, SortOrder};
    use glam::{Affine2, Vec2};

    use engine_core::prelude::TextureId;

    use super::{DRAG_SCALE, card_pick_system};
    use crate::card::component::Card;
    use crate::card::component::CardZone;
    use crate::card::interaction::drag_state::DragState;
    use crate::card::reader::ReaderDragState;
    use crate::hand::cards::Hand;
    use crate::hand::layout::HandSpring;
    use crate::test_helpers::{AddBodyLog, ColliderLog, DampingLog, SpyPhysicsBackend};
    use engine_core::scale_spring::ScaleSpring;

    fn run_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(card_pick_system);
        schedule.run(world);
    }

    fn default_collider() -> Collider {
        Collider::Aabb(Vec2::new(30.0, 45.0))
    }

    fn insert_pick_resources(world: &mut World) {
        use crate::card::reader::ReaderDragState;
        use crate::stash::grid::StashGrid;
        use crate::stash::toggle::StashVisible;
        world.insert_resource(Hand::new(10));
        world.insert_resource(PhysicsRes::new(Box::new(NullPhysicsBackend::default())));
        world.insert_resource(StashGrid::new(10, 10, 1));
        world.insert_resource(StashVisible(false));
        world.insert_resource(ReaderDragState::default());
    }

    #[test]
    fn when_left_click_on_single_table_card_then_drag_state_contains_that_entity() {
        // Arrange
        let mut world = World::new();
        let card_entity = world
            .spawn((
                crate::test_helpers::make_test_card(),
                CardZone::Table,
                default_collider(),
                GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
                SortOrder::new(0),
            ))
            .id();
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::ZERO);
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());
        world.insert_resource(ReaderDragState::default());
        insert_pick_resources(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        let drag = world.resource::<DragState>().dragging;
        assert_eq!(drag.map(|d| d.entity), Some(card_entity));
    }

    #[test]
    fn when_left_click_at_card_center_then_drag_state_is_some() {
        // Arrange
        let mut world = World::new();
        world.spawn((
            Card::face_down(TextureId(3), TextureId(4)),
            CardZone::Table,
            default_collider(),
            GlobalTransform2D(Affine2::from_translation(Vec2::new(100.0, 50.0))),
            SortOrder::new(0),
        ));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::new(100.0, 50.0));
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());
        world.insert_resource(ReaderDragState::default());
        insert_pick_resources(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        assert!(world.resource::<DragState>().dragging.is_some());
    }

    #[test]
    fn when_left_click_on_table_card_then_drag_info_records_table_origin_zone() {
        // Arrange
        let mut world = World::new();
        world.spawn((
            Card::face_down(TextureId(5), TextureId(6)),
            CardZone::Table,
            default_collider(),
            GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
            SortOrder::new(0),
        ));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::ZERO);
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());
        world.insert_resource(ReaderDragState::default());
        insert_pick_resources(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        let drag = world.resource::<DragState>().dragging;
        assert_eq!(drag.map(|d| d.origin_zone), Some(CardZone::Table));
    }

    #[test]
    fn when_left_click_outside_all_cards_then_drag_state_remains_none() {
        // Arrange
        let mut world = World::new();
        world.spawn((
            crate::test_helpers::make_test_card(),
            CardZone::Table,
            default_collider(),
            GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
            SortOrder::new(0),
        ));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::new(200.0, 200.0));
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());
        world.insert_resource(ReaderDragState::default());
        insert_pick_resources(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        assert!(world.resource::<DragState>().dragging.is_none());
    }

    #[test]
    fn when_left_click_with_no_table_cards_then_drag_state_remains_none() {
        // Arrange
        let mut world = World::new();
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::ZERO);
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());
        world.insert_resource(ReaderDragState::default());
        insert_pick_resources(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        assert!(world.resource::<DragState>().dragging.is_none());
    }

    /// @doc: Sort order selects which card to pick when overlapping—highest sort is topmost
    #[test]
    fn when_two_overlapping_cards_then_picks_highest_sort_order() {
        // Arrange
        let mut world = World::new();
        let _card_a = world
            .spawn((
                crate::test_helpers::make_test_card(),
                CardZone::Table,
                default_collider(),
                GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
                SortOrder::new(0),
            ))
            .id();
        let card_b = world
            .spawn((
                Card::face_down(TextureId(3), TextureId(4)),
                CardZone::Table,
                default_collider(),
                GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
                SortOrder::new(5),
            ))
            .id();
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::ZERO);
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());
        world.insert_resource(ReaderDragState::default());
        insert_pick_resources(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        let drag = world.resource::<DragState>().dragging;
        assert_eq!(drag.map(|d| d.entity), Some(card_b));
    }

    /// @doc: Sort order bump on pick prevents z-fighting when overlapping cards are rearranged
    #[test]
    fn when_card_picked_then_sort_order_bumped_above_all_others() {
        // Arrange
        let mut world = World::new();
        let card_a = world
            .spawn((
                crate::test_helpers::make_test_card(),
                CardZone::Table,
                default_collider(),
                GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
                SortOrder::new(0),
            ))
            .id();
        world.spawn((
            Card::face_down(TextureId(3), TextureId(4)),
            CardZone::Table,
            default_collider(),
            GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 0.0))),
            SortOrder::new(7),
        ));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::ZERO);
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());
        world.insert_resource(ReaderDragState::default());
        insert_pick_resources(&mut world);

        // Act
        run_system(&mut world);
        world.flush();

        // Assert — pick system inserts LocalSortOrder above max table sort
        let local = world
            .entity(card_a)
            .get::<LocalSortOrder>()
            .expect("picked card should have LocalSortOrder");
        assert!(
            local.0 > 7,
            "picked card LocalSortOrder {} should be > 7",
            local.0
        );
    }

    #[test]
    fn when_already_dragging_then_new_click_does_not_replace_drag() {
        // Arrange
        let mut world = World::new();
        let card_a = world.spawn_empty().id();
        world.spawn((
            crate::test_helpers::make_test_card(),
            CardZone::Table,
            default_collider(),
            GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
            SortOrder::new(0),
        ));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::ZERO);
        world.insert_resource(mouse);
        world.insert_resource(DragState {
            dragging: Some(crate::card::interaction::drag_state::DragInfo {
                entity: card_a,
                local_grab_offset: Vec2::ZERO,
                origin_zone: CardZone::Table,
                stash_cursor_follow: false,
                origin_position: Vec2::ZERO,
            }),
        });
        insert_pick_resources(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        let drag = world.resource::<DragState>().dragging.unwrap();
        assert_eq!(drag.entity, card_a);
    }

    /// @doc: Local grab offset transformed by inverse rotation—drag stays aligned to card frame even after rotation
    #[test]
    fn when_card_picked_at_offset_then_local_grab_offset_is_inverse_rotated() {
        // Arrange
        let mut world = World::new();
        let angle = std::f32::consts::FRAC_PI_4;
        let transform =
            Affine2::from_scale_angle_translation(Vec2::ONE, angle, Vec2::new(100.0, 50.0));
        world.spawn((
            crate::test_helpers::make_test_card(),
            CardZone::Table,
            default_collider(),
            GlobalTransform2D(transform),
            SortOrder::new(0),
        ));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::new(110.0, 50.0));
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());
        world.insert_resource(ReaderDragState::default());
        insert_pick_resources(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        let offset = world
            .resource::<DragState>()
            .dragging
            .unwrap()
            .local_grab_offset;
        let expected_x = 10.0_f32 * angle.cos();
        let expected_y = -10.0_f32 * angle.sin();
        assert!(
            (offset.x - expected_x).abs() < 1e-4,
            "offset.x={} expected ~{expected_x}",
            offset.x
        );
        assert!(
            (offset.y - expected_y).abs() < 1e-4,
            "offset.y={} expected ~{expected_y}",
            offset.y
        );
    }

    #[test]
    fn when_card_picked_at_center_then_local_grab_offset_is_zero() {
        // Arrange
        let mut world = World::new();
        world.spawn((
            crate::test_helpers::make_test_card(),
            CardZone::Table,
            default_collider(),
            GlobalTransform2D(Affine2::from_translation(Vec2::new(100.0, 50.0))),
            SortOrder::new(0),
        ));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::new(100.0, 50.0));
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());
        world.insert_resource(ReaderDragState::default());
        insert_pick_resources(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        let offset = world
            .resource::<DragState>()
            .dragging
            .unwrap()
            .local_grab_offset;
        assert!(
            offset.length() < 1e-6,
            "offset should be ~zero, got {offset}"
        );
    }

    #[test]
    fn when_cursor_on_edge_of_card_then_card_is_picked() {
        // Arrange
        let mut world = World::new();
        world.spawn((
            crate::test_helpers::make_test_card(),
            CardZone::Table,
            default_collider(),
            GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
            SortOrder::new(0),
        ));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::new(30.0, 0.0));
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());
        world.insert_resource(ReaderDragState::default());
        insert_pick_resources(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        assert!(world.resource::<DragState>().dragging.is_some());
    }

    /// @doc: OBB hit test accounts for card rotation—axis-aligned check would miss rotated cards
    #[test]
    fn when_rotated_card_clicked_inside_obb_then_picked() {
        // Arrange
        let mut world = World::new();
        let angle = std::f32::consts::FRAC_PI_4;
        world.spawn((
            crate::test_helpers::make_test_card(),
            CardZone::Table,
            default_collider(),
            GlobalTransform2D(Affine2::from_scale_angle_translation(
                Vec2::ONE,
                angle,
                Vec2::ZERO,
            )),
            SortOrder::new(0),
        ));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::new(20.0, 20.0));
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());
        world.insert_resource(ReaderDragState::default());
        insert_pick_resources(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        assert!(world.resource::<DragState>().dragging.is_some());
    }

    #[test]
    fn when_rotated_card_clicked_outside_obb_then_not_picked() {
        // Arrange
        let mut world = World::new();
        let angle = std::f32::consts::FRAC_PI_4;
        world.spawn((
            crate::test_helpers::make_test_card(),
            CardZone::Table,
            default_collider(),
            GlobalTransform2D(Affine2::from_scale_angle_translation(
                Vec2::ONE,
                angle,
                Vec2::ZERO,
            )),
            SortOrder::new(0),
        ));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::new(50.0, 0.0));
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());
        world.insert_resource(ReaderDragState::default());
        insert_pick_resources(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        assert!(world.resource::<DragState>().dragging.is_none());
    }

    fn make_spy_physics() -> (
        Box<dyn PhysicsBackend + Send + Sync>,
        AddBodyLog,
        ColliderLog,
        DampingLog,
    ) {
        let bodies: AddBodyLog = Arc::new(Mutex::new(Vec::new()));
        let colliders: ColliderLog = Arc::new(Mutex::new(Vec::new()));
        let dampings: DampingLog = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyPhysicsBackend::new()
            .with_add_body_log(bodies.clone())
            .with_collider_log(colliders.clone())
            .with_damping_log(dampings.clone());
        (Box::new(spy), bodies, colliders, dampings)
    }

    #[test]
    fn when_left_click_on_hand_card_then_drag_origin_is_hand() {
        // Arrange
        let mut world = World::new();
        let card_entity = world
            .spawn((
                crate::test_helpers::make_test_card(),
                CardZone::Hand(0),
                default_collider(),
                GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
                SortOrder::new(0),
            ))
            .id();
        let mut hand = Hand::new(10);
        hand.add(card_entity).unwrap();
        world.insert_resource(hand);
        let (spy, _, _, _) = make_spy_physics();
        world.insert_resource(PhysicsRes::new(spy));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::ZERO);
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());
        world.insert_resource(ReaderDragState::default());
        world.insert_resource(crate::stash::grid::StashGrid::new(10, 10, 1));
        world.insert_resource(crate::stash::toggle::StashVisible(false));

        // Act
        run_system(&mut world);

        // Assert
        let drag = world.resource::<DragState>().dragging;
        assert_eq!(drag.map(|d| d.origin_zone), Some(CardZone::Hand(0)));
    }

    #[test]
    fn when_pick_from_hand_then_card_removed_from_hand_resource() {
        // Arrange
        let mut world = World::new();
        let card_entity = world
            .spawn((
                crate::test_helpers::make_test_card(),
                CardZone::Hand(0),
                default_collider(),
                GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
                SortOrder::new(0),
            ))
            .id();
        let mut hand = Hand::new(10);
        hand.add(card_entity).unwrap();
        world.insert_resource(hand);
        let (spy, _, _, _) = make_spy_physics();
        world.insert_resource(PhysicsRes::new(spy));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::ZERO);
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());
        world.insert_resource(ReaderDragState::default());
        world.insert_resource(crate::stash::grid::StashGrid::new(10, 10, 1));
        world.insert_resource(crate::stash::toggle::StashVisible(false));

        // Act
        run_system(&mut world);

        // Assert
        assert!(world.resource::<Hand>().is_empty());
    }

    /// @doc: Cards entering the table get physics body—no longer UI-managed, joinable in collisions
    #[test]
    fn when_pick_from_hand_then_physics_body_added() {
        // Arrange
        let mut world = World::new();
        let card_entity = world
            .spawn((
                crate::test_helpers::make_test_card(),
                CardZone::Hand(0),
                default_collider(),
                GlobalTransform2D(Affine2::from_translation(Vec2::new(50.0, 200.0))),
                SortOrder::new(0),
            ))
            .id();
        let mut hand = Hand::new(10);
        hand.add(card_entity).unwrap();
        world.insert_resource(hand);
        let (spy, bodies, _, _) = make_spy_physics();
        world.insert_resource(PhysicsRes::new(spy));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::new(50.0, 200.0));
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());
        world.insert_resource(ReaderDragState::default());
        world.insert_resource(crate::stash::grid::StashGrid::new(10, 10, 1));
        world.insert_resource(crate::stash::toggle::StashVisible(false));

        // Act
        run_system(&mut world);

        // Assert
        let calls = bodies.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, card_entity);
        assert_eq!(calls[0].1, Vec2::new(50.0, 200.0));
    }

    /// @doc: Render layer shifts to World on pick—hand cards drawn over table, picked cards below UI
    #[test]
    fn when_pick_from_hand_then_render_layer_becomes_world() {
        // Arrange
        let mut world = World::new();
        let card_entity = world
            .spawn((
                crate::test_helpers::make_test_card(),
                CardZone::Hand(0),
                default_collider(),
                GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
                RenderLayer::UI,
                SortOrder::new(0),
            ))
            .id();
        let mut hand = Hand::new(10);
        hand.add(card_entity).unwrap();
        world.insert_resource(hand);
        let (spy, _, _, _) = make_spy_physics();
        world.insert_resource(PhysicsRes::new(spy));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::ZERO);
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());
        world.insert_resource(ReaderDragState::default());
        world.insert_resource(crate::stash::grid::StashGrid::new(10, 10, 1));
        world.insert_resource(crate::stash::toggle::StashVisible(false));

        // Act
        run_system(&mut world);

        // Assert
        let layer = world.get::<RenderLayer>(card_entity).unwrap();
        assert_eq!(*layer, RenderLayer::World);
    }

    /// @doc: Hand card priority over table cards—hand cards above table in picking, sort order breaks ties
    #[test]
    fn when_hand_card_and_table_card_overlap_then_highest_sort_wins() {
        // Arrange
        let mut world = World::new();
        let _table_card = world
            .spawn((
                crate::test_helpers::make_test_card(),
                CardZone::Table,
                default_collider(),
                GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
                SortOrder::new(3),
            ))
            .id();
        let hand_card = world
            .spawn((
                Card::face_down(TextureId(3), TextureId(4)),
                CardZone::Hand(0),
                default_collider(),
                GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
                SortOrder::new(10),
            ))
            .id();
        let mut hand = Hand::new(10);
        hand.add(hand_card).unwrap();
        world.insert_resource(hand);
        let (spy, _, _, _) = make_spy_physics();
        world.insert_resource(PhysicsRes::new(spy));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::ZERO);
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());
        world.insert_resource(ReaderDragState::default());
        world.insert_resource(crate::stash::grid::StashGrid::new(10, 10, 1));
        world.insert_resource(crate::stash::toggle::StashVisible(false));

        // Act
        run_system(&mut world);

        // Assert
        let drag = world.resource::<DragState>().dragging;
        assert_eq!(drag.map(|d| d.entity), Some(hand_card));
    }

    #[test]
    fn when_pick_from_hand_then_scale_spring_inserted() {
        // Arrange
        let mut world = World::new();
        let card_entity = world
            .spawn((
                crate::test_helpers::make_test_card(),
                CardZone::Hand(0),
                default_collider(),
                GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
                SortOrder::new(0),
            ))
            .id();
        let mut hand = Hand::new(10);
        hand.add(card_entity).unwrap();
        world.insert_resource(hand);
        let (spy, _, _, _) = make_spy_physics();
        world.insert_resource(PhysicsRes::new(spy));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::ZERO);
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());
        world.insert_resource(ReaderDragState::default());
        world.insert_resource(crate::stash::grid::StashGrid::new(10, 10, 1));
        world.insert_resource(crate::stash::toggle::StashVisible(false));

        // Act
        run_system(&mut world);

        // Assert
        let spring = world.get::<ScaleSpring>(card_entity);
        assert!(spring.is_some(), "ScaleSpring should be inserted");
        assert_eq!(spring.unwrap().target, DRAG_SCALE);
    }

    #[test]
    fn when_pick_from_hand_then_handspring_removed() {
        // Arrange
        let mut world = World::new();
        let card_entity = world
            .spawn((
                crate::test_helpers::make_test_card(),
                CardZone::Hand(0),
                default_collider(),
                GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
                SortOrder::new(0),
                HandSpring::new(),
            ))
            .id();
        let mut hand = Hand::new(10);
        hand.add(card_entity).unwrap();
        world.insert_resource(hand);
        let (spy, _, _, _) = make_spy_physics();
        world.insert_resource(PhysicsRes::new(spy));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::ZERO);
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());
        world.insert_resource(ReaderDragState::default());
        world.insert_resource(crate::stash::grid::StashGrid::new(10, 10, 1));
        world.insert_resource(crate::stash::toggle::StashVisible(false));

        // Act
        run_system(&mut world);

        // Assert
        assert!(
            world.get::<HandSpring>(card_entity).is_none(),
            "HandSpring should be removed when picking from hand"
        );
    }

    #[test]
    fn when_pick_from_table_then_scale_spring_target_is_drag_elevation() {
        // Arrange
        let mut world = World::new();
        let card_entity = world
            .spawn((
                crate::test_helpers::make_test_card(),
                CardZone::Table,
                default_collider(),
                GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
                SortOrder::new(0),
            ))
            .id();
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::ZERO);
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());
        world.insert_resource(ReaderDragState::default());
        insert_pick_resources(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        let spring = world
            .get::<ScaleSpring>(card_entity)
            .expect("table card should get ScaleSpring on pick");
        assert_eq!(spring.target, DRAG_SCALE);
    }

    #[test]
    fn when_left_click_on_stash_card_then_drag_info_records_stash_origin_and_slot_vacated() {
        // Arrange
        use crate::stash::grid::StashGrid;
        use crate::stash::toggle::StashVisible;

        let mut world = World::new();
        // col=2, row=3 center: x = 20 + 2*54 + 25 = 153, y = 20 + 3*79 + 37 = 294
        let card_entity = world
            .spawn((
                Card::face_down(TextureId(20), TextureId(21)),
                CardZone::Stash {
                    page: 0,
                    col: 2,
                    row: 3,
                },
                Collider::Aabb(Vec2::new(30.0, 45.0)),
                GlobalTransform2D(Affine2::IDENTITY),
                RenderLayer::UI,
                SortOrder::new(0),
            ))
            .id();
        let mut grid = StashGrid::new(4, 5, 1);
        grid.place(0, 2, 3, card_entity).unwrap();
        world.insert_resource(grid);
        world.insert_resource(StashVisible(true));
        world.insert_resource(DragState::default());
        world.insert_resource(ReaderDragState::default());
        world.insert_resource(Hand::new(10));
        world.insert_resource(PhysicsRes::new(Box::new(NullPhysicsBackend::default())));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_screen_pos(Vec2::new(153.0, 294.0));
        world.insert_resource(mouse);

        // Act
        run_system(&mut world);

        // Assert
        let drag = world.resource::<DragState>().dragging.unwrap();
        assert_eq!(drag.entity, card_entity);
        assert_eq!(
            drag.origin_zone,
            CardZone::Stash {
                page: 0,
                col: 2,
                row: 3
            }
        );
        assert!(
            world.resource::<StashGrid>().get(0, 2, 3).is_none(),
            "stash slot should be empty after pick"
        );
    }

    /// @doc: Stash picks preserve UI layer and skip physics—cards stay in cursor-follow mode during drag
    #[test]
    fn when_left_click_on_stash_card_then_no_physics_body_added_and_render_layer_stays_ui() {
        // Arrange
        use crate::stash::grid::StashGrid;
        use crate::stash::toggle::StashVisible;

        let mut world = World::new();
        // col=2, row=3 center: x = 153, y = 20 + 3*79 + 37 = 294
        let card_entity = world
            .spawn((
                Card::face_down(TextureId(30), TextureId(31)),
                CardZone::Stash {
                    page: 0,
                    col: 2,
                    row: 3,
                },
                Collider::Aabb(Vec2::new(30.0, 45.0)),
                GlobalTransform2D(Affine2::IDENTITY),
                RenderLayer::UI,
                SortOrder::new(0),
            ))
            .id();
        let mut grid = StashGrid::new(4, 5, 1);
        grid.place(0, 2, 3, card_entity).unwrap();
        world.insert_resource(grid);
        world.insert_resource(StashVisible(true));
        world.insert_resource(DragState::default());
        world.insert_resource(ReaderDragState::default());
        world.insert_resource(Hand::new(10));
        let (spy, bodies, _, _) = make_spy_physics();
        world.insert_resource(PhysicsRes::new(spy));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_screen_pos(Vec2::new(153.0, 294.0));
        world.insert_resource(mouse);

        // Act
        run_system(&mut world);

        // Assert — stash picks must NOT create physics bodies (cursor-follow mode)
        assert!(
            bodies.lock().unwrap().is_empty(),
            "add_body should NOT be called for stash card (cursor-follow mode)"
        );
        assert_eq!(
            *world.get::<RenderLayer>(card_entity).unwrap(),
            RenderLayer::UI,
            "RenderLayer should stay UI for stash card"
        );
    }

    #[test]
    fn when_stash_hidden_and_slot_clicked_then_pick_not_triggered() {
        // Arrange
        use crate::stash::grid::StashGrid;
        use crate::stash::toggle::StashVisible;

        let mut world = World::new();
        // col=0, row=0 center at (45, 45)
        let card_entity = world
            .spawn((
                crate::test_helpers::make_test_card(),
                CardZone::Stash {
                    page: 0,
                    col: 0,
                    row: 0,
                },
                Collider::Aabb(Vec2::new(30.0, 45.0)),
                // Place far from world cursor (default world_pos is ZERO) so
                // the existing world-space hit test cannot pick this card
                GlobalTransform2D(Affine2::from_translation(Vec2::new(500.0, 500.0))),
                RenderLayer::UI,
                SortOrder::new(0),
            ))
            .id();
        let mut grid = StashGrid::new(4, 5, 1);
        grid.place(0, 0, 0, card_entity).unwrap();
        world.insert_resource(grid);
        world.insert_resource(StashVisible(false));
        world.insert_resource(DragState::default());
        world.insert_resource(ReaderDragState::default());
        world.insert_resource(Hand::new(10));
        world.insert_resource(PhysicsRes::new(Box::new(NullPhysicsBackend::default())));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_screen_pos(Vec2::new(45.0, 45.0));
        world.insert_resource(mouse);

        // Act
        run_system(&mut world);

        // Assert
        assert!(
            world.resource::<DragState>().dragging.is_none(),
            "stash pick should not trigger when stash is hidden"
        );
    }

    #[test]
    fn when_left_click_on_stash_card_then_drag_info_stash_cursor_follow_is_true() {
        // Arrange
        use crate::stash::grid::StashGrid;
        use crate::stash::toggle::StashVisible;

        let mut world = World::new();
        // slot (col=0, row=0) center: x=45.0, y=57.5
        let card_entity = world
            .spawn((
                Card::face_down(TextureId(40), TextureId(41)),
                CardZone::Stash {
                    page: 0,
                    col: 0,
                    row: 0,
                },
                Collider::Aabb(Vec2::new(30.0, 45.0)),
                GlobalTransform2D(Affine2::IDENTITY),
                RenderLayer::UI,
                SortOrder::new(0),
            ))
            .id();
        let mut grid = StashGrid::new(10, 10, 1);
        grid.place(0, 0, 0, card_entity).unwrap();
        world.insert_resource(grid);
        world.insert_resource(StashVisible(true));
        world.insert_resource(DragState::default());
        world.insert_resource(ReaderDragState::default());
        world.insert_resource(Hand::new(10));
        world.insert_resource(PhysicsRes::new(Box::new(NullPhysicsBackend::default())));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_screen_pos(Vec2::new(45.0, 57.5));
        world.insert_resource(mouse);

        // Act
        run_system(&mut world);

        // Assert
        let drag = world.resource::<DragState>().dragging.unwrap();
        assert!(
            drag.stash_cursor_follow,
            "picking from a stash slot must set stash_cursor_follow=true"
        );
    }

    #[test]
    fn when_pick_from_stash_then_scale_spring_target_is_drag_elevation() {
        // Arrange
        use crate::stash::grid::StashGrid;
        use crate::stash::toggle::StashVisible;

        let mut world = World::new();
        let card_entity = world
            .spawn((
                Card::face_down(TextureId(50), TextureId(51)),
                CardZone::Stash {
                    page: 0,
                    col: 0,
                    row: 0,
                },
                Collider::Aabb(Vec2::new(30.0, 45.0)),
                GlobalTransform2D(Affine2::IDENTITY),
                RenderLayer::UI,
                SortOrder::new(0),
            ))
            .id();
        let mut grid = StashGrid::new(10, 10, 1);
        grid.place(0, 0, 0, card_entity).unwrap();
        world.insert_resource(grid);
        world.insert_resource(StashVisible(true));
        world.insert_resource(DragState::default());
        world.insert_resource(ReaderDragState::default());
        world.insert_resource(Hand::new(10));
        world.insert_resource(PhysicsRes::new(Box::new(NullPhysicsBackend::default())));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_screen_pos(Vec2::new(45.0, 57.5));
        world.insert_resource(mouse);

        // Act
        run_system(&mut world);

        // Assert
        let spring = world
            .get::<ScaleSpring>(card_entity)
            .expect("stash card should get ScaleSpring on pick");
        assert_eq!(spring.target, DRAG_SCALE);
    }

    #[test]
    fn when_left_click_on_table_card_then_drag_info_stash_cursor_follow_is_false() {
        // Arrange
        let mut world = World::new();
        world.spawn((
            Card::face_down(TextureId(42), TextureId(43)),
            CardZone::Table,
            default_collider(),
            GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
            SortOrder::new(0),
        ));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::ZERO);
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());
        world.insert_resource(ReaderDragState::default());
        insert_pick_resources(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        let drag = world.resource::<DragState>().dragging.unwrap();
        assert!(
            !drag.stash_cursor_follow,
            "picking a table card must leave stash_cursor_follow=false"
        );
    }

    #[test]
    fn when_table_card_picked_then_origin_position_stored_from_transform() {
        // Arrange
        let mut world = World::new();
        world.spawn((
            crate::test_helpers::make_test_card(),
            CardZone::Table,
            default_collider(),
            GlobalTransform2D(Affine2::from_translation(Vec2::new(100.0, 200.0))),
            SortOrder::new(0),
        ));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::new(100.0, 200.0));
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());
        world.insert_resource(ReaderDragState::default());
        insert_pick_resources(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        let drag = world.resource::<DragState>().dragging.unwrap();
        assert_eq!(drag.origin_position, Vec2::new(100.0, 200.0));
    }
}
