#![allow(clippy::unwrap_used)]

use std::sync::Arc;
use std::time::Instant;

use bytemuck::{Pod, Zeroable};
use rapier2d::prelude::*;
use winit::window::Window;

pub const WINDOW_WIDTH: u32 = 1280;
pub const WINDOW_HEIGHT: u32 = 720;
pub const MAX_PARTICLES: u32 = 100000;
pub const WORKGROUP_SIZE: u32 = 256;
// Phase 4: spawners replace old hopper system (SPAWNER_BATCH_SIZE + SPAWNER_INTERVAL).
pub const HOPPER_LEFT_X: f32 = -0.45;
pub const HOPPER_LEFT_HALF: f32 = 0.2;
pub const HOPPER_RIGHT_X: f32 = 0.45;
pub const HOPPER_RIGHT_HALF: f32 = 0.2;
pub const HOPPER_Y: f32 = 0.75;
pub const GRID_W: u32 = 256;
pub const GRID_H: u32 = 256;
pub const TOTAL_GRID_CELLS: u32 = GRID_W * GRID_H;
pub const SUB_STEPS: u32 = 16;
/// Fixed physics timestep — simulation always advances in 1/60s increments.
/// Per-substep dt = `FIXED_DT` / `SUB_STEPS` (GPU and Rapier both use this).
pub const FIXED_DT: f32 = 1.0 / 60.0;
/// Max particles that can be killed in a single frame (ring buffer size for GPU→CPU swap-remove).
pub const MAX_KILLS_PER_FRAME: u32 = 4096;

pub const MAX_SPECIES: u32 = 8;
pub const MAX_MACHINES: u32 = 16;
pub const PADDLE_COUNT: u32 = 10;
pub const CAPSULE_HALF_LEN: f32 = 0.22;
pub const CAPSULE_RADIUS: f32 = 0.055;
pub const CONVEYOR_ANGLE_DEG: f32 = 45.0;
pub const PADDLE_HW: f32 = 0.012;
pub const PADDLE_HH: f32 = 0.035;
pub const CONVEYOR_SPEED: f32 = 0.45;
pub const SENSOR_HALF: f32 = 0.06;

// Polymer bond constants for Green(2) particles.
// Green particles form softbody-style mesh bonds (max 4 per particle) that break under stress.
// Mirror copy of WGSL constants in form_bonds.wgsl and solve_bonds.wgsl.
#[allow(dead_code)]
pub const GREEN_SPECIES: u32 = 2;
pub const INVALID_BOND: u32 = 0xFFFF_FFFF;
#[allow(dead_code)]
pub const MAX_BONDS_PER_PARTICLE: u32 = 4;
#[allow(dead_code)]
pub const BOND_FORMATION_MULTIPLIER: f32 = 3.0;
#[allow(dead_code)]
pub const BOND_BREAK_MULTIPLIER: f32 = 5.0;
#[allow(dead_code)]
pub const BOND_COMPLIANCE: f32 = 0.04;

pub const MAX_OUTLIERS: u32 = 64;
pub const MAX_PHASING: u32 = 32;

// Phase 4: Spawner + SDF + Kill barrier constants
pub const SPAWNER_BATCH_SIZE: u32 = 1000;
pub const SPAWNER_INTERVAL: f32 = 60.0;
pub const SDF_RES: u32 = 256;
pub const KILL_Y: f32 = -0.8;
pub const BRUSH_RADIUS: f32 = 0.15;

/// Convert a binary occupancy grid (negative = wall, positive = free)
/// into a signed distance field with distances in texel units.
/// Uses BFS from boundary cells for approximate EDT.
/// Positive = free space distance to wall, negative = depth inside wall.
pub fn compute_distance_field(grid: &mut [f32], res: u32) {
    use std::collections::VecDeque;

    let n = (res * res) as usize;
    let mut dist = vec![f32::MAX; n];
    let mut queue = VecDeque::with_capacity(n / 4);
    let res_i = res as i32;

    // Seed boundary cells: any cell adjacent to a cell of opposite sign.
    for y in 0..res_i {
        for x in 0..res_i {
            let idx = (y as u32 * res + x as u32) as usize;
            let inside = grid[idx] < 0.0;
            let mut on_boundary = false;
            for &(dx, dy) in &[(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
                let nx = x + dx;
                let ny = y + dy;
                if nx >= 0 && nx < res_i && ny >= 0 && ny < res_i {
                    let ni = (ny as u32 * res + nx as u32) as usize;
                    if (grid[ni] < 0.0) != inside {
                        on_boundary = true;
                        break;
                    }
                }
            }
            if on_boundary {
                dist[idx] = 0.5;
                queue.push_back((x as u32, y as u32));
            }
        }
    }

    // BFS propagation with 8-connectivity.
    while let Some((x, y)) = queue.pop_front() {
        let idx = (y * res + x) as usize;
        let d = dist[idx];
        for &(dx, dy) in &[
            (1i32, 0i32),
            (-1, 0),
            (0, 1),
            (0, -1),
            (1, 1),
            (1, -1),
            (-1, 1),
            (-1, -1),
        ] {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx < 0 || nx >= res_i || ny < 0 || ny >= res_i {
                continue;
            }
            let ni = (ny as u32 * res + nx as u32) as usize;
            let step = if dx.abs() + dy.abs() > 1 {
                std::f32::consts::SQRT_2
            } else {
                1.0
            };
            let nd = d + step;
            if nd < dist[ni] {
                dist[ni] = nd;
                queue.push_back((nx as u32, ny as u32));
            }
        }
    }

    // Apply sign: negative inside wall, positive in free space.
    for i in 0..n {
        let sign = if grid[i] < 0.0 { -1.0 } else { 1.0 };
        grid[i] = sign * dist[i];
    }
}

/// Compute distance field on a copy of the binary grid, returning the
/// SDF-ready data for GPU upload. Leaves the original grid unchanged
/// so further painting works on binary values.
pub fn sdf_for_upload(grid: &[f32], res: u32) -> Vec<f32> {
    let mut copy = grid.to_vec();
    compute_distance_field(&mut copy, res);
    copy
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct BondSlot {
    pub partner: u32,
    pub rest: f32,
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
pub struct ReactionMatrix {
    // Dense 8×8 flat array: result[species_A * MAX_SPECIES + species_B].
    // Row/col 0 = Red, 1 = Blue, 2 = Green, 3-7 = reserved.
    pub results: [u32; (MAX_SPECIES * MAX_SPECIES) as usize],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct SimParams {
    pub particle_count: u32,
    pub dt: f32,
    pub gravity: f32,
    pub particle_radius: f32,
    pub wall_min_x: f32,
    pub wall_min_y: f32,
    pub wall_max_x: f32,
    pub wall_max_y: f32,
    pub friction_mu: f32,
    pub grid_cell_size: f32,
    pub grid_width: u32,
    pub grid_height: u32,
    pub disable_velocity_cap: u32,
    pub sub_steps: u32,
    pub kill_y: f32,
    pub _pad_final: u32,  // wgpu 24.x uniform alignment
    pub _pad_final2: u32, // wgpu 24.x uniform alignment
}

#[repr(u32)]
#[derive(Copy, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum MachineKind {
    Conveyor = 0,
    Grinder = 1,
    Heater = 2,
    Paddle = 3,
    Spawner = 4,
}

// GPU-compatible machine uniform (must match WGSL Machine struct layout).
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct GpuMachine {
    pub pos_x: f32,
    pub pos_y: f32,
    pub cos_angle: f32,
    pub sin_angle: f32,
    pub half_width: f32,
    pub half_height: f32,
    pub kind: u32,
    pub input_species: u32,
    pub output_species: u32,
    pub angular_velocity: f32,
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
pub struct MachineParams {
    pub count: u32,
    #[allow(clippy::pub_underscore_fields)]
    pub _pad: [u32; 3],
    pub machines: [GpuMachine; MAX_MACHINES as usize],
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
pub struct Recipe {
    pub input_species: u32,
    pub input_count: u32,
    pub output_species: u32,
    pub output_count: u32,
    pub cycle_time: f32,
}

// CPU-side machine state — buffer tracking, cycle progression, brightness.
pub struct MachineCpuState {
    pub recipe: Recipe,
    pub input_accumulated: u32,
    pub cycles_completed: u32,
    pub cycle_timer: f32,
    pub consumed_this_frame: u32,
    pub color_base: [f32; 3],
}

// Machine definition on CPU side (Rapier2D handles for physics).
pub struct MachineDef {
    pub kind: MachineKind,
    pub body_handle: RigidBodyHandle,
    pub input_species: u32,
    pub output_species: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct RenderParams {
    pub screen_width: f32,
    pub screen_height: f32,
    pub particle_radius: f32,
    pub particle_count: u32,
    pub cursor_x: f32,
    pub cursor_y: f32,
    pub cursor_radius: f32,
    pub cursor_active: u32,
}

// GPU-compatible machine render data (must match WGSL MachineRender struct).
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct GpuMachineRender {
    pub pos_x: f32,
    pub pos_y: f32,
    pub cos_angle: f32,
    pub sin_angle: f32,
    pub half_width: f32,
    pub half_height: f32,
    pub color_r: f32,
    pub color_g: f32,
    pub color_b: f32,
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
pub struct MachineRenderParams {
    pub screen_width: f32,
    pub screen_height: f32,
    pub machine_count: u32,
}

// Phase 4: SDF wall params — matched by WGSL SdfParams struct.
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct SdfParams {
    pub resolution: u32,
    pub world_min_x: f32,
    pub world_min_y: f32,
    pub world_max_x: f32,
    pub world_max_y: f32,
}

// SDF visualization overlay params (matched by sdf_viz.wgsl).
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct SdfVizParams {
    pub screen_width: f32,
    pub screen_height: f32,
    pub world_min_x: f32,
    pub world_min_y: f32,
    pub world_max_x: f32,
    pub world_max_y: f32,
}

// Phase 4: Conveyor endpoint state for drag interaction.
#[derive(Copy, Clone)]
pub struct ConveyorEndpoints {
    pub endpoint_a: [f32; 2],
    pub endpoint_b: [f32; 2],
}

impl Default for ConveyorEndpoints {
    fn default() -> Self {
        let angle = CONVEYOR_ANGLE_DEG.to_radians();
        let (s, c) = (angle.sin(), angle.cos());
        let pivot = [0.0f32, -0.22f32];
        Self {
            endpoint_a: [
                pivot[0] - c * CAPSULE_HALF_LEN,
                pivot[1] - s * CAPSULE_HALF_LEN,
            ],
            endpoint_b: [
                pivot[0] + c * CAPSULE_HALF_LEN,
                pivot[1] + s * CAPSULE_HALF_LEN,
            ],
        }
    }
}

// Phase 4: Input modes.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Mode {
    Drag,
    Draw,
}

pub fn parse_flag_arg(name: &str, default: u32) -> u32 {
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

pub struct Buffers {
    pub positions: wgpu::Buffer,
    pub velocities: wgpu::Buffer,
    pub forces: wgpu::Buffer,
    pub prev_positions: wgpu::Buffer,
    pub species: wgpu::Buffer,
    pub params: wgpu::Buffer,
    pub cell_indices: wgpu::Buffer,
    pub cell_counts: wgpu::Buffer,
    pub cell_offsets: wgpu::Buffer,
    pub sorted_indices: wgpu::Buffer,
    pub morton_keys: wgpu::Buffer,
    pub reaction_matrix: wgpu::Buffer,
    // Polymer bond data: flat array, MAX_PARTICLES * 4 slots.
    // Particle i owns slots [i*4+0, i*4+1, i*4+2, i*4+3].
    pub bonds: wgpu::Buffer,
    // Per-machine atomic counter: GPU-side consumption tracking.
    pub machine_counters: wgpu::Buffer,
    // GPU outlier/phasing detection buffers.
    pub outlier_buf: wgpu::Buffer,
    pub outlier_count_buf: wgpu::Buffer,
    pub phasing_buf: wgpu::Buffer,
    pub phasing_count_buf: wgpu::Buffer,
    // Phase 4: SDF wall texture + params.
    pub sdf_tex: wgpu::Texture,
    pub sdf_params_buf: wgpu::Buffer,
    // Kill barrier: atomic counter + ring buffer (GPU writes killed indices, CPU swap-removes).
    pub kill_count_buf: wgpu::Buffer,
    pub kill_indices_buf: wgpu::Buffer,
}

pub struct ComputePipelines {
    pub predict: wgpu::ComputePipeline,
    pub clear_cells: wgpu::ComputePipeline,
    pub prefix_scan: wgpu::ComputePipeline,
    pub project: wgpu::ComputePipeline,
    pub apply: wgpu::ComputePipeline,
    pub reaction: wgpu::ComputePipeline,
    pub morton_keys: wgpu::ComputePipeline,
    pub morton_count: wgpu::ComputePipeline,
    pub morton_scatter: wgpu::ComputePipeline,
    pub form_bonds: wgpu::ComputePipeline,
    pub form_bonds_resolve: wgpu::ComputePipeline,
    pub solve_bonds: wgpu::ComputePipeline,
    pub detect_outliers: wgpu::ComputePipeline,
    pub detect_phasing: wgpu::ComputePipeline,
}

pub struct MachineRenderState {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,
    pub params_buf: wgpu::Buffer,
    pub data_buf: wgpu::Buffer,
}

pub struct SdfVizState {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,
    pub params_buf: wgpu::Buffer,
}

pub struct RenderState {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,
    pub params_buf: wgpu::Buffer,
    pub machine: MachineRenderState,
}

pub struct RapierState {
    pub pipeline: PhysicsPipeline,
    pub integration_parameters: IntegrationParameters,
    pub island_manager: IslandManager,
    pub broad_phase: DefaultBroadPhase,
    pub narrow_phase: NarrowPhase,
    pub bodies: RigidBodySet,
    pub colliders: ColliderSet,
    pub impulse_joints: ImpulseJointSet,
    pub multibody_joints: MultibodyJointSet,
    pub ccd_solver: CCDSolver,
}

pub mod capture;
pub mod state;

#[allow(clippy::struct_excessive_bools)]
pub struct State {
    pub window: Arc<Window>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub buffers: Buffers,
    pub particle_bg: wgpu::BindGroup,
    pub grid_bg: wgpu::BindGroup,
    pub kill_bg: wgpu::BindGroup,
    pub pipelines: ComputePipelines,
    pub render: RenderState,
    pub sdf_viz: SdfVizState,
    pub detection_bg: wgpu::BindGroup,
    pub sim_params: SimParams,
    pub fps_tracker: FpsTracker,
    pub benchmark: bool,
    pub bench_frame: u32,
    pub bench_start: Option<Instant>,
    pub bench_done: bool,
    pub test_bond_form: bool,
    pub test_bond_constrain: bool,
    pub test_bond_break: bool,
    pub test_paddle_stability: bool,
    pub test_paddle_root_cause: bool,
    pub test_sdf_bowl: bool,
    pub test_phase: u32,
    pub test_report_done: bool,
    pub diagnose: bool,
    pub diag_frame: u32,
    pub diag_staging: Option<wgpu::Buffer>,
    pub rapier: RapierState,
    pub machines: Vec<MachineDef>,
    pub machines_cpu: Vec<MachineCpuState>,
    pub machine_params_buf: wgpu::Buffer,
    pub counters_staging: wgpu::Buffer,
    pub machine_time: f32,
    pub num_particles: u32,
    pub sub_steps: u32,
    pub max_speed_seen: f32,
    pub outlier_staging: Option<wgpu::Buffer>,
    pub phasing_staging: Option<wgpu::Buffer>,
    pub outlier_count_staging: Option<wgpu::Buffer>,
    pub phasing_count_staging: Option<wgpu::Buffer>,
    // Phase 4: SDF walls.
    pub sdf_grid: Vec<f32>,
    pub sdf_dirty: bool,
    // Phase 4: Conveyor endpoints.
    pub conveyor_endpoints: ConveyorEndpoints,
    // Phase 4: Input mode + drag state.
    pub mode: Mode,
    pub mouse_world: [f32; 2],
    pub dragging: Option<usize>,       // machine index being dragged
    pub dragging_endpoint: Option<u8>, // 0 = endpoint_a, 1 = endpoint_b
    pub draw_erase: bool,              // true = erasing, false = painting in Draw mode
    pub brush_radius: f32,             // current brush radius (adjusted by scroll)
    pub last_frame: Option<Instant>,   // real wall-clock frame time
    // Phase 4: Spawner timers (per-spawner Instant for wall-clock batching).
    pub spawner_next_fire: Vec<Instant>,
    pub no_benchmark: bool,
    /// Fixed-timestep accumulator — when >= `FIXED_DT`, we step physics.
    pub physics_accumulator: f32,
    /// Staging buffers for kill barrier GPU→CPU readback.
    pub kill_count_staging: wgpu::Buffer,
    pub kill_indices_staging: wgpu::Buffer,
    /// Scratch buffer for swap-remove: holds one particle's data (pos+vel+prev = 24 bytes).
    pub swap_scratch: wgpu::Buffer,
    pub kill_data_pending: bool,
}

pub struct FpsTracker {
    last_fps_update: Instant,
    frame_count: u32,
    fps: f32,
    sim_time_ms: f32,
    pub last_frame: Instant,
}

impl Default for FpsTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl FpsTracker {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            last_fps_update: now,
            frame_count: 0,
            fps: 0.0,
            sim_time_ms: 0.0,
            last_frame: now,
        }
    }

    pub fn begin_frame(&mut self) {
        self.last_frame = Instant::now();
    }

    pub fn end_frame(&mut self) -> bool {
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

pub fn sdf_texture_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: false },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

pub fn create_buffers(device: &wgpu::Device) -> Buffers {
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
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
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
        sdf_tex: device.create_texture(&wgpu::TextureDescriptor {
            label: Some("sdf_tex"),
            size: wgpu::Extent3d {
                width: SDF_RES,
                height: SDF_RES,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        }),
        sdf_params_buf: device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("sdf_params"),
            size: size_of::<SdfParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }),
        kill_count_buf: device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("kill_count"),
            size: 4,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        }),
        kill_indices_buf: device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("kill_indices"),
            size: u64::from(MAX_KILLS_PER_FRAME) * 4,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        }),
    }
}

pub fn storage_entry(binding: u32, read_only: bool) -> wgpu::BindGroupLayoutEntry {
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

pub fn uniform_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
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

pub fn create_particle_bgl(device: &wgpu::Device) -> wgpu::BindGroupLayout {
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
            sdf_texture_entry(11),    // sdf_tex (float texture, read-only)
            uniform_entry(12),        // sdf_params
        ],
    })
}

pub fn create_detection_bgl(device: &wgpu::Device) -> wgpu::BindGroupLayout {
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

pub fn create_grid_bgl(device: &wgpu::Device) -> wgpu::BindGroupLayout {
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

pub fn create_kill_bgl(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("kill_bgl"),
        entries: &[
            storage_entry(0, false), // kill_count (atomic<u32>)
            storage_entry(1, false), // kill_indices (array<u32>)
        ],
    })
}

pub fn create_bind_groups(
    device: &wgpu::Device,
    buffers: &Buffers,
    machine_params_buf: &wgpu::Buffer,
    particle_bgl: &wgpu::BindGroupLayout,
    grid_bgl: &wgpu::BindGroupLayout,
    kill_bgl: &wgpu::BindGroupLayout,
    sdf_tex_view: &wgpu::TextureView,
) -> (wgpu::BindGroup, wgpu::BindGroup, wgpu::BindGroup) {
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
            wgpu::BindGroupEntry {
                binding: 11,
                resource: wgpu::BindingResource::TextureView(sdf_tex_view),
            },
            wgpu::BindGroupEntry {
                binding: 12,
                resource: buffers.sdf_params_buf.as_entire_binding(),
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
    let kill_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("kill_bg"),
        layout: kill_bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: buffers.kill_count_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: buffers.kill_indices_buf.as_entire_binding(),
            },
        ],
    });
    (particle_bg, grid_bg, kill_bg)
}

pub fn make_compute_pipeline(
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

pub fn create_detection_bg(
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

pub fn create_pipelines(
    device: &wgpu::Device,
    particle_bgl: &wgpu::BindGroupLayout,
    grid_bgl: &wgpu::BindGroupLayout,
    kill_bgl: &wgpu::BindGroupLayout,
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
    let apply_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("apply_pl"),
        bind_group_layouts: &[particle_bgl, grid_bgl, kill_bgl],
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
        apply: make_compute_pipeline(device, &apply_layout, &project_shader, "apply", "apply"),
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

pub fn create_render_state(
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

pub fn create_sdf_viz_state(
    device: &wgpu::Device,
    sdf_tex_view: &wgpu::TextureView,
    format: wgpu::TextureFormat,
) -> SdfVizState {
    let params_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("sdf_viz_params"),
        size: size_of::<SdfVizParams>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("sdf_viz_bgl"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
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
        label: Some("sdf_viz_bg"),
        layout: &bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(sdf_tex_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: params_buf.as_entire_binding(),
            },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("sdf_viz_pl"),
        bind_group_layouts: &[&bgl],
        push_constant_ranges: &[],
    });

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("sdf_viz"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/sdf_viz.wgsl").into()),
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("sdf_viz"),
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

    SdfVizState {
        pipeline,
        bind_group,
        params_buf,
    }
}

pub fn init_machines(
    mut rapier: RapierState,
    endpoints: &ConveyorEndpoints,
) -> (Vec<MachineDef>, Vec<MachineCpuState>, RapierState) {
    let mut machines = Vec::with_capacity(6); // conveyor + grinder + heater + 2 spawners
    let mut cpu_states = Vec::with_capacity(6);

    // Conveyor: capsule-shaped belt defined by two draggable endpoints.
    // Paddles ride along the perimeter, computed in update_machines().
    // The body is a kinematic OBB — particles collide with the full capsule
    // interior so they can't pass through.
    let ea = endpoints.endpoint_a;
    let eb = endpoints.endpoint_b;
    let pivot = [(ea[0] + eb[0]) * 0.5, (ea[1] + eb[1]) * 0.5];
    let dx = eb[0] - ea[0];
    let dy = eb[1] - ea[1];
    let conv_angle = dy.atan2(dx);
    let conv_half = 0.5 * (dx * dx + dy * dy).sqrt();
    let conv_body_hw = CAPSULE_RADIUS; // perpendicular to belt
    let conv_body_hh = conv_half; // along belt
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

    // Spawner (Red, species 0) — at left hopper position.
    let spawner_half = 0.06;
    {
        let body = RigidBodyBuilder::fixed()
            .translation(Vec2::new(HOPPER_LEFT_X, HOPPER_Y))
            .build();
        let body_handle = rapier.bodies.insert(body);
        rapier.colliders.insert_with_parent(
            ColliderBuilder::cuboid(spawner_half, spawner_half)
                .sensor(true)
                .build(),
            body_handle,
            &mut rapier.bodies,
        );
        machines.push(MachineDef {
            kind: MachineKind::Spawner,
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
                cycle_time: SPAWNER_INTERVAL,
            },
            input_accumulated: 0,
            cycles_completed: 0,
            cycle_timer: 0.0,
            consumed_this_frame: 0,
            color_base: [0.85, 0.15, 0.15], // Red-tinted border
        });
    }

    // Spawner (Blue, species 1) — at right hopper position.
    {
        let body = RigidBodyBuilder::fixed()
            .translation(Vec2::new(HOPPER_RIGHT_X, HOPPER_Y))
            .build();
        let body_handle = rapier.bodies.insert(body);
        rapier.colliders.insert_with_parent(
            ColliderBuilder::cuboid(spawner_half, spawner_half)
                .sensor(true)
                .build(),
            body_handle,
            &mut rapier.bodies,
        );
        machines.push(MachineDef {
            kind: MachineKind::Spawner,
            body_handle,
            input_species: 1,
            output_species: 1,
        });
        cpu_states.push(MachineCpuState {
            recipe: Recipe {
                input_species: 1,
                input_count: 0,
                output_species: 1,
                output_count: 0,
                cycle_time: SPAWNER_INTERVAL,
            },
            input_accumulated: 0,
            cycles_completed: 0,
            cycle_timer: 0.0,
            consumed_this_frame: 0,
            color_base: [0.15, 0.15, 0.85], // Blue-tinted border
        });
    }

    (machines, cpu_states, rapier)
}
