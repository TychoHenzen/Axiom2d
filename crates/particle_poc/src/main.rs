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
const HOPPER_X_MIN: f32 = -0.3;
const HOPPER_X_MAX: f32 = 0.3;
const HOPPER_Y: f32 = 0.75;
const GRID_W: u32 = 160;
const GRID_H: u32 = 160;
const TOTAL_GRID_CELLS: u32 = GRID_W * GRID_H;
// 16 substeps of dt=1/960 → exactly 1/60 s of simulated time per frame (real
// time at 60 FPS). Many small substeps with one constraint iteration each
// converge far better than few substeps with many iterations (Macklin et al.,
// "Small Steps in Physics Simulation"): constraint error scales with dt².
const SUB_STEPS: u32 = 16;

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
    spring_k: f32,
    damping: f32,
    friction_mu: f32,
    grid_cell_size: f32,
    grid_width: u32,
    grid_height: u32,
    _pad: [u32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct RenderParams {
    screen_width: f32,
    screen_height: f32,
    particle_radius: f32,
    particle_count: u32,
}

fn main() {
    let benchmark = std::env::args().any(|a| a == "--benchmark");
    let diagnose = std::env::args().any(|a| a == "--diagnose");
    let event_loop = EventLoop::new().expect("event loop");
    let mut app = App {
        state: None,
        benchmark,
        diagnose,
    };
    event_loop.run_app(&mut app).expect("run");
}

struct App {
    state: Option<State>,
    benchmark: bool,
    diagnose: bool,
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
}

struct ComputePipelines {
    predict: wgpu::ComputePipeline,
    clear_cells: wgpu::ComputePipeline,
    assign_cells: wgpu::ComputePipeline,
    prefix_scan: wgpu::ComputePipeline,
    scatter: wgpu::ComputePipeline,
    project: wgpu::ComputePipeline,
    apply: wgpu::ComputePipeline,
    reaction: wgpu::ComputePipeline,
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
    diagnose: bool,
    diag_frame: u32,
    diag_staging: Option<wgpu::Buffer>,
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
    let grid_storage = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
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
            storage_entry(0, false), // positions
            storage_entry(1, false), // velocities
            storage_entry(2, false), // species
            storage_entry(3, false), // forces
            storage_entry(4, false), // prev_positions
            uniform_entry(7),        // params
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
        ],
    })
}

fn create_bind_groups(
    device: &wgpu::Device,
    buffers: &Buffers,
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
        assign_cells: make_compute_pipeline(
            device,
            &spatial_layout,
            &spatial_shader,
            "assign_cells",
            "assign_cells",
        ),
        prefix_scan: make_compute_pipeline(
            device,
            &spatial_layout,
            &spatial_shader,
            "prefix_scan",
            "prefix_scan",
        ),
        scatter: make_compute_pipeline(
            device,
            &spatial_layout,
            &spatial_shader,
            "scatter",
            "scatter",
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
    fn new(window: Arc<Window>, benchmark: bool, diagnose: bool) -> Self {
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
        let (particle_bg, grid_bg) =
            create_bind_groups(&device, &buffers, &particle_bgl, &grid_bgl);
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
            // spring_k / damping are unused since the PBD rewrite (contacts are
            // position constraints, not spring forces). Kept as zeroed fields so
            // the uniform layout shared by all shaders stays unchanged.
            spring_k: 0.0,
            damping: 0.0,
            friction_mu: 0.3,
            grid_cell_size: 0.01,
            grid_width: GRID_W,
            grid_height: GRID_H,
            _pad: [0; 2],
        };

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
            diagnose,
            diag_frame: 0,
            diag_staging: None,
        }
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

    fn spawn(&mut self) {
        let current = self.sim_params.particle_count;
        let to_spawn = SPAWN_RATE.min(MAX_PARTICLES - current);
        if to_spawn == 0 {
            return;
        }

        let mut positions = Vec::with_capacity(to_spawn as usize);
        let mut velocities = Vec::with_capacity(to_spawn as usize);
        let mut species = Vec::with_capacity(to_spawn as usize);

        // One row per frame. Row spacing check: particles fall v*dt_frame ≈ 0.5/60 = 8.3mm
        // per frame, spawn spacing 0.6/130 = 4.6mm > contact_dist 4mm → no overlap at spawn.
        let hopper_width = HOPPER_X_MAX - HOPPER_X_MIN;
        let spacing = hopper_width / SPAWN_RATE as f32;
        let r = self.sim_params.particle_radius;
        let max_jitter = 0.5 * (spacing - 2.0 * r);
        for i in 0..to_spawn {
            let hash = (current + i).wrapping_mul(0x9E37_79B9);
            let jitter = (((hash >> 16) ^ hash) as f32 / u32::MAX as f32) - 0.5;
            let t = (i as f32 + 0.5) / to_spawn as f32;
            let x = HOPPER_X_MIN + t * hopper_width + jitter * max_jitter;
            positions.push([x, HOPPER_Y]);
            velocities.push([0.0f32, -0.5]);
            species.push(u32::from(!(current + i).is_multiple_of(2)));
        }

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
        let r = self.sim_params.particle_radius;
        let spacing = 2.1 * r;
        let usable_w = self.sim_params.wall_max_x - self.sim_params.wall_min_x - 2.0 * r;
        let cols = (usable_w / spacing) as u32;

        let mut positions = Vec::with_capacity(MAX_PARTICLES as usize);
        let mut velocities = Vec::with_capacity(MAX_PARTICLES as usize);
        let mut species = Vec::with_capacity(MAX_PARTICLES as usize);

        for i in 0..MAX_PARTICLES {
            let col = i % cols;
            let row = i / cols;
            let x = self.sim_params.wall_min_x + r + col as f32 * spacing;
            let y = self.sim_params.wall_max_y - r - row as f32 * spacing;
            positions.push([x, y]);
            velocities.push([0.0f32, 0.0]);
            species.push(i % 2);
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
        self.sim_params.particle_count = MAX_PARTICLES;
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

        for _ in 0..SUB_STEPS {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("pbd_substep"),
                ..Default::default()
            });
            pass.set_bind_group(0, &self.particle_bg, &[]);
            pass.set_bind_group(1, &self.grid_bg, &[]);

            // Predict positions first so the hash grid matches where
            // particles actually are during constraint projection.
            pass.set_pipeline(&self.pipelines.predict);
            pass.dispatch_workgroups(particle_wg, 1, 1);

            pass.set_pipeline(&self.pipelines.clear_cells);
            pass.dispatch_workgroups(grid_wg, 1, 1);

            pass.set_pipeline(&self.pipelines.assign_cells);
            pass.dispatch_workgroups(particle_wg, 1, 1);

            pass.set_pipeline(&self.pipelines.prefix_scan);
            pass.dispatch_workgroups(1, 1, 1);

            pass.set_pipeline(&self.pipelines.scatter);
            pass.dispatch_workgroups(particle_wg, 1, 1);

            pass.set_pipeline(&self.pipelines.project);
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
        self.queue.submit(std::iter::once(encoder.finish()));
    }

    fn render(&mut self) {
        const WARMUP_FRAMES: u32 = 10;
        const MEASURED_FRAMES: u32 = 300;

        if self.benchmark {
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

        let frame = if let Ok(f) = self.surface.get_current_texture() {
            f
        } else {
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
            let total_needed = WARMUP_FRAMES + MEASURED_FRAMES;
            if self.bench_frame >= total_needed {
                self.device.poll(wgpu::Maintain::Wait);
                let elapsed = self.bench_start.unwrap().elapsed();
                let avg_ms = elapsed.as_secs_f64() * 1000.0 / f64::from(MEASURED_FRAMES);
                let count = self.sim_params.particle_count;
                let verdict = if avg_ms < 16.67 { "PASS" } else { "FAIL" };
                println!("Benchmark: {MEASURED_FRAMES} frames, {count} particles");
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
        self.state = Some(State::new(window, self.benchmark, self.diagnose));
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
