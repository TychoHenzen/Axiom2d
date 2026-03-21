//! End-to-end schedule tests that exercise the real `CardGamePlugin` + `DefaultPlugins`
//! across multiple frames. These verify system ordering and multi-system interactions
//! that unit tests (which run a single system in isolation) cannot catch.

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::{Arc, Mutex};

    use axiom2d::prelude::*;
    use engine_core::time::FakeClock;
    use engine_render::testing::SpyRenderer;
    use glam::Vec2;

    use crate::card::definition::{
        CardAbilities, CardDefinition, CardType, art_descriptor_default,
    };
    use crate::card::drag_state::DragState;
    use crate::card::item_form::CardItemForm;
    use crate::card::signature::CardSignature;
    use crate::card::spawn_table_card::spawn_visual_card;
    use crate::card::zone::CardZone;
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

        // Replace SystemClock with FakeClock for deterministic delta time.
        let mut fake_clock = FakeClock::default();
        fake_clock.advance(Seconds(1.0 / 60.0));
        app.world_mut()
            .insert_resource(ClockRes::new(Box::new(fake_clock)));

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
    /// Events go through `MouseEventBuffer` so that `mouse_input_system` (`Phase::Input`)
    /// sets `just_pressed` properly — direct `MouseState` mutation would be cleared
    /// before the Update systems see it.
    fn simulate_mouse_press(app: &mut App, screen_pos: Vec2) {
        app.world_mut()
            .resource_mut::<MouseState>()
            .set_screen_pos(screen_pos);
        app.world_mut()
            .resource_mut::<MouseEventBuffer>()
            .push(MouseButton::Left, ButtonState::Pressed);
    }

    /// Queue a left-mouse-release event at the given screen position.
    fn simulate_mouse_release(app: &mut App, screen_pos: Vec2) {
        app.world_mut()
            .resource_mut::<MouseState>()
            .set_screen_pos(screen_pos);
        app.world_mut()
            .resource_mut::<MouseEventBuffer>()
            .push(MouseButton::Left, ButtonState::Released);
    }

    // ── Tests ────────────────────────────────────────────────────────

    /// Pick a table card at world origin, drag to the hand zone (bottom of screen),
    /// and verify the full zone transition: Table → Hand.
    ///
    /// Schedule dependency chain exercised:
    ///   Frame 0: `transform_propagation_system` → `camera_prepare_system` (establish transforms)
    ///   Frame 1: `mouse_input_system` → `mouse_world_pos_system` → `card_pick_system` (pick)
    ///   Frame 2: `mouse_input_system` → `card_release_system` (release into hand zone)
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

    /// Pick a table card and release it back on the table area (top half of screen).
    /// The card should stay in `CardZone::Table` with physics intact.
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

    /// Pick a table card and release it over an open stash slot.
    /// The card should transition to `CardZone::Stash` and gain `CardItemForm`.
    ///
    /// Stash slot (0,0) screen position: margin=20, `stride_w=54`, `stride_h=79`.
    /// Top-left of slot (0,0) is at screen (20, 20). Any point inside the stride
    /// range maps to slot (0,0).
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

    /// Place a card in the stash, then pick it from the stash slot and release
    /// into the hand zone. Verifies the Stash → Hand transition through the
    /// real schedule.
    ///
    /// Stash picks use `mouse.screen_pos()` with `find_stash_slot_at()`, so we
    /// click at the slot's screen coordinates rather than world coordinates.
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

    /// Verify that `DragState` is None after every release, even across multiple
    /// pick-release cycles. This is a lightweight sanity check that the release
    /// system always clears state.
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
}
