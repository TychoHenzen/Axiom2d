use wgpu::util::DeviceExt;

use engine_core::color::Color;

use crate::rect::Rect;
use crate::renderer::Renderer;
use crate::shader::ShaderHandle;

use super::gpu_init::create_shape_pipeline_set;
use super::renderer::{ShapeDrawRecord, WgpuRenderer};
use super::types::{
    BloomParamsUniform, FullscreenBuffers, FullscreenPass, QUAD_INDICES, TextureData,
    compute_batch_ranges, create_texture_bind_group, rect_to_instance, run_fullscreen_pass,
};

impl WgpuRenderer {
    fn begin_scene_pass<'a>(
        encoder: &'a mut wgpu::CommandEncoder,
        target_view: &'a wgpu::TextureView,
        clear_color: wgpu::Color,
    ) -> wgpu::RenderPass<'a> {
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear_color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        })
    }

    #[allow(clippy::cast_possible_truncation)]
    fn draw_quad_instances(&self, pass: &mut wgpu::RenderPass) {
        let instance_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&self.pending_instances),
                usage: wgpu::BufferUsages::VERTEX,
            });
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

    fn create_shape_buffers(&self) -> (wgpu::Buffer, wgpu::Buffer) {
        let vb = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(self.shape_batch.vertices()),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let ib = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(self.shape_batch.indices()),
                usage: wgpu::BufferUsages::INDEX,
            });
        (vb, ib)
    }

    fn create_model_bind_group(&self, aligned_entry: usize) -> wgpu::BindGroup {
        let model_data = pack_model_uniform_data(&self.shape_draws, aligned_entry);
        let model_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: &model_data,
                usage: wgpu::BufferUsages::UNIFORM,
            });
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.model_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &model_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(64),
                }),
            }],
        })
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
        aligned_entry: usize,
    ) {
        let total_indices = self.shape_batch.index_count() as u32;
        let mut last_key: Option<(ShaderHandle, crate::material::BlendMode)> = None;
        for (i, draw) in self.shape_draws.iter().enumerate() {
            let key = (draw.shader_handle, draw.blend_mode);
            if last_key != Some(key) {
                pass.set_pipeline(self.select_shape_pipeline(key));
                last_key = Some(key);
            }
            let dyn_offset = (i * aligned_entry) as u32;
            pass.set_bind_group(1, model_bg, &[dyn_offset]);
            let idx_start = draw.index_offset;
            let idx_end = self
                .shape_draws
                .get(i + 1)
                .map_or(total_indices, |d| d.index_offset);
            pass.draw_indexed(idx_start..idx_end, 0, 0..1);
        }
    }

    fn draw_shape_batches(&self, pass: &mut wgpu::RenderPass) {
        let (vb, ib) = self.create_shape_buffers();
        let aligned_entry = (self.model_uniform_align as usize).max(64);
        let model_bg = self.create_model_bind_group(aligned_entry);
        pass.set_bind_group(0, &self.camera_bind_group, &[]);
        pass.set_vertex_buffer(0, vb.slice(..));
        pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
        self.issue_shape_draw_calls(pass, &model_bg, aligned_entry);
    }

    pub(super) fn draw_scene_to(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target_view: &wgpu::TextureView,
    ) {
        let clear_color = wgpu::Color {
            r: f64::from(self.clear_color.r),
            g: f64::from(self.clear_color.g),
            b: f64::from(self.clear_color.b),
            a: f64::from(self.clear_color.a),
        };
        let mut pass = Self::begin_scene_pass(encoder, target_view, clear_color);
        if !self.pending_instances.is_empty() {
            self.draw_quad_instances(&mut pass);
        }
        if !self.shape_batch.is_empty() {
            self.draw_shape_batches(&mut pass);
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
        #[allow(clippy::cast_possible_truncation)]
        self.shape_draws.push(ShapeDrawRecord {
            blend_mode: self.current_blend_mode,
            shader_handle: self.active_shader,
            index_offset: self.shape_batch.index_count() as u32,
            model,
        });
        self.shape_batch.push(vertices, indices, color);
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

    fn set_material_uniforms(&mut self, _data: &[u8]) {}

    fn bind_material_texture(&mut self, _texture: engine_core::types::TextureId, _binding: u32) {}

    fn compile_shader(&mut self, handle: ShaderHandle, source: &str) {
        if self.shader_cache.contains_key(&handle) {
            return;
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
        );
        self.shader_cache.insert(handle, pipelines);
    }

    fn upload_atlas(&mut self, atlas: &crate::atlas::TextureAtlas) {
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
                eprintln!("surface error: {e}");
                return;
            }
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        if self.post_process_pending
            && let Some(pp) = self.post_process.as_ref()
        {
            self.draw_scene_to(&mut encoder, &pp.scene_view);
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
        let cfg = self.bloom_config();
        if let Some(pp) = &mut self.post_process {
            pp.resize(&self.device, &cfg);
        }
    }
}
