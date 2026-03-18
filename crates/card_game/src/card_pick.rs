use bevy_ecs::prelude::{Commands, Entity, Query, Res, ResMut};
use engine_input::prelude::{MouseButton, MouseState};
use engine_physics::hit_test::{collider_half_extents, local_space_hit};
use engine_physics::prelude::{Collider, PhysicsRes, RigidBody};
use engine_scene::prelude::{GlobalTransform2D, RenderLayer, SortOrder};
use glam::Vec2;

use crate::card::Card;
use crate::card_damping::{BASE_ANGULAR_DRAG, BASE_LINEAR_DRAG};
use crate::card_zone::CardZone;
use crate::drag_state::{DragInfo, DragState};
use crate::hand::Hand;
use crate::hand_layout::HandSpring;
use crate::scale_spring::ScaleSpring;

pub const CARD_COLLISION_GROUP: u32 = 0b0001;
pub const CARD_COLLISION_FILTER: u32 = 0b0010;
pub const DRAGGED_COLLISION_GROUP: u32 = 0;
pub const DRAGGED_COLLISION_FILTER: u32 = 0;

#[allow(clippy::too_many_arguments)]
pub fn card_pick_system(
    mouse: Res<MouseState>,
    mut drag_state: ResMut<DragState>,
    mut hand: ResMut<Hand>,
    mut physics: ResMut<PhysicsRes>,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &Card,
        &CardZone,
        &GlobalTransform2D,
        &Collider,
        &mut SortOrder,
    )>,
) {
    if drag_state.dragging.is_some() {
        return;
    }
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let cursor = mouse.world_pos();
    let max_sort = max_table_sort_order(&query);
    let Some((entity, zone, local_grab_offset, collider)) = find_card_under_cursor(&query, cursor)
    else {
        return;
    };

    if let CardZone::Hand(_) = zone {
        transition_hand_to_table(
            entity,
            &mut hand,
            &mut physics,
            &mut commands,
            &query,
            &collider,
        );
    }

    if matches!(zone, CardZone::Table) {
        physics.set_collision_group(entity, DRAGGED_COLLISION_GROUP, DRAGGED_COLLISION_FILTER);
    }

    drag_state.dragging = Some(DragInfo {
        entity,
        local_grab_offset,
        origin_zone: zone,
    });
    if let Ok((_, _, _, _, _, mut sort)) = query.get_mut(entity) {
        sort.0 = max_sort + 1;
    }
}

fn max_table_sort_order(
    query: &Query<(
        Entity,
        &Card,
        &CardZone,
        &GlobalTransform2D,
        &Collider,
        &mut SortOrder,
    )>,
) -> i32 {
    query
        .iter()
        .filter(|(_, _, zone, _, _, _)| **zone == CardZone::Table)
        .map(|(_, _, _, _, _, sort)| sort.0)
        .max()
        .unwrap_or(0)
}

fn find_card_under_cursor(
    query: &Query<(
        Entity,
        &Card,
        &CardZone,
        &GlobalTransform2D,
        &Collider,
        &mut SortOrder,
    )>,
    cursor: Vec2,
) -> Option<(Entity, CardZone, Vec2, Collider)> {
    query
        .iter()
        .filter(|(_, _, _, transform, collider, _)| {
            let Some(half) = collider_half_extents(collider) else {
                return false;
            };
            let cursor_local = transform.0.inverse().transform_point2(cursor);
            local_space_hit(cursor_local, half)
        })
        .max_by_key(|(_, _, _, _, _, sort)| sort.0)
        .map(|(entity, _, zone, transform, collider, _)| {
            let cursor_delta = cursor - transform.0.translation;
            let local_grab_offset = transform.0.matrix2.inverse().mul_vec2(cursor_delta);
            (entity, *zone, local_grab_offset, collider.clone())
        })
}

#[allow(clippy::too_many_arguments)]
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
        &mut SortOrder,
    )>,
    collider: &Collider,
) {
    hand.remove(entity);
    let position = query
        .get(entity)
        .map(|(_, _, _, t, _, _)| t.0.translation)
        .unwrap_or(Vec2::ZERO);
    physics.add_body(entity, &RigidBody::Dynamic, position);
    physics.add_collider(entity, collider);
    physics.set_damping(entity, BASE_LINEAR_DRAG, BASE_ANGULAR_DRAG);
    physics.set_collision_group(entity, DRAGGED_COLLISION_GROUP, DRAGGED_COLLISION_FILTER);
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
    use engine_core::prelude::{Seconds, TextureId};
    use engine_input::prelude::{MouseButton, MouseState};
    use engine_physics::prelude::{
        Collider, CollisionEvent, NullPhysicsBackend, PhysicsBackend, PhysicsRes, RigidBody,
    };
    use engine_scene::prelude::{GlobalTransform2D, RenderLayer, SortOrder};
    use glam::{Affine2, Vec2};

    use super::card_pick_system;
    use crate::card::Card;
    use crate::card_zone::CardZone;
    use crate::drag_state::DragState;
    use crate::hand::Hand;
    use crate::hand_layout::HandSpring;
    use crate::scale_spring::ScaleSpring;

    fn run_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(card_pick_system);
        schedule.run(world);
    }

    fn default_collider() -> Collider {
        Collider::Aabb(Vec2::new(30.0, 45.0))
    }

    fn insert_pick_resources(world: &mut World) {
        world.insert_resource(Hand::new(10));
        world.insert_resource(PhysicsRes::new(Box::new(NullPhysicsBackend::new())));
    }

    #[test]
    fn when_left_click_on_single_table_card_then_drag_state_contains_that_entity() {
        // Arrange
        let mut world = World::new();
        let card_entity = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Table,
                default_collider(),
                GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
                SortOrder(0),
            ))
            .id();
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::ZERO);
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());
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
            SortOrder(0),
        ));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::new(100.0, 50.0));
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());
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
            SortOrder(0),
        ));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::ZERO);
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());
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
            Card::face_down(TextureId(1), TextureId(2)),
            CardZone::Table,
            default_collider(),
            GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
            SortOrder(0),
        ));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::new(200.0, 200.0));
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());
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
        insert_pick_resources(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        assert!(world.resource::<DragState>().dragging.is_none());
    }

    #[test]
    fn when_two_overlapping_cards_then_picks_highest_sort_order() {
        // Arrange
        let mut world = World::new();
        let _card_a = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Table,
                default_collider(),
                GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
                SortOrder(0),
            ))
            .id();
        let card_b = world
            .spawn((
                Card::face_down(TextureId(3), TextureId(4)),
                CardZone::Table,
                default_collider(),
                GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
                SortOrder(5),
            ))
            .id();
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::ZERO);
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());
        insert_pick_resources(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        let drag = world.resource::<DragState>().dragging;
        assert_eq!(drag.map(|d| d.entity), Some(card_b));
    }

    #[test]
    fn when_card_picked_then_sort_order_bumped_above_all_others() {
        // Arrange
        let mut world = World::new();
        let card_a = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Table,
                default_collider(),
                GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
                SortOrder(0),
            ))
            .id();
        world.spawn((
            Card::face_down(TextureId(3), TextureId(4)),
            CardZone::Table,
            default_collider(),
            GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 0.0))),
            SortOrder(7),
        ));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::ZERO);
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());
        insert_pick_resources(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        let picked_sort = world.entity(card_a).get::<SortOrder>().unwrap().0;
        assert!(
            picked_sort > 7,
            "picked card sort {picked_sort} should be > 7"
        );
    }

    #[test]
    fn when_already_dragging_then_new_click_does_not_replace_drag() {
        // Arrange
        let mut world = World::new();
        let card_a = world.spawn_empty().id();
        world.spawn((
            Card::face_down(TextureId(1), TextureId(2)),
            CardZone::Table,
            default_collider(),
            GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
            SortOrder(0),
        ));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::ZERO);
        world.insert_resource(mouse);
        world.insert_resource(DragState {
            dragging: Some(crate::drag_state::DragInfo {
                entity: card_a,
                local_grab_offset: Vec2::ZERO,
                origin_zone: CardZone::Table,
            }),
        });
        insert_pick_resources(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        let drag = world.resource::<DragState>().dragging.unwrap();
        assert_eq!(drag.entity, card_a);
    }

    #[test]
    fn when_card_picked_at_offset_then_local_grab_offset_is_inverse_rotated() {
        // Arrange
        let mut world = World::new();
        let angle = std::f32::consts::FRAC_PI_4;
        let transform =
            Affine2::from_scale_angle_translation(Vec2::ONE, angle, Vec2::new(100.0, 50.0));
        world.spawn((
            Card::face_down(TextureId(1), TextureId(2)),
            CardZone::Table,
            default_collider(),
            GlobalTransform2D(transform),
            SortOrder(0),
        ));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::new(110.0, 50.0));
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());
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
            Card::face_down(TextureId(1), TextureId(2)),
            CardZone::Table,
            default_collider(),
            GlobalTransform2D(Affine2::from_translation(Vec2::new(100.0, 50.0))),
            SortOrder(0),
        ));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::new(100.0, 50.0));
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());
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
            Card::face_down(TextureId(1), TextureId(2)),
            CardZone::Table,
            default_collider(),
            GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
            SortOrder(0),
        ));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::new(30.0, 0.0));
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());
        insert_pick_resources(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        assert!(world.resource::<DragState>().dragging.is_some());
    }

    #[test]
    fn when_rotated_card_clicked_inside_obb_then_picked() {
        // Arrange
        let mut world = World::new();
        let angle = std::f32::consts::FRAC_PI_4;
        world.spawn((
            Card::face_down(TextureId(1), TextureId(2)),
            CardZone::Table,
            default_collider(),
            GlobalTransform2D(Affine2::from_scale_angle_translation(
                Vec2::ONE,
                angle,
                Vec2::ZERO,
            )),
            SortOrder(0),
        ));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::new(20.0, 20.0));
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());
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
            Card::face_down(TextureId(1), TextureId(2)),
            CardZone::Table,
            default_collider(),
            GlobalTransform2D(Affine2::from_scale_angle_translation(
                Vec2::ONE,
                angle,
                Vec2::ZERO,
            )),
            SortOrder(0),
        ));
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::new(50.0, 0.0));
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());
        insert_pick_resources(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        assert!(world.resource::<DragState>().dragging.is_none());
    }

    // --- Hand pick tests ---

    type BodyLog = Arc<Mutex<Vec<(Entity, Vec2)>>>;
    type ColliderLog = Arc<Mutex<Vec<Entity>>>;
    type DampingLog = Arc<Mutex<Vec<(Entity, f32, f32)>>>;

    struct SpyPhysicsBackend {
        bodies: BodyLog,
        colliders: ColliderLog,
        dampings: DampingLog,
    }

    impl SpyPhysicsBackend {
        fn new(bodies: BodyLog, colliders: ColliderLog, dampings: DampingLog) -> Self {
            Self {
                bodies,
                colliders,
                dampings,
            }
        }
    }

    impl PhysicsBackend for SpyPhysicsBackend {
        fn step(&mut self, _dt: Seconds) {}
        fn add_body(&mut self, entity: Entity, _body_type: &RigidBody, position: Vec2) -> bool {
            self.bodies.lock().unwrap().push((entity, position));
            true
        }
        fn add_collider(&mut self, entity: Entity, _collider: &Collider) -> bool {
            self.colliders.lock().unwrap().push(entity);
            true
        }
        fn remove_body(&mut self, _: Entity) {}
        fn body_position(&self, _: Entity) -> Option<Vec2> {
            None
        }
        fn body_rotation(&self, _: Entity) -> Option<f32> {
            None
        }
        fn drain_collision_events(&mut self) -> Vec<CollisionEvent> {
            Vec::new()
        }
        fn body_linear_velocity(&self, _: Entity) -> Option<Vec2> {
            None
        }
        fn set_linear_velocity(&mut self, _: Entity, _: Vec2) {}
        fn set_angular_velocity(&mut self, _: Entity, _: f32) {}
        fn add_force_at_point(&mut self, _: Entity, _: Vec2, _: Vec2) {}
        fn body_angular_velocity(&self, _: Entity) -> Option<f32> {
            None
        }
        fn set_damping(&mut self, entity: Entity, linear: f32, angular: f32) {
            self.dampings
                .lock()
                .unwrap()
                .push((entity, linear, angular));
        }
        fn set_collision_group(&mut self, _: Entity, _: u32, _: u32) {}
    }

    fn make_spy_physics() -> (
        Box<dyn PhysicsBackend + Send + Sync>,
        BodyLog,
        ColliderLog,
        DampingLog,
    ) {
        let bodies: BodyLog = Arc::new(Mutex::new(Vec::new()));
        let colliders: ColliderLog = Arc::new(Mutex::new(Vec::new()));
        let dampings: DampingLog = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyPhysicsBackend::new(bodies.clone(), colliders.clone(), dampings.clone());
        (Box::new(spy), bodies, colliders, dampings)
    }

    #[test]
    fn when_left_click_on_hand_card_then_drag_origin_is_hand() {
        // Arrange
        let mut world = World::new();
        let card_entity = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Hand(0),
                default_collider(),
                GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
                SortOrder(0),
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
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Hand(0),
                default_collider(),
                GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
                SortOrder(0),
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

        // Act
        run_system(&mut world);

        // Assert
        assert!(world.resource::<Hand>().is_empty());
    }

    #[test]
    fn when_pick_from_hand_then_physics_body_added() {
        // Arrange
        let mut world = World::new();
        let card_entity = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Hand(0),
                default_collider(),
                GlobalTransform2D(Affine2::from_translation(Vec2::new(50.0, 200.0))),
                SortOrder(0),
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

        // Act
        run_system(&mut world);

        // Assert
        let calls = bodies.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, card_entity);
        assert_eq!(calls[0].1, Vec2::new(50.0, 200.0));
    }

    #[test]
    fn when_pick_from_hand_then_render_layer_becomes_world() {
        // Arrange
        let mut world = World::new();
        let card_entity = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Hand(0),
                default_collider(),
                GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
                RenderLayer::UI,
                SortOrder(0),
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

        // Act
        run_system(&mut world);

        // Assert
        let layer = world.get::<RenderLayer>(card_entity).unwrap();
        assert_eq!(*layer, RenderLayer::World);
    }

    #[test]
    fn when_hand_card_and_table_card_overlap_then_highest_sort_wins() {
        // Arrange
        let mut world = World::new();
        let _table_card = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Table,
                default_collider(),
                GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
                SortOrder(3),
            ))
            .id();
        let hand_card = world
            .spawn((
                Card::face_down(TextureId(3), TextureId(4)),
                CardZone::Hand(0),
                default_collider(),
                GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
                SortOrder(10),
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
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Hand(0),
                default_collider(),
                GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
                SortOrder(0),
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

        // Act
        run_system(&mut world);

        // Assert
        let spring = world.get::<ScaleSpring>(card_entity);
        assert!(spring.is_some(), "ScaleSpring should be inserted");
        assert_eq!(spring.unwrap().target, 1.0);
    }

    #[test]
    fn when_pick_from_hand_then_handspring_removed() {
        // Arrange
        let mut world = World::new();
        let card_entity = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Hand(0),
                default_collider(),
                GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
                SortOrder(0),
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

        // Act
        run_system(&mut world);

        // Assert
        assert!(
            world.get::<HandSpring>(card_entity).is_none(),
            "HandSpring should be removed when picking from hand"
        );
    }

    #[test]
    fn when_pick_from_table_then_no_scale_spring() {
        // Arrange
        let mut world = World::new();
        let card_entity = world
            .spawn((
                Card::face_down(TextureId(1), TextureId(2)),
                CardZone::Table,
                default_collider(),
                GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
                SortOrder(0),
            ))
            .id();
        let mut mouse = MouseState::default();
        mouse.press(MouseButton::Left);
        mouse.set_world_pos(Vec2::ZERO);
        world.insert_resource(mouse);
        world.insert_resource(DragState::default());
        insert_pick_resources(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        assert!(
            world.get::<ScaleSpring>(card_entity).is_none(),
            "table cards should not get ScaleSpring"
        );
    }
}
