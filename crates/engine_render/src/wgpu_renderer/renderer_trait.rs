use std::{
    io::Write,
    path::{Path, PathBuf},
};

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

#[derive(Clone, Copy, PartialEq, Eq)]
enum BoundShapeSource {
    Batched,
    Persistent(crate::renderer::GpuMeshHandle),
}

const MODEL_UNIFORM_SIZE: usize = std::mem::size_of::<[[f32; 4]; 4]>();

struct PendingFrameCapture {
    request_file: PathBuf,
    output_file: PathBuf,
    buffer: wgpu::Buffer,
    bytes_per_row: u32,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
}

impl WgpuRenderer {
    fn begin_frame_capture(
        &self,
        texture: &wgpu::Texture,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Option<PendingFrameCapture> {
        let request_file = self.frame_capture_request_file.as_deref()?;
        let output_file = take_frame_capture_request(request_file)?;

        let (bytes_per_row, buffer_size) = match frame_capture_layout(
            self.surface_format,
            self.config.width,
            self.config.height,
        ) {
            Ok(layout) => layout,
            Err(error) => {
                report_frame_capture_result(request_file, Err(error));
                return None;
            }
        };
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ui-test-frame-capture"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        super::copy_texture_to_staging(
            encoder,
            texture,
            &buffer,
            self.config.width,
            self.config.height,
            bytes_per_row,
        );

        Some(PendingFrameCapture {
            request_file: request_file.to_path_buf(),
            output_file,
            buffer,
            bytes_per_row,
            width: self.config.width,
            height: self.config.height,
            format: self.surface_format,
        })
    }

    fn finish_frame_capture(&self, capture: PendingFrameCapture) {
        let result = save_frame_capture(&self.device, &capture);
        if let Err(error) = &result {
            tracing::error!("UI test frame capture failed: {error}");
        }
        report_frame_capture_result(&capture.request_file, result);
    }

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
                MODEL_UNIFORM_SIZE as u64,
                model_data.len(),
            )
        });
        model_upload.ensure_capacity(
            &self.device,
            &self.model_bind_group_layout,
            MODEL_UNIFORM_SIZE as u64,
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

    fn push_shape_draw(&mut self, source: MeshSource, model: [[f32; 4]; 4]) {
        let material_uniforms = self.pending_material.take_uniforms();
        let material_textures = self.pending_material.take_textures();
        self.shape_draws.push(ShapeDrawRecord {
            blend_mode: self.current_blend_mode,
            shader_handle: self.active_shader,
            source,
            model,
            material_uniforms,
            material_textures,
        });
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
        let mut last_source: Option<BoundShapeSource> = None;

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
                    if last_source != Some(BoundShapeSource::Batched) {
                        if let Some((vb, ib)) = batched_buffers {
                            pass.set_vertex_buffer(0, vb.slice(..));
                            pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
                        }
                        last_source = Some(BoundShapeSource::Batched);
                    }
                    pass.draw_indexed(*index_start..*index_start + *index_count, 0, 0..1);
                }
                MeshSource::Persistent { handle } => {
                    if let Some(mesh) = self.persistent_meshes.get(handle) {
                        if last_source != Some(BoundShapeSource::Persistent(*handle)) {
                            pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                            pass.set_index_buffer(
                                mesh.index_buffer.slice(..),
                                wgpu::IndexFormat::Uint32,
                            );
                            last_source = Some(BoundShapeSource::Persistent(*handle));
                        }
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
        let aligned_entry =
            align_to_uniform_offset(MODEL_UNIFORM_SIZE, self.model_uniform_align as usize);
        let model_bg = self.prepare_model_bind_group(aligned_entry)?;
        let (material_bg, material_entry_size) = self.prepare_material_bind_group()?;
        let batched_buffers = if self.shape_batch.is_empty() {
            None
        } else {
            self.prepare_shape_buffers()
        };
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
    let mut data = vec![0u8; draws.len() * aligned_entry];
    for (slot, draw) in data.chunks_exact_mut(aligned_entry).zip(draws) {
        slot[..MODEL_UNIFORM_SIZE].copy_from_slice(bytemuck::bytes_of(&draw.model));
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

fn save_frame_capture(device: &wgpu::Device, capture: &PendingFrameCapture) -> Result<(), String> {
    let slice = capture.buffer.slice(..);
    let (sender, receiver) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    let _ = device.poll(wgpu::Maintain::Wait);
    receiver
        .recv()
        .map_err(|error| format!("capture map callback failed: {error}"))?
        .map_err(|error| format!("capture buffer map failed: {error:?}"))?;

    let pixels = slice.get_mapped_range();
    let result = write_frame_bmp(
        &capture.output_file,
        capture.width,
        capture.height,
        capture.bytes_per_row,
        capture.format,
        &pixels,
    );
    drop(pixels);
    capture.buffer.unmap();
    result
}

fn take_frame_capture_request(request_file: &Path) -> Option<PathBuf> {
    if !request_file.exists() {
        return None;
    }

    let processing_file = request_file.with_extension("processing");
    if let Err(error) = std::fs::rename(request_file, &processing_file) {
        if request_file.exists() {
            report_frame_capture_result(
                request_file,
                Err(format!("could not claim capture request: {error}")),
            );
        }
        return None;
    }

    let output_path = std::fs::read_to_string(&processing_file)
        .map(|path| PathBuf::from(path.trim()))
        .map_err(|error| format!("could not read capture output path: {error}"))
        .and_then(|path| {
            if path.as_os_str().is_empty() {
                Err("capture output path was empty".to_owned())
            } else {
                Ok(path)
            }
        });
    let remove_result = std::fs::remove_file(&processing_file);
    if let Err(error) = remove_result {
        report_frame_capture_result(
            request_file,
            Err(format!("could not consume capture request: {error}")),
        );
        return None;
    }
    match output_path {
        Ok(path) => Some(path),
        Err(error) => {
            report_frame_capture_result(request_file, Err(error));
            None
        }
    }
}

fn frame_capture_layout(
    format: wgpu::TextureFormat,
    width: u32,
    height: u32,
) -> Result<(u32, u64), String> {
    if !matches!(
        format,
        wgpu::TextureFormat::Bgra8Unorm
            | wgpu::TextureFormat::Bgra8UnormSrgb
            | wgpu::TextureFormat::Rgba8Unorm
            | wgpu::TextureFormat::Rgba8UnormSrgb
    ) {
        return Err(format!(
            "unsupported surface format for BMP capture: {format:?}"
        ));
    }
    let row_bytes = width
        .checked_mul(4)
        .ok_or_else(|| "surface row size overflowed".to_owned())?;
    let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let bytes_per_row = row_bytes
        .div_ceil(alignment)
        .checked_mul(alignment)
        .ok_or_else(|| "aligned surface row size overflowed".to_owned())?;
    let buffer_size = u64::from(bytes_per_row)
        .checked_mul(u64::from(height))
        .ok_or_else(|| "surface staging buffer size overflowed".to_owned())?;
    Ok((bytes_per_row, buffer_size))
}

fn write_frame_bmp(
    path: &Path,
    width: u32,
    height: u32,
    bytes_per_row: u32,
    format: wgpu::TextureFormat,
    pixels: &[u8],
) -> Result<(), String> {
    let row_bytes = width
        .checked_mul(4)
        .ok_or_else(|| "BMP row size overflowed".to_owned())?;
    let image_size = row_bytes
        .checked_mul(height)
        .ok_or_else(|| "BMP image size overflowed".to_owned())?;
    let file_size = image_size
        .checked_add(54)
        .ok_or_else(|| "BMP file size overflowed".to_owned())?;
    let height_usize =
        usize::try_from(height).map_err(|_| "BMP height exceeds usize".to_owned())?;
    let width = i32::try_from(width).map_err(|_| "BMP width exceeds i32".to_owned())?;
    let height = i32::try_from(height).map_err(|_| "BMP height exceeds i32".to_owned())?;
    let source_pitch =
        usize::try_from(bytes_per_row).map_err(|_| "capture row pitch exceeds usize".to_owned())?;
    let row_bytes =
        usize::try_from(row_bytes).map_err(|_| "BMP row size exceeds usize".to_owned())?;
    if source_pitch < row_bytes {
        return Err("capture row pitch was smaller than the pixel row".to_owned());
    }
    let required_pixels = source_pitch
        .checked_mul(height_usize)
        .ok_or_else(|| "capture buffer size overflowed".to_owned())?;
    if pixels.len() < required_pixels {
        return Err("capture buffer was shorter than the surface dimensions".to_owned());
    }
    let bgra_format = matches!(
        format,
        wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb
    );
    let rgba_format = matches!(
        format,
        wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Rgba8UnormSrgb
    );
    if !bgra_format && !rgba_format {
        return Err(format!(
            "unsupported surface format for BMP capture: {format:?}"
        ));
    }

    let mut header = Vec::with_capacity(54);
    header.extend_from_slice(b"BM");
    header.extend_from_slice(&file_size.to_le_bytes());
    header.extend_from_slice(&0u32.to_le_bytes());
    header.extend_from_slice(&54u32.to_le_bytes());
    header.extend_from_slice(&40u32.to_le_bytes());
    header.extend_from_slice(&width.to_le_bytes());
    header.extend_from_slice(&height.to_le_bytes());
    header.extend_from_slice(&1u16.to_le_bytes());
    header.extend_from_slice(&32u16.to_le_bytes());
    header.extend_from_slice(&0u32.to_le_bytes());
    header.extend_from_slice(&image_size.to_le_bytes());
    header.extend_from_slice(&0i32.to_le_bytes());
    header.extend_from_slice(&0i32.to_le_bytes());
    header.extend_from_slice(&0u32.to_le_bytes());
    header.extend_from_slice(&0u32.to_le_bytes());

    let mut output = std::fs::File::create(path).map_err(|error| error.to_string())?;
    output
        .write_all(&header)
        .map_err(|error| error.to_string())?;
    let mut converted_row = vec![0; row_bytes];
    for row_index in (0..height_usize).rev() {
        let row_start = row_index * source_pitch;
        let row = &pixels[row_start..row_start + row_bytes];
        if bgra_format {
            output.write_all(row).map_err(|error| error.to_string())?;
        } else {
            for (source_pixel, bmp_pixel) in
                row.chunks_exact(4).zip(converted_row.chunks_exact_mut(4))
            {
                bmp_pixel.copy_from_slice(&[
                    source_pixel[2],
                    source_pixel[1],
                    source_pixel[0],
                    source_pixel[3],
                ]);
            }
            output
                .write_all(&converted_row)
                .map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn report_frame_capture_result(request_file: &Path, result: Result<(), String>) {
    let result_file = request_file.with_extension("result");
    let temporary_result_file = request_file.with_extension("result.tmp");
    let contents = match result {
        Ok(()) => "ok\n".to_owned(),
        Err(error) => format!("error={error}\n"),
    };
    if let Err(error) = std::fs::write(&temporary_result_file, contents)
        .and_then(|()| std::fs::rename(&temporary_result_file, &result_file))
    {
        tracing::error!("failed to report UI test frame capture result: {error}");
    }
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
        let index_start = self.shape_batch.index_count() as u32;
        self.shape_batch.push(vertices, indices, color);
        #[allow(clippy::cast_possible_truncation)]
        let index_count = (self.shape_batch.index_count() as u32) - index_start;
        self.push_shape_draw(
            MeshSource::Batched {
                index_start,
                index_count,
            },
            model,
        );
    }

    fn draw_colored_mesh(
        &mut self,
        vertices: &[crate::shape::ColorVertex],
        indices: &[u32],
        model: [[f32; 4]; 4],
    ) {
        #[allow(clippy::cast_possible_truncation)]
        let index_start = self.shape_batch.index_count() as u32;
        let shape_verts: &[ShapeVertex] = bytemuck::cast_slice(vertices);
        self.shape_batch.push_colored(shape_verts, indices);
        #[allow(clippy::cast_possible_truncation)]
        let index_count = (self.shape_batch.index_count() as u32) - index_start;
        self.push_shape_draw(
            MeshSource::Batched {
                index_start,
                index_count,
            },
            model,
        );
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
        self.push_shape_draw(MeshSource::Persistent { handle }, model);
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
        self.texture_lookups.clone_from(&atlas.lookups);
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
            Err(wgpu::SurfaceError::OutOfMemory) => panic!("GPU out of memory"),
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
            present_scene_post_process(self, &mut encoder, &view);
        } else {
            self.draw_scene_to(&mut encoder, &view);
        }
        let capture = self.begin_frame_capture(&frame.texture, &mut encoder);
        self.queue.submit(Some(encoder.finish()));
        frame.present();
        if let Some(capture) = capture {
            self.finish_frame_capture(capture);
        }
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

fn present_scene_post_process(
    renderer: &mut WgpuRenderer,
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
) {
    let scene_view = renderer
        .post_process
        .as_ref()
        .map(|pp| pp.scene_view.clone());
    let target = scene_view.as_ref().unwrap_or(view);
    renderer.draw_scene_to(encoder, target);
    renderer.execute_bloom(encoder, view);
    renderer.post_process_pending = false;
}

#[cfg(test)]
mod frame_capture_tests {
    use std::path::{Path, PathBuf};

    use super::{
        PendingFrameCapture, frame_capture_layout, report_frame_capture_result, save_frame_capture,
        take_frame_capture_request, write_frame_bmp,
    };

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "axiom2d-frame-capture-{}-{name}",
            std::process::id()
        ))
    }

    fn result_contents(request_file: &Path) -> String {
        std::fs::read_to_string(request_file.with_extension("result"))
            .expect("capture result should be readable")
    }

    #[test]
    fn consumes_capture_request_and_returns_output_path() {
        let request_file = temp_path("valid.request");
        let output_file = temp_path("valid.bmp");
        std::fs::write(&request_file, format!(" {} \n", output_file.display()))
            .expect("capture request should be writable");

        assert_eq!(take_frame_capture_request(&request_file), Some(output_file));
        assert!(!request_file.exists());
        assert!(!request_file.with_extension("processing").exists());
        assert!(!request_file.with_extension("result").exists());
    }

    #[test]
    fn reports_missing_output_path_errors() {
        let empty_request = temp_path("empty.request");
        std::fs::write(&empty_request, " \n").expect("empty capture request should be writable");
        assert_eq!(take_frame_capture_request(&empty_request), None);
        assert_eq!(
            result_contents(&empty_request),
            "error=capture output path was empty\n"
        );
        std::fs::remove_file(empty_request.with_extension("result"))
            .expect("empty capture result should be removable");

        let invalid_request = temp_path("invalid.request");
        std::fs::write(&invalid_request, [0xff])
            .expect("invalid capture request should be writable");
        assert_eq!(take_frame_capture_request(&invalid_request), None);
        assert!(
            result_contents(&invalid_request)
                .starts_with("error=could not read capture output path:")
        );
        std::fs::remove_file(invalid_request.with_extension("result"))
            .expect("invalid capture result should be removable");
    }

    #[test]
    fn reports_request_claim_and_consume_errors() {
        let claim_request = temp_path("claim.request");
        let claim_processing = claim_request.with_extension("processing");
        std::fs::write(&claim_request, "output.bmp").expect("claim request should be writable");
        std::fs::create_dir(&claim_processing).expect("claim processing path should be creatable");
        assert_eq!(take_frame_capture_request(&claim_request), None);
        assert!(
            result_contents(&claim_request).starts_with("error=could not claim capture request:")
        );
        std::fs::remove_file(&claim_request).expect("claim request should be removable");
        std::fs::remove_dir(claim_processing).expect("claim processing path should be removable");
        std::fs::remove_file(claim_request.with_extension("result"))
            .expect("claim result should be removable");

        let consume_request = temp_path("consume.request");
        std::fs::create_dir(&consume_request).expect("consume request path should be creatable");
        assert_eq!(take_frame_capture_request(&consume_request), None);
        assert!(
            result_contents(&consume_request)
                .starts_with("error=could not consume capture request:")
        );
        std::fs::remove_dir(consume_request.with_extension("processing"))
            .expect("consume processing path should be removable");
        std::fs::remove_file(consume_request.with_extension("result"))
            .expect("consume result should be removable");
    }

    #[test]
    fn reports_success_and_ignores_missing_requests() {
        let missing_request = temp_path("missing.request");
        assert_eq!(take_frame_capture_request(&missing_request), None);

        let request_file = temp_path("result.request");
        report_frame_capture_result(&request_file, Ok(()));
        assert_eq!(result_contents(&request_file), "ok\n");
        std::fs::remove_file(request_file.with_extension("result"))
            .expect("capture result should be removable");
    }

    #[test]
    fn validates_capture_layout_without_a_device() {
        assert_eq!(
            frame_capture_layout(wgpu::TextureFormat::Rgba8Unorm, 100, 2),
            Ok((512, 1024))
        );
        assert!(
            frame_capture_layout(wgpu::TextureFormat::R8Unorm, 1, 1)
                .expect_err("unsupported surface formats should be rejected")
                .starts_with("unsupported surface format for BMP capture:")
        );
        assert_eq!(
            frame_capture_layout(wgpu::TextureFormat::Rgba8Unorm, u32::MAX, 1),
            Err("surface row size overflowed".to_owned())
        );
        assert_eq!(
            frame_capture_layout(wgpu::TextureFormat::Rgba8Unorm, u32::MAX / 4, 1),
            Err("aligned surface row size overflowed".to_owned())
        );
    }

    #[test]
    fn reports_bmp_validation_and_file_errors() {
        let path = temp_path("errors.bmp");
        let error = |width, height, bytes_per_row, format, pixels: &[u8]| {
            write_frame_bmp(&path, width, height, bytes_per_row, format, pixels)
                .expect_err("invalid BMP input should return an error")
        };
        assert_eq!(
            error(u32::MAX, 1, u32::MAX, wgpu::TextureFormat::Rgba8Unorm, &[]),
            "BMP row size overflowed"
        );
        assert_eq!(
            error(
                u32::MAX / 4,
                2,
                u32::MAX,
                wgpu::TextureFormat::Rgba8Unorm,
                &[]
            ),
            "BMP image size overflowed"
        );
        assert_eq!(
            error(
                u32::MAX / 4,
                1,
                u32::MAX,
                wgpu::TextureFormat::Rgba8Unorm,
                &[]
            ),
            "BMP file size overflowed"
        );
        assert_eq!(
            error(1, 1, 3, wgpu::TextureFormat::Rgba8Unorm, &[]),
            "capture row pitch was smaller than the pixel row"
        );
        assert_eq!(
            error(1, 1, 4, wgpu::TextureFormat::Rgba8Unorm, &[]),
            "capture buffer was shorter than the surface dimensions"
        );
        assert!(
            error(1, 1, 4, wgpu::TextureFormat::R8Unorm, &[0; 4])
                .starts_with("unsupported surface format for BMP capture:")
        );

        std::fs::create_dir(&path).expect("capture output directory should be creatable");
        assert!(write_frame_bmp(&path, 1, 1, 4, wgpu::TextureFormat::Rgba8Unorm, &[0; 4]).is_err());
        std::fs::remove_dir(path).expect("capture output directory should be removable");
    }

    #[test]
    fn saves_mapped_frame_to_bmp_without_renderer() {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let Some(adapter) =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                force_fallback_adapter: true,
                ..Default::default()
            }))
        else {
            return;
        };
        let Ok((device, queue)) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None))
        else {
            return;
        };

        let output_file = temp_path("mapped.bmp");
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 256,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut pixels = vec![0xff; 256];
        pixels[..4].copy_from_slice(&[1, 2, 3, 4]);
        queue.write_buffer(&buffer, 0, &pixels);
        queue.submit([]);
        let _ = device.poll(wgpu::Maintain::Wait);
        let capture = PendingFrameCapture {
            request_file: temp_path("mapped.request"),
            output_file: output_file.clone(),
            buffer,
            bytes_per_row: 256,
            width: 1,
            height: 1,
            format: wgpu::TextureFormat::Rgba8Unorm,
        };
        assert_eq!(save_frame_capture(&device, &capture), Ok(()));
        let bmp = std::fs::read(&output_file).expect("captured BMP should be readable");
        assert_eq!(&bmp[54..], &[3, 2, 1, 4]);
        std::fs::remove_file(output_file).expect("captured BMP should be removable");
    }

    #[test]
    fn writes_padded_bgra_and_rgba_rows_to_bmp() -> Result<(), String> {
        let rgba_path = std::env::temp_dir().join(format!(
            "axiom2d-frame-capture-{}-rgba.bmp",
            std::process::id()
        ));
        let bgra_path = rgba_path.with_file_name(format!(
            "axiom2d-frame-capture-{}-bgra.bmp",
            std::process::id()
        ));
        let mut rgba_pixels = vec![0xff; 512];
        rgba_pixels[..4].copy_from_slice(&[1, 2, 3, 4]);
        rgba_pixels[256..260].copy_from_slice(&[5, 6, 7, 8]);
        let mut bgra_pixels = vec![0xff; 512];
        bgra_pixels[..4].copy_from_slice(&[3, 2, 1, 4]);
        bgra_pixels[256..260].copy_from_slice(&[7, 6, 5, 8]);

        write_frame_bmp(
            &rgba_path,
            1,
            2,
            256,
            wgpu::TextureFormat::Rgba8Unorm,
            &rgba_pixels,
        )?;
        write_frame_bmp(
            &bgra_path,
            1,
            2,
            256,
            wgpu::TextureFormat::Bgra8Unorm,
            &bgra_pixels,
        )?;

        let rgba_bmp = std::fs::read(&rgba_path).map_err(|error| error.to_string())?;
        let bgra_bmp = std::fs::read(&bgra_path).map_err(|error| error.to_string())?;
        assert_eq!(rgba_bmp.len(), 62);
        assert_eq!(&rgba_bmp[54..], &[7, 6, 5, 8, 3, 2, 1, 4]);
        assert_eq!(bgra_bmp, rgba_bmp);
        std::fs::remove_file(rgba_path).map_err(|error| error.to_string())?;
        std::fs::remove_file(bgra_path).map_err(|error| error.to_string())?;
        Ok(())
    }
}
