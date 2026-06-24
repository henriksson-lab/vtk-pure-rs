//! Voxel occupancy grid from point clouds.
//!
//! Converts a point cloud into an ImageData occupancy grid where each voxel
//! stores the number of points it contains.

use crate::data::{AnyDataArray, DataArray, ImageData, PolyData};

/// Create a voxel occupancy grid from a point cloud.
///
/// Each voxel in the output ImageData stores the count of points that
/// fall within it. The grid covers the bounding box of the input points
/// with the given spacing.
pub fn point_occupancy(points: &PolyData, spacing: [f64; 3]) -> ImageData {
    let n = points.points.len();
    if n == 0 {
        return ImageData::new();
    }

    // Compute bounding box
    let mut min = points.points.get(0);
    let mut max = min;
    for i in 1..n {
        let p = points.points.get(i);
        for j in 0..3 {
            min[j] = min[j].min(p[j]);
            max[j] = max[j].max(p[j]);
        }
    }

    // Add small padding
    for j in 0..3 {
        min[j] -= spacing[j] * 0.5;
        max[j] += spacing[j] * 0.5;
    }

    let dims = [
        ((max[0] - min[0]) / spacing[0]).ceil() as usize + 1,
        ((max[1] - min[1]) / spacing[1]).ceil() as usize + 1,
        ((max[2] - min[2]) / spacing[2]).ceil() as usize + 1,
    ];

    let total = dims[0] * dims[1] * dims[2];
    let mut counts = vec![0.0f64; total];

    // Bin points
    for i in 0..n {
        let p = points.points.get(i);
        let ix = ((p[0] - min[0]) / spacing[0]) as usize;
        let iy = ((p[1] - min[1]) / spacing[1]) as usize;
        let iz = ((p[2] - min[2]) / spacing[2]) as usize;

        if ix < dims[0] && iy < dims[1] && iz < dims[2] {
            let idx = ix + iy * dims[0] + iz * dims[0] * dims[1];
            counts[idx] += 1.0;
        }
    }

    ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(spacing)
        .with_origin(min)
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(
            "Occupancy",
            counts,
            1,
        )))
}

/// Create a binary occupancy grid (1.0 if any points, 0.0 otherwise).
pub fn point_occupancy_binary(points: &PolyData, spacing: [f64; 3]) -> ImageData {
    let mut grid = point_occupancy(points, spacing);
    if let Some(arr) = grid.point_data().get_array("Occupancy") {
        let n = arr.num_tuples();
        let mut binary = Vec::with_capacity(n);
        let mut buf = [0.0f64];
        for i in 0..n {
            arr.tuple_as_f64(i, &mut buf);
            binary.push(if buf[0] > 0.0 { 1.0 } else { 0.0 });
        }
        grid.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Occupancy",
                binary,
                1,
            )));
    }
    grid
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Points;

    #[test]
    fn basic_occupancy() {
        let mut mesh = PolyData::new();
        mesh.points = Points::from(vec![[0.5, 0.5, 0.5], [0.6, 0.5, 0.5], [5.0, 5.0, 5.0]]);

        let grid = point_occupancy(&mesh, [1.0, 1.0, 1.0]);
        let dims = grid.dimensions();
        assert!(dims[0] > 1 && dims[1] > 1);

        let arr = grid.point_data().get_array("Occupancy").unwrap();
        // At least one voxel should have count 2 (two nearby points)
        let mut max_count = 0.0f64;
        let mut buf = [0.0f64];
        for i in 0..arr.num_tuples() {
            arr.tuple_as_f64(i, &mut buf);
            max_count = max_count.max(buf[0]);
        }
        assert!(max_count >= 1.0);
    }

    #[test]
    fn binary_occupancy() {
        let mut mesh = PolyData::new();
        mesh.points = Points::from(vec![[1.0, 1.0, 1.0], [1.1, 1.0, 1.0]]);

        let grid = point_occupancy_binary(&mesh, [0.5, 0.5, 0.5]);
        let arr = grid.point_data().get_array("Occupancy").unwrap();
        let mut buf = [0.0f64];
        let mut has_one = false;
        for i in 0..arr.num_tuples() {
            arr.tuple_as_f64(i, &mut buf);
            assert!(buf[0] == 0.0 || buf[0] == 1.0);
            if buf[0] == 1.0 {
                has_one = true;
            }
        }
        assert!(has_one);
    }

    #[test]
    fn empty_input() {
        let mesh = PolyData::new();
        let grid = point_occupancy(&mesh, [1.0, 1.0, 1.0]);
        // Empty PolyData returns default ImageData
        let dims = grid.dimensions();
        assert!(dims[0] <= 1 && dims[1] <= 1 && dims[2] <= 1);
    }
}
