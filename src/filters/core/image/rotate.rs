//! Image rotation (90/180/270 degrees and arbitrary).

use crate::data::{AnyDataArray, DataArray, ImageData};
use crate::types::{Scalar, ScalarType};

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

fn reindex_array<F>(
    arr: &AnyDataArray,
    old_dims: [usize; 3],
    new_dims: [usize; 3],
    source_index: F,
) -> Option<AnyDataArray>
where
    F: Fn(usize, usize, usize) -> usize,
{
    let old_n = old_dims[0] * old_dims[1] * old_dims[2];
    let new_n = new_dims[0] * new_dims[1] * new_dims[2];
    let num_components = arr.num_components();
    if arr.num_tuples() != old_n {
        return None;
    }

    let mut buf = vec![0.0f64; num_components];
    let mut values = Vec::with_capacity(new_n * num_components);
    for z in 0..new_dims[2] {
        for y in 0..new_dims[1] {
            for x in 0..new_dims[0] {
                arr.tuple_as_f64(source_index(x, y, z), &mut buf);
                values.extend_from_slice(&buf);
            }
        }
    }

    Some(array_from_f64_values(
        arr.name(),
        values,
        num_components,
        arr.scalar_type(),
    ))
}

fn transform_image<F>(
    input: &ImageData,
    scalars: &str,
    new_dims: [usize; 3],
    new_extent: [i64; 6],
    source_index: F,
    spacing: [f64; 3],
) -> ImageData
where
    F: Copy + Fn(usize, usize, usize) -> usize,
{
    let old_dims = input.dimensions();
    let n = old_dims[0] * old_dims[1] * old_dims[2];
    match input.point_data().get_array(scalars) {
        Some(a) if a.num_tuples() == n => {}
        _ => return input.clone(),
    }

    let mut result = input.clone();
    result.set_extent(new_extent);
    result.set_spacing(spacing);
    let mut new_attrs = input.point_data().clone();
    for i in 0..input.point_data().num_arrays() {
        let a = input.point_data().get_array_by_index(i).unwrap();
        new_attrs.field_data_mut().add_array(
            reindex_array(a, old_dims, new_dims, source_index).unwrap_or_else(|| a.clone()),
        );
    }
    *result.point_data_mut() = new_attrs;
    result
}

/// Rotate image 90 degrees clockwise.
pub fn rotate_90(input: &ImageData, scalars: &str) -> ImageData {
    let dims = input.dimensions();
    let (nx, ny, nz) = (dims[0], dims[1], dims[2]);
    let sp = input.spacing();
    let extent = input.extent();
    transform_image(
        input,
        scalars,
        [ny, nx, nz],
        [
            extent[2], extent[3], extent[0], extent[1], extent[4], extent[5],
        ],
        |x, y, z| y + (ny - 1 - x) * nx + z * nx * ny,
        [sp[1], sp[0], sp[2]],
    )
}

/// Rotate image 180 degrees.
pub fn rotate_180(input: &ImageData, scalars: &str) -> ImageData {
    let dims = input.dimensions();
    let (nx, ny) = (dims[0], dims[1]);
    transform_image(
        input,
        scalars,
        dims,
        input.extent(),
        |x, y, z| (nx - 1 - x) + (ny - 1 - y) * nx + z * nx * ny,
        input.spacing(),
    )
}

/// Rotate image 270 degrees clockwise (= 90 counter-clockwise).
pub fn rotate_270(input: &ImageData, scalars: &str) -> ImageData {
    let dims = input.dimensions();
    let (nx, ny, nz) = (dims[0], dims[1], dims[2]);
    let sp = input.spacing();
    let extent = input.extent();
    transform_image(
        input,
        scalars,
        [ny, nx, nz],
        [
            extent[2], extent[3], extent[0], extent[1], extent[4], extent[5],
        ],
        |x, y, z| (nx - 1 - y) + x * nx + z * nx * ny,
        [sp[1], sp[0], sp[2]],
    )
}

/// Flip image horizontally (mirror along vertical axis).
pub fn flip_horizontal(input: &ImageData, scalars: &str) -> ImageData {
    let dims = input.dimensions();
    let (nx, ny) = (dims[0], dims[1]);
    transform_image(
        input,
        scalars,
        dims,
        input.extent(),
        |x, y, z| (nx - 1 - x) + y * nx + z * nx * ny,
        input.spacing(),
    )
}

/// Flip image vertically (mirror along horizontal axis).
pub fn flip_vertical(input: &ImageData, scalars: &str) -> ImageData {
    let dims = input.dimensions();
    let (nx, ny) = (dims[0], dims[1]);
    transform_image(
        input,
        scalars,
        dims,
        input.extent(),
        |x, y, z| x + (ny - 1 - y) * nx + z * nx * ny,
        input.spacing(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_rotate_90() {
        let img = ImageData::from_function(
            [4, 3, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let r = rotate_90(&img, "v");
        assert_eq!(r.dimensions(), [3, 4, 1]);
    }
    #[test]
    fn test_rotate_180() {
        let img = ImageData::from_function(
            [4, 4, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let r = rotate_180(&img, "v");
        assert_eq!(r.dimensions(), [4, 4, 1]);
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 3.0).abs() < 1e-10); // last pixel becomes first
    }
    #[test]
    fn test_flip_h() {
        let img = ImageData::from_function(
            [4, 4, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let r = flip_horizontal(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 3.0).abs() < 1e-10);
    }
    #[test]
    fn test_flip_v() {
        let img = ImageData::from_function(
            [4, 4, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, y, _| y,
        );
        let r = flip_vertical(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 3.0).abs() < 1e-10);
    }

    #[test]
    fn rotate_90_preserves_all_slices() {
        let img = ImageData::from_function(
            [2, 2, 2],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, z| x + y * 10.0 + z * 100.0,
        );
        let r = rotate_90(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        assert_eq!(arr.num_tuples(), 8);
        assert_eq!(
            arr.to_f64_vec(),
            vec![10.0, 0.0, 11.0, 1.0, 110.0, 100.0, 111.0, 101.0]
        );
    }

    #[test]
    fn rotate_180_does_not_reverse_z_slices() {
        let img = ImageData::from_function(
            [2, 2, 2],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, z| x + y * 10.0 + z * 100.0,
        );
        let r = rotate_180(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        assert_eq!(
            arr.to_f64_vec(),
            vec![11.0, 10.0, 1.0, 0.0, 111.0, 110.0, 101.0, 100.0]
        );
    }

    #[test]
    fn rotate_reindexes_all_image_sized_arrays_and_preserves_type() {
        let mut img = ImageData::with_dimensions(2, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec(
                "selected",
                vec![1, 2],
                1,
            )));
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "other",
                vec![10.0, 20.0],
                1,
            )));

        let r = rotate_180(&img, "selected");
        let selected = r.point_data().get_array("selected").unwrap();
        assert_eq!(selected.scalar_type(), crate::types::ScalarType::I32);
        assert_eq!(selected.to_f64_vec(), vec![2.0, 1.0]);
        assert_eq!(
            r.point_data().get_array("other").unwrap().to_f64_vec(),
            vec![20.0, 10.0]
        );
    }

    #[test]
    fn rotate_preserves_active_scalars_and_transforms_extent() {
        let mut img = ImageData::with_dimensions(2, 3, 1);
        img.set_extent([10, 11, 20, 22, 5, 5]);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![0.0; 6], 1)));
        img.point_data_mut().set_active_scalars("v");

        let r = rotate_90(&img, "v");
        assert_eq!(r.extent(), [20, 22, 10, 11, 5, 5]);
        assert!(r.point_data().scalars().is_some());
    }
}
