#![allow(clippy::unwrap_used)]

use std::sync::Arc;
use std::time::Instant;

use bytemuck::{Pod, Zeroable};
use rapier2d::prelude::*;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;
const MAX_PARTICLES: u32 = 100000;
const WORKGROUP_SIZE: u32 = 256;
const SPAWN_RATE: u32 = 130;
// Two hoppers: Red on left, Blue on right. Particles meet in center where mixer spins.
const HOPPER_LEFT_X: f32 = -0.45;
const HOPPER_LEFT_HALF: f32 = 0.2;
const HOPPER_RIGHT_X: f32 = 0.45;
const HOPPER_RIGHT_HALF: f32 = 0.2;
const HOPPER_Y: f32 = 0.75;
const GRID_W: u32 = 256;
const GRID_H: u32 = 256;
const TOTAL_GRID_CELLS: u32 = GRID_W * GRID_H;
// 8 substeps of dt=1/480 = exactly 1/60s per frame.
const SUB_STEPS: u32 = 16;

const MAX_SPECIES: u32 = 8;
const MAX_MACHINES: u32 = 16;
const PADDLE_COUNT: u32 = 10;
const CAPSULE_HALF_LEN: f32 = 0.22;
const CAPSULE_RADIUS: f32 = 0.055;
const CONVEYOR_ANGLE_DEG: f32 = 45.0;
const PADDLE_HW: f32 = 0.012;
const PADDLE_HH: f32 = 0.035;
const CONVEYOR_SPEED: f32 = 0.45;
const SENSOR_HALF: f32 = 0.06;

// Polymer bond constants for Green(2) particles.
// Green particles form softbody-style mesh bonds (max 4 per particle) that break under stress.
// Mirror copy of WGSL constants in form_bonds.wgsl and solve_bonds.wgsl.
#[allow(dead_code)]
const GREEN_SPECIES: u32 = 2;
const INVALID_BOND: u32 = 0xFFFF_FFFF;
#[allow(dead_code)]
const MAX_BONDS_PER_PARTICLE: u32 = 4;
#[allow(dead_code)]
const BOND_FORMATION_MULTIPLIER: f32 = 3.0;
#[allow(dead_code)]
const BOND_BREAK_MULTIPLIER: f32 = 5.0;
#[allow(dead_code)]
const BOND_COMPLIANCE: f32 = 0.04;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct BondSlot {
    partner: u32,
    rest: f32,
}

impl Default for BondSlot {
    fn default() -> Self {
        Self {
            partner: INVALID_BOND,
            rest: 0.0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct ReactionMatrix {
    // Dense 8×8 flat array: result[species_A * MAX_SPECIES + species_B].
    // Row/col 0 = Red, 1 = Blue, 2 = Green, 3-7 = reserved.
    results: [u32; (MAX_SPECIES * MAX_SPECIES) as usize],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub(crate) struct SimParams {
    particle_count: u32,
    dt: f32,
    gravity: f32,
    particle_radius: f32,
    wall_min_x: f32,
    wall_min_y: f32,
    wall_max_x: f32,
    wall_max_y: f32,
    friction_mu: f32,
    grid_cell_size: f32,
    grid_width: u32,
    grid_height: u32,
    disable_velocity_cap: u32,
    sub_steps: u32,
}

#[repr(u32)]
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub(crate) enum MachineKind {
    Conveyor = 0,
    Grinder = 1,
    Heater = 2,
    Paddle = 3,
}

// GPU-compatible machine uniform (must match WGSL Machine struct layout).
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct GpuMachine {
    pos_x: f32,
    pos_y: f32,
    cos_angle: f32,
    sin_angle: f32,
    half_width: f32,
    half_height: f32,
    kind: u32,
    input_species: u32,
    output_species: u32,
    angular_velocity: f32,
}

impl Default for GpuMachine {
    fn default() -> Self {
        Self {
            pos_x: 0.0,
            pos_y: 0.0,
            cos_angle: 1.0,
            sin_angle: 0.0,
            half_width: 0.0,
            half_height: 0.0,
            kind: 0,
            input_species: 0,
            output_species: 0,
            angular_velocity: 0.0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct MachineParams {
    count: u32,
    _pad: [u32; 3],
    machines: [GpuMachine; MAX_MACHINES as usize],
}

impl Default for MachineParams {
    fn default() -> Self {
        Self {
            count: 0,
            _pad: [0; 3],
            machines: [GpuMachine::default(); MAX_MACHINES as usize],
        }
    }
}

// Recipe: what a machine consumes and produces per cycle.
#[allow(dead_code)]
struct Recipe {
    input_species: u32,
    input_count: u32,
    output_species: u32,
    output_count: u32,
    cycle_time: f32,
}

// CPU-side machine state — buffer tracking, cycle progression, brightness.
struct MachineCpuState {
    recipe: Recipe,
    input_accumulated: u32,
    cycles_completed: u32,
    cycle_timer: f32,
    consumed_this_frame: u32,
    color_base: [f32; 3],
}

// Machine definition on CPU side (Rapier2D handles for physics).
struct MachineDef {
    kind: MachineKind,
    body_handle: RigidBodyHandle,
    input_species: u32,
    output_species: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct RenderParams {
    screen_width: f32,
    screen_height: f32,
    particle_radius: f32,
    particle_count: u32,
}

// GPU-compatible machine render data (must match WGSL MachineRender struct).
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct GpuMachineRender {
    pos_x: f32,
    pos_y: f32,
    cos_angle: f32,
    sin_angle: f32,
    half_width: f32,
    half_height: f32,
    color_r: f32,
    color_g: f32,
    color_b: f32,
}

impl Default for GpuMachineRender {
    fn default() -> Self {
        Self {
            pos_x: 0.0,
            pos_y: 0.0,
            cos_angle: 1.0,
            sin_angle: 0.0,
            half_width: 0.0,
            half_height: 0.0,
            color_r: 1.0,
            color_g: 1.0,
            color_b: 1.0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct MachineRenderParams {
    screen_width: f32,
    screen_height: f32,
    machine_count: u32,
}

fn parse_flag_arg(name: &str, default: u32) -> u32 {
    let mut args = std::env::args();
    while let Some(arg) = args.next() {
        if arg == name
            && let Some(val) = args.next()
            && let Ok(n) = val.parse::<u32>()
        {
            return n;
        }
    }
    default
}

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

const MAX_OUTLIERS: u32 = 64;
const MAX_PHASING: u32 = 32;

pub(crate) struct Buffers {
    positions: wgpu::Buffer,
    velocities: wgpu::Buffer,
    forces: wgpu::Buffer,
    prev_positions: wgpu::Buffer,
    species: wgpu::Buffer,
    params: wgpu::Buffer,
    cell_indices: wgpu::Buffer,
    cell_counts: wgpu::Buffer,
    cell_offsets: wgpu::Buffer,
    sorted_indices: wgpu::Buffer,
    morton_keys: wgpu::Buffer,
    reaction_matrix: wgpu::Buffer,
    // Polymer bond data: flat array, MAX_PARTICLES * 4 slots.
    // Particle i owns slots [i*4+0, i*4+1, i*4+2, i*4+3].
    bonds: wgpu::Buffer,
    // Per-machine atomic counter: GPU-side consumption tracking.
    machine_counters: wgpu::Buffer,
    // GPU outlier/phasing detection buffers.
    outlier_buf: wgpu::Buffer,
    outlier_count_buf: wgpu::Buffer,
    phasing_buf: wgpu::Buffer,
    phasing_count_buf: wgpu::Buffer,
}

pub(crate) struct ComputePipelines {
    predict: wgpu::ComputePipeline,
    clear_cells: wgpu::ComputePipeline,
    prefix_scan: wgpu::ComputePipeline,
    project: wgpu::ComputePipeline,
    apply: wgpu::ComputePipeline,
    reaction: wgpu::ComputePipeline,
    morton_keys: wgpu::ComputePipeline,
    morton_count: wgpu::ComputePipeline,
    morton_scatter: wgpu::ComputePipeline,
    form_bonds: wgpu::ComputePipeline,
    form_bonds_resolve: wgpu::ComputePipeline,
    solve_bonds: wgpu::ComputePipeline,
    detect_outliers: wgpu::ComputePipeline,
    detect_phasing: wgpu::ComputePipeline,
}

pub(crate) struct MachineRenderState {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    params_buf: wgpu::Buffer,
    data_buf: wgpu::Buffer,
}

pub(crate) struct RenderState {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    params_buf: wgpu::Buffer,
    machine: MachineRenderState,
}

struct RapierState {
    pipeline: PhysicsPipeline,
    integration_parameters: IntegrationParameters,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    bodies: RigidBodySet,
    colliders: ColliderSet,
    impulse_joints: ImpulseJointSet,
    multibody_joints: MultibodyJointSet,
    ccd_solver: CCDSolver,
}

mod state;

#[allow(clippy::struct_excessive_bools)]
struct State {
    window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    buffers: Buffers,
    particle_bg: wgpu::BindGroup,
    grid_bg: wgpu::BindGroup,
    pipelines: ComputePipelines,
    render: RenderState,
    detection_bg: wgpu::BindGroup,
    sim_params: SimParams,
    fps_tracker: FpsTracker,
    benchmark: bool,
    bench_frame: u32,
    bench_start: Option<Instant>,
    bench_done: bool,
    test_bond_form: bool,
    test_bond_constrain: bool,
    test_bond_break: bool,
    test_paddle_stability: bool,
    test_paddle_root_cause: bool,
    test_phase: u32,
    test_report_done: bool,
    diagnose: bool,
    diag_frame: u32,
    diag_staging: Option<wgpu::Buffer>,
    rapier: RapierState,
    machines: Vec<MachineDef>,
    machines_cpu: Vec<MachineCpuState>,
    machine_params_buf: wgpu::Buffer,
    counters_staging: wgpu::Buffer,
    machine_time: f32,
    num_particles: u32,
    sub_steps: u32,
    max_speed_seen: f32,
    outlier_staging: Option<wgpu::Buffer>,
    phasing_staging: Option<wgpu::Buffer>,
    outlier_count_staging: Option<wgpu::Buffer>,
    phasing_count_staging: Option<wgpu::Buffer>,
}

struct FpsTracker {
    last_fps_update: Instant,
    frame_count: u32,
    fps: f32,
    sim_time_ms: f32,
    last_frame: Instant,
}

impl FpsTracker {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            last_fps_update: now,
            frame_count: 0,
            fps: 0.0,
            sim_time_ms: 0.0,
            last_frame: now,
        }
    }

    fn begin_frame(&mut self) {
        self.last_frame = Instant::now();
    }

    fn end_frame(&mut self) -> bool {
        let frame_time = self.last_frame.elapsed();
        self.sim_time_ms = frame_time.as_secs_f32() * 1000.0;
        self.frame_count += 1;
        let elapsed = self.last_fps_update.elapsed().as_secs_f32();
        if elapsed >= 0.5 {
            self.fps = self.frame_count as f32 / elapsed;
            self.frame_count = 0;
            self.last_fps_update = Instant::now();
            return true;
        }
        false
    }
}

fn create_buffers(device: &wgpu::Device) -> Buffers {
    let particle_storage = wgpu::BufferUsages::STORAGE
        | wgpu::BufferUsages::COPY_DST
        | wgpu::BufferUsages::COPY_SRC
        | wgpu::BufferUsages::VERTEX;
    let grid_storage =
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC;
    Buffers {
        positions: device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("positions"),
            size: u64::from(MAX_PARTICLES) * 8,
            usage: particle_storage,
            mapped_at_creation: false,
        }),
        velocities: device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("velocities"),
            size: u64::from(MAX_PARTICLES) * 8,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        }),
        forces: device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("forces"),
            size: u64::from(MAX_PARTICLES) * 8,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }),
        prev_positions: device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("prev_positions"),
            size: u64::from(MAX_PARTICLES) * 8,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }),
        species: device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("species"),
            size: u64::from(MAX_PARTICLES) * 4,
            usage: particle_storage,
            mapped_at_creation: false,
        }),
        params: device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("sim_params"),
            size: size_of::<SimParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }),
        cell_indices: device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("cell_indices"),
            size: u64::from(MAX_PARTICLES) * 4,
            usage: grid_storage,
            mapped_at_creation: false,
        }),
        cell_counts: device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("cell_counts"),
            size: u64::from(TOTAL_GRID_CELLS) * 4,
            usage: grid_storage,
            mapped_at_creation: false,
        }),
        cell_offsets: device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("cell_offsets"),
            size: u64::from(TOTAL_GRID_CELLS) * 4,
            usage: grid_storage,
            mapped_at_creation: false,
        }),
        sorted_indices: device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("sorted_indices"),
            size: u64::from(MAX_PARTICLES) * 4,
            usage: grid_storage,
            mapped_at_creation: false,
        }),
        morton_keys: device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("morton_keys"),
            size: u64::from(MAX_PARTICLES) * 4,
            usage: grid_storage,
            mapped_at_creation: false,
        }),
        reaction_matrix: device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("reaction_matrix"),
            size: size_of::<ReactionMatrix>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }),
        bonds: device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("bonds"),
            size: u64::from(MAX_PARTICLES) * 4 * size_of::<BondSlot>() as u64,
            usage: grid_storage,
            mapped_at_creation: false,
        }),
        machine_counters: device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("machine_counters"),
            size: u64::from(MAX_MACHINES) * 4,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        }),
        outlier_buf: device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("outlier_buf"),
            size: 2048,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        }),
        outlier_count_buf: device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("outlier_count"),
            size: 4,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        }),
        phasing_buf: device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("phasing_buf"),
            size: 512,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        }),
        phasing_count_buf: device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("phasing_count"),
            size: 4,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        }),
    }
}

fn storage_entry(binding: u32, read_only: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn uniform_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn create_particle_bgl(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("particle_bgl"),
        entries: &[
            storage_entry(0, false),  // positions
            storage_entry(1, false),  // velocities
            storage_entry(2, false),  // species
            storage_entry(3, false),  // forces
            storage_entry(4, false),  // prev_positions
            uniform_entry(7),         // params
            storage_entry(8, true),   // machine_params (read-only storage)
            storage_entry(9, true),   // reaction_matrix (read-only storage)
            storage_entry(10, false), // bonds (flat array, 4 slots per particle)
        ],
    })
}

fn create_detection_bgl(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("detection_bgl"),
        entries: &[
            storage_entry(0, true),  // positions (read-only)
            storage_entry(1, true),  // velocities (read-only)
            storage_entry(2, false), // outlier_count (atomic<u32>)
            storage_entry(3, false), // phasing_count (atomic<u32>)
            storage_entry(4, false), // outlier_data (array<u32>)
            storage_entry(5, false), // phasing_data (array<u32>)
            uniform_entry(6),        // params
            storage_entry(7, true),  // machine_params (read-only)
        ],
    })
}

fn create_grid_bgl(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("grid_bgl"),
        entries: &[
            storage_entry(0, false), // cell_indices
            storage_entry(1, false), // cell_counts
            storage_entry(2, false), // cell_offsets
            storage_entry(3, false), // sorted_indices
            storage_entry(4, false), // morton_keys
            storage_entry(5, false), // machine_counters (atomic<u32> per machine)
        ],
    })
}

fn create_bind_groups(
    device: &wgpu::Device,
    buffers: &Buffers,
    machine_params_buf: &wgpu::Buffer,
    particle_bgl: &wgpu::BindGroupLayout,
    grid_bgl: &wgpu::BindGroupLayout,
) -> (wgpu::BindGroup, wgpu::BindGroup) {
    let particle_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("particle_bg"),
        layout: particle_bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: buffers.positions.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: buffers.velocities.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: buffers.species.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: buffers.forces.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: buffers.prev_positions.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 7,
                resource: buffers.params.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 8,
                resource: machine_params_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 9,
                resource: buffers.reaction_matrix.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 10,
                resource: buffers.bonds.as_entire_binding(),
            },
        ],
    });
    let grid_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("grid_bg"),
        layout: grid_bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: buffers.cell_indices.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: buffers.cell_counts.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: buffers.cell_offsets.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: buffers.sorted_indices.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: buffers.morton_keys.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: buffers.machine_counters.as_entire_binding(),
            },
        ],
    });
    (particle_bg, grid_bg)
}

fn make_compute_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    module: &wgpu::ShaderModule,
    entry: &str,
    label: &str,
) -> wgpu::ComputePipeline {
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(label),
        layout: Some(layout),
        module,
        entry_point: Some(entry),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    })
}

fn create_detection_bg(
    device: &wgpu::Device,
    buffers: &Buffers,
    machine_params_buf: &wgpu::Buffer,
    detection_bgl: &wgpu::BindGroupLayout,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("detection_bg"),
        layout: detection_bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: buffers.positions.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: buffers.velocities.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: buffers.outlier_count_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: buffers.phasing_count_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: buffers.outlier_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: buffers.phasing_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 6,
                resource: buffers.params.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 7,
                resource: machine_params_buf.as_entire_binding(),
            },
        ],
    })
}

fn create_pipelines(
    device: &wgpu::Device,
    particle_bgl: &wgpu::BindGroupLayout,
    grid_bgl: &wgpu::BindGroupLayout,
    detection_bgl: &wgpu::BindGroupLayout,
) -> ComputePipelines {
    let integrate_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("integrate_pl"),
        bind_group_layouts: &[particle_bgl],
        push_constant_ranges: &[],
    });
    let spatial_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("spatial_pl"),
        bind_group_layouts: &[particle_bgl, grid_bgl],
        push_constant_ranges: &[],
    });
    let detection_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("detection_pl"),
        bind_group_layouts: &[detection_bgl],
        push_constant_ranges: &[],
    });

    let integrate_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("integrate"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/integrate.wgsl").into()),
    });
    let spatial_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("spatial_hash"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/spatial_hash.wgsl").into()),
    });
    let reaction_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("reaction"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/reaction.wgsl").into()),
    });
    let project_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("project"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/project.wgsl").into()),
    });
    let morton_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("morton_sort"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/morton_sort.wgsl").into()),
    });
    let form_bonds_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("form_bonds"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/form_bonds.wgsl").into()),
    });
    let solve_bonds_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("solve_bonds"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/solve_bonds.wgsl").into()),
    });
    let detection_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("detection"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/detection.wgsl").into()),
    });

    ComputePipelines {
        predict: make_compute_pipeline(
            device,
            &integrate_layout,
            &integrate_shader,
            "main",
            "predict",
        ),
        clear_cells: make_compute_pipeline(
            device,
            &spatial_layout,
            &spatial_shader,
            "clear_cells",
            "clear_cells",
        ),
        prefix_scan: make_compute_pipeline(
            device,
            &spatial_layout,
            &spatial_shader,
            "prefix_scan",
            "prefix_scan",
        ),
        project: make_compute_pipeline(
            device,
            &spatial_layout,
            &project_shader,
            "project",
            "project",
        ),
        apply: make_compute_pipeline(device, &spatial_layout, &project_shader, "apply", "apply"),
        reaction: make_compute_pipeline(
            device,
            &spatial_layout,
            &reaction_shader,
            "react",
            "reaction",
        ),
        morton_keys: make_compute_pipeline(
            device,
            &spatial_layout,
            &morton_shader,
            "morton_keys_kernel",
            "morton_keys",
        ),
        morton_count: make_compute_pipeline(
            device,
            &spatial_layout,
            &morton_shader,
            "morton_count",
            "morton_count",
        ),
        morton_scatter: make_compute_pipeline(
            device,
            &spatial_layout,
            &morton_shader,
            "morton_scatter",
            "morton_scatter",
        ),
        form_bonds: make_compute_pipeline(
            device,
            &spatial_layout,
            &form_bonds_shader,
            "form_bonds_propose",
            "form_bonds",
        ),
        form_bonds_resolve: make_compute_pipeline(
            device,
            &spatial_layout,
            &form_bonds_shader,
            "form_bonds_resolve",
            "form_bonds_resolve",
        ),
        solve_bonds: make_compute_pipeline(
            device,
            &spatial_layout,
            &solve_bonds_shader,
            "solve_bonds",
            "solve_bonds",
        ),
        detect_outliers: make_compute_pipeline(
            device,
            &detection_layout,
            &detection_shader,
            "detect_outliers",
            "detect_outliers",
        ),
        detect_phasing: make_compute_pipeline(
            device,
            &detection_layout,
            &detection_shader,
            "detect_phasing",
            "detect_phasing",
        ),
    }
}

fn create_render_state(
    device: &wgpu::Device,
    buffers: &Buffers,
    format: wgpu::TextureFormat,
) -> RenderState {
    let params_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("render_params"),
        size: size_of::<RenderParams>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("render_bgl"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("render_bg"),
        layout: &bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: buffers.positions.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: buffers.species.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: params_buf.as_entire_binding(),
            },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("render_pl"),
        bind_group_layouts: &[&bgl],
        push_constant_ranges: &[],
    });

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("particle_render"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/particle.wgsl").into()),
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("particle_render"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    RenderState {
        pipeline,
        bind_group,
        params_buf,
        machine: create_machine_render_state(device, format),
    }
}

fn create_machine_render_state(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
) -> MachineRenderState {
    let data_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("machine_render_data"),
        size: u64::from(MAX_MACHINES) * size_of::<GpuMachineRender>() as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let params_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("machine_render_params"),
        size: size_of::<MachineRenderParams>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("machine_render_bgl"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("machine_render_bg"),
        layout: &bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: data_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: params_buf.as_entire_binding(),
            },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("machine_render_pl"),
        bind_group_layouts: &[&bgl],
        push_constant_ranges: &[],
    });

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("machine_render"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/machine.wgsl").into()),
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("machine_render"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    MachineRenderState {
        pipeline,
        bind_group,
        params_buf,
        data_buf,
    }
}

fn init_machines(mut rapier: RapierState) -> (Vec<MachineDef>, Vec<MachineCpuState>, RapierState) {
    let mut machines = Vec::with_capacity(3);
    let mut cpu_states = Vec::with_capacity(3);

    // Conveyor: capsule-shaped belt tilted diagonally.
    // Paddles ride along the perimeter, computed in update_machines().
    // The body is a kinematic OBB at CONVEYOR_ANGLE_DEG — particles collide
    // with the full capsule interior so they can't pass through.
    let conv_angle = CONVEYOR_ANGLE_DEG.to_radians();
    let conv_body_hw = CAPSULE_RADIUS; // perpendicular to belt
    let conv_body_hh = CAPSULE_HALF_LEN; // along belt
    let pivot = [0.0, -0.22];
    {
        let body = RigidBodyBuilder::kinematic_position_based()
            .translation(Vec2::new(pivot[0], pivot[1]))
            .rotation(conv_angle)
            .build();
        let body_handle = rapier.bodies.insert(body);
        rapier.colliders.insert_with_parent(
            ColliderBuilder::cuboid(conv_body_hw, conv_body_hh).build(),
            body_handle,
            &mut rapier.bodies,
        );
        machines.push(MachineDef {
            kind: MachineKind::Conveyor,
            body_handle,
            input_species: 0,
            output_species: 0,
        });
        cpu_states.push(MachineCpuState {
            recipe: Recipe {
                input_species: 0,
                input_count: 0,
                output_species: 0,
                output_count: 0,
                cycle_time: 0.0,
            },
            input_accumulated: 0,
            cycles_completed: 0,
            cycle_timer: 0.0,
            consumed_this_frame: 0,
            color_base: [0.25, 0.28, 0.35],
        });
    }

    // Grinder: static sensor box — transmutes Red(0) → Yellow(3).
    let grinder_pos = [0.52, -0.1];
    {
        let body = RigidBodyBuilder::fixed()
            .translation(Vec2::new(grinder_pos[0], grinder_pos[1]))
            .build();
        let body_handle = rapier.bodies.insert(body);
        rapier.colliders.insert_with_parent(
            ColliderBuilder::cuboid(SENSOR_HALF, SENSOR_HALF)
                .sensor(true)
                .build(),
            body_handle,
            &mut rapier.bodies,
        );
        machines.push(MachineDef {
            kind: MachineKind::Grinder,
            body_handle,
            input_species: 0,
            output_species: 3,
        });
        cpu_states.push(MachineCpuState {
            recipe: Recipe {
                input_species: 0,
                input_count: 10,
                output_species: 3,
                output_count: 10,
                cycle_time: 1.0,
            },
            input_accumulated: 0,
            cycles_completed: 0,
            cycle_timer: 1.0,
            consumed_this_frame: 0,
            color_base: [0.9, 0.55, 0.15],
        });
    }

    // Heater: static sensor box — transmutes Blue(1) → Purple(4).
    let heater_pos = [-0.52, -0.1];
    {
        let body = RigidBodyBuilder::fixed()
            .translation(Vec2::new(heater_pos[0], heater_pos[1]))
            .build();
        let body_handle = rapier.bodies.insert(body);
        rapier.colliders.insert_with_parent(
            ColliderBuilder::cuboid(SENSOR_HALF, SENSOR_HALF)
                .sensor(true)
                .build(),
            body_handle,
            &mut rapier.bodies,
        );
        machines.push(MachineDef {
            kind: MachineKind::Heater,
            body_handle,
            input_species: 1,
            output_species: 4,
        });
        cpu_states.push(MachineCpuState {
            recipe: Recipe {
                input_species: 1,
                input_count: 10,
                output_species: 4,
                output_count: 10,
                cycle_time: 1.5,
            },
            input_accumulated: 0,
            cycles_completed: 0,
            cycle_timer: 1.5,
            consumed_this_frame: 0,
            color_base: [0.85, 0.25, 0.1],
        });
    }

    (machines, cpu_states, rapier)
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
