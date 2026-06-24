//! GPU-accelerated element-wise math operations.

use crate::data::DataArray;
use crate::filters::gpu::GpuContext;

/// Element-wise operation type.
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum MapOp {
    Abs = 0,
    Sqrt = 1,
    Log = 2,
    Exp = 3,
    Clamp = 4,
    Pow = 5,
    Sin = 6,
    Cos = 7,
    Square = 8,
    Reciprocal = 9,
}

/// Apply an element-wise math operation on the GPU.
///
/// `param1` and `param2` are used by Clamp (min, max) and Pow (exponent).
pub fn gpu_map_array(
    ctx: &GpuContext,
    input: &DataArray<f32>,
    op: MapOp,
    param1: f32,
    param2: f32,
) -> DataArray<f32> {
    let data = input.as_slice();
    let n = data.len();
    if n == 0 {
        return DataArray::from_vec(input.name(), vec![], input.num_components());
    }

    let shader = ctx
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("map shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("compute_map.wgsl").into()),
        });

    let input_buf = ctx.create_storage_buffer(data);
    let output_buf = ctx.create_output_buffer((n * 4) as u64);

    let params = [op as u32 as f32, param1, param2, n as f32];
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
            entries: &[bgl_storage_ro(0), bgl_storage_rw(1), bgl_uniform(2)],
        });

    let pipeline = create_compute_pipeline(ctx, &shader, &bgl);
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

    ctx.dispatch(&pipeline, &bind_group, ((n as u32) + 255) / 256);
    let result = ctx.read_buffer(&output_buf, (n * 4) as u64);
    DataArray::from_vec(input.name(), result, input.num_components())
}

fn bgl_storage_ro(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn bgl_storage_rw(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: false },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn bgl_uniform(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn create_compute_pipeline(
    ctx: &GpuContext,
    shader: &wgpu::ShaderModule,
    bgl: &wgpu::BindGroupLayout,
) -> wgpu::ComputePipeline {
    let layout = ctx
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[bgl],
            push_constant_ranges: &[],
        });
    ctx.device
        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&layout),
            module: shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gpu_sqrt() {
        let ctx = match GpuContext::new() {
            Ok(c) => c,
            Err(_) => return,
        };
        let input = DataArray::from_vec("v", vec![4.0f32, 9.0, 16.0, 25.0], 1);
        let output = gpu_map_array(&ctx, &input, MapOp::Sqrt, 0.0, 0.0);
        let d = output.as_slice();
        assert!((d[0] - 2.0).abs() < 1e-4);
        assert!((d[1] - 3.0).abs() < 1e-4);
        assert!((d[2] - 4.0).abs() < 1e-4);
    }
}
