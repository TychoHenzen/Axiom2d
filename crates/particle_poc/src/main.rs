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
struct SimParams {
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
enum MachineKind {
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

struct Buffers {
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

struct ComputePipelines {
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

struct MachineRenderState {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    params_buf: wgpu::Buffer,
    data_buf: wgpu::Buffer,
}

struct RenderState {
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

impl State {
    #[allow(clippy::fn_params_excessive_bools)]
    fn new(
        window: Arc<Window>,
        benchmark: bool,
        diagnose: bool,
        test_bond_form: bool,
        test_bond_constrain: bool,
        test_bond_break: bool,
        test_paddle_stability: bool,
        test_paddle_root_cause: bool,
        num_particles: u32,
        sub_steps: u32,
    ) -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let surface = instance.create_surface(window.clone()).expect("surface");
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            power_preference: wgpu::PowerPreference::HighPerformance,
            ..Default::default()
        }))
        .expect("no compatible GPU adapter");

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None))
                .expect("failed to create device");

        let size = window.inner_size();
        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .or_else(|| caps.formats.first().copied())
            .expect("no surface formats");
        let present_mode = if benchmark || diagnose {
            wgpu::PresentMode::AutoNoVsync
        } else {
            wgpu::PresentMode::AutoVsync
        };
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        let buffers = create_buffers(&device);
        let particle_bgl = create_particle_bgl(&device);
        let grid_bgl = create_grid_bgl(&device);
        let detection_bgl = create_detection_bgl(&device);

        // Init Rapier2D for machine rigid bodies.
        let rapier = RapierState {
            pipeline: PhysicsPipeline::new(),
            integration_parameters: IntegrationParameters::default(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            impulse_joints: ImpulseJointSet::new(),
            multibody_joints: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
        };

        let machine_params_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("machine_params"),
            size: size_of::<MachineParams>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let (particle_bg, grid_bg) = create_bind_groups(
            &device,
            &buffers,
            &machine_params_buf,
            &particle_bgl,
            &grid_bgl,
        );
        let detection_bg =
            create_detection_bg(&device, &buffers, &machine_params_buf, &detection_bgl);
        let pipelines = create_pipelines(&device, &particle_bgl, &grid_bgl, &detection_bgl);
        let render = create_render_state(&device, &buffers, surface_config.format);

        let sim_params = SimParams {
            particle_count: 0,
            dt: 1.0 / 960.0,
            // Scaled so max free-fall speed (from full box height ≈ 1.5) is
            // sqrt(2·1.2·1.5) ≈ 1.9 ≈ one particle radius per substep — the
            // hard ceiling below which particles cannot tunnel through each
            // other. Also keeps bottom-of-pile pressure (∝ g · layer count)
            // within what the projection solver can support: at g=9.81 the
            // ~225-layer pile crushed its bottom layer into instability.
            gravity: -1.2,
            particle_radius: 0.002,
            wall_min_x: -0.8,
            wall_min_y: -0.8,
            wall_max_x: 0.8,
            wall_max_y: 0.8,
            friction_mu: 0.3,
            grid_cell_size: 1.6 / GRID_W as f32,
            grid_width: GRID_W,
            grid_height: GRID_H,
            disable_velocity_cap: 0,
            sub_steps,
        };

        // Initialize reaction matrix.
        // Red(0)+Blue(1) → both become Green(2)
        // Green(2)+Purple(4) → Green becomes Yellow(3), Purple stays Purple
        let mut reaction_matrix = ReactionMatrix {
            results: [0u32; (MAX_SPECIES * MAX_SPECIES) as usize],
        };
        reaction_matrix.results[1] = 2; // Red(0) + Blue(1) → Green(2)
        reaction_matrix.results[MAX_SPECIES as usize] = 2; // Blue(1) + Red(0) → Green(2)
        reaction_matrix.results[2 * MAX_SPECIES as usize + 4] = 3; // Green(2) + Purple(4) → Yellow(3)
        reaction_matrix.results[4 * MAX_SPECIES as usize + 2] = 4; // Purple(4) + Green(2) → Purple(4)
        queue.write_buffer(
            &buffers.reaction_matrix,
            0,
            bytemuck::bytes_of(&reaction_matrix),
        );

        // Initialize bond buffer: all slots invalid.
        {
            let invalid_bonds = vec![BondSlot::default(); MAX_PARTICLES as usize * 4];
            queue.write_buffer(&buffers.bonds, 0, bytemuck::cast_slice(&invalid_bonds));
        }

        // Staging buffer for counter readback (reused each frame).
        // Initialize with zeros so frame 0 readback is a safe no-op.
        let counters_staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("counters_staging"),
            size: u64::from(MAX_MACHINES) * 4,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        queue.write_buffer(
            &counters_staging,
            0,
            bytemuck::cast_slice(&[0u32; MAX_MACHINES as usize]),
        );

        // Create machine definitions, CPU state, and Rapier2D bodies.
        let (machines, machines_cpu, rapier) = init_machines(rapier);

        Self {
            window,
            device,
            queue,
            surface,
            surface_config,
            buffers,
            particle_bg,
            grid_bg,
            pipelines,
            render,
            detection_bg,
            sim_params,
            fps_tracker: FpsTracker::new(),
            benchmark,
            bench_frame: 0,
            bench_start: None,
            bench_done: false,
            test_bond_form,
            test_bond_constrain,
            test_bond_break,
            test_paddle_stability,
            test_paddle_root_cause,
            test_phase: 0,
            test_report_done: false,
            diagnose,
            diag_frame: 0,
            diag_staging: None,
            rapier,
            machines,
            machines_cpu,
            machine_params_buf,
            counters_staging,
            machine_time: 0.0,
            num_particles,
            sub_steps,
            max_speed_seen: 0.0,
            outlier_staging: None,
            phasing_staging: None,
            outlier_count_staging: None,
            phasing_count_staging: None,
        }
    }

    fn verify_stability(&mut self) -> bool {
        let n = self.sim_params.particle_count as usize;
        // Read both positions and velocities.
        let pos_bytes = u64::from(MAX_PARTICLES) * 8;
        let vel_bytes = u64::from(MAX_PARTICLES) * 8;
        let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("stability_staging"),
            size: pos_bytes + vel_bytes,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("stability_copy"),
            });
        encoder.copy_buffer_to_buffer(&self.buffers.positions, 0, &staging, 0, pos_bytes);
        encoder.copy_buffer_to_buffer(&self.buffers.velocities, 0, &staging, pos_bytes, vel_bytes);
        self.queue.submit(std::iter::once(encoder.finish()));
        let slice = staging.slice(..);
        slice.map_async(wgpu::MapMode::Read, |r| r.expect("map stability staging"));
        self.device.poll(wgpu::Maintain::Wait);
        let data = slice.get_mapped_range();
        let positions: &[[f32; 2]] = bytemuck::cast_slice(&data[..n * 8]);
        let vel_offset = pos_bytes as usize;
        let velocities: &[[f32; 2]] = bytemuck::cast_slice(&data[vel_offset..vel_offset + n * 8]);

        // Stability: no NaN, no OOB, zero 5σ KE outliers above median.
        // Read species and bonds for outlier logging.
        let bonds = self.read_bonds();
        let sp_bytes = u64::from(n as u32) * 4;
        let sp_staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("stab_sp"),
            size: sp_bytes,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        {
            let mut enc = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("stab_sp_copy"),
                });
            enc.copy_buffer_to_buffer(&self.buffers.species, 0, &sp_staging, 0, sp_bytes);
            self.queue.submit(std::iter::once(enc.finish()));
        }
        let sp_slice = sp_staging.slice(..);
        sp_slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device.poll(wgpu::Maintain::Wait);
        let sp_data = sp_slice.get_mapped_range();
        let species: &[u32] = bytemuck::cast_slice(&sp_data[..n * 4]);

        let p = &self.sim_params;
        let mut nan_count = 0usize;
        let mut max_speed = 0.0f32;
        let mut oob = 0usize;
        let margin = 2.0 * p.particle_radius;

        // Collect per-particle kinetic energy for statistical analysis.
        let mut kes: Vec<f32> = Vec::with_capacity(n);

        for i in 0..n {
            let [px, py] = positions[i];
            let [vx, vy] = velocities[i];
            if !px.is_finite() || !py.is_finite() || !vx.is_finite() || !vy.is_finite() {
                nan_count += 1;
                continue;
            }
            let speed = (vx * vx + vy * vy).sqrt();
            max_speed = max_speed.max(speed);
            let ke = 0.5 * (vx * vx + vy * vy);
            kes.push(ke);
            if px < p.wall_min_x - margin
                || px > p.wall_max_x + margin
                || py < p.wall_min_y - margin
                || py > p.wall_max_y + margin
            {
                oob += 1;
            }
        }

        // Compute median and stddev of kinetic energy.
        kes.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let median_ke = if kes.is_empty() {
            0.0
        } else if kes.len() % 2 == 1 {
            kes[kes.len() / 2]
        } else {
            (kes[kes.len() / 2 - 1] + kes[kes.len() / 2]) * 0.5
        };
        let mean_ke = kes.iter().sum::<f32>() / (kes.len().max(1) as f32);
        let variance = kes
            .iter()
            .map(|k| (k - mean_ke) * (k - mean_ke))
            .sum::<f32>()
            / (kes.len().max(1) as f32);
        let stddev_ke = variance.sqrt();
        let outlier_threshold = median_ke + 5.0 * stddev_ke;

        // Count and log outliers in second pass.
        let mut ke_outliers = 0usize;
        let mut green_outliers = 0usize;
        if stddev_ke > 1e-10 {
            // Collect all outlier indices with KE for sorting.
            let mut outlier_list: Vec<(usize, f32)> = Vec::new();
            for i in 0..n {
                let [px, py] = positions[i];
                let [vx, vy] = velocities[i];
                if !px.is_finite() || !py.is_finite() || !vx.is_finite() || !vy.is_finite() {
                    continue;
                }
                let ke = 0.5 * (vx * vx + vy * vy);
                if ke > outlier_threshold {
                    outlier_list.push((i, ke));
                    if species[i] == GREEN_SPECIES {
                        green_outliers += 1;
                    }
                }
            }
            ke_outliers = outlier_list.len();

            // Sort by KE descending, log top 30.
            outlier_list.sort_unstable_by(|a, b| {
                b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
            });
            for (_idx, (i, ke)) in outlier_list.iter().enumerate().take(30) {
                let slot_a = bonds[*i * 4];
                let slot_b = bonds[*i * 4 + 1];
                let slot_c = bonds[*i * 4 + 2];
                let slot_d = bonds[*i * 4 + 3];
                let [px, py] = positions[*i];
                let [vx, vy] = velocities[*i];
                let sp = species[*i];
                let is_bonded = slot_a.partner != INVALID_BOND
                    || slot_b.partner != INVALID_BOND
                    || slot_c.partner != INVALID_BOND
                    || slot_d.partner != INVALID_BOND;
                let bond_count = u32::from(slot_a.partner != INVALID_BOND)
                    + u32::from(slot_b.partner != INVALID_BOND)
                    + u32::from(slot_c.partner != INVALID_BOND)
                    + u32::from(slot_d.partner != INVALID_BOND);
                println!(
                    "  outlier[{}] sp={} pos=({:.4},{:.4}) vel=({:.4},{:.4}) \
                     KE={:.6} bonds={} a=(p={} r={:.4}) b=(p={} r={:.4}) \
                     c=(p={} r={:.4}) d=(p={} r={:.4}) {}",
                    i,
                    sp,
                    px,
                    py,
                    vx,
                    vy,
                    *ke,
                    bond_count,
                    slot_a.partner,
                    slot_a.rest,
                    slot_b.partner,
                    slot_b.rest,
                    slot_c.partner,
                    slot_c.rest,
                    slot_d.partner,
                    slot_d.rest,
                    if is_bonded { "(bonded)" } else { "(free)" },
                );
            }
            if ke_outliers > 30 {
                println!(
                    "  ... and {} more KE outliers (top 30 shown, {green_outliers} green)",
                    ke_outliers - 30
                );
            }
        }

        drop(sp_data);
        sp_staging.unmap();

        let tracked_max = self.max_speed_seen;
        let nan_ok = nan_count == 0;
        let oob_ok = oob == 0;
        let ke_ok = ke_outliers == 0;
        let speed_ok = max_speed < 2.0;
        let tracked_ok = tracked_max < 2.0;
        let all_ok = nan_ok && oob_ok && ke_ok && speed_ok && tracked_ok;

        println!(
            "Stability: n={n} nan={nan_count} oob={oob} \
             ke_outliers={ke_outliers}/{n} (green={green_outliers} median={median_ke:.6} σ={stddev_ke:.6} thresh={outlier_threshold:.6}) \
             vmax={max_speed:.3} tracked_max={tracked_max:.3} — {}",
            if all_ok { "PASS" } else { "FAIL" }
        );
        if !nan_ok {
            println!("  FAIL: {nan_count} NaN/infinite values");
        }
        if !oob_ok {
            println!("  FAIL: {oob} out-of-bounds particles");
        }
        if !ke_ok {
            println!(
                "  FAIL: {ke_outliers} KE outliers > median+5σ (must be 0, {green_outliers} green)"
            );
        }
        if !speed_ok {
            println!("  FAIL: max_speed={max_speed:.3} >= 2.0");
        }
        if !tracked_ok {
            println!(
                "  FAIL: tracked_max_speed={tracked_max:.3} >= 2.0 (transient spike detected during simulation)"
            );
        }

        drop(data);
        staging.unmap();
        all_ok
    }

    fn read_diagnostics(&mut self) {
        let n = self.sim_params.particle_count as usize;
        if n == 0 {
            println!("diag frame {:>5}: n=0", self.diag_frame);
            return;
        }
        let bytes_per = u64::from(MAX_PARTICLES) * 8;
        let staging = self.diag_staging.get_or_insert_with(|| {
            self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("diag_staging"),
                size: bytes_per * 2,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            })
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("diag_copy"),
            });
        encoder.copy_buffer_to_buffer(&self.buffers.positions, 0, staging, 0, bytes_per);
        encoder.copy_buffer_to_buffer(&self.buffers.velocities, 0, staging, bytes_per, bytes_per);
        self.queue.submit(std::iter::once(encoder.finish()));

        let slice = staging.slice(..);
        slice.map_async(wgpu::MapMode::Read, |r| r.expect("map diag staging"));
        self.device.poll(wgpu::Maintain::Wait);

        let data = slice.get_mapped_range();
        let all: &[[f32; 2]] = bytemuck::cast_slice(&data);
        let positions = &all[..n];
        let velocities = &all[MAX_PARTICLES as usize..MAX_PARTICLES as usize + n];

        let mut nan_count = 0usize;
        let mut max_speed = 0.0f32;
        let mut ke = 0.0f64;
        let mut momentum = [0.0f64; 2];
        let mut oob = 0usize;
        let (mut min_y, mut max_y) = (f32::MAX, f32::MIN);
        let p = &self.sim_params;
        for i in 0..n {
            let [px, py] = positions[i];
            let [vx, vy] = velocities[i];
            if !px.is_finite() || !py.is_finite() || !vx.is_finite() || !vy.is_finite() {
                nan_count += 1;
                continue;
            }
            let speed = (vx * vx + vy * vy).sqrt();
            max_speed = max_speed.max(speed);
            ke += 0.5 * f64::from(speed) * f64::from(speed);
            momentum[0] += f64::from(vx);
            momentum[1] += f64::from(vy);
            min_y = min_y.min(py);
            max_y = max_y.max(py);
            let m = 2.0 * p.particle_radius;
            if px < p.wall_min_x - m
                || px > p.wall_max_x + m
                || py < p.wall_min_y - m
                || py > p.wall_max_y + m
            {
                oob += 1;
            }
        }

        // CPU spatial binning for overlap stats
        let cell = 2.0 * p.particle_radius;
        let mut bins: std::collections::HashMap<(i32, i32), Vec<u32>> =
            std::collections::HashMap::new();
        for (i, &[px, py]) in positions.iter().enumerate() {
            if !px.is_finite() || !py.is_finite() {
                continue;
            }
            let key = ((px / cell).floor() as i32, (py / cell).floor() as i32);
            bins.entry(key).or_default().push(i as u32);
        }
        let contact_dist = 2.0 * p.particle_radius;
        let mut max_overlap = 0.0f32;
        let mut contacts = 0usize;
        let mut deep_contacts = 0usize; // overlap > 50% of radius
        for (&(cx, cy), ids) in &bins {
            for &i in ids {
                let [ix, iy] = positions[i as usize];
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        if let Some(neighbors) = bins.get(&(cx + dx, cy + dy)) {
                            for &j in neighbors {
                                if j <= i {
                                    continue;
                                }
                                let [jx, jy] = positions[j as usize];
                                let d = ((ix - jx).powi(2) + (iy - jy).powi(2)).sqrt();
                                if d < contact_dist {
                                    let ov = contact_dist - d;
                                    max_overlap = max_overlap.max(ov);
                                    contacts += 1;
                                    if ov > 0.5 * p.particle_radius {
                                        deep_contacts += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        drop(data);
        staging.unmap();

        let frame = self.diag_frame;
        let (px, py) = (momentum[0], momentum[1]);
        let ov_pct = max_overlap / p.particle_radius * 100.0;
        println!(
            "diag frame {frame:>5}: n={n} nan={nan_count} vmax={max_speed:.3} KE={ke:.3} \
             px={px:.3} py={py:.3} ymin={min_y:.3} ymax={max_y:.3} oob={oob} \
             contacts={contacts} deep={deep_contacts} maxOv={ov_pct:.1}%r"
        );
    }

    fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        if size.width > 0 && size.height > 0 {
            self.surface_config.width = size.width;
            self.surface_config.height = size.height;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }

    /// Capsule perimeter parameterization: given arc-length `s` along the
    /// perimeter, returns (`local_x`, `local_y`, `tangent_angle`) in capsule-local
    /// coordinates where the capsule is centered at origin, long axis along X.
    /// Walk CCW starting from rightmost point (l+r, 0).
    ///   L = half-length from center to cap center
    ///   R = capsule radius (half-thickness)
    fn capsule_perimeter_point(s: f32, l: f32, r: f32) -> ([f32; 2], f32) {
        let cap_arc = std::f32::consts::PI * r;
        let straight = 2.0 * l;
        let perimeter = 2.0 * cap_arc + 2.0 * straight;
        let s = ((s % perimeter) + perimeter) % perimeter;

        if s < cap_arc {
            // Right cap: bottom → top (CCW). φ from -π/2 to π/2.
            let phi = -std::f32::consts::FRAC_PI_2 + s / r;
            let lx = l + r * phi.cos();
            let ly = r * phi.sin();
            ([lx, ly], phi + std::f32::consts::FRAC_PI_2)
        } else if s < cap_arc + straight {
            // Top edge: right → left.
            let t = (s - cap_arc) / straight;
            let lx = l - t * 2.0 * l;
            let ly = r;
            ([lx, ly], std::f32::consts::PI)
        } else if s < 2.0 * cap_arc + straight {
            // Left cap: top → bottom (CCW). φ from π/2 → π → 3π/2 (outside arc).
            let phi = std::f32::consts::FRAC_PI_2 + (s - cap_arc - straight) / r;
            let lx = -l + r * phi.cos();
            let ly = r * phi.sin();
            ([lx, ly], phi + std::f32::consts::FRAC_PI_2)
        } else {
            // Bottom edge: left → right.
            let t = (s - 2.0 * cap_arc - straight) / straight;
            let lx = -l + t * 2.0 * l;
            let ly = -r;
            ([lx, ly], 0.0)
        }
    }

    fn update_machines(&mut self) {
        let dt = 1.0 / 60.0;
        self.machine_time += dt;

        // Read GPU counters from previous frame's compute pass.
        // The copy (counters → staging) was submitted last frame in simulate().
        // On frame 0 the staging is all zeros (fresh buffer) — safe no-op.
        {
            let slice = self.counters_staging.slice(..);
            slice.map_async(wgpu::MapMode::Read, |_| {});
            self.device.poll(wgpu::Maintain::Wait);
            let data = slice.get_mapped_range();
            let counters: &[u32] = bytemuck::cast_slice(&data[..self.machines.len() * 4]);
            for (i, cpu) in self.machines_cpu.iter_mut().enumerate() {
                let eaten = counters.get(i).copied().unwrap_or(0);
                cpu.consumed_this_frame = eaten;
                // Conveyor has no recipe — skip tracking.
                if cpu.recipe.input_count > 0 {
                    cpu.input_accumulated += eaten;
                    cpu.cycle_timer -= dt;
                    while cpu.input_accumulated >= cpu.recipe.input_count {
                        cpu.input_accumulated -= cpu.recipe.input_count;
                        cpu.cycles_completed += 1;
                        cpu.cycle_timer = cpu.recipe.cycle_time;
                    }
                }
            }
            drop(data);
            self.counters_staging.unmap();
        }

        // Reset GPU counters to zero for this frame's compute pass.
        let zeros = [0u32; MAX_MACHINES as usize];
        self.queue.write_buffer(
            &self.buffers.machine_counters,
            0,
            bytemuck::cast_slice(&zeros),
        );

        // Step Rapier2D to get updated transforms.
        self.rapier.integration_parameters.dt = dt;
        self.rapier.pipeline.step(
            Vec2::new(0.0, 0.0),
            &self.rapier.integration_parameters,
            &mut self.rapier.island_manager,
            &mut self.rapier.broad_phase,
            &mut self.rapier.narrow_phase,
            &mut self.rapier.bodies,
            &mut self.rapier.colliders,
            &mut self.rapier.impulse_joints,
            &mut self.rapier.multibody_joints,
            &mut self.rapier.ccd_solver,
            &(),
            &(),
        );

        // Capsule conveyor body stays at fixed angle — no rotation.
        // Paddles orbit along capsule perimeter at CONVEYOR_SPEED.
        let conveyor_angle = CONVEYOR_ANGLE_DEG.to_radians();
        let (c_cos, c_sin) = (conveyor_angle.cos(), conveyor_angle.sin());
        let cap_perim = 2.0 * std::f32::consts::PI * CAPSULE_RADIUS + 4.0 * CAPSULE_HALF_LEN;
        let paddle_phase =
            (self.machine_time * CONVEYOR_SPEED * cap_perim / std::f32::consts::TAU) % cap_perim;

        let real_n = self.machines.len() as u32;
        let total_n = real_n + PADDLE_COUNT;

        let mut sim = MachineParams {
            count: total_n,
            ..Default::default()
        };
        let mut renders = [GpuMachineRender::default(); MAX_MACHINES as usize];

        // Fill real machines from Rapier bodies.
        for (i, def) in self.machines.iter().enumerate() {
            let body = self.rapier.bodies.get(def.body_handle).unwrap();
            let pos = body.position();
            let t = pos.translation;
            let angle = pos.rotation.angle();
            let (hw, hh) = if def.kind as u32 == 0 {
                // Capsule body: interior of the tilted conveyor belt.
                // Must match Rapier collider shape: half_width = CAPSULE_RADIUS,
                // half_height = CAPSULE_HALF_LEN. Particles cannot pass through.
                (CAPSULE_RADIUS, CAPSULE_HALF_LEN)
            } else {
                (SENSOR_HALF, SENSOR_HALF)
            };
            sim.machines[i] = GpuMachine {
                pos_x: t.x,
                pos_y: t.y,
                cos_angle: angle.cos(),
                sin_angle: angle.sin(),
                half_width: hw,
                half_height: hh,
                kind: def.kind as u32,
                input_species: def.input_species,
                output_species: def.output_species,
                angular_velocity: if def.kind as u32 == 0 {
                    CONVEYOR_SPEED
                } else {
                    0.0
                },
            };
            let cpu = &self.machines_cpu[i];
            let activity = (cpu.consumed_this_frame as f32 / 20.0).clamp(0.0, 1.0);
            let brightness = 0.6 + 0.4 * activity;
            let cb = cpu.color_base;
            // Render track slightly thinner than collision hull.
            let (rhw, rhh) = if def.kind as u32 == 0 {
                (CAPSULE_RADIUS * 0.8, CAPSULE_HALF_LEN)
            } else {
                (hw, hh)
            };
            renders[i] = GpuMachineRender {
                pos_x: t.x,
                pos_y: t.y,
                cos_angle: angle.cos(),
                sin_angle: angle.sin(),
                half_width: rhw,
                half_height: rhh,
                color_r: (cb[0] * brightness).clamp(0.0, 1.0),
                color_g: (cb[1] * brightness).clamp(0.0, 1.0),
                color_b: (cb[2] * brightness).clamp(0.0, 1.0),
            };
        }

        // Compute paddle positions along capsule perimeter.
        let cx = sim.machines[0].pos_x;
        let cy = sim.machines[0].pos_y;
        let paddle_color = [0.45f32, 0.50, 0.60];
        for p in 0..PADDLE_COUNT {
            let s = (paddle_phase + p as f32 * cap_perim / PADDLE_COUNT as f32) % cap_perim;
            let (pos, local_tangent) =
                Self::capsule_perimeter_point(s, CAPSULE_HALF_LEN, CAPSULE_RADIUS);
            let lx = pos[0];
            let ly = pos[1];
            // Capsule long axis = X in capsule-local, but conveyor long axis = local Y.
            // Rotate by π/2+θ so capsule X → world dir (-sin_θ, cos_θ).
            let wx = cx - lx * c_sin - ly * c_cos;
            let wy = cy + lx * c_cos - ly * c_sin;
            let world_tangent = local_tangent + std::f32::consts::FRAC_PI_2 + conveyor_angle;
            let idx = (real_n + p) as usize;
            sim.machines[idx] = GpuMachine {
                pos_x: wx,
                pos_y: wy,
                cos_angle: world_tangent.cos(),
                sin_angle: world_tangent.sin(),
                half_width: PADDLE_HW,
                half_height: PADDLE_HH,
                kind: MachineKind::Paddle as u32,
                input_species: 0,
                output_species: 0,
                angular_velocity: CONVEYOR_SPEED,
            };
            renders[idx] = GpuMachineRender {
                pos_x: wx,
                pos_y: wy,
                cos_angle: world_tangent.cos(),
                sin_angle: world_tangent.sin(),
                half_width: PADDLE_HW,
                half_height: PADDLE_HH,
                color_r: paddle_color[0],
                color_g: paddle_color[1],
                color_b: paddle_color[2],
            };
        }

        self.queue
            .write_buffer(&self.machine_params_buf, 0, bytemuck::bytes_of(&sim));
        self.queue.write_buffer(
            &self.render.machine.data_buf,
            0,
            bytemuck::cast_slice(&renders[..total_n as usize]),
        );
    }

    fn spawn(&mut self) {
        let current = self.sim_params.particle_count;
        let to_spawn = SPAWN_RATE.min(MAX_PARTICLES - current);
        if to_spawn == 0 {
            return;
        }

        let mut positions = Vec::with_capacity(to_spawn as usize);
        let mut velocities = Vec::with_capacity(to_spawn as usize);
        let mut species = Vec::with_capacity(to_spawn as usize);

        let r = self.sim_params.particle_radius;
        // Each hopper gets roughly half. Alternating which hopper fires first
        // keeps the split even over time.
        let left_count = to_spawn / 2;
        let right_count = to_spawn - left_count;

        // Helper: fill one hopper's row
        let mut spawn_hopper = |count: u32, center_x: f32, half_w: f32, sp: u32| {
            let hopper_width = 2.0 * half_w;
            let spacing = hopper_width / count as f32;
            let max_jitter = 0.5 * (spacing - 2.0 * r).max(0.0);
            for i in 0..count {
                let hash = (current + i + sp * SPAWN_RATE).wrapping_mul(0x9E37_79B9);
                let jitter = (((hash >> 16) ^ hash) as f32 / u32::MAX as f32) - 0.5;
                let t = (i as f32 + 0.5) / count as f32;
                let x = center_x - half_w + t * hopper_width + jitter * max_jitter;
                positions.push([x, HOPPER_Y]);
                velocities.push([0.0f32, -0.5]);
                species.push(sp);
            }
        };

        spawn_hopper(left_count, HOPPER_LEFT_X, HOPPER_LEFT_HALF, 0);
        spawn_hopper(right_count, HOPPER_RIGHT_X, HOPPER_RIGHT_HALF, 1);

        let offset = u64::from(current);
        self.queue.write_buffer(
            &self.buffers.positions,
            offset * 8,
            bytemuck::cast_slice(&positions),
        );
        self.queue.write_buffer(
            &self.buffers.velocities,
            offset * 8,
            bytemuck::cast_slice(&velocities),
        );
        self.queue.write_buffer(
            &self.buffers.species,
            offset * 4,
            bytemuck::cast_slice(&species),
        );
        self.sim_params.particle_count = current + to_spawn;
    }

    fn spawn_all(&mut self) {
        let count = self.num_particles;
        let r = self.sim_params.particle_radius;
        let spacing = 2.1 * r;
        let usable_w = self.sim_params.wall_max_x - self.sim_params.wall_min_x - 2.0 * r;
        let cols = (usable_w / spacing) as u32;

        let mut positions: Vec<[f32; 2]> = Vec::with_capacity(count as usize);
        let mut velocities: Vec<[f32; 2]> = Vec::with_capacity(count as usize);
        let mut species: Vec<u32> = Vec::with_capacity(count as usize);

        for i in 0..count {
            let col = i % cols;
            let row = i / cols;
            let x = self.sim_params.wall_min_x + r + col as f32 * spacing;
            let y = self.sim_params.wall_max_y - r - row as f32 * spacing;
            positions.push([x, y]);
            velocities.push([0.0f32, 0.0]);
            // Left half of box = Red (0), right half = Blue (1)
            species.push(u32::from(x >= 0.0));
        }

        self.queue
            .write_buffer(&self.buffers.positions, 0, bytemuck::cast_slice(&positions));
        self.queue.write_buffer(
            &self.buffers.velocities,
            0,
            bytemuck::cast_slice(&velocities),
        );
        self.queue
            .write_buffer(&self.buffers.species, 0, bytemuck::cast_slice(&species));
        self.sim_params.particle_count = count;
    }

    fn test_setup(&mut self) {
        let n = MAX_PARTICLES as usize;

        // Zero gravity for bond tests; paddle tests need gravity for realistic interaction.
        let is_bond_test = self.test_bond_form || self.test_bond_constrain || self.test_bond_break;
        if is_bond_test {
            self.sim_params.gravity = 0.0;
        }
        self.sim_params.disable_velocity_cap = u32::from(self.test_paddle_root_cause);

        // Write test particle data via staging buffers with explicit encoder+submit
        // to ensure writes land before subsequent compute passes.
        {
            let pos_bytes = u64::from(MAX_PARTICLES) * 8;
            let vel_bytes = pos_bytes;
            let sp_bytes = u64::from(MAX_PARTICLES) * 4;
            let total = pos_bytes + vel_bytes + sp_bytes;
            let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("test_init_staging"),
                size: total,
                usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::MAP_WRITE,
                mapped_at_creation: true,
            });
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("test_init"),
                });

            if self.test_bond_form {
                let r = self.sim_params.particle_radius;
                let spacing = 2.0 * r;
                let pos_data = [[-spacing * 0.5, 0.0f32], [spacing * 0.5, 0.0f32]];
                let vel_data = [[0.0f32, 0.0f32]; 2];
                let sp_data = [GREEN_SPECIES; 2];
                {
                    let mut view = staging.slice(..).get_mapped_range_mut();
                    view[..16].copy_from_slice(bytemuck::cast_slice(&pos_data));
                    view[(pos_bytes as usize)..(pos_bytes as usize) + 16]
                        .copy_from_slice(bytemuck::cast_slice(&vel_data));
                    view[(pos_bytes as usize + vel_bytes as usize)
                        ..(pos_bytes as usize + vel_bytes as usize) + 8]
                        .copy_from_slice(bytemuck::cast_slice(&sp_data));
                }
                staging.unmap();
                self.sim_params.particle_count = 2;
            } else if self.test_bond_constrain || self.test_bond_break {
                // constrain or break: set up mutual bond between 2 green particles
                let dist = if self.test_bond_constrain { 0.06 } else { 0.30 };
                let half = dist * 0.5;
                let pos_data = [[-half, 0.0f32], [half, 0.0f32]];
                let vel_data = [[0.0f32, 0.0f32]; 2];
                let sp_data = [GREEN_SPECIES; 2];
                {
                    let mut view = staging.slice(..).get_mapped_range_mut();
                    view[..16].copy_from_slice(bytemuck::cast_slice(&pos_data));
                    view[(pos_bytes as usize)..(pos_bytes as usize) + 16]
                        .copy_from_slice(bytemuck::cast_slice(&vel_data));
                    view[(pos_bytes as usize + vel_bytes as usize)
                        ..(pos_bytes as usize + vel_bytes as usize) + 8]
                        .copy_from_slice(bytemuck::cast_slice(&sp_data));
                }
                staging.unmap();

                self.sim_params.particle_count = 2;

                // Clear all bond slots, then write mutual bond: 0↔1 via slot 0.
                let invalid = vec![BondSlot::default(); n * 4];
                self.queue
                    .write_buffer(&self.buffers.bonds, 0, bytemuck::cast_slice(&invalid));
                let mut bonds = vec![BondSlot::default(); n * 4];
                // Particle 0 slot 0 → partner 1, Particle 1 slot 0 → partner 0
                bonds[0] = BondSlot {
                    partner: 1,
                    rest: 0.03,
                };
                bonds[4] = BondSlot {
                    partner: 0,
                    rest: 0.03,
                };
                self.queue
                    .write_buffer(&self.buffers.bonds, 0, bytemuck::cast_slice(&bonds));
            } else if self.test_paddle_stability || self.test_paddle_root_cause {
                // Spawn particles directly on the belt surface (upward-facing side).
                // Capsule: center (0,-0.22), rotated 45°, half_width=0.055, half_height=0.22.
                // Belt surface points (upward-facing, perpendicular = (-sin45, cos45)):
                //   for t in [-0.22, 0.22]: pos = (0,-0.22) + t*(cos45,sin45) + 0.055*(-sin45,cos45)
                let count = 1000u32;
                let r = self.sim_params.particle_radius;
                let spacing = 2.1 * r;
                let cols = 50u32;
                let mut positions = Vec::with_capacity(count as usize);
                let mut velocities = Vec::with_capacity(count as usize);
                let mut species = Vec::with_capacity(count as usize);
                // Spawn on belt surface so particles immediately rest against capsule body.
                let cos45 = std::f32::consts::FRAC_1_SQRT_2;
                let sin45 = std::f32::consts::FRAC_1_SQRT_2;
                let belt_perp_x = -sin45;
                let belt_perp_y = cos45;
                let belt_tan_x = cos45;
                let belt_tan_y = sin45;
                let belt_cx = 0.0f32;
                let belt_cy = -0.22f32;
                let surface_offset = CAPSULE_RADIUS + r;
                let rows = count / cols;
                for row in 0..rows {
                    for col in 0..cols {
                        let t =
                            -CAPSULE_HALF_LEN + col as f32 * 2.0 * CAPSULE_HALF_LEN / cols as f32;
                        let x = belt_cx + t * belt_tan_x + surface_offset * belt_perp_x;
                        let y = belt_cy + t * belt_tan_y + surface_offset * belt_perp_y
                            - row as f32 * spacing;
                        positions.push([x, y]);
                        velocities.push([0.0f32, 0.0]);
                        species.push((row * cols + col) % 3);
                    }
                }
                {
                    let mut view = staging.slice(..).get_mapped_range_mut();
                    let pos_slice = bytemuck::cast_slice(&positions);
                    view[..pos_slice.len()].copy_from_slice(pos_slice);
                    let vel_slice = bytemuck::cast_slice(&velocities);
                    view[(pos_bytes as usize)..(pos_bytes as usize) + vel_slice.len()]
                        .copy_from_slice(vel_slice);
                    let sp_slice = bytemuck::cast_slice(&species);
                    view[(pos_bytes as usize + vel_bytes as usize)
                        ..(pos_bytes as usize + vel_bytes as usize) + sp_slice.len()]
                        .copy_from_slice(sp_slice);
                }
                staging.unmap();
                self.sim_params.particle_count = count;
                // Machines stay active (not nulled) — paddles collide.
            }

            encoder.copy_buffer_to_buffer(&staging, 0, &self.buffers.positions, 0, pos_bytes);
            encoder.copy_buffer_to_buffer(
                &staging,
                pos_bytes,
                &self.buffers.velocities,
                0,
                vel_bytes,
            );
            encoder.copy_buffer_to_buffer(
                &staging,
                pos_bytes + vel_bytes,
                &self.buffers.species,
                0,
                sp_bytes,
            );
            self.queue.submit(std::iter::once(encoder.finish()));
        }
    }

    fn read_bonds(&mut self) -> Vec<BondSlot> {
        let n = MAX_PARTICLES as usize;
        let total_bytes = (n * 4 * size_of::<BondSlot>()) as u64;
        let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("bond_readback_staging"),
            size: total_bytes,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("bond_readback"),
            });
        encoder.copy_buffer_to_buffer(&self.buffers.bonds, 0, &staging, 0, total_bytes);
        self.queue.submit(std::iter::once(encoder.finish()));
        let slice = staging.slice(..);
        slice.map_async(wgpu::MapMode::Read, |r| r.expect("map bond staging"));
        self.device.poll(wgpu::Maintain::Wait);
        let data = slice.get_mapped_range();
        let all: Vec<BondSlot> = bytemuck::cast_slice(&data[..total_bytes as usize]).to_vec();
        drop(data);
        staging.unmap();
        all
    }

    fn read_test_positions(&mut self, count: u32) -> Vec<[f32; 2]> {
        let bytes = u64::from(count) * 8;
        let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("test_pos_staging"),
            size: bytes,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("test_pos_readback"),
            });
        encoder.copy_buffer_to_buffer(&self.buffers.positions, 0, &staging, 0, bytes);
        self.queue.submit(std::iter::once(encoder.finish()));
        let slice = staging.slice(..);
        slice.map_async(wgpu::MapMode::Read, |r| r.expect("map test pos staging"));
        self.device.poll(wgpu::Maintain::Wait);
        let data = slice.get_mapped_range();
        let positions: Vec<[f32; 2]> = bytemuck::cast_slice(&data[..bytes as usize]).to_vec();
        drop(data);
        staging.unmap();
        positions
    }

    fn test_verify(&mut self) {
        if self.test_bond_form {
            let bonds = self.read_bonds();
            let has_bond = bonds.iter().any(|b| b.partner != INVALID_BOND);
            if has_bond {
                println!("test-bond-form: PASS");
            } else {
                eprintln!("test-bond-form: FAIL — no bonds formed");
                std::process::exit(1);
            }
        } else if self.test_bond_constrain {
            let positions = self.read_test_positions(2);
            let dx = positions[0][0] - positions[1][0];
            let dy = positions[0][1] - positions[1][1];
            let current_dist = (dx * dx + dy * dy).sqrt();
            let initial_dist = 0.06;
            if current_dist < initial_dist * 0.95 {
                println!("test-bond-constrain: PASS (dist {initial_dist:.4} -> {current_dist:.4})");
            } else {
                eprintln!("test-bond-constrain: FAIL — distance not reduced ({current_dist:.4})");
                std::process::exit(1);
            }
        } else if self.test_bond_break {
            let bonds = self.read_bonds();
            let all_cleared = bonds.iter().all(|b| b.partner == INVALID_BOND);
            if all_cleared {
                println!("test-bond-break: PASS");
            } else {
                eprintln!("test-bond-break: FAIL — bonds not cleared");
                std::process::exit(1);
            }
        } else if self.test_paddle_stability || self.test_paddle_root_cause {
            // Verify after 10-second run: speed cap held, no NaN, no tunneling.
            let n = self.sim_params.particle_count as usize;
            // Read positions + velocities.
            let pos_bytes = (n * 8) as u64;
            let vel_bytes = (n * 8) as u64;
            let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("paddle_verify_staging"),
                size: pos_bytes + vel_bytes,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("paddle_verify"),
                });
            encoder.copy_buffer_to_buffer(&self.buffers.positions, 0, &staging, 0, pos_bytes);
            encoder.copy_buffer_to_buffer(
                &self.buffers.velocities,
                0,
                &staging,
                pos_bytes,
                vel_bytes,
            );
            self.queue.submit(std::iter::once(encoder.finish()));
            let slice = staging.slice(..);
            slice.map_async(wgpu::MapMode::Read, |r| r.expect("map paddle verify"));
            self.device.poll(wgpu::Maintain::Wait);
            let data = slice.get_mapped_range();
            let positions: &[[f32; 2]] = bytemuck::cast_slice(&data[..n * 8]);
            let vel_offset = pos_bytes as usize;
            let velocities: &[[f32; 2]] =
                bytemuck::cast_slice(&data[vel_offset..vel_offset + n * 8]);

            let mut nan_count = 0usize;
            let mut max_speed = 0.0f32;
            for i in 0..n {
                let [px, py] = positions[i];
                let [vx, vy] = velocities[i];
                if !px.is_finite() || !py.is_finite() || !vx.is_finite() || !vy.is_finite() {
                    nan_count += 1;
                    continue;
                }
                let speed = (vx * vx + vy * vy).sqrt();
                max_speed = max_speed.max(speed);
            }
            drop(data);
            staging.unmap();

            let nan_ok = nan_count == 0;
            let speed_ok = max_speed < 2.0;
            let tracked_ok = self.max_speed_seen < 2.0;
            let all_ok = nan_ok && speed_ok && tracked_ok;

            println!(
                "test-paddle-stability: n={n} nan={nan_count} vmax={max_speed:.3} tracked_max={:.3} — {}",
                self.max_speed_seen,
                if all_ok { "PASS" } else { "FAIL" },
            );
            if !nan_ok {
                println!("  FAIL: {nan_count} NaN values");
            }
            if !speed_ok {
                println!("  FAIL: max_speed={max_speed:.3} >= 2.0");
            }
            if !tracked_ok {
                println!(
                    "  FAIL: tracked_max_speed={:.3} >= 2.0",
                    self.max_speed_seen
                );
            }
            if !all_ok {
                std::process::exit(1);
            }
        }
    }

    fn simulate(&mut self) {
        self.queue.write_buffer(
            &self.buffers.params,
            0,
            bytemuck::bytes_of(&self.sim_params),
        );

        // Clear outlier/phasing atomic counters and data before this frame's detection passes.
        let zero4: [u8; 4] = [0; 4];
        self.queue
            .write_buffer(&self.buffers.outlier_count_buf, 0, &zero4);
        self.queue
            .write_buffer(&self.buffers.phasing_count_buf, 0, &zero4);

        let pc = self.sim_params.particle_count;
        let particle_wg = pc.div_ceil(WORKGROUP_SIZE);
        let grid_wg = TOTAL_GRID_CELLS.div_ceil(WORKGROUP_SIZE);

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("compute"),
            });

        for _ in 0..self.sub_steps {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("pbd_substep"),
                ..Default::default()
            });
            pass.set_bind_group(0, &self.particle_bg, &[]);
            pass.set_bind_group(1, &self.grid_bg, &[]);

            pass.set_pipeline(&self.pipelines.predict);
            pass.dispatch_workgroups(particle_wg, 1, 1);

            pass.set_pipeline(&self.pipelines.morton_keys);
            pass.dispatch_workgroups(particle_wg, 1, 1);

            pass.set_pipeline(&self.pipelines.clear_cells);
            pass.dispatch_workgroups(grid_wg, 1, 1);

            pass.set_pipeline(&self.pipelines.morton_count);
            pass.dispatch_workgroups(particle_wg, 1, 1);

            pass.set_pipeline(&self.pipelines.prefix_scan);
            pass.dispatch_workgroups(1, 1, 1);

            pass.set_pipeline(&self.pipelines.morton_scatter);
            pass.dispatch_workgroups(particle_wg, 1, 1);

            pass.set_pipeline(&self.pipelines.project);
            pass.dispatch_workgroups(particle_wg, 1, 1);

            pass.set_pipeline(&self.pipelines.solve_bonds);
            pass.dispatch_workgroups(particle_wg, 1, 1);

            pass.set_pipeline(&self.pipelines.apply);
            pass.dispatch_workgroups(particle_wg, 1, 1);
        }
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("reaction"),
                ..Default::default()
            });
            pass.set_bind_group(0, &self.particle_bg, &[]);
            pass.set_bind_group(1, &self.grid_bg, &[]);
            pass.set_pipeline(&self.pipelines.reaction);
            pass.dispatch_workgroups(particle_wg, 1, 1);
        }
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("form_bonds"),
                ..Default::default()
            });
            pass.set_bind_group(0, &self.particle_bg, &[]);
            pass.set_bind_group(1, &self.grid_bg, &[]);
            pass.set_pipeline(&self.pipelines.form_bonds);
            pass.dispatch_workgroups(particle_wg, 1, 1);
            pass.set_pipeline(&self.pipelines.form_bonds_resolve);
            pass.dispatch_workgroups(particle_wg, 1, 1);
        }
        // Outlier + phasing detection (one dispatch per frame, not per substep).
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("detection"),
                ..Default::default()
            });
            pass.set_bind_group(0, &self.detection_bg, &[]);
            pass.set_pipeline(&self.pipelines.detect_outliers);
            pass.dispatch_workgroups(particle_wg, 1, 1);
            pass.set_pipeline(&self.pipelines.detect_phasing);
            pass.dispatch_workgroups(particle_wg, 1, 1);
        }
        // Copy results to staging for CPU readback.
        let outlier_staging = self.outlier_staging.get_or_insert_with(|| {
            self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("outlier_staging"),
                size: 2048,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            })
        });
        let phasing_staging = self.phasing_staging.get_or_insert_with(|| {
            self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("phasing_staging"),
                size: 512,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            })
        });
        let outlier_count_staging = self.outlier_count_staging.get_or_insert_with(|| {
            self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("outlier_count_staging"),
                size: 4,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            })
        });
        let phasing_count_staging = self.phasing_count_staging.get_or_insert_with(|| {
            self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("phasing_count_staging"),
                size: 4,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            })
        });
        encoder.copy_buffer_to_buffer(&self.buffers.outlier_buf, 0, outlier_staging, 0, 2048);
        encoder.copy_buffer_to_buffer(&self.buffers.phasing_buf, 0, phasing_staging, 0, 512);
        encoder.copy_buffer_to_buffer(
            &self.buffers.outlier_count_buf,
            0,
            outlier_count_staging,
            0,
            4,
        );
        encoder.copy_buffer_to_buffer(
            &self.buffers.phasing_count_buf,
            0,
            phasing_count_staging,
            0,
            4,
        );
        // Copy GPU counters → staging for next frame's CPU readback.
        encoder.copy_buffer_to_buffer(
            &self.buffers.machine_counters,
            0,
            &self.counters_staging,
            0,
            u64::from(MAX_MACHINES) * 4,
        );
        self.queue.submit(std::iter::once(encoder.finish()));
    }

    fn read_outliers(&mut self, _frame: u32) -> (f32, usize) {
        let count_staging = self
            .outlier_count_staging
            .as_ref()
            .expect("outlier_count_staging missing");
        let count_slice = count_staging.slice(..);
        count_slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device.poll(wgpu::Maintain::Wait);
        let count_data = count_slice.get_mapped_range();
        let count = u32::from_le_bytes([count_data[0], count_data[1], count_data[2], count_data[3]])
            .min(MAX_OUTLIERS) as usize;
        drop(count_data);
        count_staging.unmap();

        if count == 0 {
            return (0.0, 0);
        }

        let staging = self
            .outlier_staging
            .as_ref()
            .expect("outlier_staging missing");
        let slice = staging.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device.poll(wgpu::Maintain::Wait);
        let data = slice.get_mapped_range();
        let words: &[u32] = bytemuck::cast_slice(&data[..]);
        let mut max_speed = 0.0f32;
        for k in 0..count {
            let base = k * 6;
            let speed = f32::from_bits(words[base + 5]);
            max_speed = max_speed.max(speed);
        }
        drop(data);
        staging.unmap();
        (max_speed, count)
    }

    fn read_phasing(&mut self, _frame: u32) -> usize {
        let count_staging = self
            .phasing_count_staging
            .as_ref()
            .expect("phasing_count_staging missing");
        let count_slice = count_staging.slice(..);
        count_slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device.poll(wgpu::Maintain::Wait);
        let count_data = count_slice.get_mapped_range();
        let count = u32::from_le_bytes([count_data[0], count_data[1], count_data[2], count_data[3]])
            .min(MAX_PHASING) as usize;
        drop(count_data);
        count_staging.unmap();
        count
    }

    fn render(&mut self) {
        const WARMUP_FRAMES: u32 = 10;
        const BENCH_DURATION_SECS: f64 = 10.0;

        self.update_machines();

        if self.test_bond_form || self.test_bond_constrain || self.test_bond_break {
            self.fps_tracker.begin_frame();
            self.device.poll(wgpu::Maintain::Wait);
            let null_machines = MachineParams::default();
            self.queue.write_buffer(
                &self.machine_params_buf,
                0,
                bytemuck::bytes_of(&null_machines),
            );
            if self.test_phase == 0 {
                self.test_setup();
            }
            self.simulate();
            self.test_phase += 1;
            if self.test_phase >= 5 && !self.test_report_done {
                self.test_verify();
                self.test_report_done = true;
                self.bench_done = true;
            }
        } else if self.test_paddle_stability || self.test_paddle_root_cause {
            // 10-second stability/root-cause test with active machines.
            if self.test_phase == 0 {
                self.test_setup();
            }
            self.test_phase += 1;
            self.update_machines();
            if self.bench_frame == WARMUP_FRAMES {
                self.bench_start = Some(Instant::now());
            }
            self.simulate();
            if self.bench_frame > WARMUP_FRAMES {
                let (frame_max, _n) = self.read_outliers(self.bench_frame);
                let phasing_count = self.read_phasing(self.bench_frame);
                if phasing_count > 0 {
                    eprintln!(
                        "  WARN frame {}: {} phasing events detected",
                        self.bench_frame, phasing_count
                    );
                }
                // Track max speed from GPU readback.
                if frame_max > self.max_speed_seen {
                    self.max_speed_seen = frame_max;
                }
            }
            self.bench_frame += 1;
            if self.bench_start.is_some() {
                let elapsed = self.bench_start.unwrap().elapsed().as_secs_f64();
                if elapsed >= BENCH_DURATION_SECS {
                    self.test_verify();
                    self.test_report_done = true;
                    self.bench_done = true;
                }
            }
        } else if self.benchmark {
            if self.bench_frame == 0 {
                self.spawn_all();
            }
            if self.bench_frame == WARMUP_FRAMES {
                self.bench_start = Some(Instant::now());
            }
            self.simulate();
            // Per-frame outlier + phasing detection during measurement window.
            if self.bench_frame > WARMUP_FRAMES {
                let (frame_max, _n) = self.read_outliers(self.bench_frame);
                let phasing_count = self.read_phasing(self.bench_frame);
                if phasing_count > 0 {
                    eprintln!(
                        "  FAIL frame {}: {} phasing events detected",
                        self.bench_frame, phasing_count
                    );
                }
                if frame_max > self.max_speed_seen {
                    self.max_speed_seen = frame_max;
                }
            }
        } else {
            self.fps_tracker.begin_frame();
            self.spawn();
            self.simulate();
        }

        if self.diagnose {
            if self.diag_frame.is_multiple_of(10) {
                self.read_diagnostics();
            }
            self.diag_frame += 1;
            if self.diag_frame >= 1200 {
                self.bench_done = true;
            }
        }

        let render_params = RenderParams {
            screen_width: self.surface_config.width as f32,
            screen_height: self.surface_config.height as f32,
            particle_radius: self.sim_params.particle_radius,
            particle_count: self.sim_params.particle_count,
        };
        self.queue.write_buffer(
            &self.render.params_buf,
            0,
            bytemuck::bytes_of(&render_params),
        );

        let Ok(frame) = self.surface.get_current_texture() else {
            self.surface.configure(&self.device, &self.surface_config);
            return;
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("scene"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.02,
                            g: 0.02,
                            b: 0.05,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });

            // Draw machines first (behind particles).
            let machine_count = self.machines.len() as u32 + PADDLE_COUNT;
            if machine_count > 0 {
                let mparams = MachineRenderParams {
                    screen_width: self.surface_config.width as f32,
                    screen_height: self.surface_config.height as f32,
                    machine_count,
                };
                self.queue.write_buffer(
                    &self.render.machine.params_buf,
                    0,
                    bytemuck::bytes_of(&mparams),
                );
                pass.set_pipeline(&self.render.machine.pipeline);
                pass.set_bind_group(0, &self.render.machine.bind_group, &[]);
                pass.draw(0..machine_count * 6, 0..1);
            }

            // Draw particles on top.
            if self.sim_params.particle_count > 0 {
                pass.set_pipeline(&self.render.pipeline);
                pass.set_bind_group(0, &self.render.bind_group, &[]);
                pass.draw(0..self.sim_params.particle_count * 6, 0..1);
            }
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();

        if self.benchmark {
            self.bench_frame += 1;
            if self.bench_frame >= WARMUP_FRAMES
                && self
                    .bench_start
                    .is_some_and(|s| s.elapsed().as_secs_f64() >= BENCH_DURATION_SECS)
            {
                self.device.poll(wgpu::Maintain::Wait);
                let elapsed = self.bench_start.unwrap().elapsed();
                let measured_frames = self.bench_frame - WARMUP_FRAMES;
                let count = self.sim_params.particle_count;
                let avg_ms = elapsed.as_secs_f64() * 1000.0 / f64::from(measured_frames);
                let frame_ok = avg_ms < 16.67;
                let stable_ok = self.verify_stability();
                let verdict = if frame_ok && stable_ok {
                    "PASS"
                } else {
                    "FAIL"
                };
                println!(
                    "Benchmark: {measured_frames} frames in {:.2}s, {count} particles",
                    elapsed.as_secs_f64()
                );
                println!("Average frame time: {avg_ms:.2}ms");
                println!("Result: {verdict}");
                self.bench_done = true;
            }
        } else if self.fps_tracker.end_frame() {
            self.window.set_title(&format!(
                "Particle Idle PoC | FPS: {:.0} | Particles: {} | Sim: {:.2}ms",
                self.fps_tracker.fps, self.sim_params.particle_count, self.fps_tracker.sim_time_ms,
            ));
        }
    }
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
