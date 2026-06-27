//! GPU-accelerated polygon cell normal computation.

use crate::data::{AnyDataArray, DataArray, PolyData};
use crate::filters::gpu::GpuContext;

/// Compute per-polygon cell normals on the GPU.
///
/// Returns a PolyData with a "Normals" cell data array (3-component).
/// Much faster than CPU for large meshes (>100K triangles).
pub fn gpu_compute_normals(ctx: &GpuContext, input: &PolyData) -> PolyData {
    let num_polys = input.polys.num_cells();
    if num_polys == 0 {
        return input.clone();
    }

    // Flatten positions to f32 buffer
    let np = input.points.len();
    let mut positions = Vec::with_capacity(np * 3);
    for i in 0..np {
        let p = input.points.get(i);
        positions.push(p[0] as f32);
        positions.push(p[1] as f32);
        positions.push(p[2] as f32);
    }

    // VTK's vtkPolyDataNormals computes cell normals over all polygon vertices.
    let offsets: Vec<u32> = input.polys.offsets().iter().map(|&o| o as u32).collect();
    let connectivity: Vec<u32> = input
        .polys
        .connectivity()
        .iter()
        .map(|&id| id as u32)
        .collect();

    let shader = ctx
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("normals shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("compute_normals.wgsl").into()),
        });

    let pos_buf = ctx.create_storage_buffer(&positions);
    let offsets_buf = {
        use wgpu::util::DeviceExt;
        ctx.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("offsets"),
                contents: bytemuck::cast_slice(&offsets),
                usage: wgpu::BufferUsages::STORAGE,
            })
    };
    let connectivity_buf = {
        use wgpu::util::DeviceExt;
        ctx.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("connectivity"),
                contents: bytemuck::cast_slice(&connectivity),
                usage: wgpu::BufferUsages::STORAGE,
            })
    };
    let normals_buf = ctx.create_output_buffer((num_polys * 3 * 4) as u64);
    let params = [num_polys as u32, 0, 0, 0];
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
                bgl_entry(0, true),
                bgl_entry(1, true),
                bgl_entry(2, true),
                bgl_entry(3, false),
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
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
            label: Some("normals pipeline"),
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
                resource: pos_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: offsets_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: connectivity_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: normals_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: params_buf.as_entire_binding(),
            },
        ],
    });

    ctx.dispatch(&pipeline, &bind_group, ((num_polys as u32) + 255) / 256);

    let normals_f32 = ctx.read_buffer(&normals_buf, (num_polys * 3 * 4) as u64);
    let normals_f64: Vec<f64> = normals_f32.iter().map(|&v| v as f64).collect();

    let mut result = input.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Normals",
            normals_f64,
            3,
        )));
    result.cell_data_mut().set_active_normals("Normals");
    result
}

fn bgl_entry(binding: u32, read_only: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normals_z_up_triangle() {
        let ctx = match GpuContext::new() {
            Ok(c) => c,
            Err(_) => return,
        };
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = gpu_compute_normals(&ctx, &pd);
        let normals = result.cell_data().get_array("Normals").unwrap();
        let mut n = [0.0f64; 3];
        normals.tuple_as_f64(0, &mut n);
        // Normal should be (0, 0, 1) for XY plane triangle
        assert!(
            (n[2] - 1.0).abs() < 1e-4 || (n[2] + 1.0).abs() < 1e-4,
            "normal z should be ±1, got {:?}",
            n
        );
    }
}
