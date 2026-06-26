use crate::data::{AnyDataArray, DataArray, ImageData};
use crate::types::{Scalar, ScalarType};
use std::collections::VecDeque;

fn array_from_f64_values(
    name: &str,
    values: Vec<f64>,
    num_components: usize,
    scalar_type: ScalarType,
) -> AnyDataArray {
    fn cast_array<T: Scalar>(name: &str, values: Vec<f64>, num_components: usize) -> AnyDataArray
    where
        AnyDataArray: From<DataArray<T>>,
    {
        AnyDataArray::from(DataArray::from_vec(
            name,
            values.into_iter().map(T::from_f64).collect(),
            num_components,
        ))
    }

    match scalar_type {
        ScalarType::F32 => cast_array::<f32>(name, values, num_components),
        ScalarType::F64 => cast_array::<f64>(name, values, num_components),
        ScalarType::I8 => cast_array::<i8>(name, values, num_components),
        ScalarType::I16 => cast_array::<i16>(name, values, num_components),
        ScalarType::I32 => cast_array::<i32>(name, values, num_components),
        ScalarType::I64 => cast_array::<i64>(name, values, num_components),
        ScalarType::U8 => cast_array::<u8>(name, values, num_components),
        ScalarType::U16 => cast_array::<u16>(name, values, num_components),
        ScalarType::U32 => cast_array::<u32>(name, values, num_components),
        ScalarType::U64 => cast_array::<u64>(name, values, num_components),
    }
}

/// Flood fill starting from a seed voxel in an ImageData scalar field.
///
/// All connected voxels whose value is within `tolerance` of the seed voxel's
/// value are replaced with `fill_value`. Connectivity is 6-connected (face
/// neighbors only). The result is a new ImageData with the modified scalar array.
pub fn flood_fill(
    input: &ImageData,
    scalars: &str,
    seed: [usize; 3],
    fill_value: f64,
    tolerance: f64,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx: usize = dims[0] as usize;
    let ny: usize = dims[1] as usize;
    let nz: usize = dims[2] as usize;
    let total: usize = nx * ny * nz;
    let nc = arr.num_components();

    if seed[0] >= nx || seed[1] >= ny || seed[2] >= nz {
        return input.clone();
    }

    // Read all values into a mutable buffer
    let n: usize = arr.num_tuples();
    if n != total {
        return input.clone();
    }

    let mut values: Vec<f64> = vec![0.0; total * nc];
    let mut buf = vec![0.0f64; nc];
    for i in 0..total {
        arr.tuple_as_f64(i, &mut buf);
        values[i * nc..(i + 1) * nc].copy_from_slice(&buf);
    }

    // Index helper: VTK ImageData uses x-fastest ordering
    let idx = |x: usize, y: usize, z: usize| -> usize { x + y * nx + z * nx * ny };

    let seed_idx: usize = idx(seed[0], seed[1], seed[2]);
    let seed_value: f64 = values[seed_idx * nc];

    // BFS flood fill
    let mut visited: Vec<bool> = vec![false; total];
    let mut queue: VecDeque<[usize; 3]> = VecDeque::new();
    queue.push_back(seed);
    visited[seed_idx] = true;

    while let Some(pos) = queue.pop_front() {
        let ci: usize = idx(pos[0], pos[1], pos[2]);
        values[ci * nc] = fill_value;

        // 6-connected neighbors
        let neighbors: [[i64; 3]; 6] = [
            [-1, 0, 0],
            [1, 0, 0],
            [0, -1, 0],
            [0, 1, 0],
            [0, 0, -1],
            [0, 0, 1],
        ];

        for off in &neighbors {
            let nx2: i64 = pos[0] as i64 + off[0];
            let ny2: i64 = pos[1] as i64 + off[1];
            let nz2: i64 = pos[2] as i64 + off[2];

            if nx2 < 0
                || nx2 >= nx as i64
                || ny2 < 0
                || ny2 >= ny as i64
                || nz2 < 0
                || nz2 >= nz as i64
            {
                continue;
            }

            let nx2: usize = nx2 as usize;
            let ny2: usize = ny2 as usize;
            let nz2: usize = nz2 as usize;
            let ni: usize = idx(nx2, ny2, nz2);

            if !visited[ni] {
                let diff: f64 = (values[ni * nc] - seed_value).abs();
                if diff <= tolerance {
                    visited[ni] = true;
                    queue.push_back([nx2, ny2, nz2]);
                }
            }
        }
    }

    // Build output
    let mut img = input.clone();
    let mut new_attrs = crate::data::DataSetAttributes::new();
    for i in 0..input.point_data().num_arrays() {
        let a = input.point_data().get_array_by_index(i).unwrap();
        if a.name() == scalars {
            new_attrs.add_array(array_from_f64_values(
                scalars,
                values.clone(),
                nc,
                arr.scalar_type(),
            ));
        } else {
            new_attrs.add_array(a.clone());
        }
    }
    *img.point_data_mut() = new_attrs;
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_image_3x3() -> ImageData {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        // Uniform value of 1.0 except center = 5.0
        let mut values: Vec<f64> = vec![1.0; 9];
        values[4] = 5.0; // center voxel
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("Scalar", values, 1)));
        img
    }

    #[test]
    fn fill_connected_region() {
        let img = make_image_3x3();
        // Fill from corner (0,0,0), tolerance 0.5 => only voxels with value ~1.0
        let result = flood_fill(&img, "Scalar", [0, 0, 0], 99.0, 0.5);
        let arr = result.point_data().get_array("Scalar").unwrap();
        let mut buf = [0.0f64];

        // Corner should be filled
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 99.0);

        // Center (value 5.0) should NOT be filled (outside tolerance)
        arr.tuple_as_f64(4, &mut buf);
        assert_eq!(buf[0], 5.0);
    }

    #[test]
    fn fill_with_high_tolerance_fills_everything() {
        let img = make_image_3x3();
        let result = flood_fill(&img, "Scalar", [0, 0, 0], 42.0, 10.0);
        let arr = result.point_data().get_array("Scalar").unwrap();
        let mut buf = [0.0f64];
        for i in 0..9 {
            arr.tuple_as_f64(i, &mut buf);
            assert_eq!(buf[0], 42.0, "voxel {} not filled", i);
        }
    }

    #[test]
    fn seed_out_of_bounds_returns_clone() {
        let img = make_image_3x3();
        let result = flood_fill(&img, "Scalar", [10, 10, 10], 0.0, 1.0);
        let arr = result.point_data().get_array("Scalar").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0); // unchanged
    }

    #[test]
    fn fill_preserves_components_and_type() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec(
                "v",
                vec![1, 7, 1, 8, 5, 9],
                2,
            )));

        let result = flood_fill(&img, "v", [0, 0, 0], 42.0, 0.5);
        let arr = result.point_data().get_array("v").unwrap();
        assert_eq!(arr.num_components(), 2);
        assert_eq!(arr.scalar_type(), crate::types::ScalarType::I32);
        assert_eq!(arr.to_f64_vec_flat(), vec![42.0, 7.0, 42.0, 8.0, 5.0, 9.0]);
    }
}
