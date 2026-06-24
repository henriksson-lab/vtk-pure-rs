use crate::data::{AnyDataArray, DataArray, ImageData};

/// Extract a sub-region of an ImageData by index extent.
///
/// `sub_extent` is `[i_min, i_max, j_min, j_max, k_min, k_max]` in index space.
/// Returns a new ImageData with matching spacing and adjusted origin.
pub fn extract_region(input: &ImageData, sub_extent: [usize; 6]) -> ImageData {
    let dims = input.dimensions();
    let i0 = sub_extent[0].min(dims[0].saturating_sub(1));
    let i1 = sub_extent[1].min(dims[0].saturating_sub(1));
    let j0 = sub_extent[2].min(dims[1].saturating_sub(1));
    let j1 = sub_extent[3].min(dims[1].saturating_sub(1));
    let k0 = sub_extent[4].min(dims[2].saturating_sub(1));
    let k1 = sub_extent[5].min(dims[2].saturating_sub(1));

    let new_nx = i1 - i0 + 1;
    let new_ny = j1 - j0 + 1;
    let new_nz = k1 - k0 + 1;

    let spacing = input.spacing();
    let origin = input.origin();
    let ext = input.extent();

    let new_origin = [
        origin[0] + (ext[0] as f64 + i0 as f64) * spacing[0],
        origin[1] + (ext[2] as f64 + j0 as f64) * spacing[1],
        origin[2] + (ext[4] as f64 + k0 as f64) * spacing[2],
    ];

    let mut result = ImageData::with_dimensions(new_nx, new_ny, new_nz);
    result.set_spacing(spacing);
    result.set_origin(new_origin);

    // Copy scalar data for the sub-region
    for arr_idx in 0..input.point_data().num_arrays() {
        let arr = match input.point_data().get_array_by_index(arr_idx) {
            Some(a) => a,
            None => continue,
        };
        let nc = arr.num_components();
        let mut data = Vec::with_capacity(new_nx * new_ny * new_nz * nc);
        let mut buf = vec![0.0f64; nc];

        for k in k0..=k1 {
            for j in j0..=j1 {
                for i in i0..=i1 {
                    let src_idx = input.index_from_ijk(i, j, k);
                    arr.tuple_as_f64(src_idx, &mut buf);
                    data.extend_from_slice(&buf);
                }
            }
        }

        let name = arr.name().to_string();
        let out_arr = AnyDataArray::F64(DataArray::from_vec(&name, data, nc));
        result.point_data_mut().add_array(out_arr);
        if result.point_data().scalars().is_none() {
            result.point_data_mut().set_active_scalars(&name);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::DataSet;

    #[test]
    fn extract_subregion() {
        let mut img = ImageData::with_dimensions(5, 5, 5);
        let n = img.num_points();
        let scalars: Vec<f64> = (0..n).map(|i| i as f64).collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("idx", scalars, 1)));
        img.point_data_mut().set_active_scalars("idx");

        let sub = extract_region(&img, [1, 3, 1, 3, 1, 3]);
        assert_eq!(sub.dimensions(), [3, 3, 3]);
        assert_eq!(sub.num_points(), 27);

        let s = sub.point_data().scalars().unwrap();
        assert_eq!(s.num_tuples(), 27);
    }

    #[test]
    fn full_extent() {
        let img = ImageData::with_dimensions(3, 3, 3);
        let sub = extract_region(&img, [0, 2, 0, 2, 0, 2]);
        assert_eq!(sub.dimensions(), [3, 3, 3]);
    }
}
