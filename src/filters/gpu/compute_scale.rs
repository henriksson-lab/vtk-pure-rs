//! GPU-accelerated array scaling: output[i] = input[i] * scale + offset.

use crate::data::DataArray;
use crate::filters::gpu::GpuContext;

/// Scale a DataArray<f32> on the GPU: output[i] = input[i] * scale.
pub fn gpu_scale_array(ctx: &GpuContext, input: &DataArray<f32>, scale: f32) -> DataArray<f32> {
    gpu_scale_offset_array(ctx, input, scale, 0.0)
}

/// Scale and offset a DataArray<f32> on the GPU: output[i] = input[i] * scale + offset.
pub fn gpu_scale_offset_array(
    ctx: &GpuContext,
    input: &DataArray<f32>,
    scale: f32,
    offset: f32,
) -> DataArray<f32> {
    let data = input.as_slice();
    let n = data.len();
    if n == 0 {
        return DataArray::from_vec(input.name(), vec![], input.num_components());
    }

    let shader = ctx
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("scale shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("compute_scale.wgsl").into()),
        });

    let input_buf = ctx.create_storage_buffer(data);
    let output_buf = ctx.create_output_buffer((n * 4) as u64);

    let params = [scale, offset, n as f32, 0.0f32];
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
            label: Some("scale pipeline"),
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

    let workgroups = ((n as u32) + 255) / 256;
    ctx.dispatch(&pipeline, &bind_group, workgroups);

    let result = ctx.read_buffer(&output_buf, (n * 4) as u64);
    DataArray::from_vec(input.name(), result, input.num_components())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_array() {
        let ctx = match GpuContext::new() {
            Ok(c) => c,
            Err(_) => return, // no GPU available
        };
        let input = DataArray::from_vec("test", vec![1.0f32, 2.0, 3.0, 4.0], 1);
        let output = gpu_scale_array(&ctx, &input, 3.0);
        let data = output.as_slice();
        assert_eq!(data.len(), 4);
        assert!((data[0] - 3.0).abs() < 1e-5);
        assert!((data[1] - 6.0).abs() < 1e-5);
        assert!((data[2] - 9.0).abs() < 1e-5);
        assert!((data[3] - 12.0).abs() < 1e-5);
    }

    #[test]
    fn scale_offset() {
        let ctx = match GpuContext::new() {
            Ok(c) => c,
            Err(_) => return,
        };
        let input = DataArray::from_vec("v", vec![0.0f32, 1.0, 2.0], 1);
        let output = gpu_scale_offset_array(&ctx, &input, 2.0, 10.0);
        let data = output.as_slice();
        assert!((data[0] - 10.0).abs() < 1e-5);
        assert!((data[1] - 12.0).abs() < 1e-5);
        assert!((data[2] - 14.0).abs() < 1e-5);
    }
}
