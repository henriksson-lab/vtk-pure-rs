use crate::data::{AnyDataArray, DataArray, Points, PolyData};
use crate::types::{Scalar, ScalarType};
use std::collections::BTreeMap;

/// Voxel-based point cloud downsampling.
///
/// Divides space into a regular grid of voxels with the given `voxel_size`.
/// For each occupied voxel, emits a single representative point at the
/// centroid of all points falling within that voxel.
///
/// Returns a PolyData containing only the representative points.
pub fn voxel_grid(input: &PolyData, voxel_size: f64) -> PolyData {
    let n = input.points.len();
    if n == 0 || voxel_size <= 0.0 {
        return input.clone();
    }

    let bounds = input.points.bounds();
    let origin = [bounds.x_min, bounds.y_min, bounds.z_min];
    let size = [
        bounds.x_max - bounds.x_min,
        bounds.y_max - bounds.y_min,
        bounds.z_max - bounds.z_min,
    ];
    let divisions = [
        ((size[0] / voxel_size) as i64).max(1),
        ((size[1] / voxel_size) as i64).max(1),
        ((size[2] / voxel_size) as i64).max(1),
    ];
    let spacing = [
        inflate_zero_length(size[0]) / divisions[0] as f64,
        inflate_zero_length(size[1]) / divisions[1] as f64,
        inflate_zero_length(size[2]) / divisions[2] as f64,
    ];
    let mut bins: BTreeMap<i64, VoxelBin> = BTreeMap::new();

    for i in 0..n {
        let p = input.points.get(i);
        let ix = bucket_index(p[0], origin[0], spacing[0], divisions[0]);
        let iy = bucket_index(p[1], origin[1], spacing[1], divisions[1]);
        let iz = bucket_index(p[2], origin[2], spacing[2], divisions[2]);
        let key = ix + iy * divisions[0] + iz * divisions[0] * divisions[1];
        bins.entry(key).or_default().add_point(i, p);
    }

    let mut out_points = Points::<f64>::new();

    for bin in bins.values() {
        let nf = bin.point_ids.len() as f64;
        out_points.push([bin.sum[0] / nf, bin.sum[1] / nf, bin.sum[2] / nf]);
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    average_point_data(input, &bins, &mut pd);
    pd
}

fn inflate_zero_length(size: f64) -> f64 {
    if size == 0.0 {
        1.0
    } else {
        size
    }
}

fn bucket_index(value: f64, origin: f64, spacing: f64, divisions: i64) -> i64 {
    (((value - origin) / spacing) as i64).clamp(0, divisions - 1)
}

#[derive(Default)]
struct VoxelBin {
    sum: [f64; 3],
    point_ids: Vec<usize>,
}

impl VoxelBin {
    fn add_point(&mut self, point_id: usize, point: [f64; 3]) {
        self.sum[0] += point[0];
        self.sum[1] += point[1];
        self.sum[2] += point[2];
        self.point_ids.push(point_id);
    }
}

fn average_point_data(input: &PolyData, bins: &BTreeMap<i64, VoxelBin>, output: &mut PolyData) {
    for array in input.point_data().iter() {
        if array.num_tuples() != input.points.len() {
            continue;
        }

        let num_components = array.num_components();
        let mut values = Vec::with_capacity(bins.len() * num_components);
        let mut tuple = vec![0.0; num_components];
        let mut sum = vec![0.0; num_components];

        for bin in bins.values() {
            sum.fill(0.0);
            for &point_id in &bin.point_ids {
                array.tuple_as_f64(point_id, &mut tuple);
                for c in 0..num_components {
                    sum[c] += tuple[c];
                }
            }

            let inv_count = 1.0 / bin.point_ids.len() as f64;
            values.extend(sum.iter().map(|v| v * inv_count));
        }

        output.point_data_mut().add_array(cast_array(
            array.name(),
            values,
            num_components,
            array.scalar_type(),
        ));
    }
}

fn cast_array(
    name: &str,
    values: Vec<f64>,
    num_components: usize,
    scalar_type: ScalarType,
) -> AnyDataArray {
    fn cast<T: Scalar>(name: &str, values: Vec<f64>, num_components: usize) -> DataArray<T> {
        DataArray::from_vec(
            name,
            values.into_iter().map(T::from_f64).collect(),
            num_components,
        )
    }

    match scalar_type {
        ScalarType::F32 => AnyDataArray::F32(cast::<f32>(name, values, num_components)),
        ScalarType::F64 => AnyDataArray::F64(cast::<f64>(name, values, num_components)),
        ScalarType::I8 => AnyDataArray::I8(cast::<i8>(name, values, num_components)),
        ScalarType::I16 => AnyDataArray::I16(cast::<i16>(name, values, num_components)),
        ScalarType::I32 => AnyDataArray::I32(cast::<i32>(name, values, num_components)),
        ScalarType::I64 => AnyDataArray::I64(cast::<i64>(name, values, num_components)),
        ScalarType::U8 => AnyDataArray::U8(cast::<u8>(name, values, num_components)),
        ScalarType::U16 => AnyDataArray::U16(cast::<u16>(name, values, num_components)),
        ScalarType::U32 => AnyDataArray::U32(cast::<u32>(name, values, num_components)),
        ScalarType::U64 => AnyDataArray::U64(cast::<u64>(name, values, num_components)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn downsample_clusters() {
        let mut pd = PolyData::new();
        // 100 points clustered in 10 voxels
        for i in 0..100 {
            pd.points.push([(i % 10) as f64 * 0.01, 0.0, 0.0]);
        }
        let result = voxel_grid(&pd, 0.05);
        assert_eq!(result.points.len(), 1);
    }

    #[test]
    fn distinct_voxels() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([10.0, 0.0, 0.0]);
        let result = voxel_grid(&pd, 1.0);
        assert_eq!(result.points.len(), 2);
        assert_eq!(result.verts.num_cells(), 0);
    }

    #[test]
    fn single_voxel_centroid() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.1, 0.0, 0.0]);
        pd.points.push([0.2, 0.0, 0.0]);
        let result = voxel_grid(&pd, 1.0);
        assert_eq!(result.points.len(), 1);
        let p = result.points.get(0);
        assert!((p[0] - 0.1).abs() < 1e-10);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(voxel_grid(&pd, 1.0).points.len(), 0);
    }

    #[test]
    fn leaf_size_grid_is_anchored_to_input_bounds() {
        let mut pd = PolyData::new();
        pd.points.push([0.75, 0.0, 0.0]);
        pd.points.push([1.25, 0.0, 0.0]);
        pd.points.push([1.75, 0.0, 0.0]);

        let result = voxel_grid(&pd, 0.5);

        assert_eq!(result.points.len(), 2);
        assert_eq!(result.points.get(0), [0.75, 0.0, 0.0]);
        assert_eq!(result.points.get(1), [1.5, 0.0, 0.0]);
    }

    #[test]
    fn averages_point_data() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.1, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "values",
                vec![2.0, 4.0, 10.0],
                1,
            )));

        let result = voxel_grid(&pd, 0.5);
        let arr = result.point_data().get_array("values").unwrap();
        let mut value = [0.0];
        arr.tuple_as_f64(0, &mut value);
        assert!((value[0] - 3.0).abs() < 1e-10);
        arr.tuple_as_f64(1, &mut value);
        assert!((value[0] - 10.0).abs() < 1e-10);
    }
}
