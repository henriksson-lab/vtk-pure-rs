use bytemuck::{Pod, Zeroable};

use crate::render_wgpu::mesh::Vertex;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct PickUniforms {
    mvp: [[f32; 4]; 4],
    actor_id: u32,
    _pad: [u32; 3],
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
    /// Actor/prop index.
    pub actor_id: u32,
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
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
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
                    format: wgpu::TextureFormat::Rgba32Uint,
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
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });

        Self {
            pipeline,
            uniform_buffer,
            bind_group_layout,
            bind_group,
        }
    }

    /// Decode a VTK WebGPU selector ID tuple.
    ///
    /// The shader writes `{cell, prop, composite, process} + 1`, reserving zero
    /// for background pixels.
    pub fn decode_ids(
        cell: u32,
        actor: u32,
        _composite: u32,
        _process: u32,
    ) -> Option<GpuPickResult> {
        if actor == 0 || cell == 0 {
            return None;
        }
        Some(GpuPickResult {
            actor_id: actor - 1,
            cell_id: cell - 1,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_ids_background() {
        assert!(GpuPicker::decode_ids(0, 0, 0, 0).is_none());
    }

    #[test]
    fn decode_ids_valid() {
        let result = GpuPicker::decode_ids(6, 3, 1, 1).unwrap();
        assert_eq!(result.actor_id, 2);
        assert_eq!(result.cell_id, 5);
    }

    #[test]
    fn decode_ids_large_cell() {
        let result = GpuPicker::decode_ids(65_537, 1, 1, 1).unwrap();
        assert_eq!(result.cell_id, 65_536);
    }
}
