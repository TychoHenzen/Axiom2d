use std::path::Path;

use image::RgbaImage;
use wgpu::util::DeviceExt;

use engine_core::color::Color;
use engine_core::types::TextureId;

use crate::atlas::TextureAtlas;
use crate::material::{BlendMode, ShaderHandle};
use crate::rect::Rect;
use crate::renderer::Renderer;
use crate::wgpu_renderer::{
    Instance, QUAD_INDICES, QUAD_VERTICES, QuadVertex, SHADER_SRC, SHAPE_SHADER_SRC, ShapeBatch,
    ShapeVertex, blend_mode_to_blend_state, compute_batch_ranges, create_texture_bind_group,
    rect_to_instance,
};

const HEADLESS_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

pub struct HeadlessRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    output_texture: wgpu::Texture,
    width: u32,
    height: u32,
    quad_pipelines: [wgpu::RenderPipeline; 3],
    quad_vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_group: wgpu::BindGroup,
    camera_uniform_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    shape_pipelines: [wgpu::RenderPipeline; 3],
    clear_color: Color,
    pending_instances: Vec<Instance>,
    instance_blend_modes: Vec<BlendMode>,
    current_blend_mode: BlendMode,
    shape_batch: ShapeBatch,
    shape_blend_modes: Vec<BlendMode>,
    shape_index_offsets: Vec<u32>,
}

impl HeadlessRenderer {
    #[allow(clippy::too_many_lines)]
    pub fn new(width: u32, height: u32) -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            force_fallback_adapter: true,
            compatible_surface: None,
            ..Default::default()
        }))
        .expect("no software fallback adapter found");

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None))
                .expect("failed to create GPU device");

        let output_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: HEADLESS_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(SHADER_SRC.into()),
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let identity: [[f32; 4]; 4] = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let camera_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&identity),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        let quad_pipelines = BlendMode::ALL.map(|mode| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[
                        wgpu::VertexBufferLayout {
                            array_stride: size_of::<QuadVertex>() as wgpu::BufferAddress,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 0,
                                shader_location: 0,
                            }],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: size_of::<Instance>() as wgpu::BufferAddress,
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &[
                                wgpu::VertexAttribute {
                                    format: wgpu::VertexFormat::Float32x4,
                                    offset: 0,
                                    shader_location: 1,
                                },
                                wgpu::VertexAttribute {
                                    format: wgpu::VertexFormat::Float32x4,
                                    offset: 16,
                                    shader_location: 2,
                                },
                                wgpu::VertexAttribute {
                                    format: wgpu::VertexFormat::Float32x4,
                                    offset: 32,
                                    shader_location: 3,
                                },
                            ],
                        },
                    ],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: HEADLESS_FORMAT,
                        blend: Some(blend_mode_to_blend_state(mode)),
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
            })
        });

        let quad_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&QUAD_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let texture_bind_group =
            create_texture_bind_group(&device, &queue, &texture_bind_group_layout, 1, 1, &[255; 4]);

        let shape_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(SHAPE_SHADER_SRC.into()),
        });

        let shape_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let shape_pipelines = BlendMode::ALL.map(|mode| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&shape_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shape_shader,
                    entry_point: Some("vs_shape"),
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: size_of::<ShapeVertex>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 0,
                                shader_location: 0,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x4,
                                offset: 8,
                                shader_location: 1,
                            },
                        ],
                    }],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shape_shader,
                    entry_point: Some("fs_shape"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: HEADLESS_FORMAT,
                        blend: Some(blend_mode_to_blend_state(mode)),
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
            })
        });

        Self {
            device,
            queue,
            output_texture,
            width,
            height,
            quad_pipelines,
            quad_vertex_buffer,
            index_buffer,
            texture_bind_group_layout,
            texture_bind_group,
            camera_uniform_buffer,
            camera_bind_group,
            shape_pipelines,
            clear_color: Color::BLACK,
            pending_instances: Vec::new(),
            instance_blend_modes: Vec::new(),
            current_blend_mode: BlendMode::Alpha,
            shape_batch: ShapeBatch::new(),
            shape_blend_modes: Vec::new(),
            shape_index_offsets: Vec::new(),
        }
    }

    pub fn render_to_buffer(&mut self) -> Vec<u8> {
        let view = self
            .output_texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        self.draw_scene_to(&mut encoder, &view);

        let padded_row = padded_row_bytes(self.width, 4);
        let buffer_size = wgpu::BufferAddress::from(padded_row * self.height);
        let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
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

        self.queue.submit(Some(encoder.finish()));

        let slice = staging.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device.poll(wgpu::Maintain::Wait);

        let data = slice.get_mapped_range();
        let pixels = strip_row_padding(&data, self.width, self.height, padded_row, 4);
        drop(data);
        staging.unmap();

        self.pending_instances.clear();
        self.instance_blend_modes.clear();
        self.current_blend_mode = BlendMode::Alpha;
        self.shape_batch.clear();
        self.shape_blend_modes.clear();
        self.shape_index_offsets.clear();

        pixels
    }

    #[allow(clippy::cast_possible_truncation)]
    fn draw_scene_to(&self, encoder: &mut wgpu::CommandEncoder, target_view: &wgpu::TextureView) {
        let clear_color = wgpu::Color {
            r: f64::from(self.clear_color.r),
            g: f64::from(self.clear_color.g),
            b: f64::from(self.clear_color.b),
            a: f64::from(self.clear_color.a),
        };

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
        });

        if !self.pending_instances.is_empty() {
            let instance_buffer =
                self.device
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

        if !self.shape_batch.is_empty() {
            let vertex_buffer = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(self.shape_batch.vertices()),
                    usage: wgpu::BufferUsages::VERTEX,
                });

            let shape_index_buffer =
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::cast_slice(self.shape_batch.indices()),
                        usage: wgpu::BufferUsages::INDEX,
                    });

            pass.set_bind_group(0, &self.camera_bind_group, &[]);
            pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            pass.set_index_buffer(shape_index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            let total_indices = self.shape_batch.index_count() as u32;
            for (mode, shape_range) in compute_batch_ranges(&self.shape_blend_modes) {
                let idx_start = self.shape_index_offsets[shape_range.start as usize];
                let idx_end = if (shape_range.end as usize) < self.shape_index_offsets.len() {
                    self.shape_index_offsets[shape_range.end as usize]
                } else {
                    total_indices
                };
                pass.set_pipeline(&self.shape_pipelines[mode.index()]);
                pass.draw_indexed(idx_start..idx_end, 0, 0..1);
            }
        }
    }
}

impl Renderer for HeadlessRenderer {
    fn clear(&mut self, color: Color) {
        self.clear_color = color;
        self.pending_instances.clear();
        self.instance_blend_modes.clear();
        self.current_blend_mode = BlendMode::Alpha;
        self.shape_batch.clear();
        self.shape_blend_modes.clear();
        self.shape_index_offsets.clear();
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

    fn draw_shape(&mut self, vertices: &[[f32; 2]], indices: &[u32], color: Color) {
        self.shape_blend_modes.push(self.current_blend_mode);
        #[allow(clippy::cast_possible_truncation)]
        self.shape_index_offsets
            .push(self.shape_batch.index_count() as u32);
        self.shape_batch.push(vertices, indices, color);
    }

    fn set_blend_mode(&mut self, mode: BlendMode) {
        self.current_blend_mode = mode;
    }

    fn set_shader(&mut self, _shader: ShaderHandle) {}

    fn set_material_uniforms(&mut self, _data: &[u8]) {}

    fn bind_material_texture(&mut self, _texture: TextureId, _binding: u32) {}

    fn upload_atlas(&mut self, atlas: &TextureAtlas) {
        self.texture_bind_group = create_texture_bind_group(
            &self.device,
            &self.queue,
            &self.texture_bind_group_layout,
            atlas.width,
            atlas.height,
            &atlas.data,
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
        (self.width, self.height)
    }

    fn apply_post_process(&mut self) {}

    fn present(&mut self) {}

    fn resize(&mut self, _width: u32, _height: u32) {}
}

#[derive(Debug, thiserror::Error)]
pub enum GoldenError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("image error: {0}")]
    Image(#[from] image::ImageError),
}

pub fn save_golden(path: &Path, pixels: &[u8], width: u32, height: u32) -> Result<(), GoldenError> {
    let img = RgbaImage::from_raw(width, height, pixels.to_vec())
        .expect("pixel buffer must have exactly width * height * 4 bytes");
    img.save(path)?;
    Ok(())
}

pub fn load_golden(path: &Path) -> Result<(Vec<u8>, u32, u32), GoldenError> {
    let img = image::open(path)?.into_rgba8();
    let (w, h) = img.dimensions();
    Ok((img.into_raw(), w, h))
}

const COPY_BYTES_PER_ROW_ALIGNMENT: u32 = 256;

pub fn padded_row_bytes(width: u32, bytes_per_pixel: u32) -> u32 {
    let raw = width * bytes_per_pixel;
    let align = COPY_BYTES_PER_ROW_ALIGNMENT;
    raw.div_ceil(align) * align
}

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

pub fn ssim_compare(a: &[u8], b: &[u8], width: u32, height: u32) -> f32 {
    let img_a = RgbaImage::from_raw(width, height, a.to_vec())
        .expect("buffer a must have exactly width * height * 4 bytes");
    let img_b = RgbaImage::from_raw(width, height, b.to_vec())
        .expect("buffer b must have exactly width * height * 4 bytes");
    image_compare::rgba_hybrid_compare(&img_a, &img_b)
        .expect("images must have identical dimensions")
        .score as f32
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::{
        HeadlessRenderer, load_golden, padded_row_bytes, save_golden, ssim_compare,
        strip_row_padding,
    };

    #[test]
    fn when_comparing_identical_buffers_then_ssim_score_is_one() {
        // Arrange
        let a: Vec<u8> = vec![255, 0, 0, 255].repeat(64 * 64);
        let b: Vec<u8> = vec![255, 0, 0, 255].repeat(64 * 64);

        // Act
        let score = ssim_compare(&a, &b, 64, 64);

        // Assert
        assert!(
            (score - 1.0).abs() < f32::EPSILON,
            "identical pixel buffers must yield SSIM=1.0, got {score}"
        );
    }

    #[test]
    fn when_comparing_different_buffers_then_ssim_score_is_less_than_one() {
        // Arrange
        let a: Vec<u8> = vec![255, 0, 0, 255].repeat(64 * 64);
        let b: Vec<u8> = vec![0, 0, 255, 255].repeat(64 * 64);

        // Act
        let score = ssim_compare(&a, &b, 64, 64);

        // Assert
        assert!(
            score < 1.0,
            "different buffers must yield SSIM<1.0, got {score}"
        );
    }

    #[test]
    fn when_comparing_slightly_different_buffers_then_ssim_above_threshold() {
        // Arrange
        let a: Vec<u8> = vec![255, 0, 0, 255].repeat(64 * 64);
        let mut b = a.clone();
        b[0] = 254;

        // Act
        let score = ssim_compare(&a, &b, 64, 64);

        // Assert
        assert!(
            score >= 0.99,
            "single-pixel change in 64x64 must stay above 0.99 threshold, got {score}"
        );
    }

    #[test]
    fn when_computing_padded_row_bytes_then_returns_multiple_of_256() {
        // Act
        let result = padded_row_bytes(65, 4);

        // Assert
        assert_eq!(result, 512);
        assert_eq!(result % 256, 0);
    }

    #[test]
    fn when_width_already_aligned_then_padded_row_bytes_unchanged() {
        // Act
        let result = padded_row_bytes(64, 4);

        // Assert
        assert_eq!(result, 256);
    }

    #[test]
    fn when_stripping_row_padding_then_produces_packed_rgba() {
        // Arrange
        let width = 2u32;
        let height = 2u32;
        let padded = padded_row_bytes(width, 4) as usize;
        let mut data = vec![0u8; padded * height as usize];
        data[0..4].copy_from_slice(&[255, 0, 0, 255]);
        data[4..8].copy_from_slice(&[0, 255, 0, 255]);
        data[padded..padded + 4].copy_from_slice(&[0, 0, 255, 255]);
        data[padded + 4..padded + 8].copy_from_slice(&[255, 255, 255, 255]);

        // Act
        let packed = strip_row_padding(&data, width, height, padded as u32, 4);

        // Assert
        assert_eq!(packed.len(), 2 * 2 * 4);
        assert_eq!(&packed[0..4], &[255, 0, 0, 255]);
        assert_eq!(&packed[4..8], &[0, 255, 0, 255]);
        assert_eq!(&packed[8..12], &[0, 0, 255, 255]);
        assert_eq!(&packed[12..16], &[255, 255, 255, 255]);
    }

    use crate::rect::Rect;
    use crate::renderer::Renderer;
    use engine_core::types::Pixels;

    #[test]
    #[ignore] // requires GPU (software fallback adapter)
    fn when_creating_headless_renderer_then_viewport_matches() {
        // Arrange / Act
        let renderer = HeadlessRenderer::new(128, 128);

        // Assert
        assert_eq!(renderer.viewport_size(), (128, 128));
    }

    #[test]
    #[ignore] // requires GPU (software fallback adapter)
    fn when_clearing_with_red_then_readback_pixels_are_all_red() {
        // Arrange
        let mut renderer = HeadlessRenderer::new(64, 64);
        let red = engine_core::color::Color::new(1.0, 0.0, 0.0, 1.0);

        // Act
        renderer.clear(red);
        let pixels = renderer.render_to_buffer();

        // Assert
        assert_eq!(pixels.len(), 64 * 64 * 4);
        for chunk in pixels.chunks_exact(4) {
            assert_eq!(chunk[0], 255, "R channel");
            assert_eq!(chunk[1], 0, "G channel");
            assert_eq!(chunk[2], 0, "B channel");
            assert_eq!(chunk[3], 255, "A channel");
        }
    }

    #[test]
    fn when_saving_golden_image_then_file_exists_at_expected_path() {
        // Arrange
        let dir = std::env::temp_dir().join("axiom2d_golden_test_save");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.png");
        let pixels: Vec<u8> = vec![255, 0, 0, 255].repeat(4 * 4);

        // Act
        save_golden(&path, &pixels, 4, 4).unwrap();

        // Assert
        assert!(path.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn when_loading_saved_golden_then_pixels_match_original() {
        // Arrange
        let dir = std::env::temp_dir().join("axiom2d_golden_test_roundtrip");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("roundtrip.png");
        let original: Vec<u8> = vec![255, 0, 0, 255].repeat(4 * 4);
        save_golden(&path, &original, 4, 4).unwrap();

        // Act
        let (loaded, w, h) = load_golden(&path).unwrap();

        // Assert
        assert_eq!(w, 4);
        assert_eq!(h, 4);
        assert_eq!(loaded, original);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn when_loading_nonexistent_golden_then_returns_error() {
        // Arrange
        let path = std::path::Path::new("/nonexistent/golden.png");

        // Act
        let result = load_golden(path);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn when_comparing_largely_different_buffers_then_ssim_below_threshold() {
        // Arrange
        let mut a: Vec<u8> = vec![255, 0, 0, 255].repeat(64 * 64);
        for y in 0..32 {
            for x in 0..32 {
                let idx = (y * 64 + x) * 4;
                a[idx] = 0;
                a[idx + 2] = 255;
            }
        }
        let b: Vec<u8> = vec![255, 0, 0, 255].repeat(64 * 64);

        // Act
        let score = ssim_compare(&a, &b, 64, 64);

        // Assert
        assert!(
            score < 0.99,
            "25% different pixels must fail 0.99 threshold, got {score}"
        );
    }

    #[test]
    #[ignore] // requires GPU
    fn when_drawing_white_rect_on_black_then_rect_region_is_white() {
        // Arrange
        let mut renderer = HeadlessRenderer::new(64, 64);
        let black = engine_core::color::Color::new(0.0, 0.0, 0.0, 1.0);
        renderer.clear(black);

        let proj = crate::camera::CameraUniform::from_camera(
            &crate::camera::Camera2D {
                position: glam::Vec2::new(32.0, 32.0),
                zoom: 1.0,
            },
            64.0,
            64.0,
        );
        renderer.set_view_projection(proj.view_proj);

        let white_rect = Rect {
            x: Pixels(16.0),
            y: Pixels(16.0),
            width: Pixels(32.0),
            height: Pixels(32.0),
            color: engine_core::color::Color::WHITE,
        };
        renderer.draw_rect(white_rect);

        // Act
        let pixels = renderer.render_to_buffer();

        // Assert
        let center_idx = (32 * 64 + 32) * 4;
        assert_eq!(pixels[center_idx], 255, "center R");
        assert_eq!(pixels[center_idx + 1], 255, "center G");
        assert_eq!(pixels[center_idx + 2], 255, "center B");
        assert_eq!(pixels[0], 0, "corner R");
        assert_eq!(pixels[1], 0, "corner G");
        assert_eq!(pixels[2], 0, "corner B");
    }

    #[test]
    #[ignore] // requires GPU
    fn when_rendering_same_scene_twice_then_buffers_are_identical() {
        // Arrange
        let mut renderer = HeadlessRenderer::new(64, 64);
        let blue = engine_core::color::Color::new(0.0, 0.0, 1.0, 1.0);

        // Act
        renderer.clear(blue);
        let pixels_a = renderer.render_to_buffer();
        renderer.clear(blue);
        let pixels_b = renderer.render_to_buffer();

        // Assert
        assert_eq!(
            pixels_a, pixels_b,
            "two renders of the same scene must be identical"
        );
    }

    #[test]
    #[ignore] // requires GPU
    fn when_rendered_frame_compared_to_golden_then_ssim_passes_threshold() {
        // Arrange
        let mut renderer = HeadlessRenderer::new(64, 64);
        let blue = engine_core::color::Color::new(0.0, 0.0, 1.0, 1.0);
        renderer.clear(blue);
        let pixels = renderer.render_to_buffer();

        let golden = pixels.clone();

        // Act
        let score = ssim_compare(&pixels, &golden, 64, 64);

        // Assert
        assert!(
            score >= 0.99,
            "identical render vs golden must pass 0.99 threshold, got {score}"
        );
    }

    #[test]
    #[ignore] // requires GPU
    fn when_rendered_frame_differs_from_golden_then_ssim_fails_threshold() {
        // Arrange
        let mut renderer = HeadlessRenderer::new(64, 64);
        let blue = engine_core::color::Color::new(0.0, 0.0, 1.0, 1.0);
        renderer.clear(blue);
        let blue_pixels = renderer.render_to_buffer();

        let red = engine_core::color::Color::new(1.0, 0.0, 0.0, 1.0);
        renderer.clear(red);
        let red_pixels = renderer.render_to_buffer();

        // Act
        let score = ssim_compare(&red_pixels, &blue_pixels, 64, 64);

        // Assert
        assert!(
            score < 0.99,
            "different render vs golden must fail 0.99 threshold, got {score}"
        );
    }

    #[test]
    #[ignore] // requires GPU
    fn when_rendering_circle_shape_then_center_pixel_is_non_background() {
        // Arrange
        let mut renderer = HeadlessRenderer::new(128, 128);
        let black = engine_core::color::Color::new(0.0, 0.0, 0.0, 1.0);
        renderer.clear(black);

        let proj = crate::camera::CameraUniform::from_camera(
            &crate::camera::Camera2D {
                position: glam::Vec2::new(64.0, 64.0),
                zoom: 1.0,
            },
            128.0,
            128.0,
        );
        renderer.set_view_projection(proj.view_proj);

        let mesh = crate::shape::tessellate(&crate::shape::ShapeVariant::Circle { radius: 20.0 });
        let vertices: Vec<[f32; 2]> = mesh
            .vertices
            .iter()
            .map(|v| [v[0] + 64.0, v[1] + 64.0])
            .collect();
        renderer.draw_shape(&vertices, &mesh.indices, engine_core::color::Color::WHITE);

        // Act
        let pixels = renderer.render_to_buffer();

        // Assert
        let idx = (64 * 128 + 64) * 4;
        let is_non_black = pixels[idx] > 0 || pixels[idx + 1] > 0 || pixels[idx + 2] > 0;
        assert!(
            is_non_black,
            "center pixel should be non-black after drawing circle, got [{}, {}, {}]",
            pixels[idx],
            pixels[idx + 1],
            pixels[idx + 2]
        );
    }
}
