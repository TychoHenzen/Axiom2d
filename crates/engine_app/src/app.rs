use std::sync::Arc;
use std::time::{Duration, Instant};

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowId};

use engine_input::button_state::ButtonState;
use engine_input::key_code::KeyCode;
use engine_input::mouse_button::MouseButton;

use engine_core::prelude::EventBus;
use engine_core::profiler::FrameProfiler;
use engine_core::types::Pixels;
use engine_ecs::prelude::{
    IntoScheduleConfigs, PHASE_COUNT, Phase, Schedule, ScheduleSystem, World,
};
use engine_input::prelude::{KeyInputEvent, MouseInputEvent, MouseState};
use engine_render::prelude::RendererRes;
use engine_render::window::WindowConfig;

use crate::window_size::WindowSize;

pub trait Plugin {
    fn build(&self, app: &mut App);
}

pub struct App {
    plugin_count: usize,
    window_config: WindowConfig,
    window: Option<Arc<Window>>,
    world: World,
    schedules: [Schedule; PHASE_COUNT],
}

impl App {
    pub fn new() -> Self {
        let mut world = World::new();
        world.insert_resource(WindowSize::default());
        world.insert_resource(engine_core::time::DeltaTime::default());
        Self {
            plugin_count: 0,
            window_config: WindowConfig::default(),
            window: None,
            world,
            schedules: Phase::ALL.map(Schedule::new),
        }
    }

    pub fn set_window_config(&mut self, config: WindowConfig) -> &mut Self {
        self.update_window_size(config.width, config.height);
        self.window_config = config;
        self
    }

    pub fn add_plugin(&mut self, plugin: impl Plugin) -> &mut Self {
        plugin.build(self);
        self.plugin_count += 1;
        self
    }

    pub fn plugin_count(&self) -> usize {
        self.plugin_count
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    pub fn schedule_count(&self) -> usize {
        PHASE_COUNT
    }

    pub fn add_systems<M>(
        &mut self,
        phase: Phase,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self {
        self.schedules[phase.index()].add_systems(systems);
        self
    }

    #[doc(hidden)]
    pub fn set_renderer(
        &mut self,
        renderer: Box<dyn engine_render::renderer::Renderer + Send + Sync>,
    ) {
        self.world.insert_resource(RendererRes::new(renderer));
    }

    #[doc(hidden)]
    pub fn handle_key_event(
        &mut self,
        physical_key: PhysicalKey,
        state: winit::event::ElementState,
        is_synthetic: bool,
    ) {
        if is_synthetic {
            return;
        }

        if let PhysicalKey::Code(winit_key) = physical_key
            && let Some(mut bus) = self.world.get_resource_mut::<EventBus<KeyInputEvent>>()
        {
            bus.push(KeyInputEvent {
                key: KeyCode::from(winit_key),
                state: ButtonState::from(state),
            });
        }
    }

    #[doc(hidden)]
    pub fn handle_cursor_moved(&mut self, position: glam::Vec2) {
        if let Some(mut mouse) = self.world.get_resource_mut::<MouseState>() {
            mouse.set_screen_pos(position);
        }
    }

    #[doc(hidden)]
    pub fn handle_mouse_button(
        &mut self,
        button: winit::event::MouseButton,
        state: winit::event::ElementState,
    ) {
        if let Some(mut bus) = self.world.get_resource_mut::<EventBus<MouseInputEvent>>() {
            bus.push(MouseInputEvent {
                button: MouseButton::from(button),
                state: ButtonState::from(state),
            });
        }
    }

    #[doc(hidden)]
    pub fn handle_mouse_wheel(&mut self, delta: glam::Vec2) {
        if let Some(mut mouse) = self.world.get_resource_mut::<MouseState>() {
            mouse.add_scroll_delta(delta);
        }
    }

    fn update_window_size(&mut self, width: u32, height: u32) {
        self.world.insert_resource(WindowSize {
            width: Pixels(width as f32),
            height: Pixels(height as f32),
        });
    }

    #[doc(hidden)]
    pub fn handle_resize(&mut self, width: u32, height: u32) {
        self.update_window_size(width, height);
        if let Some(mut renderer) = self.world.get_resource_mut::<RendererRes>() {
            renderer.resize(width, height);
        }
    }

    pub fn handle_redraw(&mut self) {
        let frame_start = Instant::now();
        for (i, schedule) in self.schedules.iter_mut().enumerate() {
            let start = Instant::now();
            schedule.run(&mut self.world);
            let elapsed_us = start.elapsed().as_micros() as u64;
            if let Some(mut profiler) = self.world.get_resource_mut::<FrameProfiler>() {
                profiler.record_phase(Phase::ALL[i].name(), elapsed_us);
            }
        }
        if let Some(mut renderer) = self.world.get_resource_mut::<RendererRes>() {
            renderer.present();
        }
        if let Some(mut profiler) = self.world.get_resource_mut::<FrameProfiler>() {
            profiler.end_frame();
        }
        if let Some(window) = &self.window {
            window.set_title(&format_window_title(
                self.window_config.title,
                frame_start.elapsed(),
            ));
        }
    }

    pub fn run(&mut self) {
        // INVARIANT: EventLoop::new() only fails if the OS windowing system is
        // unavailable (e.g. no display server). No recovery is possible.
        let event_loop = EventLoop::new().expect("failed to create event loop");
        // INVARIANT: run_app() only fails on OS-level event loop errors.
        // The game cannot continue without an event loop.
        event_loop
            .run_app(self)
            .expect("event loop exited with error");
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = Window::default_attributes()
            .with_title(self.window_config.title)
            .with_inner_size(winit::dpi::LogicalSize::new(
                f64::from(self.window_config.width),
                f64::from(self.window_config.height),
            ))
            .with_resizable(self.window_config.resizable)
            .with_visible(false);
        // INVARIANT: create_window() only fails if the OS cannot allocate a
        // window (out of resources, no display). No recovery is possible.
        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .expect("failed to create window"),
        );
        let renderer = engine_render::create_renderer(window.clone(), &self.window_config);

        self.world.insert_resource(RendererRes::new(renderer));
        self.window = Some(window.clone());

        // Reset the clock so the first DeltaTime is near-zero, not the
        // seconds spent on GPU initialization.
        self.world
            .insert_resource(engine_core::prelude::ClockRes::new(Box::new(
                engine_core::time::SystemClock::default(),
            )));

        // Render a few frames while the window is still hidden to ensure
        // the GPU has fully presented before the window becomes visible.
        // This avoids the white flash from the OS compositor.
        for _ in 0..3 {
            self.handle_redraw();
        }
        window.set_visible(true);
        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => self.handle_resize(size.width, size.height),
            WindowEvent::RedrawRequested => self.handle_redraw(),
            WindowEvent::KeyboardInput {
                event,
                is_synthetic,
                ..
            } => {
                self.handle_key_event(event.physical_key, event.state, is_synthetic);
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.handle_cursor_moved(glam::Vec2::new(position.x as f32, position.y as f32));
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.handle_mouse_button(button, state);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll = match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => glam::Vec2::new(x, y),
                    winit::event::MouseScrollDelta::PixelDelta(pos) => {
                        glam::Vec2::new(pos.x as f32, pos.y as f32)
                    }
                };
                self.handle_mouse_wheel(scroll);
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

pub fn format_window_title(base_title: &str, frame_time: Duration) -> String {
    let fps = if frame_time.is_zero() {
        0.0
    } else {
        1.0 / frame_time.as_secs_f64()
    };
    format!("{base_title} - {fps:.0} FPS")
}
