// State implementation — extracted from main.rs
#![allow(clippy::wildcard_imports)]

use std::sync::Arc;
use std::time::Instant;

use rapier2d::prelude::*;
use winit::window::Window;

use crate::*;

impl State {
    #[allow(clippy::fn_params_excessive_bools)]
    pub fn new(
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

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
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

    pub fn render(&mut self) {
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
