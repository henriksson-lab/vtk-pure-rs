//! Spatial data decomposition for distributed processing.
//!
//! Partitions meshes and grids into pieces for MPI distribution.

use crate::data::{AnyDataArray, DataArray, ImageData, PolyData};

/// A partition of a distributed dataset.
#[derive(Debug, Clone)]
pub struct Partition {
    /// Rank/process ID that owns this partition.
    pub rank: usize,
    /// The local data for this partition.
    pub data: PolyData,
    /// Global point IDs (maps local index → global index).
    pub global_point_ids: Vec<usize>,
    /// Global cell IDs.
    pub global_cell_ids: Vec<usize>,
}

/// Decompose a PolyData into N partitions using spatial bisection.
///
/// Recursively bisects along the longest axis of the bounding box.
/// Each partition gets roughly equal numbers of cells.
pub fn decompose_poly_data(input: &PolyData, num_partitions: usize) -> Vec<Partition> {
    if num_partitions <= 1 || input.polys.num_cells() == 0 {
        return vec![Partition {
            rank: 0,
            data: input.clone(),
            global_point_ids: (0..input.points.len()).collect(),
            global_cell_ids: (0..input.polys.num_cells()).collect(),
        }];
    }

    // Compute cell centroids
    let nc = input.polys.num_cells();
    let mut centroids: Vec<(usize, [f64; 3])> = Vec::with_capacity(nc);
    for ci in 0..nc {
        let cell = input.polys.cell(ci);
        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut cz = 0.0;
        for &vid in cell {
            let p = input.points.get(vid as usize);
            cx += p[0];
            cy += p[1];
            cz += p[2];
        }
        let n = cell.len() as f64;
        centroids.push((ci, [cx / n, cy / n, cz / n]));
    }

    // Find longest axis
    let (mut xmin, mut xmax) = (f64::MAX, f64::MIN);
    let (mut ymin, mut ymax) = (f64::MAX, f64::MIN);
    let (mut zmin, mut zmax) = (f64::MAX, f64::MIN);
    for (_, c) in &centroids {
        xmin = xmin.min(c[0]);
        xmax = xmax.max(c[0]);
        ymin = ymin.min(c[1]);
        ymax = ymax.max(c[1]);
        zmin = zmin.min(c[2]);
        zmax = zmax.max(c[2]);
    }
    let ranges = [xmax - xmin, ymax - ymin, zmax - zmin];
    let axis = if ranges[0] >= ranges[1] && ranges[0] >= ranges[2] {
        0
    } else if ranges[1] >= ranges[2] {
        1
    } else {
        2
    };

    // Sort by chosen axis
    centroids.sort_by(|a, b| {
        a.1[axis]
            .partial_cmp(&b.1[axis])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Split into num_partitions groups
    let chunk_size = (nc + num_partitions - 1) / num_partitions;
    let mut partitions = Vec::new();

    for (rank, chunk) in centroids.chunks(chunk_size).enumerate() {
        let cell_indices: Vec<usize> = chunk.iter().map(|(ci, _)| *ci).collect();
        let partition = extract_partition(input, rank, &cell_indices);
        partitions.push(partition);
    }

    partitions
}

/// Decompose ImageData into N slabs along the Z axis.
pub fn decompose_image_data(input: &ImageData, num_partitions: usize) -> Vec<ImageData> {
    let dims = input.dimensions();
    let nz = dims[2];
    if num_partitions <= 1 || nz <= 1 {
        return vec![input.clone()];
    }

    let slices_per_part = (nz + num_partitions - 1) / num_partitions;
    let spacing = input.spacing();
    let origin = input.origin();
    let mut parts = Vec::new();

    for p in 0..num_partitions {
        let z_start = p * slices_per_part;
        let z_end = ((p + 1) * slices_per_part).min(nz);
        if z_start >= nz {
            break;
        }
        let local_nz = z_end - z_start;

        let mut slab = ImageData::with_dimensions(dims[0], dims[1], local_nz);
        slab.set_spacing(spacing);
        slab.set_origin([
            origin[0],
            origin[1],
            origin[2] + z_start as f64 * spacing[2],
        ]);

        // Copy scalar data for this slab
        if let Some(arr) = input.point_data().get_array_by_index(0) {
            let nc = arr.num_components();
            let slice_size = dims[0] * dims[1];
            let mut data = Vec::with_capacity(local_nz * slice_size * nc);
            for z in z_start..z_end {
                for idx in 0..slice_size {
                    let global_idx = z * slice_size + idx;
                    let mut vals = vec![0.0f64; nc];
                    arr.tuple_as_f64(global_idx, &mut vals);
                    data.extend_from_slice(&vals);
                }
            }
            slab.point_data_mut()
                .add_array(AnyDataArray::F64(DataArray::from_vec(arr.name(), data, nc)));
        }

        parts.push(slab);
    }

    parts
}

fn extract_partition(input: &PolyData, rank: usize, cell_indices: &[usize]) -> Partition {
    let mut point_map = vec![usize::MAX; input.points.len()];
    let mut new_points = crate::data::Points::<f64>::new();
    let mut global_point_ids = Vec::new();
    let mut new_polys = crate::data::CellArray::new();

    for &ci in cell_indices {
        let cell = input.polys.cell(ci);
        for &vid in cell {
            let vi = vid as usize;
            if point_map[vi] == usize::MAX {
                point_map[vi] = new_points.len();
                global_point_ids.push(vi);
                new_points.push(input.points.get(vi));
            }
        }
        let remapped: Vec<i64> = cell.iter().map(|&v| point_map[v as usize] as i64).collect();
        new_polys.push_cell(&remapped);
    }

    let mut data = PolyData::new();
    data.points = new_points;
    data.polys = new_polys;

    Partition {
        rank,
        data,
        global_point_ids,
        global_cell_ids: cell_indices.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decompose_into_two() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
                [2.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let parts = decompose_poly_data(&pd, 2);
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].data.polys.num_cells(), 1);
        assert_eq!(parts[1].data.polys.num_cells(), 1);
        assert_eq!(parts[0].rank, 0);
        assert_eq!(parts[1].rank, 1);
    }

    #[test]
    fn decompose_single() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let parts = decompose_poly_data(&pd, 1);
        assert_eq!(parts.len(), 1);
    }

    #[test]
    fn decompose_image_slabs() {
        let mut img = ImageData::with_dimensions(4, 4, 8);
        img.set_spacing([1.0, 1.0, 1.0]);
        let vals = vec![0.0f64; 4 * 4 * 8];
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("s", vals, 1)));

        let slabs = decompose_image_data(&img, 4);
        assert_eq!(slabs.len(), 4);
        for slab in &slabs {
            assert_eq!(slab.dimensions()[2], 2);
        }
    }
}
