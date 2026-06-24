use bytemuck::{Pod, Zeroable};
use glam::Mat4;

use crate::render::{LightType, Scene};

use crate::render_wgpu::mesh::Vertex;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct ShadowUniforms {
    light_vp: [[f32; 4]; 4],
    model: [[f32; 4]; 4],
}

pub struct ShadowPass {
    pipeline: wgpu::RenderPipeline,
    #[allow(dead_code)]
    bind_group_layout: wgpu::BindGroupLayout,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    depth_texture: Option<wgpu::Texture>,
    depth_view: Option<wgpu::TextureView>,
    resolution: u32,
}

impl ShadowPass {
    pub fn new(device: &wgpu::Device) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shadow shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shadow_shader.wgsl").into()),
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("shadow uniforms"),
            size: std::mem::size_of::<ShadowUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("shadow bgl"),
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

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shadow bg"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("shadow pl"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("shadow pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_shadow"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x3,
                        offset: 0,
                        shader_location: 0,
                    }],
                }],
                compilation_options: Default::default(),
            },
            fragment: None, // depth-only
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: Default::default(),
                bias: wgpu::DepthBiasState {
                    constant: 2,
                    slope_scale: 2.0,
                    clamp: 0.0,
                },
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            pipeline,
            bind_group_layout,
            uniform_buffer,
            bind_group,
            depth_texture: None,
            depth_view: None,
            resolution: 0,
        }
    }

    /// Ensure shadow map texture exists at the right resolution.
    fn ensure_texture(&mut self, device: &wgpu::Device, resolution: u32) {
        if self.resolution == resolution && self.depth_texture.is_some() {
            return;
        }
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("shadow map"),
            size: wgpu::Extent3d {
                width: resolution,
                height: resolution,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());
        self.depth_texture = Some(texture);
        self.depth_view = Some(view);
        self.resolution = resolution;
    }

    /// Render shadow depth pass. Returns the shadow map texture view and
    /// light VP matrix for use in the main pass.
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        scene: &Scene,
        meshes: &[(wgpu::Buffer, wgpu::Buffer, u32, [f64; 3], f64)], // (vb, ib, num_idx, pos, scale)
    ) -> Option<(&wgpu::TextureView, [[f32; 4]; 4])> {
        if !scene.shadows.enabled {
            return None;
        }

        // Find first directional light for shadow casting
        let dir_light = scene
            .lights
            .iter()
            .find(|l| l.enabled && matches!(l.light_type, LightType::Directional))?;

        let center = scene.camera.focal_point;
        let center_arr = [center.x, center.y, center.z];
        let light_dir = dir_light.direction;
        let light_vp = scene.shadows.light_vp_matrix(
            [
                light_dir[0] as f64,
                light_dir[1] as f64,
                light_dir[2] as f64,
            ],
            center_arr,
        );

        let light_vp_f32: [[f32; 4]; 4] = [
            [
                light_vp[0][0] as f32,
                light_vp[0][1] as f32,
                light_vp[0][2] as f32,
                light_vp[0][3] as f32,
            ],
            [
                light_vp[1][0] as f32,
                light_vp[1][1] as f32,
                light_vp[1][2] as f32,
                light_vp[1][3] as f32,
            ],
            [
                light_vp[2][0] as f32,
                light_vp[2][1] as f32,
                light_vp[2][2] as f32,
                light_vp[2][3] as f32,
            ],
            [
                light_vp[3][0] as f32,
                light_vp[3][1] as f32,
                light_vp[3][2] as f32,
                light_vp[3][3] as f32,
            ],
        ];

        self.ensure_texture(device, scene.shadows.resolution);
        let depth_view = self.depth_view.as_ref().unwrap();

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("shadow pass"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });

            pass.set_pipeline(&self.pipeline);

            for (vb, ib, num_indices, position, scale) in meshes {
                let model = Mat4::from_scale_rotation_translation(
                    glam::Vec3::splat(*scale as f32),
                    glam::Quat::IDENTITY,
                    glam::Vec3::new(position[0] as f32, position[1] as f32, position[2] as f32),
                );
                let light_vp_mat = Mat4::from_cols_array_2d(&light_vp_f32);
                let shadow_mvp = light_vp_mat * model;

                let uniforms = ShadowUniforms {
                    light_vp: shadow_mvp.to_cols_array_2d(),
                    model: model.to_cols_array_2d(),
                };
                queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

                pass.set_bind_group(0, &self.bind_group, &[]);
                pass.set_vertex_buffer(0, vb.slice(..));
                pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..*num_indices, 0, 0..1);
            }
        }

        Some((depth_view, light_vp_f32))
    }

    #[allow(dead_code)]
    pub fn depth_view(&self) -> Option<&wgpu::TextureView> {
        self.depth_view.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shadow_uniforms_size() {
        assert_eq!(std::mem::size_of::<ShadowUniforms>(), 128);
    }
}
