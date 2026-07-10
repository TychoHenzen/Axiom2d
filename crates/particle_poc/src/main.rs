#![allow(clippy::unwrap_used)]

use std::sync::Arc;

use particle_poc::{MAX_PARTICLES, SUB_STEPS, State, WINDOW_HEIGHT, WINDOW_WIDTH, parse_flag_arg};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

fn main() {
    let benchmark = std::env::args().any(|a| a == "--benchmark");
    let diagnose = std::env::args().any(|a| a == "--diagnose");
    let test_bond_form = std::env::args().any(|a| a == "--test-bond-form");
    let test_bond_constrain = std::env::args().any(|a| a == "--test-bond-constrain");
    let test_bond_break = std::env::args().any(|a| a == "--test-bond-break");
    let test_paddle_stability = std::env::args().any(|a| a == "--test-paddle-stability");
    let test_paddle_root_cause = std::env::args().any(|a| a == "--test-paddle-root-cause");
    let num_particles = parse_flag_arg("--particles", MAX_PARTICLES);
    let sub_steps = parse_flag_arg("--substeps", SUB_STEPS);
    let event_loop = EventLoop::new().expect("event loop");
    let mut app = App {
        state: None,
        benchmark,
        diagnose,
        test_bond_form,
        test_bond_constrain,
        test_bond_break,
        test_paddle_stability,
        test_paddle_root_cause,
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
    test_bond_form: bool,
    test_bond_constrain: bool,
    test_bond_break: bool,
    test_paddle_stability: bool,
    test_paddle_root_cause: bool,
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
        self.state = Some(State::new(
            window,
            self.benchmark,
            self.diagnose,
            self.test_bond_form,
            self.test_bond_constrain,
            self.test_bond_break,
            self.test_paddle_stability,
            self.test_paddle_root_cause,
            self.num_particles,
            self.sub_steps,
        ));
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
            _ => {}
        }
    }
}
