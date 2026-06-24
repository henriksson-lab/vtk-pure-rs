//! GPU-accelerated parallel reduction (sum).

use crate::data::DataArray;
use crate::filters::gpu::GpuContext;

/// Compute the sum of all elements in a DataArray<f32> using GPU parallel reduction.
pub fn gpu_reduce_sum(ctx: &GpuContext, input: &DataArray<f32>) -> f32 {
    let data = input.as_slice();
    let n = data.len();
    if n == 0 {
        return 0.0;
    }

    let shader = ctx
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("reduce shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("compute_reduce.wgsl").into()),
        });

    let workgroup_size = 256u32;
    let num_workgroups = (n as u32 + workgroup_size - 1) / workgroup_size;

    let input_buf = ctx.create_storage_buffer(data);
    let output_buf = ctx.create_output_buffer((num_workgroups as u64) * 4);

    let params = [n as f32, 0.0, 0.0, 0.0];
    let params_buf = {
        use wgpu::util::DeviceExt;
        ctx.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("params"),
                contents: bytemuck::cast_slice(&params),
                usage: wgpu::BufferUsages::UNIFORM,
            })
    };

    let bgl = ctx
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

    let pipeline_layout = ctx
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
        });

    let pipeline = ctx
        .device
        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("reduce pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

    let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: input_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: output_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: params_buf.as_entire_binding(),
            },
        ],
    });

    ctx.dispatch(&pipeline, &bind_group, num_workgroups);

    // Read back partial sums and finish on CPU
    let partial = ctx.read_buffer(&output_buf, (num_workgroups as u64) * 4);
    partial.iter().sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reduce_sum_basic() {
        let ctx = match GpuContext::new() {
            Ok(c) => c,
            Err(_) => return,
        };
        let input = DataArray::from_vec("v", vec![1.0f32; 1000], 1);
        let sum = gpu_reduce_sum(&ctx, &input);
        assert!((sum - 1000.0).abs() < 1.0, "sum should be ~1000, got {sum}");
    }
}
