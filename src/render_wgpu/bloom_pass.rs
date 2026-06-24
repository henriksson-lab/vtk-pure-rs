use bytemuck::{Pod, Zeroable};

use crate::render::BloomConfig;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct BloomUniforms {
    threshold: f32,
    intensity: f32,
    texel_size: [f32; 2],
    horizontal: f32,
    _pad: [f32; 3],
    weights: [f32; 16], // up to 16-tap kernel
    num_weights: f32,
    _pad2: [f32; 3],
}

pub struct BloomPass {
    extract_pipeline: wgpu::RenderPipeline,
    blur_pipeline: wgpu::RenderPipeline,
    composite_pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    uniform_buffer: wgpu::Buffer,
    sampler: wgpu::Sampler,
    // Intermediate textures, lazily created
    bright_texture: Option<(wgpu::Texture, wgpu::TextureView)>,
    blur_texture: Option<(wgpu::Texture, wgpu::TextureView)>,
    tex_width: u32,
    tex_height: u32,
}

impl BloomPass {
    pub fn new(device: &wgpu::Device, color_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("bloom shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("bloom_shader.wgsl").into()),
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("bloom uniforms"),
            size: std::mem::size_of::<BloomUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("bloom sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bloom bgl"),
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
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("bloom pl"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let make_pipeline = |label: &str, entry: &str, blend: Option<wgpu::BlendState>| {
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
                        format: color_format,
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

        let extract_pipeline = make_pipeline("bloom extract", "fs_extract", None);
        let blur_pipeline = make_pipeline("bloom blur", "fs_blur", None);
        let composite_pipeline = make_pipeline(
            "bloom composite",
            "fs_composite",
            Some(wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent::OVER,
            }),
        );

        Self {
            extract_pipeline,
            blur_pipeline,
            composite_pipeline,
            bind_group_layout,
            uniform_buffer,
            sampler,
            bright_texture: None,
            blur_texture: None,
            tex_width: 0,
            tex_height: 0,
        }
    }

    pub fn ensure_textures(
        &mut self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) {
        // Use half resolution for bloom
        let w = (width / 2).max(1);
        let h = (height / 2).max(1);
        if self.tex_width == w && self.tex_height == h && self.bright_texture.is_some() {
            return;
        }
        let make_tex = |label: &str| {
            let tex = device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let view = tex.create_view(&Default::default());
            (tex, view)
        };
        self.bright_texture = Some(make_tex("bloom bright"));
        self.blur_texture = Some(make_tex("bloom blur"));
        self.tex_width = w;
        self.tex_height = h;
    }

    fn make_bind_group(
        &self,
        device: &wgpu::Device,
        texture_view: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bloom bg"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        })
    }

    /// Run bloom post-processing on the scene color target.
    /// `source_view` is the main rendered image (also the compositing target).
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        source_view: &wgpu::TextureView,
        config: &BloomConfig,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) {
        if !config.enabled {
            return;
        }

        self.ensure_textures(device, width, height, format);
        let bright_view = &self.bright_texture.as_ref().unwrap().1;
        let blur_view = &self.blur_texture.as_ref().unwrap().1;

        let weights = config.gaussian_weights(9);
        let mut weights_arr = [0.0f32; 16];
        for (i, &w) in weights.iter().enumerate().take(16) {
            weights_arr[i] = w as f32;
        }

        let tw = (width / 2).max(1) as f32;
        let th = (height / 2).max(1) as f32;

        // Pass 1: Extract bright pixels from source → bright texture
        {
            let uniforms = BloomUniforms {
                threshold: config.threshold as f32,
                intensity: config.intensity as f32,
                texel_size: [1.0 / width as f32, 1.0 / height as f32],
                horizontal: 0.0,
                _pad: [0.0; 3],
                weights: weights_arr,
                num_weights: weights.len() as f32,
                _pad2: [0.0; 3],
            };
            queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

            let bg = self.make_bind_group(device, source_view);
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("bloom extract"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: bright_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });
            pass.set_pipeline(&self.extract_pipeline);
            pass.set_bind_group(0, &bg, &[]);
            pass.draw(0..3, 0..1);
        }

        // Pass 2: Horizontal blur: bright → blur
        {
            let uniforms = BloomUniforms {
                threshold: config.threshold as f32,
                intensity: config.intensity as f32,
                texel_size: [1.0 / tw, 1.0 / th],
                horizontal: 1.0,
                _pad: [0.0; 3],
                weights: weights_arr,
                num_weights: weights.len() as f32,
                _pad2: [0.0; 3],
            };
            queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

            let bg = self.make_bind_group(device, bright_view);
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("bloom blur h"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: blur_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });
            pass.set_pipeline(&self.blur_pipeline);
            pass.set_bind_group(0, &bg, &[]);
            pass.draw(0..3, 0..1);
        }

        // Pass 3: Vertical blur: blur → bright
        {
            let uniforms = BloomUniforms {
                threshold: config.threshold as f32,
                intensity: config.intensity as f32,
                texel_size: [1.0 / tw, 1.0 / th],
                horizontal: 0.0,
                _pad: [0.0; 3],
                weights: weights_arr,
                num_weights: weights.len() as f32,
                _pad2: [0.0; 3],
            };
            queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

            let bg = self.make_bind_group(device, blur_view);
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("bloom blur v"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: bright_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });
            pass.set_pipeline(&self.blur_pipeline);
            pass.set_bind_group(0, &bg, &[]);
            pass.draw(0..3, 0..1);
        }

        // Pass 4: Additive composite: bright → source (additive blend)
        {
            let uniforms = BloomUniforms {
                threshold: config.threshold as f32,
                intensity: config.intensity as f32,
                texel_size: [1.0 / width as f32, 1.0 / height as f32],
                horizontal: 0.0,
                _pad: [0.0; 3],
                weights: weights_arr,
                num_weights: weights.len() as f32,
                _pad2: [0.0; 3],
            };
            queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

            let bg = self.make_bind_group(device, bright_view);
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("bloom composite"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: source_view,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bloom_uniforms_size() {
        // Verify alignment (must be 16-byte aligned for WebGPU)
        assert!(std::mem::size_of::<BloomUniforms>() % 16 == 0);
    }
}
