use bytemuck::{Pod, Zeroable};

use crate::render_wgpu::mesh::Vertex;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct PickUniforms {
    mvp: [[f32; 4]; 4],
    actor_id: f32,
    _pad: [f32; 3],
}

/// GPU-accelerated picker that renders actor/cell IDs to an offscreen buffer.
#[allow(dead_code)]
pub struct GpuPicker {
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

/// Result of a GPU pick operation.
#[derive(Debug, Clone, Copy)]
pub struct GpuPickResult {
    /// Actor index (0-254, 255 = background).
    pub actor_id: u8,
    /// Cell (triangle) index within the actor.
    pub cell_id: u32,
}

impl GpuPicker {
    pub fn new(device: &wgpu::Device) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("pick shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("pick_shader.wgsl").into()),
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("pick uniforms"),
            size: std::mem::size_of::<PickUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("pick bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("pick bg"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pick pl"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("pick pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_pick"),
                buffers: &[Vertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_pick"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });

        Self {
            pipeline,
            uniform_buffer,
            bind_group_layout,
            bind_group,
        }
    }

    /// Decode a pixel's RGBA values into a pick result.
    pub fn decode_pixel(r: u8, g: u8, b: u8, a: u8) -> Option<GpuPickResult> {
        if a == 0 {
            return None;
        } // background
        let actor_id = r;
        if actor_id == 255 {
            return None;
        }
        let cell_id = ((g as u32) << 8) | (b as u32);
        Some(GpuPickResult { actor_id, cell_id })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_pixel_background() {
        assert!(GpuPicker::decode_pixel(0, 0, 0, 0).is_none());
    }

    #[test]
    fn decode_pixel_valid() {
        let result = GpuPicker::decode_pixel(2, 0, 5, 255).unwrap();
        assert_eq!(result.actor_id, 2);
        assert_eq!(result.cell_id, 5);
    }

    #[test]
    fn decode_pixel_large_cell() {
        let result = GpuPicker::decode_pixel(0, 1, 0, 255).unwrap();
        assert_eq!(result.cell_id, 256);
    }
}
