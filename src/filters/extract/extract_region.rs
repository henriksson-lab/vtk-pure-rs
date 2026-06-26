use crate::data::{AnyDataArray, DataArray, DataSetAttributes, ImageData};

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

    if i0 > i1 || j0 > j1 || k0 > k1 {
        return ImageData::new();
    }

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
        if arr.num_tuples() != input.num_points() {
            continue;
        }

        let mut tuple_ids = Vec::with_capacity(new_nx * new_ny * new_nz);
        for k in k0..=k1 {
            for j in j0..=j1 {
                for i in i0..=i1 {
                    tuple_ids.push(input.index_from_ijk(i, j, k));
                }
            }
        }

        let name = arr.name().to_string();
        result
            .point_data_mut()
            .add_array(select_tuples(arr, &tuple_ids));
        if result.point_data().scalars().is_none() {
            result.point_data_mut().set_active_scalars(&name);
        }
    }
    copy_active_attributes(input.point_data(), result.point_data_mut());

    result
}

fn select_tuples(array: &AnyDataArray, tuple_ids: &[usize]) -> AnyDataArray {
    macro_rules! select {
        ($array:expr, $variant:ident) => {{
            let mut out = DataArray::new($array.name(), $array.num_components());
            for &tuple_id in tuple_ids {
                out.push_tuple($array.tuple(tuple_id));
            }
            AnyDataArray::$variant(out)
        }};
    }

    match array {
        AnyDataArray::F32(a) => select!(a, F32),
        AnyDataArray::F64(a) => select!(a, F64),
        AnyDataArray::I8(a) => select!(a, I8),
        AnyDataArray::I16(a) => select!(a, I16),
        AnyDataArray::I32(a) => select!(a, I32),
        AnyDataArray::I64(a) => select!(a, I64),
        AnyDataArray::U8(a) => select!(a, U8),
        AnyDataArray::U16(a) => select!(a, U16),
        AnyDataArray::U32(a) => select!(a, U32),
        AnyDataArray::U64(a) => select!(a, U64),
    }
}

fn copy_active_attributes(input: &DataSetAttributes, output: &mut DataSetAttributes) {
    if let Some(array) = input.scalars() {
        output.set_active_scalars(array.name());
    }
    if let Some(array) = input.vectors() {
        output.set_active_vectors(array.name());
    }
    if let Some(array) = input.normals() {
        output.set_active_normals(array.name());
    }
    if let Some(array) = input.tcoords() {
        output.set_active_tcoords(array.name());
    }
    if let Some(array) = input.tensors() {
        output.set_active_tensors(array.name());
    }
    if let Some(array) = input.global_ids() {
        output.set_active_global_ids(array.name());
    }
    if let Some(array) = input.pedigree_ids() {
        output.set_active_pedigree_ids(array.name());
    }
    if let Some(array) = input.edge_flags() {
        output.set_active_edge_flags(array.name());
    }
    if let Some(array) = input.tangents() {
        output.set_active_tangents(array.name());
    }
    if let Some(array) = input.rational_weights() {
        output.set_active_rational_weights(array.name());
    }
    if let Some(array) = input.higher_order_degrees() {
        output.set_active_higher_order_degrees(array.name());
    }
    if let Some(array) = input.process_ids() {
        output.set_active_process_ids(array.name());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
