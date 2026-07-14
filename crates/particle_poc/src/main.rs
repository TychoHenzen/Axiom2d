#![allow(clippy::unwrap_used)]

use std::sync::Arc;

use particle_poc::{State, WINDOW_HEIGHT, WINDOW_WIDTH, parse_flag_arg};
use winit::application::ApplicationHandler;
use winit::event::KeyEvent;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

fn main() {
    let benchmark = std::env::args().any(|a| a == "--benchmark");
    let diagnose = std::env::args().any(|a| a == "--diagnose");
    let no_benchmark = std::env::args().any(|a| a == "--no-benchmark");
    let test_bond_form = std::env::args().any(|a| a == "--test-bond-form");
    let test_bond_constrain = std::env::args().any(|a| a == "--test-bond-constrain");
    let test_bond_break = std::env::args().any(|a| a == "--test-bond-break");
    let test_paddle_stability = std::env::args().any(|a| a == "--test-paddle-stability");
    let test_paddle_root_cause = std::env::args().any(|a| a == "--test-paddle-root-cause");
    let test_sdf_bowl = std::env::args().any(|a| a == "--test-sdf-bowl");
    let num_particles = parse_flag_arg("--particles", particle_poc::MAX_PARTICLES);
    let sub_steps = parse_flag_arg("--substeps", particle_poc::SUB_STEPS);
    let event_loop = EventLoop::new().expect("event loop");
    let mut app = App {
        state: None,
        benchmark,
        diagnose,
        no_benchmark,
        test_bond_form,
        test_bond_constrain,
        test_bond_break,
        test_paddle_stability,
        test_paddle_root_cause,
        test_sdf_bowl,
        num_particles,
        sub_steps,
    };
    event_loop.run_app(&mut app).expect("run");
}

#[allow(clippy::struct_excessive_bools)]
struct App {
    state: Option<State>,
    benchmark: bool,
    diagnose: bool,
    no_benchmark: bool,
    test_bond_form: bool,
    test_bond_constrain: bool,
    test_bond_break: bool,
    test_paddle_stability: bool,
    test_paddle_root_cause: bool,
    test_sdf_bowl: bool,
    num_particles: u32,
    sub_steps: u32,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Particle Idle PoC")
                        .with_inner_size(winit::dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT)),
                )
                .expect("window"),
        );
        let mut state = State::new(
            window.clone(),
            self.benchmark,
            self.diagnose,
            self.test_bond_form,
            self.test_bond_constrain,
            self.test_bond_break,
            self.test_paddle_stability,
            self.test_paddle_root_cause,
            self.test_sdf_bowl,
            self.num_particles,
            self.sub_steps,
        );
        state.no_benchmark = self.no_benchmark;
        self.state = Some(state);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(state) = &mut self.state else {
            return;
        };
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size),
            WindowEvent::RedrawRequested => {
                state.render();
                if state.bench_done {
                    event_loop.exit();
                } else {
                    state.window.request_redraw();
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        logical_key: key,
                        ..
                    },
                ..
            } => {
                if key == Key::Named(NamedKey::Tab) {
                    state.toggle_mode();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                state.mouse_move(position.x as f32, position.y as f32);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let dy = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => y,
                    winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 100.0,
                };
                state.scroll(dy);
            }
            WindowEvent::MouseInput {
                state: button_state,
                button,
                ..
            } => {
                let pressed = button_state == ElementState::Pressed;
                let secondary = button == MouseButton::Right;
                state.mouse_button(pressed, secondary);
            }
            _ => {}
        }
    }
}
