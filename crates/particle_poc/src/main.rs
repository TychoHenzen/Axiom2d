#![allow(clippy::unwrap_used)]

use std::sync::Arc;
use std::time::Instant;

use bytemuck::{Pod, Zeroable};
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
const CONVEYOR_PIVOT_X: f32 = 0.0;
const CONVEYOR_PIVOT_Y: f32 = -0.5;
const GRID_W: u32 = 256;
const GRID_H: u32 = 256;
const TOTAL_GRID_CELLS: u32 = GRID_W * GRID_H;
// 8 substeps of dt=1/480 = exactly 1/60s per frame.
const SUB_STEPS: u32 = 16;

const MAX_SPECIES: u32 = 8;

// Polymer bond constants for Green(2) particles.
// Green particles form chain-like bonds (max 2 per particle) that break under stress.
// Mirror copy of WGSL constants in form_bonds.wgsl and solve_bonds.wgsl.
#[allow(dead_code)]
const GREEN_SPECIES: u32 = 2;
const INVALID_BOND: u32 = 0xFFFF_FFFF;
#[allow(dead_code)]
const MAX_BONDS_PER_PARTICLE: u32 = 2;
#[allow(dead_code)]
const BOND_FORMATION_MULTIPLIER: f32 = 3.0;
#[allow(dead_code)]
const BOND_BREAK_MULTIPLIER: f32 = 3.0;
#[allow(dead_code)]
const BOND_COMPLIANCE: f32 = 0.15;

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
    _pad: [u32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct ConveyorParams {
    pivot_x: f32,
    pivot_y: f32,
    cos_angle: f32,
    sin_angle: f32,
    half_width: f32,
    half_height: f32,
    angular_velocity: f32,
    _pad: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct RenderParams {
    screen_width: f32,
    screen_height: f32,
    particle_radius: f32,
    particle_count: u32,
}

fn parse_flag_arg(name: &str, default: u32) -> u32 {
    let mut args = std::env::args();
    while let Some(arg) = args.next() {
        if arg == name {
            if let Some(val) = args.next() {
                if let Ok(n) = val.parse::<u32>() {
                    return n;
                }
            }
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
        num_particles,
        sub_steps,
    };
    event_loop.run_app(&mut app).expect("run");
}

struct App {
    state: Option<State>,
    benchmark: bool,
    diagnose: bool,
    test_bond_form: bool,
    test_bond_constrain: bool,
    test_bond_break: bool,
    num_particles: u32,
    sub_steps: u32,
}

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
    // Polymer bond data (per-particle, max 2 bonds each, packed as BondSlot).
    bond_slot_a: wgpu::Buffer,
    bond_slot_b: wgpu::Buffer,
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
}

struct RenderState {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    params_buf: wgpu::Buffer,
}

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
    sim_params: SimParams,
    fps_tracker: FpsTracker,
    benchmark: bool,
    bench_frame: u32,
    bench_start: Option<Instant>,
    bench_done: bool,
    test_bond_form: bool,
    test_bond_constrain: bool,
    test_bond_break: bool,
    test_phase: u32,
    test_report_done: bool,
    diagnose: bool,
    diag_frame: u32,
    diag_staging: Option<wgpu::Buffer>,
    conveyor_time: f32,
    conveyor_buf: wgpu::Buffer,
    num_particles: u32,
    sub_steps: u32,
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
    let grid_storage = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC;
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
        bond_slot_a: device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("bond_slot_a"),
            size: u64::from(MAX_PARTICLES) * size_of::<BondSlot>() as u64,
            usage: grid_storage,
            mapped_at_creation: false,
        }),
        bond_slot_b: device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("bond_slot_b"),
            size: u64::from(MAX_PARTICLES) * size_of::<BondSlot>() as u64,
            usage: grid_storage,
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
            uniform_entry(8),         // conveyor
            storage_entry(9, true),   // reaction_matrix (read-only storage)
            storage_entry(10, false), // bond_slot_a
            storage_entry(11, false), // bond_slot_b
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
        ],
    })
}

fn create_bind_groups(
    device: &wgpu::Device,
    buffers: &Buffers,
    conveyor_buf: &wgpu::Buffer,
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
                resource: conveyor_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 9,
                resource: buffers.reaction_matrix.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 10,
                resource: buffers.bond_slot_a.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 11,
                resource: buffers.bond_slot_b.as_entire_binding(),
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

fn create_pipelines(
    device: &wgpu::Device,
    particle_bgl: &wgpu::BindGroupLayout,
    grid_bgl: &wgpu::BindGroupLayout,
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
    }
}

impl State {
    fn new(window: Arc<Window>, benchmark: bool, diagnose: bool, test_bond_form: bool, test_bond_constrain: bool, test_bond_break: bool, num_particles: u32, sub_steps: u32) -> Self {
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

        let conveyor_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("conveyor_params"),
            size: size_of::<ConveyorParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let (particle_bg, grid_bg) =
            create_bind_groups(&device, &buffers, &conveyor_buf, &particle_bgl, &grid_bgl);
        let pipelines = create_pipelines(&device, &particle_bgl, &grid_bgl);
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
            _pad: [0; 2],
        };

        // Initialize reaction matrix: Red(0)+Blue(1) → Green(2), symmetric.
        let mut reaction_matrix = ReactionMatrix {
            results: [0u32; (MAX_SPECIES * MAX_SPECIES) as usize],
        };
        reaction_matrix.results[(0 * MAX_SPECIES + 1) as usize] = 2; // Red + Blue → Green
        reaction_matrix.results[(1 * MAX_SPECIES + 0) as usize] = 2; // Blue + Red → Green
        queue.write_buffer(
            &buffers.reaction_matrix,
            0,
            bytemuck::bytes_of(&reaction_matrix),
        );

        // Initialize bond buffers: all slots invalid.
        {
            let invalid_bonds = vec![BondSlot::default(); MAX_PARTICLES as usize];
            queue.write_buffer(
                &buffers.bond_slot_a,
                0,
                bytemuck::cast_slice(&invalid_bonds),
            );
            queue.write_buffer(
                &buffers.bond_slot_b,
                0,
                bytemuck::cast_slice(&invalid_bonds),
            );
        }

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
            sim_params,
            fps_tracker: FpsTracker::new(),
            benchmark,
            bench_frame: 0,
            bench_start: None,
            bench_done: false,
            test_bond_form,
            test_bond_constrain,
            test_bond_break,
            test_phase: 0,
            test_report_done: false,
            diagnose,
            diag_frame: 0,
            diag_staging: None,
            conveyor_time: 0.0,
            conveyor_buf,
            num_particles,
            sub_steps,
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
        encoder.copy_buffer_to_buffer(
            &self.buffers.velocities,
            0,
            &staging,
            pos_bytes,
            vel_bytes,
        );
        self.queue.submit(std::iter::once(encoder.finish()));
        let slice = staging.slice(..);
        slice.map_async(wgpu::MapMode::Read, |r| r.expect("map stability staging"));
        self.device.poll(wgpu::Maintain::Wait);
        let data = slice.get_mapped_range();
        let positions: &[[f32; 2]] = bytemuck::cast_slice(&data[..n * 8]);
        let vel_offset = pos_bytes as usize;
        let velocities: &[[f32; 2]] =
            bytemuck::cast_slice(&data[vel_offset..vel_offset + n * 8]);

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
        let variance = kes.iter().map(|k| (k - mean_ke) * (k - mean_ke)).sum::<f32>()
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
            outlier_list.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            for (idx, (i, ke)) in outlier_list.iter().enumerate().take(30) {
                let slot_a = bonds[*i];
                let slot_b = bonds[n + *i];
                let [px, py] = positions[*i];
                let [vx, vy] = velocities[*i];
                let sp = species[*i];
                let is_bonded = slot_a.partner != INVALID_BOND || slot_b.partner != INVALID_BOND;
                println!(
                    "  outlier[{}] sp={} pos=({:.4},{:.4}) vel=({:.4},{:.4}) \
                     KE={:.6} bond_a=(p={} r={:.4}) bond_b=(p={} r={:.4}) {}",
                    i, sp, px, py, vx, vy, *ke,
                    slot_a.partner, slot_a.rest,
                    slot_b.partner, slot_b.rest,
                    if is_bonded { "(bonded)" } else { "(free)" },
                );
            }
            if ke_outliers > 30 {
                println!("  ... and {} more KE outliers (top 30 shown, {green_outliers} green)", ke_outliers - 30);
            }
        }

        drop(sp_data);
        sp_staging.unmap();

        let nan_ok = nan_count == 0;
        let oob_ok = oob == 0;
        let ke_ok = ke_outliers == 0;
        let speed_ok = max_speed < 2.0;
        let all_ok = nan_ok && oob_ok && ke_ok && speed_ok;

        println!(
            "Stability: n={n} nan={nan_count} oob={oob} \
             ke_outliers={ke_outliers}/{n} (green={green_outliers} median={median_ke:.6} σ={stddev_ke:.6} thresh={outlier_threshold:.6}) \
             vmax={max_speed:.3} — {}",
            if all_ok { "PASS" } else { "FAIL" }
        );
        if !nan_ok {
            println!("  FAIL: {nan_count} NaN/infinite values");
        }
        if !oob_ok {
            println!("  FAIL: {oob} out-of-bounds particles");
        }
        if !ke_ok {
            println!("  FAIL: {ke_outliers} KE outliers > median+5σ (must be 0, {green_outliers} green)");
        }
        if !speed_ok {
            println!("  FAIL: max_speed={max_speed:.3} >= 2.0");
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

    fn update_conveyor(&mut self) {
        let dt = 1.0 / 60.0;
        self.conveyor_time += dt;

        // Slow spin: ~0.6 rad/s, one full rotation every ~10.5 seconds.
        let angular_velocity = 0.6;
        let angle = self.conveyor_time * angular_velocity;

        let pivot_x = CONVEYOR_PIVOT_X;
        let pivot_y = CONVEYOR_PIVOT_Y;
        let hw = 0.2;
        let hh = 0.007;

        let conveyor_params = ConveyorParams {
            pivot_x,
            pivot_y,
            cos_angle: angle.cos(),
            sin_angle: angle.sin(),
            half_width: hw,
            half_height: hh,
            angular_velocity,
            _pad: 0,
        };
        self.queue
            .write_buffer(&self.conveyor_buf, 0, bytemuck::bytes_of(&conveyor_params));
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

        self.sim_params.gravity = 0.0;

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
                    view[(pos_bytes as usize + vel_bytes as usize)..
                        (pos_bytes as usize + vel_bytes as usize) + 8]
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
                    view[(pos_bytes as usize + vel_bytes as usize)..
                        (pos_bytes as usize + vel_bytes as usize) + 8]
                        .copy_from_slice(bytemuck::cast_slice(&sp_data));
                }
                staging.unmap();

                self.sim_params.particle_count = 2;

                // Clear all bond slots, then write mutual bond: 0↔1 via slot_a.
                let invalid = vec![BondSlot::default(); n];
                self.queue.write_buffer(
                    &self.buffers.bond_slot_a,
                    0,
                    bytemuck::cast_slice(&invalid),
                );
                self.queue.write_buffer(
                    &self.buffers.bond_slot_b,
                    0,
                    bytemuck::cast_slice(&invalid),
                );
                let mut bonds_a = vec![BondSlot::default(); n];
                bonds_a[0] = BondSlot {
                    partner: 1,
                    rest: 0.03,
                };
                bonds_a[1] = BondSlot {
                    partner: 0,
                    rest: 0.03,
                };
                self.queue.write_buffer(
                    &self.buffers.bond_slot_a,
                    0,
                    bytemuck::cast_slice(&bonds_a),
                );
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
        let slot_bytes = (n * size_of::<BondSlot>()) as u64;
        let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("bond_readback_staging"),
            size: slot_bytes * 2,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("bond_readback"),
            });
        encoder.copy_buffer_to_buffer(&self.buffers.bond_slot_a, 0, &staging, 0, slot_bytes);
        encoder.copy_buffer_to_buffer(
            &self.buffers.bond_slot_b,
            0,
            &staging,
            slot_bytes,
            slot_bytes,
        );
        self.queue.submit(std::iter::once(encoder.finish()));
        let slice = staging.slice(..);
        slice.map_async(wgpu::MapMode::Read, |r| r.expect("map bond staging"));
        self.device.poll(wgpu::Maintain::Wait);
        let data = slice.get_mapped_range();
        let bonds_a: &[BondSlot] = bytemuck::cast_slice(&data[..slot_bytes as usize]);
        let bonds_b: &[BondSlot] =
            bytemuck::cast_slice(&data[slot_bytes as usize..2 * slot_bytes as usize]);
        let mut all = Vec::with_capacity(2 * n);
        all.extend_from_slice(bonds_a);
        all.extend_from_slice(bonds_b);
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
                println!(
                    "test-bond-constrain: PASS (dist {:.4} -> {:.4})",
                    initial_dist, current_dist
                );
            } else {
                eprintln!(
                    "test-bond-constrain: FAIL — distance not reduced ({:.4})",
                    current_dist
                );
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
        }
    }

    fn simulate(&mut self) {
        self.queue.write_buffer(
            &self.buffers.params,
            0,
            bytemuck::bytes_of(&self.sim_params),
        );

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
        self.queue.submit(std::iter::once(encoder.finish()));
    }

    fn render(&mut self) {
        const WARMUP_FRAMES: u32 = 10;
        const BENCH_DURATION_SECS: f64 = 10.0;

        self.update_conveyor();

        let any_test =
            self.test_bond_form || self.test_bond_constrain || self.test_bond_break;

        if any_test {
            self.fps_tracker.begin_frame();
            // Ensure previous frame's GPU work is complete before modifying buffers.
            self.device.poll(wgpu::Maintain::Wait);
            // Disable conveyor for small bond tests — move far from particles.
            let null_conveyor = ConveyorParams {
                pivot_x: 100.0,
                pivot_y: 100.0,
                cos_angle: 1.0,
                sin_angle: 0.0,
                half_width: 0.0,
                half_height: 0.0,
                angular_velocity: 0.0,
                _pad: 0,
            };
            self.queue.write_buffer(
                &self.conveyor_buf,
                0,
                bytemuck::bytes_of(&null_conveyor),
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
        } else if self.benchmark {
            if self.bench_frame == 0 {
                self.spawn_all();
            }
            if self.bench_frame == WARMUP_FRAMES {
                self.bench_start = Some(Instant::now());
            }
            self.simulate();
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
                label: Some("particles"),
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
                let verdict = if frame_ok && stable_ok { "PASS" } else { "FAIL" };
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
