//! Screen-Space Ambient Occlusion (SSAO) post-processing pass.
//!
//! Computes ambient occlusion from the depth buffer using hemisphere sampling.
//! Two passes: AO computation → bilateral blur → applied as multiplier.

use bytemuck::{Pod, Zeroable};

pub use crate::render::SsaoConfig;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct SsaoUniforms {
    projection: [[f32; 4]; 4],
    inv_projection: [[f32; 4]; 4],
    radius: f32,
    bias: f32,
    intensity: f32,
    num_samples: f32,
    blur: f32,
    _pad0: f32,
    texel_size: [f32; 2],
    near: f32,
    far: f32,
    _pad1: [f32; 2],
    // 32 sample kernel positions (hemisphere)
    samples: [[f32; 4]; 32],
}

pub struct SsaoPass {
    ao_pipeline: wgpu::RenderPipeline,
    composite_pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    uniform_buffer: wgpu::Buffer,
    sampler: wgpu::Sampler,
    dummy_ao_view: wgpu::TextureView,
    ao_texture: Option<(wgpu::Texture, wgpu::TextureView)>,
    tex_width: u32,
    tex_height: u32,
}

impl SsaoPass {
    pub fn new(device: &wgpu::Device, color_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ssao shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("ssao_shader.wgsl").into()),
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ssao uniforms"),
            size: std::mem::size_of::<SsaoUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ssao sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        let dummy_ao_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ssao dummy ao"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let dummy_ao_view = dummy_ao_texture.create_view(&Default::default());

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ssao bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ssao pl"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let make_pipeline = |label: &str,
                             entry: &str,
                             format: wgpu::TextureFormat,
                             blend: Option<wgpu::BlendState>| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_fullscreen"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some(entry),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
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
        };

        // AO writes to an R8 texture.
        let ao_pipeline = make_pipeline("ssao ao", "fs_ssao", wgpu::TextureFormat::R8Unorm, None);
        // Composite multiplies onto the color target
        let composite_pipeline = make_pipeline(
            "ssao composite",
            "fs_composite",
            color_format,
            Some(wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::Zero,
                    dst_factor: wgpu::BlendFactor::Src,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent::OVER,
            }),
        );

        Self {
            ao_pipeline,
            composite_pipeline,
            bind_group_layout,
            uniform_buffer,
            sampler,
            dummy_ao_view,
            ao_texture: None,
            tex_width: 0,
            tex_height: 0,
        }
    }

    pub fn ensure_textures(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        if self.tex_width == width && self.tex_height == height && self.ao_texture.is_some() {
            return;
        }
        let make_tex = |label: &str| {
            let tex = device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::R8Unorm,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let view = tex.create_view(&Default::default());
            (tex, view)
        };
        self.ao_texture = Some(make_tex("ssao ao tex"));
        self.tex_width = width;
        self.tex_height = height;
    }

    /// Generate VTK-style hemisphere sample kernel.
    fn generate_kernel(num_samples: u32) -> [[f32; 4]; 32] {
        let mut samples = [[0.0f32; 4]; 32];
        let n = num_samples.min(32) as usize;
        let mut rng = VtkSsaoRng::new();
        let mut i = 0;
        while i < n {
            let mut sample = [
                rng.next_f32() * 2.0 - 1.0,
                rng.next_f32() * 2.0 - 1.0,
                rng.next_f32(),
            ];

            let norm =
                (sample[0] * sample[0] + sample[1] * sample[1] + sample[2] * sample[2]).sqrt();
            if norm > 1.0 {
                continue;
            }

            let scale = i as f32 / n as f32;
            let scale = 0.1 + 0.9 * scale * scale;
            sample[0] *= scale;
            sample[1] *= scale;
            sample[2] *= scale;
            samples[i] = [sample[0], sample[1], sample[2], 0.0];
            i += 1;
        }
        samples
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        config: &SsaoConfig,
        projection: glam::Mat4,
        width: u32,
        height: u32,
        near: f32,
        far: f32,
    ) {
        if !config.enabled {
            return;
        }

        self.ensure_textures(device, width, height);
        let ao_view = &self.ao_texture.as_ref().unwrap().1;

        let inv_proj = projection.inverse();
        let samples = Self::generate_kernel(config.num_samples);

        let uniforms = SsaoUniforms {
            projection: projection.to_cols_array_2d(),
            inv_projection: inv_proj.to_cols_array_2d(),
            radius: config.radius,
            bias: config.bias,
            intensity: config.intensity,
            num_samples: config.num_samples as f32,
            blur: if config.blur { 1.0 } else { 0.0 },
            _pad0: 0.0,
            texel_size: [1.0 / width as f32, 1.0 / height as f32],
            near,
            far,
            _pad1: [0.0; 2],
            samples,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

        // Pass 1: Compute AO from depth buffer
        {
            let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ssao ao bg"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.uniform_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(depth_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(&self.dummy_ao_view),
                    },
                ],
            });
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ssao ao pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: ao_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });
            pass.set_pipeline(&self.ao_pipeline);
            pass.set_bind_group(0, &bg, &[]);
            pass.draw(0..3, 0..1);
        }

        // Pass 2: Composite (multiply AO onto color target)
        {
            let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ssao composite bg"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.uniform_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(depth_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(ao_view),
                    },
                ],
            });
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ssao composite pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: color_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });
            pass.set_pipeline(&self.composite_pipeline);
            pass.set_bind_group(0, &bg, &[]);
            pass.draw(0..3, 0..1);
        }
    }
}

struct VtkSsaoRng {
    state: u32,
}

impl VtkSsaoRng {
    fn new() -> Self {
        Self { state: 1 }
    }

    fn next_f32(&mut self) -> f32 {
        self.state = self
            .state
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        ((self.state >> 8) as f32) * (1.0 / 16_777_216.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kernel_generation() {
        let kernel = SsaoPass::generate_kernel(16);
        // All z components should be positive (hemisphere)
        for i in 0..16 {
            assert!(
                kernel[i][2] >= 0.0,
                "sample {i} z={} should be >= 0",
                kernel[i][2]
            );
        }
    }

    #[test]
    fn vtk_ssao_rng_range() {
        let mut rng = VtkSsaoRng::new();
        for _ in 0..100 {
            let v = rng.next_f32();
            assert!(v >= 0.0 && v < 1.0);
        }
    }

    #[test]
    fn config_defaults() {
        let c = SsaoConfig::default();
        assert!(!c.enabled);
        assert_eq!(c.num_samples, 32);
    }
}
