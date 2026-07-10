// HeadlessCapture — headless GPU simulation + off-screen render for testing,
// following the same pattern as engine_render::testing::HeadlessRenderer.
//
// Creates a software-fallback wgpu device, simulates PBD particle physics,
// and renders to an off-screen Rgba8Unorm texture for pixel readback.
#![allow(clippy::wildcard_imports)]

use bytemuck;
use rapier2d::prelude::*;

use crate::*;

const HEADLESS_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
const COPY_BYTES_PER_ROW_ALIGNMENT: u32 = 256;

/// Configuration for headless capture resolution, particle size, and physics.
#[derive(Clone)]
pub struct CaptureConfig {
    /// Render target width in pixels.
    pub width: u32,
    /// Render target height in pixels.
    pub height: u32,
    /// World-space particle radius (larger than real-time for visibility; default 0.02).
    pub particle_radius: f32,
    /// Left wall bound in world space (default -0.8).
    pub wall_min_x: f32,
    /// Right wall bound in world space (default 0.8).
    pub wall_max_x: f32,
    /// Bottom wall bound in world space (default -0.8).
    pub wall_min_y: f32,
    /// Top wall bound in world space (default 0.8).
    pub wall_max_y: f32,
    /// Gravity strength (default -1.2; use 0.0 for stability tests).
    pub gravity: f32,
    /// PBD sub-steps per frame (default 16).
    pub sub_steps: u32,
    /// Maximum particle count (default 1000 for tests).
    pub num_particles: u32,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            width: 256,
            height: 256,
            particle_radius: 0.02,
            wall_min_x: -0.8,
            wall_max_x: 0.8,
            wall_min_y: -0.8,
            wall_max_y: 0.8,
            gravity: -1.2,
            sub_steps: 16,
            num_particles: 1000,
        }
    }
}

/// Headless simulation + off-screen rendering for testing `particle_poc`.
pub struct HeadlessCapture {
    device: wgpu::Device,
    queue: wgpu::Queue,
    output_texture: wgpu::Texture,
    width: u32,
    height: u32,
    buffers: Buffers,
    particle_bg: wgpu::BindGroup,
    grid_bg: wgpu::BindGroup,
    detection_bg: wgpu::BindGroup,
    pipelines: ComputePipelines,
    render: RenderState,
    sim_params: SimParams,
    rapier: RapierState,
    machines: Vec<MachineDef>,
    machines_cpu: Vec<MachineCpuState>,
    machine_params_buf: wgpu::Buffer,
    counters_staging: wgpu::Buffer,
    machine_time: f32,
    outlier_staging: Option<wgpu::Buffer>,
    phasing_staging: Option<wgpu::Buffer>,
    outlier_count_staging: Option<wgpu::Buffer>,
    phasing_count_staging: Option<wgpu::Buffer>,
}

impl HeadlessCapture {
    /// Try to create a headless capture, returning `None` if no GPU adapter is available.
    /// Uses `force_fallback_adapter` so tests skip gracefully on headless CI.
    pub fn try_new(config: CaptureConfig) -> Option<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter_opts = wgpu::RequestAdapterOptions {
            force_fallback_adapter: true,
            ..Default::default()
        };
        let adapter = pollster::block_on(instance.request_adapter(&adapter_opts))?;
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None))
                .ok()?;

        let output_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("headless_output"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: HEADLESS_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let buffers = create_buffers(&device);
        let particle_bgl = create_particle_bgl(&device);
        let grid_bgl = create_grid_bgl(&device);
        let detection_bgl = create_detection_bgl(&device);

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
        let render = create_render_state(&device, &buffers, HEADLESS_FORMAT);

        let sim_params = SimParams {
            particle_count: 0,
            dt: 1.0 / 960.0,
            gravity: config.gravity,
            particle_radius: config.particle_radius,
            wall_min_x: config.wall_min_x,
            wall_min_y: config.wall_min_y,
            wall_max_x: config.wall_max_x,
            wall_max_y: config.wall_max_y,
            friction_mu: 0.3,
            grid_cell_size: (config.wall_max_x - config.wall_min_x) / GRID_W as f32,
            grid_width: GRID_W,
            grid_height: GRID_H,
            disable_velocity_cap: 0,
            sub_steps: config.sub_steps,
        };

        // Initialize reaction matrix.
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

        let (machines, machines_cpu, rapier) = init_machines(rapier);

        Some(Self {
            device,
            queue,
            output_texture,
            width: config.width,
            height: config.height,
            buffers,
            particle_bg,
            grid_bg,
            detection_bg,
            pipelines,
            render,
            sim_params,
            rapier,
            machines,
            machines_cpu,
            machine_params_buf,
            counters_staging,
            machine_time: 0.0,
            outlier_staging: None,
            phasing_staging: None,
            outlier_count_staging: None,
            phasing_count_staging: None,
        })
    }

    /// Number of active particles.
    pub fn particle_count(&self) -> u32 {
        self.sim_params.particle_count
    }

    /// Spawn particles at given positions with given species and zero velocity.
    pub fn spawn_at(&mut self, positions: &[[f32; 2]], species: &[u32]) {
        let count = positions.len().min(species.len()) as u32;
        if count == 0 {
            return;
        }
        let offset = u64::from(self.sim_params.particle_count);
        let mut velocities = vec![[0.0f32; 2]; count as usize];
        // Apply initial downward velocity so particles settle.
        if self.sim_params.gravity != 0.0 {
            for v in &mut velocities {
                *v = [0.0, -0.5];
            }
        }
        self.queue.write_buffer(
            &self.buffers.positions,
            offset * 8,
            bytemuck::cast_slice(positions),
        );
        self.queue.write_buffer(
            &self.buffers.velocities,
            offset * 8,
            bytemuck::cast_slice(&velocities),
        );
        self.queue.write_buffer(
            &self.buffers.species,
            offset * 4,
            bytemuck::cast_slice(species),
        );
        self.sim_params.particle_count += count;
    }

    /// Fill the simulation box with particles arranged in a grid, alternating Red(0) and Blue(1).
    pub fn spawn_grid(&mut self, count: u32) {
        let r = self.sim_params.particle_radius;
        let spacing = 2.1 * r;
        let usable_w = self.sim_params.wall_max_x - self.sim_params.wall_min_x - 2.0 * r;
        let cols = (usable_w / spacing) as u32;
        let count = count.min(MAX_PARTICLES - self.sim_params.particle_count);

        let mut positions: Vec<[f32; 2]> = Vec::with_capacity(count as usize);
        let mut velocities: Vec<[f32; 2]> = Vec::with_capacity(count as usize);
        let mut species: Vec<u32> = Vec::with_capacity(count as usize);

        for i in 0..count {
            let col = i % cols;
            let _ = i / cols;
            let x = self.sim_params.wall_min_x + r + col as f32 * spacing;
            let y = self.sim_params.wall_max_y - r;
            positions.push([x, y]);
            velocities.push([0.0f32; 2]);
            species.push(u32::from(x >= 0.0));
        }

        let offset = u64::from(self.sim_params.particle_count);
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
        self.sim_params.particle_count += count;
    }

    /// Set up a mutual bond between two particles. Writes bond data directly to GPU buffer.
    pub fn set_mutual_bond(&self, a: u32, b: u32, rest_length: f32) {
        let n = MAX_PARTICLES as usize;
        let mut bonds: Vec<BondSlot> = (0..n * 4).map(|_| BondSlot::default()).collect();
        bonds[a as usize * 4] = BondSlot {
            partner: b,
            rest: rest_length,
        };
        bonds[b as usize * 4] = BondSlot {
            partner: a,
            rest: rest_length,
        };
        self.queue
            .write_buffer(&self.buffers.bonds, 0, bytemuck::cast_slice(&bonds));
    }

    /// Run one frame: update machines, simulate, write render params.
    pub fn step(&mut self) {
        self.update_machines();
        self.simulate();
    }

    /// Run N frames.
    pub fn step_n(&mut self, n: u32) {
        for _ in 0..n {
            self.step();
        }
    }

    /// Read back particle positions from GPU.
    pub fn read_positions(&self, count: u32) -> Vec<[f32; 2]> {
        if count == 0 {
            return Vec::new();
        }
        let bytes = u64::from(count) * 8;
        let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("read_pos_staging"),
            size: bytes,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        {
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("read_pos"),
                });
            encoder.copy_buffer_to_buffer(&self.buffers.positions, 0, &staging, 0, bytes);
            self.queue.submit(std::iter::once(encoder.finish()));
        }
        let slice = staging.slice(..);
        slice.map_async(wgpu::MapMode::Read, |r| r.expect("map pos staging"));
        self.device.poll(wgpu::Maintain::Wait);
        let data = slice.get_mapped_range();
        let positions: Vec<[f32; 2]> = bytemuck::cast_slice(&data[..bytes as usize]).to_vec();
        drop(data);
        staging.unmap();
        positions
    }

    /// Read back particle species from GPU.
    pub fn read_species(&self, count: u32) -> Vec<u32> {
        if count == 0 {
            return Vec::new();
        }
        let bytes = u64::from(count) * 4;
        let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("read_sp_staging"),
            size: bytes,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        {
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("read_sp"),
                });
            encoder.copy_buffer_to_buffer(&self.buffers.species, 0, &staging, 0, bytes);
            self.queue.submit(std::iter::once(encoder.finish()));
        }
        let slice = staging.slice(..);
        slice.map_async(wgpu::MapMode::Read, |r| r.expect("map sp staging"));
        self.device.poll(wgpu::Maintain::Wait);
        let data = slice.get_mapped_range();
        let species: Vec<u32> = bytemuck::cast_slice(&data[..bytes as usize]).to_vec();
        drop(data);
        staging.unmap();
        species
    }

    /// Read back bonds from GPU.
    pub fn read_bonds(&self) -> Vec<BondSlot> {
        let n = MAX_PARTICLES as usize;
        let total_bytes = (n * 4 * size_of::<BondSlot>()) as u64;
        let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("read_bonds_staging"),
            size: total_bytes,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        {
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("read_bonds"),
                });
            encoder.copy_buffer_to_buffer(&self.buffers.bonds, 0, &staging, 0, total_bytes);
            self.queue.submit(std::iter::once(encoder.finish()));
        }
        let slice = staging.slice(..);
        slice.map_async(wgpu::MapMode::Read, |r| r.expect("map bonds staging"));
        self.device.poll(wgpu::Maintain::Wait);
        let data = slice.get_mapped_range();
        let all: Vec<BondSlot> = bytemuck::cast_slice(&data[..total_bytes as usize]).to_vec();
        drop(data);
        staging.unmap();
        all
    }

    /// Read back phasing event count from GPU atomic counter.
    /// Must be called after `step()` / `step_n()` — the count is reset each frame.
    pub fn read_phasing_count(&self) -> u32 {
        let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("read_phasing_count_staging"),
            size: 4,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        {
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("read_phasing_count"),
                });
            encoder.copy_buffer_to_buffer(
                &self.buffers.phasing_count_buf,
                0,
                &staging,
                0,
                4,
            );
            self.queue.submit(std::iter::once(encoder.finish()));
        }
        let slice = staging.slice(..);
        slice.map_async(wgpu::MapMode::Read, |r| r.expect("map phasing count staging"));
        self.device.poll(wgpu::Maintain::Wait);
        let data = slice.get_mapped_range();
        let count = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        drop(data);
        staging.unmap();
        count
    }

    /// Read back particle velocities from GPU.
    pub fn read_velocities(&self, count: u32) -> Vec<[f32; 2]> {
        if count == 0 {
            return Vec::new();
        }
        let bytes = u64::from(count) * 8;
        let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("read_vel_staging"),
            size: bytes,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        {
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("read_vel"),
                });
            encoder.copy_buffer_to_buffer(&self.buffers.velocities, 0, &staging, 0, bytes);
            self.queue.submit(std::iter::once(encoder.finish()));
        }
        let slice = staging.slice(..);
        slice.map_async(wgpu::MapMode::Read, |r| r.expect("map vel staging"));
        self.device.poll(wgpu::Maintain::Wait);
        let data = slice.get_mapped_range();
        let velocities: Vec<[f32; 2]> = bytemuck::cast_slice(&data[..bytes as usize]).to_vec();
        drop(data);
        staging.unmap();
        velocities
    }

    /// Render current state to off-screen texture and read back RGBA pixels.
    /// Returns packed `[width * height * 4]` bytes.
    pub fn render_to_buffer(&mut self) -> Vec<u8> {
        let view = self
            .output_texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("capture_render"),
            });

        // --- Render pass ---
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("capture_scene"),
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

            // Draw machines first.
            let machine_count = self.machines.len() as u32 + PADDLE_COUNT;
            if machine_count > 0 {
                let mparams = MachineRenderParams {
                    screen_width: self.width as f32,
                    screen_height: self.height as f32,
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
                let render_params = RenderParams {
                    screen_width: self.width as f32,
                    screen_height: self.height as f32,
                    particle_radius: self.sim_params.particle_radius,
                    particle_count: self.sim_params.particle_count,
                };
                self.queue.write_buffer(
                    &self.render.params_buf,
                    0,
                    bytemuck::bytes_of(&render_params),
                );
                pass.set_pipeline(&self.render.pipeline);
                pass.set_bind_group(0, &self.render.bind_group, &[]);
                pass.draw(0..self.sim_params.particle_count * 6, 0..1);
            }
        }

        // Read back pixels.
        let padded_row = padded_row_bytes(self.width, 4);
        let buffer_size = wgpu::BufferAddress::from(padded_row * self.height);
        let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("capture_staging"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.output_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &staging,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_row),
                    rows_per_image: Some(self.height),
                },
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(std::iter::once(encoder.finish()));

        let slice = staging.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device.poll(wgpu::Maintain::Wait);

        let data = slice.get_mapped_range();
        let pixels = strip_row_padding(&data, self.width, self.height, padded_row, 4);
        drop(data);
        staging.unmap();
        pixels
    }

    /// Compare a captured pixel buffer against a golden image loaded from PNG bytes.
    pub fn ssim_compare(&self, captured: &[u8], golden: &[u8]) -> f32 {
        let img_a = image::RgbaImage::from_raw(self.width, self.height, captured.to_vec())
            .expect("captured pixels must match dimensions");
        let img_b = image::RgbaImage::from_raw(self.width, self.height, golden.to_vec())
            .expect("golden pixels must match dimensions");
        image_compare::rgba_hybrid_compare(&img_a, &img_b)
            .expect("images must have identical dimensions")
            .score as f32
    }
}

/// Compare two RGBA pixel buffers via SSIM.
pub fn ssim_compare(pixels_a: &[u8], pixels_b: &[u8], width: u32, height: u32) -> f32 {
    let img_a = image::RgbaImage::from_raw(width, height, pixels_a.to_vec())
        .expect("pixels_a must match dimensions");
    let img_b = image::RgbaImage::from_raw(width, height, pixels_b.to_vec())
        .expect("pixels_b must match dimensions");
    image_compare::rgba_hybrid_compare(&img_a, &img_b)
        .expect("images must have identical dimensions")
        .score as f32
}

impl HeadlessCapture {
    fn simulate(&mut self) {
        self.queue.write_buffer(
            &self.buffers.params,
            0,
            bytemuck::bytes_of(&self.sim_params),
        );

        // Clear outlier/phasing atomic counters.
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
                label: Some("capture_compute"),
            });

        // PBD sub-step loop.
        let sub_steps = self.sim_params.sub_steps;
        for _ in 0..sub_steps {
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

        // Reaction pass (once per frame).
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

        // Bond formation passes (once per frame).
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

        // Detection passes (once per frame).
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

        // Copy detection data + machine counters to staging buffers.
        let os = self.outlier_staging.get_or_insert_with(|| {
            self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("outlier_staging"),
                size: 2048,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            })
        });
        let ps = self.phasing_staging.get_or_insert_with(|| {
            self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("phasing_staging"),
                size: 512,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            })
        });
        let ocs = self.outlier_count_staging.get_or_insert_with(|| {
            self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("outlier_count_staging"),
                size: 4,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            })
        });
        let pcs = self.phasing_count_staging.get_or_insert_with(|| {
            self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("phasing_count_staging"),
                size: 4,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            })
        });
        encoder.copy_buffer_to_buffer(&self.buffers.outlier_buf, 0, os, 0, 2048);
        encoder.copy_buffer_to_buffer(&self.buffers.phasing_buf, 0, ps, 0, 512);
        encoder.copy_buffer_to_buffer(&self.buffers.outlier_count_buf, 0, ocs, 0, 4);
        encoder.copy_buffer_to_buffer(&self.buffers.phasing_count_buf, 0, pcs, 0, 4);
        encoder.copy_buffer_to_buffer(
            &self.buffers.machine_counters,
            0,
            &self.counters_staging,
            0,
            u64::from(MAX_MACHINES) * 4,
        );

        self.queue.submit(std::iter::once(encoder.finish()));
    }

    fn update_machines(&mut self) {
        let dt = 1.0 / 60.0;
        self.machine_time += dt;

        // Read GPU counters from previous frame.
        {
            let slice = self.counters_staging.slice(..);
            slice.map_async(wgpu::MapMode::Read, |_| {});
            self.device.poll(wgpu::Maintain::Wait);
            let data = slice.get_mapped_range();
            let counters: &[u32] = bytemuck::cast_slice(&data[..self.machines.len() * 4]);
            for (i, cpu) in self.machines_cpu.iter_mut().enumerate() {
                let eaten = counters.get(i).copied().unwrap_or(0);
                cpu.consumed_this_frame = eaten;
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

        // Reset GPU counters.
        let zeros = [0u32; MAX_MACHINES as usize];
        self.queue.write_buffer(
            &self.buffers.machine_counters,
            0,
            bytemuck::cast_slice(&zeros),
        );

        // Step Rapier2D.
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

        // Capsule conveyor body stays at fixed angle.
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
            let (hw, hh) = if def.kind as u32 == MachineKind::Conveyor as u32 {
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
                angular_velocity: if def.kind as u32 == MachineKind::Conveyor as u32 {
                    CONVEYOR_SPEED
                } else {
                    0.0
                },
            };
            let cpu = &self.machines_cpu[i];
            let activity = (cpu.consumed_this_frame as f32 / 20.0).clamp(0.0, 1.0);
            let brightness = 0.6 + 0.4 * activity;
            let cb = cpu.color_base;
            let (rhw, rhh) = if def.kind as u32 == MachineKind::Conveyor as u32 {
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
        // Push paddle center outward from perimeter so the tracing point
        // sits at 90% toward the inner edge instead of at center.
        let paddle_push = PADDLE_HH * 0.9;
        for p in 0..PADDLE_COUNT {
            let s = (paddle_phase + p as f32 * cap_perim / PADDLE_COUNT as f32) % cap_perim;
            let (pos, local_tangent) = capsule_perimeter_point(s, CAPSULE_HALF_LEN, CAPSULE_RADIUS);
            let lx = pos[0];
            let ly = pos[1];

            // Outward normal in capsule-local space: direction from centerline to perimeter.
            let nearest_lx = lx.clamp(-CAPSULE_HALF_LEN, CAPSULE_HALF_LEN);
            let out_lx = lx - nearest_lx;
            let out_ly = ly; // centerline at y=0
            let out_len = (out_lx * out_lx + out_ly * out_ly).sqrt();
            let n_lx = out_lx / out_len;
            let n_ly = out_ly / out_len;
            // Transform outward normal to world space (same rotation as positions).
            let nwx = -n_lx * c_sin - n_ly * c_cos;
            let nwy = n_lx * c_cos - n_ly * c_sin;

            let wx = cx - lx * c_sin - ly * c_cos + nwx * paddle_push;
            let wy = cy + lx * c_cos - ly * c_sin + nwy * paddle_push;
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
}

/// Padded row bytes for wgpu texture-to-buffer copy alignment.
pub fn padded_row_bytes(width: u32, bytes_per_pixel: u32) -> u32 {
    let raw = width * bytes_per_pixel;
    raw.div_ceil(COPY_BYTES_PER_ROW_ALIGNMENT) * COPY_BYTES_PER_ROW_ALIGNMENT
}

/// Strip wgpu row padding from a readback buffer.
pub fn strip_row_padding(
    data: &[u8],
    width: u32,
    height: u32,
    padded_row: u32,
    bytes_per_pixel: u32,
) -> Vec<u8> {
    let row_bytes = width * bytes_per_pixel;
    let mut out = Vec::with_capacity((row_bytes * height) as usize);
    for y in 0..height {
        let start = (y * padded_row) as usize;
        let end = start + row_bytes as usize;
        out.extend_from_slice(&data[start..end]);
    }
    out
}

/// Capsule perimeter parameterization: given arc-length `s`, returns
/// `(local_x, local_y, tangent_angle)` in capsule-local coordinates.
fn capsule_perimeter_point(s: f32, l: f32, r: f32) -> ([f32; 2], f32) {
    let cap_arc = std::f32::consts::PI * r;
    let straight = 2.0 * l;
    let perimeter = 2.0 * cap_arc + 2.0 * straight;
    let s = ((s % perimeter) + perimeter) % perimeter;

    if s < cap_arc {
        let phi = -std::f32::consts::FRAC_PI_2 + s / r;
        let lx = l + r * phi.cos();
        let ly = r * phi.sin();
        ([lx, ly], phi + std::f32::consts::FRAC_PI_2)
    } else if s < cap_arc + straight {
        let t = (s - cap_arc) / straight;
        let lx = l - t * 2.0 * l;
        let ly = r;
        ([lx, ly], std::f32::consts::PI)
    } else if s < 2.0 * cap_arc + straight {
        let phi = std::f32::consts::FRAC_PI_2 + (s - cap_arc - straight) / r;
        let lx = -l + r * phi.cos();
        let ly = r * phi.sin();
        ([lx, ly], phi + std::f32::consts::FRAC_PI_2)
    } else {
        let t = (s - 2.0 * cap_arc - straight) / straight;
        let lx = -l + t * 2.0 * l;
        let ly = -r;
        ([lx, ly], 0.0)
    }
}
