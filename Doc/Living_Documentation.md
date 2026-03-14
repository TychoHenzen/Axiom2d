# Axiom2d Living Documentation

> Auto-generated from 641 test cases. Last updated: 2026-03-14.

<details>
<summary><strong>axiom2d</strong> (15 tests)</summary>

<blockquote>
<details>
<summary><strong>test default_plugins</strong> (15 tests)</summary>

<blockquote>
<details>
<summary>✅ When audio feature on, then audio res is present</summary>

<code>crates\axiom2d\src\default_plugins.rs:383</code>

```rust
        // Arrange
        let app = app_with_default_plugins();

        // Assert
        assert!(
            app.world()
                .get_resource::<engine_audio::audio_res::AudioRes>()
                .is_some()
        );
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When audio feature on, then play sound buffer is present</summary>

<code>crates\axiom2d\src\default_plugins.rs:369</code>

```rust
        // Arrange
        let app = app_with_default_plugins();

        // Assert
        assert!(
            app.world()
                .get_resource::<engine_audio::play_sound_buffer::PlaySoundBuffer>()
                .is_some()
        );
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When mouse button pressed and two frames run, then just pressed cleared</summary>

<code>crates\axiom2d\src\default_plugins.rs:415</code>

```rust
        // Arrange
        let mut app = app_with_default_plugins();
        app.world_mut()
            .resource_mut::<engine_input::prelude::MouseEventBuffer>()
            .push(winit::event::MouseButton::Left, ElementState::Pressed);
        app.handle_redraw();

        // Act
        app.handle_redraw();

        // Assert
        let mouse = app.world().resource::<engine_input::prelude::MouseState>();
        assert!(!mouse.just_pressed(winit::event::MouseButton::Left));
        assert!(mouse.pressed(winit::event::MouseButton::Left));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When atlas inserted and frame runs, then upload atlas called</summary>

<code>crates\axiom2d\src\default_plugins.rs:294</code>

```rust
        // Arrange
        let log = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let spy = engine_render::testing::SpyRenderer::new(std::sync::Arc::clone(&log));

        let mut app = app_with_default_plugins();
        app.world_mut()
            .insert_resource(engine_render::prelude::RendererRes::new(Box::new(spy)));
        app.world_mut()
            .insert_resource(engine_render::prelude::TextureAtlas {
                data: vec![255; 4],
                width: 1,
                height: 1,
                lookups: std::collections::HashMap::default(),
            });

        // Act
        app.handle_redraw();

        // Assert
        let calls = log.lock().unwrap();
        assert!(calls.contains(&"upload_atlas".to_string()));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When renderer injected and frame runs, then clear called</summary>

<code>crates\axiom2d\src\default_plugins.rs:208</code>

```rust
        // Arrange
        let log = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let spy = engine_render::testing::SpyRenderer::new(std::sync::Arc::clone(&log));

        let mut app = app_with_default_plugins();
        app.world_mut()
            .insert_resource(engine_render::prelude::RendererRes::new(Box::new(spy)));

        // Act
        app.handle_redraw();

        // Assert
        let calls = log.lock().unwrap();
        assert!(calls.contains(&"clear".to_string()));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When shape entity exists and frame runs, then draw shape called</summary>

<code>crates\axiom2d\src\default_plugins.rs:262</code>

```rust
        use engine_core::prelude::Color;
        use engine_render::prelude::{RendererRes, Shape, ShapeVariant};
        use engine_scene::prelude::GlobalTransform2D;
        use glam::Affine2;

        // Arrange
        let log = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let spy = engine_render::testing::SpyRenderer::new(std::sync::Arc::clone(&log))
            .with_viewport(800, 600);

        let mut app = app_with_default_plugins();
        app.world_mut()
            .insert_resource(RendererRes::new(Box::new(spy)));
        app.world_mut().spawn((
            Shape {
                variant: ShapeVariant::Circle { radius: 10.0 },
                color: Color::WHITE,
            },
            GlobalTransform2D(Affine2::IDENTITY),
        ));

        // Act
        app.handle_redraw();

        // Assert
        let calls = log.lock().unwrap();
        assert!(calls.iter().any(|c| c.starts_with("draw_shape")));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When mouse button pressed and frame runs, then mouse state reflects button</summary>

<code>crates\axiom2d\src\default_plugins.rs:396</code>

```rust
        // Arrange
        let mut app = app_with_default_plugins();
        app.world_mut()
            .resource_mut::<engine_input::prelude::MouseEventBuffer>()
            .push(winit::event::MouseButton::Left, ElementState::Pressed);

        // Act
        app.handle_redraw();

        // Assert
        assert!(
            app.world()
                .resource::<engine_input::prelude::MouseState>()
                .just_pressed(winit::event::MouseButton::Left)
        );
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When entity has transform2d, then global transform set after frame</summary>

<code>crates\axiom2d\src\default_plugins.rs:162</code>

```rust
        use engine_core::prelude::{Transform2D, Vec2};

        // Arrange
        let mut app = app_with_default_plugins();
        let entity = app
            .world_mut()
            .spawn(Transform2D {
                position: Vec2::new(100.0, 200.0),
                ..Default::default()
            })
            .id();

        // Act
        app.handle_redraw();

        // Assert
        let global = app
            .world()
            .get::<engine_scene::prelude::GlobalTransform2D>(entity)
            .expect("GlobalTransform2D should be set");
        assert_eq!(global.0.translation, Vec2::new(100.0, 200.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When key pressed and second frame runs, then just pressed is false</summary>

<code>crates\axiom2d\src\default_plugins.rs:433</code>

```rust
        // Arrange
        let mut app = app_with_default_plugins();
        app.world_mut()
            .resource_mut::<InputEventBuffer>()
            .push(KeyCode::Space, ElementState::Pressed);
        app.handle_redraw(); // first frame — just_pressed should be true

        // Act
        app.handle_redraw(); // second frame — no new events

        // Assert
        assert!(
            !app.world()
                .resource::<InputState>()
                .just_pressed(KeyCode::Space)
        );
        assert!(app.world().resource::<InputState>().pressed(KeyCode::Space));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When child of entity spawned, then children component created after frame</summary>

<code>crates\axiom2d\src\default_plugins.rs:143</code>

```rust
        // Arrange
        let mut app = app_with_default_plugins();
        let parent = app.world_mut().spawn_empty().id();
        app.world_mut()
            .spawn(engine_scene::prelude::ChildOf(parent));

        // Act
        app.handle_redraw();

        // Assert
        assert!(
            app.world()
                .get::<engine_scene::prelude::Children>(parent)
                .is_some()
        );
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When atlas uploaded, then draw sprite also called same frame</summary>

<code>crates\axiom2d\src\default_plugins.rs:320</code>

```rust
        use engine_core::prelude::{Color, Pixels, TextureId};
        use engine_render::prelude::{RendererRes, Sprite};
        use engine_scene::prelude::GlobalTransform2D;
        use glam::Affine2;

        // Arrange
        let log = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let spy = engine_render::testing::SpyRenderer::new(std::sync::Arc::clone(&log))
            .with_viewport(800, 600);

        let mut app = app_with_default_plugins();
        app.world_mut()
            .insert_resource(RendererRes::new(Box::new(spy)));
        app.world_mut()
            .insert_resource(engine_render::prelude::TextureAtlas {
                data: vec![255; 4],
                width: 1,
                height: 1,
                lookups: std::collections::HashMap::default(),
            });
        app.world_mut().spawn((
            Sprite {
                texture: TextureId(0),
                uv_rect: [0.0, 0.0, 1.0, 1.0],
                color: Color::WHITE,
                width: Pixels(32.0),
                height: Pixels(32.0),
            },
            GlobalTransform2D(Affine2::IDENTITY),
        ));

        // Act
        app.handle_redraw();

        // Assert
        let calls = log.lock().unwrap();
        let upload_idx = calls.iter().position(|c| c == "upload_atlas");
        let sprite_idx = calls.iter().position(|c| c == "draw_sprite");
        assert!(upload_idx.is_some(), "upload_atlas should be called");
        assert!(sprite_idx.is_some(), "draw_sprite should be called");
        assert!(
            upload_idx.unwrap() < sprite_idx.unwrap(),
            "upload_atlas should run before draw_sprite"
        );
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When entity has visible false, then effective visibility false after frame</summary>

<code>crates\axiom2d\src\default_plugins.rs:187</code>

```rust
        // Arrange
        let mut app = app_with_default_plugins();
        let entity = app
            .world_mut()
            .spawn(engine_scene::prelude::Visible(false))
            .id();

        // Act
        app.handle_redraw();

        // Assert
        let eff = app
            .world()
            .get::<engine_scene::prelude::EffectiveVisibility>(entity)
            .expect("EffectiveVisibility should be set");
        assert!(!eff.0);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When key pressed and frame runs, then input state reflects key</summary>

<code>crates\axiom2d\src\default_plugins.rs:107</code>

```rust
        // Arrange
        let mut app = app_with_default_plugins();
        app.world_mut()
            .resource_mut::<InputEventBuffer>()
            .push(KeyCode::Space, ElementState::Pressed);

        // Act
        app.handle_redraw();

        // Assert
        assert!(
            app.world()
                .resource::<InputState>()
                .just_pressed(KeyCode::Space)
        );
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When frame advanced with fake clock, then delta time is updated</summary>

<code>crates\axiom2d\src\default_plugins.rs:126</code>

```rust
        // Arrange
        let mut app = app_with_default_plugins();
        let mut fake = FakeClock::new();
        fake.advance(engine_core::prelude::Seconds(0.016));
        app.world_mut()
            .insert_resource(ClockRes::new(Box::new(fake)));

        // Act
        app.handle_redraw();

        // Assert
        let dt = app.world().resource::<engine_core::prelude::DeltaTime>();
        assert_eq!(dt.0, engine_core::prelude::Seconds(0.016));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When sprite entity exists and frame runs, then draw sprite called</summary>

<code>crates\axiom2d\src\default_plugins.rs:227</code>

```rust
        use engine_core::prelude::{Color, Pixels, TextureId};
        use engine_render::prelude::{RendererRes, Sprite};
        use engine_scene::prelude::GlobalTransform2D;
        use glam::Affine2;

        // Arrange
        let log = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let spy = engine_render::testing::SpyRenderer::new(std::sync::Arc::clone(&log))
            .with_viewport(800, 600);

        let mut app = app_with_default_plugins();
        app.world_mut()
            .insert_resource(RendererRes::new(Box::new(spy)));
        app.world_mut().spawn((
            Sprite {
                texture: TextureId(0),
                uv_rect: [0.0, 0.0, 1.0, 1.0],
                color: Color::WHITE,
                width: Pixels(32.0),
                height: Pixels(32.0),
            },
            GlobalTransform2D(Affine2::IDENTITY),
        ));

        // Act
        app.handle_redraw();

        // Assert
        let calls = log.lock().unwrap();
        assert!(calls.iter().any(|c| c.starts_with("draw_sprite")));
```

</details>
</blockquote>

</details>
</blockquote>

</details>

<details>
<summary><strong>demo</strong> (0 tests)</summary>

</details>

<details>
<summary><strong>engine_app</strong> (39 tests)</summary>

<blockquote>
<details>
<summary><strong>test app</strong> (35 tests)</summary>

<blockquote>
<details>
<summary>✅ When add plugin chained twice, then does not panic</summary>

<code>crates\engine_app\src\app.rs:257</code>

```rust
        // Arrange
        let mut app = App::new();

        // Act
        app.add_plugin(NoOpPlugin).add_plugin(AnotherNoOpPlugin);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When app new called, then window size resource is present</summary>

<code>crates\engine_app\src\app.rs:595</code>

```rust
        // Act
        let app = App::new();

        // Assert
        let size = app.world().get_resource::<WindowSize>();
        assert!(size.is_some());
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When app receives keyboard press, then event pushed to buffer</summary>

<code>crates\engine_app\src\app.rs:670</code>

```rust
        // Arrange
        let mut app = App::new();
        app.world_mut().insert_resource(InputEventBuffer::default());

        // Act
        app.handle_key_event(PhysicalKey::Code(KeyCode::ArrowLeft), ElementState::Pressed);

        // Assert
        let mut buffer = app.world_mut().resource_mut::<InputEventBuffer>();
        let events: Vec<_> = buffer.drain().collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], (KeyCode::ArrowLeft, ElementState::Pressed));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When app created, then renderer res not yet in world</summary>

<code>crates\engine_app\src\app.rs:570</code>

```rust
        // Act
        let app = App::new();

        // Assert
        assert!(app.world().get_resource::<RendererRes>().is_none());
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When app new called, then plugin count is zero</summary>

<code>crates\engine_app\src\app.rs:238</code>

```rust
        // Act
        let app = App::new();

        // Assert
        assert_eq!(app.plugin_count(), 0);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When app receives keyboard release, then release event pushed to buffer</summary>

<code>crates\engine_app\src\app.rs:686</code>

```rust
        // Arrange
        let mut app = App::new();
        app.world_mut().insert_resource(InputEventBuffer::default());

        // Act
        app.handle_key_event(
            PhysicalKey::Code(KeyCode::ArrowLeft),
            ElementState::Released,
        );

        // Assert
        let mut buffer = app.world_mut().resource_mut::<InputEventBuffer>();
        let events: Vec<_> = buffer.drain().collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], (KeyCode::ArrowLeft, ElementState::Released));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When app new called, then delta time resource is present</summary>

<code>crates\engine_app\src\app.rs:623</code>

```rust
        // Act
        let app = App::new();

        // Assert
        assert!(
            app.world()
                .get_resource::<engine_core::time::DeltaTime>()
                .is_some()
        );
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When app receives unidentified physical key, then buffer remains empty</summary>

<code>crates\engine_app\src\app.rs:705</code>

```rust
        use winit::keyboard::NativeKeyCode;

        // Arrange
        let mut app = App::new();
        app.world_mut().insert_resource(InputEventBuffer::default());

        // Act
        app.handle_key_event(
            PhysicalKey::Unidentified(NativeKeyCode::Unidentified),
            ElementState::Pressed,
        );

        // Assert
        let mut buffer = app.world_mut().resource_mut::<InputEventBuffer>();
        assert_eq!(buffer.drain().count(), 0);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When cursor moved event received by app, then screen pos updated</summary>

<code>crates\engine_app\src\app.rs:724</code>

```rust
        // Arrange
        let mut app = App::new();
        app.world_mut()
            .insert_resource(engine_input::prelude::MouseState::default());

        // Act
        app.handle_cursor_moved(glam::Vec2::new(320.0, 240.0));

        // Assert
        let mouse = app.world().resource::<engine_input::prelude::MouseState>();
        assert_eq!(mouse.screen_pos(), glam::Vec2::new(320.0, 240.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When cursor moved without mouse state resource, then does not panic</summary>

<code>crates\engine_app\src\app.rs:739</code>

```rust
        // Arrange
        let mut app = App::new();

        // Act
        app.handle_cursor_moved(glam::Vec2::new(100.0, 100.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When add systems chained, then builder pattern works</summary>

<code>crates\engine_app\src\app.rs:437</code>

```rust
        fn noop() {}

        // Act
        let mut app = App::new();
        app.set_window_config(WindowConfig::default())
            .add_systems(Phase::Update, noop);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When handle redraw called, then pre update runs before update</summary>

*Phase execution order is fixed: Input -> `PreUpdate` -> Update -> `PostUpdate` -> Render*

<code>crates\engine_app\src\app.rs:637</code>

```rust
        use engine_core::time::{ClockRes, DeltaTime, FakeClock, time_system};

        #[derive(Resource)]
        struct CapturedDelta(engine_core::types::Seconds);

        fn capture_delta(
            dt: engine_ecs::prelude::Res<DeltaTime>,
            mut captured: ResMut<CapturedDelta>,
        ) {
            captured.0 = dt.0;
        }

        // Arrange
        let mut app = App::new();
        let mut fake = FakeClock::new();
        fake.advance(engine_core::types::Seconds(0.016));
        app.world_mut()
            .insert_resource(ClockRes::new(Box::new(fake)));
        app.world_mut()
            .insert_resource(CapturedDelta(engine_core::types::Seconds(0.0)));
        app.add_systems(Phase::PreUpdate, time_system);
        app.add_systems(Phase::Update, capture_delta);

        // Act
        app.handle_redraw();

        // Assert
        let captured = app.world().resource::<CapturedDelta>();
        assert_eq!(captured.0, engine_core::types::Seconds(0.016));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When handle redraw called, then present called via renderer res</summary>

<code>crates\engine_app\src\app.rs:343</code>

```rust
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(Arc::clone(&log));

        let mut app = App::new();
        app.set_renderer(Box::new(spy));

        // Act
        app.handle_redraw();

        // Assert
        assert_eq!(log.lock().unwrap().as_slice(), &["present"]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When app default called, then plugin count is zero</summary>

<code>crates\engine_app\src\app.rs:315</code>

```rust
        // Act
        let app = App::default();

        // Assert
        assert_eq!(app.plugin_count(), 0);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When handle redraw called twice, then system runs twice</summary>

<code>crates\engine_app\src\app.rs:382</code>

```rust
        // Arrange
        let mut app = App::new();
        app.world_mut().insert_resource(Counter(0));
        app.add_systems(Phase::Update, increment);

        // Act
        app.handle_redraw();
        app.handle_redraw();

        // Assert
        assert_eq!(app.world().resource::<Counter>().0, 2);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When handle redraw called without renderer res, then does not panic</summary>

<code>crates\engine_app\src\app.rs:359</code>

```rust
        // Arrange
        let mut app = App::new();

        // Act
        app.handle_redraw();
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When handle resize called, then renderer resize is called</summary>

<code>crates\engine_app\src\app.rs:579</code>

```rust
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(Arc::clone(&log));

        let mut app = App::new();
        app.set_renderer(Box::new(spy));

        // Act
        app.handle_resize(1024, 768);

        // Assert
        assert_eq!(log.lock().unwrap().as_slice(), &["resize"]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When handle resize called, then window size resource is updated</summary>

*Resize updates both the `WindowSize` resource and calls `renderer.resize()` — dual sync*

<code>crates\engine_app\src\app.rs:606</code>

```rust
        // Arrange
        let mut app = App::new();
        let log = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(Arc::clone(&log));
        app.set_renderer(Box::new(spy));

        // Act
        app.handle_resize(1024, 768);

        // Assert
        let size = app.world().resource::<WindowSize>();
        assert_eq!(size.width, Pixels(1024.0));
        assert_eq!(size.height, Pixels(768.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When mouse button event received without buffer resource, then does not panic</summary>

<code>crates\engine_app\src\app.rs:770</code>

```rust
        // Arrange
        let mut app = App::new();

        // Act
        app.handle_mouse_button(winit::event::MouseButton::Left, ElementState::Pressed);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When new app created, then five schedules exist</summary>

<code>crates\engine_app\src\app.rs:447</code>

```rust
        // Act
        let app = App::new();

        // Assert
        assert_eq!(app.schedule_count(), 5);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When mouse button event received by app, then event pushed to buffer</summary>

<code>crates\engine_app\src\app.rs:748</code>

```rust
        // Arrange
        let mut app = App::new();
        app.world_mut()
            .insert_resource(engine_input::prelude::MouseEventBuffer::default());

        // Act
        app.handle_mouse_button(winit::event::MouseButton::Right, ElementState::Pressed);

        // Assert
        let mut buffer = app
            .world_mut()
            .resource_mut::<engine_input::prelude::MouseEventBuffer>();
        let events: Vec<_> = buffer.drain().collect();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            (winit::event::MouseButton::Right, ElementState::Pressed)
        );
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When plugin added, then build called exactly once</summary>

<code>crates\engine_app\src\app.rs:300</code>

```rust
        // Arrange
        let counter = Rc::new(Cell::new(0u32));
        let plugin = CountingPlugin {
            counter: Rc::clone(&counter),
        };

        // Act
        App::new().add_plugin(plugin);

        // Assert
        assert_eq!(counter.get(), 1);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When one plugin added, then plugin count is one</summary>

<code>crates\engine_app\src\app.rs:266</code>

```rust
        // Arrange
        let mut app = App::new();

        // Act
        app.add_plugin(NoOpPlugin);

        // Assert
        assert_eq!(app.plugin_count(), 1);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When plugin calls add systems, then system runs during handle redraw</summary>

<code>crates\engine_app\src\app.rs:512</code>

```rust
        struct CounterPlugin;
        impl Plugin for CounterPlugin {
            fn build(&self, app: &mut App) {
                app.world_mut().insert_resource(Counter(0));
                app.add_systems(Phase::Update, increment);
            }
        }

        // Arrange
        let mut app = App::new();
        app.add_plugin(CounterPlugin);

        // Act
        app.handle_redraw();

        // Assert
        assert_eq!(app.world().resource::<Counter>().0, 1);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When plugin inserts resource, then resource persists after build</summary>

<code>crates\engine_app\src\app.rs:533</code>

```rust
        #[derive(Resource)]
        struct Gravity(f32);

        struct GravityPlugin;
        impl Plugin for GravityPlugin {
            fn build(&self, app: &mut App) {
                app.world_mut().insert_resource(Gravity(9.81));
            }
        }

        // Arrange
        let mut app = App::new();

        // Act
        app.add_plugin(GravityPlugin);

        // Assert
        let g = app.world().resource::<Gravity>().0;
        assert!((g - 9.81).abs() < f32::EPSILON);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When resource inserted into app world, then value is readable</summary>

<code>crates\engine_app\src\app.rs:456</code>

```rust
        #[derive(Resource)]
        struct Score(u32);

        // Arrange
        let mut app = App::new();

        // Act
        app.world_mut().insert_resource(Score(7));
        let result = app.world().resource::<Score>().0;

        // Assert
        assert_eq!(result, 7);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When scroll event received by app, then mouse state scroll delta accumulated</summary>

<code>crates\engine_app\src\app.rs:779</code>

```rust
        // Arrange
        let mut app = App::new();
        app.world_mut()
            .insert_resource(engine_input::prelude::MouseState::default());

        // Act
        app.handle_mouse_wheel(glam::Vec2::new(0.0, 3.0));

        // Assert
        let mouse = app.world().resource::<engine_input::prelude::MouseState>();
        assert_eq!(mouse.scroll_delta(), glam::Vec2::new(0.0, 3.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When render phase system uses renderer res, then draw calls precede present</summary>

<code>crates\engine_app\src\app.rs:472</code>

```rust
        fn render_system(mut renderer: ResMut<RendererRes>) {
            renderer.clear(Color::BLACK);
        }

        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(Arc::clone(&log));

        let mut app = App::new();
        app.add_systems(Phase::Render, render_system);
        app.set_renderer(Box::new(spy));

        // Act
        app.handle_redraw();

        // Assert
        assert_eq!(log.lock().unwrap().as_slice(), &["clear", "present"]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When set window config called, then config is stored</summary>

<code>crates\engine_app\src\app.rs:324</code>

```rust
        // Arrange
        let mut app = App::new();
        let config = WindowConfig {
            title: "Test",
            width: 800,
            height: 600,
            vsync: false,
            resizable: false,
        };

        // Act
        app.set_window_config(config);

        // Assert
        assert_eq!(app.window_config, config);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When set renderer called, then renderer res present in world</summary>

<code>crates\engine_app\src\app.rs:556</code>

```rust
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(Arc::clone(&log));
        let mut app = App::new();

        // Act
        app.set_renderer(Box::new(spy));

        // Assert
        assert!(app.world().get_resource::<RendererRes>().is_some());
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When set window config called, then window size reflects config</summary>

<code>crates\engine_app\src\app.rs:794</code>

```rust
        // Arrange
        let mut app = App::new();
        let config = WindowConfig {
            width: 1920,
            height: 1080,
            ..WindowConfig::default()
        };

        // Act
        app.set_window_config(config);

        // Assert
        let size = app.world().resource::<WindowSize>();
        assert_eq!(size.width, Pixels(1920.0));
        assert_eq!(size.height, Pixels(1080.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When two distinct plugins added, then plugin count is two</summary>

<code>crates\engine_app\src\app.rs:278</code>

```rust
        // Arrange
        let mut app = App::new();

        // Act
        app.add_plugin(NoOpPlugin).add_plugin(AnotherNoOpPlugin);

        // Assert
        assert_eq!(app.plugin_count(), 2);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When system added to update phase, then runs during handle redraw</summary>

<code>crates\engine_app\src\app.rs:368</code>

```rust
        // Arrange
        let mut app = App::new();
        app.world_mut().insert_resource(Counter(0));
        app.add_systems(Phase::Update, increment);

        // Act
        app.handle_redraw();

        // Assert
        assert_eq!(app.world().resource::<Counter>().0, 1);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When update systems exist, then schedules run and present called</summary>

<code>crates\engine_app\src\app.rs:493</code>

```rust
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(Arc::clone(&log));

        let mut app = App::new();
        app.world_mut().insert_resource(Counter(0));
        app.add_systems(Phase::Update, increment);
        app.set_renderer(Box::new(spy));

        // Act
        app.handle_redraw();

        // Assert
        assert_eq!(app.world().resource::<Counter>().0, 1);
        assert_eq!(log.lock().unwrap().as_slice(), &["present"]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When systems in all phases, then run in canonical order</summary>

<code>crates\engine_app\src\app.rs:397</code>

```rust
        #[derive(Resource, Default)]
        struct Log(Vec<&'static str>);

        fn log_input(mut log: ResMut<Log>) {
            log.0.push("input");
        }
        fn log_pre_update(mut log: ResMut<Log>) {
            log.0.push("pre_update");
        }
        fn log_update(mut log: ResMut<Log>) {
            log.0.push("update");
        }
        fn log_post_update(mut log: ResMut<Log>) {
            log.0.push("post_update");
        }
        fn log_render(mut log: ResMut<Log>) {
            log.0.push("render");
        }

        // Arrange
        let mut app = App::new();
        app.world_mut().insert_resource(Log::default());
        app.add_systems(Phase::Input, log_input);
        app.add_systems(Phase::PreUpdate, log_pre_update);
        app.add_systems(Phase::Update, log_update);
        app.add_systems(Phase::PostUpdate, log_post_update);
        app.add_systems(Phase::Render, log_render);

        // Act
        app.handle_redraw();

        // Assert
        assert_eq!(
            app.world().resource::<Log>().0,
            vec!["input", "pre_update", "update", "post_update", "render"]
        );
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test mouse_world_pos_system</strong> (4 tests)</summary>

<blockquote>
<details>
<summary>✅ When world pos system runs with camera, then world pos is screen to world converted</summary>

<code>crates\engine_app\src\mouse_world_pos_system.rs:54</code>

```rust
        // Arrange
        let mut world = setup_world(Vec2::new(400.0, 300.0), 800, 600);
        world.spawn(Camera2D::default());

        // Act
        run_system(&mut world);

        // Assert
        let mouse = world.resource::<MouseState>();
        assert_eq!(mouse.world_pos(), Vec2::new(0.0, 0.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When world pos system runs with no camera, then uses default camera</summary>

<code>crates\engine_app\src\mouse_world_pos_system.rs:68</code>

```rust
        // Arrange
        let mut world = setup_world(Vec2::new(400.0, 300.0), 800, 600);

        // Act
        run_system(&mut world);

        // Assert
        let mouse = world.resource::<MouseState>();
        assert_eq!(mouse.world_pos(), Vec2::new(0.0, 0.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When world pos system runs with zoomed camera, then center still maps to camera pos</summary>

<code>crates\engine_app\src\mouse_world_pos_system.rs:81</code>

```rust
        // Arrange
        let mut world = setup_world(Vec2::new(400.0, 300.0), 800, 600);
        world.spawn(Camera2D {
            zoom: 2.0,
            ..Camera2D::default()
        });

        // Act
        run_system(&mut world);

        // Assert
        let mouse = world.resource::<MouseState>();
        assert_eq!(mouse.world_pos(), Vec2::new(0.0, 0.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When world pos system runs with offset cursor and zoom, then world pos is scaled</summary>

<code>crates\engine_app\src\mouse_world_pos_system.rs:98</code>

```rust
        // Arrange
        let mut world = setup_world(Vec2::new(600.0, 300.0), 800, 600);
        world.spawn(Camera2D {
            zoom: 2.0,
            ..Camera2D::default()
        });

        // Act
        run_system(&mut world);

        // Assert
        let mouse = world.resource::<MouseState>();
        assert!((mouse.world_pos().x - 100.0).abs() < 1e-4);
        assert!(mouse.world_pos().y.abs() < 1e-4);
```

</details>
</blockquote>

</details>
</blockquote>

</details>

<details>
<summary><strong>engine_assets</strong> (30 tests)</summary>

<blockquote>
<details>
<summary><strong>test asset_server</strong> (14 tests)</summary>

<blockquote>
<details>
<summary>✅ When getting unknown handle, then returns none</summary>

<code>crates\engine_assets\src\asset_server.rs:131</code>

```rust
        // Arrange
        let server: AssetServer<String> = AssetServer::default();
        let unknown = Handle::<String>::new(99);

        // Act
        let value = server.get(unknown);

        // Assert
        assert_eq!(value, None);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When adding asset, then returns handle with id zero</summary>

<code>crates\engine_assets\src\asset_server.rs:92</code>

```rust
        // Arrange
        let mut server: AssetServer<String> = AssetServer::default();

        // Act
        let handle = server.add("hello".to_string());

        // Assert
        assert_eq!(handle.id, 0);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When adding second asset, then returns different handle</summary>

<code>crates\engine_assets\src\asset_server.rs:104</code>

```rust
        // Arrange
        let mut server: AssetServer<String> = AssetServer::default();

        // Act
        let first = server.add("hello".to_string());
        let second = server.add("world".to_string());

        // Assert
        assert_ne!(first, second);
        assert_eq!(second.id, 1);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When getting by handle, then returns stored value</summary>

<code>crates\engine_assets\src\asset_server.rs:118</code>

```rust
        // Arrange
        let mut server = AssetServer::default();
        let handle = server.add("hello".to_string());

        // Act
        let value = server.get(handle);

        // Assert
        assert_eq!(value, Some(&"hello".to_string()));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When getting mut, then mutation is visible on next get</summary>

<code>crates\engine_assets\src\asset_server.rs:144</code>

```rust
        // Arrange
        let mut server = AssetServer::default();
        let handle = server.add("hello".to_string());

        // Act
        if let Some(v) = server.get_mut(handle) {
            *v = "world".to_string();
        }

        // Assert
        assert_eq!(server.get(handle), Some(&"world".to_string()));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When clone handle called, then ref count increments</summary>

<code>crates\engine_assets\src\asset_server.rs:171</code>

```rust
        // Arrange
        let mut server = AssetServer::default();
        let handle = server.add("hello".to_string());

        // Act
        server.clone_handle(handle);

        // Assert
        assert_eq!(server.ref_count(handle), Some(2));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When loading nonexistent file, then returns io error</summary>

<code>crates\engine_assets\src\asset_server.rs:263</code>

```rust
        // Arrange
        let mut server: AssetServer<String> = AssetServer::default();

        // Act
        let result = server.load("/no/such/file.ron");

        // Assert
        assert!(matches!(result, Err(AssetError::Io(_))));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When asset added, then ref count is one</summary>

<code>crates\engine_assets\src\asset_server.rs:159</code>

```rust
        // Arrange
        let mut server = AssetServer::default();

        // Act
        let handle = server.add("hello".to_string());

        // Assert
        assert_eq!(server.ref_count(handle), Some(1));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When remove with ref count above one, then decrements without evicting</summary>

<code>crates\engine_assets\src\asset_server.rs:184</code>

```rust
        // Arrange
        let mut server = AssetServer::default();
        let handle = server.add("hello".to_string());
        server.clone_handle(handle);

        // Act
        let removed = server.remove(handle);

        // Assert
        assert!(removed);
        assert_eq!(server.ref_count(handle), Some(1));
        assert_eq!(server.get(handle), Some(&"hello".to_string()));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When remove with ref count one, then evicts asset</summary>

<code>crates\engine_assets\src\asset_server.rs:200</code>

```rust
        // Arrange
        let mut server = AssetServer::default();
        let handle = server.add("hello".to_string());

        // Act
        let removed = server.remove(handle);

        // Assert
        assert!(removed);
        assert_eq!(server.get(handle), None);
        assert_eq!(server.ref_count(handle), None);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When remove unknown handle, then returns false</summary>

<code>crates\engine_assets\src\asset_server.rs:215</code>

```rust
        // Arrange
        let mut server: AssetServer<String> = AssetServer::default();
        let unknown = Handle::<String>::new(42);

        // Act
        let removed = server.remove(unknown);

        // Assert
        assert!(!removed);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When loading valid ron file, then returns handle to deserialized value</summary>

<code>crates\engine_assets\src\asset_server.rs:292</code>

```rust
        // Arrange
        let dir = std::env::temp_dir().join("axiom2d_test_tc013");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("data.ron");
        std::fs::write(&path, "[1, 2, 3]").unwrap();
        let mut server: AssetServer<Vec<i32>> = AssetServer::default();

        // Act
        let handle = server.load(path.to_str().unwrap()).unwrap();

        // Assert
        assert_eq!(server.get(handle), Some(&vec![1, 2, 3]));
        let _ = std::fs::remove_dir_all(&dir);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When loading invalid ron, then returns parse error</summary>

<code>crates\engine_assets\src\asset_server.rs:275</code>

```rust
        // Arrange
        let dir = std::env::temp_dir().join("axiom2d_test_tc012");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("bad.ron");
        std::fs::write(&path, "this is not valid RON {{{").unwrap();
        let mut server: AssetServer<Vec<i32>> = AssetServer::default();

        // Act
        let result = server.load(path.to_str().unwrap());

        // Assert
        assert!(matches!(result, Err(AssetError::Parse(_))));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn when_loading_valid_ron_file_then_returns_handle_to_deserialized_value() {
        // Arrange
        let dir = std::env::temp_dir().join("axiom2d_test_tc013");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("data.ron");
        std::fs::write(&path, "[1, 2, 3]").unwrap();
        let mut server: AssetServer<Vec<i32>> = AssetServer::default();

        // Act
        let handle = server.load(path.to_str().unwrap()).unwrap();

        // Assert
        assert_eq!(server.get(handle), Some(&vec![1, 2, 3]));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When cloned n times, then ref count lifecycle is correct</summary>

<code>crates\engine_assets\src\asset_server.rs:229</code>

```rust
            clone_count in 1usize..=5,
        ) {
            // Arrange
            let mut server: AssetServer<String> = AssetServer::default();
            let handle = server.add("test".to_string());
            for _ in 0..clone_count {
                server.clone_handle(handle);
            }
            let expected_initial = 1 + clone_count;
            assert_eq!(server.ref_count(handle), Some(expected_initial));

            // Act — remove (clone_count) times, asset should still exist
            for k in 0..clone_count {
                assert!(server.remove(handle));
                assert_eq!(
                    server.ref_count(handle),
                    Some(expected_initial - 1 - k),
                    "after {} removes",
                    k + 1
                );
            }

            // Act — final remove evicts
            assert!(server.remove(handle));
            assert_eq!(server.ref_count(handle), None);
            assert_eq!(server.get(handle), None);

            // Act — extra remove returns false
            assert!(!server.remove(handle));
        }
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test handle</strong> (4 tests)</summary>

<blockquote>
<details>
<summary>✅ When different ids, then btreeset orders by id</summary>

<code>crates\engine_assets\src\handle.rs:90</code>

```rust
        // Arrange
        let lower = Handle::<u32>::new(1);
        let higher = Handle::<u32>::new(2);
        let mut set = BTreeSet::new();

        // Act
        set.insert(higher);
        set.insert(lower);
        let ordered: Vec<u32> = set.iter().map(|h| h.id).collect();

        // Assert
        assert_ne!(lower, higher);
        assert_eq!(ordered, vec![1, 2]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When different ids, then hash values differ</summary>

<code>crates\engine_assets\src\handle.rs:120</code>

```rust
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Arrange
        let a = Handle::<u32>::new(1);
        let b = Handle::<u32>::new(2);

        // Act
        let hash_a = {
            let mut h = DefaultHasher::new();
            a.hash(&mut h);
            h.finish()
        };
        let hash_b = {
            let mut h = DefaultHasher::new();
            b.hash(&mut h);
            h.finish()
        };

        // Assert
        assert_ne!(hash_a, hash_b);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When partial cmp called, then returns some ordering</summary>

<code>crates\engine_assets\src\handle.rs:107</code>

```rust
        // Arrange
        let a = Handle::<u32>::new(1);
        let b = Handle::<u32>::new(2);

        // Act
        let result = a.partial_cmp(&b);

        // Assert
        assert_eq!(result, Some(std::cmp::Ordering::Less));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When same id, then hashmap deduplicates</summary>

<code>crates\engine_assets\src\handle.rs:73</code>

```rust
        // Arrange
        let a = Handle::<u32>::new(42);
        let b = Handle::<u32>::new(42);
        let mut map: HashMap<Handle<u32>, &str> = HashMap::new();

        // Act
        map.insert(a, "first");
        map.insert(b, "second");

        // Assert
        assert_eq!(map.len(), 1);
        assert_eq!(map[&a], "second");
        assert!(format!("{a:?}").contains("42"));
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test scene</strong> (12 tests)</summary>

<blockquote>
<details>
<summary>✅ When invalid ron deserialized as scene def, then returns error</summary>

<code>crates\engine_assets\src\scene.rs:141</code>

```rust
        // Arrange
        let bad_ron = "{ nodes: [ { broken }";

        // Act
        let result = ron::from_str::<SceneDef>(bad_ron);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn when_minimal_scene_def_serialized_to_pretty_ron_then_snapshot_matches() {
        // Arrange
        let scene = SceneDef {
            nodes: vec![
                minimal_node("parent"),
                SceneNodeDef {
                    parent_index: Some(0),
                    ..minimal_node("child")
                },
            ],
        };

        // Act
        let pretty = ron::ser::to_string_pretty(&scene, ron::ser::PrettyConfig::default()).unwrap();

        // Assert
        insta::assert_snapshot!(pretty);
    }

    #[test]
    fn when_scene_node_def_with_all_fields_serialized_to_pretty_ron_then_snapshot_matches() {
        // Arrange
        let node = SceneNodeDef {
            name: "full".to_owned(),
            transform: Transform2D {
                position: Vec2::new(10.0, 20.0),
                rotation: 1.0,
                scale: Vec2::splat(2.0),
            },
            parent_index: Some(0),
            visible: Some(Visible(true)),
            render_layer: Some(RenderLayer::Foreground),
            sort_order: Some(SortOrder(3)),
            sprite: Some(Sprite {
                texture: TextureId(1),
                uv_rect: [0.0, 0.0, 1.0, 1.0],
                color: Color::RED,
                width: Pixels(16.0),
                height: Pixels(16.0),
            }),
            shape: Some(Shape {
                variant: ShapeVariant::Circle { radius: 10.0 },
                color: Color::BLUE,
            }),
            camera: Some(Camera2D {
                position: Vec2::new(50.0, 50.0),
                zoom: 1.5,
            }),
            rigid_body: Some(RigidBody::Dynamic),
            collider: Some(Collider::Circle(5.0)),
            material: Some(Material2d::default()),
            bloom_settings: Some(BloomSettings::default()),
            audio_emitter: Some(AudioEmitter {
                volume: 0.9,
                max_distance: 300.0,
            }),
        };
        let scene = SceneDef { nodes: vec![node] };

        // Act
        let pretty = ron::ser::to_string_pretty(&scene, ron::ser::PrettyConfig::default()).unwrap();

        // Assert
        insta::assert_snapshot!(pretty);
    }

    #[test]
    fn when_scene_with_convex_polygon_collider_serialized_to_pretty_ron_then_snapshot_matches() {
        // Arrange
        let node = SceneNodeDef {
            collider: Some(Collider::ConvexPolygon(vec![
                Vec2::new(0.0, 10.0),
                Vec2::new(10.0, 0.0),
                Vec2::new(0.0, -10.0),
                Vec2::new(-10.0, 0.0),
            ])),
            ..minimal_node("diamond")
        };
        let scene = SceneDef { nodes: vec![node] };

        // Act
        let pretty = ron::ser::to_string_pretty(&scene, ron::ser::PrettyConfig::default()).unwrap();

        // Assert
        insta::assert_snapshot!(pretty);
    }

    #[test]
    fn when_scene_with_shape_variants_serialized_to_pretty_ron_then_snapshot_matches() {
        // Arrange
        let scene = SceneDef {
            nodes: vec![
                SceneNodeDef {
                    shape: Some(Shape {
                        variant: ShapeVariant::Circle { radius: 25.0 },
                        color: Color::RED,
                    }),
                    ..minimal_node("circle_entity")
                },
                SceneNodeDef {
                    shape: Some(Shape {
                        variant: ShapeVariant::Polygon {
                            points: vec![
                                Vec2::new(0.0, 0.0),
                                Vec2::new(50.0, 0.0),
                                Vec2::new(25.0, 43.3),
                            ],
                        },
                        color: Color::GREEN,
                    }),
                    ..minimal_node("triangle_entity")
                },
            ],
        };

        // Act
        let pretty = ron::ser::to_string_pretty(&scene, ron::ser::PrettyConfig::default()).unwrap();

        // Assert
        insta::assert_snapshot!(pretty);
    }

    #[test]
    fn when_scene_with_audio_emitter_serialized_to_pretty_ron_then_snapshot_matches() {
        // Arrange
        let node = SceneNodeDef {
            audio_emitter: Some(AudioEmitter {
                volume: 0.75,
                max_distance: 500.0,
            }),
            ..minimal_node("speaker")
        };
        let scene = SceneDef { nodes: vec![node] };

        // Act
        let pretty = ron::ser::to_string_pretty(&scene, ron::ser::PrettyConfig::default()).unwrap();

        // Assert
        insta::assert_snapshot!(pretty);
    }

    #[test]
    fn when_scene_with_material_serialized_to_pretty_ron_then_snapshot_matches() {
        // Arrange
        let node = SceneNodeDef {
            material: Some(Material2d {
                blend_mode: BlendMode::Additive,
                shader: ShaderHandle(3),
                textures: vec![
                    TextureBinding {
                        texture: TextureId(0),
                        binding: 0,
                    },
                    TextureBinding {
                        texture: TextureId(1),
                        binding: 1,
                    },
                ],
                uniforms: vec![64, 128, 255],
            }),
            ..minimal_node("glowing")
        };
        let scene = SceneDef { nodes: vec![node] };

        // Act
        let pretty = ron::ser::to_string_pretty(&scene, ron::ser::PrettyConfig::default()).unwrap();

        // Assert
        insta::assert_snapshot!(pretty);
    }

    #[test]
    fn when_scene_def_with_all_component_types_roundtrips_then_all_fields_survive() {
        // Arrange
        let node = SceneNodeDef {
            name: "full".to_owned(),
            transform: Transform2D {
                position: Vec2::new(10.0, 20.0),
                rotation: 1.0,
                scale: Vec2::splat(2.0),
            },
            visible: Some(Visible(true)),
            render_layer: Some(RenderLayer::Foreground),
            sort_order: Some(SortOrder(3)),
            sprite: Some(Sprite {
                texture: TextureId(1),
                uv_rect: [0.0, 0.0, 1.0, 1.0],
                color: Color::RED,
                width: Pixels(16.0),
                height: Pixels(16.0),
            }),
            shape: Some(Shape {
                variant: ShapeVariant::Circle { radius: 10.0 },
                color: Color::BLUE,
            }),
            camera: Some(Camera2D {
                position: Vec2::new(50.0, 50.0),
                zoom: 1.5,
            }),
            rigid_body: Some(RigidBody::Dynamic),
            collider: Some(Collider::Circle(5.0)),
            material: Some(Material2d::default()),
            bloom_settings: Some(BloomSettings::default()),
            audio_emitter: Some(AudioEmitter {
                volume: 0.9,
                max_distance: 300.0,
            }),
            ..Default::default()
        };
        let scene = SceneDef { nodes: vec![node] };

        // Act
        let ron = ron::to_string(&scene).unwrap();
        let back: SceneDef = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(scene, back);
    }
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When scene node def serialized to ron, then deserializes to equal value</summary>

<code>crates\engine_assets\src\scene.rs:61</code>

```rust
        // Arrange
        let node = SceneNodeDef {
            name: "player".to_owned(),
            transform: Transform2D {
                position: Vec2::new(100.0, 200.0),
                rotation: 0.5,
                scale: Vec2::ONE,
            },
            render_layer: Some(RenderLayer::Characters),
            sort_order: Some(SortOrder(5)),
            ..minimal_node("player")
        };

        // Act
        let ron = ron::to_string(&node).unwrap();
        let back: SceneNodeDef = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(node, back);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When scene def with parent child serialized, then parent index is preserved</summary>

<code>crates\engine_assets\src\scene.rs:118</code>

```rust
        // Arrange
        let scene = SceneDef {
            nodes: vec![
                minimal_node("parent"),
                SceneNodeDef {
                    parent_index: Some(0),
                    ..minimal_node("child")
                },
            ],
        };

        // Act
        let ron = ron::to_string(&scene).unwrap();
        let back: SceneDef = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(back.nodes.len(), 2);
        assert_eq!(back.nodes[0].parent_index, None);
        assert_eq!(back.nodes[1].parent_index, Some(0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When scene node with none sprite serialized, then roundtrips as none</summary>

<code>crates\engine_assets\src\scene.rs:84</code>

```rust
        // Arrange
        let node = minimal_node("empty");

        // Act
        let ron = ron::to_string(&node).unwrap();
        let back: SceneNodeDef = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(back.sprite, None);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When scene def with all component types roundtrips, then all fields survive</summary>

<code>crates\engine_assets\src\scene.rs:325</code>

```rust
        // Arrange
        let node = SceneNodeDef {
            name: "full".to_owned(),
            transform: Transform2D {
                position: Vec2::new(10.0, 20.0),
                rotation: 1.0,
                scale: Vec2::splat(2.0),
            },
            visible: Some(Visible(true)),
            render_layer: Some(RenderLayer::Foreground),
            sort_order: Some(SortOrder(3)),
            sprite: Some(Sprite {
                texture: TextureId(1),
                uv_rect: [0.0, 0.0, 1.0, 1.0],
                color: Color::RED,
                width: Pixels(16.0),
                height: Pixels(16.0),
            }),
            shape: Some(Shape {
                variant: ShapeVariant::Circle { radius: 10.0 },
                color: Color::BLUE,
            }),
            camera: Some(Camera2D {
                position: Vec2::new(50.0, 50.0),
                zoom: 1.5,
            }),
            rigid_body: Some(RigidBody::Dynamic),
            collider: Some(Collider::Circle(5.0)),
            material: Some(Material2d::default()),
            bloom_settings: Some(BloomSettings::default()),
            audio_emitter: Some(AudioEmitter {
                volume: 0.9,
                max_distance: 300.0,
            }),
            ..Default::default()
        };
        let scene = SceneDef { nodes: vec![node] };

        // Act
        let ron = ron::to_string(&scene).unwrap();
        let back: SceneDef = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(scene, back);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When scene node with some sprite serialized, then roundtrips with matching fields</summary>

<code>crates\engine_assets\src\scene.rs:97</code>

```rust
        // Arrange
        let sprite = Sprite {
            texture: TextureId(3),
            uv_rect: [0.0, 0.0, 1.0, 1.0],
            color: Color::WHITE,
            width: Pixels(32.0),
            height: Pixels(32.0),
        };
        let mut node = minimal_node("hero");
        node.sprite = Some(sprite);

        // Act
        let ron = ron::to_string(&node).unwrap();
        let back: SceneNodeDef = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(back.sprite, Some(sprite));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When minimal scene def serialized to pretty ron, then snapshot matches</summary>

<code>crates\engine_assets\src\scene.rs:153</code>

```rust
        // Arrange
        let scene = SceneDef {
            nodes: vec![
                minimal_node("parent"),
                SceneNodeDef {
                    parent_index: Some(0),
                    ..minimal_node("child")
                },
            ],
        };

        // Act
        let pretty = ron::ser::to_string_pretty(&scene, ron::ser::PrettyConfig::default()).unwrap();

        // Assert
        insta::assert_snapshot!(pretty);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When scene node def with all fields serialized to pretty ron, then snapshot matches</summary>

<code>crates\engine_assets\src\scene.rs:173</code>

```rust
        // Arrange
        let node = SceneNodeDef {
            name: "full".to_owned(),
            transform: Transform2D {
                position: Vec2::new(10.0, 20.0),
                rotation: 1.0,
                scale: Vec2::splat(2.0),
            },
            parent_index: Some(0),
            visible: Some(Visible(true)),
            render_layer: Some(RenderLayer::Foreground),
            sort_order: Some(SortOrder(3)),
            sprite: Some(Sprite {
                texture: TextureId(1),
                uv_rect: [0.0, 0.0, 1.0, 1.0],
                color: Color::RED,
                width: Pixels(16.0),
                height: Pixels(16.0),
            }),
            shape: Some(Shape {
                variant: ShapeVariant::Circle { radius: 10.0 },
                color: Color::BLUE,
            }),
            camera: Some(Camera2D {
                position: Vec2::new(50.0, 50.0),
                zoom: 1.5,
            }),
            rigid_body: Some(RigidBody::Dynamic),
            collider: Some(Collider::Circle(5.0)),
            material: Some(Material2d::default()),
            bloom_settings: Some(BloomSettings::default()),
            audio_emitter: Some(AudioEmitter {
                volume: 0.9,
                max_distance: 300.0,
            }),
        };
        let scene = SceneDef { nodes: vec![node] };

        // Act
        let pretty = ron::ser::to_string_pretty(&scene, ron::ser::PrettyConfig::default()).unwrap();

        // Assert
        insta::assert_snapshot!(pretty);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When scene with material serialized to pretty ron, then snapshot matches</summary>

<code>crates\engine_assets\src\scene.rs:295</code>

```rust
        // Arrange
        let node = SceneNodeDef {
            material: Some(Material2d {
                blend_mode: BlendMode::Additive,
                shader: ShaderHandle(3),
                textures: vec![
                    TextureBinding {
                        texture: TextureId(0),
                        binding: 0,
                    },
                    TextureBinding {
                        texture: TextureId(1),
                        binding: 1,
                    },
                ],
                uniforms: vec![64, 128, 255],
            }),
            ..minimal_node("glowing")
        };
        let scene = SceneDef { nodes: vec![node] };

        // Act
        let pretty = ron::ser::to_string_pretty(&scene, ron::ser::PrettyConfig::default()).unwrap();

        // Assert
        insta::assert_snapshot!(pretty);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When scene with convex polygon collider serialized to pretty ron, then snapshot matches</summary>

<code>crates\engine_assets\src\scene.rs:220</code>

```rust
        // Arrange
        let node = SceneNodeDef {
            collider: Some(Collider::ConvexPolygon(vec![
                Vec2::new(0.0, 10.0),
                Vec2::new(10.0, 0.0),
                Vec2::new(0.0, -10.0),
                Vec2::new(-10.0, 0.0),
            ])),
            ..minimal_node("diamond")
        };
        let scene = SceneDef { nodes: vec![node] };

        // Act
        let pretty = ron::ser::to_string_pretty(&scene, ron::ser::PrettyConfig::default()).unwrap();

        // Assert
        insta::assert_snapshot!(pretty);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When scene with shape variants serialized to pretty ron, then snapshot matches</summary>

<code>crates\engine_assets\src\scene.rs:241</code>

```rust
        // Arrange
        let scene = SceneDef {
            nodes: vec![
                SceneNodeDef {
                    shape: Some(Shape {
                        variant: ShapeVariant::Circle { radius: 25.0 },
                        color: Color::RED,
                    }),
                    ..minimal_node("circle_entity")
                },
                SceneNodeDef {
                    shape: Some(Shape {
                        variant: ShapeVariant::Polygon {
                            points: vec![
                                Vec2::new(0.0, 0.0),
                                Vec2::new(50.0, 0.0),
                                Vec2::new(25.0, 43.3),
                            ],
                        },
                        color: Color::GREEN,
                    }),
                    ..minimal_node("triangle_entity")
                },
            ],
        };

        // Act
        let pretty = ron::ser::to_string_pretty(&scene, ron::ser::PrettyConfig::default()).unwrap();

        // Assert
        insta::assert_snapshot!(pretty);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When scene with audio emitter serialized to pretty ron, then snapshot matches</summary>

<code>crates\engine_assets\src\scene.rs:276</code>

```rust
        // Arrange
        let node = SceneNodeDef {
            audio_emitter: Some(AudioEmitter {
                volume: 0.75,
                max_distance: 500.0,
            }),
            ..minimal_node("speaker")
        };
        let scene = SceneDef { nodes: vec![node] };

        // Act
        let pretty = ron::ser::to_string_pretty(&scene, ron::ser::PrettyConfig::default()).unwrap();

        // Assert
        insta::assert_snapshot!(pretty);
```

</details>
</blockquote>

</details>
</blockquote>

</details>

<details>
<summary><strong>engine_audio</strong> (67 tests)</summary>

<blockquote>
<details>
<summary><strong>test audio_backend</strong> (7 tests)</summary>

<blockquote>
<details>
<summary>✅ When play called, then play count increments</summary>

<code>crates\engine_audio\src\audio_backend.rs:66</code>

```rust
        // Arrange
        let mut backend = NullAudioBackend::new();
        let sound = minimal_sound();

        // Act
        backend.play(&sound);

        // Assert
        assert_eq!(backend.play_count(), 1);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When play on track called, then play count increments</summary>

<code>crates\engine_audio\src\audio_backend.rs:123</code>

```rust
        // Arrange
        let mut backend = NullAudioBackend::new();
        let sound = minimal_sound();

        // Act
        backend.play_on_track(&sound, MixerTrack::Music);

        // Assert
        assert_eq!(backend.play_count(), 1);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When set volume called, then does not panic</summary>

<code>crates\engine_audio\src\audio_backend.rs:102</code>

```rust
        // Arrange
        let mut backend = NullAudioBackend::new();

        // Act
        backend.set_volume(0.0);
        backend.set_volume(0.5);
        backend.set_volume(1.0);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When stop called, then does not panic</summary>

<code>crates\engine_audio\src\audio_backend.rs:93</code>

```rust
        // Arrange
        let mut backend = NullAudioBackend::new();

        // Act
        backend.stop(PlaybackId(42));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When set track volume on null backend, then no panic</summary>

<code>crates\engine_audio\src\audio_backend.rs:113</code>

```rust
        // Arrange
        let mut backend = NullAudioBackend::new();

        // Act
        backend.set_track_volume(MixerTrack::Music, 0.5);
        backend.set_track_volume(MixerTrack::Sfx, 0.0);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When play called twice, then ids differ</summary>

<code>crates\engine_audio\src\audio_backend.rs:79</code>

```rust
        // Arrange
        let mut backend = NullAudioBackend::new();
        let sound = minimal_sound();

        // Act
        let id1 = backend.play(&sound);
        let id2 = backend.play(&sound);

        // Assert
        assert_ne!(id1, id2);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When three sounds played, then play count returns three</summary>

<code>crates\engine_audio\src\audio_backend.rs:136</code>

```rust
        // Arrange
        let mut backend = NullAudioBackend::new();
        let sound = minimal_sound();

        // Act
        backend.play(&sound);
        backend.play(&sound);
        backend.play(&sound);

        // Assert
        assert_eq!(backend.play_count(), 3);
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test cpal_backend</strong> (16 tests)</summary>

<blockquote>
<details>
<summary>✅ When global and track volume both half, then output quarter</summary>

*Effective volume = `global_volume` * `track_volume` — multiplicative stacking*

<code>crates\engine_audio\src\cpal_backend.rs:389</code>

```rust
        // Arrange
        let mut track_volumes = [1.0; TRACK_COUNT];
        track_volumes[MixerTrack::Music.index()] = 0.5;
        let mut state = test_state_with_tracks(
            0.5,
            track_volumes,
            vec![active_on_track(1, vec![1.0], MixerTrack::Music)],
        );
        let mut output = vec![0.0; 1];

        // Act
        mix_into(&mut output, &mut state);

        // Assert: 1.0 * 0.5 * 0.5 = 0.25
        assert!((output[0] - 0.25).abs() < f32::EPSILON);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When mix into with two tracks, then per track volume applied</summary>

<code>crates\engine_audio\src\cpal_backend.rs:366</code>

```rust
        // Arrange
        let mut track_volumes = [1.0; TRACK_COUNT];
        track_volumes[MixerTrack::Music.index()] = 0.5;
        let mut state = test_state_with_tracks(
            1.0,
            track_volumes,
            vec![
                active_on_track(1, vec![0.8], MixerTrack::Music),
                active_on_track(2, vec![0.4], MixerTrack::Sfx),
            ],
        );
        let mut output = vec![0.0; 1];

        // Act
        mix_into(&mut output, &mut state);

        // Assert: 0.8 * 0.5 + 0.4 * 1.0 = 0.8
        assert!((output[0] - 0.8).abs() < f32::EPSILON);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When single sound at full volume, then output matches samples</summary>

<code>crates\engine_audio\src\cpal_backend.rs:301</code>

```rust
        // Arrange
        let mut state = test_state(1.0, vec![active(1, vec![0.2, 0.4, 0.6])]);
        let mut output = vec![0.0; 3];

        // Act
        mix_into(&mut output, &mut state);

        // Assert
        assert!((output[0] - 0.2).abs() < f32::EPSILON);
        assert!((output[1] - 0.4).abs() < f32::EPSILON);
        assert!((output[2] - 0.6).abs() < f32::EPSILON);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When single sound at half volume, then output is scaled</summary>

<code>crates\engine_audio\src\cpal_backend.rs:316</code>

```rust
        // Arrange
        let mut state = test_state(0.5, vec![active(1, vec![0.8, 0.8])]);
        let mut output = vec![0.0; 2];

        // Act
        mix_into(&mut output, &mut state);

        // Assert
        assert!((output[0] - 0.4).abs() < f32::EPSILON);
        assert!((output[1] - 0.4).abs() < f32::EPSILON);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When sound partially consumed, then only remaining samples mixed</summary>

<code>crates\engine_audio\src\cpal_backend.rs:441</code>

```rust
        // Arrange — 4 samples, consume 3 in first call, 1 remains
        let mut state = test_state(1.0, vec![active(1, vec![0.1, 0.2, 0.3, 0.4])]);
        let mut output1 = vec![0.0; 3];
        let mut output2 = vec![0.0; 3];

        // Act
        mix_into(&mut output1, &mut state);
        mix_into(&mut output2, &mut state);

        // Assert
        assert!((output2[0] - 0.4).abs() < f32::EPSILON);
        assert!(output2[1].abs() < f32::EPSILON);
        assert!(output2[2].abs() < f32::EPSILON);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When sound shorter than buffer, then removed after last sample</summary>

*Sounds auto-evict when cursor reaches end — no explicit `stop()` needed for one-shots*

<code>crates\engine_audio\src\cpal_backend.rs:349</code>

```rust
        // Arrange
        let mut state = test_state(1.0, vec![active(1, vec![0.5, 0.5])]);
        let mut output = vec![0.0; 4];

        // Act
        mix_into(&mut output, &mut state);

        // Assert
        assert!((output[0] - 0.5).abs() < f32::EPSILON);
        assert!((output[1] - 0.5).abs() < f32::EPSILON);
        assert!((output[2]).abs() < f32::EPSILON);
        assert!((output[3]).abs() < f32::EPSILON);
        assert!(state.active_sounds.is_empty());
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When sound longer than buffer, then cursor advances</summary>

<code>crates\engine_audio\src\cpal_backend.rs:421</code>

```rust
        // Arrange
        let mut state = test_state(1.0, vec![active(1, vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6])]);
        let mut output1 = vec![0.0; 3];
        let mut output2 = vec![0.0; 3];

        // Act
        mix_into(&mut output1, &mut state);
        mix_into(&mut output2, &mut state);

        // Assert
        assert!((output1[0] - 0.1).abs() < f32::EPSILON);
        assert!((output1[1] - 0.2).abs() < f32::EPSILON);
        assert!((output1[2] - 0.3).abs() < f32::EPSILON);
        assert!((output2[0] - 0.4).abs() < f32::EPSILON);
        assert!((output2[1] - 0.5).abs() < f32::EPSILON);
        assert!((output2[2] - 0.6).abs() < f32::EPSILON);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When two active sounds, then output is sum</summary>

*Audio mixing is additive — all active sounds summed into output buffer, scaled by volume*

<code>crates\engine_audio\src\cpal_backend.rs:331</code>

```rust
        // Arrange
        let mut state = test_state(
            1.0,
            vec![active(1, vec![0.3, 0.3]), active(2, vec![0.1, 0.1])],
        );
        let mut output = vec![0.0; 2];

        // Act
        mix_into(&mut output, &mut state);

        // Assert
        assert!((output[0] - 0.4).abs() < f32::EPSILON);
        assert!((output[1] - 0.4).abs() < f32::EPSILON);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When stop called, then sound removed from active list</summary>

<code>crates\engine_audio\src\cpal_backend.rs:274</code>

```rust
        // Arrange
        let mut backend = CpalBackend::new();
        let id = backend.play(&sound_with_samples(vec![0.5]));

        // Act
        backend.stop(id);

        // Assert
        assert_eq!(backend.active_sound_count(), 0);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When play called twice, then ids are unique</summary>

<code>crates\engine_audio\src\cpal_backend.rs:238</code>

```rust
        // Arrange
        let mut backend = CpalBackend::new();
        let sound = minimal_sound();

        // Act
        let id1 = backend.play(&sound);
        let id2 = backend.play(&sound);

        // Assert
        assert_ne!(id1, id2);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When constructed, then volume is one</summary>

<code>crates\engine_audio\src\cpal_backend.rs:229</code>

```rust
        // Act
        let backend = CpalBackend::new();

        // Assert
        assert!((backend.volume() - 1.0).abs() < f32::EPSILON);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When stop with unknown id, then does not panic</summary>

<code>crates\engine_audio\src\cpal_backend.rs:252</code>

```rust
        // Arrange
        let mut backend = CpalBackend::new();

        // Act
        backend.stop(PlaybackId(999));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When set volume called, then volume changes</summary>

<code>crates\engine_audio\src\cpal_backend.rs:458</code>

```rust
        // Arrange
        let mut backend = CpalBackend::new();

        // Act
        backend.set_volume(0.5);

        // Assert
        assert!((backend.volume() - 0.5).abs() < f32::EPSILON);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When set track volume on cpal, then internal state updated</summary>

<code>crates\engine_audio\src\cpal_backend.rs:408</code>

```rust
        // Arrange
        let mut backend = CpalBackend::new();

        // Act
        backend.set_track_volume(MixerTrack::Sfx, 0.6);

        // Assert
        assert!((backend.track_volume(MixerTrack::Sfx) - 0.6).abs() < f32::EPSILON);
        assert!((backend.track_volume(MixerTrack::Music) - 1.0).abs() < f32::EPSILON);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When play called, then active sound added</summary>

<code>crates\engine_audio\src\cpal_backend.rs:261</code>

```rust
        // Arrange
        let mut backend = CpalBackend::new();
        let sound = sound_with_samples(vec![0.5, 0.5]);

        // Act
        let _id = backend.play(&sound);

        // Assert
        assert_eq!(backend.active_sound_count(), 1);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When two sounds and stop one, then other remains</summary>

<code>crates\engine_audio\src\cpal_backend.rs:287</code>

```rust
        // Arrange
        let mut backend = CpalBackend::new();
        let id1 = backend.play(&sound_with_samples(vec![0.5]));
        let _id2 = backend.play(&sound_with_samples(vec![0.3]));

        // Act
        backend.stop(id1);

        // Assert
        assert_eq!(backend.active_sound_count(), 1);
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test mixer</strong> (4 tests)</summary>

<blockquote>
<details>
<summary>✅ When default mixer state, then all tracks are one</summary>

<code>crates\engine_audio\src\mixer.rs:69</code>

```rust
        // Arrange
        let state = MixerState::default();

        // Assert
        for track in MixerTrack::ALL {
            assert!(
                (state.track_volume(track) - 1.0).abs() < f32::EPSILON,
                "{track:?} should default to 1.0"
            );
        }
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When set track volume, then only that track changes</summary>

<code>crates\engine_audio\src\mixer.rs:83</code>

```rust
        // Arrange
        let mut state = MixerState::default();

        // Act
        state.set_track_volume(MixerTrack::Music, 0.3);

        // Assert
        assert!((state.track_volume(MixerTrack::Music) - 0.3).abs() < f32::EPSILON);
        assert!((state.track_volume(MixerTrack::Sfx) - 1.0).abs() < f32::EPSILON);
        assert!((state.track_volume(MixerTrack::Ambient) - 1.0).abs() < f32::EPSILON);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When volume above one, then stored unchanged</summary>

<code>crates\engine_audio\src\mixer.rs:97</code>

```rust
        // Arrange
        let mut state = MixerState::default();

        // Act
        state.set_track_volume(MixerTrack::Music, 2.0);

        // Assert
        assert!((state.track_volume(MixerTrack::Music) - 2.0).abs() < f32::EPSILON);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When mixer track variants serialized to ron, then each deserializes to matching variant</summary>

<code>crates\engine_audio\src\mixer.rs:60</code>

```rust
        for track in MixerTrack::ALL {
            let ron = ron::to_string(&track).unwrap();
            let back: MixerTrack = ron::from_str(&ron).unwrap();
            assert_eq!(track, back);
        }
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test play_sound_buffer</strong> (4 tests)</summary>

<blockquote>
<details>
<summary>✅ When drained, then buffer is empty</summary>

<code>crates\engine_audio\src\play_sound_buffer.rs:114</code>

```rust
        // Arrange
        let mut buffer = PlaySoundBuffer::default();
        buffer.push(PlaySound::new("a"));
        buffer.push(PlaySound::new("b"));

        // Act
        let _ = buffer.drain().count();
        let remaining: Vec<_> = buffer.drain().collect();

        // Assert
        assert!(remaining.is_empty());
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When play sound new, then track defaults to sfx</summary>

<code>crates\engine_audio\src\play_sound_buffer.rs:82</code>

```rust
        // Act
        let cmd = PlaySound::new("beep");

        // Assert
        assert_eq!(cmd.track, MixerTrack::Sfx);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When play sound on track, then track is preserved</summary>

<code>crates\engine_audio\src\play_sound_buffer.rs:91</code>

```rust
        // Act
        let cmd = PlaySound::on_track("bgm", MixerTrack::Music);

        // Assert
        assert_eq!(cmd.track, MixerTrack::Music);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When push and drain, then returns one command</summary>

<code>crates\engine_audio\src\play_sound_buffer.rs:100</code>

```rust
        // Arrange
        let mut buffer = PlaySoundBuffer::default();
        buffer.push(PlaySound::new("beep"));

        // Act
        let commands: Vec<_> = buffer.drain().collect();

        // Assert
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].name, "beep");
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test play_sound_system</strong> (9 tests)</summary>

<blockquote>
<details>
<summary>✅ When no sound library, then audio not called</summary>

<code>crates\engine_audio\src\play_sound_system.rs:202</code>

```rust
        // Arrange
        let play_count = Arc::new(Mutex::new(0u32));
        let mut world = setup_world(&play_count);
        // No SoundLibrary inserted
        world
            .resource_mut::<PlaySoundBuffer>()
            .push(PlaySound::new("beep"));

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(*play_count.lock().unwrap(), 0);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When both gains zero, then play sound skips backend</summary>

<code>crates\engine_audio\src\play_sound_system.rs:337</code>

```rust
        // Arrange
        let play_count = Arc::new(Mutex::new(0u32));
        let mut world = setup_world(&play_count);
        let mut library = SoundLibrary::default();
        library.register("beep", test_effect());
        world.insert_resource(library);
        let mut cmd = PlaySound::new("beep");
        cmd.spatial_gains = Some(crate::spatial::SpatialGains {
            left: 0.0,
            right: 0.0,
        });
        world.resource_mut::<PlaySoundBuffer>().push(cmd);

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(*play_count.lock().unwrap(), 0);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When spatial gains present, then play sound applies them</summary>

<code>crates\engine_audio\src\play_sound_system.rs:315</code>

```rust
        // Arrange
        let play_count = Arc::new(Mutex::new(0u32));
        let mut world = setup_world(&play_count);
        let mut library = SoundLibrary::default();
        library.register("beep", test_effect());
        world.insert_resource(library);
        let mut cmd = PlaySound::new("beep");
        cmd.spatial_gains = Some(crate::spatial::SpatialGains {
            left: 0.3,
            right: 0.9,
        });
        world.resource_mut::<PlaySoundBuffer>().push(cmd);

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(*play_count.lock().unwrap(), 1);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When known sound, then audio play is called</summary>

<code>crates\engine_audio\src\play_sound_system.rs:163</code>

```rust
        // Arrange
        let play_count = Arc::new(Mutex::new(0u32));
        let mut world = setup_world(&play_count);
        let mut library = SoundLibrary::default();
        library.register("beep", test_effect());
        world.insert_resource(library);
        world
            .resource_mut::<PlaySoundBuffer>()
            .push(PlaySound::new("beep"));

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(*play_count.lock().unwrap(), 1);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When unknown sound name, then audio not called</summary>

<code>crates\engine_audio\src\play_sound_system.rs:219</code>

```rust
        // Arrange
        let play_count = Arc::new(Mutex::new(0u32));
        let mut world = setup_world(&play_count);
        let mut library = SoundLibrary::default();
        library.register("beep", test_effect());
        world.insert_resource(library);
        world
            .resource_mut::<PlaySoundBuffer>()
            .push(PlaySound::new("missing"));

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(*play_count.lock().unwrap(), 0);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When unknown sound name, then buffer is still drained</summary>

<code>crates\engine_audio\src\play_sound_system.rs:295</code>

```rust
        // Arrange
        let play_count = Arc::new(Mutex::new(0u32));
        let mut world = setup_world(&play_count);
        let mut library = SoundLibrary::default();
        library.register("beep", test_effect());
        world.insert_resource(library);
        world
            .resource_mut::<PlaySoundBuffer>()
            .push(PlaySound::new("missing"));

        // Act
        run_system(&mut world);

        // Assert
        let remaining: Vec<_> = world.resource_mut::<PlaySoundBuffer>().drain().collect();
        assert!(remaining.is_empty());
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When play sound without mixer state, then runs normally</summary>

<code>crates\engine_audio\src\play_sound_system.rs:275</code>

```rust
        // Arrange
        let play_count = Arc::new(Mutex::new(0u32));
        let mut world = setup_world(&play_count);
        let mut library = SoundLibrary::default();
        library.register("beep", test_effect());
        world.insert_resource(library);
        // No MixerState inserted
        world
            .resource_mut::<PlaySoundBuffer>()
            .push(PlaySound::new("beep"));

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(*play_count.lock().unwrap(), 1);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When known sound, then buffer is drained</summary>

<code>crates\engine_audio\src\play_sound_system.rs:182</code>

```rust
        // Arrange
        let play_count = Arc::new(Mutex::new(0u32));
        let mut world = setup_world(&play_count);
        let mut library = SoundLibrary::default();
        library.register("beep", test_effect());
        world.insert_resource(library);
        world
            .resource_mut::<PlaySoundBuffer>()
            .push(PlaySound::new("beep"));

        // Act
        run_system(&mut world);

        // Assert
        let remaining: Vec<_> = world.resource_mut::<PlaySoundBuffer>().drain().collect();
        assert!(remaining.is_empty());
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When play sound with mixer state, then track volume forwarded</summary>

<code>crates\engine_audio\src\play_sound_system.rs:238</code>

```rust
        // Arrange
        let play_count = Arc::new(Mutex::new(0u32));
        let played_tracks = Arc::new(Mutex::new(Vec::new()));
        let track_volume_calls = Arc::new(Mutex::new(Vec::new()));
        let mut world = World::new();
        world.insert_resource(PlaySoundBuffer::default());
        world.insert_resource(AudioRes::new(Box::new(
            SpyAudioBackend::with_track_captures(
                Arc::clone(&play_count),
                Arc::clone(&played_tracks),
                Arc::clone(&track_volume_calls),
            ),
        )));
        let mut library = SoundLibrary::default();
        library.register("bgm", test_effect());
        world.insert_resource(library);
        let mut mixer = MixerState::default();
        mixer.set_track_volume(MixerTrack::Music, 0.3);
        world.insert_resource(mixer);
        world
            .resource_mut::<PlaySoundBuffer>()
            .push(PlaySound::on_track("bgm", MixerTrack::Music));

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(*play_count.lock().unwrap(), 1);
        assert_eq!(played_tracks.lock().unwrap()[0], MixerTrack::Music);
        let calls = track_volume_calls.lock().unwrap();
        let music_call = calls.iter().find(|(t, _)| *t == MixerTrack::Music);
        assert!(music_call.is_some());
        assert!((music_call.unwrap().1 - 0.3).abs() < f32::EPSILON);
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test sound_data</strong> (2 tests)</summary>

<blockquote>
<details>
<summary>✅ When stereo, then frame count is half sample len</summary>

<code>crates\engine_audio\src\sound_data.rs:37</code>

```rust
        // Arrange
        let sound = SoundData {
            samples: vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8],
            sample_rate: 44_100,
            channels: 2,
        };

        // Act
        let frames = sound.frame_count();

        // Assert
        assert_eq!(frames, 4);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When mono, then frame count equals sample len</summary>

<code>crates\engine_audio\src\sound_data.rs:21</code>

```rust
        // Arrange
        let sound = SoundData {
            samples: vec![0.1, 0.2, 0.3, 0.4],
            sample_rate: 44_100,
            channels: 1,
        };

        // Act
        let frames = sound.frame_count();

        // Assert
        assert_eq!(frames, 4);
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test sound_effect</strong> (5 tests)</summary>

<blockquote>
<details>
<summary>✅ When nonzero amplitude graph, then samples are not all zero</summary>

<code>crates\engine_audio\src\sound_effect.rs:87</code>

```rust
        // Arrange
        let effect = test_effect();

        // Act
        let sound = effect.synthesize(44_100, 0.01);

        // Assert
        assert!(sound.samples.iter().any(|&s| s != 0.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When synthesize called, then sound data has mono channel count</summary>

<code>crates\engine_audio\src\sound_effect.rs:63</code>

```rust
        // Arrange
        let effect = test_effect();

        // Act
        let sound = effect.synthesize(44_100, 0.1);

        // Assert
        assert_eq!(sound.channels, 1);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When synthesize called twice, then each call returns fresh sound data</summary>

<code>crates\engine_audio\src\sound_effect.rs:99</code>

```rust
        // Arrange
        let effect = test_effect();

        // Act
        let sound_a = effect.synthesize(44_100, 0.1);
        let sound_b = effect.synthesize(44_100, 0.1);

        // Assert
        assert_eq!(sound_a.samples.len(), sound_b.samples.len());
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When synthesize called, then sound data has correct sample rate</summary>

<code>crates\engine_audio\src\sound_effect.rs:51</code>

```rust
        // Arrange
        let effect = test_effect();

        // Act
        let sound = effect.synthesize(44_100, 1.0);

        // Assert
        assert_eq!(sound.sample_rate, 44_100);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When synthesize called, then sample length equals frame count times channels</summary>

<code>crates\engine_audio\src\sound_effect.rs:75</code>

```rust
        // Arrange
        let effect = test_effect();

        // Act
        let sound = effect.synthesize(44_100, 1.0);

        // Assert
        assert_eq!(sound.samples.len(), 44_100);
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test sound_library</strong> (3 tests)</summary>

<blockquote>
<details>
<summary>✅ When empty library, then get returns none</summary>

<code>crates\engine_audio\src\sound_library.rs:35</code>

```rust
        // Arrange
        let library = SoundLibrary::default();

        // Act
        let result = library.get("explosion");

        // Assert
        assert!(result.is_none());
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When registered, then get with different name returns none</summary>

<code>crates\engine_audio\src\sound_library.rs:60</code>

```rust
        // Arrange
        let mut library = SoundLibrary::default();
        library.register("beep", test_effect());

        // Act
        let result = library.get("boom");

        // Assert
        assert!(result.is_none());
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When registered, then get with same name returns some</summary>

<code>crates\engine_audio\src\sound_library.rs:47</code>

```rust
        // Arrange
        let mut library = SoundLibrary::default();
        library.register("beep", test_effect());

        // Act
        let result = library.get("beep");

        // Assert
        assert!(result.is_some());
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test spatial</strong> (17 tests)</summary>

<blockquote>
<details>
<summary>✅ When audio emitter serialized to ron, then deserializes to equal value</summary>

<code>crates\engine_audio\src\spatial.rs:119</code>

```rust
        // Arrange
        let emitter = AudioEmitter {
            volume: 0.8,
            max_distance: 500.0,
        };

        // Act
        let ron = ron::to_string(&emitter).unwrap();
        let back: AudioEmitter = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(emitter, back);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When distance exceeds max, then attenuation clamped to zero</summary>

<code>crates\engine_audio\src\spatial.rs:201</code>

```rust
        // Act
        let result = distance_attenuation(200.0, 100.0);

        // Assert
        assert!(result.abs() < f32::EPSILON);
        assert!(result >= 0.0);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When distance equals max, then attenuation is zero</summary>

<code>crates\engine_audio\src\spatial.rs:183</code>

```rust
        // Act
        let result = distance_attenuation(50.0, 50.0);

        // Assert
        assert!(result.abs() < f32::EPSILON);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When distance half max, then attenuation is half</summary>

<code>crates\engine_audio\src\spatial.rs:192</code>

```rust
        // Act
        let result = distance_attenuation(50.0, 100.0);

        // Assert
        assert!((result - 0.5).abs() < f32::EPSILON);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When distance zero, then attenuation is one</summary>

<code>crates\engine_audio\src\spatial.rs:174</code>

```rust
        // Act
        let result = distance_attenuation(0.0, 100.0);

        // Assert
        assert!((result - 1.0).abs() < f32::EPSILON);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When emitter ahead of listener, then gains equal</summary>

*Centered panning when emitter is on listener's forward axis — no left/right bias*

<code>crates\engine_audio\src\spatial.rs:236</code>

```rust
        // Act
        let (left, right) = compute_pan(Vec2::ZERO, Vec2::new(0.0, 10.0));

        // Assert
        let expected = std::f32::consts::FRAC_1_SQRT_2;
        assert!(
            (left - expected).abs() < 0.001,
            "left should be ~0.707, got {left}"
        );
        assert!(
            (right - expected).abs() < 0.001,
            "right should be ~0.707, got {right}"
        );
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When emitter at listener, then gains equal no nan</summary>

*Coincident positions must not produce NaN — atan2(0,0) edge case handled by defaulting to centered pan*

<code>crates\engine_audio\src\spatial.rs:254</code>

```rust
        // Act
        let (left, right) = compute_pan(Vec2::ZERO, Vec2::ZERO);

        // Assert
        assert!(!left.is_nan());
        assert!(!right.is_nan());
        let expected = std::f32::consts::FRAC_1_SQRT_2;
        assert!(
            (left - expected).abs() < 0.001,
            "left should be ~0.707, got {left}"
        );
        assert!(
            (right - expected).abs() < 0.001,
            "right should be ~0.707, got {right}"
        );
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When emitter beyond max distance, then gains are zero</summary>

*Linear distance attenuation drops to zero beyond `max_distance`, effectively culling inaudible sounds*

<code>crates\engine_audio\src\spatial.rs:376</code>

```rust
        // Arrange
        let mut world = setup_world();
        spawn_listener(&mut world, 0.0, 0.0);
        let emitter = spawn_emitter(&mut world, 200.0, 0.0, 1.0, 100.0);
        world
            .resource_mut::<PlaySoundBuffer>()
            .push(PlaySound::at_emitter("beep", emitter));

        // Act
        run_spatial_system(&mut world);

        // Assert
        let cmds: Vec<_> = world.resource_mut::<PlaySoundBuffer>().drain().collect();
        let gains = cmds[0].spatial_gains.expect("should have spatial gains");
        assert!(gains.left.abs() < f32::EPSILON);
        assert!(gains.right.abs() < f32::EPSILON);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When emitter left of listener, then left gain one</summary>

<code>crates\engine_audio\src\spatial.rs:225</code>

```rust
        // Act
        let (left, right) = compute_pan(Vec2::ZERO, Vec2::new(-10.0, 0.0));

        // Assert
        assert!((left - 1.0).abs() < 0.001, "left should be ~1, got {left}");
        assert!(right.abs() < 0.001, "right should be ~0, got {right}");
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When emitter right of listener, then right gain one</summary>

*Constant-power stereo panning — emitter fully to the right produces 100% right channel gain*

<code>crates\engine_audio\src\spatial.rs:212</code>

```rust
        // Act
        let (left, right) = compute_pan(Vec2::ZERO, Vec2::new(10.0, 0.0));

        // Assert
        assert!(left.abs() < 0.001, "left should be ~0, got {left}");
        assert!(
            (right - 1.0).abs() < 0.001,
            "right should be ~1, got {right}"
        );
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When emitter is child entity, then world position used</summary>

*Spatial audio uses `GlobalTransform2D` (world space), not local `Transform2D` — hierarchy must propagate first*

<code>crates\engine_audio\src\spatial.rs:415</code>

```rust
        // Arrange
        let mut world = setup_world();
        spawn_listener(&mut world, 0.0, 0.0);

        // Parent at (80, 0), child emitter at local (0, 0) -> world (80, 0)
        let parent = world
            .spawn((Transform2D {
                position: Vec2::new(80.0, 0.0),
                ..Default::default()
            },))
            .id();
        let child = world
            .spawn((
                Transform2D::default(),
                ChildOf(parent),
                AudioEmitter {
                    volume: 1.0,
                    max_distance: 200.0,
                },
            ))
            .id();

        // Run hierarchy + transform propagation first
        let mut schedule = Schedule::default();
        schedule.add_systems(
            (
                hierarchy_maintenance_system,
                transform_propagation_system,
                spatial_audio_system,
            )
                .chain(),
        );
        world
            .resource_mut::<PlaySoundBuffer>()
            .push(PlaySound::at_emitter("beep", child));
        schedule.run(&mut world);

        // Assert
        let cmds: Vec<_> = world.resource_mut::<PlaySoundBuffer>().drain().collect();
        assert_eq!(cmds.len(), 1);
        let gains = cmds[0].spatial_gains.expect("should have spatial gains");
        // Emitter at world (80, 0) relative to listener at (0, 0)
        // -> right-panned, distance attenuation = 1 - 80/200 = 0.6
        assert!(gains.right > gains.left, "should be right-panned");
        assert!(
            gains.right > 0.0 && gains.right < 1.0,
            "should have distance attenuation"
        );
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When listener nonzero and emitter to left, then left gain dominates</summary>

<code>crates\engine_audio\src\spatial.rs:273</code>

```rust
        // Arrange — emitter at x=150, listener at x=200: emitter is LEFT of listener
        let listener = Vec2::new(200.0, 0.0);
        let emitter = Vec2::new(150.0, 0.0);

        // Act
        let (left, right) = compute_pan(listener, emitter);

        // Assert — diff = (-50, 0) → leftward, so left should dominate
        assert!(left > right, "left={left} should exceed right={right}");
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When emitter to right, then spatial gains reflect pan and attenuation</summary>

<code>crates\engine_audio\src\spatial.rs:325</code>

```rust
        // Arrange
        let mut world = setup_world();
        spawn_listener(&mut world, 0.0, 0.0);
        let emitter = spawn_emitter(&mut world, 50.0, 0.0, 1.0, 100.0);
        world
            .resource_mut::<PlaySoundBuffer>()
            .push(PlaySound::at_emitter("beep", emitter));

        // Act
        run_spatial_system(&mut world);

        // Assert
        let cmds: Vec<_> = world.resource_mut::<PlaySoundBuffer>().drain().collect();
        assert_eq!(cmds.len(), 1);
        let gains = cmds[0].spatial_gains.expect("should have spatial gains");
        assert!(
            gains.right > gains.left,
            "right should be louder: L={} R={}",
            gains.left,
            gains.right
        );
        assert!(
            gains.right < 1.0,
            "attenuation should reduce from full: R={}",
            gains.right
        );
        assert!(gains.right > 0.0);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When no listener, then system runs without panic</summary>

*Without an `AudioListener` entity, spatial processing is a no-op — gains remain unchanged*

<code>crates\engine_audio\src\spatial.rs:357</code>

```rust
        // Arrange
        let mut world = setup_world();
        let emitter = spawn_emitter(&mut world, 50.0, 0.0, 1.0, 100.0);
        world
            .resource_mut::<PlaySoundBuffer>()
            .push(PlaySound::at_emitter("beep", emitter));

        // Act
        run_spatial_system(&mut world);

        // Assert
        let cmds: Vec<_> = world.resource_mut::<PlaySoundBuffer>().drain().collect();
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].spatial_gains.is_none());
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When play sound without emitter, then gains unchanged</summary>

<code>crates\engine_audio\src\spatial.rs:396</code>

```rust
        // Arrange
        let mut world = setup_world();
        spawn_listener(&mut world, 0.0, 0.0);
        world
            .resource_mut::<PlaySoundBuffer>()
            .push(PlaySound::new("beep"));

        // Act
        run_spatial_system(&mut world);

        // Assert
        let cmds: Vec<_> = world.resource_mut::<PlaySoundBuffer>().drain().collect();
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].spatial_gains.is_none());
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When any two positions, then constant power property holds</summary>

<code>crates\engine_audio\src\spatial.rs:302</code>

```rust
            lx in -1000.0_f32..=1000.0,
            ly in -1000.0_f32..=1000.0,
            ex in -1000.0_f32..=1000.0,
            ey in -1000.0_f32..=1000.0,
        ) {
            // Act
            let (left, right) = compute_pan(Vec2::new(lx, ly), Vec2::new(ex, ey));

            // Assert — constant-power: left^2 + right^2 ≈ 1.0
            let power = left * left + right * right;
            assert!(
                (power - 1.0).abs() < 1e-4,
                "constant-power violated: L={left}, R={right}, L²+R²={power}"
            );

            // Assert — gains in [0, 1]
            assert!((0.0..=1.0).contains(&left), "left gain {left} out of [0,1]");
            assert!((0.0..=1.0).contains(&right), "right gain {right} out of [0,1]");
        }
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When any distance, then attenuation in zero to one</summary>

<code>crates\engine_audio\src\spatial.rs:287</code>

```rust
            distance in 0.0_f32..=1000.0,
            max_distance in 0.001_f32..=1000.0,
        ) {
            // Act
            let result = distance_attenuation(distance, max_distance);

            // Assert
            assert!(
                (0.0..=1.0).contains(&result),
                "attenuation {result} out of [0,1] for distance={distance}, max={max_distance}"
            );
        }

        #[test]
        fn when_any_two_positions_then_constant_power_property_holds(
            lx in -1000.0_f32..=1000.0,
            ly in -1000.0_f32..=1000.0,
            ex in -1000.0_f32..=1000.0,
            ey in -1000.0_f32..=1000.0,
        ) {
            // Act
            let (left, right) = compute_pan(Vec2::new(lx, ly), Vec2::new(ex, ey));

            // Assert — constant-power: left^2 + right^2 ≈ 1.0
            let power = left * left + right * right;
            assert!(
                (power - 1.0).abs() < 1e-4,
                "constant-power violated: L={left}, R={right}, L²+R²={power}"
            );

            // Assert — gains in [0, 1]
            assert!((0.0..=1.0).contains(&left), "left gain {left} out of [0,1]");
            assert!((0.0..=1.0).contains(&right), "right gain {right} out of [0,1]");
        }
```

</details>
</blockquote>

</details>
</blockquote>

</details>

<details>
<summary><strong>engine_core</strong> (39 tests)</summary>

<blockquote>
<details>
<summary><strong>test color</strong> (5 tests)</summary>

<blockquote>
<details>
<summary>✅ When color from u8 called, then converts to normalized f32</summary>

<code>crates\engine_core\src\color.rs:96</code>

```rust
        // Act
        let c = Color::from_u8(255, 128, 64, 255);

        // Assert
        assert_eq!(c.r, 1.0);
        assert!((c.g - 128.0 / 255.0).abs() < 1e-6);
        assert!((c.b - 64.0 / 255.0).abs() < 1e-6);
        assert_eq!(c.a, 1.0);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When color serialized to ron, then deserializes to equal value</summary>

<code>crates\engine_core\src\color.rs:70</code>

```rust
        // Arrange
        let color = Color::new(0.1, 0.5, 0.9, 0.75);

        // Act
        let ron = ron::to_string(&color).unwrap();
        let back: Color = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(color, back);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When transparent color serialized to ron, then roundtrip preserves zero alpha</summary>

<code>crates\engine_core\src\color.rs:83</code>

```rust
        // Arrange
        let color = Color::TRANSPARENT;

        // Act
        let ron = ron::to_string(&color).unwrap();
        let back: Color = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(color, back);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When any u8 inputs, then from u8 components in zero to one</summary>

<code>crates\engine_core\src\color.rs:109</code>

```rust
            r in proptest::num::u8::ANY,
            g in proptest::num::u8::ANY,
            b in proptest::num::u8::ANY,
            a in proptest::num::u8::ANY,
        ) {
            // Act
            let c = Color::from_u8(r, g, b, a);

            // Assert
            assert!((0.0..=1.0).contains(&c.r), "r={} out of range", c.r);
            assert!((0.0..=1.0).contains(&c.g), "g={} out of range", c.g);
            assert!((0.0..=1.0).contains(&c.b), "b={} out of range", c.b);
            assert!((0.0..=1.0).contains(&c.a), "a={} out of range", c.a);
        }

        #[test]
        fn when_any_finite_color_then_ron_roundtrip_preserves_value(
            r in -1e6_f32..=1e6,
            g in -1e6_f32..=1e6,
            b in -1e6_f32..=1e6,
            a in -1e6_f32..=1e6,
        ) {
            // Arrange
            let color = Color::new(r, g, b, a);

            // Act
            let ron = ron::to_string(&color).unwrap();
            let back: Color = ron::from_str(&ron).unwrap();

            // Assert
            assert_eq!(color, back);
        }
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When any finite color, then ron roundtrip preserves value</summary>

<code>crates\engine_core\src\color.rs:126</code>

```rust
            r in -1e6_f32..=1e6,
            g in -1e6_f32..=1e6,
            b in -1e6_f32..=1e6,
            a in -1e6_f32..=1e6,
        ) {
            // Arrange
            let color = Color::new(r, g, b, a);

            // Act
            let ron = ron::to_string(&color).unwrap();
            let back: Color = ron::from_str(&ron).unwrap();

            // Assert
            assert_eq!(color, back);
        }
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test error</strong> (4 tests)</summary>

<blockquote>
<details>
<summary>✅ When engine error boxed, then implements std error</summary>

<code>crates\engine_core\src\error.rs:40</code>

```rust
        // Act
        let _: Box<dyn std::error::Error> = Box::new(EngineError::NotFound("x".into()));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When not found displayed, then contains resource name</summary>

<code>crates\engine_core\src\error.rs:16</code>

```rust
        // Arrange
        let err = EngineError::NotFound("player".into());

        // Act
        let display = format!("{err}");

        // Assert
        assert!(display.contains("player"));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When engine error debug formatted, then identifies variant</summary>

<code>crates\engine_core\src\error.rs:46</code>

```rust
        // Arrange
        let err = EngineError::NotFound("x".into());

        // Act
        let debug = format!("{err:?}");

        // Assert
        assert!(debug.contains("NotFound"));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When invalid input displayed, then contains reason</summary>

<code>crates\engine_core\src\error.rs:28</code>

```rust
        // Arrange
        let err = EngineError::InvalidInput("negative scale".into());

        // Act
        let display = format!("{err}");

        // Assert
        assert!(display.contains("negative scale"));
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test time</strong> (13 tests)</summary>

<blockquote>
<details>
<summary>✅ When clock res derefmut, then reaches inner delta</summary>

<code>crates\engine_core\src\time.rs:204</code>

```rust
        // Arrange
        let mut fake = FakeClock::new();
        fake.advance(Seconds(0.25));
        let mut clock_res = ClockRes::new(Box::new(fake));

        // Act
        let dt = clock_res.delta();

        // Assert
        assert_eq!(dt, Seconds(0.25));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When fake clock advanced, then delta returns advancement</summary>

*`FakeClock` enables deterministic testing — `advance()` accumulates, `delta()` drains*

<code>crates\engine_core\src\time.rs:149</code>

```rust
        // Arrange
        let mut clock = FakeClock::new();

        // Act
        clock.advance(Seconds(0.016));

        // Assert
        assert_eq!(clock.delta(), Seconds(0.016));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When fake clock advanced multiple times, then delta accumulates</summary>

<code>crates\engine_core\src\time.rs:175</code>

```rust
        // Arrange
        let mut clock = FakeClock::new();

        // Act
        clock.advance(Seconds(0.1));
        clock.advance(Seconds(0.1));
        clock.advance(Seconds(0.1));

        // Assert
        let dt = clock.delta().0;
        assert!((dt - 0.3).abs() < f32::EPSILON * 4.0);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When fake clock constructed, then delta is zero</summary>

<code>crates\engine_core\src\time.rs:139</code>

```rust
        // Act
        let mut clock = FakeClock::new();

        // Assert
        assert_eq!(clock.delta(), Seconds(0.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When fake clock behind dyn time, then delta is correct</summary>

<code>crates\engine_core\src\time.rs:190</code>

```rust
        // Arrange
        let mut clock = FakeClock::new();
        clock.advance(Seconds(0.5));
        let dyn_clock: &mut dyn Time = &mut clock;

        // Act
        let dt = dyn_clock.delta();

        // Assert
        assert_eq!(dt, Seconds(0.5));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When fake clock delta called twice, then second call returns zero</summary>

<code>crates\engine_core\src\time.rs:161</code>

```rust
        // Arrange
        let mut clock = FakeClock::new();
        clock.advance(Seconds(0.016));
        clock.delta();

        // Act
        let second = clock.delta();

        // Assert
        assert_eq!(second, Seconds(0.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When tick across frames, then accumulator carries forward</summary>

*Accumulator carries sub-step remainder across frames, ensuring no simulation time is lost*

<code>crates\engine_core\src\time.rs:260</code>

```rust
        // Arrange — use binary-exact fractions to avoid f32 rounding
        let mut ts = FixedTimestep::with_step_size(Seconds(0.25));
        ts.tick(Seconds(0.375)); // 1 step, remainder 0.125

        // Act
        let steps = ts.tick(Seconds(0.125)); // 0.125 + 0.125 = 0.25 → 1 step

        // Assert
        assert_eq!(steps, 1);
        assert!(ts.accumulator.0.abs() < f32::EPSILON);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When tick below step size, then returns zero steps</summary>

*Sub-step deltas accumulate silently — no simulation steps fire until a full `step_size` is reached*

<code>crates\engine_core\src\time.rs:219</code>

```rust
        // Arrange
        let mut ts = FixedTimestep::with_step_size(Seconds(0.016));

        // Act
        let steps = ts.tick(Seconds(0.010));

        // Assert
        assert_eq!(steps, 0);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When tick exactly one step, then returns one step</summary>

<code>crates\engine_core\src\time.rs:231</code>

```rust
        // Arrange
        let mut ts = FixedTimestep::with_step_size(Seconds(0.016));

        // Act
        let steps = ts.tick(Seconds(0.016));

        // Assert
        assert_eq!(steps, 1);
        assert!(ts.accumulator.0.abs() < f32::EPSILON);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When tick large delta, then returns multiple steps and retains remainder</summary>

*Fix Your Timestep pattern — large frame deltas produce multiple fixed steps with leftover accumulated for the next frame*

<code>crates\engine_core\src\time.rs:245</code>

```rust
        // Arrange
        let mut ts = FixedTimestep::with_step_size(Seconds(0.016));

        // Act
        let steps = ts.tick(Seconds(0.050));

        // Assert
        assert_eq!(steps, 3);
        let remainder = ts.accumulator.0;
        assert!((remainder - 0.002).abs() < f32::EPSILON * 10.0);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When time system runs, then delta time updated from clock</summary>

<code>crates\engine_core\src\time.rs:274</code>

```rust
        // Arrange
        let mut world = bevy_ecs::world::World::new();
        let mut fake = FakeClock::new();
        fake.advance(Seconds(0.016));
        world.insert_resource(ClockRes::new(Box::new(fake)));
        world.insert_resource(DeltaTime::default());
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(time_system);

        // Act
        schedule.run(&mut world);

        // Assert
        let dt = world.resource::<DeltaTime>();
        assert_eq!(dt.0, Seconds(0.016));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When time system runs twice without advance, then second delta is zero</summary>

<code>crates\engine_core\src\time.rs:320</code>

```rust
        // Arrange
        let mut world = bevy_ecs::world::World::new();
        let mut fake = FakeClock::new();
        fake.advance(Seconds(0.016));
        world.insert_resource(ClockRes::new(Box::new(fake)));
        world.insert_resource(DeltaTime::default());
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(time_system);

        // Act — frame 1 consumes the advance, frame 2 has nothing
        schedule.run(&mut world);
        schedule.run(&mut world);

        // Assert
        assert_eq!(world.resource::<DeltaTime>().0, Seconds(0.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When any positive delta and step size, then accumulator stays below step size</summary>

<code>crates\engine_core\src\time.rs:294</code>

```rust
            step_size in 0.001_f32..=1.0,
            delta in 0.0_f32..=2.0,
        ) {
            // Arrange
            let mut ts = FixedTimestep::with_step_size(Seconds(step_size));

            // Act
            let _steps = ts.tick(Seconds(delta));

            // Assert
            assert!(
                ts.accumulator.0 >= 0.0,
                "accumulator should be non-negative, got {}",
                ts.accumulator.0
            );
            assert!(
                ts.accumulator.0 < step_size + f32::EPSILON * 16.0,
                "accumulator {} should be < step_size {}",
                ts.accumulator.0,
                step_size
            );
        }
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test transform</strong> (11 tests)</summary>

<blockquote>
<details>
<summary>✅ When default transform converted to affine2, then equals identity</summary>

<code>crates\engine_core\src\transform.rs:68</code>

```rust
        // Act
        let affine = Transform2D::default().to_affine2();

        // Assert
        assert_eq!(affine, Affine2::IDENTITY);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When transform has negative scale, then affine2 preserves flip</summary>

<code>crates\engine_core\src\transform.rs:161</code>

```rust
        // Arrange
        let t = Transform2D {
            scale: Vec2::new(-1.0, 1.0),
            ..Transform2D::default()
        };

        // Act
        let affine = t.to_affine2();

        // Assert
        assert_eq!(affine, Affine2::from_scale(Vec2::new(-1.0, 1.0)));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When transform has rotation only, then affine2 is pure rotation</summary>

<code>crates\engine_core\src\transform.rs:92</code>

```rust
        // Arrange
        let t = Transform2D {
            rotation: std::f32::consts::FRAC_PI_2,
            ..Transform2D::default()
        };

        // Act
        let affine = t.to_affine2();

        // Assert
        assert_eq!(affine, Affine2::from_angle(std::f32::consts::FRAC_PI_2));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When transform2d serialized to ron, then deserializes to equal value</summary>

<code>crates\engine_core\src\transform.rs:35</code>

```rust
        // Arrange
        let transform = Transform2D {
            position: Vec2::new(100.0, -50.0),
            rotation: 1.2,
            scale: Vec2::new(2.0, 0.5),
        };

        // Act
        let ron = ron::to_string(&transform).unwrap();
        let back: Transform2D = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(transform, back);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When transform2d with negative rotation serialized to ron, then roundtrip preserves sign</summary>

<code>crates\engine_core\src\transform.rs:52</code>

```rust
        // Arrange
        let transform = Transform2D {
            rotation: -std::f32::consts::FRAC_PI_2,
            ..Transform2D::default()
        };

        // Act
        let ron = ron::to_string(&transform).unwrap();
        let back: Transform2D = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(transform, back);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When transform composed, then order is scale rotate translate</summary>

<code>crates\engine_core\src\transform.rs:143</code>

```rust
        // Arrange
        let t = Transform2D {
            position: Vec2::new(1.0, 0.0),
            rotation: std::f32::consts::FRAC_PI_2,
            scale: Vec2::new(2.0, 1.0),
        };

        // Act
        let affine = t.to_affine2();

        // Assert
        let translation = affine.translation;
        assert!((translation.x - 1.0).abs() < 1e-6);
        assert!(translation.y.abs() < 1e-6);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When transform has all components, then affine2 matches scale angle translation</summary>

<code>crates\engine_core\src\transform.rs:122</code>

```rust
        // Arrange
        let t = Transform2D {
            position: Vec2::new(10.0, -5.0),
            rotation: std::f32::consts::FRAC_PI_4,
            scale: Vec2::splat(2.0),
        };

        // Act
        let affine = t.to_affine2();

        // Assert
        let expected = Affine2::from_scale_angle_translation(
            Vec2::splat(2.0),
            std::f32::consts::FRAC_PI_4,
            Vec2::new(10.0, -5.0),
        );
        assert_eq!(affine, expected);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When transform has full circle rotation, then affine2 is near identity</summary>

<code>crates\engine_core\src\transform.rs:201</code>

```rust
        // Arrange
        let t = Transform2D {
            rotation: std::f32::consts::TAU,
            ..Transform2D::default()
        };

        // Act
        let affine = t.to_affine2();

        // Assert
        let id = Affine2::IDENTITY;
        assert!((affine.matrix2.x_axis - id.matrix2.x_axis).length() < 1e-6);
        assert!((affine.matrix2.y_axis - id.matrix2.y_axis).length() < 1e-6);
        assert!((affine.translation - id.translation).length() < 1e-6);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When transform has scale only, then affine2 is pure scale</summary>

<code>crates\engine_core\src\transform.rs:107</code>

```rust
        // Arrange
        let t = Transform2D {
            scale: Vec2::new(2.0, 3.0),
            ..Transform2D::default()
        };

        // Act
        let affine = t.to_affine2();

        // Assert
        assert_eq!(affine, Affine2::from_scale(Vec2::new(2.0, 3.0)));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When transform has translation only, then affine2 is pure translation</summary>

<code>crates\engine_core\src\transform.rs:77</code>

```rust
        // Arrange
        let t = Transform2D {
            position: Vec2::new(3.0, 5.0),
            ..Transform2D::default()
        };

        // Act
        let affine = t.to_affine2();

        // Assert
        assert_eq!(affine, Affine2::from_translation(Vec2::new(3.0, 5.0)));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When any finite transform2d, then ron roundtrip preserves value</summary>

<code>crates\engine_core\src\transform.rs:177</code>

```rust
            px in -1000.0_f32..=1000.0,
            py in -1000.0_f32..=1000.0,
            rot in -std::f32::consts::TAU..=std::f32::consts::TAU,
            sx in -1000.0_f32..=1000.0,
            sy in -1000.0_f32..=1000.0,
        ) {
            // Arrange
            let transform = Transform2D {
                position: Vec2::new(px, py),
                rotation: rot,
                scale: Vec2::new(sx, sy),
            };

            // Act
            let ron = ron::to_string(&transform).unwrap();
            let back: Transform2D = ron::from_str(&ron).unwrap();

            // Assert
            assert_eq!(transform, back);
        }
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test types</strong> (6 tests)</summary>

<blockquote>
<details>
<summary>✅ When newtypes serialized to ron, then deserialize to equal value</summary>

<code>crates\engine_core\src\types.rs:65</code>

```rust
        // Arrange
        let pixels = Pixels(123.456);
        let seconds = Seconds(0.016);
        let texture_id = TextureId(42);

        // Act
        let pixels_ron = ron::to_string(&pixels).unwrap();
        let seconds_ron = ron::to_string(&seconds).unwrap();
        let texture_id_ron = ron::to_string(&texture_id).unwrap();

        let pixels_back: Pixels = ron::from_str(&pixels_ron).unwrap();
        let seconds_back: Seconds = ron::from_str(&seconds_ron).unwrap();
        let texture_id_back: TextureId = ron::from_str(&texture_id_ron).unwrap();

        // Assert
        assert_eq!(pixels, pixels_back);
        assert_eq!(seconds, seconds_back);
        assert_eq!(texture_id, texture_id_back);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When negative pixels serialized to ron, then roundtrip preserves sign</summary>

<code>crates\engine_core\src\types.rs:87</code>

```rust
        // Arrange
        let pixels = Pixels(-42.5);

        // Act
        let ron = ron::to_string(&pixels).unwrap();
        let back: Pixels = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(pixels, back);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When pixels arithmetic, then add sub mul produce correct results</summary>

<code>crates\engine_core\src\types.rs:100</code>

```rust
        assert_eq!(Pixels(1.5) + Pixels(2.5), Pixels(4.0));
        assert_eq!(Pixels(5.0) - Pixels(2.0), Pixels(3.0));
        assert_eq!(Pixels(4.0) * 0.5, Pixels(2.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When seconds arithmetic, then add sub mul produce correct results</summary>

<code>crates\engine_core\src\types.rs:107</code>

```rust
        assert_eq!(Seconds(0.5) + Seconds(0.25), Seconds(0.75));
        assert_eq!(Seconds(1.0) - Seconds(0.25), Seconds(0.75));
        assert_eq!(Seconds(0.016) * 2.0, Seconds(0.032));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When any finite pixels, then ron roundtrip preserves value</summary>

<code>crates\engine_core\src\types.rs:115</code>

```rust
            // Arrange
            let pixels = Pixels(v);

            // Act
            let ron = ron::to_string(&pixels).unwrap();
            let back: Pixels = ron::from_str(&ron).unwrap();

            // Assert
            assert_eq!(pixels, back);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When any finite seconds, then ron roundtrip preserves value</summary>

<code>crates\engine_core\src\types.rs:128</code>

```rust
            // Arrange
            let seconds = Seconds(v);

            // Act
            let ron = ron::to_string(&seconds).unwrap();
            let back: Seconds = ron::from_str(&ron).unwrap();

            // Assert
            assert_eq!(seconds, back);
```

</details>
</blockquote>

</details>
</blockquote>

</details>

<details>
<summary><strong>engine_ecs</strong> (1 tests)</summary>

<blockquote>
<details>
<summary><strong>test schedule</strong> (1 tests)</summary>

<blockquote>
<details>
<summary>✅ When index, then matches declaration order</summary>

<code>crates\engine_ecs\src\schedule.rs:34</code>

```rust
        for (expected, phase) in Phase::ALL.iter().enumerate() {
            assert_eq!(
                phase.index(),
                expected,
                "{phase:?} should have index {expected}"
            );
        }
```

</details>
</blockquote>

</details>
</blockquote>

</details>

<details>
<summary><strong>engine_input</strong> (54 tests)</summary>

<blockquote>
<details>
<summary><strong>test action_map</strong> (2 tests)</summary>

<blockquote>
<details>
<summary>✅ When single key bound to action, then bindings for returns that key</summary>

<code>crates\engine_input\src\action_map.rs:51</code>

```rust
        // Arrange
        let mut map = ActionMap::default();

        // Act
        map.bind("jump", vec![KeyCode::Space]);

        // Assert
        assert_eq!(map.bindings_for("jump"), &[KeyCode::Space]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When multiple keys bound to same action, then all keys returned</summary>

<code>crates\engine_input\src\action_map.rs:36</code>

```rust
        // Arrange
        let mut map = ActionMap::default();

        // Act
        map.bind("move_right", vec![KeyCode::ArrowRight, KeyCode::KeyD]);

        // Assert
        assert_eq!(
            map.bindings_for("move_right"),
            &[KeyCode::ArrowRight, KeyCode::KeyD]
        );
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test input_event_buffer</strong> (2 tests)</summary>

<blockquote>
<details>
<summary>✅ When key event pushed, then drain returns one event</summary>

<code>crates\engine_input\src\input_event_buffer.rs:30</code>

```rust
        // Arrange
        let mut buffer = InputEventBuffer::default();

        // Act
        buffer.push(KeyCode::ArrowRight, ElementState::Pressed);

        // Assert
        assert_eq!(buffer.drain().count(), 1);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When buffer drained, then returns all events and buffer is empty</summary>

<code>crates\engine_input\src\input_event_buffer.rs:42</code>

```rust
        // Arrange
        let mut buffer = InputEventBuffer::default();
        buffer.push(KeyCode::ArrowLeft, ElementState::Pressed);
        buffer.push(KeyCode::ArrowRight, ElementState::Released);

        // Act
        let events: Vec<_> = buffer.drain().collect();

        // Assert
        assert_eq!(events.len(), 2);
        assert_eq!(events[0], (KeyCode::ArrowLeft, ElementState::Pressed));
        assert_eq!(events[1], (KeyCode::ArrowRight, ElementState::Released));
        assert_eq!(buffer.drain().count(), 0);
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test input_state</strong> (17 tests)</summary>

<blockquote>
<details>
<summary>✅ When frame cleared, then just released is false</summary>

<code>crates\engine_input\src\mouse_state.rs:177</code>

```rust
        // Arrange
        let mut state = MouseState::default();
        state.press(MouseButton::Left);
        state.release(MouseButton::Left);

        // Act
        state.clear_frame_state();

        // Assert
        assert!(!state.just_released(MouseButton::Left));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When action not in map, then action pressed returns false</summary>

<code>crates\engine_input\src\input_state.rs:162</code>

```rust
        // Arrange
        let mut state = InputState::default();
        state.press(KeyCode::Space);
        let map = ActionMap::default();

        // Act
        let result = state.action_pressed(&map, "jump");

        // Assert
        assert!(!result);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When bound key held across frame clear, then action just pressed returns false</summary>

<code>crates\engine_input\src\input_state.rs:234</code>

```rust
        // Arrange
        let mut state = InputState::default();
        state.press(KeyCode::Space);
        state.clear_frame_state();
        let mut map = ActionMap::default();
        map.bind("jump", vec![KeyCode::Space]);

        // Act
        let result = state.action_just_pressed(&map, "jump");

        // Assert
        assert!(!result);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When bound key is just pressed, then action just pressed returns true</summary>

<code>crates\engine_input\src\input_state.rs:219</code>

```rust
        // Arrange
        let mut state = InputState::default();
        state.press(KeyCode::Space);
        let mut map = ActionMap::default();
        map.bind("jump", vec![KeyCode::Space]);

        // Act
        let result = state.action_just_pressed(&map, "jump");

        // Assert
        assert!(result);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When bound key is not pressed, then action pressed returns false</summary>

<code>crates\engine_input\src\input_state.rs:176</code>

```rust
        // Arrange
        let state = InputState::default();
        let mut map = ActionMap::default();
        map.bind("jump", vec![KeyCode::Space]);

        // Act
        let result = state.action_pressed(&map, "jump");

        // Assert
        assert!(!result);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When frame cleared, then held key stays pressed</summary>

<code>crates\engine_input\src\input_state.rs:280</code>

```rust
        // Arrange
        let mut state = InputState::default();
        state.press(KeyCode::ArrowLeft);

        // Act
        state.clear_frame_state();

        // Assert
        assert!(state.pressed(KeyCode::ArrowLeft));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When frame cleared, then just pressed is false for held key</summary>

<code>crates\engine_input\src\input_state.rs:135</code>

```rust
        // Arrange
        let mut state = InputState::default();
        state.press(KeyCode::ArrowUp);

        // Act
        state.clear_frame_state();

        // Assert
        assert!(!state.just_pressed(KeyCode::ArrowUp));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When input state default, then no keys are pressed</summary>

<code>crates\engine_input\src\input_state.rs:61</code>

```rust
        // Arrange
        let state = InputState::default();

        // Act
        let result = state.pressed(KeyCode::Space);

        // Assert
        assert!(!result);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When action not in map, then action just pressed returns false</summary>

<code>crates\engine_input\src\input_state.rs:205</code>

```rust
        // Arrange
        let mut state = InputState::default();
        state.press(KeyCode::Space);
        let map = ActionMap::default();

        // Act
        let result = state.action_just_pressed(&map, "jump");

        // Assert
        assert!(!result);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When bound key is pressed, then action pressed returns true</summary>

<code>crates\engine_input\src\input_state.rs:190</code>

```rust
        // Arrange
        let mut state = InputState::default();
        state.press(KeyCode::Space);
        let mut map = ActionMap::default();
        map.bind("jump", vec![KeyCode::Space]);

        // Act
        let result = state.action_pressed(&map, "jump");

        // Assert
        assert!(result);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When key pressed, then just pressed returns true</summary>

<code>crates\engine_input\src\input_state.rs:85</code>

```rust
        // Arrange
        let mut state = InputState::default();

        // Act
        state.press(KeyCode::ArrowRight);

        // Assert
        assert!(state.just_pressed(KeyCode::ArrowRight));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When key pressed, then pressed returns true</summary>

<code>crates\engine_input\src\input_state.rs:73</code>

```rust
        // Arrange
        let mut state = InputState::default();

        // Act
        state.press(KeyCode::ArrowRight);

        // Assert
        assert!(state.pressed(KeyCode::ArrowRight));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When key pressed, then just released returns false</summary>

<code>crates\engine_input\src\input_state.rs:97</code>

```rust
        // Arrange
        let mut state = InputState::default();

        // Act
        state.press(KeyCode::ArrowRight);

        // Assert
        assert!(!state.just_released(KeyCode::ArrowRight));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When key released after press, then just released returns true</summary>

<code>crates\engine_input\src\input_state.rs:122</code>

```rust
        // Arrange
        let mut state = InputState::default();
        state.press(KeyCode::Space);

        // Act
        state.release(KeyCode::Space);

        // Assert
        assert!(state.just_released(KeyCode::Space));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When key released after press, then pressed returns false</summary>

<code>crates\engine_input\src\input_state.rs:109</code>

```rust
        // Arrange
        let mut state = InputState::default();
        state.press(KeyCode::Space);

        // Act
        state.release(KeyCode::Space);

        // Assert
        assert!(!state.pressed(KeyCode::Space));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When one of multiple bound keys is just pressed, then action just pressed returns true</summary>

<code>crates\engine_input\src\input_state.rs:250</code>

```rust
        // Arrange
        let mut state = InputState::default();
        state.press(KeyCode::KeyD);
        let mut map = ActionMap::default();
        map.bind("move_right", vec![KeyCode::ArrowRight, KeyCode::KeyD]);

        // Act
        let result = state.action_just_pressed(&map, "move_right");

        // Assert
        assert!(result);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When one of multiple bound keys is pressed, then action pressed returns true</summary>

<code>crates\engine_input\src\input_state.rs:265</code>

```rust
        // Arrange
        let mut state = InputState::default();
        state.press(KeyCode::KeyD);
        let mut map = ActionMap::default();
        map.bind("move_right", vec![KeyCode::ArrowRight, KeyCode::KeyD]);

        // Act
        let result = state.action_pressed(&map, "move_right");

        // Assert
        assert!(result);
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test input_system</strong> (7 tests)</summary>

<blockquote>
<details>
<summary>✅ When system runs, then buffer is drained</summary>

<code>crates\engine_input\src\input_system.rs:111</code>

```rust
        // Arrange
        let mut world = setup_world();
        world
            .resource_mut::<InputEventBuffer>()
            .push(KeyCode::ArrowRight, ElementState::Pressed);

        // Act
        run_input_system(&mut world);

        // Assert
        assert_eq!(world.resource_mut::<InputEventBuffer>().drain().count(), 0);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When system runs second frame, then just released is cleared</summary>

<code>crates\engine_input\src\input_system.rs:144</code>

```rust
        // Arrange
        let mut world = setup_world();
        world.resource_mut::<InputState>().press(KeyCode::Space);
        world.resource_mut::<InputState>().clear_frame_state();
        world
            .resource_mut::<InputEventBuffer>()
            .push(KeyCode::Space, ElementState::Released);
        run_input_system(&mut world);

        // Act
        run_input_system(&mut world);

        // Assert
        assert!(!world.resource::<InputState>().just_released(KeyCode::Space));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When system runs second frame, then just pressed is cleared</summary>

<code>crates\engine_input\src\input_system.rs:126</code>

```rust
        // Arrange
        let mut world = setup_world();
        world
            .resource_mut::<InputEventBuffer>()
            .push(KeyCode::ArrowDown, ElementState::Pressed);
        run_input_system(&mut world);

        // Act
        run_input_system(&mut world);

        // Assert
        let state = world.resource::<InputState>();
        assert!(!state.just_pressed(KeyCode::ArrowDown));
        assert!(state.pressed(KeyCode::ArrowDown));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When release event in buffer, then key is just released</summary>

<code>crates\engine_input\src\input_system.rs:94</code>

```rust
        // Arrange
        let mut world = setup_world();
        world.resource_mut::<InputState>().press(KeyCode::Space);
        world.resource_mut::<InputState>().clear_frame_state();
        world
            .resource_mut::<InputEventBuffer>()
            .push(KeyCode::Space, ElementState::Released);

        // Act
        run_input_system(&mut world);

        // Assert
        assert!(world.resource::<InputState>().just_released(KeyCode::Space));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When release event in buffer, then key is not pressed</summary>

<code>crates\engine_input\src\input_system.rs:77</code>

```rust
        // Arrange
        let mut world = setup_world();
        world.resource_mut::<InputState>().press(KeyCode::Space);
        world.resource_mut::<InputState>().clear_frame_state();
        world
            .resource_mut::<InputEventBuffer>()
            .push(KeyCode::Space, ElementState::Released);

        // Act
        run_input_system(&mut world);

        // Assert
        assert!(!world.resource::<InputState>().pressed(KeyCode::Space));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When press event in buffer, then key is just pressed</summary>

<code>crates\engine_input\src\input_system.rs:58</code>

```rust
        // Arrange
        let mut world = setup_world();
        world
            .resource_mut::<InputEventBuffer>()
            .push(KeyCode::ArrowRight, ElementState::Pressed);

        // Act
        run_input_system(&mut world);

        // Assert
        assert!(
            world
                .resource::<InputState>()
                .just_pressed(KeyCode::ArrowRight)
        );
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When press event in buffer, then key is pressed</summary>

<code>crates\engine_input\src\input_system.rs:43</code>

```rust
        // Arrange
        let mut world = setup_world();
        world
            .resource_mut::<InputEventBuffer>()
            .push(KeyCode::ArrowRight, ElementState::Pressed);

        // Act
        run_input_system(&mut world);

        // Assert
        assert!(world.resource::<InputState>().pressed(KeyCode::ArrowRight));
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test mouse_event_buffer</strong> (2 tests)</summary>

<blockquote>
<details>
<summary>✅ When buffer drained, then buffer is empty on second drain</summary>

<code>crates\engine_input\src\mouse_event_buffer.rs:44</code>

```rust
        // Arrange
        let mut buffer = MouseEventBuffer::default();
        buffer.push(MouseButton::Left, ElementState::Pressed);
        buffer.push(MouseButton::Right, ElementState::Released);

        // Act
        let _: Vec<_> = buffer.drain().collect();

        // Assert
        assert_eq!(buffer.drain().count(), 0);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When button event pushed, then drain returns that event</summary>

<code>crates\engine_input\src\mouse_event_buffer.rs:30</code>

```rust
        // Arrange
        let mut buffer = MouseEventBuffer::default();

        // Act
        buffer.push(MouseButton::Left, ElementState::Pressed);
        let events: Vec<_> = buffer.drain().collect();

        // Assert
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], (MouseButton::Left, ElementState::Pressed));
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test mouse_input_system</strong> (5 tests)</summary>

<blockquote>
<details>
<summary>✅ When mouse input system runs second frame, then just pressed is cleared</summary>

<code>crates\engine_input\src\mouse_input_system.rs:112</code>

```rust
        // Arrange
        let mut world = setup_world();
        world
            .resource_mut::<MouseEventBuffer>()
            .push(MouseButton::Left, ElementState::Pressed);
        run_mouse_system(&mut world);

        // Act
        run_mouse_system(&mut world);

        // Assert
        let state = world.resource::<MouseState>();
        assert!(!state.just_pressed(MouseButton::Left));
        assert!(state.pressed(MouseButton::Left));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When press event in buffer, then mouse input system sets button pressed</summary>

<code>crates\engine_input\src\mouse_input_system.rs:44</code>

```rust
        // Arrange
        let mut world = setup_world();
        world
            .resource_mut::<MouseEventBuffer>()
            .push(MouseButton::Left, ElementState::Pressed);

        // Act
        run_mouse_system(&mut world);

        // Assert
        assert!(world.resource::<MouseState>().pressed(MouseButton::Left));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When mouse input system runs, then buffer is drained</summary>

<code>crates\engine_input\src\mouse_input_system.rs:97</code>

```rust
        // Arrange
        let mut world = setup_world();
        world
            .resource_mut::<MouseEventBuffer>()
            .push(MouseButton::Left, ElementState::Pressed);

        // Act
        run_mouse_system(&mut world);

        // Assert
        assert_eq!(world.resource_mut::<MouseEventBuffer>().drain().count(), 0);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When release event in buffer, then mouse input system sets just released</summary>

<code>crates\engine_input\src\mouse_input_system.rs:78</code>

```rust
        // Arrange
        let mut world = setup_world();
        world.resource_mut::<MouseState>().press(MouseButton::Left);
        world.resource_mut::<MouseState>().clear_frame_state();
        world
            .resource_mut::<MouseEventBuffer>()
            .push(MouseButton::Left, ElementState::Released);

        // Act
        run_mouse_system(&mut world);

        // Assert
        let state = world.resource::<MouseState>();
        assert!(state.just_released(MouseButton::Left));
        assert!(!state.pressed(MouseButton::Left));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When press event in buffer, then mouse input system sets just pressed</summary>

<code>crates\engine_input\src\mouse_input_system.rs:59</code>

```rust
        // Arrange
        let mut world = setup_world();
        world
            .resource_mut::<MouseEventBuffer>()
            .push(MouseButton::Right, ElementState::Pressed);

        // Act
        run_mouse_system(&mut world);

        // Assert
        assert!(
            world
                .resource::<MouseState>()
                .just_pressed(MouseButton::Right)
        );
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test mouse_state</strong> (19 tests)</summary>

<blockquote>
<details>
<summary>✅ When action bound to mouse button and button just pressed, then action just pressed returns true</summary>

<code>crates\engine_input\src\mouse_state.rs:296</code>

```rust
        // Arrange
        let mut state = MouseState::default();
        state.press(MouseButton::Left);
        let mut map = crate::action_map::ActionMap::default();
        map.bind_mouse("fire", vec![MouseButton::Left]);

        // Act
        let result = state.action_just_pressed(&map, "fire");

        // Assert
        assert!(result);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When action bound to mouse button and button not pressed, then action pressed returns false</summary>

<code>crates\engine_input\src\mouse_state.rs:281</code>

```rust
        // Arrange
        let state = MouseState::default();
        let mut map = crate::action_map::ActionMap::default();
        map.bind_mouse("fire", vec![MouseButton::Left]);

        // Act
        let result = state.action_pressed(&map, "fire");

        // Assert
        assert!(!result);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When action bound to mouse button and button pressed, then action pressed returns true</summary>

<code>crates\engine_input\src\mouse_state.rs:266</code>

```rust
        // Arrange
        let mut state = MouseState::default();
        state.press(MouseButton::Left);
        let mut map = crate::action_map::ActionMap::default();
        map.bind_mouse("fire", vec![MouseButton::Left]);

        // Act
        let result = state.action_pressed(&map, "fire");

        // Assert
        assert!(result);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When button held but frame cleared, then action just pressed returns false</summary>

<code>crates\engine_input\src\mouse_state.rs:312</code>

```rust
        // Arrange
        let mut state = MouseState::default();
        state.press(MouseButton::Left);
        state.clear_frame_state();
        let mut map = crate::action_map::ActionMap::default();
        map.bind_mouse("fire", vec![MouseButton::Left]);

        // Act
        let result = state.action_just_pressed(&map, "fire");

        // Assert
        assert!(!result);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When button pressed, then just pressed returns true</summary>

<code>crates\engine_input\src\mouse_state.rs:113</code>

```rust
        // Arrange
        let mut state = MouseState::default();

        // Act
        state.press(MouseButton::Left);

        // Assert
        assert!(state.just_pressed(MouseButton::Left));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When button pressed, then just released returns false</summary>

<code>crates\engine_input\src\mouse_state.rs:125</code>

```rust
        // Arrange
        let mut state = MouseState::default();

        // Act
        state.press(MouseButton::Left);

        // Assert
        assert!(!state.just_released(MouseButton::Left));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When button pressed, then pressed returns true</summary>

<code>crates\engine_input\src\mouse_state.rs:101</code>

```rust
        // Arrange
        let mut state = MouseState::default();

        // Act
        state.press(MouseButton::Left);

        // Assert
        assert!(state.pressed(MouseButton::Left));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When button released after press, then just released returns true</summary>

<code>crates\engine_input\src\mouse_state.rs:150</code>

```rust
        // Arrange
        let mut state = MouseState::default();
        state.press(MouseButton::Right);

        // Act
        state.release(MouseButton::Right);

        // Assert
        assert!(state.just_released(MouseButton::Right));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When button released after press, then pressed returns false</summary>

<code>crates\engine_input\src\mouse_state.rs:137</code>

```rust
        // Arrange
        let mut state = MouseState::default();
        state.press(MouseButton::Right);

        // Act
        state.release(MouseButton::Right);

        // Assert
        assert!(!state.pressed(MouseButton::Right));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When cursor moved, then screen pos is updated</summary>

<code>crates\engine_input\src\mouse_state.rs:191</code>

```rust
        // Arrange
        let mut state = MouseState::default();

        // Act
        state.set_screen_pos(Vec2::new(100.0, 200.0));

        // Assert
        assert_eq!(state.screen_pos(), Vec2::new(100.0, 200.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When frame cleared, then just pressed is false for held button</summary>

<code>crates\engine_input\src\mouse_state.rs:163</code>

```rust
        // Arrange
        let mut state = MouseState::default();
        state.press(MouseButton::Left);

        // Act
        state.clear_frame_state();

        // Assert
        assert!(!state.just_pressed(MouseButton::Left));
        assert!(state.pressed(MouseButton::Left));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When frame cleared, then just released is false</summary>

<code>crates\engine_input\src\mouse_state.rs:177</code>

```rust
        // Arrange
        let mut state = MouseState::default();
        state.press(MouseButton::Left);
        state.release(MouseButton::Left);

        // Act
        state.clear_frame_state();

        // Assert
        assert!(!state.just_released(MouseButton::Left));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When frame cleared, then scroll delta is zero</summary>

<code>crates\engine_input\src\mouse_state.rs:215</code>

```rust
        // Arrange
        let mut state = MouseState::default();
        state.add_scroll_delta(Vec2::new(2.0, 5.0));

        // Act
        state.clear_frame_state();

        // Assert
        assert_eq!(state.scroll_delta(), Vec2::ZERO);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When multiple scroll events in one frame, then delta is sum</summary>

<code>crates\engine_input\src\mouse_state.rs:228</code>

```rust
        // Arrange
        let mut state = MouseState::default();

        // Act
        state.add_scroll_delta(Vec2::new(0.0, 1.0));
        state.add_scroll_delta(Vec2::new(0.0, 1.0));

        // Assert
        assert_eq!(state.scroll_delta(), Vec2::new(0.0, 2.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When no buttons pressed, then mouse state reports nothing pressed</summary>

<code>crates\engine_input\src\mouse_state.rs:90</code>

```rust
        // Arrange
        let state = MouseState::default();

        // Assert
        assert!(!state.pressed(MouseButton::Left));
        assert!(!state.pressed(MouseButton::Middle));
        assert!(!state.pressed(MouseButton::Right));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When screen pos set, then clear frame state does not reset it</summary>

<code>crates\engine_input\src\mouse_state.rs:241</code>

```rust
        // Arrange
        let mut state = MouseState::default();
        state.set_screen_pos(Vec2::new(50.0, 75.0));

        // Act
        state.clear_frame_state();

        // Assert
        assert_eq!(state.screen_pos(), Vec2::new(50.0, 75.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When scroll accumulated, then scroll delta reflects total</summary>

<code>crates\engine_input\src\mouse_state.rs:203</code>

```rust
        // Arrange
        let mut state = MouseState::default();

        // Act
        state.add_scroll_delta(Vec2::new(0.0, 3.0));

        // Assert
        assert_eq!(state.scroll_delta(), Vec2::new(0.0, 3.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When unbound mouse action queried, then action pressed returns false</summary>

<code>crates\engine_input\src\mouse_state.rs:328</code>

```rust
        // Arrange
        let mut state = MouseState::default();
        state.press(MouseButton::Left);
        let map = crate::action_map::ActionMap::default();

        // Act
        let result = state.action_pressed(&map, "fire");

        // Assert
        assert!(!result);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When world pos set, then world pos is readable</summary>

<code>crates\engine_input\src\mouse_state.rs:254</code>

```rust
        // Arrange
        let mut state = MouseState::default();

        // Act
        state.set_world_pos(Vec2::new(300.0, -150.0));

        // Assert
        assert_eq!(state.world_pos(), Vec2::new(300.0, -150.0));
```

</details>
</blockquote>

</details>
</blockquote>

</details>

<details>
<summary><strong>engine_physics</strong> (44 tests)</summary>

<blockquote>
<details>
<summary><strong>test collider</strong> (2 tests)</summary>

<blockquote>
<details>
<summary>✅ When collider variants serialized to ron, then each deserializes to equal value</summary>

<code>crates\engine_physics\src\collider.rs:36</code>

```rust
        // Arrange
        let colliders = [
            Collider::Circle(15.0),
            Collider::Aabb(Vec2::new(32.0, 64.0)),
            Collider::ConvexPolygon(vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(10.0, 0.0),
                Vec2::new(5.0, 8.0),
            ]),
        ];

        for collider in &colliders {
            // Act
            let ron = ron::to_string(collider).unwrap();
            let back: Collider = ron::from_str(&ron).unwrap();

            // Assert
            assert_eq!(*collider, back);
        }
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When convex polygon collider debug formatted, then snapshot matches</summary>

<code>crates\engine_physics\src\collider.rs:18</code>

```rust
        // Arrange
        let collider = Collider::ConvexPolygon(vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(15.0, 8.0),
            Vec2::new(5.0, 14.0),
            Vec2::new(-5.0, 8.0),
        ]);

        // Act
        let debug = format!("{collider:#?}");

        // Assert
        insta::assert_snapshot!(debug);
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test collision_event</strong> (3 tests)</summary>

<blockquote>
<details>
<summary>✅ When empty buffer drained, then returns empty iterator</summary>

<code>crates\engine_physics\src\collision_event.rs:40</code>

```rust
        // Arrange
        let mut buffer = CollisionEventBuffer::default();

        // Act
        let events: Vec<_> = buffer.drain().collect();

        // Assert
        assert!(events.is_empty());
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When event pushed and drained, then yields that event</summary>

<code>crates\engine_physics\src\collision_event.rs:52</code>

```rust
        // Arrange
        let entities = spawn_entities(2);
        let mut buffer = CollisionEventBuffer::default();
        let event = CollisionEvent {
            entity_a: entities[0],
            entity_b: entities[1],
            kind: CollisionKind::Started,
        };
        buffer.push(event);

        // Act
        let events: Vec<_> = buffer.drain().collect();

        // Assert
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], event);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When drained twice, then second drain returns empty</summary>

<code>crates\engine_physics\src\collision_event.rs:72</code>

```rust
        // Arrange
        let entities = spawn_entities(2);
        let mut buffer = CollisionEventBuffer::default();
        buffer.push(CollisionEvent {
            entity_a: entities[0],
            entity_b: entities[1],
            kind: CollisionKind::Started,
        });
        buffer.push(CollisionEvent {
            entity_a: entities[1],
            entity_b: entities[0],
            kind: CollisionKind::Stopped,
        });

        // Act
        let _ = buffer.drain().count();
        let second: Vec<_> = buffer.drain().collect();

        // Assert
        assert!(second.is_empty());
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test physics_backend</strong> (8 tests)</summary>

<blockquote>
<details>
<summary>✅ When add collider without body, then returns false</summary>

<code>crates\engine_physics\src\physics_backend.rs:157</code>

```rust
        // Arrange
        let mut backend = NullPhysicsBackend::new();
        let entity = spawn_entity();

        // Act
        let result = backend.add_collider(entity, &Collider::Circle(1.0));

        // Assert
        assert!(!result);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When body position queried for unregistered, then returns none</summary>

<code>crates\engine_physics\src\physics_backend.rs:105</code>

```rust
        // Arrange
        let backend = NullPhysicsBackend::new();
        let entity = spawn_entity();

        // Act
        let pos = backend.body_position(entity);
        let rot = backend.body_rotation(entity);

        // Assert
        assert!(pos.is_none());
        assert!(rot.is_none());
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When remove body for unknown entity, then no panic</summary>

<code>crates\engine_physics\src\physics_backend.rs:135</code>

```rust
        // Arrange
        let mut backend = NullPhysicsBackend::new();
        let entity = spawn_entity();

        // Act
        backend.remove_body(entity);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When remove body, then entity is deregistered</summary>

<code>crates\engine_physics\src\physics_backend.rs:120</code>

```rust
        // Arrange
        let mut backend = NullPhysicsBackend::new();
        let entity = spawn_entity();
        backend.add_body(entity, &RigidBody::Dynamic, Vec2::ZERO);

        // Act
        backend.remove_body(entity);
        let re_add = backend.add_body(entity, &RigidBody::Dynamic, Vec2::ZERO);

        // Assert
        assert!(re_add);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When step called, then step count increments</summary>

<code>crates\engine_physics\src\physics_backend.rs:76</code>

```rust
        // Arrange
        let mut backend = NullPhysicsBackend::new();

        // Act
        backend.step(Seconds(0.016));
        backend.step(Seconds(0.016));
        backend.step(Seconds(0.016));

        // Assert
        assert_eq!(backend.step_count(), 3);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When add body, then returns true and duplicate returns false</summary>

<code>crates\engine_physics\src\physics_backend.rs:90</code>

```rust
        // Arrange
        let mut backend = NullPhysicsBackend::new();
        let entity = spawn_entity();

        // Act
        let first = backend.add_body(entity, &RigidBody::Dynamic, Vec2::ZERO);
        let second = backend.add_body(entity, &RigidBody::Dynamic, Vec2::ZERO);

        // Assert
        assert!(first);
        assert!(!second);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When null backend drain collision events, then returns empty</summary>

<code>crates\engine_physics\src\physics_backend.rs:145</code>

```rust
        // Arrange
        let mut backend = NullPhysicsBackend::new();

        // Act
        let events = backend.drain_collision_events();

        // Assert
        assert!(events.is_empty());
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When add collider, then returns true</summary>

<code>crates\engine_physics\src\physics_backend.rs:170</code>

```rust
        // Arrange
        let mut backend = NullPhysicsBackend::new();
        let entity = spawn_entity();
        backend.add_body(entity, &RigidBody::Dynamic, Vec2::ZERO);

        // Act
        let result = backend.add_collider(entity, &Collider::Circle(1.0));

        // Assert
        assert!(result);
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test physics_step_system</strong> (4 tests)</summary>

<blockquote>
<details>
<summary>✅ When backend produces events, then buffer contains them</summary>

<code>crates\engine_physics\src\physics_step_system.rs:143</code>

```rust
        // Arrange
        let step_count = Arc::new(AtomicU32::new(0));
        let entities = spawn_entities(2);
        let event = CollisionEvent {
            entity_a: entities[0],
            entity_b: entities[1],
            kind: CollisionKind::Started,
        };
        let mut world = World::new();
        world.insert_resource(PhysicsRes::new(Box::new(
            SpyPhysicsBackend::new(Arc::clone(&step_count)).with_events(vec![event]),
        )));
        world.insert_resource(CollisionEventBuffer::default());
        world.insert_resource(DeltaTime(Seconds(0.016)));
        let mut schedule = Schedule::default();
        schedule.add_systems(physics_step_system);

        // Act
        schedule.run(&mut world);

        // Assert
        let mut buffer = world.resource_mut::<CollisionEventBuffer>();
        let events: Vec<_> = buffer.drain().collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], event);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When system runs with no events, then buffer remains empty</summary>

<code>crates\engine_physics\src\physics_step_system.rs:110</code>

```rust
        // Arrange
        let step_count = Arc::new(AtomicU32::new(0));
        let mut world = setup_world(Arc::clone(&step_count));
        let mut schedule = Schedule::default();
        schedule.add_systems(physics_step_system);

        // Act
        schedule.run(&mut world);

        // Assert
        let mut buffer = world.resource_mut::<CollisionEventBuffer>();
        let events: Vec<_> = buffer.drain().collect();
        assert!(events.is_empty());
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When system runs, then backend is stepped</summary>

<code>crates\engine_physics\src\physics_step_system.rs:95</code>

```rust
        // Arrange
        let step_count = Arc::new(AtomicU32::new(0));
        let mut world = setup_world(Arc::clone(&step_count));
        let mut schedule = Schedule::default();
        schedule.add_systems(physics_step_system);

        // Act
        schedule.run(&mut world);

        // Assert
        assert_eq!(step_count.load(Ordering::Relaxed), 1);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When system runs twice, then backend stepped twice</summary>

<code>crates\engine_physics\src\physics_step_system.rs:127</code>

```rust
        // Arrange
        let step_count = Arc::new(AtomicU32::new(0));
        let mut world = setup_world(Arc::clone(&step_count));
        let mut schedule = Schedule::default();
        schedule.add_systems(physics_step_system);

        // Act
        schedule.run(&mut world);
        schedule.run(&mut world);

        // Assert
        assert_eq!(step_count.load(Ordering::Relaxed), 2);
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test physics_sync_system</strong> (12 tests)</summary>

<blockquote>
<details>
<summary>✅ When backend returns both position and rotation, then both fields updated</summary>

<code>crates\engine_physics\src\physics_sync_system.rs:152</code>

```rust
        // Arrange
        let mut world = World::new();
        let entity = world
            .spawn((RigidBody::Dynamic, Transform2D::default()))
            .id();
        world.insert_resource(PhysicsRes::new(Box::new(SpyPhysicsBackend::with_both(
            entity,
            Vec2::new(5.0, -3.0),
            std::f32::consts::FRAC_PI_2,
        ))));

        // Act
        run_sync(&mut world);

        // Assert
        let transform = world.get::<Transform2D>(entity).unwrap();
        assert_eq!(transform.position, Vec2::new(5.0, -3.0));
        assert!((transform.rotation - std::f32::consts::FRAC_PI_2).abs() < 1e-6);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When backend returns none for unregistered entity, then transform is unchanged</summary>

<code>crates\engine_physics\src\physics_sync_system.rs:174</code>

```rust
        // Arrange
        let mut world = World::new();
        let entity = world
            .spawn((
                RigidBody::Dynamic,
                Transform2D {
                    position: Vec2::new(99.0, 99.0),
                    rotation: 1.0,
                    ..Transform2D::default()
                },
            ))
            .id();
        world.insert_resource(PhysicsRes::new(Box::new(SpyPhysicsBackend::empty())));

        // Act
        run_sync(&mut world);

        // Assert
        let transform = world.get::<Transform2D>(entity).unwrap();
        assert_eq!(transform.position, Vec2::new(99.0, 99.0));
        assert!((transform.rotation - 1.0).abs() < f32::EPSILON);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When backend returns position only, then rotation field is unchanged</summary>

*Position and rotation are synced independently — either can be None without affecting the other*

<code>crates\engine_physics\src\physics_sync_system.rs:200</code>

```rust
        // Arrange
        let mut world = World::new();
        let entity = world
            .spawn((
                RigidBody::Dynamic,
                Transform2D {
                    rotation: 2.5,
                    ..Transform2D::default()
                },
            ))
            .id();
        world.insert_resource(PhysicsRes::new(Box::new(SpyPhysicsBackend::with_position(
            entity,
            Vec2::new(1.0, 2.0),
        ))));

        // Act
        run_sync(&mut world);

        // Assert
        let transform = world.get::<Transform2D>(entity).unwrap();
        assert_eq!(transform.position, Vec2::new(1.0, 2.0));
        assert!((transform.rotation - 2.5).abs() < f32::EPSILON);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When backend returns position, then transform position is updated</summary>

*One-way sync: physics backend → `Transform2D`. ECS is the read side, rapier is the authority*

<code>crates\engine_physics\src\physics_sync_system.rs:110</code>

```rust
        // Arrange
        let mut world = World::new();
        let entity = world
            .spawn((RigidBody::Dynamic, Transform2D::default()))
            .id();
        world.insert_resource(PhysicsRes::new(Box::new(SpyPhysicsBackend::with_position(
            entity,
            Vec2::new(10.0, 20.0),
        ))));

        // Act
        run_sync(&mut world);

        // Assert
        let transform = world.get::<Transform2D>(entity).unwrap();
        assert_eq!(transform.position, Vec2::new(10.0, 20.0));
        assert!((transform.rotation - 0.0).abs() < f32::EPSILON);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When backend returns rotation only, then position field is unchanged</summary>

<code>crates\engine_physics\src\physics_sync_system.rs:227</code>

```rust
        // Arrange
        let mut world = World::new();
        let entity = world
            .spawn((
                RigidBody::Dynamic,
                Transform2D {
                    position: Vec2::new(7.0, 8.0),
                    ..Transform2D::default()
                },
            ))
            .id();
        world.insert_resource(PhysicsRes::new(Box::new(SpyPhysicsBackend::with_rotation(
            entity, 0.5,
        ))));

        // Act
        run_sync(&mut world);

        // Assert
        let transform = world.get::<Transform2D>(entity).unwrap();
        assert_eq!(transform.position, Vec2::new(7.0, 8.0));
        assert!((transform.rotation - 0.5).abs() < f32::EPSILON);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When backend returns rotation, then transform rotation is updated</summary>

<code>crates\engine_physics\src\physics_sync_system.rs:131</code>

```rust
        // Arrange
        let mut world = World::new();
        let entity = world
            .spawn((RigidBody::Dynamic, Transform2D::default()))
            .id();
        world.insert_resource(PhysicsRes::new(Box::new(SpyPhysicsBackend::with_rotation(
            entity,
            std::f32::consts::FRAC_PI_4,
        ))));

        // Act
        run_sync(&mut world);

        // Assert
        let transform = world.get::<Transform2D>(entity).unwrap();
        assert!((transform.rotation - std::f32::consts::FRAC_PI_4).abs() < 1e-6);
        assert_eq!(transform.position, Vec2::ZERO);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When multiple entities registered, then each entity receives its own transform</summary>

<code>crates\engine_physics\src\physics_sync_system.rs:280</code>

```rust
        // Arrange
        let mut world = World::new();
        let entity_a = world
            .spawn((RigidBody::Dynamic, Transform2D::default()))
            .id();
        let entity_b = world
            .spawn((RigidBody::Dynamic, Transform2D::default()))
            .id();
        let mut positions = HashMap::new();
        positions.insert(entity_a, Vec2::new(1.0, 0.0));
        positions.insert(entity_b, Vec2::new(0.0, 2.0));
        world.insert_resource(PhysicsRes::new(Box::new(SpyPhysicsBackend {
            positions,
            rotations: HashMap::new(),
        })));

        // Act
        run_sync(&mut world);

        // Assert
        let transform_a = world.get::<Transform2D>(entity_a).unwrap();
        assert_eq!(transform_a.position, Vec2::new(1.0, 0.0));
        let transform_b = world.get::<Transform2D>(entity_b).unwrap();
        assert_eq!(transform_b.position, Vec2::new(0.0, 2.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When entity has no rigid body, then its transform is not touched</summary>

*Only entities with `RigidBody` participate in physics sync — plain transforms are untouched*

<code>crates\engine_physics\src\physics_sync_system.rs:309</code>

```rust
        // Arrange
        let mut world = World::new();
        let physics_entity = world
            .spawn((RigidBody::Dynamic, Transform2D::default()))
            .id();
        let plain_entity = world
            .spawn(Transform2D {
                position: Vec2::new(50.0, 50.0),
                ..Transform2D::default()
            })
            .id();
        world.insert_resource(PhysicsRes::new(Box::new(SpyPhysicsBackend::with_position(
            physics_entity,
            Vec2::new(1.0, 1.0),
        ))));

        // Act
        run_sync(&mut world);

        // Assert
        let transform = world.get::<Transform2D>(plain_entity).unwrap();
        assert_eq!(transform.position, Vec2::new(50.0, 50.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When no entities have rigid body, then system runs without panic</summary>

<code>crates\engine_physics\src\physics_sync_system.rs:99</code>

```rust
        // Arrange
        let mut world = World::new();
        world.insert_resource(PhysicsRes::new(Box::new(NullPhysicsBackend::new())));

        // Act + Assert
        run_sync(&mut world);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When rigid body is kinematic, then transform is still synced</summary>

<code>crates\engine_physics\src\physics_sync_system.rs:355</code>

```rust
        // Arrange
        let mut world = World::new();
        let entity = world
            .spawn((RigidBody::Kinematic, Transform2D::default()))
            .id();
        world.insert_resource(PhysicsRes::new(Box::new(SpyPhysicsBackend::with_position(
            entity,
            Vec2::new(6.0, 7.0),
        ))));

        // Act
        run_sync(&mut world);

        // Assert
        let transform = world.get::<Transform2D>(entity).unwrap();
        assert_eq!(transform.position, Vec2::new(6.0, 7.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When rigid body is static, then transform is still synced</summary>

<code>crates\engine_physics\src\physics_sync_system.rs:335</code>

```rust
        // Arrange
        let mut world = World::new();
        let entity = world
            .spawn((RigidBody::Static, Transform2D::default()))
            .id();
        world.insert_resource(PhysicsRes::new(Box::new(SpyPhysicsBackend::with_position(
            entity,
            Vec2::new(3.0, 4.0),
        ))));

        // Act
        run_sync(&mut world);

        // Assert
        let transform = world.get::<Transform2D>(entity).unwrap();
        assert_eq!(transform.position, Vec2::new(3.0, 4.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When system runs, then transform scale is never modified</summary>

<code>crates\engine_physics\src\physics_sync_system.rs:253</code>

```rust
        // Arrange
        let mut world = World::new();
        let entity = world
            .spawn((
                RigidBody::Dynamic,
                Transform2D {
                    scale: Vec2::new(3.0, 0.5),
                    ..Transform2D::default()
                },
            ))
            .id();
        world.insert_resource(PhysicsRes::new(Box::new(SpyPhysicsBackend::with_both(
            entity,
            Vec2::new(1.0, 1.0),
            1.0,
        ))));

        // Act
        run_sync(&mut world);

        // Assert
        let transform = world.get::<Transform2D>(entity).unwrap();
        assert_eq!(transform.scale, Vec2::new(3.0, 0.5));
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test rapier_backend</strong> (14 tests)</summary>

<blockquote>
<details>
<summary>✅ When add collider for unknown entity, then returns false</summary>

<code>crates\engine_physics\src\rapier_backend.rs:270</code>

```rust
        // Arrange
        let mut backend = RapierBackend::new(Vec2::ZERO);
        let entity = spawn_entity();

        // Act
        let result = backend.add_collider(entity, &Collider::Circle(1.0));

        // Assert
        assert!(!result);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When body type mapping, then static is fixed and kinematic is position based</summary>

*Body type mapping: ECS Static → rapier Fixed (immovable), ECS Kinematic → rapier `KinematicPositionBased` (script-driven)*

<code>crates\engine_physics\src\rapier_backend.rs:219</code>

```rust
        // Arrange
        let mut backend = RapierBackend::new(Vec2::ZERO);
        let entities = spawn_entities(2);
        let static_entity = entities[0];
        let kinematic_entity = entities[1];

        // Act
        backend.add_body(static_entity, &RigidBody::Static, Vec2::ZERO);
        backend.add_body(kinematic_entity, &RigidBody::Kinematic, Vec2::ZERO);

        // Assert
        let static_handle = backend.entity_to_handle[&static_entity];
        let kinematic_handle = backend.entity_to_handle[&kinematic_entity];
        let static_body = backend.bodies.get(static_handle).unwrap();
        let kinematic_body = backend.bodies.get(kinematic_handle).unwrap();
        assert_eq!(
            static_body.body_type(),
            rapier2d::prelude::RigidBodyType::Fixed
        );
        assert_eq!(
            kinematic_body.body_type(),
            rapier2d::prelude::RigidBodyType::KinematicPositionBased
        );
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When dynamic body added, then position is queryable</summary>

*Body type mapping: ECS Dynamic → rapier Dynamic (free motion under forces)*

<code>crates\engine_physics\src\rapier_backend.rs:187</code>

```rust
        // Arrange
        let mut backend = RapierBackend::new(Vec2::ZERO);
        let entity = spawn_entity();

        // Act
        let added = backend.add_body(entity, &RigidBody::Dynamic, Vec2::new(3.0, 7.0));
        let pos = backend.body_position(entity);

        // Assert
        assert!(added);
        let pos = pos.unwrap();
        assert!((pos.x - 3.0).abs() < 1e-4);
        assert!((pos.y - 7.0).abs() < 1e-4);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When collider variants added, then all return true</summary>

<code>crates\engine_physics\src\rapier_backend.rs:246</code>

```rust
        // Arrange
        let mut backend = RapierBackend::new(Vec2::ZERO);
        let entities = spawn_entities(3);
        let (e1, e2, e3) = (entities[0], entities[1], entities[2]);
        backend.add_body(e1, &RigidBody::Dynamic, Vec2::ZERO);
        backend.add_body(e2, &RigidBody::Dynamic, Vec2::ZERO);
        backend.add_body(e3, &RigidBody::Dynamic, Vec2::ZERO);

        // Act
        let circle = backend.add_collider(e1, &Collider::Circle(2.0));
        let aabb = backend.add_collider(e2, &Collider::Aabb(Vec2::new(1.0, 0.5)));
        let polygon = backend.add_collider(
            e3,
            &Collider::ConvexPolygon(vec![Vec2::ZERO, Vec2::new(1.0, 0.0), Vec2::new(0.5, 1.0)]),
        );

        // Assert
        assert!(circle);
        assert!(aabb);
        assert!(polygon);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When dynamic body added, then rotation returns some</summary>

*Entity removal must clean up both rapier `RigidBody` and the entity↔handle map*

<code>crates\engine_physics\src\rapier_backend.rs:300</code>

```rust
        // Arrange
        let mut backend = RapierBackend::new(Vec2::ZERO);
        let entity = spawn_entity();
        backend.add_body(entity, &RigidBody::Dynamic, Vec2::ZERO);

        // Act
        let rotation = backend.body_rotation(entity);

        // Assert
        let rotation = rotation.expect("should return Some for living body");
        assert!(rotation.abs() < 1e-4, "initial rotation should be ~0");
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When remove body on rapier, then position returns none</summary>

<code>crates\engine_physics\src\rapier_backend.rs:315</code>

```rust
        // Arrange
        let mut backend = RapierBackend::new(Vec2::ZERO);
        let entity = spawn_entity();
        backend.add_body(entity, &RigidBody::Dynamic, Vec2::new(1.0, 2.0));

        // Act
        backend.remove_body(entity);

        // Assert
        assert!(backend.body_position(entity).is_none());
        assert!(backend.body_rotation(entity).is_none());
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When no colliders step and drain, then no events</summary>

<code>crates\engine_physics\src\rapier_backend.rs:330</code>

```rust
        // Arrange
        let mut backend = RapierBackend::new(Vec2::ZERO);

        // Act
        backend.step(Seconds(0.016));
        let events = backend.drain_collision_events();

        // Assert
        assert!(events.is_empty());
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When rapier step on empty world, then no panic</summary>

<code>crates\engine_physics\src\rapier_backend.rs:177</code>

```rust
        // Arrange
        let mut backend = RapierBackend::new(Vec2::new(0.0, -9.81));

        // Act
        backend.step(Seconds(0.016));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When remove body for unknown entity on rapier, then no panic</summary>

<code>crates\engine_physics\src\rapier_backend.rs:406</code>

```rust
        // Arrange
        let mut backend = RapierBackend::new(Vec2::ZERO);
        let entity = spawn_entity();

        // Act
        backend.remove_body(entity);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When dynamic body steps under gravity, then y changes</summary>

<code>crates\engine_physics\src\rapier_backend.rs:283</code>

```rust
        // Arrange
        let mut backend = RapierBackend::new(Vec2::new(0.0, -9.81));
        let entity = spawn_entity();
        backend.add_body(entity, &RigidBody::Dynamic, Vec2::new(0.0, 10.0));
        backend.add_collider(entity, &Collider::Circle(0.5));

        // Act
        backend.step(Seconds(0.1));

        // Assert
        let pos = backend.body_position(entity).unwrap();
        assert!(pos.y < 10.0, "expected y < 10.0, got {}", pos.y);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When same entity added twice, then second returns false</summary>

<code>crates\engine_physics\src\rapier_backend.rs:204</code>

```rust
        // Arrange
        let mut backend = RapierBackend::new(Vec2::ZERO);
        let entity = spawn_entity();
        backend.add_body(entity, &RigidBody::Dynamic, Vec2::ZERO);

        // Act
        let second = backend.add_body(entity, &RigidBody::Dynamic, Vec2::ZERO);

        // Assert
        assert!(!second);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When body removed after collision, then drain does not panic</summary>

<code>crates\engine_physics\src\rapier_backend.rs:389</code>

```rust
        // Arrange
        let mut backend = RapierBackend::new(Vec2::ZERO);
        let entities = spawn_entities(2);
        backend.add_body(entities[0], &RigidBody::Dynamic, Vec2::ZERO);
        backend.add_collider(entities[0], &Collider::Circle(1.0));
        backend.add_body(entities[1], &RigidBody::Dynamic, Vec2::ZERO);
        backend.add_collider(entities[1], &Collider::Circle(1.0));
        backend.step(Seconds(0.016));
        backend.remove_body(entities[0]);

        // Act
        backend.step(Seconds(0.016));
        let _ = backend.drain_collision_events();
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When drain called twice without step, then second is empty</summary>

<code>crates\engine_physics\src\rapier_backend.rs:370</code>

```rust
        // Arrange
        let mut backend = RapierBackend::new(Vec2::ZERO);
        let entities = spawn_entities(2);
        backend.add_body(entities[0], &RigidBody::Dynamic, Vec2::ZERO);
        backend.add_collider(entities[0], &Collider::Circle(1.0));
        backend.add_body(entities[1], &RigidBody::Dynamic, Vec2::ZERO);
        backend.add_collider(entities[1], &Collider::Circle(1.0));
        backend.step(Seconds(0.016));
        let _ = backend.drain_collision_events();

        // Act
        let events = backend.drain_collision_events();

        // Assert
        assert!(events.is_empty());
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When two overlapping circles step, then started event with correct entities</summary>

*Collision events flow: rapier `ChannelEventCollector` → drain → `CollisionEventBuffer` with entity resolution*

<code>crates\engine_physics\src\rapier_backend.rs:344</code>

```rust
        // Arrange
        let mut backend = RapierBackend::new(Vec2::ZERO);
        let entities = spawn_entities(2);
        backend.add_body(entities[0], &RigidBody::Dynamic, Vec2::ZERO);
        backend.add_collider(entities[0], &Collider::Circle(1.0));
        backend.add_body(entities[1], &RigidBody::Dynamic, Vec2::ZERO);
        backend.add_collider(entities[1], &Collider::Circle(1.0));

        // Act
        backend.step(Seconds(0.016));
        let events = backend.drain_collision_events();

        // Assert
        assert_eq!(events.len(), 1, "expected 1 event, got {events:?}");
        assert_eq!(events[0].kind, CollisionKind::Started);
        let pair = (events[0].entity_a, events[0].entity_b);
        assert!(
            pair == (entities[0], entities[1]) || pair == (entities[1], entities[0]),
            "expected entities {:?}, got {:?}",
            (entities[0], entities[1]),
            pair
        );
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test rigid_body</strong> (1 tests)</summary>

<blockquote>
<details>
<summary>✅ When rigid body variants serialized to ron, then each deserializes to matching variant</summary>

<code>crates\engine_physics\src\rigid_body.rs:17</code>

```rust
        for body in [RigidBody::Dynamic, RigidBody::Static, RigidBody::Kinematic] {
            let ron = ron::to_string(&body).unwrap();
            let back: RigidBody = ron::from_str(&ron).unwrap();
            assert_eq!(body, back);
        }
```

</details>
</blockquote>

</details>
</blockquote>

</details>

<details>
<summary><strong>engine_render</strong> (232 tests)</summary>

<blockquote>
<details>
<summary><strong>test atlas</strong> (31 tests)</summary>

<blockquote>
<details>
<summary>✅ When adding image, then uv rect is normalized to zero one</summary>

<code>crates\engine_render\src\atlas.rs:259</code>

```rust
        // Arrange
        let mut builder = AtlasBuilder::new(256, 256);

        // Act
        let handle = builder.add_image(2, 2, &[255; 16]).unwrap();

        // Assert
        let [u0, v0, u1, v1] = handle.uv_rect;
        assert!((0.0..=1.0).contains(&u0));
        assert!((0.0..=1.0).contains(&v0));
        assert!((0.0..=1.0).contains(&u1));
        assert!((0.0..=1.0).contains(&v1));
        assert!(u1 > u0, "uv_rect must have positive width");
        assert!(v1 > v0, "uv_rect must have positive height");
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When adding two images, then each has distinct texture id</summary>

<code>crates\engine_render\src\atlas.rs:289</code>

```rust
        // Arrange
        let mut builder = AtlasBuilder::new(256, 256);

        // Act
        let h1 = builder.add_image(2, 2, &[255; 16]).unwrap();
        let h2 = builder.add_image(2, 2, &[0; 16]).unwrap();

        // Assert
        assert_ne!(h1.texture_id, h2.texture_id);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When adding single image, then returns handle with valid texture id</summary>

<code>crates\engine_render\src\atlas.rs:247</code>

```rust
        // Arrange
        let mut builder = AtlasBuilder::new(512, 512);

        // Act
        let result = builder.add_image(1, 1, &[255, 0, 0, 255]);

        // Assert
        assert!(result.is_ok());
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When adding two images, then uv rects do not overlap</summary>

<code>crates\engine_render\src\atlas.rs:302</code>

```rust
        // Arrange
        let mut builder = AtlasBuilder::new(256, 256);

        // Act
        let h1 = builder.add_image(4, 4, &[255; 64]).unwrap();
        let h2 = builder.add_image(4, 4, &[0; 64]).unwrap();

        // Assert — convert to pixel rects and check no overlap
        let [u0a, v0a, u1a, v1a] = h1.uv_rect;
        let [u0b, v0b, u1b, v1b] = h2.uv_rect;
        let no_overlap = u1a <= u0b || u1b <= u0a || v1a <= v0b || v1b <= v0a;
        assert!(
            no_overlap,
            "uv_rects overlap: {:?} vs {:?}",
            h1.uv_rect, h2.uv_rect
        );
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When adding many images, then all uv rects are non overlapping</summary>

<code>crates\engine_render\src\atlas.rs:322</code>

```rust
        // Arrange
        let mut builder = AtlasBuilder::new(512, 512);
        let pixel_data = [128u8; 32 * 32 * 4];

        // Act
        let handles: Vec<_> = (0..16)
            .map(|_| builder.add_image(32, 32, &pixel_data).unwrap())
            .collect();

        // Assert — pairwise non-overlap
        for i in 0..handles.len() {
            for j in (i + 1)..handles.len() {
                let [u0a, v0a, u1a, v1a] = handles[i].uv_rect;
                let [u0b, v0b, u1b, v1b] = handles[j].uv_rect;
                let no_overlap = u1a <= u0b || u1b <= u0a || v1a <= v0b || v1b <= v0a;
                assert!(no_overlap, "handles {i} and {j} overlap");
            }
        }
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When adding zero height image, then returns invalid dimensions</summary>

<code>crates\engine_render\src\atlas.rs:399</code>

```rust
        // Arrange
        let mut builder = AtlasBuilder::new(256, 256);

        // Act
        let result = builder.add_image(4, 0, &[]);

        // Assert
        assert!(matches!(result, Err(AtlasError::InvalidDimensions)));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When adding zero width image, then returns invalid dimensions</summary>

<code>crates\engine_render\src\atlas.rs:387</code>

```rust
        // Arrange
        let mut builder = AtlasBuilder::new(256, 256);

        // Act
        let result = builder.add_image(0, 4, &[]);

        // Assert
        assert!(matches!(result, Err(AtlasError::InvalidDimensions)));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When atlas full, then returns no space error</summary>

<code>crates\engine_render\src\atlas.rs:356</code>

```rust
        // Arrange
        let mut builder = AtlasBuilder::new(4, 4);
        builder.add_image(4, 4, &[255; 64]).unwrap();

        // Act
        let result = builder.add_image(1, 1, &[0; 4]);

        // Assert
        assert!(matches!(result, Err(AtlasError::NoSpace)));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When adding image larger than atlas, then returns no space error</summary>

<code>crates\engine_render\src\atlas.rs:344</code>

```rust
        // Arrange
        let mut builder = AtlasBuilder::new(8, 8);

        // Act
        let result = builder.add_image(16, 16, &[0; 16 * 16 * 4]);

        // Assert
        assert!(matches!(result, Err(AtlasError::NoSpace)));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When builder created, then reports matching dimensions</summary>

<code>crates\engine_render\src\atlas.rs:233</code>

```rust
        // Arrange
        let builder = AtlasBuilder::new(512, 256);

        // Act
        let w = builder.width();
        let h = builder.height();

        // Assert
        assert_eq!(w, 512);
        assert_eq!(h, 256);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When adding image that fills atlas, then uv rect is full range</summary>

<code>crates\engine_render\src\atlas.rs:277</code>

```rust
        // Arrange
        let mut builder = AtlasBuilder::new(4, 4);

        // Act
        let handle = builder.add_image(4, 4, &[255; 64]).unwrap();

        // Assert
        assert_eq!(handle.uv_rect, [0.0, 0.0, 1.0, 1.0]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When building atlas, then all rows of image are correctly placed</summary>

<code>crates\engine_render\src\atlas.rs:568</code>

```rust
        // Arrange
        let mut builder = AtlasBuilder::new(4, 4);
        // Row 0: red, green; Row 1: blue, white
        #[rustfmt::skip]
        let data = [
            255, 0, 0, 255,    0, 255, 0, 255,
            0, 0, 255, 255,    255, 255, 255, 255,
        ];
        let handle = builder.add_image(2, 2, &data).unwrap();

        // Act
        let atlas = builder.build();

        // Assert
        let [u0, v0, _, _] = handle.uv_rect;
        let px = (u0 * atlas.width as f32) as usize;
        let py = (v0 * atlas.height as f32) as usize;
        let stride = atlas.width as usize * 4;
        assert_eq!(&atlas.data[py * stride + px * 4..][..4], [255, 0, 0, 255]);
        assert_eq!(
            &atlas.data[py * stride + (px + 1) * 4..][..4],
            [0, 255, 0, 255]
        );
        assert_eq!(
            &atlas.data[(py + 1) * stride + px * 4..][..4],
            [0, 0, 255, 255]
        );
        assert_eq!(
            &atlas.data[(py + 1) * stride + (px + 1) * 4..][..4],
            [255, 255, 255, 255]
        );
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When building empty atlas, then pixel buffer is all zeros</summary>

<code>crates\engine_render\src\atlas.rs:207</code>

```rust
        // Arrange
        let builder = AtlasBuilder::new(4, 4);

        // Act
        let atlas = builder.build();

        // Assert
        assert_eq!(atlas.data, vec![0u8; 4 * 4 * 4]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When atlas uploaded marker present, then upload atlas not called</summary>

<code>crates\engine_render\src\atlas.rs:720</code>

```rust
        // Arrange
        let mut world = bevy_ecs::world::World::new();
        let log = insert_spy(&mut world);
        world.insert_resource(minimal_atlas());
        world.insert_resource(AtlasUploaded);

        // Act
        run_system(&mut world);

        // Assert
        assert!(!log.lock().unwrap().contains(&"upload_atlas".to_string()));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When atlas present, then upload atlas called</summary>

<code>crates\engine_render\src\atlas.rs:669</code>

```rust
        // Arrange
        let mut world = bevy_ecs::world::World::new();
        let log = insert_spy(&mut world);
        world.insert_resource(minimal_atlas());

        // Act
        run_system(&mut world);

        // Assert
        assert!(log.lock().unwrap().contains(&"upload_atlas".to_string()));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When building atlas, then pixel data appears at correct offset</summary>

<code>crates\engine_render\src\atlas.rs:456</code>

```rust
        // Arrange
        let mut builder = AtlasBuilder::new(8, 8);
        let red = [255, 0, 0, 255].repeat(2 * 2);
        let handle = builder.add_image(2, 2, &red).unwrap();

        // Act
        let atlas = builder.build();

        // Assert — sample the top-left pixel of the allocation
        let [u0, v0, _, _] = handle.uv_rect;
        let px = (u0 * atlas.width as f32) as usize;
        let py = (v0 * atlas.height as f32) as usize;
        let offset = (py * atlas.width as usize + px) * 4;
        assert_eq!(&atlas.data[offset..offset + 4], &[255, 0, 0, 255]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When building atlas with image, then buffer size matches atlas</summary>

<code>crates\engine_render\src\atlas.rs:219</code>

```rust
        // Arrange
        let mut builder = AtlasBuilder::new(64, 128);
        let pixel_data = vec![255u8; 2 * 2 * 4];
        builder.add_image(2, 2, &pixel_data).unwrap();

        // Act
        let atlas = builder.build();

        // Assert
        assert_eq!(atlas.data.len(), 64 * 128 * 4);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When data length mismatches, then returns error</summary>

<code>crates\engine_render\src\atlas.rs:369</code>

```rust
        // Arrange
        let mut builder = AtlasBuilder::new(256, 256);

        // Act
        let result = builder.add_image(1, 1, &[255, 0, 0]);

        // Assert
        assert!(matches!(
            result,
            Err(AtlasError::DataLengthMismatch {
                expected: 4,
                actual: 3
            })
        ));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When building atlas with two images, then neither overwrites the other</summary>

<code>crates\engine_render\src\atlas.rs:474</code>

```rust
        // Arrange
        let mut builder = AtlasBuilder::new(16, 16);
        let red = [255, 0, 0, 255].repeat(2 * 2);
        let blue = [0, 0, 255, 255].repeat(2 * 2);
        let h_red = builder.add_image(2, 2, &red).unwrap();
        let h_blue = builder.add_image(2, 2, &blue).unwrap();

        // Act
        let atlas = builder.build();

        // Assert — sample one pixel from each allocation
        let sample = |uv: [f32; 4]| -> &[u8] {
            let px = (uv[0] * atlas.width as f32) as usize;
            let py = (uv[1] * atlas.height as f32) as usize;
            let off = (py * atlas.width as usize + px) * 4;
            &atlas.data[off..off + 4]
        };
        assert_eq!(sample(h_red.uv_rect), &[255, 0, 0, 255]);
        assert_eq!(sample(h_blue.uv_rect), &[0, 0, 255, 255]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When loading invalid bytes, then returns decode error</summary>

<code>crates\engine_render\src\atlas.rs:523</code>

```rust
        // Act
        let result = load_image_bytes(&[0x00, 0x01, 0x02]);

        // Assert
        assert!(matches!(result, Err(AtlasError::DecodeError(_))));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When loading image and adding to atlas, then dimensions preserved</summary>

<code>crates\engine_render\src\atlas.rs:532</code>

```rust
        // Arrange
        let png_bytes = make_1x1_png(0, 255, 0, 255);
        let img = load_image_bytes(&png_bytes).unwrap();
        let mut builder = AtlasBuilder::new(256, 256);

        // Act
        let handle = builder.add_image(img.width, img.height, &img.data).unwrap();

        // Assert — UV rect encodes a 1x1 pixel region
        let [u0, v0, u1, v1] = handle.uv_rect;
        let pixel_w = ((u1 - u0) * 256.0).round() as u32;
        let pixel_h = ((v1 - v0) * 256.0).round() as u32;
        assert_eq!(pixel_w, 1);
        assert_eq!(pixel_h, 1);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When loading valid png, then returns correct image data</summary>

<code>crates\engine_render\src\atlas.rs:509</code>

```rust
        // Arrange
        let png_bytes = make_1x1_png(255, 0, 0, 255);

        // Act
        let img = load_image_bytes(&png_bytes).unwrap();

        // Assert
        assert_eq!(img.width, 1);
        assert_eq!(img.height, 1);
        assert_eq!(img.data, vec![255, 0, 0, 255]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When looking up known texture id, then returns matching uv rect</summary>

<code>crates\engine_render\src\atlas.rs:411</code>

```rust
        // Arrange
        let mut builder = AtlasBuilder::new(256, 256);
        let handle = builder.add_image(2, 2, &[255; 16]).unwrap();
        let atlas = builder.build();

        // Act
        let result = atlas.lookup(handle.texture_id);

        // Assert
        assert_eq!(result, Some(handle.uv_rect));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When looking up multiple textures, then each returns its own uv rect</summary>

<code>crates\engine_render\src\atlas.rs:439</code>

```rust
        // Arrange
        let mut builder = AtlasBuilder::new(64, 64);
        let h1 = builder.add_image(4, 4, &[255; 64]).unwrap();
        let h2 = builder.add_image(4, 4, &[128; 64]).unwrap();
        let h3 = builder.add_image(4, 4, &[0; 64]).unwrap();
        let atlas = builder.build();

        // Act + Assert
        assert_eq!(atlas.lookup(h1.texture_id), Some(h1.uv_rect));
        assert_eq!(atlas.lookup(h2.texture_id), Some(h2.uv_rect));
        assert_eq!(atlas.lookup(h3.texture_id), Some(h3.uv_rect));
        assert_ne!(h1.uv_rect, h2.uv_rect);
        assert_ne!(h2.uv_rect, h3.uv_rect);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When looking up unknown texture id, then returns none</summary>

<code>crates\engine_render\src\atlas.rs:425</code>

```rust
        // Arrange
        let mut builder = AtlasBuilder::new(256, 256);
        builder.add_image(2, 2, &[255; 16]).unwrap();
        let atlas = builder.build();

        // Act
        let result = atlas.lookup(TextureId(99));

        // Assert
        assert_eq!(result, None);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When normalizing uv rect, then output is in zero one range</summary>

<code>crates\engine_render\src\atlas.rs:550</code>

```rust
        // Act
        let uv = normalize_uv_rect(10, 20, 40, 60, 100, 100);

        // Assert
        assert_eq!(uv, [0.10, 0.20, 0.50, 0.80]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When normalizing uv rect at origin, then starts at zero</summary>

<code>crates\engine_render\src\atlas.rs:559</code>

```rust
        // Act
        let uv = normalize_uv_rect(0, 0, 32, 32, 256, 256);

        // Assert
        assert_eq!(uv, [0.0, 0.0, 0.125, 0.125]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When second image at nonzero y, then uv height matches image ratio</summary>

<code>crates\engine_render\src\atlas.rs:603</code>

```rust
        // Arrange — narrow atlas forces second image to y > 0
        let mut builder = AtlasBuilder::new(4, 8);
        builder.add_image(4, 4, &[0u8; 64]).unwrap();

        // Act
        let h2 = builder.add_image(2, 2, &[0u8; 16]).unwrap();

        // Assert
        let uv_height = h2.uv_rect[3] - h2.uv_rect[1];
        let expected = 2.0 / 8.0;
        assert!(
            (uv_height - expected).abs() < 1e-6,
            "UV height {uv_height} should be {expected}"
        );
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When no atlas, then upload atlas not called</summary>

<code>crates\engine_render\src\atlas.rs:683</code>

```rust
        // Arrange
        let mut world = bevy_ecs::world::World::new();
        let log = insert_spy(&mut world);

        // Act
        run_system(&mut world);

        // Assert
        assert!(!log.lock().unwrap().contains(&"upload_atlas".to_string()));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When second image offset, then handle uv matches build lookup</summary>

<code>crates\engine_render\src\atlas.rs:621</code>

```rust
        // Arrange — narrow atlas forces second image to y > 0
        let mut builder = AtlasBuilder::new(4, 8);
        builder.add_image(4, 4, &[0u8; 4 * 4 * 4]).unwrap();
        let data = [255u8; 2 * 2 * 4];
        let handle = builder.add_image(2, 2, &data).unwrap();

        // Act
        let atlas = builder.build();

        // Assert — handle UV (from add_image) must match lookup (from build)
        let looked_up = atlas.lookup(handle.texture_id).unwrap();
        assert_eq!(handle.uv_rect, looked_up);

        // Also verify pixel data at the UV location
        let [u0, v0, _, _] = handle.uv_rect;
        let px = (u0 * atlas.width as f32) as usize;
        let py = (v0 * atlas.height as f32) as usize;
        let stride = atlas.width as usize * 4;
        let off = py * stride + px * 4;
        assert_eq!(&atlas.data[off..off + 4], [255, 255, 255, 255]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When system runs twice, then upload atlas called only once</summary>

<code>crates\engine_render\src\atlas.rs:696</code>

```rust
        // Arrange
        let mut world = bevy_ecs::world::World::new();
        let log = insert_spy(&mut world);
        world.insert_resource(minimal_atlas());
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(upload_atlas_system);

        // Act
        schedule.run(&mut world);
        schedule.run(&mut world);

        // Assert
        let calls: Vec<_> = log
            .lock()
            .unwrap()
            .iter()
            .filter(|s| *s == "upload_atlas")
            .cloned()
            .collect();
        assert_eq!(calls.len(), 1);
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test bloom</strong> (9 tests)</summary>

- System tests::when bloom disabled then post process system skips
<blockquote>
<details>
<summary>✅ When gaussian weights computed, then kernel is symmetric</summary>

*Symmetry allows separable (H+V) blur — same kernel for both passes*

<code>crates\engine_render\src\bloom.rs:83</code>

```rust
        // Act
        let weights = compute_gaussian_weights(3);

        // Assert
        let n = weights.len();
        for i in 0..=3 {
            let mirror = n - 1 - i;
            assert!(
                (weights[i] - weights[mirror]).abs() < 1e-5,
                "weights[{i}]={} != weights[{mirror}]={}",
                weights[i],
                weights[mirror],
            );
        }
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When gaussian weights radius0, then single weight of one</summary>

<code>crates\engine_render\src\bloom.rs:101</code>

```rust
        // Act
        let weights = compute_gaussian_weights(0);

        // Assert
        assert_eq!(weights.len(), 1);
        assert!(
            (weights[0] - 1.0).abs() < 1e-5,
            "single weight must be 1.0, got {}",
            weights[0],
        );
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When gaussian weights radius1, then center is largest</summary>

<code>crates\engine_render\src\bloom.rs:56</code>

```rust
        // Act
        let weights = compute_gaussian_weights(1);

        // Assert
        assert_eq!(weights.len(), 3);
        assert!(weights[1] > weights[0]);
        assert!(weights[1] > weights[2]);
```

</details>
</blockquote>
- System tests::when no bloom settings then post process system skips
- System tests::when post process system runs then log records apply post process
<blockquote>
<details>
<summary>✅ When gaussian weights radius3, then sum is one</summary>

*Normalized kernel ensures bloom doesn't change overall image brightness*

<code>crates\engine_render\src\bloom.rs:68</code>

```rust
        // Act
        let weights = compute_gaussian_weights(3);

        // Assert
        assert_eq!(weights.len(), 7);
        let sum: f32 = weights.iter().sum();
        assert!(
            (sum - 1.0).abs() < 1e-5,
            "weights must sum to 1.0, got {sum}"
        );
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When gaussian weights radius3, then weight ratios match formula</summary>

<code>crates\engine_render\src\bloom.rs:238</code>

```rust
        // The Gaussian formula: w(x) = exp(-x^2 / (2*sigma^2)), sigma = max(radius, 1)
        // For radius=3, sigma=3: denominator = 2*3*3 = 18
        // This radius is chosen so 2*sigma != 2+sigma (6 != 5),
        // catching mutations like `2.0 * sigma` → `2.0 + sigma`.
        let weights = compute_gaussian_weights(3);

        // Assert
        assert_eq!(weights.len(), 7);
        let center = weights[3]; // x=0
        let adjacent = weights[2]; // x=-1
        let edge = weights[0]; // x=-3

        // Ratio w(0)/w(1) = exp(1/18)
        let expected_adj_ratio = (1.0_f32 / 18.0).exp();
        let actual_adj_ratio = center / adjacent;
        assert!(
            (actual_adj_ratio - expected_adj_ratio).abs() < 1e-4,
            "center/adjacent ratio: expected {expected_adj_ratio}, got {actual_adj_ratio}"
        );

        // Ratio w(0)/w(3) = exp(9/18) = exp(0.5)
        let expected_edge_ratio = (0.5_f32).exp();
        let actual_edge_ratio = center / edge;
        assert!(
            (actual_edge_ratio - expected_edge_ratio).abs() < 1e-4,
            "center/edge ratio: expected {expected_edge_ratio}, got {actual_edge_ratio}"
        );
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When any radius, then gaussian weights sum to one and are symmetric</summary>

<code>crates\engine_render\src\bloom.rs:196</code>

```rust
            radius in 0u32..=16,
        ) {
            // Act
            let weights = compute_gaussian_weights(radius);
            let size = weights.len();

            // Assert — length
            assert_eq!(size, (2 * radius + 1) as usize);

            // Assert — sum to 1.0
            let sum: f32 = weights.iter().sum();
            assert!(
                (sum - 1.0).abs() < 1e-5,
                "weights must sum to 1.0, got {sum}"
            );

            // Assert — symmetry
            for i in 0..size / 2 {
                let mirror = size - 1 - i;
                assert!(
                    (weights[i] - weights[mirror]).abs() < 1e-6,
                    "weights[{i}]={} != weights[{mirror}]={}",
                    weights[i],
                    weights[mirror],
                );
            }

            // Assert — center is maximum
            let center = radius as usize;
            for (i, w) in weights.iter().enumerate() {
                assert!(
                    weights[center] >= *w,
                    "center {} should be >= weights[{i}]={}",
                    weights[center],
                    w,
                );
            }
        }
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test camera</strong> (31 tests)</summary>

<blockquote>
<details>
<summary>✅ When camera2d created with defaults, then position is zero and zoom is one</summary>

<code>crates\engine_render\src\camera.rs:144</code>

```rust
        // Act
        let camera = Camera2D::default();

        // Assert
        assert_eq!(camera.position, Vec2::ZERO);
        assert_eq!(camera.zoom, 1.0);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When camera at nonzero position, then view matrix translates by negative position</summary>

<code>crates\engine_render\src\camera.rs:166</code>

```rust
        // Arrange
        let camera = Camera2D {
            position: Vec2::new(100.0, 200.0),
            zoom: 1.0,
        };

        // Act
        let view = compute_view_matrix(&camera);

        // Assert
        let translation = view.w_axis;
        assert!((translation.x - (-100.0)).abs() < 1e-6);
        assert!((translation.y - (-200.0)).abs() < 1e-6);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When camera at origin with zoom half, then view matrix scales by half</summary>

<code>crates\engine_render\src\camera.rs:199</code>

```rust
        // Arrange
        let camera = Camera2D {
            position: Vec2::ZERO,
            zoom: 0.5,
        };

        // Act
        let view = compute_view_matrix(&camera);

        // Assert
        assert!((view.x_axis.x - 0.5).abs() < 1e-6);
        assert!((view.y_axis.y - 0.5).abs() < 1e-6);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When camera at origin with zoom one, then view matrix is identity</summary>

<code>crates\engine_render\src\camera.rs:154</code>

```rust
        // Arrange
        let camera = Camera2D::default();

        // Act
        let view = compute_view_matrix(&camera);

        // Assert
        assert_eq!(view, Mat4::IDENTITY);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When camera2d serialized to ron, then deserializes to equal value</summary>

<code>crates\engine_render\src\camera.rs:128</code>

```rust
        // Arrange
        let camera = Camera2D {
            position: Vec2::new(150.0, -75.0),
            zoom: 2.5,
        };

        // Act
        let ron = ron::to_string(&camera).unwrap();
        let back: Camera2D = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(camera, back);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When camera at origin with zoom two, then view matrix scales by two</summary>

<code>crates\engine_render\src\camera.rs:183</code>

```rust
        // Arrange
        let camera = Camera2D {
            position: Vec2::ZERO,
            zoom: 2.0,
        };

        // Act
        let view = compute_view_matrix(&camera);

        // Assert
        assert!((view.x_axis.x - 2.0).abs() < 1e-6);
        assert!((view.y_axis.y - 2.0).abs() < 1e-6);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When camera at position with nonunit zoom, then view matrix combines translation and scale</summary>

<code>crates\engine_render\src\camera.rs:215</code>

```rust
        // Arrange
        let camera = Camera2D {
            position: Vec2::new(50.0, 100.0),
            zoom: 2.0,
        };

        // Act
        let view = compute_view_matrix(&camera);

        // Assert
        assert!((view.x_axis.x - 2.0).abs() < 1e-6);
        assert!((view.y_axis.y - 2.0).abs() < 1e-6);
        assert!((view.w_axis.x - (-100.0)).abs() < 1e-6);
        assert!((view.w_axis.y - (-200.0)).abs() < 1e-6);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When camera prepare system runs with camera, then set view projection called</summary>

<code>crates\engine_render\src\camera.rs:532</code>

```rust
        // Arrange
        let mut world = bevy_ecs::world::World::new();
        let log = insert_spy_with_viewport(&mut world, 800, 600);
        world.spawn(Camera2D::default());

        // Act
        run_camera_prepare(&mut world);

        // Assert
        let log = log.lock().unwrap();
        assert!(log.contains(&"set_view_projection".to_string()));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When camera prepare system runs without camera, then default ortho set</summary>

*`camera_prepare_system` always sets a projection — defaults to viewport-centered ortho when no `Camera2D` entity exists*

<code>crates\engine_render\src\camera.rs:548</code>

```rust
        // Arrange
        let mut world = bevy_ecs::world::World::new();
        let log = insert_spy_with_viewport(&mut world, 800, 600);

        // Act
        run_camera_prepare(&mut world);

        // Assert
        let log = log.lock().unwrap();
        assert!(log.contains(&"set_view_projection".to_string()));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When camera uniform from camera at center, then viewport corners map to ndc corners</summary>

<code>crates\engine_render\src\camera.rs:511</code>

```rust
        // Arrange
        let camera = Camera2D {
            position: Vec2::new(400.0, 300.0),
            zoom: 1.0,
        };

        // Act
        let uniform = CameraUniform::from_camera(&camera, 800.0, 600.0);

        // Assert
        let vp = Mat4::from_cols_array_2d(&uniform.view_proj);
        let top_left = vp * glam::Vec4::new(0.0, 0.0, 0.0, 1.0);
        let bottom_right = vp * glam::Vec4::new(800.0, 600.0, 0.0, 1.0);
        assert!((top_left.x - (-1.0)).abs() < 1e-5);
        assert!((top_left.y - 1.0).abs() < 1e-5);
        assert!((bottom_right.x - 1.0).abs() < 1e-5);
        assert!((bottom_right.y - (-1.0)).abs() < 1e-5);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When camera uniform from camera at origin zoom one, then origin maps to ndc center</summary>

*Default camera produces pixel-perfect 1:1 mapping — world origin lands at NDC center*

<code>crates\engine_render\src\camera.rs:479</code>

```rust
        // Arrange
        let camera = Camera2D::default();

        // Act
        let uniform = CameraUniform::from_camera(&camera, 800.0, 600.0);

        // Assert — camera at origin: world (0,0) is the view center → NDC (0,0)
        let vp = Mat4::from_cols_array_2d(&uniform.view_proj);
        let origin_ndc = vp * glam::Vec4::new(0.0, 0.0, 0.0, 1.0);
        assert!((origin_ndc.x).abs() < 1e-5);
        assert!((origin_ndc.y).abs() < 1e-5);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When camera uniform y flip, then top maps to positive ndc y</summary>

<code>crates\engine_render\src\camera.rs:562</code>

```rust
        // Arrange — camera centered on viewport, zoom 1
        let camera = Camera2D {
            position: Vec2::new(400.0, 300.0),
            zoom: 1.0,
        };

        // Act
        let uniform = CameraUniform::from_camera(&camera, 800.0, 600.0);

        // Assert — top of viewport (y=0) maps to NDC y=+1 (Y-flip)
        let vp = Mat4::from_cols_array_2d(&uniform.view_proj);
        let top_center = vp * glam::Vec4::new(400.0, 0.0, 0.0, 1.0);
        assert!(
            (top_center.y - 1.0).abs() < 1e-4,
            "top of viewport should map to NDC y=+1, got {}",
            top_center.y
        );
        // Bottom of viewport (y=600) maps to NDC y=-1
        let bottom_center = vp * glam::Vec4::new(400.0, 600.0, 0.0, 1.0);
        assert!(
            (bottom_center.y - (-1.0)).abs() < 1e-4,
            "bottom of viewport should map to NDC y=-1, got {}",
            bottom_center.y
        );
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When entity completely above view, then aabb intersects returns false</summary>

<code>crates\engine_render\src\camera.rs:423</code>

```rust
        // Act / Assert
        assert!(!aabb_intersects_view_rect(
            Vec2::new(0.0, -200.0),
            Vec2::new(100.0, -10.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(800.0, 600.0),
        ));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When entity completely below view, then aabb intersects returns false</summary>

<code>crates\engine_render\src\camera.rs:434</code>

```rust
        // Act / Assert
        assert!(!aabb_intersects_view_rect(
            Vec2::new(0.0, 650.0),
            Vec2::new(100.0, 800.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(800.0, 600.0),
        ));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When entity completely left of view, then aabb intersects returns false</summary>

*Frustum culling AABB test — entity fully outside on any axis means no intersection*

<code>crates\engine_render\src\camera.rs:401</code>

```rust
        // Act / Assert
        assert!(!aabb_intersects_view_rect(
            Vec2::new(-200.0, 0.0),
            Vec2::new(-10.0, 100.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(800.0, 600.0),
        ));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When entity completely right of view, then aabb intersects returns false</summary>

<code>crates\engine_render\src\camera.rs:412</code>

```rust
        // Act / Assert
        assert!(!aabb_intersects_view_rect(
            Vec2::new(850.0, 0.0),
            Vec2::new(1000.0, 100.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(800.0, 600.0),
        ));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When entity contains entire view, then aabb intersects returns true</summary>

<code>crates\engine_render\src\camera.rs:467</code>

```rust
        // Act / Assert
        assert!(aabb_intersects_view_rect(
            Vec2::new(0.0, 0.0),
            Vec2::new(800.0, 600.0),
            Vec2::new(100.0, 100.0),
            Vec2::new(700.0, 500.0),
        ));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When entity exactly touches view edge, then aabb intersects returns true</summary>

<code>crates\engine_render\src\camera.rs:456</code>

```rust
        // Act / Assert
        assert!(aabb_intersects_view_rect(
            Vec2::new(-10.0, 0.0),
            Vec2::new(0.0, 100.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(800.0, 600.0),
        ));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When entity fully inside view, then aabb intersects returns true</summary>

<code>crates\engine_render\src\camera.rs:389</code>

```rust
        // Act / Assert
        assert!(aabb_intersects_view_rect(
            Vec2::new(100.0, 100.0),
            Vec2::new(200.0, 200.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(800.0, 600.0),
        ));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When screen center, then screen to world returns camera position</summary>

<code>crates\engine_render\src\camera.rs:283</code>

```rust
        // Arrange
        let camera = Camera2D {
            position: Vec2::new(400.0, 300.0),
            zoom: 1.0,
        };

        // Act
        let world = screen_to_world(Vec2::new(400.0, 300.0), &camera, 800.0, 600.0);

        // Assert
        assert!((world.x - 400.0).abs() < 1e-4);
        assert!((world.y - 300.0).abs() < 1e-4);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When no camera, then system uses viewport center</summary>

<code>crates\engine_render\src\camera.rs:626</code>

```rust
        // Arrange
        let mut world = bevy_ecs::world::World::new();
        let matrix: crate::testing::MatrixCapture = Arc::new(Mutex::new(None));
        let log = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log)
            .with_viewport(800, 600)
            .with_matrix_capture(matrix.clone());
        world.insert_resource(RendererRes::new(Box::new(spy)));

        // Act
        run_camera_prepare(&mut world);

        // Assert
        let actual = matrix.lock().unwrap().unwrap();
        let expected_cam = Camera2D {
            position: Vec2::new(400.0, 300.0),
            zoom: 1.0,
        };
        let expected = CameraUniform::from_camera(&expected_cam, 800.0, 600.0);
        assert_eq!(actual, expected.view_proj);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When screen to world after world to screen, then recovers original point</summary>

*`world_to_screen` and `screen_to_world` are exact inverses — roundtrip recovers the original point*

<code>crates\engine_render\src\camera.rs:300</code>

```rust
        // Arrange
        let camera = Camera2D {
            position: Vec2::new(150.0, 75.0),
            zoom: 1.5,
        };
        let original = Vec2::new(200.0, 100.0);

        // Act
        let screen = world_to_screen(original, &camera, 800.0, 600.0);
        let recovered = screen_to_world(screen, &camera, 800.0, 600.0);

        // Assert
        assert!((recovered.x - original.x).abs() < 1e-4);
        assert!((recovered.y - original.y).abs() < 1e-4);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When entity partially overlaps left edge, then aabb intersects returns true</summary>

<code>crates\engine_render\src\camera.rs:445</code>

```rust
        // Act / Assert
        assert!(aabb_intersects_view_rect(
            Vec2::new(-50.0, 100.0),
            Vec2::new(50.0, 200.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(800.0, 600.0),
        ));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When view rect at zoom one, then half extents equal half viewport</summary>

<code>crates\engine_render\src\camera.rs:353</code>

```rust
        // Arrange
        let camera = Camera2D {
            position: Vec2::new(400.0, 300.0),
            zoom: 1.0,
        };

        // Act
        let (min, max) = camera_view_rect(&camera, 800.0, 600.0);

        // Assert
        assert!((min.x - 0.0).abs() < 1e-4);
        assert!((min.y - 0.0).abs() < 1e-4);
        assert!((max.x - 800.0).abs() < 1e-4);
        assert!((max.y - 600.0).abs() < 1e-4);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When view rect at zoom two, then half extents are halved</summary>

<code>crates\engine_render\src\camera.rs:371</code>

```rust
        // Arrange
        let camera = Camera2D {
            position: Vec2::new(400.0, 300.0),
            zoom: 2.0,
        };

        // Act
        let (min, max) = camera_view_rect(&camera, 800.0, 600.0);

        // Assert
        assert!((min.x - 200.0).abs() < 1e-4);
        assert!((min.y - 150.0).abs() < 1e-4);
        assert!((max.x - 600.0).abs() < 1e-4);
        assert!((max.y - 450.0).abs() < 1e-4);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When world point at viewport corner, then world to screen returns corner</summary>

<code>crates\engine_render\src\camera.rs:250</code>

```rust
        // Arrange
        let camera = Camera2D {
            position: Vec2::new(400.0, 300.0),
            zoom: 1.0,
        };

        // Act
        let screen = world_to_screen(Vec2::new(800.0, 600.0), &camera, 800.0, 600.0);

        // Assert
        assert!((screen.x - 800.0).abs() < 1e-4);
        assert!((screen.y - 600.0).abs() < 1e-4);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When world point at zoom two, then world to screen reflects magnification</summary>

*Zoom multiplies screen-space distances — zoom 2 means objects appear 2x larger*

<code>crates\engine_render\src\camera.rs:267</code>

```rust
        // Arrange
        let camera = Camera2D {
            position: Vec2::new(400.0, 300.0),
            zoom: 2.0,
        };

        // Act
        let screen = world_to_screen(Vec2::new(450.0, 300.0), &camera, 800.0, 600.0);

        // Assert
        assert!((screen.x - 500.0).abs() < 1e-4);
        assert!((screen.y - 300.0).abs() < 1e-4);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When world point matches camera center, then world to screen returns screen center</summary>

*Camera position defines the world point that appears at screen center*

<code>crates\engine_render\src\camera.rs:234</code>

```rust
        // Arrange
        let camera = Camera2D {
            position: Vec2::new(400.0, 300.0),
            zoom: 1.0,
        };

        // Act
        let screen = world_to_screen(Vec2::new(400.0, 300.0), &camera, 800.0, 600.0);

        // Assert
        assert!((screen.x - 400.0).abs() < 1e-4);
        assert!((screen.y - 300.0).abs() < 1e-4);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When viewport width zero, then camera prepare system skips</summary>

<code>crates\engine_render\src\camera.rs:590</code>

```rust
        // Arrange
        let mut world = bevy_ecs::world::World::new();
        let log = insert_spy_with_viewport(&mut world, 0, 600);
        world.spawn(Camera2D::default());

        // Act
        run_camera_prepare(&mut world);

        // Assert
        let log = log.lock().unwrap();
        assert!(
            !log.contains(&"set_view_projection".to_string()),
            "should skip when viewport width is zero"
        );
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When viewport height zero, then camera prepare system skips</summary>

<code>crates\engine_render\src\camera.rs:608</code>

```rust
        // Arrange
        let mut world = bevy_ecs::world::World::new();
        let log = insert_spy_with_viewport(&mut world, 800, 0);
        world.spawn(Camera2D::default());

        // Act
        run_camera_prepare(&mut world);

        // Assert
        let log = log.lock().unwrap();
        assert!(
            !log.contains(&"set_view_projection".to_string()),
            "should skip when viewport height is zero"
        );
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When any world point, then screen to world of world to screen recovers original</summary>

<code>crates\engine_render\src\camera.rs:319</code>

```rust
            wx in -1000.0_f32..=1000.0,
            wy in -1000.0_f32..=1000.0,
            cx in -500.0_f32..=500.0,
            cy in -500.0_f32..=500.0,
            zoom in 0.1_f32..=10.0,
            vw in 1.0_f32..=2000.0,
            vh in 1.0_f32..=2000.0,
        ) {
            // Arrange
            let camera = Camera2D { position: Vec2::new(cx, cy), zoom };
            let point = Vec2::new(wx, wy);

            // Act
            let screen = world_to_screen(point, &camera, vw, vh);
            let recovered = screen_to_world(screen, &camera, vw, vh);

            // Assert
            assert!(
                (recovered.x - point.x).abs() < 1e-2,
                "x: expected {}, got {}",
                point.x,
                recovered.x
            );
            assert!(
                (recovered.y - point.y).abs() < 1e-2,
                "y: expected {}, got {}",
                point.y,
                recovered.y
            );
        }
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test clear</strong> (1 tests)</summary>

<blockquote>
<details>
<summary>✅ When clear system runs, then renderer clear receives clear color value</summary>

<code>crates\engine_render\src\clear.rs:31</code>

```rust
        // Arrange
        let expected_color = Color::new(0.1, 0.2, 0.3, 1.0);
        let log = Arc::new(Mutex::new(Vec::new()));
        let color_capture = Arc::new(Mutex::new(None));
        let spy = SpyRenderer::new(log.clone()).with_color_capture(color_capture.clone());

        let mut world = bevy_ecs::world::World::new();
        world.insert_resource(RendererRes::new(Box::new(spy)));
        world.insert_resource(ClearColor(expected_color));

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(clear_system);

        // Act
        schedule.run(&mut world);

        // Assert
        assert_eq!(*color_capture.lock().unwrap(), Some(expected_color));
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test material</strong> (16 tests)</summary>

<blockquote>
<details>
<summary>✅ When all blend mode variants call index, then each returns discriminant</summary>

<code>crates\engine_render\src\material.rs:310</code>

```rust
        // Act / Assert
        assert_eq!(BlendMode::Alpha.index(), 0);
        assert_eq!(BlendMode::Additive.index(), 1);
        assert_eq!(BlendMode::Multiply.index(), 2);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When comparing blend modes, then alpha less than additive less than multiply</summary>

<code>crates\engine_render\src\material.rs:185</code>

```rust
        // Arrange
        let alpha = BlendMode::Alpha;
        let additive = BlendMode::Additive;
        let multiply = BlendMode::Multiply;

        // Act / Assert
        assert!(alpha < additive);
        assert!(additive < multiply);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When comparing shader handles, then ordered by inner u32</summary>

<code>crates\engine_render\src\material.rs:298</code>

```rust
        // Arrange
        let a = ShaderHandle(0);
        let b = ShaderHandle(1);
        let c = ShaderHandle(99);

        // Act / Assert
        assert!(a < b);
        assert!(b < c);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When effective shader handle with none, then returns default</summary>

<code>crates\engine_render\src\material.rs:274</code>

```rust
        // Act
        let result = effective_shader_handle(None);

        // Assert
        assert_eq!(result, ShaderHandle(0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When blend mode variants serialized to ron, then each deserializes to matching variant</summary>

<code>crates\engine_render\src\material.rs:176</code>

```rust
        for mode in BlendMode::ALL {
            let ron = ron::to_string(&mode).unwrap();
            let back: BlendMode = ron::from_str(&ron).unwrap();
            assert_eq!(mode, back);
        }
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When effective shader handle with some, then returns material shader</summary>

<code>crates\engine_render\src\material.rs:283</code>

```rust
        // Arrange
        let material = Material2d {
            shader: ShaderHandle(99),
            ..Material2d::default()
        };

        // Act
        let result = effective_shader_handle(Some(&material));

        // Assert
        assert_eq!(result, ShaderHandle(99));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When looking up unregistered handle, then returns none</summary>

<code>crates\engine_render\src\material.rs:224</code>

```rust
        // Arrange
        let registry = ShaderRegistry::new();

        // Act
        let result = registry.lookup(ShaderHandle(99));

        // Assert
        assert_eq!(result, None);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When preprocessing multiple lines, then separated by newlines</summary>

<code>crates\engine_render\src\material.rs:369</code>

```rust
        // Arrange
        let source = "line_one\nline_two\nline_three";
        let defines = HashSet::new();

        // Act
        let result = preprocess(source, &defines);

        // Assert
        assert_eq!(result, "line_one\nline_two\nline_three");
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When preprocessing nested ifdef with outer defined inner not, then inner excluded</summary>

<code>crates\engine_render\src\material.rs:318</code>

```rust
        // Arrange
        let source = "before\n#ifdef OUTER\nmiddle\n#ifdef INNER\nskipped\n#endif\nafter_inner\n#endif\nfooter";
        let mut defines = HashSet::new();
        defines.insert("OUTER");

        // Act
        let result = preprocess(source, &defines);

        // Assert
        assert!(result.contains("before"));
        assert!(result.contains("middle"));
        assert!(!result.contains("skipped"));
        assert!(result.contains("after_inner"));
        assert!(result.contains("footer"));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When preprocessing outer undefined inner defined, then entire block excluded</summary>

<code>crates\engine_render\src\material.rs:351</code>

```rust
        // Arrange — OUTER undefined, INNER defined; everything inside OUTER must be skipped
        let source = "before\n#ifdef OUTER\nouter_only\n#ifdef INNER\ninner_only\n#endif\nafter_inner\n#endif\nfooter";
        let mut defines = HashSet::new();
        defines.insert("INNER");

        // Act
        let result = preprocess(source, &defines);

        // Assert
        assert!(result.contains("before"));
        assert!(result.contains("footer"));
        assert!(!result.contains("outer_only"));
        assert!(!result.contains("inner_only"));
        assert!(!result.contains("after_inner"));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When preprocessing with define present, then ifdef block included</summary>

*#ifdef preprocessor conditionally includes shader blocks — enables feature-based shader variants*

<code>crates\engine_render\src\material.rs:237</code>

```rust
        // Arrange
        let source = "header\n#ifdef MY_FEATURE\nfeature_line\n#endif\nfooter";
        let mut defines = HashSet::new();
        defines.insert("MY_FEATURE");

        // Act
        let result = preprocess(source, &defines);

        // Assert
        assert!(result.contains("feature_line"));
        assert!(result.contains("header"));
        assert!(result.contains("footer"));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When preprocessing without define, then ifdef block excluded</summary>

<code>crates\engine_render\src\material.rs:336</code>

```rust
        // Arrange
        let source = "header\n#ifdef MY_FEATURE\nfeature_line\n#endif\nfooter";
        let defines = HashSet::new();

        // Act
        let result = preprocess(source, &defines);

        // Assert
        assert!(!result.contains("feature_line"));
        assert!(result.contains("header"));
        assert!(result.contains("footer"));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When registering multiple shaders, then handles are unique</summary>

<code>crates\engine_render\src\material.rs:211</code>

```rust
        // Arrange
        let mut registry = ShaderRegistry::new();

        // Act
        let h1 = registry.register("shader_a");
        let h2 = registry.register("shader_b");

        // Assert
        assert_ne!(h1, h2);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When registering shader, then lookup returns same source</summary>

<code>crates\engine_render\src\material.rs:197</code>

```rust
        // Arrange
        let mut registry = ShaderRegistry::new();
        let source = "@vertex fn vs_main() {}";

        // Act
        let handle = registry.register(source);
        let result = registry.lookup(handle);

        // Assert
        assert_eq!(result, Some(source));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When shader registry used as resource in system, then lookup works</summary>

<code>crates\engine_render\src\material.rs:253</code>

```rust
        use bevy_ecs::prelude::{Res, Schedule, World};

        // Arrange
        let mut registry = ShaderRegistry::new();
        let _handle = registry.register("@vertex fn vs_main() {}");
        let mut world = World::new();
        world.insert_resource(registry);
        let mut schedule = Schedule::default();
        schedule.add_systems(|registry: Res<ShaderRegistry>| {
            assert_eq!(
                registry.lookup(ShaderHandle(0)),
                Some("@vertex fn vs_main() {}")
            );
        });

        // Act / Assert
        schedule.run(&mut world);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When material2d with textures and uniforms debug formatted, then snapshot matches</summary>

<code>crates\engine_render\src\material.rs:150</code>

```rust
        // Arrange
        let material = Material2d {
            blend_mode: BlendMode::Additive,
            shader: ShaderHandle(7),
            textures: vec![
                TextureBinding {
                    texture: TextureId(0),
                    binding: 0,
                },
                TextureBinding {
                    texture: TextureId(1),
                    binding: 1,
                },
            ],
            uniforms: vec![0, 128, 255],
        };

        // Act
        let debug = format!("{material:#?}");

        // Assert
        insta::assert_snapshot!(debug);
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test rect</strong> (2 tests)</summary>

<blockquote>
<details>
<summary>✅ When rect has negative pixel values, then stores without clamping</summary>

<code>crates\engine_render\src\rect.rs:54</code>

```rust
        // Act
        let r = Rect {
            x: Pixels(-10.0),
            y: Pixels(-20.0),
            width: Pixels(-100.0),
            height: Pixels(-50.0),
            color: Color::WHITE,
        };

        // Assert
        assert_eq!(r.x, Pixels(-10.0));
        assert_eq!(r.y, Pixels(-20.0));
        assert_eq!(r.width, Pixels(-100.0));
        assert_eq!(r.height, Pixels(-50.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When rect serialized to ron, then deserializes to equal value</summary>

<code>crates\engine_render\src\rect.rs:35</code>

```rust
        // Arrange
        let rect = Rect {
            x: Pixels(10.0),
            y: Pixels(-20.0),
            width: Pixels(100.0),
            height: Pixels(50.0),
            color: Color::new(0.5, 0.6, 0.7, 0.8),
        };

        // Act
        let ron = ron::to_string(&rect).unwrap();
        let back: Rect = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(rect, back);
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test renderer</strong> (13 tests)</summary>

<blockquote>
<details>
<summary>✅ When null renderer applies post process, then does not panic</summary>

<code>crates\engine_render\src\renderer.rs:198</code>

```rust
        // Arrange
        let mut renderer = NullRenderer;

        // Act
        renderer.apply_post_process();
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When null renderer clears, then does not panic</summary>

<code>crates\engine_render\src\renderer.rs:93</code>

```rust
        // Arrange
        let mut renderer = NullRenderer;

        // Act
        renderer.clear(Color::BLACK);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When null renderer draws rect, then does not panic</summary>

<code>crates\engine_render\src\renderer.rs:111</code>

```rust
        // Arrange
        let mut renderer = NullRenderer;

        // Act
        renderer.draw_rect(sample_rect());
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When null renderer draws shape, then does not panic</summary>

<code>crates\engine_render\src\renderer.rs:129</code>

```rust
        // Arrange
        let mut renderer = NullRenderer;
        let vertices = [[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]];
        let indices = [0u32, 1, 2];

        // Act
        renderer.draw_shape(&vertices, &indices, Color::WHITE);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When null renderer draws sprite, then does not panic</summary>

<code>crates\engine_render\src\renderer.rs:120</code>

```rust
        // Arrange
        let mut renderer = NullRenderer;

        // Act
        renderer.draw_sprite(sample_rect(), [0.0, 0.0, 1.0, 1.0]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When null renderer resizes, then does not panic</summary>

<code>crates\engine_render\src\renderer.rs:149</code>

```rust
        // Arrange
        let mut renderer = NullRenderer;

        // Act
        renderer.resize(800, 600);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When null renderer presents, then does not panic</summary>

<code>crates\engine_render\src\renderer.rs:102</code>

```rust
        // Arrange
        let mut renderer = NullRenderer;

        // Act
        renderer.present();
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When null renderer set shader and uniforms and texture, then does not panic</summary>

<code>crates\engine_render\src\renderer.rs:187</code>

```rust
        // Arrange
        let mut renderer = NullRenderer;

        // Act
        renderer.set_shader(crate::material::ShaderHandle(0));
        renderer.set_material_uniforms(&[1, 2, 3]);
        renderer.bind_material_texture(engine_core::types::TextureId(0), 2);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When null renderer set blend mode, then does not panic</summary>

<code>crates\engine_render\src\renderer.rs:176</code>

```rust
        // Arrange
        let mut renderer = NullRenderer;

        // Act
        renderer.set_blend_mode(BlendMode::Alpha);
        renderer.set_blend_mode(BlendMode::Additive);
        renderer.set_blend_mode(BlendMode::Multiply);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When null renderer upload atlas, then does not panic</summary>

<code>crates\engine_render\src\renderer.rs:167</code>

```rust
        // Arrange
        let mut renderer = NullRenderer;

        // Act
        renderer.upload_atlas(&minimal_atlas());
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When null renderer set view projection, then does not panic</summary>

<code>crates\engine_render\src\renderer.rs:140</code>

```rust
        // Arrange
        let mut renderer = NullRenderer;

        // Act
        renderer.set_view_projection([[0.0f32; 4]; 4]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When null renderer viewport size, then returns zero zero</summary>

<code>crates\engine_render\src\renderer.rs:207</code>

```rust
        // Arrange
        let renderer = NullRenderer;

        // Act
        let (w, h) = renderer.viewport_size();

        // Assert
        assert_eq!(w, 0);
        assert_eq!(h, 0);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When renderer res in world, then system can call clear via resmut</summary>

<code>crates\engine_render\src\renderer.rs:220</code>

```rust
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log.clone());
        let mut world = bevy_ecs::world::World::new();
        world.insert_resource(RendererRes::new(Box::new(spy)));
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(|mut renderer: bevy_ecs::prelude::ResMut<RendererRes>| {
            renderer.clear(Color::BLACK);
        });

        // Act
        schedule.run(&mut world);

        // Assert
        assert_eq!(log.lock().unwrap().as_slice(), &["clear"]);
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test shape</strong> (37 tests)</summary>

<blockquote>
<details>
<summary>✅ When circle aabb, then width and height equal double radius</summary>

<code>crates\engine_render\src\shape.rs:347</code>

```rust
        // Arrange
        let variant = ShapeVariant::Circle { radius: 50.0 };

        // Act
        let (min, max) = shape_aabb(&variant);

        // Assert
        assert_eq!(min, Vec2::new(-50.0, -50.0));
        assert_eq!(max, Vec2::new(50.0, 50.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When polygon aabb, then matches point extents</summary>

<code>crates\engine_render\src\shape.rs:360</code>

```rust
        // Arrange
        let variant = ShapeVariant::Polygon {
            points: vec![
                Vec2::new(-10.0, -20.0),
                Vec2::new(30.0, 40.0),
                Vec2::new(5.0, -5.0),
            ],
        };

        // Act
        let (min, max) = shape_aabb(&variant);

        // Assert
        assert_eq!(min, Vec2::new(-10.0, -20.0));
        assert_eq!(max, Vec2::new(30.0, 40.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When shape circle serialized to ron, then deserializes to equal value</summary>

<code>crates\engine_render\src\shape.rs:195</code>

```rust
        // Arrange
        let shape = Shape {
            variant: ShapeVariant::Circle { radius: 25.0 },
            color: Color::new(0.0, 1.0, 0.0, 1.0),
        };

        // Act
        let ron = ron::to_string(&shape).unwrap();
        let back: Shape = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(shape, back);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When shape barely inside view due to y radius, then drawn</summary>

<code>crates\engine_render\src\shape.rs:852</code>

```rust
        // Circle at (400, 590) r=30 → AABB y: [560, 620] overlaps view [0, 600].
        let mut world = World::new();
        let calls = insert_spy_with_shape_and_viewport(&mut world, 800, 600);
        world.spawn(Camera2D {
            position: Vec2::new(400.0, 300.0),
            zoom: 1.0,
        });
        world.spawn((
            default_shape(),
            GlobalTransform2D(Affine2::from_translation(Vec2::new(400.0, 590.0))),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = calls.lock().unwrap();
        assert_eq!(calls.len(), 1, "shape at bottom view edge should be drawn");
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When shape at negative pos inside view, then drawn</summary>

<code>crates\engine_render\src\shape.rs:896</code>

```rust
        // Circle r=30 at (-20,-20) → AABB [-50, 10] edge-touches view [-50, 50].
        let mut world = World::new();
        let calls = insert_spy_with_shape_and_viewport(&mut world, 100, 100);
        world.spawn(Camera2D {
            position: Vec2::new(0.0, 0.0),
            zoom: 1.0,
        });
        world.spawn((
            default_shape(),
            GlobalTransform2D(Affine2::from_translation(Vec2::new(-20.0, -20.0))),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = calls.lock().unwrap();
        assert_eq!(
            calls.len(),
            1,
            "shape at negative pos inside view should be drawn"
        );
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When shape at known position, then vertices offset by translation</summary>

<code>crates\engine_render\src\shape.rs:714</code>

```rust
        // Arrange
        let mut world = World::new();
        let calls = insert_spy_with_shape_capture(&mut world);
        world.spawn((
            default_shape(),
            GlobalTransform2D(Affine2::from_translation(Vec2::new(100.0, 200.0))),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        for vertex in &calls[0].0 {
            assert!(vertex[0] >= 100.0 - 30.0, "x={} should be >= 70", vertex[0]);
            assert!(
                vertex[1] >= 200.0 - 30.0,
                "y={} should be >= 170",
                vertex[1]
            );
        }
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When shape barely inside view due to radius, then drawn</summary>

<code>crates\engine_render\src\shape.rs:830</code>

```rust
        // Circle at (790, 300) r=30 → AABB [760, 820] overlaps view [0, 800].
        let mut world = World::new();
        let calls = insert_spy_with_shape_and_viewport(&mut world, 800, 600);
        world.spawn(Camera2D {
            position: Vec2::new(400.0, 300.0),
            zoom: 1.0,
        });
        world.spawn((
            default_shape(), // circle radius 30
            GlobalTransform2D(Affine2::from_translation(Vec2::new(790.0, 300.0))),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = calls.lock().unwrap();
        assert_eq!(calls.len(), 1, "shape at view edge should be drawn");
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When no camera entity, then all shapes drawn without culling</summary>

<code>crates\engine_render\src\shape.rs:1049</code>

```rust
        // Arrange
        let mut world = World::new();
        let log = insert_spy(&mut world);
        world.spawn((
            default_shape(),
            GlobalTransform2D(Affine2::from_translation(Vec2::new(5000.0, 5000.0))),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let count = log
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.as_str() == "draw_shape")
            .count();
        assert_eq!(count, 1);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When shape fully inside camera view, then drawn</summary>

<code>crates\engine_render\src\shape.rs:789</code>

```rust
        // Arrange
        let mut world = World::new();
        let log = insert_spy_with_viewport(&mut world, 800, 600);
        world.spawn(Camera2D {
            position: Vec2::new(400.0, 300.0),
            zoom: 1.0,
        });
        world.spawn((
            default_shape(),
            GlobalTransform2D(Affine2::from_translation(Vec2::new(400.0, 300.0))),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let count = log
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.as_str() == "draw_shape")
            .count();
        assert_eq!(count, 1);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When shape fully outside camera view, then not drawn</summary>

<code>crates\engine_render\src\shape.rs:762</code>

```rust
        // Arrange
        let mut world = World::new();
        let log = insert_spy_with_viewport(&mut world, 800, 600);
        world.spawn(Camera2D {
            position: Vec2::new(400.0, 300.0),
            zoom: 1.0,
        });
        world.spawn((
            default_shape(),
            GlobalTransform2D(Affine2::from_translation(Vec2::new(2000.0, 300.0))),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let count = log
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.as_str() == "draw_shape")
            .count();
        assert_eq!(count, 0);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When shape has material uniforms, then set material uniforms called</summary>

<code>crates\engine_render\src\shape.rs:959</code>

```rust
        // Arrange
        let mut world = World::new();
        let uniform_calls = insert_spy_with_uniform_capture(&mut world);
        world.spawn((
            default_shape(),
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                uniforms: vec![7, 8],
                ..Material2d::default()
            },
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = uniform_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[vec![7u8, 8]]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When shape has material, then set shader called with material shader</summary>

<code>crates\engine_render\src\shape.rs:922</code>

```rust
        // Arrange
        let mut world = World::new();
        let shader_calls = insert_spy_with_shader_capture(&mut world);
        world.spawn((
            default_shape(),
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                shader: ShaderHandle(3),
                ..Material2d::default()
            },
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = shader_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[ShaderHandle(3)]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When shape has no material, then set blend mode called with alpha</summary>

<code>crates\engine_render\src\shape.rs:455</code>

```rust
        // Arrange
        let mut world = World::new();
        let blend_calls = insert_spy_with_blend_capture(&mut world);
        world.spawn((default_shape(), GlobalTransform2D(Affine2::IDENTITY)));

        // Act
        run_system(&mut world);

        // Assert
        let calls = blend_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[BlendMode::Alpha]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When shape has additive material, then set blend mode called with additive</summary>

<code>crates\engine_render\src\shape.rs:470</code>

```rust
        // Arrange
        let mut world = World::new();
        let blend_calls = insert_spy_with_blend_capture(&mut world);
        world.spawn((
            default_shape(),
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                blend_mode: BlendMode::Additive,
                ..Material2d::default()
            },
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = blend_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[BlendMode::Additive]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When shape has no material, then set shader called with default</summary>

<code>crates\engine_render\src\shape.rs:944</code>

```rust
        // Arrange
        let mut world = World::new();
        let shader_calls = insert_spy_with_shader_capture(&mut world);
        world.spawn((default_shape(), GlobalTransform2D(Affine2::IDENTITY)));

        // Act
        run_system(&mut world);

        // Assert
        let calls = shader_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[ShaderHandle(0)]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When shape polygon serialized to ron, then deserializes with point order preserved</summary>

<code>crates\engine_render\src\shape.rs:211</code>

```rust
        // Arrange
        let shape = Shape {
            variant: ShapeVariant::Polygon {
                points: vec![
                    Vec2::new(0.0, 0.0),
                    Vec2::new(100.0, 0.0),
                    Vec2::new(50.0, 86.6),
                ],
            },
            color: Color::RED,
        };

        // Act
        let ron = ron::to_string(&shape).unwrap();
        let back: Shape = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(shape, back);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When shape near view min edge, then drawn</summary>

<code>crates\engine_render\src\shape.rs:874</code>

```rust
        // Circle r=30 at (5,5) → AABB [-25, 35] overlaps view [0, 100].
        let mut world = World::new();
        let calls = insert_spy_with_shape_and_viewport(&mut world, 100, 100);
        world.spawn(Camera2D {
            position: Vec2::new(50.0, 50.0),
            zoom: 1.0,
        });
        world.spawn((
            default_shape(),
            GlobalTransform2D(Affine2::from_translation(Vec2::new(5.0, 5.0))),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = calls.lock().unwrap();
        assert_eq!(calls.len(), 1, "shape near view min edge should be drawn");
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When shape with effective visibility false, then not drawn</summary>

<code>crates\engine_render\src\shape.rs:566</code>

```rust
        // Arrange
        let mut world = World::new();
        let log = insert_spy(&mut world);
        world.spawn((
            default_shape(),
            GlobalTransform2D(Affine2::IDENTITY),
            EffectiveVisibility(false),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let count = log
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.as_str() == "draw_shape")
            .count();
        assert_eq!(count, 0);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When shape with global transform, then draw shape called once</summary>

<code>crates\engine_render\src\shape.rs:526</code>

```rust
        // Arrange
        let mut world = World::new();
        let log = insert_spy(&mut world);
        world.spawn((default_shape(), GlobalTransform2D(Affine2::IDENTITY)));

        // Act
        run_system(&mut world);

        // Assert
        let count = log
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.as_str() == "draw_shape")
            .count();
        assert_eq!(count, 1);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When shape has texture bindings, then bind material texture called</summary>

<code>crates\engine_render\src\shape.rs:981</code>

```rust
        // Arrange
        let mut world = World::new();
        let texture_bind_calls = insert_spy_with_texture_bind_capture(&mut world);
        world.spawn((
            default_shape(),
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                textures: vec![TextureBinding {
                    texture: TextureId(4),
                    binding: 0,
                }],
                ..Material2d::default()
            },
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = texture_bind_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[(TextureId(4), 0)]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When shape has no render layer, then treated as world layer</summary>

<code>crates\engine_render\src\shape.rs:681</code>

```rust
        // Arrange
        let mut world = World::new();
        let calls = insert_spy_with_shape_capture(&mut world);
        let red = Color::new(1.0, 0.0, 0.0, 1.0);
        let blue = Color::new(0.0, 0.0, 1.0, 1.0);
        world.spawn((
            Shape {
                color: red,
                ..default_shape()
            },
            GlobalTransform2D(Affine2::IDENTITY),
        ));
        world.spawn((
            Shape {
                color: blue,
                ..default_shape()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            RenderLayer::Background,
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = calls.lock().unwrap();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].2, blue);
        assert_eq!(calls[1].2, red);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When tessellating circle, then all indices within vertex bounds</summary>

<code>crates\engine_render\src\shape.rs:259</code>

```rust
        // Arrange
        let variant = ShapeVariant::Circle { radius: 50.0 };

        // Act
        let mesh = tessellate(&variant);

        // Assert
        let vertex_count = mesh.vertices.len() as u32;
        for &index in &mesh.indices {
            assert!(
                index < vertex_count,
                "index {index} out of bounds (vertex count {vertex_count})"
            );
        }
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When shape without global transform, then draw shape not called</summary>

<code>crates\engine_render\src\shape.rs:546</code>

```rust
        // Arrange
        let mut world = World::new();
        let log = insert_spy(&mut world);
        world.spawn(default_shape());

        // Act
        run_system(&mut world);

        // Assert
        let count = log
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.as_str() == "draw_shape")
            .count();
        assert_eq!(count, 0);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When tessellating circle, then index count is multiple of three</summary>

<code>crates\engine_render\src\shape.rs:247</code>

```rust
        // Arrange
        let variant = ShapeVariant::Circle { radius: 50.0 };

        // Act
        let mesh = tessellate(&variant);

        // Assert
        assert_eq!(mesh.indices.len() % 3, 0);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When shape with known color, then draw shape receives matching color</summary>

<code>crates\engine_render\src\shape.rs:740</code>

```rust
        // Arrange
        let mut world = World::new();
        let calls = insert_spy_with_shape_capture(&mut world);
        let color = Color::new(1.0, 0.0, 0.0, 1.0);
        world.spawn((
            Shape {
                color,
                ..default_shape()
            },
            GlobalTransform2D(Affine2::IDENTITY),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = calls.lock().unwrap();
        assert_eq!(calls[0].2, color);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When tessellating circle, then produces nonempty vertices and indices</summary>

*Lyon `FillTessellator` generates triangle fan — all circle vertices lie at radius distance from origin*

<code>crates\engine_render\src\shape.rs:234</code>

```rust
        // Arrange
        let variant = ShapeVariant::Circle { radius: 50.0 };

        // Act
        let mesh = tessellate(&variant);

        // Assert
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When tessellating larger circle, then more vertices than smaller</summary>

<code>crates\engine_render\src\shape.rs:289</code>

```rust
        // Arrange
        let small = ShapeVariant::Circle { radius: 10.0 };
        let large = ShapeVariant::Circle { radius: 100.0 };

        // Act
        let small_mesh = tessellate(&small);
        let large_mesh = tessellate(&large);

        // Assert
        assert!(large_mesh.vertices.len() >= small_mesh.vertices.len());
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When tessellating polygon with fewer than three points, then returns empty mesh</summary>

*Degenerate polygons (< 3 vertices) produce empty mesh — no GPU draw call issued*

<code>crates\engine_render\src\shape.rs:380</code>

```rust
        // Arrange
        let variant = ShapeVariant::Polygon {
            points: vec![Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0)],
        };

        // Act
        let mesh = tessellate(&variant);

        // Assert
        assert!(mesh.vertices.is_empty());
        assert!(mesh.indices.is_empty());
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When tessellating quad polygon, then valid triangulated mesh</summary>

<code>crates\engine_render\src\shape.rs:322</code>

```rust
        // Arrange
        let variant = ShapeVariant::Polygon {
            points: vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(100.0, 0.0),
                Vec2::new(100.0, 100.0),
                Vec2::new(0.0, 100.0),
            ],
        };

        // Act
        let mesh = tessellate(&variant);

        // Assert
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());
        assert_eq!(mesh.indices.len() % 3, 0);
        let vertex_count = mesh.vertices.len() as u32;
        for &index in &mesh.indices {
            assert!(index < vertex_count);
        }
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When tessellating triangle polygon, then produces three vertices and three indices</summary>

<code>crates\engine_render\src\shape.rs:303</code>

```rust
        // Arrange
        let variant = ShapeVariant::Polygon {
            points: vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(100.0, 0.0),
                Vec2::new(50.0, 86.6),
            ],
        };

        // Act
        let mesh = tessellate(&variant);

        // Assert
        assert_eq!(mesh.vertices.len(), 3);
        assert_eq!(mesh.indices.len(), 3);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When tessellating zero radius circle, then does not panic</summary>

<code>crates\engine_render\src\shape.rs:277</code>

```rust
        // Arrange
        let variant = ShapeVariant::Circle { radius: 0.0 };

        // Act
        let mesh = tessellate(&variant);

        // Assert
        assert_eq!(mesh.indices.len() % 3, 0);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When two shapes on different layers, then background drawn before world</summary>

<code>crates\engine_render\src\shape.rs:611</code>

```rust
        // Arrange
        let mut world = World::new();
        let calls = insert_spy_with_shape_capture(&mut world);
        let red = Color::new(1.0, 0.0, 0.0, 1.0);
        let blue = Color::new(0.0, 0.0, 1.0, 1.0);
        world.spawn((
            Shape {
                color: red,
                ..default_shape()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            RenderLayer::World,
        ));
        world.spawn((
            Shape {
                color: blue,
                ..default_shape()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            RenderLayer::Background,
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = calls.lock().unwrap();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].2, blue);
        assert_eq!(calls[1].2, red);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When two shapes same layer different sort order, then lower drawn first</summary>

<code>crates\engine_render\src\shape.rs:645</code>

```rust
        // Arrange
        let mut world = World::new();
        let calls = insert_spy_with_shape_capture(&mut world);
        let red = Color::new(1.0, 0.0, 0.0, 1.0);
        let blue = Color::new(0.0, 0.0, 1.0, 1.0);
        world.spawn((
            Shape {
                color: red,
                ..default_shape()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            RenderLayer::World,
            SortOrder(10),
        ));
        world.spawn((
            Shape {
                color: blue,
                ..default_shape()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            RenderLayer::World,
            SortOrder(1),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = calls.lock().unwrap();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].2, blue);
        assert_eq!(calls[1].2, red);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When two shapes with different blend modes, then set blend mode in sorted order</summary>

<code>crates\engine_render\src\shape.rs:492</code>

```rust
        // Arrange
        let mut world = World::new();
        let blend_calls = insert_spy_with_blend_capture(&mut world);
        world.spawn((
            default_shape(),
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                blend_mode: BlendMode::Multiply,
                ..Material2d::default()
            },
        ));
        world.spawn((default_shape(), GlobalTransform2D(Affine2::IDENTITY)));

        // Act
        run_system(&mut world);

        // Assert
        let calls = blend_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[BlendMode::Alpha, BlendMode::Multiply]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When two visible shapes, then draw shape called twice</summary>

<code>crates\engine_render\src\shape.rs:590</code>

```rust
        // Arrange
        let mut world = World::new();
        let log = insert_spy(&mut world);
        world.spawn((default_shape(), GlobalTransform2D(Affine2::IDENTITY)));
        world.spawn((default_shape(), GlobalTransform2D(Affine2::IDENTITY)));

        // Act
        run_system(&mut world);

        // Assert
        let count = log
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.as_str() == "draw_shape")
            .count();
        assert_eq!(count, 2);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When two shapes with different shaders, then shader dominates blend in sort</summary>

<code>crates\engine_render\src\shape.rs:1006</code>

```rust
        // Arrange
        let mut world = World::new();
        let calls = insert_spy_with_shape_capture(&mut world);
        let red = Color::new(1.0, 0.0, 0.0, 1.0);
        let blue = Color::new(0.0, 0.0, 1.0, 1.0);
        // Shape A: ShaderHandle(1), BlendMode::Alpha
        world.spawn((
            Shape {
                color: red,
                ..default_shape()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                shader: ShaderHandle(1),
                blend_mode: BlendMode::Alpha,
                ..Material2d::default()
            },
        ));
        // Shape B: ShaderHandle(0), BlendMode::Additive
        world.spawn((
            Shape {
                color: blue,
                ..default_shape()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                shader: ShaderHandle(0),
                blend_mode: BlendMode::Additive,
                ..Material2d::default()
            },
        ));

        // Act
        run_system(&mut world);

        // Assert — ShaderHandle(0) < ShaderHandle(1), so blue drawn first
        let calls = calls.lock().unwrap();
        assert_eq!(calls[0].2, blue);
        assert_eq!(calls[1].2, red);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When polygon shape variant debug formatted, then snapshot matches</summary>

<code>crates\engine_render\src\shape.rs:176</code>

```rust
        // Arrange
        let variant = ShapeVariant::Polygon {
            points: vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(100.0, 0.0),
                Vec2::new(80.0, 60.0),
                Vec2::new(20.0, 60.0),
            ],
        };

        // Act
        let debug = format!("{variant:#?}");

        // Assert
        insta::assert_snapshot!(debug);
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test sprite</strong> (42 tests)</summary>

<blockquote>
<details>
<summary>✅ When entity has effective visibility true, then draw sprite called</summary>

<code>crates\engine_render\src\sprite.rs:231</code>

```rust
        // Arrange
        let mut world = World::new();
        let log = insert_spy(&mut world);
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::IDENTITY),
            EffectiveVisibility(true),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let count = log
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.as_str() == "draw_sprite")
            .count();
        assert_eq!(count, 1);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When entity has effective visibility false, then draw sprite not called</summary>

*EffectiveVisibility(false) is the earliest cull — filtered before sorting or frustum tests*

<code>crates\engine_render\src\sprite.rs:187</code>

```rust
        // Arrange
        let mut world = World::new();
        let log = insert_spy(&mut world);
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::IDENTITY),
            EffectiveVisibility(false),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let count = log
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.as_str() == "draw_sprite")
            .count();
        assert_eq!(count, 0);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When different layers, then layer overrides blend mode order</summary>

<code>crates\engine_render\src\sprite.rs:695</code>

```rust
        // Arrange
        let mut world = World::new();
        let (blend_calls, sprite_calls) = insert_spy_with_blend_and_sprite_capture(&mut world);
        let red = Color::new(1.0, 0.0, 0.0, 1.0);
        let blue = Color::new(0.0, 0.0, 1.0, 1.0);
        world.spawn((
            Sprite {
                color: red,
                ..default_sprite()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            RenderLayer::Background,
            Material2d {
                blend_mode: BlendMode::Additive,
                ..Material2d::default()
            },
        ));
        world.spawn((
            Sprite {
                color: blue,
                ..default_sprite()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            RenderLayer::World,
        ));

        // Act
        run_system(&mut world);

        // Assert
        let draws = sprite_calls.lock().unwrap();
        assert_eq!(draws.len(), 2);
        assert_eq!(draws[0].0.color, red);
        assert_eq!(draws[1].0.color, blue);
        let blends = blend_calls.lock().unwrap();
        assert_eq!(blends.as_slice(), &[BlendMode::Additive, BlendMode::Alpha]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When entity has no effective visibility, then draw sprite called</summary>

<code>crates\engine_render\src\sprite.rs:211</code>

```rust
        // Arrange
        let mut world = World::new();
        let log = insert_spy(&mut world);
        world.spawn((default_sprite(), GlobalTransform2D(Affine2::IDENTITY)));

        // Act
        run_system(&mut world);

        // Assert
        let count = log
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.as_str() == "draw_sprite")
            .count();
        assert_eq!(count, 1);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When invisible entity with material, then no blend or draw calls</summary>

<code>crates\engine_render\src\sprite.rs:778</code>

```rust
        // Arrange
        let mut world = World::new();
        let blend_calls = insert_spy_with_blend_capture(&mut world);
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::IDENTITY),
            EffectiveVisibility(false),
            Material2d {
                blend_mode: BlendMode::Additive,
                ..Material2d::default()
            },
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = blend_calls.lock().unwrap();
        assert!(calls.is_empty());
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When no camera entity, then all sprites drawn without culling</summary>

*Without a `Camera2D` entity, frustum culling is disabled entirely — all sprites are drawn*

<code>crates\engine_render\src\sprite.rs:868</code>

```rust
        // Arrange
        let mut world = World::new();
        let log = insert_spy(&mut world);
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::from_translation(glam::Vec2::new(2000.0, 300.0))),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let count = log
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.as_str() == "draw_sprite")
            .count();
        assert_eq!(count, 1);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When entity has sprite and global transform, then draw sprite called once</summary>

<code>crates\engine_render\src\sprite.rs:146</code>

```rust
        // Arrange
        let mut world = World::new();
        let log = insert_spy(&mut world);
        world.spawn((default_sprite(), GlobalTransform2D(Affine2::IDENTITY)));

        // Act
        run_system(&mut world);

        // Assert
        let count = log
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.as_str() == "draw_sprite")
            .count();
        assert_eq!(count, 1);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When entity has sprite but no global transform, then draw sprite not called</summary>

<code>crates\engine_render\src\sprite.rs:166</code>

```rust
        // Arrange
        let mut world = World::new();
        let log = insert_spy(&mut world);
        world.spawn(default_sprite());

        // Act
        run_system(&mut world);

        // Assert
        let count = log
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.as_str() == "draw_sprite")
            .count();
        assert_eq!(count, 0);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When sprite at known position, then rect xy match translation</summary>

<code>crates\engine_render\src\sprite.rs:415</code>

```rust
        // Arrange
        let mut world = World::new();
        let calls = insert_spy_with_capture(&mut world);
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::from_translation(glam::Vec2::new(100.0, 200.0))),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = calls.lock().unwrap();
        assert_eq!(calls[0].0.x, Pixels(100.0));
        assert_eq!(calls[0].0.y, Pixels(200.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When sprite fully inside camera view, then draw sprite called</summary>

<code>crates\engine_render\src\sprite.rs:840</code>

```rust
        // Arrange
        let mut world = World::new();
        let log = insert_spy_with_viewport(&mut world, 800, 600);
        world.spawn(Camera2D {
            position: glam::Vec2::new(400.0, 300.0),
            zoom: 1.0,
        });
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::from_translation(glam::Vec2::new(400.0, 300.0))),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let count = log
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.as_str() == "draw_sprite")
            .count();
        assert_eq!(count, 1);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When same shader different blend, then blend sorts within shader group</summary>

<code>crates\engine_render\src\sprite.rs:1157</code>

```rust
        // Arrange
        let mut world = World::new();
        let blend_calls = insert_spy_with_blend_capture(&mut world);
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                shader: ShaderHandle(0),
                blend_mode: BlendMode::Additive,
                ..Material2d::default()
            },
        ));
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                shader: ShaderHandle(0),
                blend_mode: BlendMode::Alpha,
                ..Material2d::default()
            },
        ));

        // Act
        run_system(&mut world);

        // Assert — Alpha < Additive in Ord
        let calls = blend_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[BlendMode::Alpha, BlendMode::Additive]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When same layer and blend different sort order, then lower sort first</summary>

<code>crates\engine_render\src\sprite.rs:735</code>

```rust
        // Arrange
        let mut world = World::new();
        let calls = insert_spy_with_capture(&mut world);
        let red = Color::new(1.0, 0.0, 0.0, 1.0);
        let blue = Color::new(0.0, 0.0, 1.0, 1.0);
        world.spawn((
            Sprite {
                color: red,
                ..default_sprite()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            RenderLayer::World,
            SortOrder(10),
            Material2d {
                blend_mode: BlendMode::Additive,
                ..Material2d::default()
            },
        ));
        world.spawn((
            Sprite {
                color: blue,
                ..default_sprite()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            RenderLayer::World,
            SortOrder(1),
            Material2d {
                blend_mode: BlendMode::Additive,
                ..Material2d::default()
            },
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = calls.lock().unwrap();
        assert_eq!(calls[0].0.color, blue);
        assert_eq!(calls[1].0.color, red);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When sprite fully outside camera view, then draw sprite not called</summary>

*Frustum culling skips draw calls for sprites whose AABB falls entirely outside the camera view rect*

<code>crates\engine_render\src\sprite.rs:813</code>

```rust
        // Arrange
        let mut world = World::new();
        let log = insert_spy_with_viewport(&mut world, 800, 600);
        world.spawn(Camera2D {
            position: glam::Vec2::new(400.0, 300.0),
            zoom: 1.0,
        });
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::from_translation(glam::Vec2::new(2000.0, 300.0))),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let count = log
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.as_str() == "draw_sprite")
            .count();
        assert_eq!(count, 0);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When sprite has additive material, then set blend mode called with additive</summary>

<code>crates\engine_render\src\sprite.rs:561</code>

```rust
        // Arrange
        let mut world = World::new();
        let blend_calls = insert_spy_with_blend_capture(&mut world);
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                blend_mode: BlendMode::Additive,
                ..Material2d::default()
            },
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = blend_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[BlendMode::Additive]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When sprite has empty uniforms, then set material uniforms not called</summary>

<code>crates\engine_render\src\sprite.rs:1092</code>

```rust
        // Arrange
        let mut world = World::new();
        let log = insert_spy(&mut world);
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                uniforms: vec![],
                ..Material2d::default()
            },
        ));

        // Act
        run_system(&mut world);

        // Assert
        let log = log.lock().unwrap();
        assert!(!log.iter().any(|s| s == "set_material_uniforms"));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When sprite has material uniforms, then set material uniforms called</summary>

<code>crates\engine_render\src\sprite.rs:1055</code>

```rust
        // Arrange
        let mut world = World::new();
        let uniform_calls = insert_spy_with_uniform_capture(&mut world);
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                uniforms: vec![1, 2, 3, 4],
                ..Material2d::default()
            },
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = uniform_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[vec![1u8, 2, 3, 4]]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When sprite has material, then set shader called with material shader</summary>

<code>crates\engine_render\src\sprite.rs:955</code>

```rust
        // Arrange
        let mut world = World::new();
        let shader_calls = insert_spy_with_shader_capture(&mut world);
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                shader: ShaderHandle(5),
                ..Material2d::default()
            },
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = shader_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[ShaderHandle(5)]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When sprite has multiply material, then set blend mode called with multiply</summary>

<code>crates\engine_render\src\sprite.rs:583</code>

```rust
        // Arrange
        let mut world = World::new();
        let blend_calls = insert_spy_with_blend_capture(&mut world);
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                blend_mode: BlendMode::Multiply,
                ..Material2d::default()
            },
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = blend_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[BlendMode::Multiply]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When sprite has no material, then bind material texture not called</summary>

<code>crates\engine_render\src\sprite.rs:1245</code>

```rust
        // Arrange
        let mut world = World::new();
        let log = insert_spy(&mut world);
        world.spawn((default_sprite(), GlobalTransform2D(Affine2::IDENTITY)));

        // Act
        run_system(&mut world);

        // Assert
        let log = log.lock().unwrap();
        assert!(!log.iter().any(|s| s == "bind_material_texture"));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When sprite has multiple texture bindings, then all forwarded in order</summary>

<code>crates\engine_render\src\sprite.rs:1214</code>

```rust
        // Arrange
        let mut world = World::new();
        let texture_bind_calls = insert_spy_with_texture_bind_capture(&mut world);
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                textures: vec![
                    TextureBinding {
                        texture: TextureId(1),
                        binding: 0,
                    },
                    TextureBinding {
                        texture: TextureId(2),
                        binding: 1,
                    },
                ],
                ..Material2d::default()
            },
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = texture_bind_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[(TextureId(1), 0), (TextureId(2), 1)]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When sprite has no render layer, then treated as world layer</summary>

<code>crates\engine_render\src\sprite.rs:347</code>

```rust
        // Arrange
        let mut world = World::new();
        let calls = insert_spy_with_capture(&mut world);
        let red = Color::new(1.0, 0.0, 0.0, 1.0);
        let blue = Color::new(0.0, 0.0, 1.0, 1.0);
        world.spawn((
            Sprite {
                color: red,
                ..default_sprite()
            },
            GlobalTransform2D(Affine2::IDENTITY),
        ));
        world.spawn((
            Sprite {
                color: blue,
                ..default_sprite()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            RenderLayer::Background,
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = calls.lock().unwrap();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].0.color, blue);
        assert_eq!(calls[1].0.color, red);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When sprite has no material, then set blend mode called with alpha</summary>

<code>crates\engine_render\src\sprite.rs:546</code>

```rust
        // Arrange
        let mut world = World::new();
        let blend_calls = insert_spy_with_blend_capture(&mut world);
        world.spawn((default_sprite(), GlobalTransform2D(Affine2::IDENTITY)));

        // Act
        run_system(&mut world);

        // Assert
        let calls = blend_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[BlendMode::Alpha]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When sprite has no material, then set material uniforms not called</summary>

<code>crates\engine_render\src\sprite.rs:1077</code>

```rust
        // Arrange
        let mut world = World::new();
        let log = insert_spy(&mut world);
        world.spawn((default_sprite(), GlobalTransform2D(Affine2::IDENTITY)));

        // Act
        run_system(&mut world);

        // Assert
        let log = log.lock().unwrap();
        assert!(!log.iter().any(|s| s == "set_material_uniforms"));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When sprite has no sort order, then treated as zero</summary>

<code>crates\engine_render\src\sprite.rs:380</code>

```rust
        // Arrange
        let mut world = World::new();
        let calls = insert_spy_with_capture(&mut world);
        let red = Color::new(1.0, 0.0, 0.0, 1.0);
        let blue = Color::new(0.0, 0.0, 1.0, 1.0);
        world.spawn((
            Sprite {
                color: red,
                ..default_sprite()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            RenderLayer::World,
            SortOrder(-1),
        ));
        world.spawn((
            Sprite {
                color: blue,
                ..default_sprite()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            RenderLayer::World,
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = calls.lock().unwrap();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].0.color, red);
        assert_eq!(calls[1].0.color, blue);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When sprite has no material, then set shader called with default</summary>

<code>crates\engine_render\src\sprite.rs:977</code>

```rust
        // Arrange
        let mut world = World::new();
        let shader_calls = insert_spy_with_shader_capture(&mut world);
        world.spawn((default_sprite(), GlobalTransform2D(Affine2::IDENTITY)));

        // Act
        run_system(&mut world);

        // Assert
        let calls = shader_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[ShaderHandle(0)]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When sprite serialized to ron, then deserializes to equal value</summary>

<code>crates\engine_render\src\sprite.rs:127</code>

```rust
        // Arrange
        let sprite = Sprite {
            texture: TextureId(7),
            uv_rect: [0.1, 0.2, 0.9, 0.8],
            color: Color::new(1.0, 0.5, 0.0, 1.0),
            width: Pixels(64.0),
            height: Pixels(128.0),
        };

        // Act
        let ron = ron::to_string(&sprite).unwrap();
        let back: Sprite = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(sprite, back);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When sprite has texture bindings, then bind material texture called</summary>

<code>crates\engine_render\src\sprite.rs:1189</code>

```rust
        // Arrange
        let mut world = World::new();
        let texture_bind_calls = insert_spy_with_texture_bind_capture(&mut world);
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                textures: vec![TextureBinding {
                    texture: TextureId(1),
                    binding: 2,
                }],
                ..Material2d::default()
            },
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = texture_bind_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[(TextureId(1), 2)]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When sprite just inside view right edge due to width, then drawn</summary>

<code>crates\engine_render\src\sprite.rs:891</code>

```rust
        // Sprite at x=-5, width=32 → AABB [-5, 27] overlaps view [0, 800].
        // If + were mutated to -: max = -5-32 = -37 < 0 → culled.
        let mut world = World::new();
        let log = insert_spy_with_viewport(&mut world, 800, 600);
        world.spawn(Camera2D {
            position: glam::Vec2::new(400.0, 300.0),
            zoom: 1.0,
        });
        world.spawn((
            Sprite {
                width: Pixels(32.0),
                height: Pixels(32.0),
                ..default_sprite()
            },
            GlobalTransform2D(Affine2::from_translation(glam::Vec2::new(-5.0, 300.0))),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let count = log
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.as_str() == "draw_sprite")
            .count();
        assert_eq!(count, 1, "sprite overlapping left edge should be drawn");
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When sprite just inside view bottom edge due to height, then drawn</summary>

<code>crates\engine_render\src\sprite.rs:923</code>

```rust
        // Sprite at y=-5, height=32 → AABB [-5, 27] overlaps view [0, 600].
        // If + mutated to -: max_y = -5-32 = -37 < 0 → culled.
        let mut world = World::new();
        let log = insert_spy_with_viewport(&mut world, 800, 600);
        world.spawn(Camera2D {
            position: glam::Vec2::new(400.0, 300.0),
            zoom: 1.0,
        });
        world.spawn((
            Sprite {
                width: Pixels(32.0),
                height: Pixels(32.0),
                ..default_sprite()
            },
            GlobalTransform2D(Affine2::from_translation(glam::Vec2::new(400.0, -5.0))),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let count = log
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.as_str() == "draw_sprite")
            .count();
        assert_eq!(count, 1, "sprite overlapping top edge should be drawn");
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When sprite with known dimensions, then rect size matches</summary>

<code>crates\engine_render\src\sprite.rs:434</code>

```rust
        // Arrange
        let mut world = World::new();
        let calls = insert_spy_with_capture(&mut world);
        world.spawn((
            Sprite {
                width: Pixels(48.0),
                height: Pixels(96.0),
                ..default_sprite()
            },
            GlobalTransform2D(Affine2::IDENTITY),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = calls.lock().unwrap();
        assert_eq!(calls[0].0.width, Pixels(48.0));
        assert_eq!(calls[0].0.height, Pixels(96.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When sprite with known color, then rect color matches</summary>

<code>crates\engine_render\src\sprite.rs:457</code>

```rust
        // Arrange
        let mut world = World::new();
        let calls = insert_spy_with_capture(&mut world);
        let color = Color::new(1.0, 0.0, 0.5, 1.0);
        world.spawn((
            Sprite {
                color,
                ..default_sprite()
            },
            GlobalTransform2D(Affine2::IDENTITY),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = calls.lock().unwrap();
        assert_eq!(calls[0].0.color, color);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When sprite straddles camera view edge, then draw sprite called</summary>

*Edge-touching sprites are drawn — conservative culling avoids popping artifacts at view boundaries*

<code>crates\engine_render\src\sprite.rs:1261</code>

```rust
        // Arrange
        let mut world = World::new();
        let log = insert_spy_with_viewport(&mut world, 800, 600);
        world.spawn(Camera2D {
            position: glam::Vec2::new(400.0, 300.0),
            zoom: 1.0,
        });
        // Sprite at x=795, width=32 → AABB [795, 827] overlaps view [0, 800]
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::from_translation(glam::Vec2::new(795.0, 300.0))),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let count = log
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.as_str() == "draw_sprite")
            .count();
        assert_eq!(count, 1);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When sprite with known uv rect, then draw sprite receives matching uv</summary>

<code>crates\engine_render\src\sprite.rs:479</code>

```rust
        // Arrange
        let mut world = World::new();
        let calls = insert_spy_with_capture(&mut world);
        let uv = [0.25, 0.0, 0.75, 1.0];
        world.spawn((
            Sprite {
                uv_rect: uv,
                ..default_sprite()
            },
            GlobalTransform2D(Affine2::IDENTITY),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = calls.lock().unwrap();
        assert_eq!(calls[0].1, uv);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When two sprites same layer different sort order, then lower drawn first</summary>

<code>crates\engine_render\src\sprite.rs:311</code>

```rust
        // Arrange
        let mut world = World::new();
        let calls = insert_spy_with_capture(&mut world);
        let red = Color::new(1.0, 0.0, 0.0, 1.0);
        let blue = Color::new(0.0, 0.0, 1.0, 1.0);
        world.spawn((
            Sprite {
                color: red,
                ..default_sprite()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            RenderLayer::World,
            SortOrder(10),
        ));
        world.spawn((
            Sprite {
                color: blue,
                ..default_sprite()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            RenderLayer::World,
            SortOrder(1),
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = calls.lock().unwrap();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].0.color, blue);
        assert_eq!(calls[1].0.color, red);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When two sprites with different blend modes, then set blend mode called in sorted order</summary>

<code>crates\engine_render\src\sprite.rs:605</code>

```rust
        // Arrange
        let mut world = World::new();
        let blend_calls = insert_spy_with_blend_capture(&mut world);
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                blend_mode: BlendMode::Additive,
                ..Material2d::default()
            },
        ));
        world.spawn((default_sprite(), GlobalTransform2D(Affine2::IDENTITY)));

        // Act
        run_system(&mut world);

        // Assert
        let calls = blend_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[BlendMode::Alpha, BlendMode::Additive]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When two sprites on different layers, then background drawn before world</summary>

*`RenderLayer` is the primary sort key — Background draws before World regardless of `SortOrder`*

<code>crates\engine_render\src\sprite.rs:277</code>

```rust
        // Arrange
        let mut world = World::new();
        let calls = insert_spy_with_capture(&mut world);
        let red = Color::new(1.0, 0.0, 0.0, 1.0);
        let blue = Color::new(0.0, 0.0, 1.0, 1.0);
        world.spawn((
            Sprite {
                color: red,
                ..default_sprite()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            RenderLayer::World,
        ));
        world.spawn((
            Sprite {
                color: blue,
                ..default_sprite()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            RenderLayer::Background,
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = calls.lock().unwrap();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].0.color, blue);
        assert_eq!(calls[1].0.color, red);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When two sprites with different shaders, then set shader called for each</summary>

<code>crates\engine_render\src\sprite.rs:992</code>

```rust
        // Arrange
        let mut world = World::new();
        let shader_calls = insert_spy_with_shader_capture(&mut world);
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                shader: ShaderHandle(1),
                ..Material2d::default()
            },
        ));
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                shader: ShaderHandle(2),
                ..Material2d::default()
            },
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = shader_calls.lock().unwrap();
        assert_eq!(calls.len(), 2);
        assert!(calls.contains(&ShaderHandle(1)));
        assert!(calls.contains(&ShaderHandle(2)));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When two sprites with same blend mode, then both drawn</summary>

<code>crates\engine_render\src\sprite.rs:660</code>

```rust
        // Arrange
        let mut world = World::new();
        let log = insert_spy(&mut world);
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                blend_mode: BlendMode::Additive,
                ..Material2d::default()
            },
        ));
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                blend_mode: BlendMode::Additive,
                ..Material2d::default()
            },
        ));

        // Act
        run_system(&mut world);

        // Assert
        let count = log
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.as_str() == "draw_sprite")
            .count();
        assert_eq!(count, 2);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When two sprites with different shaders, then shader dominates blend in sort</summary>

<code>crates\engine_render\src\sprite.rs:1114</code>

```rust
        // Arrange
        let mut world = World::new();
        let calls = insert_spy_with_capture(&mut world);
        let red = Color::new(1.0, 0.0, 0.0, 1.0);
        let blue = Color::new(0.0, 0.0, 1.0, 1.0);
        // Sprite A: ShaderHandle(1), BlendMode::Alpha
        world.spawn((
            Sprite {
                color: red,
                ..default_sprite()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                shader: ShaderHandle(1),
                blend_mode: BlendMode::Alpha,
                ..Material2d::default()
            },
        ));
        // Sprite B: ShaderHandle(0), BlendMode::Additive
        world.spawn((
            Sprite {
                color: blue,
                ..default_sprite()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                shader: ShaderHandle(0),
                blend_mode: BlendMode::Additive,
                ..Material2d::default()
            },
        ));

        // Act
        run_system(&mut world);

        // Assert — ShaderHandle(0) < ShaderHandle(1), so blue drawn first
        let calls = calls.lock().unwrap();
        assert_eq!(calls[0].0.color, blue);
        assert_eq!(calls[1].0.color, red);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When two sprites with same shader, then set shader called once</summary>

<code>crates\engine_render\src\sprite.rs:1024</code>

```rust
        // Arrange
        let mut world = World::new();
        let shader_calls = insert_spy_with_shader_capture(&mut world);
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                shader: ShaderHandle(2),
                ..Material2d::default()
            },
        ));
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                shader: ShaderHandle(2),
                ..Material2d::default()
            },
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = shader_calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0], ShaderHandle(2));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When two visible sprites, then draw sprite called twice</summary>

<code>crates\engine_render\src\sprite.rs:255</code>

```rust
        // Arrange
        let mut world = World::new();
        let log = insert_spy(&mut world);
        world.spawn((default_sprite(), GlobalTransform2D(Affine2::IDENTITY)));
        world.spawn((default_sprite(), GlobalTransform2D(Affine2::IDENTITY)));

        // Act
        run_system(&mut world);

        // Assert
        let count = log
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.as_str() == "draw_sprite")
            .count();
        assert_eq!(count, 2);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When two sprites with same blend mode, then set blend mode called once</summary>

*`apply_material` deduplicates — `set_blend_mode` only called when mode actually changes between sprites*

<code>crates\engine_render\src\sprite.rs:629</code>

```rust
        // Arrange
        let mut world = World::new();
        let blend_calls = insert_spy_with_blend_capture(&mut world);
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                blend_mode: BlendMode::Additive,
                ..Material2d::default()
            },
        ));
        world.spawn((
            default_sprite(),
            GlobalTransform2D(Affine2::IDENTITY),
            Material2d {
                blend_mode: BlendMode::Additive,
                ..Material2d::default()
            },
        ));

        // Act
        run_system(&mut world);

        // Assert
        let calls = blend_calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0], BlendMode::Additive);
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test testing</strong> (17 tests)</summary>

<blockquote>
<details>
<summary>✅ When apply post process called, then log records apply post process</summary>

<code>crates\engine_render\src\testing.rs:379</code>

```rust
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let mut spy = SpyRenderer::new(log.clone());

        // Act
        spy.apply_post_process();

        // Assert
        assert_eq!(log.lock().unwrap().as_slice(), &["apply_post_process"]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When bind material texture called with capture, then entry matches</summary>

<code>crates\engine_render\src\testing.rs:435</code>

```rust
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let texture_bind_calls: TextureBindCallLog = Arc::new(Mutex::new(Vec::new()));
        let mut spy = SpyRenderer::new(log).with_texture_bind_capture(texture_bind_calls.clone());

        // Act
        spy.bind_material_texture(engine_core::types::TextureId(3), 1);

        // Assert
        let calls = texture_bind_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[(engine_core::types::TextureId(3), 1)]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When clear called, then log records clear string</summary>

<code>crates\engine_render\src\testing.rs:218</code>

```rust
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let mut spy = SpyRenderer::new(log.clone());

        // Act
        spy.clear(Color::WHITE);

        // Assert
        assert_eq!(log.lock().unwrap().as_slice(), &["clear"]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When clear called with color capture, then color is stored</summary>

<code>crates\engine_render\src\testing.rs:450</code>

```rust
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let color_capture = Arc::new(Mutex::new(None));
        let mut spy = SpyRenderer::new(log.clone()).with_color_capture(color_capture.clone());
        let expected = Color::new(1.0, 0.0, 0.5, 1.0);

        // Act
        spy.clear(expected);

        // Assert
        assert_eq!(*color_capture.lock().unwrap(), Some(expected));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When draw rect called, then log records draw rect string</summary>

<code>crates\engine_render\src\testing.rs:231</code>

```rust
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let mut spy = SpyRenderer::new(log.clone());
        let rect = Rect::default();

        // Act
        spy.draw_rect(rect);

        // Assert
        assert_eq!(log.lock().unwrap().as_slice(), &["draw_rect"]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When draw shape called, then log records draw shape string</summary>

<code>crates\engine_render\src\testing.rs:297</code>

```rust
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let mut spy = SpyRenderer::new(log.clone());

        // Act
        spy.draw_shape(
            &[[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]],
            &[0, 1, 2],
            Color::WHITE,
        );

        // Assert
        assert_eq!(log.lock().unwrap().as_slice(), &["draw_shape"]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When draw shape called with capture, then color matches</summary>

<code>crates\engine_render\src\testing.rs:314</code>

```rust
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let shape_calls: ShapeCallLog = Arc::new(Mutex::new(Vec::new()));
        let mut spy = SpyRenderer::new(log).with_shape_capture(shape_calls.clone());
        let color = Color::new(1.0, 0.0, 0.0, 1.0);

        // Act
        spy.draw_shape(&[[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]], &[0, 1, 2], color);

        // Assert
        let calls = shape_calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].2, color);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When draw sprite called, then log records draw sprite string</summary>

<code>crates\engine_render\src\testing.rs:271</code>

```rust
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let mut spy = SpyRenderer::new(log.clone());

        // Act
        spy.draw_sprite(Rect::default(), [0.0, 0.0, 1.0, 1.0]);

        // Assert
        assert_eq!(log.lock().unwrap().as_slice(), &["draw_sprite"]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When present called, then log records present string</summary>

<code>crates\engine_render\src\testing.rs:245</code>

```rust
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let mut spy = SpyRenderer::new(log.clone());

        // Act
        spy.present();

        // Assert
        assert_eq!(log.lock().unwrap().as_slice(), &["present"]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When resize called, then log records resize string</summary>

<code>crates\engine_render\src\testing.rs:258</code>

```rust
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let mut spy = SpyRenderer::new(log.clone());

        // Act
        spy.resize(800, 600);

        // Assert
        assert_eq!(log.lock().unwrap().as_slice(), &["resize"]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When set blend mode called, then log records set blend mode</summary>

<code>crates\engine_render\src\testing.rs:331</code>

```rust
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let mut spy = SpyRenderer::new(log.clone());

        // Act
        spy.set_blend_mode(BlendMode::Additive);

        // Assert
        assert_eq!(log.lock().unwrap().as_slice(), &["set_blend_mode"]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When set blend mode called twice with capture, then both calls recorded in order</summary>

<code>crates\engine_render\src\testing.rs:344</code>

```rust
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let blend_calls: BlendCallLog = Arc::new(Mutex::new(Vec::new()));
        let mut spy = SpyRenderer::new(log).with_blend_capture(blend_calls.clone());

        // Act
        spy.set_blend_mode(BlendMode::Alpha);
        spy.set_blend_mode(BlendMode::Additive);

        // Assert
        let calls = blend_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[BlendMode::Alpha, BlendMode::Additive]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When set material uniforms called with capture, then bytes match</summary>

<code>crates\engine_render\src\testing.rs:420</code>

```rust
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let uniform_calls: UniformCallLog = Arc::new(Mutex::new(Vec::new()));
        let mut spy = SpyRenderer::new(log).with_uniform_capture(uniform_calls.clone());

        // Act
        spy.set_material_uniforms(&[10, 20]);

        // Assert
        let calls = uniform_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[vec![10u8, 20]]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When set shader called, then log records set shader</summary>

<code>crates\engine_render\src\testing.rs:392</code>

```rust
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let mut spy = SpyRenderer::new(log.clone());

        // Act
        spy.set_shader(crate::material::ShaderHandle(42));

        // Assert
        assert_eq!(log.lock().unwrap().as_slice(), &["set_shader"]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When set shader called with capture, then handle matches</summary>

<code>crates\engine_render\src\testing.rs:405</code>

```rust
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let shader_calls: ShaderCallLog = Arc::new(Mutex::new(Vec::new()));
        let mut spy = SpyRenderer::new(log).with_shader_capture(shader_calls.clone());

        // Act
        spy.set_shader(crate::material::ShaderHandle(7));

        // Assert
        let calls = shader_calls.lock().unwrap();
        assert_eq!(calls.as_slice(), &[crate::material::ShaderHandle(7)]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When set view projection called, then log records set view projection</summary>

<code>crates\engine_render\src\testing.rs:284</code>

```rust
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let mut spy = SpyRenderer::new(log.clone());

        // Act
        spy.set_view_projection([[0.0f32; 4]; 4]);

        // Assert
        assert_eq!(log.lock().unwrap().as_slice(), &["set_view_projection"]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When upload atlas called, then log records upload atlas</summary>

<code>crates\engine_render\src\testing.rs:360</code>

```rust
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let mut spy = SpyRenderer::new(log.clone());
        let atlas = crate::atlas::TextureAtlas {
            data: vec![255; 4],
            width: 1,
            height: 1,
            lookups: std::collections::HashMap::default(),
        };

        // Act
        spy.upload_atlas(&atlas);

        // Assert
        assert_eq!(log.lock().unwrap().as_slice(), &["upload_atlas"]);
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test visual_regression</strong> (10 tests)</summary>

<blockquote>
<details>
<summary>✅ When computing padded row bytes, then returns multiple of 256</summary>

<code>crates\engine_render\src\visual_regression.rs:634</code>

```rust
        // Act
        let result = padded_row_bytes(65, 4);

        // Assert
        assert_eq!(result, 512);
        assert_eq!(result % 256, 0);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When loading nonexistent golden, then returns error</summary>

<code>crates\engine_render\src\visual_regression.rs:748</code>

```rust
        // Arrange
        let path = std::path::Path::new("/nonexistent/golden.png");

        // Act
        let result = load_golden(path);

        // Assert
        assert!(result.is_err());
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When stripping row padding, then produces packed rgba</summary>

<code>crates\engine_render\src\visual_regression.rs:653</code>

```rust
        // Arrange
        let width = 2u32;
        let height = 2u32;
        let padded = padded_row_bytes(width, 4) as usize;
        let mut data = vec![0u8; padded * height as usize];
        data[0..4].copy_from_slice(&[255, 0, 0, 255]);
        data[4..8].copy_from_slice(&[0, 255, 0, 255]);
        data[padded..padded + 4].copy_from_slice(&[0, 0, 255, 255]);
        data[padded + 4..padded + 8].copy_from_slice(&[255, 255, 255, 255]);

        // Act
        let packed = strip_row_padding(&data, width, height, padded as u32, 4);

        // Assert
        assert_eq!(packed.len(), 2 * 2 * 4);
        assert_eq!(&packed[0..4], &[255, 0, 0, 255]);
        assert_eq!(&packed[4..8], &[0, 255, 0, 255]);
        assert_eq!(&packed[8..12], &[0, 0, 255, 255]);
        assert_eq!(&packed[12..16], &[255, 255, 255, 255]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When width already aligned, then padded row bytes unchanged</summary>

<code>crates\engine_render\src\visual_regression.rs:644</code>

```rust
        // Act
        let result = padded_row_bytes(64, 4);

        // Assert
        assert_eq!(result, 256);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When loading saved golden, then pixels match original</summary>

<code>crates\engine_render\src\visual_regression.rs:728</code>

```rust
        // Arrange
        let dir = std::env::temp_dir().join("axiom2d_golden_test_roundtrip");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("roundtrip.png");
        let original: Vec<u8> = [255, 0, 0, 255].repeat(4 * 4);
        save_golden(&path, &original, 4, 4).unwrap();

        // Act
        let (loaded, w, h) = load_golden(&path).unwrap();

        // Assert
        assert_eq!(w, 4);
        assert_eq!(h, 4);
        assert_eq!(loaded, original);
        let _ = std::fs::remove_dir_all(&dir);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When saving golden image, then file exists at expected path</summary>

<code>crates\engine_render\src\visual_regression.rs:711</code>

```rust
        // Arrange
        let dir = std::env::temp_dir().join("axiom2d_golden_test_save");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.png");
        let pixels: Vec<u8> = [255, 0, 0, 255].repeat(4 * 4);

        // Act
        save_golden(&path, &pixels, 4, 4).unwrap();

        // Assert
        assert!(path.exists());
        let _ = std::fs::remove_dir_all(&dir);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When comparing identical buffers, then ssim score is one</summary>

<code>crates\engine_render\src\visual_regression.rs:585</code>

```rust
        // Arrange
        let a: Vec<u8> = [255, 0, 0, 255].repeat(64 * 64);
        let b: Vec<u8> = [255, 0, 0, 255].repeat(64 * 64);

        // Act
        let score = ssim_compare(&a, &b, 64, 64);

        // Assert
        assert!(
            (score - 1.0).abs() < f32::EPSILON,
            "identical pixel buffers must yield SSIM=1.0, got {score}"
        );
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When comparing different buffers, then ssim score is less than one</summary>

<code>crates\engine_render\src\visual_regression.rs:601</code>

```rust
        // Arrange
        let a: Vec<u8> = [255, 0, 0, 255].repeat(64 * 64);
        let b: Vec<u8> = [0, 0, 255, 255].repeat(64 * 64);

        // Act
        let score = ssim_compare(&a, &b, 64, 64);

        // Assert
        assert!(
            score < 1.0,
            "different buffers must yield SSIM<1.0, got {score}"
        );
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When comparing largely different buffers, then ssim below threshold</summary>

<code>crates\engine_render\src\visual_regression.rs:760</code>

```rust
        // Arrange
        let mut a: Vec<u8> = [255, 0, 0, 255].repeat(64 * 64);
        for y in 0..32 {
            for x in 0..32 {
                let idx = (y * 64 + x) * 4;
                a[idx] = 0;
                a[idx + 2] = 255;
            }
        }
        let b: Vec<u8> = [255, 0, 0, 255].repeat(64 * 64);

        // Act
        let score = ssim_compare(&a, &b, 64, 64);

        // Assert
        assert!(
            score < 0.99,
            "25% different pixels must fail 0.99 threshold, got {score}"
        );
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When comparing slightly different buffers, then ssim above threshold</summary>

<code>crates\engine_render\src\visual_regression.rs:617</code>

```rust
        // Arrange
        let a: Vec<u8> = [255, 0, 0, 255].repeat(64 * 64);
        let mut b = a.clone();
        b[0] = 254;

        // Act
        let score = ssim_compare(&a, &b, 64, 64);

        // Assert
        assert!(
            score >= 0.99,
            "single-pixel change in 64x64 must stay above 0.99 threshold, got {score}"
        );
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test wgpu_renderer</strong> (23 tests)</summary>

<blockquote>
<details>
<summary>✅ When all same blend mode, then single batch</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1832</code>

```rust
        use crate::material::BlendMode;

        // Arrange
        let modes = [BlendMode::Alpha, BlendMode::Alpha, BlendMode::Alpha];

        // Act
        let batches = compute_batch_ranges(&modes);

        // Assert
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0], (BlendMode::Alpha, 0..3));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When any rect, then uv rect is full texture</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1537</code>

```rust
        // Arrange
        let rect = Rect {
            x: Pixels(100.0),
            y: Pixels(100.0),
            width: Pixels(50.0),
            height: Pixels(50.0),
            color: Color::RED,
        };

        // Act
        let instance = rect_to_instance(&rect);

        // Assert
        assert_eq!(instance.uv_rect, [0.0, 0.0, 1.0, 1.0]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When batch is empty, then is empty returns true</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1704</code>

```rust
        // Act
        let batch = ShapeBatch::new();

        // Assert
        assert!(batch.is_empty());
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When batch cleared, then vertex and index counts are zero</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1688</code>

```rust
        // Arrange
        let vertices = [[0.0_f32, 0.0], [1.0, 0.0], [0.5, 1.0]];
        let indices = [0_u32, 1, 2];
        let mut batch = ShapeBatch::new();
        batch.push(&vertices, &indices, Color::RED);

        // Act
        batch.clear();

        // Assert
        assert_eq!(batch.vertex_count(), 0);
        assert_eq!(batch.index_count(), 0);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When blend mode additive, then blend state uses src alpha one</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1768</code>

```rust
        // Act
        let result = blend_mode_to_blend_state(crate::material::BlendMode::Additive);

        // Assert
        let expected = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
        };
        assert_eq!(result, expected);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When blend mode alpha, then blend state is alpha blending</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1759</code>

```rust
        // Act
        let result = blend_mode_to_blend_state(crate::material::BlendMode::Alpha);

        // Assert
        assert_eq!(result, wgpu::BlendState::ALPHA_BLENDING);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When blend mode multiply, then blend state uses dst zero</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1789</code>

```rust
        // Act
        let result = blend_mode_to_blend_state(crate::material::BlendMode::Multiply);

        // Assert
        let expected = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::Dst,
                dst_factor: wgpu::BlendFactor::Zero,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::Zero,
                operation: wgpu::BlendOperation::Add,
            },
        };
        assert_eq!(result, expected);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When colored rect, then instance color matches</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1555</code>

```rust
        // Arrange
        let rect = Rect {
            x: Pixels(0.0),
            y: Pixels(0.0),
            width: Pixels(100.0),
            height: Pixels(100.0),
            color: Color::RED,
        };

        // Act
        let instance = rect_to_instance(&rect);

        // Assert
        assert_eq!(instance.color, [1.0, 0.0, 0.0, 1.0]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When fullscreen quad indices resolved, then two ccw triangles cover quad</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1740</code>

```rust
        // Act
        let tri0: [[f32; 2]; 3] = [
            FULLSCREEN_QUAD_VERTICES[QUAD_INDICES[0] as usize].position,
            FULLSCREEN_QUAD_VERTICES[QUAD_INDICES[1] as usize].position,
            FULLSCREEN_QUAD_VERTICES[QUAD_INDICES[2] as usize].position,
        ];
        let tri1: [[f32; 2]; 3] = [
            FULLSCREEN_QUAD_VERTICES[QUAD_INDICES[3] as usize].position,
            FULLSCREEN_QUAD_VERTICES[QUAD_INDICES[4] as usize].position,
            FULLSCREEN_QUAD_VERTICES[QUAD_INDICES[5] as usize].position,
        ];

        // Assert
        assert_eq!(tri0, [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0]]);
        assert_eq!(tri1, [[-1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When fullscreen quad vertices queried, then four corners span ndc</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1609</code>

```rust
        // Act
        let positions: [[f32; 2]; 4] = [
            FULLSCREEN_QUAD_VERTICES[0].position,
            FULLSCREEN_QUAD_VERTICES[1].position,
            FULLSCREEN_QUAD_VERTICES[2].position,
            FULLSCREEN_QUAD_VERTICES[3].position,
        ];

        // Assert
        assert_eq!(
            positions,
            [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]]
        );
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When mixed blend modes, then batches split at boundaries</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1810</code>

```rust
        use crate::material::BlendMode;

        // Arrange
        let modes = [
            BlendMode::Alpha,
            BlendMode::Alpha,
            BlendMode::Additive,
            BlendMode::Alpha,
        ];

        // Act
        let batches = compute_batch_ranges(&modes);

        // Assert
        assert_eq!(batches.len(), 3);
        assert_eq!(batches[0], (BlendMode::Alpha, 0..2));
        assert_eq!(batches[1], (BlendMode::Additive, 2..3));
        assert_eq!(batches[2], (BlendMode::Alpha, 3..4));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When negative dimensions, then stored without clamping</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1591</code>

```rust
        // Arrange
        let rect = Rect {
            x: Pixels(400.0),
            y: Pixels(300.0),
            width: Pixels(-100.0),
            height: Pixels(-50.0),
            color: Color::WHITE,
        };

        // Act
        let instance = rect_to_instance(&rect);

        // Assert
        assert_eq!(instance.world_rect, [400.0, 300.0, -100.0, -50.0]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When no items, then empty batches</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1847</code>

```rust
        // Act
        let batches = compute_batch_ranges(&[]);

        // Assert
        assert!(batches.is_empty());
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When offset rect, then instance encodes position and size</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1519</code>

```rust
        // Arrange
        let rect = Rect {
            x: Pixels(200.0),
            y: Pixels(150.0),
            width: Pixels(400.0),
            height: Pixels(300.0),
            color: Color::WHITE,
        };

        // Act
        let instance = rect_to_instance(&rect);

        // Assert
        assert_eq!(instance.world_rect, [200.0, 150.0, 400.0, 300.0]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When quad indices used, then two triangles cover unit square</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1482</code>

```rust
        // Act
        let tri0: [[f32; 2]; 3] = [
            QUAD_VERTICES[QUAD_INDICES[0] as usize].position,
            QUAD_VERTICES[QUAD_INDICES[1] as usize].position,
            QUAD_VERTICES[QUAD_INDICES[2] as usize].position,
        ];
        let tri1: [[f32; 2]; 3] = [
            QUAD_VERTICES[QUAD_INDICES[3] as usize].position,
            QUAD_VERTICES[QUAD_INDICES[4] as usize].position,
            QUAD_VERTICES[QUAD_INDICES[5] as usize].position,
        ];

        // Assert
        assert_eq!(tri0, [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]]);
        assert_eq!(tri1, [[0.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When rect at origin, then instance encodes world coordinates</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1501</code>

```rust
        // Arrange
        let rect = Rect {
            x: Pixels(0.0),
            y: Pixels(0.0),
            width: Pixels(800.0),
            height: Pixels(600.0),
            color: Color::WHITE,
        };

        // Act
        let instance = rect_to_instance(&rect);

        // Assert
        assert_eq!(instance.world_rect, [0.0, 0.0, 800.0, 600.0]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When shape vertex size checked, then exactly 24 bytes</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1626</code>

```rust
        // Act
        let size = std::mem::size_of::<ShapeVertex>();

        // Assert
        assert_eq!(size, 24);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When shape vertices cast to bytes, then no panic</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1635</code>

```rust
        // Arrange
        let vertices = [
            ShapeVertex {
                position: [0.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            ShapeVertex {
                position: [1.0, 0.0],
                color: [1.0, 0.0, 0.0, 1.0],
            },
        ];

        // Act
        let bytes: &[u8] = bytemuck::cast_slice(&vertices);

        // Assert
        assert_eq!(bytes.len(), 48);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When single shape pushed, then vertex and index counts match input</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1656</code>

```rust
        // Arrange
        let vertices = [[0.0_f32, 0.0], [1.0, 0.0], [0.5, 1.0]];
        let indices = [0_u32, 1, 2];
        let mut batch = ShapeBatch::new();

        // Act
        batch.push(&vertices, &indices, Color::RED);

        // Assert
        assert_eq!(batch.vertex_count(), 3);
        assert_eq!(batch.index_count(), 3);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When triangle pushed, then vertices returns three and is empty false</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1713</code>

```rust
        // Arrange
        let positions = [[0.0_f32, 0.0], [1.0, 0.0], [0.5, 1.0]];
        let indices = [0_u32, 1, 2];
        let mut batch = ShapeBatch::new();

        // Act
        batch.push(&positions, &indices, Color::RED);

        // Assert
        assert!(!batch.is_empty());
        assert_eq!(batch.vertices().len(), 3);
        assert_eq!(batch.vertices()[0].position, [0.0, 0.0]);
        assert_eq!(batch.vertices()[1].position, [1.0, 0.0]);
        assert_eq!(batch.vertices()[2].position, [0.5, 1.0]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When zero size rect, then no panic and zero dimensions</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1573</code>

```rust
        // Arrange
        let rect = Rect {
            x: Pixels(400.0),
            y: Pixels(300.0),
            width: Pixels(0.0),
            height: Pixels(0.0),
            color: Color::WHITE,
        };

        // Act
        let instance = rect_to_instance(&rect);

        // Assert
        assert_eq!(instance.world_rect, [400.0, 300.0, 0.0, 0.0]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When two shapes pushed, then second indices are offset by first vertex count</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1671</code>

```rust
        // Arrange
        let tri_verts = [[0.0_f32, 0.0], [1.0, 0.0], [0.5, 1.0]];
        let tri_indices = [0_u32, 1, 2];
        let quad_verts = [[2.0_f32, 0.0], [3.0, 0.0], [3.0, 1.0], [2.0, 1.0]];
        let quad_indices = [0_u32, 1, 2, 0, 2, 3];
        let mut batch = ShapeBatch::new();
        batch.push(&tri_verts, &tri_indices, Color::RED);

        // Act
        batch.push(&quad_verts, &quad_indices, Color::BLUE);

        // Assert
        assert_eq!(&batch.indices()[3..], &[3_u32, 4, 5, 3, 5, 6]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When shape shader parsed, then no error</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1731</code>

```rust
        // Act
        let result = naga::front::wgsl::parse_str(SHAPE_SHADER_SRC);

        // Assert
        assert!(result.is_ok(), "WGSL parse error: {result:?}");
```

</details>
</blockquote>

</details>
</blockquote>

</details>

<details>
<summary><strong>engine_scene</strong> (42 tests)</summary>

<blockquote>
<details>
<summary><strong>test hierarchy</strong> (10 tests)</summary>

<blockquote>
<details>
<summary>✅ When two parents each have one child, then each parent children is independent</summary>

<code>crates\engine_scene\src\hierarchy.rs:78</code>

```rust
        // Arrange
        let mut world = World::new();
        let parent_a = world.spawn_empty().id();
        let parent_b = world.spawn_empty().id();
        let child_x = world.spawn(ChildOf(parent_a)).id();
        let child_y = world.spawn(ChildOf(parent_b)).id();

        // Act
        run_hierarchy_system(&mut world);

        // Assert
        let children_a = world
            .get::<Children>(parent_a)
            .expect("parent_a should have Children");
        assert_eq!(children_a.0, vec![child_x]);
        let children_b = world
            .get::<Children>(parent_b)
            .expect("parent_b should have Children");
        assert_eq!(children_b.0, vec![child_y]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When two children share same parent, then children contains both</summary>

<code>crates\engine_scene\src\hierarchy.rs:58</code>

```rust
        // Arrange
        let mut world = World::new();
        let parent = world.spawn_empty().id();
        let child_a = world.spawn(ChildOf(parent)).id();
        let child_b = world.spawn(ChildOf(parent)).id();

        // Act
        run_hierarchy_system(&mut world);

        // Assert
        let children = world
            .get::<Children>(parent)
            .expect("parent should have Children");
        assert_eq!(children.0.len(), 2);
        assert!(children.0.contains(&child_a));
        assert!(children.0.contains(&child_b));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When entity has child of, then hierarchy system adds it to parent children</summary>

<code>crates\engine_scene\src\hierarchy.rs:41</code>

```rust
        // Arrange
        let mut world = World::new();
        let parent = world.spawn_empty().id();
        let child = world.spawn(ChildOf(parent)).id();

        // Act
        run_hierarchy_system(&mut world);

        // Assert
        let children = world
            .get::<Children>(parent)
            .expect("parent should have Children component");
        assert!(children.0.contains(&child));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When child entity is despawned, then parent children no longer contains that child</summary>

<code>crates\engine_scene\src\hierarchy.rs:179</code>

```rust
        // Arrange
        let mut world = World::new();
        let parent = world.spawn_empty().id();
        let child_a = world.spawn(ChildOf(parent)).id();
        let child_b = world.spawn(ChildOf(parent)).id();
        run_hierarchy_system(&mut world);
        world.despawn(child_a);

        // Act
        run_hierarchy_system(&mut world);

        // Assert
        let children = world
            .get::<Children>(parent)
            .expect("parent should have Children");
        assert_eq!(children.0, vec![child_b]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When child of is removed, then parent children no longer contains that child</summary>

*`hierarchy_maintenance_system` rebuilds `Children` from scratch each frame — reparenting is automatic*

<code>crates\engine_scene\src\hierarchy.rs:142</code>

```rust
        // Arrange
        let mut world = World::new();
        let parent = world.spawn_empty().id();
        let child_a = world.spawn(ChildOf(parent)).id();
        let child_b = world.spawn(ChildOf(parent)).id();
        run_hierarchy_system(&mut world);
        world.entity_mut(child_a).remove::<ChildOf>();

        // Act
        run_hierarchy_system(&mut world);

        // Assert
        let children = world
            .get::<Children>(parent)
            .expect("parent should have Children");
        assert_eq!(children.0, vec![child_b]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When system runs twice with no changes, then children remains stable</summary>

<code>crates\engine_scene\src\hierarchy.rs:123</code>

```rust
        // Arrange
        let mut world = World::new();
        let parent = world.spawn_empty().id();
        let child = world.spawn(ChildOf(parent)).id();

        // Act
        run_hierarchy_system(&mut world);
        run_hierarchy_system(&mut world);

        // Assert
        let children = world
            .get::<Children>(parent)
            .expect("parent should have Children");
        assert_eq!(children.0, vec![child]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When only child is despawned, then parent children component is removed</summary>

<code>crates\engine_scene\src\hierarchy.rs:235</code>

```rust
        // Arrange
        let mut world = World::new();
        let parent = world.spawn_empty().id();
        let child = world.spawn(ChildOf(parent)).id();
        run_hierarchy_system(&mut world);
        world.despawn(child);

        // Act
        run_hierarchy_system(&mut world);

        // Assert
        assert!(world.get::<Children>(parent).is_none());
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When multiple children belong to parent, then children vec is sorted by entity</summary>

<code>crates\engine_scene\src\hierarchy.rs:101</code>

```rust
        // Arrange
        let mut world = World::new();
        let parent = world.spawn_empty().id();
        let child_a = world.spawn(ChildOf(parent)).id();
        let child_b = world.spawn(ChildOf(parent)).id();
        let child_c = world.spawn(ChildOf(parent)).id();

        // Act
        run_hierarchy_system(&mut world);

        // Assert
        let children = world
            .get::<Children>(parent)
            .expect("parent should have Children");
        let mut sorted = children.0.clone();
        sorted.sort();
        assert_eq!(children.0, sorted);
        assert_eq!(children.0, vec![child_a, child_b, child_c]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When last child of is removed, then parent children component is removed</summary>

*Stale `Children` components are cleaned up when no `ChildOf` references remain for that parent*

<code>crates\engine_scene\src\hierarchy.rs:163</code>

```rust
        // Arrange
        let mut world = World::new();
        let parent = world.spawn_empty().id();
        let child = world.spawn(ChildOf(parent)).id();
        run_hierarchy_system(&mut world);
        world.entity_mut(child).remove::<ChildOf>();

        // Act
        run_hierarchy_system(&mut world);

        // Assert
        assert!(world.get::<Children>(parent).is_none());
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When arbitrary child of assignments, then children vec is sorted</summary>

<code>crates\engine_scene\src\hierarchy.rs:200</code>

```rust
            child_count in 2usize..=10,
            parent_count in 1usize..=3,
        ) {
            // Arrange
            let mut world = World::new();
            let parents: Vec<Entity> = (0..parent_count)
                .map(|_| world.spawn_empty().id())
                .collect();
            for i in 0..child_count {
                let parent = parents[i % parents.len()];
                world.spawn(ChildOf(parent));
            }

            // Act
            run_hierarchy_system(&mut world);

            // Assert
            for &parent in &parents {
                if let Some(children) = world.get::<Children>(parent) {
                    let sorted = {
                        let mut v = children.0.clone();
                        v.sort();
                        v
                    };
                    assert_eq!(
                        children.0, sorted,
                        "Children vec should be sorted for parent {parent:?}"
                    );
                }
            }
        }
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test render_order</strong> (5 tests)</summary>

<blockquote>
<details>
<summary>✅ When sort order serialized to ron, then deserializes to equal value</summary>

<code>crates\engine_scene\src\render_order.rs:39</code>

```rust
        // Arrange
        let order = SortOrder(-42);

        // Act
        let ron = ron::to_string(&order).unwrap();
        let back: SortOrder = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(order, back);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When entities sorted by render layer and sort order, then order is deterministic</summary>

<code>crates\engine_scene\src\render_order.rs:67</code>

```rust
        // Arrange
        let mut items = vec![
            (RenderLayer::World, SortOrder(1)),
            (RenderLayer::Background, SortOrder(0)),
            (RenderLayer::World, SortOrder(0)),
            (RenderLayer::UI, SortOrder(-1)),
        ];

        // Act
        items.sort();

        // Assert
        assert_eq!(
            items,
            vec![
                (RenderLayer::Background, SortOrder(0)),
                (RenderLayer::World, SortOrder(0)),
                (RenderLayer::World, SortOrder(1)),
                (RenderLayer::UI, SortOrder(-1)),
            ]
        );
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When render layer variants serialized to ron, then each deserializes to matching variant</summary>

<code>crates\engine_scene\src\render_order.rs:24</code>

```rust
        for layer in [
            RenderLayer::Background,
            RenderLayer::World,
            RenderLayer::Characters,
            RenderLayer::Foreground,
            RenderLayer::UI,
        ] {
            let ron = ron::to_string(&layer).unwrap();
            let back: RenderLayer = ron::from_str(&ron).unwrap();
            assert_eq!(layer, back);
        }
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When render layers compared, then background less than world less than characters less than foreground less than ui</summary>

<code>crates\engine_scene\src\render_order.rs:52</code>

```rust
        assert!(RenderLayer::Background < RenderLayer::World);
        assert!(RenderLayer::World < RenderLayer::Characters);
        assert!(RenderLayer::Characters < RenderLayer::Foreground);
        assert!(RenderLayer::Foreground < RenderLayer::UI);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When sort order values compared, then lower i32 value sorts before higher</summary>

<code>crates\engine_scene\src\render_order.rs:61</code>

```rust
        assert!(SortOrder(-1) < SortOrder(1));
        assert!(SortOrder(i32::MIN) < SortOrder(i32::MAX));
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test spawn_child</strong> (3 tests)</summary>

<blockquote>
<details>
<summary>✅ When spawn child called, then new entity also contains the provided bundle</summary>

<code>crates\engine_scene\src\spawn_child.rs:41</code>

```rust
        // Arrange
        let mut world = World::new();
        let parent = world.spawn_empty().id();

        // Act
        let child = world.spawn_child(parent, Marker);

        // Assert
        assert!(world.get::<ChildOf>(child).is_some());
        assert!(world.get::<Marker>(child).is_some());
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When spawn child called, then new entity has child of pointing to parent</summary>

<code>crates\engine_scene\src\spawn_child.rs:22</code>

```rust
        // Arrange
        let mut world = World::new();
        let parent = world.spawn_empty().id();

        // Act
        let child = world.spawn_child(parent, ());

        // Assert
        let child_of = world
            .get::<ChildOf>(child)
            .expect("child should have ChildOf");
        assert_eq!(child_of.0, parent);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When spawn child used, then hierarchy system picks up the new child</summary>

<code>crates\engine_scene\src\spawn_child.rs:55</code>

```rust
        // Arrange
        let mut world = World::new();
        let parent = world.spawn_empty().id();
        let child = world.spawn_child(parent, ());

        // Act
        run_hierarchy_system(&mut world);

        // Assert
        let children = world
            .get::<Children>(parent)
            .expect("parent should have Children");
        assert_eq!(children.0, vec![child]);
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test transform_propagation</strong> (12 tests)</summary>

<blockquote>
<details>
<summary>✅ When child has translation and parent has translation, then both accumulate</summary>

*`GlobalTransform2D` = parent.global * child.local — standard affine composition*

<code>crates\engine_scene\src\transform_propagation.rs:136</code>

```rust
        // Arrange
        let mut world = World::new();
        let parent = world
            .spawn(Transform2D {
                position: Vec2::new(10.0, 0.0),
                ..Transform2D::default()
            })
            .id();
        let child = world
            .spawn((
                Transform2D {
                    position: Vec2::new(5.0, 0.0),
                    ..Transform2D::default()
                },
                ChildOf(parent),
            ))
            .id();
        world.entity_mut(parent).insert(Children(vec![child]));

        // Act
        run_transform_system(&mut world);

        // Assert
        let child_global = world.get::<GlobalTransform2D>(child).unwrap();
        assert!((child_global.0.translation.x - 15.0).abs() < 1e-6);
        assert!((child_global.0.translation.y).abs() < 1e-6);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When entity has no transform2d, then propagation system does not insert global transform</summary>

<code>crates\engine_scene\src\transform_propagation.rs:100</code>

```rust
        // Arrange
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        // Act
        run_transform_system(&mut world);

        // Assert
        assert!(world.get::<GlobalTransform2D>(entity).is_none());
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When child has identity transform, then global transform equals parent</summary>

<code>crates\engine_scene\src\transform_propagation.rs:113</code>

```rust
        // Arrange
        let mut world = World::new();
        let parent = world
            .spawn(Transform2D {
                position: Vec2::new(5.0, 0.0),
                ..Transform2D::default()
            })
            .id();
        let child = world.spawn((Transform2D::default(), ChildOf(parent))).id();
        world.entity_mut(parent).insert(Children(vec![child]));

        // Act
        run_transform_system(&mut world);

        // Assert
        let child_global = world.get::<GlobalTransform2D>(child).unwrap();
        assert!((child_global.0.translation.x - 5.0).abs() < 1e-6);
        assert!((child_global.0.translation.y).abs() < 1e-6);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When parent has scale and child has translation, then child position is scaled</summary>

<code>crates\engine_scene\src\transform_propagation.rs:166</code>

```rust
        // Arrange
        let mut world = World::new();
        let parent = world
            .spawn(Transform2D {
                scale: Vec2::splat(2.0),
                ..Transform2D::default()
            })
            .id();
        let child = world
            .spawn((
                Transform2D {
                    position: Vec2::new(3.0, 0.0),
                    ..Transform2D::default()
                },
                ChildOf(parent),
            ))
            .id();
        world.entity_mut(parent).insert(Children(vec![child]));

        // Act
        run_transform_system(&mut world);

        // Assert
        let child_global = world.get::<GlobalTransform2D>(child).unwrap();
        assert!((child_global.0.translation.x - 6.0).abs() < 1e-6);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When root entity has translation only, then global transform matches</summary>

<code>crates\engine_scene\src\transform_propagation.rs:81</code>

```rust
        // Arrange
        let mut world = World::new();
        let t = Transform2D {
            position: Vec2::new(10.0, 20.0),
            ..Transform2D::default()
        };
        let entity = world.spawn(t).id();

        // Act
        run_transform_system(&mut world);

        // Assert
        let global = world.get::<GlobalTransform2D>(entity).unwrap();
        assert!((global.0.translation.x - 10.0).abs() < 1e-6);
        assert!((global.0.translation.y - 20.0).abs() < 1e-6);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When root entity has identity transform, then global transform equals affine2 identity</summary>

*Root entities (no `ChildOf`) copy `Transform2D` directly to `GlobalTransform2D`*

<code>crates\engine_scene\src\transform_propagation.rs:67</code>

```rust
        // Arrange
        let mut world = World::new();
        let entity = world.spawn(Transform2D::default()).id();

        // Act
        run_transform_system(&mut world);

        // Assert
        let global = world.get::<GlobalTransform2D>(entity).unwrap();
        assert_eq!(global.0, Affine2::IDENTITY);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When hierarchy system runs before propagation, then children receive global transform</summary>

<code>crates\engine_scene\src\transform_propagation.rs:305</code>

```rust
        // Arrange
        let mut world = World::new();
        let parent = world
            .spawn(Transform2D {
                position: Vec2::new(10.0, 0.0),
                ..Transform2D::default()
            })
            .id();
        let child = world
            .spawn((
                Transform2D {
                    position: Vec2::new(5.0, 0.0),
                    ..Transform2D::default()
                },
                ChildOf(parent),
            ))
            .id();

        // Act
        run_hierarchy_system(&mut world);
        run_transform_system(&mut world);

        // Assert
        let parent_global = world.get::<GlobalTransform2D>(parent).unwrap();
        assert!((parent_global.0.translation.x - 10.0).abs() < 1e-6);
        let child_global = world.get::<GlobalTransform2D>(child).unwrap();
        assert!((child_global.0.translation.x - 15.0).abs() < 1e-6);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When multiple root entities, then each gets independent global transform</summary>

<code>crates\engine_scene\src\transform_propagation.rs:278</code>

```rust
        // Arrange
        let mut world = World::new();
        let root_a = world
            .spawn(Transform2D {
                position: Vec2::new(5.0, 0.0),
                ..Transform2D::default()
            })
            .id();
        let root_b = world
            .spawn(Transform2D {
                position: Vec2::new(0.0, 7.0),
                ..Transform2D::default()
            })
            .id();

        // Act
        run_transform_system(&mut world);

        // Assert
        let a_global = world.get::<GlobalTransform2D>(root_a).unwrap();
        assert!((a_global.0.translation.x - 5.0).abs() < 1e-6);
        let b_global = world.get::<GlobalTransform2D>(root_b).unwrap();
        assert!((b_global.0.translation.y - 7.0).abs() < 1e-6);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When transform updated and system reruns, then global transform reflects new value</summary>

<code>crates\engine_scene\src\transform_propagation.rs:336</code>

```rust
        // Arrange
        let mut world = World::new();
        let entity = world
            .spawn(Transform2D {
                position: Vec2::new(1.0, 0.0),
                ..Transform2D::default()
            })
            .id();
        run_transform_system(&mut world);
        world
            .entity_mut(entity)
            .get_mut::<Transform2D>()
            .unwrap()
            .position = Vec2::new(99.0, 0.0);

        // Act
        run_transform_system(&mut world);

        // Assert
        let global = world.get::<GlobalTransform2D>(entity).unwrap();
        assert!((global.0.translation.x - 99.0).abs() < 1e-6);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When root entity has transform2d, then propagation system inserts global transform</summary>

<code>crates\engine_scene\src\transform_propagation.rs:53</code>

```rust
        // Arrange
        let mut world = World::new();
        let entity = world.spawn(Transform2D::default()).id();

        // Act
        run_transform_system(&mut world);

        // Assert
        assert!(world.get::<GlobalTransform2D>(entity).is_some());
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When three level hierarchy, then grandchild accumulates all ancestors</summary>

<code>crates\engine_scene\src\transform_propagation.rs:195</code>

```rust
        // Arrange
        let mut world = World::new();
        let root = world
            .spawn(Transform2D {
                position: Vec2::new(1.0, 0.0),
                ..Transform2D::default()
            })
            .id();
        let child = world
            .spawn((
                Transform2D {
                    position: Vec2::new(2.0, 0.0),
                    ..Transform2D::default()
                },
                ChildOf(root),
            ))
            .id();
        let grandchild = world
            .spawn((
                Transform2D {
                    position: Vec2::new(3.0, 0.0),
                    ..Transform2D::default()
                },
                ChildOf(child),
            ))
            .id();
        world.entity_mut(root).insert(Children(vec![child]));
        world.entity_mut(child).insert(Children(vec![grandchild]));

        // Act
        run_transform_system(&mut world);

        // Assert
        let grandchild_global = world.get::<GlobalTransform2D>(grandchild).unwrap();
        assert!((grandchild_global.0.translation.x - 6.0).abs() < 1e-6);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When two siblings, then each gets independent global transform</summary>

<code>crates\engine_scene\src\transform_propagation.rs:234</code>

```rust
        // Arrange
        let mut world = World::new();
        let parent = world
            .spawn(Transform2D {
                position: Vec2::new(10.0, 0.0),
                ..Transform2D::default()
            })
            .id();
        let child_a = world
            .spawn((
                Transform2D {
                    position: Vec2::new(1.0, 0.0),
                    ..Transform2D::default()
                },
                ChildOf(parent),
            ))
            .id();
        let child_b = world
            .spawn((
                Transform2D {
                    position: Vec2::new(0.0, 2.0),
                    ..Transform2D::default()
                },
                ChildOf(parent),
            ))
            .id();
        world
            .entity_mut(parent)
            .insert(Children(vec![child_a, child_b]));

        // Act
        run_transform_system(&mut world);

        // Assert
        let a_global = world.get::<GlobalTransform2D>(child_a).unwrap();
        assert!((a_global.0.translation.x - 11.0).abs() < 1e-6);
        assert!((a_global.0.translation.y).abs() < 1e-6);
        let b_global = world.get::<GlobalTransform2D>(child_b).unwrap();
        assert!((b_global.0.translation.x - 10.0).abs() < 1e-6);
        assert!((b_global.0.translation.y - 2.0).abs() < 1e-6);
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test visibility</strong> (12 tests)</summary>

<blockquote>
<details>
<summary>✅ When child has no visible component and parent is hidden, then child effective visibility is false</summary>

<code>crates\engine_scene\src\visibility.rs:255</code>

```rust
        // Arrange
        let mut world = World::new();
        let parent = world.spawn(Visible(false)).id();
        let child = world.spawn(ChildOf(parent)).id();
        world.entity_mut(parent).insert(Children(vec![child]));

        // Act
        run_visibility_system(&mut world);

        // Assert
        let effective = world.get::<EffectiveVisibility>(child).unwrap();
        assert_eq!(*effective, EffectiveVisibility(false));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When child has no visible component and parent is visible, then child effective visibility is true</summary>

<code>crates\engine_scene\src\visibility.rs:238</code>

```rust
        // Arrange
        let mut world = World::new();
        let parent = world.spawn(Visible(true)).id();
        let child = world.spawn(ChildOf(parent)).id();
        world.entity_mut(parent).insert(Children(vec![child]));

        // Act
        run_visibility_system(&mut world);

        // Assert
        let effective = world.get::<EffectiveVisibility>(child).unwrap();
        assert_eq!(*effective, EffectiveVisibility(true));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When root entity has no visible component, then visibility system inserts effective visibility true</summary>

*Visible is opt-in — entities without it default to visible (no component = no hiding)*

<code>crates\engine_scene\src\visibility.rs:96</code>

```rust
        // Arrange
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        // Act
        run_visibility_system(&mut world);

        // Assert
        let effective = world.get::<EffectiveVisibility>(entity).unwrap();
        assert_eq!(*effective, EffectiveVisibility(true));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When hierarchy system runs before visibility system, then children receive effective visibility</summary>

<code>crates\engine_scene\src\visibility.rs:199</code>

```rust
        // Arrange
        let mut world = World::new();
        let parent = world.spawn(Visible(true)).id();
        let child = world.spawn((Visible(true), ChildOf(parent))).id();

        // Act
        crate::test_helpers::run_hierarchy_system(&mut world);
        run_visibility_system(&mut world);

        // Assert
        let effective = world.get::<EffectiveVisibility>(child).unwrap();
        assert_eq!(*effective, EffectiveVisibility(true));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When parent is visible and child is hidden, then child effective visibility is false</summary>

<code>crates\engine_scene\src\visibility.rs:144</code>

```rust
        // Arrange
        let mut world = World::new();
        let parent = world.spawn(Visible(true)).id();
        let child = world.spawn((Visible(false), ChildOf(parent))).id();
        world.entity_mut(parent).insert(Children(vec![child]));

        // Act
        run_visibility_system(&mut world);

        // Assert
        let effective = world.get::<EffectiveVisibility>(child).unwrap();
        assert_eq!(*effective, EffectiveVisibility(false));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When parent is hidden and child is visible, then child effective visibility is false</summary>

*AND-logic propagation: `EffectiveVisibility` = `parent_effective` AND `child_visible`*

<code>crates\engine_scene\src\visibility.rs:128</code>

```rust
        // Arrange
        let mut world = World::new();
        let parent = world.spawn(Visible(false)).id();
        let child = world.spawn((Visible(true), ChildOf(parent))).id();
        world.entity_mut(parent).insert(Children(vec![child]));

        // Act
        run_visibility_system(&mut world);

        // Assert
        let effective = world.get::<EffectiveVisibility>(child).unwrap();
        assert_eq!(*effective, EffectiveVisibility(false));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When root entity has visible false, then visibility system inserts effective visibility false</summary>

<code>crates\engine_scene\src\visibility.rs:80</code>

```rust
        // Arrange
        let mut world = World::new();
        let entity = world.spawn(Visible(false)).id();

        // Act
        run_visibility_system(&mut world);

        // Assert
        let effective = world.get::<EffectiveVisibility>(entity).unwrap();
        assert_eq!(*effective, EffectiveVisibility(false));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When root entity has default visible, then visibility system inserts effective visibility true</summary>

<code>crates\engine_scene\src\visibility.rs:65</code>

```rust
        // Arrange
        let mut world = World::new();
        let entity = world.spawn(Visible::default()).id();

        // Act
        run_visibility_system(&mut world);

        // Assert
        let effective = world.get::<EffectiveVisibility>(entity).unwrap();
        assert_eq!(*effective, EffectiveVisibility(true));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When parent visibility changed and system reruns, then child effective visibility updates</summary>

<code>crates\engine_scene\src\visibility.rs:216</code>

```rust
        // Arrange
        let mut world = World::new();
        let parent = world.spawn(Visible(false)).id();
        let child = world.spawn((Visible(true), ChildOf(parent))).id();
        world.entity_mut(parent).insert(Children(vec![child]));
        run_visibility_system(&mut world);
        assert_eq!(
            *world.get::<EffectiveVisibility>(child).unwrap(),
            EffectiveVisibility(false)
        );
        world.entity_mut(parent).insert(Visible(true));

        // Act
        run_visibility_system(&mut world);

        // Assert
        let effective = world.get::<EffectiveVisibility>(child).unwrap();
        assert_eq!(*effective, EffectiveVisibility(true));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When three level hierarchy with hidden root, then grandchild effective visibility is false</summary>

<code>crates\engine_scene\src\visibility.rs:160</code>

```rust
        // Arrange
        let mut world = World::new();
        let root = world.spawn(Visible(false)).id();
        let child = world.spawn((Visible(true), ChildOf(root))).id();
        let grandchild = world.spawn((Visible(true), ChildOf(child))).id();
        world.entity_mut(root).insert(Children(vec![child]));
        world.entity_mut(child).insert(Children(vec![grandchild]));

        // Act
        run_visibility_system(&mut world);

        // Assert
        let effective = world.get::<EffectiveVisibility>(grandchild).unwrap();
        assert_eq!(*effective, EffectiveVisibility(false));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When visible parent has visible child, then child effective visibility is true</summary>

<code>crates\engine_scene\src\visibility.rs:111</code>

```rust
        // Arrange
        let mut world = World::new();
        let parent = world.spawn(Visible(true)).id();
        let child = world.spawn((Visible(true), ChildOf(parent))).id();
        world.entity_mut(parent).insert(Children(vec![child]));

        // Act
        run_visibility_system(&mut world);

        // Assert
        let effective = world.get::<EffectiveVisibility>(child).unwrap();
        assert_eq!(*effective, EffectiveVisibility(true));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When two siblings one hidden, then each gets independent effective visibility</summary>

<code>crates\engine_scene\src\visibility.rs:178</code>

```rust
        // Arrange
        let mut world = World::new();
        let parent = world.spawn(Visible(true)).id();
        let child_a = world.spawn((Visible(true), ChildOf(parent))).id();
        let child_b = world.spawn((Visible(false), ChildOf(parent))).id();
        world
            .entity_mut(parent)
            .insert(Children(vec![child_a, child_b]));

        // Act
        run_visibility_system(&mut world);

        // Assert
        let a_effective = world.get::<EffectiveVisibility>(child_a).unwrap();
        assert_eq!(*a_effective, EffectiveVisibility(true));
        let b_effective = world.get::<EffectiveVisibility>(child_b).unwrap();
        assert_eq!(*b_effective, EffectiveVisibility(false));
```

</details>
</blockquote>

</details>
</blockquote>

</details>

<details>
<summary><strong>engine_ui</strong> (77 tests)</summary>

<blockquote>
<details>
<summary><strong>test anchor</strong> (8 tests)</summary>

<blockquote>
<details>
<summary>✅ When bottom center anchor, then half width full height</summary>

<code>crates\engine_ui\src\anchor.rs:74</code>

```rust
        // Arrange
        let size = Vec2::new(100.0, 60.0);

        // Act
        let offset = anchor_offset(Anchor::BottomCenter, size);

        // Assert
        assert_eq!(offset, Vec2::new(-50.0, -60.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When all nine anchors with asymmetric size, then exact offsets</summary>

<code>crates\engine_ui\src\anchor.rs:115</code>

```rust
        // Arrange
        let size = Vec2::new(80.0, 40.0);

        // Act / Assert
        assert_eq!(anchor_offset(Anchor::TopLeft, size), Vec2::new(0.0, 0.0));
        assert_eq!(
            anchor_offset(Anchor::TopCenter, size),
            Vec2::new(-40.0, 0.0)
        );
        assert_eq!(anchor_offset(Anchor::TopRight, size), Vec2::new(-80.0, 0.0));
        assert_eq!(
            anchor_offset(Anchor::CenterLeft, size),
            Vec2::new(0.0, -20.0)
        );
        assert_eq!(anchor_offset(Anchor::Center, size), Vec2::new(-40.0, -20.0));
        assert_eq!(
            anchor_offset(Anchor::CenterRight, size),
            Vec2::new(-80.0, -20.0)
        );
        assert_eq!(
            anchor_offset(Anchor::BottomLeft, size),
            Vec2::new(0.0, -40.0)
        );
        assert_eq!(
            anchor_offset(Anchor::BottomCenter, size),
            Vec2::new(-40.0, -40.0)
        );
        assert_eq!(
            anchor_offset(Anchor::BottomRight, size),
            Vec2::new(-80.0, -40.0)
        );
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When center anchor, then negative half size</summary>

<code>crates\engine_ui\src\anchor.rs:38</code>

```rust
        // Arrange
        let size = Vec2::new(100.0, 60.0);

        // Act
        let offset = anchor_offset(Anchor::Center, size);

        // Assert
        assert_eq!(offset, Vec2::new(-50.0, -30.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When all nine anchors, then all offsets distinct</summary>

<code>crates\engine_ui\src\anchor.rs:150</code>

```rust
        // Arrange
        let size = Vec2::new(100.0, 60.0);
        let anchors = [
            Anchor::TopLeft,
            Anchor::TopCenter,
            Anchor::TopRight,
            Anchor::CenterLeft,
            Anchor::Center,
            Anchor::CenterRight,
            Anchor::BottomLeft,
            Anchor::BottomCenter,
            Anchor::BottomRight,
        ];

        // Act
        let offsets: Vec<Vec2> = anchors.iter().map(|a| anchor_offset(*a, size)).collect();

        // Assert
        for i in 0..offsets.len() {
            for j in (i + 1)..offsets.len() {
                assert_ne!(offsets[i], offsets[j], "{anchors:?}[{i}] and [{j}] collide");
            }
        }
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When top left anchor, then zero offset</summary>

<code>crates\engine_ui\src\anchor.rs:50</code>

```rust
        // Arrange
        let size = Vec2::new(100.0, 50.0);

        // Act
        let offset = anchor_offset(Anchor::TopLeft, size);

        // Assert
        assert_eq!(offset, Vec2::ZERO);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When top right anchor, then negative width</summary>

<code>crates\engine_ui\src\anchor.rs:62</code>

```rust
        // Arrange
        let size = Vec2::new(80.0, 40.0);

        // Act
        let offset = anchor_offset(Anchor::TopRight, size);

        // Assert
        assert_eq!(offset, Vec2::new(-80.0, 0.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When bottom right anchor and any size, then offset is negative size</summary>

<code>crates\engine_ui\src\anchor.rs:99</code>

```rust
            w in 0.0_f32..=1000.0,
            h in 0.0_f32..=1000.0,
        ) {
            // Arrange
            let size = Vec2::new(w, h);

            // Act
            let offset = anchor_offset(Anchor::BottomRight, size);

            // Assert
            assert_eq!(offset, -size);
        }
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When top left anchor and any size, then offset is zero</summary>

<code>crates\engine_ui\src\anchor.rs:87</code>

```rust
            w in 0.0_f32..=1000.0,
            h in 0.0_f32..=1000.0,
        ) {
            // Act
            let offset = anchor_offset(Anchor::TopLeft, Vec2::new(w, h));

            // Assert
            assert_eq!(offset, Vec2::ZERO);
        }

        #[test]
        fn when_bottom_right_anchor_and_any_size_then_offset_is_negative_size(
            w in 0.0_f32..=1000.0,
            h in 0.0_f32..=1000.0,
        ) {
            // Arrange
            let size = Vec2::new(w, h);

            // Act
            let offset = anchor_offset(Anchor::BottomRight, size);

            // Assert
            assert_eq!(offset, -size);
        }
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test button</strong> (8 tests)</summary>

<blockquote>
<details>
<summary>✅ When button invisible, then no draw</summary>

<code>crates\engine_ui\src\button.rs:186</code>

```rust
        // Arrange
        let (mut world, mut schedule, log, _) = setup_world_with_spy();
        world.spawn((
            Button { disabled: false },
            UiNode {
                size: Vec2::new(100.0, 40.0),
                ..UiNode::default()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            EffectiveVisibility(false),
        ));

        // Act
        schedule.run(&mut world);

        // Assert
        let calls = log.lock().unwrap();
        assert_eq!(calls.iter().filter(|c| *c == "draw_rect").count(), 0);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When button disabled, then disabled color used regardless of interaction</summary>

<code>crates\engine_ui\src\button.rs:163</code>

```rust
        // Arrange
        let (mut world, mut schedule, _, rects) = setup_world_with_spy();
        world.spawn((
            Button { disabled: true },
            UiNode {
                size: Vec2::new(100.0, 40.0),
                ..UiNode::default()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            Interaction::Hovered,
        ));

        // Act
        schedule.run(&mut world);

        // Assert
        let rects = rects.lock().unwrap();
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].color, UiTheme::default().disabled_color);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When button hovered, then hovered color used</summary>

<code>crates\engine_ui\src\button.rs:117</code>

```rust
        // Arrange
        let (mut world, mut schedule, _, rects) = setup_world_with_spy();
        world.spawn((
            Button { disabled: false },
            UiNode {
                size: Vec2::new(100.0, 40.0),
                ..UiNode::default()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            Interaction::Hovered,
        ));

        // Act
        schedule.run(&mut world);

        // Assert
        let rects = rects.lock().unwrap();
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].color, UiTheme::default().hovered_color);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When button roundtrip ron, then disabled preserved</summary>

<code>crates\engine_ui\src\button.rs:80</code>

```rust
        // Arrange
        let button = Button { disabled: true };

        // Act
        let ron_str = ron::to_string(&button).unwrap();
        let restored: Button = ron::from_str(&ron_str).unwrap();

        // Assert
        assert_eq!(restored, button);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When button with center anchor, then draw rect offset applied</summary>

<code>crates\engine_ui\src\button.rs:208</code>

```rust
        // Arrange
        let (mut world, mut schedule, _, rects) = setup_world_with_spy();
        world.spawn((
            Button { disabled: false },
            UiNode {
                size: Vec2::new(100.0, 40.0),
                anchor: Anchor::Center,
                ..UiNode::default()
            },
            GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 100.0))),
            Interaction::None,
        ));

        // Act
        schedule.run(&mut world);

        // Assert — Center anchor offset = (-50, -20), so top_left = (150, 80)
        let rects = rects.lock().unwrap();
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].x, Pixels(150.0));
        assert_eq!(rects[0].y, Pixels(80.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When button pressed, then pressed color used</summary>

<code>crates\engine_ui\src\button.rs:140</code>

```rust
        // Arrange
        let (mut world, mut schedule, _, rects) = setup_world_with_spy();
        world.spawn((
            Button { disabled: false },
            UiNode {
                size: Vec2::new(100.0, 40.0),
                ..UiNode::default()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            Interaction::Pressed,
        ));

        // Act
        schedule.run(&mut world);

        // Assert
        let rects = rects.lock().unwrap();
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].color, UiTheme::default().pressed_color);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When button not hovered, then normal color used</summary>

<code>crates\engine_ui\src\button.rs:93</code>

```rust
        // Arrange
        let (mut world, mut schedule, _, rects) = setup_world_with_spy();
        world.spawn((
            Button { disabled: false },
            UiNode {
                size: Vec2::new(100.0, 40.0),
                anchor: Anchor::TopLeft,
                ..UiNode::default()
            },
            GlobalTransform2D(Affine2::from_translation(Vec2::new(50.0, 80.0))),
            Interaction::None,
        ));

        // Act
        schedule.run(&mut world);

        // Assert
        let rects = rects.lock().unwrap();
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].color, UiTheme::default().normal_color);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When button rendered, then position and size match node</summary>

<code>crates\engine_ui\src\button.rs:233</code>

```rust
        // Arrange
        let (mut world, mut schedule, _, rects) = setup_world_with_spy();
        world.spawn((
            Button { disabled: false },
            UiNode {
                size: Vec2::new(100.0, 40.0),
                anchor: Anchor::TopLeft,
                ..UiNode::default()
            },
            GlobalTransform2D(Affine2::from_translation(Vec2::new(50.0, 80.0))),
            Interaction::None,
        ));

        // Act
        schedule.run(&mut world);

        // Assert
        let rects = rects.lock().unwrap();
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].x, Pixels(50.0));
        assert_eq!(rects[0].y, Pixels(80.0));
        assert_eq!(rects[0].width, Pixels(100.0));
        assert_eq!(rects[0].height, Pixels(40.0));
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test flex_layout</strong> (8 tests)</summary>

<blockquote>
<details>
<summary>✅ When column with bottom margin and gap, then spacing accumulates</summary>

<code>crates\engine_ui\src\flex_layout.rs:196</code>

```rust
        // Arrange
        let layout = FlexLayout {
            direction: FlexDirection::Column,
            gap: 5.0,
        };
        let children = [
            (
                Vec2::new(50.0, 20.0),
                Margin {
                    bottom: 10.0,
                    ..Margin::default()
                },
            ),
            (
                Vec2::new(50.0, 30.0),
                Margin {
                    top: 3.0,
                    bottom: 7.0,
                    ..Margin::default()
                },
            ),
            (Vec2::new(50.0, 25.0), Margin::default()),
        ];

        // Act
        let offsets = compute_flex_offsets(&layout, &children);

        // Assert
        // child[0]: cursor starts 0, leading=0, offset=(0,0), extent=20+10=30, gap=5, cursor=35
        // child[1]: leading=3, cursor=38, offset=(0,38), extent=30+7=37, gap=5, cursor=80
        // child[2]: leading=0, cursor=80, offset=(0,80)
        assert_eq!(offsets.len(), 3);
        assert_eq!(offsets[0], Vec2::new(0.0, 0.0));
        assert_eq!(offsets[1], Vec2::new(0.0, 38.0));
        assert_eq!(offsets[2], Vec2::new(0.0, 80.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When column with gap, then children vertical</summary>

<code>crates\engine_ui\src\flex_layout.rs:94</code>

```rust
        // Arrange
        let layout = FlexLayout {
            direction: FlexDirection::Column,
            gap: 4.0,
        };
        let children = [
            (Vec2::new(50.0, 20.0), Margin::default()),
            (Vec2::new(50.0, 30.0), Margin::default()),
        ];

        // Act
        let offsets = compute_flex_offsets(&layout, &children);

        // Assert
        assert_eq!(offsets, vec![Vec2::ZERO, Vec2::new(0.0, 24.0)]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When empty children, then empty offsets</summary>

<code>crates\engine_ui\src\flex_layout.rs:235</code>

```rust
        // Arrange
        let layout = FlexLayout {
            direction: FlexDirection::Row,
            gap: 5.0,
        };

        // Act
        let offsets = compute_flex_offsets(&layout, &[]);

        // Assert
        assert!(offsets.is_empty());
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When row no gap, then children horizontal</summary>

<code>crates\engine_ui\src\flex_layout.rs:56</code>

```rust
        // Arrange
        let layout = FlexLayout {
            direction: FlexDirection::Row,
            gap: 0.0,
        };
        let children = [
            (Vec2::new(40.0, 20.0), Margin::default()),
            (Vec2::new(60.0, 30.0), Margin::default()),
        ];

        // Act
        let offsets = compute_flex_offsets(&layout, &children);

        // Assert
        assert_eq!(offsets, vec![Vec2::ZERO, Vec2::new(40.0, 0.0)]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When row with gap, then gap between children</summary>

<code>crates\engine_ui\src\flex_layout.rs:75</code>

```rust
        // Arrange
        let layout = FlexLayout {
            direction: FlexDirection::Row,
            gap: 8.0,
        };
        let children = [
            (Vec2::new(40.0, 20.0), Margin::default()),
            (Vec2::new(60.0, 30.0), Margin::default()),
        ];

        // Act
        let offsets = compute_flex_offsets(&layout, &children);

        // Assert
        assert_eq!(offsets, vec![Vec2::ZERO, Vec2::new(48.0, 0.0)]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When row with margins, then margins in spacing</summary>

<code>crates\engine_ui\src\flex_layout.rs:113</code>

```rust
        // Arrange
        let layout = FlexLayout {
            direction: FlexDirection::Row,
            gap: 0.0,
        };
        let children = [
            (
                Vec2::new(40.0, 20.0),
                Margin {
                    right: 5.0,
                    ..Margin::default()
                },
            ),
            (
                Vec2::new(60.0, 30.0),
                Margin {
                    left: 3.0,
                    ..Margin::default()
                },
            ),
        ];

        // Act
        let offsets = compute_flex_offsets(&layout, &children);

        // Assert
        assert_eq!(offsets, vec![Vec2::ZERO, Vec2::new(48.0, 0.0)]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When single child, then offset at origin</summary>

<code>crates\engine_ui\src\flex_layout.rs:144</code>

```rust
        // Arrange
        let layout = FlexLayout {
            direction: FlexDirection::Row,
            gap: 10.0,
        };
        let children = [(Vec2::new(50.0, 30.0), Margin::default())];

        // Act
        let offsets = compute_flex_offsets(&layout, &children);

        // Assert
        assert_eq!(offsets, vec![Vec2::ZERO]);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When any row children, then output length matches and x offsets increase</summary>

<code>crates\engine_ui\src\flex_layout.rs:161</code>

```rust
            gap in 0.0_f32..=20.0,
            widths in proptest::collection::vec(1.0_f32..=200.0, 1..=8),
        ) {
            // Arrange
            let layout = FlexLayout {
                direction: FlexDirection::Row,
                gap,
            };
            let children: Vec<(Vec2, Margin)> = widths
                .iter()
                .map(|&w| (Vec2::new(w, 20.0), Margin::default()))
                .collect();

            // Act
            let offsets = compute_flex_offsets(&layout, &children);

            // Assert — length matches
            assert_eq!(offsets.len(), children.len());

            // Assert — x offsets are strictly increasing (positive sizes, zero margin)
            for i in 1..offsets.len() {
                assert!(
                    offsets[i].x > offsets[i - 1].x,
                    "offsets[{}].x={} should be > offsets[{}].x={}",
                    i,
                    offsets[i].x,
                    i - 1,
                    offsets[i - 1].x
                );
            }
        }
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test interaction</strong> (19 tests)</summary>

<blockquote>
<details>
<summary>✅ When cursor inside and left held, then interaction becomes pressed</summary>

<code>crates\engine_ui\src\interaction.rs:202</code>

```rust
        // Arrange
        let mut world = setup_world(Vec2::new(250.0, 120.0));
        world.resource_mut::<MouseState>().press(MouseButton::Left);

        let entity = world
            .spawn((
                UiNode {
                    size: Vec2::new(100.0, 50.0),
                    anchor: Anchor::TopLeft,
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 100.0))),
                Interaction::default(),
            ))
            .id();

        // Act
        run_system(&mut world);

        // Assert
        let interaction = world.entity(entity).get::<Interaction>().unwrap();
        assert_eq!(*interaction, Interaction::Pressed);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When cursor inside node, then interaction becomes hovered</summary>

*AABB hit-test uses `anchor_offset` to compute top-left from node position + size*

<code>crates\engine_ui\src\interaction.rs:127</code>

```rust
        // Arrange
        let mut world = setup_world(Vec2::new(250.0, 120.0));

        let entity = world
            .spawn((
                UiNode {
                    size: Vec2::new(100.0, 50.0),
                    anchor: Anchor::TopLeft,
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 100.0))),
                Interaction::default(),
            ))
            .id();

        // Act
        run_system(&mut world);

        // Assert
        let interaction = world.entity(entity).get::<Interaction>().unwrap();
        assert_eq!(*interaction, Interaction::Hovered);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When cursor enters node, then hover enter event emitted</summary>

<code>crates\engine_ui\src\interaction.rs:407</code>

```rust
        // Arrange
        let mut world = setup_world(Vec2::new(250.0, 120.0));

        let entity = world
            .spawn((
                UiNode {
                    size: Vec2::new(100.0, 50.0),
                    anchor: Anchor::TopLeft,
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 100.0))),
                Interaction::default(),
            ))
            .id();

        // Act
        run_system(&mut world);

        // Assert
        let events: Vec<UiEvent> = world.resource_mut::<UiEventBuffer>().drain().collect();
        assert!(events.contains(&UiEvent::HoverEnter(entity)));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When cursor on node boundary, then interaction becomes hovered</summary>

<code>crates\engine_ui\src\interaction.rs:152</code>

```rust
        // Arrange
        let mut world = setup_world(Vec2::new(200.0, 100.0));

        let entity = world
            .spawn((
                UiNode {
                    size: Vec2::new(100.0, 50.0),
                    anchor: Anchor::TopLeft,
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 100.0))),
                Interaction::default(),
            ))
            .id();

        // Act
        run_system(&mut world);

        // Assert
        let interaction = world.entity(entity).get::<Interaction>().unwrap();
        assert_eq!(*interaction, Interaction::Hovered);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When cursor inside x range but outside y range, then not hovered</summary>

<code>crates\engine_ui\src\interaction.rs:542</code>

```rust
        // Arrange — node at (200, 100), size 100x50, cursor at (250, 200) → inside x, outside y
        let mut world = setup_world(Vec2::new(250.0, 200.0));

        let entity = world
            .spawn((
                UiNode {
                    size: Vec2::new(100.0, 50.0),
                    anchor: Anchor::TopLeft,
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 100.0))),
                Interaction::default(),
            ))
            .id();

        // Act
        run_system(&mut world);

        // Assert
        let interaction = world.entity(entity).get::<Interaction>().unwrap();
        assert_eq!(*interaction, Interaction::None);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When cursor leaves node, then hover exit event emitted</summary>

<code>crates\engine_ui\src\interaction.rs:432</code>

```rust
        // Arrange
        let mut world = setup_world(Vec2::new(250.0, 120.0));

        let entity = world
            .spawn((
                UiNode {
                    size: Vec2::new(100.0, 50.0),
                    anchor: Anchor::TopLeft,
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 100.0))),
                Interaction::default(),
            ))
            .id();

        run_system(&mut world);
        let _ = world.resource_mut::<UiEventBuffer>().drain().count();

        // Act
        world
            .resource_mut::<MouseState>()
            .set_world_pos(Vec2::new(0.0, 0.0));
        run_system(&mut world);

        // Assert
        let events: Vec<UiEvent> = world.resource_mut::<UiEventBuffer>().drain().collect();
        assert!(events.contains(&UiEvent::HoverExit(entity)));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When cursor inside y range but outside x range, then not hovered</summary>

<code>crates\engine_ui\src\interaction.rs:567</code>

```rust
        // Arrange — node at (200, 100), size 100x50, cursor at (50, 120) → outside x, inside y
        let mut world = setup_world(Vec2::new(50.0, 120.0));

        let entity = world
            .spawn((
                UiNode {
                    size: Vec2::new(100.0, 50.0),
                    anchor: Anchor::TopLeft,
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 100.0))),
                Interaction::default(),
            ))
            .id();

        // Act
        run_system(&mut world);

        // Assert
        let interaction = world.entity(entity).get::<Interaction>().unwrap();
        assert_eq!(*interaction, Interaction::None);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When interaction roundtrip ron, then variant preserved</summary>

<code>crates\engine_ui\src\interaction.rs:592</code>

```rust
        // Arrange
        let variants = [
            Interaction::None,
            Interaction::Hovered,
            Interaction::Pressed,
        ];

        for variant in variants {
            // Act
            let ron_str = ron::to_string(&variant).unwrap();
            let restored: Interaction = ron::from_str(&ron_str).unwrap();

            // Assert
            assert_eq!(restored, variant);
        }
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When disabled button, then interaction stays none</summary>

*Disabled buttons are excluded from hit-testing entirely — not just visually dimmed*

<code>crates\engine_ui\src\interaction.rs:641</code>

```rust
        // Arrange
        let mut world = setup_world(Vec2::new(250.0, 120.0));
        world.resource_mut::<MouseState>().press(MouseButton::Left);

        let entity = world
            .spawn((
                UiNode {
                    size: Vec2::new(100.0, 50.0),
                    anchor: Anchor::TopLeft,
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 100.0))),
                Interaction::default(),
                crate::button::Button { disabled: true },
            ))
            .id();

        // Act
        run_system(&mut world);

        // Assert
        let interaction = world.entity(entity).get::<Interaction>().unwrap();
        assert_eq!(*interaction, Interaction::None);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When cursor leaves node, then interaction reverts to none</summary>

<code>crates\engine_ui\src\interaction.rs:344</code>

```rust
        // Arrange
        let mut world = setup_world(Vec2::new(250.0, 120.0));

        let entity = world
            .spawn((
                UiNode {
                    size: Vec2::new(100.0, 50.0),
                    anchor: Anchor::TopLeft,
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 100.0))),
                Interaction::default(),
            ))
            .id();

        run_system(&mut world);
        assert_eq!(
            *world.entity(entity).get::<Interaction>().unwrap(),
            Interaction::Hovered
        );

        // Act
        world
            .resource_mut::<MouseState>()
            .set_world_pos(Vec2::new(0.0, 0.0));
        run_system(&mut world);

        // Assert
        let interaction = world.entity(entity).get::<Interaction>().unwrap();
        assert_eq!(*interaction, Interaction::None);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When cursor outside node, then interaction remains none</summary>

<code>crates\engine_ui\src\interaction.rs:177</code>

```rust
        // Arrange
        let mut world = setup_world(Vec2::new(0.0, 0.0));

        let entity = world
            .spawn((
                UiNode {
                    size: Vec2::new(100.0, 50.0),
                    anchor: Anchor::TopLeft,
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 100.0))),
                Interaction::default(),
            ))
            .id();

        // Act
        run_system(&mut world);

        // Assert
        let interaction = world.entity(entity).get::<Interaction>().unwrap();
        assert_eq!(*interaction, Interaction::None);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When cursor outside and left held, then interaction remains none</summary>

<code>crates\engine_ui\src\interaction.rs:228</code>

```rust
        // Arrange
        let mut world = setup_world(Vec2::new(0.0, 0.0));
        world.resource_mut::<MouseState>().press(MouseButton::Left);

        let entity = world
            .spawn((
                UiNode {
                    size: Vec2::new(100.0, 50.0),
                    anchor: Anchor::TopLeft,
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 100.0))),
                Interaction::default(),
            ))
            .id();

        // Act
        run_system(&mut world);

        // Assert
        let interaction = world.entity(entity).get::<Interaction>().unwrap();
        assert_eq!(*interaction, Interaction::None);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When invisible entity with no prior hover, then no hover exit event</summary>

<code>crates\engine_ui\src\interaction.rs:611</code>

```rust
        // Arrange — entity starts at Interaction::None, is invisible
        let mut world = setup_world(Vec2::new(250.0, 120.0));

        let _ = world
            .spawn((
                UiNode {
                    size: Vec2::new(100.0, 50.0),
                    anchor: Anchor::TopLeft,
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 100.0))),
                Interaction::None,
                EffectiveVisibility(false),
            ))
            .id();

        // Act
        run_system(&mut world);

        // Assert — no HoverExit because prev was already None
        let events: Vec<UiEvent> = world.resource_mut::<UiEventBuffer>().drain().collect();
        assert!(
            !events.iter().any(|e| matches!(e, UiEvent::HoverExit(_))),
            "should not emit HoverExit when prev=None, got {events:?}"
        );
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When different node clicked, then focus transfers</summary>

<code>crates\engine_ui\src\interaction.rs:492</code>

```rust
        // Arrange
        let mut world = setup_world(Vec2::new(50.0, 50.0));
        world.resource_mut::<MouseState>().press(MouseButton::Left);

        let entity_a = world
            .spawn((
                UiNode {
                    size: Vec2::new(100.0, 100.0),
                    anchor: Anchor::TopLeft,
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
                Interaction::default(),
            ))
            .id();

        let _ = world
            .spawn((
                UiNode {
                    size: Vec2::new(100.0, 100.0),
                    anchor: Anchor::TopLeft,
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 200.0))),
                Interaction::default(),
            ))
            .id();

        run_system(&mut world);
        assert_eq!(world.resource::<FocusState>().focused, Some(entity_a));
        let _ = world.resource_mut::<UiEventBuffer>().drain().count();

        // Act — move cursor to entity_b and click
        {
            let mut mouse = world.resource_mut::<MouseState>();
            mouse.set_world_pos(Vec2::new(250.0, 250.0));
            mouse.clear_frame_state();
            mouse.press(MouseButton::Left);
        }
        run_system(&mut world);

        // Assert
        let focus = world.resource::<FocusState>();
        assert_ne!(focus.focused, Some(entity_a));
        let events: Vec<UiEvent> = world.resource_mut::<UiEventBuffer>().drain().collect();
        assert!(events.contains(&UiEvent::FocusLost(entity_a)));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When just pressed inside, then clicked event emitted</summary>

<code>crates\engine_ui\src\interaction.rs:378</code>

```rust
        // Arrange
        let mut world = setup_world(Vec2::new(250.0, 120.0));
        {
            let mut mouse = world.resource_mut::<MouseState>();
            mouse.press(MouseButton::Left);
        }

        let entity = world
            .spawn((
                UiNode {
                    size: Vec2::new(100.0, 50.0),
                    anchor: Anchor::TopLeft,
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 100.0))),
                Interaction::default(),
            ))
            .id();

        // Act
        run_system(&mut world);

        // Assert
        let events: Vec<UiEvent> = world.resource_mut::<UiEventBuffer>().drain().collect();
        assert!(events.contains(&UiEvent::Clicked(entity)));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When effective visibility false, then not hit tested</summary>

<code>crates\engine_ui\src\interaction.rs:279</code>

```rust
        // Arrange
        let mut world = setup_world(Vec2::new(250.0, 120.0));

        let entity = world
            .spawn((
                UiNode {
                    size: Vec2::new(100.0, 50.0),
                    anchor: Anchor::TopLeft,
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 100.0))),
                Interaction::default(),
                EffectiveVisibility(false),
            ))
            .id();

        // Act
        run_system(&mut world);

        // Assert
        let interaction = world.entity(entity).get::<Interaction>().unwrap();
        assert_eq!(*interaction, Interaction::None);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When node clicked, then focus state updated</summary>

*Click sets FocusState.focused — only one entity has focus at a time*

<code>crates\engine_ui\src\interaction.rs:464</code>

```rust
        // Arrange
        let mut world = setup_world(Vec2::new(250.0, 120.0));
        world.resource_mut::<MouseState>().press(MouseButton::Left);

        let entity = world
            .spawn((
                UiNode {
                    size: Vec2::new(100.0, 50.0),
                    anchor: Anchor::TopLeft,
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 100.0))),
                Interaction::default(),
            ))
            .id();

        // Act
        run_system(&mut world);

        // Assert
        let focus = world.resource::<FocusState>();
        assert_eq!(focus.focused, Some(entity));
        let events: Vec<UiEvent> = world.resource_mut::<UiEventBuffer>().drain().collect();
        assert!(events.contains(&UiEvent::FocusGained(entity)));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When node has center anchor, then hit test accounts for offset</summary>

<code>crates\engine_ui\src\interaction.rs:254</code>

```rust
        // Arrange
        let mut world = setup_world(Vec2::new(175.0, 180.0));

        let entity = world
            .spawn((
                UiNode {
                    size: Vec2::new(100.0, 50.0),
                    anchor: Anchor::Center,
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 200.0))),
                Interaction::default(),
            ))
            .id();

        // Act
        run_system(&mut world);

        // Assert
        let interaction = world.entity(entity).get::<Interaction>().unwrap();
        assert_eq!(*interaction, Interaction::Hovered);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When two overlapping nodes, then both receive interaction</summary>

<code>crates\engine_ui\src\interaction.rs:305</code>

```rust
        // Arrange
        let mut world = setup_world(Vec2::new(50.0, 50.0));

        let entity_a = world
            .spawn((
                UiNode {
                    size: Vec2::new(100.0, 100.0),
                    anchor: Anchor::TopLeft,
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
                Interaction::default(),
            ))
            .id();

        let entity_b = world
            .spawn((
                UiNode {
                    size: Vec2::new(100.0, 100.0),
                    anchor: Anchor::TopLeft,
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::from_translation(Vec2::ZERO)),
                Interaction::default(),
            ))
            .id();

        // Act
        run_system(&mut world);

        // Assert
        let a = *world.entity(entity_a).get::<Interaction>().unwrap();
        let b = *world.entity(entity_b).get::<Interaction>().unwrap();
        assert_eq!(a, Interaction::Hovered);
        assert_eq!(b, Interaction::Hovered);
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test layout</strong> (6 tests)</summary>

<blockquote>
<details>
<summary>✅ When column layout, then vertical stacking</summary>

<code>crates\engine_ui\src\layout.rs:161</code>

```rust
        // Arrange
        let mut world = World::new();
        let parent = spawn_flex_parent(
            &mut world,
            FlexLayout {
                direction: FlexDirection::Column,
                gap: 0.0,
            },
            Vec2::ZERO,
        );
        spawn_ui_child(&mut world, parent, Vec2::new(50.0, 30.0), Margin::default());
        let child_b = spawn_ui_child(&mut world, parent, Vec2::new(50.0, 20.0), Margin::default());

        // Act
        run_layout(&mut world);

        // Assert
        let transform = world.entity(child_b).get::<Transform2D>().unwrap();
        assert_eq!(transform.position, Vec2::new(0.0, 30.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When child has margin, then margin in spacing</summary>

<code>crates\engine_ui\src\layout.rs:206</code>

```rust
        // Arrange
        let mut world = World::new();
        let parent = spawn_flex_parent(
            &mut world,
            FlexLayout {
                direction: FlexDirection::Row,
                gap: 0.0,
            },
            Vec2::ZERO,
        );
        spawn_ui_child(
            &mut world,
            parent,
            Vec2::new(40.0, 20.0),
            Margin {
                right: 8.0,
                ..Margin::default()
            },
        );
        let child_b = spawn_ui_child(
            &mut world,
            parent,
            Vec2::new(40.0, 20.0),
            Margin {
                left: 4.0,
                ..Margin::default()
            },
        );

        // Act
        run_layout(&mut world);

        // Assert
        let transform = world.entity(child_b).get::<Transform2D>().unwrap();
        assert_eq!(transform.position, Vec2::new(52.0, 0.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When parent offset, then children relative</summary>

<code>crates\engine_ui\src\layout.rs:184</code>

```rust
        // Arrange
        let mut world = World::new();
        let parent = spawn_flex_parent(
            &mut world,
            FlexLayout {
                direction: FlexDirection::Row,
                gap: 0.0,
            },
            Vec2::new(200.0, 100.0),
        );
        let child = spawn_ui_child(&mut world, parent, Vec2::new(40.0, 20.0), Margin::default());

        // Act
        run_layout(&mut world);

        // Assert
        let transform = world.entity(child).get::<Transform2D>().unwrap();
        assert_eq!(transform.position, Vec2::new(200.0, 100.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When row layout, then first child at origin</summary>

<code>crates\engine_ui\src\layout.rs:92</code>

```rust
        // Arrange
        let mut world = World::new();
        let parent = spawn_flex_parent(
            &mut world,
            FlexLayout {
                direction: FlexDirection::Row,
                gap: 0.0,
            },
            Vec2::ZERO,
        );
        let child_a = spawn_ui_child(&mut world, parent, Vec2::new(60.0, 30.0), Margin::default());
        spawn_ui_child(&mut world, parent, Vec2::new(40.0, 20.0), Margin::default());

        // Act
        run_layout(&mut world);

        // Assert
        let transform = world.entity(child_a).get::<Transform2D>().unwrap();
        assert_eq!(transform.position, Vec2::ZERO);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When row layout, then second child offset by first width</summary>

<code>crates\engine_ui\src\layout.rs:115</code>

```rust
        // Arrange
        let mut world = World::new();
        let parent = spawn_flex_parent(
            &mut world,
            FlexLayout {
                direction: FlexDirection::Row,
                gap: 0.0,
            },
            Vec2::ZERO,
        );
        spawn_ui_child(&mut world, parent, Vec2::new(60.0, 30.0), Margin::default());
        let child_b = spawn_ui_child(&mut world, parent, Vec2::new(40.0, 20.0), Margin::default());

        // Act
        run_layout(&mut world);

        // Assert
        let transform = world.entity(child_b).get::<Transform2D>().unwrap();
        assert_eq!(transform.position, Vec2::new(60.0, 0.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When row layout with gap, then gap included</summary>

<code>crates\engine_ui\src\layout.rs:138</code>

```rust
        // Arrange
        let mut world = World::new();
        let parent = spawn_flex_parent(
            &mut world,
            FlexLayout {
                direction: FlexDirection::Row,
                gap: 10.0,
            },
            Vec2::ZERO,
        );
        spawn_ui_child(&mut world, parent, Vec2::new(60.0, 30.0), Margin::default());
        let child_b = spawn_ui_child(&mut world, parent, Vec2::new(40.0, 20.0), Margin::default());

        // Act
        run_layout(&mut world);

        // Assert
        let transform = world.entity(child_b).get::<Transform2D>().unwrap();
        assert_eq!(transform.position, Vec2::new(70.0, 0.0));
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test margin</strong> (2 tests)</summary>

<blockquote>
<details>
<summary>✅ When asymmetric margin, then correct pairs</summary>

<code>crates\engine_ui\src\margin.rs:37</code>

```rust
        // Arrange
        let margin = Margin {
            top: 5.0,
            right: 10.0,
            bottom: 15.0,
            left: 20.0,
        };

        // Act / Assert
        assert!((margin.total_horizontal() - 30.0).abs() < f32::EPSILON);
        assert!((margin.total_vertical() - 20.0).abs() < f32::EPSILON);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When zero margin, then totals zero</summary>

<code>crates\engine_ui\src\margin.rs:27</code>

```rust
        // Arrange
        let margin = Margin::default();

        // Act / Assert
        assert!((margin.total_horizontal() - 0.0).abs() < f32::EPSILON);
        assert!((margin.total_vertical() - 0.0).abs() < f32::EPSILON);
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test panel</strong> (5 tests)</summary>

<blockquote>
<details>
<summary>✅ When panel invisible, then no draw</summary>

<code>crates\engine_ui\src\panel.rs:229</code>

```rust
        // Arrange
        let (mut world, mut schedule, log, _) = setup_world_with_spy();
        world.spawn((
            Panel {
                border_color: Some(Color::RED),
                border_width: 2.0,
            },
            UiNode {
                size: Vec2::new(200.0, 150.0),
                background: Some(Color::WHITE),
                ..UiNode::default()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            EffectiveVisibility(false),
        ));

        // Act
        schedule.run(&mut world);

        // Assert
        let calls = log.lock().unwrap();
        assert_eq!(calls.iter().filter(|c| *c == "draw_rect").count(), 0);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When panel no border, then only background drawn</summary>

<code>crates\engine_ui\src\panel.rs:119</code>

```rust
        // Arrange
        let (mut world, mut schedule, log, _) = setup_world_with_spy();
        world.spawn((
            Panel {
                border_color: None,
                border_width: 0.0,
            },
            UiNode {
                size: Vec2::new(200.0, 150.0),
                anchor: Anchor::TopLeft,
                background: Some(Color::WHITE),
                ..UiNode::default()
            },
            GlobalTransform2D(Affine2::IDENTITY),
        ));

        // Act
        schedule.run(&mut world);

        // Assert
        let calls = log.lock().unwrap();
        assert_eq!(calls.iter().filter(|c| *c == "draw_rect").count(), 1);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When panel roundtrip ron, then border preserved</summary>

<code>crates\engine_ui\src\panel.rs:103</code>

```rust
        // Arrange
        let panel = Panel {
            border_color: Some(Color::from_u8(255, 0, 0, 255)),
            border_width: 3.0,
        };

        // Act
        let ron_str = ron::to_string(&panel).unwrap();
        let restored: Panel = ron::from_str(&ron_str).unwrap();

        // Assert
        assert_eq!(restored, panel);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When panel with center anchor and border, then exact border positions</summary>

<code>crates\engine_ui\src\panel.rs:172</code>

```rust
        // Arrange — panel at (200, 100) with Center anchor, size 120x80, border_width 4
        // Center offset = (-60, -40), so top_left = (140, 60)
        let (mut world, mut schedule, _, rects) = setup_world_with_spy();
        world.spawn((
            Panel {
                border_color: Some(Color::RED),
                border_width: 4.0,
            },
            UiNode {
                size: Vec2::new(120.0, 80.0),
                anchor: Anchor::Center,
                background: Some(Color::WHITE),
                ..UiNode::default()
            },
            GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 100.0))),
        ));

        // Act
        schedule.run(&mut world);

        // Assert — 1 background + 4 borders
        let rects = rects.lock().unwrap();
        assert_eq!(rects.len(), 5);

        // Background: top_left=(140,60), size=120x80
        assert_eq!(rects[0].x, Pixels(140.0));
        assert_eq!(rects[0].y, Pixels(60.0));
        assert_eq!(rects[0].width, Pixels(120.0));
        assert_eq!(rects[0].height, Pixels(80.0));

        // Top edge: x=140, y=60, w=120, h=4
        assert_eq!(rects[1].x, Pixels(140.0));
        assert_eq!(rects[1].y, Pixels(60.0));
        assert_eq!(rects[1].width, Pixels(120.0));
        assert_eq!(rects[1].height, Pixels(4.0));

        // Bottom edge: x=140, y=60+80-4=136, w=120, h=4
        assert_eq!(rects[2].x, Pixels(140.0));
        assert_eq!(rects[2].y, Pixels(136.0));
        assert_eq!(rects[2].width, Pixels(120.0));
        assert_eq!(rects[2].height, Pixels(4.0));

        // Left edge: x=140, y=60+4=64, w=4, h=80-2*4=72
        assert_eq!(rects[3].x, Pixels(140.0));
        assert_eq!(rects[3].y, Pixels(64.0));
        assert_eq!(rects[3].width, Pixels(4.0));
        assert_eq!(rects[3].height, Pixels(72.0));

        // Right edge: x=140+120-4=256, y=64, w=4, h=72
        assert_eq!(rects[4].x, Pixels(256.0));
        assert_eq!(rects[4].y, Pixels(64.0));
        assert_eq!(rects[4].width, Pixels(4.0));
        assert_eq!(rects[4].height, Pixels(72.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When panel with border, then background plus border rects drawn</summary>

<code>crates\engine_ui\src\panel.rs:145</code>

```rust
        // Arrange
        let (mut world, mut schedule, log, _) = setup_world_with_spy();
        world.spawn((
            Panel {
                border_color: Some(Color::RED),
                border_width: 4.0,
            },
            UiNode {
                size: Vec2::new(200.0, 150.0),
                anchor: Anchor::TopLeft,
                background: Some(Color::WHITE),
                ..UiNode::default()
            },
            GlobalTransform2D(Affine2::IDENTITY),
        ));

        // Act
        schedule.run(&mut world);

        // Assert
        let calls = log.lock().unwrap();
        // 1 background + 4 border edges
        assert_eq!(calls.iter().filter(|c| *c == "draw_rect").count(), 5);
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test progress_bar</strong> (7 tests)</summary>

<blockquote>
<details>
<summary>✅ When progress bar at full, then filled rect matches node width</summary>

<code>crates\engine_ui\src\progress_bar.rs:166</code>

```rust
        // Arrange
        let (mut world, mut schedule, _, rects) = setup_world_with_spy();
        world.spawn((
            ProgressBar {
                value: 100.0,
                max: 100.0,
            },
            UiNode {
                size: Vec2::new(200.0, 20.0),
                anchor: Anchor::TopLeft,
                background: Some(Color::from_u8(50, 50, 50, 255)),
                ..UiNode::default()
            },
            GlobalTransform2D(Affine2::IDENTITY),
        ));

        // Act
        schedule.run(&mut world);

        // Assert
        let rects = rects.lock().unwrap();
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[1].width, Pixels(200.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When progress bar at half, then filled rect is half width</summary>

<code>crates\engine_ui\src\progress_bar.rs:139</code>

```rust
        // Arrange
        let (mut world, mut schedule, _, rects) = setup_world_with_spy();
        world.spawn((
            ProgressBar {
                value: 50.0,
                max: 100.0,
            },
            UiNode {
                size: Vec2::new(200.0, 20.0),
                anchor: Anchor::TopLeft,
                background: Some(Color::from_u8(50, 50, 50, 255)),
                ..UiNode::default()
            },
            GlobalTransform2D(Affine2::IDENTITY),
        ));

        // Act
        schedule.run(&mut world);

        // Assert
        let rects = rects.lock().unwrap();
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[1].width, Pixels(100.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When progress bar exceeds max, then filled rect capped at node width</summary>

<code>crates\engine_ui\src\progress_bar.rs:193</code>

```rust
        // Arrange
        let (mut world, mut schedule, _, rects) = setup_world_with_spy();
        world.spawn((
            ProgressBar {
                value: 150.0,
                max: 100.0,
            },
            UiNode {
                size: Vec2::new(200.0, 20.0),
                anchor: Anchor::TopLeft,
                background: Some(Color::from_u8(50, 50, 50, 255)),
                ..UiNode::default()
            },
            GlobalTransform2D(Affine2::IDENTITY),
        ));

        // Act
        schedule.run(&mut world);

        // Assert
        let rects = rects.lock().unwrap();
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[1].width, Pixels(200.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When progress bar roundtrip ron, then value and max preserved</summary>

<code>crates\engine_ui\src\progress_bar.rs:97</code>

```rust
        // Arrange
        let bar = ProgressBar {
            value: 37.5,
            max: 200.0,
        };

        // Act
        let ron_str = ron::to_string(&bar).unwrap();
        let restored: ProgressBar = ron::from_str(&ron_str).unwrap();

        // Assert
        assert_eq!(restored, bar);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When progress bar at zero, then only background drawn</summary>

<code>crates\engine_ui\src\progress_bar.rs:113</code>

```rust
        // Arrange
        let (mut world, mut schedule, log, _) = setup_world_with_spy();
        world.spawn((
            ProgressBar {
                value: 0.0,
                max: 100.0,
            },
            UiNode {
                size: Vec2::new(200.0, 20.0),
                anchor: Anchor::TopLeft,
                background: Some(Color::from_u8(50, 50, 50, 255)),
                ..UiNode::default()
            },
            GlobalTransform2D(Affine2::IDENTITY),
        ));

        // Act
        schedule.run(&mut world);

        // Assert
        let calls = log.lock().unwrap();
        assert_eq!(calls.iter().filter(|c| *c == "draw_rect").count(), 1);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When progress bar invisible, then no draw</summary>

<code>crates\engine_ui\src\progress_bar.rs:251</code>

```rust
        // Arrange
        let (mut world, mut schedule, log, _) = setup_world_with_spy();
        world.spawn((
            ProgressBar {
                value: 50.0,
                max: 100.0,
            },
            UiNode {
                size: Vec2::new(200.0, 20.0),
                background: Some(Color::WHITE),
                ..UiNode::default()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            EffectiveVisibility(false),
        ));

        // Act
        schedule.run(&mut world);

        // Assert
        let calls = log.lock().unwrap();
        assert_eq!(calls.iter().filter(|c| *c == "draw_rect").count(), 0);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When progress bar with center anchor, then draw rect offset applied</summary>

<code>crates\engine_ui\src\progress_bar.rs:220</code>

```rust
        // Arrange
        let (mut world, mut schedule, _, rects) = setup_world_with_spy();
        world.spawn((
            ProgressBar {
                value: 50.0,
                max: 100.0,
            },
            UiNode {
                size: Vec2::new(200.0, 20.0),
                anchor: Anchor::Center,
                background: Some(Color::from_u8(50, 50, 50, 255)),
                ..UiNode::default()
            },
            GlobalTransform2D(Affine2::from_translation(Vec2::new(300.0, 100.0))),
        ));

        // Act
        schedule.run(&mut world);

        // Assert — Center anchor offset = (-100, -10), so top_left = (200, 90)
        let rects = rects.lock().unwrap();
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0].x, Pixels(200.0));
        assert_eq!(rects[0].y, Pixels(90.0));
        assert_eq!(rects[1].x, Pixels(200.0));
        assert_eq!(rects[1].y, Pixels(90.0));
        assert_eq!(rects[1].width, Pixels(100.0));
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test render</strong> (8 tests)</summary>

<blockquote>
<details>
<summary>✅ When effective visibility false, then no draw</summary>

<code>crates\engine_ui\src\render.rs:195</code>

```rust
        // Arrange
        let (mut world, mut schedule, log, _) = setup_world_with_spy();
        world.spawn((
            UiNode {
                size: Vec2::new(100.0, 50.0),
                background: Some(Color::WHITE),
                ..UiNode::default()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            EffectiveVisibility(false),
        ));

        // Act
        schedule.run(&mut world);

        // Assert
        let calls = log.lock().unwrap();
        assert_eq!(calls.iter().filter(|c| *c == "draw_rect").count(), 0);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When two ui nodes, then both drawn</summary>

<code>crates\engine_ui\src\render.rs:217</code>

```rust
        // Arrange
        let (mut world, mut schedule, log, _) = setup_world_with_spy();
        for _ in 0..2 {
            world.spawn((
                UiNode {
                    size: Vec2::new(50.0, 30.0),
                    background: Some(Color::WHITE),
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::IDENTITY),
            ));
        }

        // Act
        schedule.run(&mut world);

        // Assert
        let calls = log.lock().unwrap();
        assert_eq!(calls.iter().filter(|c| *c == "draw_rect").count(), 2);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When center anchor, then rect adjusted by half size</summary>

<code>crates\engine_ui\src\render.rs:125</code>

```rust
        // Arrange
        let (mut world, mut schedule, _, rects) = setup_world_with_spy();
        world.spawn((
            UiNode {
                size: Vec2::new(100.0, 60.0),
                anchor: Anchor::Center,
                background: Some(Color::BLUE),
                ..UiNode::default()
            },
            GlobalTransform2D(Affine2::from_translation(Vec2::new(300.0, 200.0))),
        ));

        // Act
        schedule.run(&mut world);

        // Assert
        let rects = rects.lock().unwrap();
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].x, Pixels(250.0));
        assert_eq!(rects[0].y, Pixels(170.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When top left anchor, then rect at transform position</summary>

<code>crates\engine_ui\src\render.rs:101</code>

```rust
        // Arrange
        let (mut world, mut schedule, _, rects) = setup_world_with_spy();
        world.spawn((
            UiNode {
                size: Vec2::new(80.0, 40.0),
                anchor: Anchor::TopLeft,
                background: Some(Color::RED),
                ..UiNode::default()
            },
            GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 150.0))),
        ));

        // Act
        schedule.run(&mut world);

        // Assert
        let rects = rects.lock().unwrap();
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].x, Pixels(200.0));
        assert_eq!(rects[0].y, Pixels(150.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When ui node rendered, then rect size matches node</summary>

<code>crates\engine_ui\src\render.rs:149</code>

```rust
        // Arrange
        let (mut world, mut schedule, _, rects) = setup_world_with_spy();
        world.spawn((
            UiNode {
                size: Vec2::new(120.0, 80.0),
                background: Some(Color::WHITE),
                ..UiNode::default()
            },
            GlobalTransform2D(Affine2::IDENTITY),
        ));

        // Act
        schedule.run(&mut world);

        // Assert
        let rects = rects.lock().unwrap();
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].width, Pixels(120.0));
        assert_eq!(rects[0].height, Pixels(80.0));
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When ui node rendered, then rect color matches background</summary>

<code>crates\engine_ui\src\render.rs:172</code>

```rust
        // Arrange
        let (mut world, mut schedule, _, rects) = setup_world_with_spy();
        let color = Color::new(1.0, 0.0, 0.5, 1.0);
        world.spawn((
            UiNode {
                size: Vec2::new(50.0, 50.0),
                background: Some(color),
                ..UiNode::default()
            },
            GlobalTransform2D(Affine2::IDENTITY),
        ));

        // Act
        schedule.run(&mut world);

        // Assert
        let rects = rects.lock().unwrap();
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].color, color);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When ui node has background, then draw rect called</summary>

<code>crates\engine_ui\src\render.rs:60</code>

```rust
        // Arrange
        let (mut world, mut schedule, log, _) = setup_world_with_spy();
        world.spawn((
            UiNode {
                size: Vec2::new(100.0, 50.0),
                background: Some(Color::WHITE),
                ..UiNode::default()
            },
            GlobalTransform2D(Affine2::IDENTITY),
        ));

        // Act
        schedule.run(&mut world);

        // Assert
        let calls = log.lock().unwrap();
        assert_eq!(calls.iter().filter(|c| *c == "draw_rect").count(), 1);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When ui node no background, then no draw</summary>

<code>crates\engine_ui\src\render.rs:81</code>

```rust
        // Arrange
        let (mut world, mut schedule, log, _) = setup_world_with_spy();
        world.spawn((
            UiNode {
                background: None,
                ..UiNode::default()
            },
            GlobalTransform2D(Affine2::IDENTITY),
        ));

        // Act
        schedule.run(&mut world);

        // Assert
        let calls = log.lock().unwrap();
        assert_eq!(calls.iter().filter(|c| *c == "draw_rect").count(), 0);
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test theme</strong> (1 tests)</summary>

<blockquote>
<details>
<summary>✅ When ui theme roundtrip ron, then all fields preserved</summary>

<code>crates\engine_ui\src\theme.rs:34</code>

```rust
        // Arrange
        let theme = UiTheme {
            normal_color: Color::from_u8(10, 20, 30, 255),
            hovered_color: Color::from_u8(40, 50, 60, 255),
            pressed_color: Color::from_u8(70, 80, 90, 255),
            disabled_color: Color::from_u8(100, 110, 120, 128),
            text_color: Color::from_u8(200, 210, 220, 255),
            font_size: 24.0,
        };

        // Act
        let ron_str = ron::to_string(&theme).unwrap();
        let restored: UiTheme = ron::from_str(&ron_str).unwrap();

        // Assert
        assert_eq!(restored, theme);
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test ui_event</strong> (2 tests)</summary>

<blockquote>
<details>
<summary>✅ When clicked event pushed, then drain yields exact event and buffer is empty</summary>

<code>crates\engine_ui\src\ui_event.rs:37</code>

```rust
        // Arrange
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let mut buffer = UiEventBuffer::default();
        buffer.push(UiEvent::Clicked(entity));

        // Act
        let drained: Vec<UiEvent> = buffer.drain().collect();

        // Assert
        assert_eq!(drained, vec![UiEvent::Clicked(entity)]);
        assert_eq!(buffer.drain().count(), 0);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When drained twice, then second drain is empty</summary>

<code>crates\engine_ui\src\ui_event.rs:53</code>

```rust
        // Arrange
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let mut buffer = UiEventBuffer::default();
        buffer.push(UiEvent::HoverEnter(entity));
        let _ = buffer.drain().count();

        // Act
        let second: Vec<UiEvent> = buffer.drain().collect();

        // Assert
        assert!(second.is_empty());
```

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>test ui_node</strong> (3 tests)</summary>

<blockquote>
<details>
<summary>✅ When flex layout roundtrip ron, then preserved</summary>

<code>crates\engine_ui\src\ui_node.rs:59</code>

```rust
        // Arrange
        let layout = FlexLayout {
            direction: FlexDirection::Column,
            gap: 12.5,
        };

        // Act
        let ron_str = ron::to_string(&layout).unwrap();
        let restored: FlexLayout = ron::from_str(&ron_str).unwrap();

        // Assert
        assert_eq!(restored, layout);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When text roundtrip ron, then preserved</summary>

<code>crates\engine_ui\src\ui_node.rs:75</code>

```rust
        // Arrange
        let text = Text {
            content: "Hello UI".into(),
            font_size: 24.0,
            color: Color::new(0.2, 0.8, 0.4, 1.0),
        };

        // Act
        let ron_str = ron::to_string(&text).unwrap();
        let restored: Text = ron::from_str(&ron_str).unwrap();

        // Assert
        assert_eq!(restored, text);
```

</details>
</blockquote>
<blockquote>
<details>
<summary>✅ When ui node roundtrip ron, then preserved</summary>

<code>crates\engine_ui\src\ui_node.rs:36</code>

```rust
        // Arrange
        let node = UiNode {
            size: Vec2::new(200.0, 100.0),
            anchor: Anchor::Center,
            margin: Margin {
                top: 5.0,
                right: 10.0,
                bottom: 15.0,
                left: 20.0,
            },
            background: Some(Color::new(1.0, 0.0, 0.5, 0.8)),
        };

        // Act
        let ron_str = ron::to_string(&node).unwrap();
        let restored: UiNode = ron::from_str(&ron_str).unwrap();

        // Assert
        assert_eq!(restored, node);
```

</details>
</blockquote>

</details>
</blockquote>

</details>

<details>
<summary><strong>living_docs</strong> (1 tests)</summary>

<blockquote>
<details>
<summary><strong>doc_tests</strong> (1 tests)</summary>

- Handle::Handle (line 6) - compile fail

</details>
</blockquote>

</details>

