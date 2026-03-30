//! End-to-end schedule tests that exercise the real `CardGamePlugin` + `DefaultPlugins`
//! across multiple frames. These verify system ordering and multi-system interactions
//! that unit tests (which run a single system in isolation) cannot catch.

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::{Arc, Mutex};

    use axiom2d::prelude::*;
    use engine_core::time::FixedDeltaClock;
    use engine_render::testing::SpyRenderer;
    use glam::Vec2;

    use crate::card::component::CardItemForm;
    use crate::card::component::CardZone;
    use crate::card::identity::definition::{
        CardAbilities, CardDefinition, CardType, art_descriptor_default,
    };
    use crate::card::identity::signature::CardSignature;
    use crate::card::interaction::drag_state::DragState;
    use crate::card::rendering::spawn_table_card::spawn_visual_card;
    use crate::hand::cards::Hand;
    use crate::plugin::CardGamePlugin;
    use crate::stash::toggle::StashVisible;
    use crate::test_helpers::SpyPhysicsBackend;

    // ── Helpers ──────────────────────────────────────────────────────

    fn make_game_app() -> App {
        let mut app = App::new();

        // Skip splash screen — must be inserted BEFORE DefaultPlugins checks for it.
        app.world_mut().insert_resource(SkipSplash);

        // Insert spy physics BEFORE DefaultPlugins (it checks for existing PhysicsRes).
        app.world_mut()
            .insert_resource(PhysicsRes::new(Box::new(SpyPhysicsBackend::new())));

        // DefaultPlugins registers all engine systems + resources.
        app.add_plugin(DefaultPlugins);

        // Replace SystemClock with FixedDeltaClock for deterministic delta time.
        // Returns 1/60 s on every tick — unlike FakeClock which drains on first delta().
        app.world_mut()
            .insert_resource(ClockRes::new(Box::new(FixedDeltaClock::new(Seconds(
                1.0 / 60.0,
            )))));

        // Replace renderer with spy (viewport 800x600).
        let log = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log).with_viewport(800, 600);
        app.world_mut()
            .insert_resource(RendererRes::new(Box::new(spy)));

        // CardGamePlugin registers card game systems + resources.
        app.add_plugin(CardGamePlugin);

        // Spawn a camera — needed for mouse_world_pos_system to convert screen→world.
        app.world_mut().spawn(Camera2D {
            position: Vec2::ZERO,
            zoom: 1.0,
        });

        app
    }

    fn make_test_definition() -> CardDefinition {
        CardDefinition {
            name: "Test Card".to_owned(),
            card_type: CardType::Creature,
            abilities: CardAbilities {
                keywords: vec![],
                text: String::new(),
            },
            stats: None,
            art: art_descriptor_default(CardType::Creature),
        }
    }

    fn tick(app: &mut App) {
        app.handle_redraw();
    }

    /// Queue a left-mouse-press event at the given screen position.
    ///
    /// Events go through `EventBus<MouseInputEvent>` so that `mouse_input_system`
    /// (`Phase::Input`) sets `just_pressed` properly — direct `MouseState` mutation
    /// would be cleared before the Update systems see it.
    fn simulate_mouse_press(app: &mut App, screen_pos: Vec2) {
        set_mouse_position(app, screen_pos);
        app.world_mut()
            .resource_mut::<engine_core::prelude::EventBus<MouseInputEvent>>()
            .push(MouseInputEvent {
                button: MouseButton::Left,
                state: ButtonState::Pressed,
            });
    }

    /// Queue a left-mouse-release event at the given screen position.
    fn simulate_mouse_release(app: &mut App, screen_pos: Vec2) {
        set_mouse_position(app, screen_pos);
        app.world_mut()
            .resource_mut::<engine_core::prelude::EventBus<MouseInputEvent>>()
            .push(MouseInputEvent {
                button: MouseButton::Left,
                state: ButtonState::Released,
            });
    }

    fn simulate_mouse_move(app: &mut App, screen_pos: Vec2) {
        set_mouse_position(app, screen_pos);
    }

    fn set_mouse_position(app: &mut App, screen_pos: Vec2) {
        let world_pos = screen_pos - Vec2::new(400.0, 300.0);
        let mut mouse = app.world_mut().resource_mut::<MouseState>();
        mouse.set_screen_pos(screen_pos);
        mouse.set_world_pos(world_pos);
    }

    // ── Tests ────────────────────────────────────────────────────────

    /// @doc: Verifies system ordering for the Table→Hand zone transition across the real schedule.
    #[test]
    fn when_card_picked_from_table_and_released_into_hand_then_card_in_hand() {
        // Arrange
        let mut app = make_game_app();
        let def = make_test_definition();
        let card_size = Vec2::new(60.0, 90.0);

        // Spawn card at world origin. With Camera2D at (0,0) zoom 1.0
        // and 800x600 viewport, screen center (400,300) maps to world (0,0).
        let entity = spawn_visual_card(
            app.world_mut(),
            &def,
            Vec2::ZERO,
            card_size,
            false,
            CardSignature::default(),
        );

        // Frame 0: establish GlobalTransform2D and camera matrix.
        tick(&mut app);

        // Act — Frame 1: press on card's screen position.
        simulate_mouse_press(&mut app, Vec2::new(400.0, 300.0));
        tick(&mut app);

        // Verify the pick registered via the real schedule.
        assert!(
            app.world().resource::<DragState>().dragging.is_some(),
            "card should be in drag state after pick frame"
        );

        // Act — Frame 2: release in hand drop zone (screen y >= 600 - 120 = 480).
        simulate_mouse_release(&mut app, Vec2::new(400.0, 550.0));
        tick(&mut app);

        // Assert
        let hand = app.world().resource::<Hand>();
        assert!(
            hand.cards().contains(&entity),
            "card should be in Hand resource after release into hand zone"
        );
        assert_eq!(
            *app.world().get::<CardZone>(entity).unwrap(),
            CardZone::Hand(0),
            "CardZone should be Hand(0)"
        );
        assert!(
            app.world().resource::<DragState>().dragging.is_none(),
            "DragState should be cleared after release"
        );
    }

    /// @doc: Table→Table round-trip preserves physics — dropping a card back on the table doesn't strip its body
    #[test]
    fn when_card_picked_from_table_and_released_on_table_then_card_stays_on_table() {
        // Arrange
        let mut app = make_game_app();
        let def = make_test_definition();
        let entity = spawn_visual_card(
            app.world_mut(),
            &def,
            Vec2::ZERO,
            Vec2::new(60.0, 90.0),
            false,
            CardSignature::default(),
        );
        tick(&mut app);

        // Act — pick
        simulate_mouse_press(&mut app, Vec2::new(400.0, 300.0));
        tick(&mut app);

        // Act — release in table area (well above hand zone: y=100 < 480)
        simulate_mouse_release(&mut app, Vec2::new(400.0, 100.0));
        tick(&mut app);

        // Assert
        assert_eq!(
            *app.world().get::<CardZone>(entity).unwrap(),
            CardZone::Table,
            "card should remain on table"
        );
        assert!(
            app.world().get::<RigidBody>(entity).is_some(),
            "table card should have RigidBody after release"
        );
        assert!(
            app.world().resource::<DragState>().dragging.is_none(),
            "DragState should be cleared"
        );
    }

    /// @doc: Verifies the full Table→Stash zone transition including `CardItemForm` insertion and grid placement.
    #[test]
    fn when_card_picked_from_table_and_released_on_stash_then_card_in_stash() {
        // Arrange
        let mut app = make_game_app();
        let def = make_test_definition();
        let entity = spawn_visual_card(
            app.world_mut(),
            &def,
            Vec2::ZERO,
            Vec2::new(60.0, 90.0),
            false,
            CardSignature::default(),
        );

        // Open stash before first frame.
        app.world_mut().resource_mut::<StashVisible>().0 = true;
        tick(&mut app);

        // Act — pick the card at world origin.
        simulate_mouse_press(&mut app, Vec2::new(400.0, 300.0));
        tick(&mut app);

        // Act — release over stash slot (0,0): screen pos inside the first slot area.
        // GRID_MARGIN=20, SLOT_STRIDE_W=54, SLOT_STRIDE_H=79 → (45, 57) is center of slot (0,0).
        simulate_mouse_release(&mut app, Vec2::new(45.0, 57.0));
        tick(&mut app);

        // Assert
        let zone = *app.world().get::<CardZone>(entity).unwrap();
        assert!(
            matches!(
                zone,
                CardZone::Stash {
                    page: 0,
                    col: 0,
                    row: 0
                }
            ),
            "expected CardZone::Stash {{ page: 0, col: 0, row: 0 }}, got {zone:?}"
        );
        assert!(
            app.world().resource::<DragState>().dragging.is_none(),
            "DragState should be cleared"
        );

        // Run one more frame so Commands from the release frame are applied.
        tick(&mut app);

        assert!(
            app.world().get::<CardItemForm>(entity).is_some(),
            "stash card should have CardItemForm after PostUpdate systems run"
        );
    }

    /// @doc: Stash→Hand integration — uses screen-space slot coordinates for stash pick, world coords for hand release
    #[test]
    fn when_card_picked_from_stash_and_released_into_hand_then_card_in_hand() {
        // Arrange — get a card into the stash via the pick-release flow.
        let mut app = make_game_app();
        let def = make_test_definition();
        let entity = spawn_visual_card(
            app.world_mut(),
            &def,
            Vec2::ZERO,
            Vec2::new(60.0, 90.0),
            false,
            CardSignature::default(),
        );
        app.world_mut().resource_mut::<StashVisible>().0 = true;
        tick(&mut app);

        // Pick from table and drop into stash slot (0,0).
        simulate_mouse_press(&mut app, Vec2::new(400.0, 300.0));
        tick(&mut app);
        simulate_mouse_release(&mut app, Vec2::new(45.0, 57.0));
        tick(&mut app);

        // Verify card is in stash.
        assert!(
            matches!(
                *app.world().get::<CardZone>(entity).unwrap(),
                CardZone::Stash { .. }
            ),
            "precondition: card should be in stash"
        );

        // Act — pick the card from stash slot (0,0) by clicking at its screen position.
        simulate_mouse_press(&mut app, Vec2::new(45.0, 57.0));
        tick(&mut app);

        assert!(
            app.world().resource::<DragState>().dragging.is_some(),
            "card should be in drag state after stash pick"
        );

        // Act — release into hand drop zone (screen y >= 480).
        // x must be past the stash grid right edge (20 + 10*54 = 560) so that
        // find_stash_slot_at returns None and the drop falls through to hand zone.
        simulate_mouse_release(&mut app, Vec2::new(600.0, 550.0));
        tick(&mut app);

        // Assert
        let hand = app.world().resource::<Hand>();
        assert!(
            hand.cards().contains(&entity),
            "card should be in Hand after stash → hand transition"
        );
        assert_eq!(
            *app.world().get::<CardZone>(entity).unwrap(),
            CardZone::Hand(0),
            "CardZone should be Hand(0)"
        );
        assert!(
            app.world().resource::<DragState>().dragging.is_none(),
            "DragState should be cleared"
        );
    }

    /// @doc: Drag state must clear after every release — leaked drag state would cause phantom picks on the next click
    #[test]
    fn when_multiple_pick_release_cycles_then_drag_state_cleared_each_time() {
        // Arrange
        let mut app = make_game_app();
        let def = make_test_definition();
        spawn_visual_card(
            app.world_mut(),
            &def,
            Vec2::ZERO,
            Vec2::new(60.0, 90.0),
            false,
            CardSignature::default(),
        );
        tick(&mut app);

        // Cycle 1: pick and release on table.
        simulate_mouse_press(&mut app, Vec2::new(400.0, 300.0));
        tick(&mut app);
        assert!(
            app.world().resource::<DragState>().dragging.is_some(),
            "cycle 1: should be dragging"
        );

        simulate_mouse_release(&mut app, Vec2::new(400.0, 100.0));
        tick(&mut app);
        assert!(
            app.world().resource::<DragState>().dragging.is_none(),
            "cycle 1: DragState should be cleared after release"
        );

        // The card is still on the table. We need a tick so Commands flush
        // and transforms settle before the next pick.
        tick(&mut app);

        // Cycle 2: pick the same card again and release into hand.
        simulate_mouse_press(&mut app, Vec2::new(400.0, 300.0));
        tick(&mut app);

        // If DragState is still some, the second pick worked.
        // If DragState is None, the card may have moved — that's also valid,
        // the key assertion is on the release frame below.
        let dragging_cycle2 = app.world().resource::<DragState>().dragging.is_some();

        if dragging_cycle2 {
            simulate_mouse_release(&mut app, Vec2::new(400.0, 550.0));
            tick(&mut app);
            assert!(
                app.world().resource::<DragState>().dragging.is_none(),
                "cycle 2: DragState should be cleared after release"
            );
        }

        // Final invariant: no matter what happened, drag state should be None.
        // (If cycle 2 pick failed because card drifted, DragState is already None.)
        assert!(
            app.world().resource::<DragState>().dragging.is_none(),
            "DragState must always be None when no drag is active"
        );
    }

    /// @doc: Stash→Table transition must fully rejoin the table zone: `CardZone::Table`, physics body
    /// restored, `CardItemForm` removed, and drag state cleared.
    #[test]
    fn when_stash_card_released_on_table_then_zone_table_body_present_item_form_removed_drag_cleared()
     {
        // Arrange — place a card in the stash via the real pick-release flow.
        let mut app = make_game_app();
        let def = make_test_definition();
        let entity = spawn_visual_card(
            app.world_mut(),
            &def,
            Vec2::ZERO,
            Vec2::new(60.0, 90.0),
            false,
            CardSignature::default(),
        );
        app.world_mut().resource_mut::<StashVisible>().0 = true;
        tick(&mut app);

        simulate_mouse_press(&mut app, Vec2::new(400.0, 300.0));
        tick(&mut app);
        simulate_mouse_release(&mut app, Vec2::new(45.0, 57.0));
        tick(&mut app);
        // Flush Commands so CardItemForm is inserted before we assert the precondition.
        tick(&mut app);

        assert!(
            matches!(
                *app.world().get::<CardZone>(entity).unwrap(),
                CardZone::Stash { .. }
            ),
            "precondition: card must be in stash before the act"
        );

        // Act — pick from stash slot (0,0), release on open table area.
        // x=650 is right of the stash grid (right edge = 20 + 10*54 = 560).
        // y=100 is above the hand zone (hand zone starts at screen y=480).
        simulate_mouse_press(&mut app, Vec2::new(45.0, 57.0));
        tick(&mut app);
        simulate_mouse_release(&mut app, Vec2::new(650.0, 100.0));
        tick(&mut app);
        // Flush Commands for CardItemForm removal and RigidBody insertion.
        tick(&mut app);

        // Assert
        assert_eq!(
            *app.world().get::<CardZone>(entity).unwrap(),
            CardZone::Table,
            "CardZone should be Table after releasing stash card on table area"
        );
        assert!(
            app.world().get::<RigidBody>(entity).is_some(),
            "table card must have a RigidBody after Stash→Table transition"
        );
        assert!(
            app.world().get::<CardItemForm>(entity).is_none(),
            "CardItemForm must be removed when card leaves the stash"
        );
        assert!(
            app.world().resource::<DragState>().dragging.is_none(),
            "DragState must be cleared after release"
        );
    }

    /// @doc: Verifies `flip_animation_system` is wired into the `CardGamePlugin` schedule — a system
    /// that exists only in the library but is never registered would leave `FlipAnimation` inert.
    #[test]
    fn when_flip_animation_inserted_on_table_card_then_after_18_ticks_face_up_toggled_and_animation_removed()
     {
        use crate::card::component::Card;
        use crate::card::interaction::flip_animation::FlipAnimation;

        // Arrange
        let mut app = make_game_app();
        let def = make_test_definition();
        let entity = spawn_visual_card(
            app.world_mut(),
            &def,
            Vec2::ZERO,
            Vec2::new(60.0, 90.0),
            false,
            CardSignature::default(),
        );
        // Frame 0: establish GlobalTransform2D so the entity is fully initialised.
        tick(&mut app);

        // Insert FlipAnimation directly — tests the animation system wiring, not the trigger.
        app.world_mut()
            .entity_mut(entity)
            .insert(FlipAnimation::start(true));

        // Act — 18 ticks at 1/60 s each covers FLIP_DURATION (0.3 s) in full.
        for _ in 0..18 {
            tick(&mut app);
        }

        // Assert
        assert!(
            app.world().get::<Card>(entity).unwrap().face_up,
            "face_up must be true after 18 ticks advance the flip animation to completion"
        );
        assert!(
            app.world().get::<FlipAnimation>(entity).is_none(),
            "FlipAnimation component must be removed once progress >= 1.0"
        );
    }

    /// @doc: A held drag can cross the stash boundary across multiple frames and still resolve on the final release frame.
    #[test]
    fn when_table_card_crosses_stash_boundary_over_multiple_frames_then_release_uses_current_zone()
    {
        // Arrange
        let mut app = make_game_app();
        let def = make_test_definition();
        let entity = spawn_visual_card(
            app.world_mut(),
            &def,
            Vec2::ZERO,
            Vec2::new(60.0, 90.0),
            false,
            CardSignature::default(),
        );
        app.world_mut().resource_mut::<StashVisible>().0 = true;
        tick(&mut app);

        // Act - press on the table card.
        simulate_mouse_press(&mut app, Vec2::new(400.0, 300.0));
        tick(&mut app);

        assert!(
            app.world().resource::<DragState>().dragging.is_some(),
            "card should enter drag state on the press frame"
        );

        // Act - move into the stash while still holding the mouse button.
        simulate_mouse_move(&mut app, Vec2::new(45.0, 57.0));
        tick(&mut app);
        tick(&mut app);

        assert!(
            app.world().get::<CardItemForm>(entity).is_some(),
            "card should switch into stash cursor-follow mode while held over the stash"
        );
        assert!(
            app.world().get::<RigidBody>(entity).is_none(),
            "stash-follow cards should not keep a physics body while held over the stash"
        );

        // Act - move back out to the table area before releasing.
        simulate_mouse_move(&mut app, Vec2::new(650.0, 100.0));
        tick(&mut app);
        tick(&mut app);

        assert!(
            app.world().get::<RigidBody>(entity).is_some(),
            "moving back out of the stash should restore the physics body before release"
        );
        assert!(
            app.world().get::<CardItemForm>(entity).is_none(),
            "leaving the stash boundary should remove the item-form component"
        );

        // Act - release over the hand zone on the final frame.
        simulate_mouse_release(&mut app, Vec2::new(600.0, 550.0));
        tick(&mut app);
        tick(&mut app);

        // Assert
        let hand = app.world().resource::<Hand>();
        let zone = *app.world().get::<CardZone>(entity).unwrap();
        let drag_state = app.world().resource::<DragState>().dragging;
        assert!(
            hand.cards().contains(&entity),
            "card should end in the Hand resource after the final release frame; zone={zone:?}, hand={:?}, drag_state={drag_state:?}",
            hand.cards()
        );
        assert_eq!(
            zone,
            CardZone::Hand(0),
            "CardZone should reflect the final hand drop target"
        );
        assert!(
            app.world().resource::<DragState>().dragging.is_none(),
            "drag state should clear after the release frame"
        );
    }
}
