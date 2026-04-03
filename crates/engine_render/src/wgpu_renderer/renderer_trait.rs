use engine_core::color::Color;

use crate::rect::Rect;
use crate::renderer::Renderer;
use crate::shader::ShaderHandle;

use super::gpu_init::create_shape_pipeline_set;
use super::renderer::{
    MeshSource, PersistentMesh, ShapeDrawRecord, UploadBindGroupBuffer, UploadBuffer, WgpuRenderer,
    align_to_uniform_offset, pack_material_frame_data,
};
use super::types::{
    BloomParamsUniform, FullscreenBuffers, FullscreenPass, QUAD_INDICES, ShapeVertex, TextureData,
    compute_batch_ranges, create_texture_bind_group, rect_to_instance, run_fullscreen_pass,
};

struct PreparedShapeResources {
    aligned_entry: usize,
    material_entry_size: usize,
    model_bg: wgpu::BindGroup,
    material_bg: wgpu::BindGroup,
    batched_buffers: Option<(wgpu::Buffer, wgpu::Buffer)>,
}

impl WgpuRenderer {
    fn begin_scene_pass<'a>(
        encoder: &'a mut wgpu::CommandEncoder,
        msaa_view: &'a wgpu::TextureView,
        resolve_target: &'a wgpu::TextureView,
        clear_color: wgpu::Color,
    ) -> wgpu::RenderPass<'a> {
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: msaa_view,
                resolve_target: Some(resolve_target),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear_color),
                    store: wgpu::StoreOp::Discard,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        })
    }

    #[allow(clippy::cast_possible_truncation)]
    fn draw_quad_instances(&self, pass: &mut wgpu::RenderPass, instance_buffer: &wgpu::Buffer) {
        pass.set_bind_group(0, &self.texture_bind_group, &[]);
        pass.set_bind_group(1, &self.camera_bind_group, &[]);
        pass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
        pass.set_vertex_buffer(1, instance_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        for (mode, range) in compute_batch_ranges(&self.instance_blend_modes) {
            pass.set_pipeline(&self.quad_pipelines[mode.index()]);
            pass.draw_indexed(0..QUAD_INDICES.len() as u32, 0, range);
        }
    }

    fn prepare_upload_buffer(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        slot: &mut Option<UploadBuffer>,
        usage: wgpu::BufferUsages,
        bytes: &[u8],
    ) -> wgpu::Buffer {
        let upload = slot.get_or_insert_with(|| UploadBuffer::new(device, usage, bytes.len()));
        upload.ensure_capacity(device, bytes.len());
        queue.write_buffer(&upload.buffer, 0, bytes);
        upload.buffer.clone()
    }

    fn prepare_instance_buffer(&mut self) -> Option<wgpu::Buffer> {
        let bytes = bytemuck::cast_slice(&self.pending_instances);
        (!bytes.is_empty()).then(|| {
            Self::prepare_upload_buffer(
                &self.device,
                &self.queue,
                &mut self.instance_upload_buffer,
                wgpu::BufferUsages::VERTEX,
                bytes,
            )
        })
    }

    fn prepare_shape_buffers(&mut self) -> Option<(wgpu::Buffer, wgpu::Buffer)> {
        let vertex_bytes = bytemuck::cast_slice(self.shape_batch.vertices());
        let index_bytes = bytemuck::cast_slice(self.shape_batch.indices());
        if vertex_bytes.is_empty() || index_bytes.is_empty() {
            return None;
        }
        let vb = Self::prepare_upload_buffer(
            &self.device,
            &self.queue,
            &mut self.shape_vertex_upload_buffer,
            wgpu::BufferUsages::VERTEX,
            vertex_bytes,
        );
        let ib = Self::prepare_upload_buffer(
            &self.device,
            &self.queue,
            &mut self.shape_index_upload_buffer,
            wgpu::BufferUsages::INDEX,
            index_bytes,
        );
        Some((vb, ib))
    }

    fn prepare_model_bind_group(&mut self, aligned_entry: usize) -> Option<wgpu::BindGroup> {
        let model_data = pack_model_uniform_data(&self.shape_draws, aligned_entry);
        if model_data.is_empty() {
            return None;
        }
        let model_upload = self.model_upload_buffer.get_or_insert_with(|| {
            UploadBindGroupBuffer::new(
                &self.device,
                &self.model_bind_group_layout,
                64,
                model_data.len(),
            )
        });
        model_upload.ensure_capacity(
            &self.device,
            &self.model_bind_group_layout,
            64,
            model_data.len(),
        );
        self.queue
            .write_buffer(&model_upload.storage.buffer, 0, &model_data);
        Some(model_upload.bind_group.clone())
    }

    fn prepare_material_bind_group(&mut self) -> Option<(wgpu::BindGroup, usize)> {
        let (slot_size, material_data) = pack_material_frame_data(
            &self.shape_draws,
            &self.texture_lookups,
            self.model_uniform_align as usize,
        );
        if material_data.is_empty() {
            return None;
        }
        let material_upload = self.material_upload_buffer.get_or_insert_with(|| {
            UploadBindGroupBuffer::new(
                &self.device,
                &self.material_bind_group_layout,
                slot_size as u64,
                material_data.len(),
            )
        });
        material_upload.ensure_capacity(
            &self.device,
            &self.material_bind_group_layout,
            slot_size as u64,
            material_data.len(),
        );
        self.queue
            .write_buffer(&material_upload.storage.buffer, 0, &material_data);
        Some((material_upload.bind_group.clone(), slot_size))
    }

    fn select_shape_pipeline(
        &self,
        key: (ShaderHandle, crate::material::BlendMode),
    ) -> &wgpu::RenderPipeline {
        if key.0 == ShaderHandle(0) {
            &self.shape_pipelines[key.1.index()]
        } else if let Some(cached) = self.shader_cache.get(&key.0) {
            &cached[key.1.index()]
        } else {
            &self.shape_pipelines[key.1.index()]
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn issue_shape_draw_calls(
        &self,
        pass: &mut wgpu::RenderPass,
        model_bg: &wgpu::BindGroup,
        material_bg: &wgpu::BindGroup,
        aligned_entry: usize,
        material_entry_size: usize,
        batched_buffers: Option<&(wgpu::Buffer, wgpu::Buffer)>,
    ) {
        let mut last_key: Option<(ShaderHandle, crate::material::BlendMode)> = None;
        let mut batched_bound = false;

        for (i, draw) in self.shape_draws.iter().enumerate() {
            let key = (draw.shader_handle, draw.blend_mode);
            if last_key != Some(key) {
                pass.set_pipeline(self.select_shape_pipeline(key));
                last_key = Some(key);
            }
            let dyn_offset = (i * aligned_entry) as u32;
            let material_offset = (i * material_entry_size) as u32;
            pass.set_bind_group(1, model_bg, &[dyn_offset]);
            pass.set_bind_group(2, material_bg, &[material_offset]);

            match &draw.source {
                MeshSource::Batched {
                    index_start,
                    index_count,
                } => {
                    if !batched_bound {
                        if let Some((vb, ib)) = batched_buffers {
                            pass.set_vertex_buffer(0, vb.slice(..));
                            pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
                        }
                        batched_bound = true;
                    }
                    pass.draw_indexed(*index_start..*index_start + *index_count, 0, 0..1);
                }
                MeshSource::Persistent { handle } => {
                    if let Some(mesh) = self.persistent_meshes.get(handle) {
                        pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                        pass.set_index_buffer(
                            mesh.index_buffer.slice(..),
                            wgpu::IndexFormat::Uint32,
                        );
                        batched_bound = false;
                        pass.draw_indexed(0..mesh.index_count, 0, 0..1);
                    }
                }
            }
        }
    }

    fn prepare_shape_resources(&mut self) -> Option<PreparedShapeResources> {
        if self.shape_draws.is_empty() {
            return None;
        }
        let aligned_entry = align_to_uniform_offset(64, self.model_uniform_align as usize);
        let model_bg = self.prepare_model_bind_group(aligned_entry)?;
        let (material_bg, material_entry_size) = self.prepare_material_bind_group()?;
        let batched_buffers = (!self.shape_batch.is_empty())
            .then(|| self.prepare_shape_buffers())
            .flatten();
        Some(PreparedShapeResources {
            aligned_entry,
            material_entry_size,
            model_bg,
            material_bg,
            batched_buffers,
        })
    }

    fn draw_shape_batches(&self, pass: &mut wgpu::RenderPass, prepared: &PreparedShapeResources) {
        pass.set_bind_group(0, &self.camera_bind_group, &[]);
        pass.set_bind_group(3, &self.texture_bind_group, &[]);
        self.issue_shape_draw_calls(
            pass,
            &prepared.model_bg,
            &prepared.material_bg,
            prepared.aligned_entry,
            prepared.material_entry_size,
            prepared.batched_buffers.as_ref(),
        );
    }

    pub(super) fn draw_scene_to(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        target_view: &wgpu::TextureView,
    ) {
        let instance_buffer = self.prepare_instance_buffer();
        let prepared_shapes = self.prepare_shape_resources();
        let clear_color = wgpu::Color {
            r: f64::from(self.clear_color.r),
            g: f64::from(self.clear_color.g),
            b: f64::from(self.clear_color.b),
            a: f64::from(self.clear_color.a),
        };
        let mut pass = Self::begin_scene_pass(encoder, &self.msaa_view, target_view, clear_color);
        if let Some(instance_buffer) = instance_buffer.as_ref() {
            self.draw_quad_instances(&mut pass, instance_buffer);
        }
        if let Some(prepared_shapes) = prepared_shapes.as_ref() {
            self.draw_shape_batches(&mut pass, prepared_shapes);
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn write_bloom_uniforms(&self) {
        let Some(pp) = self.post_process.as_ref() else {
            return;
        };
        let uniforms = bloom_uniforms(self.bloom_threshold, self.bloom_intensity, &self.config);
        let buffers = [
            &pp.brightness_params.0,
            &pp.h_blur_params.0,
            &pp.v_blur_params.0,
            &pp.composite_params.0,
        ];
        for (buf, uniform) in buffers.into_iter().zip(&uniforms) {
            write_bloom_param(&self.queue, buf, uniform);
        }
    }

    fn run_bloom_passes(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        swapchain_view: &wgpu::TextureView,
    ) {
        let Some(pp) = self.post_process.as_ref() else {
            return;
        };
        let bufs = FullscreenBuffers {
            vertex: &pp.fs_vertex_buffer,
            index: &self.index_buffer,
        };
        for desc in bloom_pass_descs(pp, swapchain_view) {
            run_fullscreen_pass(encoder, &bufs, &desc);
        }
    }

    pub(super) fn execute_bloom(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        swapchain_view: &wgpu::TextureView,
    ) {
        self.write_bloom_uniforms();
        self.run_bloom_passes(encoder, swapchain_view);
    }
}

fn pack_model_uniform_data(draws: &[ShapeDrawRecord], aligned_entry: usize) -> Vec<u8> {
    let buf_size = draws.len() * aligned_entry;
    let mut data = vec![0u8; buf_size];
    for (i, draw) in draws.iter().enumerate() {
        let offset = i * aligned_entry;
        let bytes: &[u8; 64] = bytemuck::cast_ref(&draw.model);
        data[offset..offset + 64].copy_from_slice(bytes);
    }
    data
}

fn bloom_uniform(
    threshold: f32,
    intensity: f32,
    direction: [f32; 2],
    texel_size: [f32; 2],
) -> BloomParamsUniform {
    BloomParamsUniform {
        threshold,
        intensity,
        direction,
        texel_size,
        _pad: [0.0; 2],
    }
}

#[allow(clippy::cast_possible_truncation)]
fn bloom_uniforms(
    threshold: f32,
    intensity: f32,
    config: &wgpu::SurfaceConfiguration,
) -> [BloomParamsUniform; 4] {
    let st = [1.0 / config.width as f32, 1.0 / config.height as f32];
    let hw = (config.width / 2).max(1) as f32;
    let hh = (config.height / 2).max(1) as f32;
    let ht = [1.0 / hw, 1.0 / hh];
    [
        bloom_uniform(threshold, 0.0, [0.0, 0.0], st),
        bloom_uniform(0.0, 0.0, [1.0, 0.0], ht),
        bloom_uniform(0.0, 0.0, [0.0, 1.0], ht),
        bloom_uniform(0.0, intensity, [0.0, 0.0], [0.0; 2]),
    ]
}

fn bloom_pass_descs<'a>(
    pp: &'a super::bloom::PostProcessResources,
    swapchain_view: &'a wgpu::TextureView,
) -> [FullscreenPass<'a>; 4] {
    [
        FullscreenPass {
            target: &pp.ping_view,
            pipeline: &pp.brightness_pipeline,
            tex_bg: &pp.scene_bg,
            params_bg: &pp.brightness_params.1,
        },
        FullscreenPass {
            target: &pp.pong_view,
            pipeline: &pp.blur_pipeline,
            tex_bg: &pp.ping_bg,
            params_bg: &pp.h_blur_params.1,
        },
        FullscreenPass {
            target: &pp.ping_view,
            pipeline: &pp.blur_pipeline,
            tex_bg: &pp.pong_bg,
            params_bg: &pp.v_blur_params.1,
        },
        FullscreenPass {
            target: swapchain_view,
            pipeline: &pp.composite_pipeline,
            tex_bg: &pp.composite_bg,
            params_bg: &pp.composite_params.1,
        },
    ]
}

fn write_bloom_param(queue: &wgpu::Queue, buffer: &wgpu::Buffer, uniform: &BloomParamsUniform) {
    queue.write_buffer(buffer, 0, bytemuck::bytes_of(uniform));
}

impl Renderer for WgpuRenderer {
    fn clear(&mut self, color: Color) {
        self.clear_color = color;
        self.reset_frame_state();
    }

    fn draw_rect(&mut self, rect: Rect) {
        self.pending_instances.push(rect_to_instance(&rect));
        self.instance_blend_modes.push(self.current_blend_mode);
    }

    fn draw_sprite(&mut self, rect: Rect, uv_rect: [f32; 4]) {
        let mut instance = rect_to_instance(&rect);
        instance.uv_rect = uv_rect;
        self.pending_instances.push(instance);
        self.instance_blend_modes.push(self.current_blend_mode);
    }

    fn draw_shape(
        &mut self,
        vertices: &[[f32; 2]],
        indices: &[u32],
        color: Color,
        model: [[f32; 4]; 4],
    ) {
        let material_uniforms = self.pending_material.take_uniforms();
        let material_textures = self.pending_material.take_textures();
        #[allow(clippy::cast_possible_truncation)]
        let index_start = self.shape_batch.index_count() as u32;
        self.shape_batch.push(vertices, indices, color);
        #[allow(clippy::cast_possible_truncation)]
        let index_count = (self.shape_batch.index_count() as u32) - index_start;
        self.shape_draws.push(ShapeDrawRecord {
            blend_mode: self.current_blend_mode,
            shader_handle: self.active_shader,
            source: MeshSource::Batched {
                index_start,
                index_count,
            },
            model,
            material_uniforms,
            material_textures,
        });
    }

    fn draw_colored_mesh(
        &mut self,
        vertices: &[crate::shape::ColorVertex],
        indices: &[u32],
        model: [[f32; 4]; 4],
    ) {
        let material_uniforms = self.pending_material.take_uniforms();
        let material_textures = self.pending_material.take_textures();
        #[allow(clippy::cast_possible_truncation)]
        let index_start = self.shape_batch.index_count() as u32;
        let shape_verts: &[ShapeVertex] = bytemuck::cast_slice(vertices);
        self.shape_batch.push_colored(shape_verts, indices);
        #[allow(clippy::cast_possible_truncation)]
        let index_count = (self.shape_batch.index_count() as u32) - index_start;
        self.shape_draws.push(ShapeDrawRecord {
            blend_mode: self.current_blend_mode,
            shader_handle: self.active_shader,
            source: MeshSource::Batched {
                index_start,
                index_count,
            },
            model,
            material_uniforms,
            material_textures,
        });
    }

    fn upload_persistent_colored_mesh(
        &mut self,
        vertices: &[crate::shape::ColorVertex],
        indices: &[u32],
    ) -> crate::renderer::GpuMeshHandle {
        use wgpu::util::DeviceExt;
        let handle = crate::renderer::GpuMeshHandle(self.next_persistent_id);
        self.next_persistent_id += 1;
        let shape_verts: &[ShapeVertex] = bytemuck::cast_slice(vertices);
        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(shape_verts),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let index_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX,
            });
        #[allow(clippy::cast_possible_truncation)]
        self.persistent_meshes.insert(
            handle,
            PersistentMesh {
                vertex_buffer,
                index_buffer,
                index_count: indices.len() as u32,
            },
        );
        handle
    }

    fn draw_persistent_colored_mesh(
        &mut self,
        handle: crate::renderer::GpuMeshHandle,
        model: [[f32; 4]; 4],
    ) {
        let material_uniforms = self.pending_material.take_uniforms();
        let material_textures = self.pending_material.take_textures();
        self.shape_draws.push(ShapeDrawRecord {
            blend_mode: self.current_blend_mode,
            shader_handle: self.active_shader,
            source: MeshSource::Persistent { handle },
            model,
            material_uniforms,
            material_textures,
        });
    }

    fn free_persistent_colored_mesh(&mut self, handle: crate::renderer::GpuMeshHandle) {
        self.persistent_meshes.remove(&handle);
    }

    fn draw_text(&mut self, text: &str, x: f32, y: f32, font_size: f32, color: Color) {
        let mut cache = std::mem::take(&mut self.glyph_cache);
        crate::font::render_text_glyphs(self, &mut cache, text, x, y, font_size, color);
        self.glyph_cache = cache;
    }

    fn set_blend_mode(&mut self, mode: crate::material::BlendMode) {
        self.current_blend_mode = mode;
    }

    fn set_shader(&mut self, shader: ShaderHandle) {
        self.active_shader = shader;
    }

    fn set_material_uniforms(&mut self, data: &[u8]) {
        self.pending_material.set_uniforms(data);
    }

    fn bind_material_texture(&mut self, texture: engine_core::types::TextureId, binding: u32) {
        self.pending_material.bind_texture(texture, binding);
    }

    fn compile_shader(
        &mut self,
        handle: ShaderHandle,
        source: &str,
    ) -> Result<(), crate::renderer::RenderError> {
        if self.shader_cache.contains_key(&handle) {
            return Ok(());
        }
        let shader_module = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(source.into()),
            });
        let pipelines = create_shape_pipeline_set(
            &self.device,
            &self.shape_pipeline_layout,
            &shader_module,
            self.surface_format,
            self.sample_count,
        );
        self.shader_cache.insert(handle, pipelines);
        Ok(())
    }

    fn upload_atlas(
        &mut self,
        atlas: &crate::atlas::TextureAtlas,
    ) -> Result<(), crate::renderer::RenderError> {
        self.texture_lookups = atlas.lookups.clone();
        self.texture_bind_group = create_texture_bind_group(
            &self.device,
            &self.queue,
            &self.texture_bind_group_layout,
            TextureData {
                width: atlas.width,
                height: atlas.height,
                data: &atlas.data,
            },
        );
        Ok(())
    }

    fn set_view_projection(&mut self, matrix: [[f32; 4]; 4]) {
        self.queue.write_buffer(
            &self.camera_uniform_buffer,
            0,
            bytemuck::cast_slice(&matrix),
        );
    }

    fn viewport_size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    fn apply_post_process(&mut self) {
        self.ensure_post_process();
        self.post_process_pending = true;
    }

    fn present(&mut self) {
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
            Err(wgpu::SurfaceError::Timeout) => return,
            Err(wgpu::SurfaceError::OutOfMemory) => {
                panic!("GPU out of memory");
            }
            Err(e) => {
                tracing::error!("surface error: {e}");
                return;
            }
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        if self.post_process_pending {
            let scene_view = self.post_process.as_ref().map(|pp| pp.scene_view.clone());
            if let Some(scene_view) = scene_view {
                self.draw_scene_to(&mut encoder, &scene_view);
            } else {
                self.draw_scene_to(&mut encoder, &view);
            }
            self.execute_bloom(&mut encoder, &view);
            self.post_process_pending = false;
        } else {
            self.draw_scene_to(&mut encoder, &view);
        }
        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width.max(1);
        self.config.height = height.max(1);
        self.surface.configure(&self.device, &self.config);
        self.msaa_view = Self::create_msaa_texture(
            &self.device,
            self.config.format,
            self.config.width,
            self.config.height,
            self.sample_count,
        );
        let cfg = self.bloom_config();
        if let Some(pp) = &mut self.post_process {
            pp.resize(&self.device, &cfg);
        }
    }
}
